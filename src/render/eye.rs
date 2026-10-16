use super::camera::Camera;
use super::level::Portal;
use super::mat::{M4, V3};

pub const ROOMS: usize = 32;
pub const FULL: [f32; 4] = [-1.0, -1.0, 1.0, 1.0];
const EMPTY: [f32; 4] = [1.0, 1.0, -1.0, -1.0];

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

fn overlaps(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] <= b[2] && b[0] <= a[2] && a[1] <= b[3] && b[1] <= a[3]
}

pub struct Eye {
    pub cam: Camera,
    pub focus: V3,
    pub reach: u32,
    pub clip: [[f32; 4]; ROOMS],
    pub detail: f32,
    pub rect: [i32; 4],
}

impl Eye {
    pub fn new<'a>(cam: Camera, focus: V3, seeds: u32, open: impl Iterator<Item = &'a Portal>, detail: f32, rect: [i32; 4]) -> Eye {
        let mut clip = [EMPTY; ROOMS];
        (0..ROOMS).filter(|r| seeds >> r & 1 != 0).for_each(|r| clip[r] = FULL);
        let reach = open.filter(|p| p.rooms & seeds != 0 && cam.frustum.aabb(p.lo, p.hi)).fold(seeds, |m, p| {
            let b = screen_box(&cam.view_proj, p.lo, p.hi);
            (0..ROOMS).filter(|r| p.rooms >> r & 1 != 0 && seeds >> r & 1 == 0).for_each(|r| clip[r] = union(clip[r], b));
            m | p.rooms
        });
        Eye { cam, focus, reach, clip, detail, rect }
    }

    pub fn sees(&self, rooms: u32, lo: V3, hi: V3) -> bool {
        let live = rooms & self.reach;
        live != 0 && self.cam.frustum.aabb(lo, hi) && {
            let b = screen_box(&self.cam.view_proj, lo, hi);
            (0..ROOMS).filter(|r| live >> r & 1 != 0).any(|r| overlaps(b, self.clip[r]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::mat::{self, Frustum};

    fn cam(eye: V3, at: V3) -> Camera {
        let vp = mat::mul(&mat::perspective(1.0, 16.0 / 9.0, 0.08, 22.0), &mat::look_at(eye, at));
        Camera { eye, view_proj: vp, frustum: Frustum::new(&vp) }
    }

    #[test]
    fn neighbour_rooms_only_show_through_their_portal() {
        let c = cam([0.0, 1.5, 0.0], [0.0, 1.0, -6.0]);
        let door = Portal { x: 0, y: 0, rooms: 0b11, lo: [-0.5, 0.0, -4.1], hi: [0.5, 2.1, -3.9] };
        let e = Eye::new(c, [0.0, 1.0, -1.0], 0b01, [&door].into_iter(), 4.0, [0, 0, 384, 216]);
        assert_eq!(e.reach, 0b11);
        assert!(e.sees(0b10, [-0.3, 0.0, -7.0], [0.3, 2.0, -6.0]));
        assert!(!e.sees(0b10, [3.0, 0.0, -7.0], [4.0, 2.0, -6.0]));
        assert!(e.sees(0b01, [3.0, 0.0, -7.0], [4.0, 2.0, -6.0]));
        assert!(!e.sees(0b100, [-0.3, 0.0, -7.0], [0.3, 2.0, -6.0]));
    }

    #[test]
    fn boxes_behind_the_eye_are_conservative() {
        let c = cam([0.0, 1.5, 0.0], [0.0, 1.0, -6.0]);
        assert_eq!(screen_box(&c.view_proj, [-1.0, 0.0, -1.0], [1.0, 2.0, 1.0]), FULL);
        let b = screen_box(&c.view_proj, [-0.1, 1.4, -5.1], [0.1, 1.6, -4.9]);
        assert!(b[2] - b[0] < 0.1 && b[0] > -0.1 && b[2] < 0.1);
    }
}
