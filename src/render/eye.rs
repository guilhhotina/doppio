use super::camera::Camera;
use super::mat::{M4, V3};
use super::zones::{Mask, Portal, ROOMS, Zones};

pub const FULL: [f32; 4] = [-1.0, -1.0, 1.0, 1.0];
const EMPTY: [f32; 4] = [1.0, 1.0, -1.0, -1.0];
const VISITS: u8 = 3;
const QUEUE: usize = 512;

pub fn screen_box(vp: &M4, lo: V3, hi: V3) -> [f32; 4] {
    (0..8)
        .map(|i| [[lo[0], hi[0]][i & 1], [lo[1], hi[1]][i >> 1 & 1], [lo[2], hi[2]][i >> 2 & 1]])
        .map(|p| {
            let c = |r: usize| vp[r] * p[0] + vp[4 + r] * p[1] + vp[8 + r] * p[2] + vp[12 + r];
            (c(0), c(1), c(3))
        })
        .try_fold(EMPTY, |b, (x, y, w)| (w > 1e-3).then(|| [b[0].min(x / w), b[1].min(y / w), b[2].max(x / w), b[3].max(y / w)]))
        .map_or(FULL, |b| [b[0].max(-1.0), b[1].max(-1.0), b[2].min(1.0), b[3].min(1.0)])
}

fn union(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0].min(b[0]), a[1].min(b[1]), a[2].max(b[2]), a[3].max(b[3])]
}

fn intersect(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0].max(b[0]), a[1].max(b[1]), a[2].min(b[2]), a[3].min(b[3])]
}

fn empty(a: [f32; 4]) -> bool {
    a[0] > a[2] || a[1] > a[3]
}

fn overlaps(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] <= b[2] && b[0] <= a[2] && a[1] <= b[3] && b[1] <= a[3]
}

fn near(p: V3, lo: V3, hi: V3) -> f32 {
    let d: [f32; 3] = std::array::from_fn(|k| (lo[k] - p[k]).max(p[k] - hi[k]).max(0.0));
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

pub fn bits(m: Mask) -> impl Iterator<Item = usize> {
    std::iter::successors((m != 0).then_some(m), |&m| Some(m & (m - 1)).filter(|&m| m != 0)).map(|m| m.trailing_zeros() as usize)
}

pub struct Eye {
    pub cam: Camera,
    pub focus: V3,
    pub reach: Mask,
    pub clip: [[f32; 4]; ROOMS],
    pub detail: f32,
    pub rect: [i32; 4],
    pub far: f32,
}

impl Eye {
    pub fn new(cam: Camera, focus: V3, seeds: Mask, zones: &Zones, open: impl Fn(&Portal) -> bool, allowed: Mask, detail: f32, rect: [i32; 4]) -> Eye {
        let seeds = seeds & allowed;
        let mut clip = [EMPTY; ROOMS];
        let mut queue = [0u8; QUEUE];
        let (mut head, mut tail) = (0usize, 0usize);
        bits(seeds).for_each(|r| {
            clip[r] = FULL;
            queue[tail % QUEUE] = r as u8;
            tail += 1;
        });
        let mut visits = [0u8; ROOMS];
        let mut reach = seeds;
        while head < tail {
            let r = queue[head % QUEUE] as usize;
            head += 1;
            if visits[r] >= VISITS {
                continue;
            }
            visits[r] += 1;
            zones.links[r].iter().map(|&p| &zones.portals[p as usize]).filter(|p| p.rooms & allowed != 0 && open(p) && cam.frustum.aabb(p.lo, p.hi)).for_each(|p| {
                let b = intersect(screen_box(&cam.view_proj, p.lo, p.hi), clip[r]);
                if empty(b) {
                    return;
                }
                bits(p.rooms & allowed & !(1 << r)).for_each(|s| {
                    let u = union(clip[s], b);
                    if u != clip[s] && tail - head < QUEUE {
                        clip[s] = u;
                        reach |= 1 << s;
                        queue[tail % QUEUE] = s as u8;
                        tail += 1;
                    }
                });
            });
        }
        Eye { cam, focus, reach, clip, detail, rect, far: f32::MAX }
    }

    pub fn sees(&self, rooms: Mask, lo: V3, hi: V3) -> bool {
        let live = rooms & self.reach;
        live != 0 && near(self.cam.eye, lo, hi) < self.far && self.cam.frustum.aabb(lo, hi) && {
            let b = screen_box(&self.cam.view_proj, lo, hi);
            bits(live).any(|r| overlaps(b, self.clip[r]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::mat::{self, Frustum};
    use super::*;
    use crate::map;

    fn cam(eye: V3, at: V3) -> Camera {
        let vp = mat::mul(&mat::perspective(1.0, 16.0 / 9.0, 0.08, 22.0), &mat::look_at(eye, at));
        Camera { eye, view_proj: vp, frustum: Frustum::new(&vp) }
    }

    #[test]
    fn bits_iterates_set_bits() {
        assert_eq!(bits(0).count(), 0);
        assert_eq!(bits(1 << 100 | 1 << 3 | 1).collect::<Vec<_>>(), vec![0, 3, 100]);
    }

    #[test]
    fn rooms_chain_through_open_portals_but_not_closed_doors() {
        let m = map::parse(include_str!("testdata/rua.map")).expect("rua");
        let z = Zones::new(&m);
        let street = z.room(10, 12).expect("street") as usize;
        let praca = z.room(10, 18).expect("praca") as usize;
        let ponte = z.room(17, 23).expect("ponte") as usize;
        let house = z.room(5, 5).expect("house") as usize;
        let c = cam([10.5, 3.0, 9.5], [17.5, 0.5, 23.0]);
        let all = z.area_mask(0);
        let e = Eye::new(c, [10.5, 1.0, 10.5], 1 << street, &z, |p| p.door.is_none(), all, 4.0, [0, 0, 384, 216]);
        assert!(e.reach >> praca & 1 != 0);
        assert!(e.reach >> ponte & 1 != 0, "two hops");
        assert!(e.reach >> house & 1 == 0, "closed door");
        let other_area = Eye::new(c, [10.5, 1.0, 10.5], 1 << street, &z, |_| true, 1 << street, 4.0, [0, 0, 384, 216]);
        assert_eq!(other_area.reach, 1 << street);
        let mut fogged = Eye::new(c, [10.5, 1.0, 10.5], 1 << street, &z, |_| true, all, 4.0, [0, 0, 384, 216]);
        assert!(fogged.sees(1 << street, [12.0, 0.0, 12.0], [13.0, 1.0, 13.0]));
        fogged.far = 3.0;
        assert!(!fogged.sees(1 << street, [12.0, 0.0, 16.0], [13.0, 1.0, 17.0]));
    }

    #[test]
    fn boxes_behind_the_eye_are_conservative() {
        let c = cam([0.0, 1.5, 0.0], [0.0, 1.0, -6.0]);
        assert_eq!(screen_box(&c.view_proj, [-1.0, 0.0, -1.0], [1.0, 2.0, 1.0]), FULL);
        let b = screen_box(&c.view_proj, [-0.1, 1.4, -5.1], [0.1, 1.6, -4.9]);
        assert!(b[2] - b[0] < 0.1 && b[0] > -0.1 && b[2] < 0.1);
    }
}
