use super::lamps;
use super::mat::{self, M4, V3};
use super::mesh::Vert;
use crate::map::Map;

const GLASS: [f32; 2] = [0.21, 0.31];
const GLASS_Y: f32 = 1.5;
const GLASS_Z: f32 = 0.036;
const LIFT: f32 = 0.012;
const SLAB: f32 = 0.02;
const MAX_POLY: usize = 9;

#[derive(Clone, Copy, Debug)]
pub struct Mirror {
    pub centre: V3,
    pub normal: V3,
    pub right: V3,
    pub room: u8,
    pub lo: V3,
    pub hi: V3,
    pub reflect: M4,
}

fn reflection(c: V3, n: V3) -> M4 {
    let k = 2.0 * mat::dot(c, n);
    std::array::from_fn(|i| match (i / 4, i % 4) {
        (3, 3) => 1.0,
        (3, r) => k * n[r],
        (_, 3) => 0.0,
        (col, row) => (col == row) as u8 as f32 - 2.0 * n[col] * n[row],
    })
}

pub fn mirrors(map: &Map) -> Vec<Mirror> {
    map.props
        .iter()
        .filter(|p| p.kind == "espelho")
        .filter_map(|p| {
            let (x, z) = (p.pos.x.to_f32(), p.pos.y.to_f32());
            let room = lamps::room_near(map, x.floor() as i32, z.floor() as i32)?;
            let xf = mat::mul(&mat::translate([x, 0.0, z]), &mat::rot_y(p.rot.to_rad()));
            let centre = mat::point(&xf, [0.0, GLASS_Y, GLASS_Z]);
            let (normal, right) = (mat::dir(&xf, [0.0, 0.0, 1.0]), mat::dir(&xf, [1.0, 0.0, 0.0]));
            let corner = |s: f32, t: f32| mat::add(centre, mat::add(mat::scale(right, s * GLASS[0]), [0.0, t * GLASS[1], 0.0]));
            let (a, b) = (corner(-1.0, -1.0), corner(1.0, 1.0));
            let lo = std::array::from_fn(|k| a[k].min(b[k]) - 0.02);
            let hi = std::array::from_fn(|k| a[k].max(b[k]) + 0.02);
            Some(Mirror { centre, normal, right, room, lo, hi, reflect: reflection(centre, normal) })
        })
        .collect()
}

#[derive(Clone, Copy, Default)]
struct P {
    v: Vert,
    st: [f32; 2],
}

fn lerp(a: &P, b: &P, t: f32) -> P {
    let f = |x: f32, y: f32| x + (y - x) * t;
    P {
        v: Vert {
            pos: std::array::from_fn(|k| f(a.v.pos[k], b.v.pos[k])),
            uv: std::array::from_fn(|k| f(a.v.uv[k] as f32, b.v.uv[k] as f32) as i16),
            nrm: std::array::from_fn(|k| f(a.v.nrm[k] as f32, b.v.nrm[k] as f32) as i8),
            tile: a.v.tile,
        },
        st: [f(a.st[0], b.st[0]), f(a.st[1], b.st[1])],
    }
}

fn clip_edge(poly: &[P], inside: impl Fn(&P) -> f32) -> ([P; MAX_POLY], usize) {
    let mut out = [P::default(); MAX_POLY];
    let mut n = 0;
    let mut put = |p: P| {
        if n < MAX_POLY {
            out[n] = p;
            n += 1;
        }
    };
    (0..poly.len()).for_each(|i| {
        let (a, b) = (&poly[i], &poly[(i + 1) % poly.len()]);
        let (da, db) = (inside(a), inside(b));
        if da >= 0.0 {
            put(*a);
        }
        if (da >= 0.0) != (db >= 0.0) {
            put(lerp(a, b, da / (da - db)));
        }
    });
    (out, n)
}

fn clip(tri: [P; 3]) -> ([P; MAX_POLY], usize) {
    let mut poly = [P::default(); MAX_POLY];
    poly[..3].copy_from_slice(&tri);
    let edges: [fn(&P) -> f32; 4] =
        [|p| GLASS[0] - p.st[0], |p| GLASS[0] + p.st[0], |p| GLASS[1] - p.st[1], |p| GLASS[1] + p.st[1]];
    edges.iter().fold((poly, 3), |(poly, n), e| if n < 3 { (poly, 0) } else { clip_edge(&poly[..n], e) })
}

impl Mirror {
    pub fn facing(&self, eye: V3) -> bool {
        mat::dot(mat::sub(eye, self.centre), self.normal) > 0.05
    }

    fn flatten(&self, eye: V3, v: &Vert) -> P {
        let behind = -mat::dot(mat::sub(v.pos, self.centre), self.normal);
        let ray = mat::sub(v.pos, eye);
        let denom = mat::dot(ray, self.normal);
        let t = if denom < -1e-4 { (mat::dot(mat::sub(self.centre, eye), self.normal) / denom).min(1.0) } else { 1.0 };
        let q = mat::add(eye, mat::scale(ray, t));
        let d = mat::sub(q, self.centre);
        let pos = mat::add(q, mat::scale(mat::norm(mat::sub(eye, q)), LIFT + SLAB * behind.max(0.0)));
        P { v: Vert { pos, ..*v }, st: [mat::dot(d, self.right), d[1]] }
    }

    pub fn emit(&self, eye: V3, verts: &[Vert], idx: &[u16], out: &mut Vec<Vert>) {
        idx.chunks_exact(3).for_each(|t| {
            let (poly, n) = clip([0, 1, 2].map(|k| self.flatten(eye, &verts[t[k] as usize])));
            (1..n.saturating_sub(1)).for_each(|i| out.extend([poly[0].v, poly[i].v, poly[i + 1].v, poly[i + 1].v]));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    fn wall_mirror() -> Mirror {
        let (c, n) = ([0.0, 1.5, 0.0], [0.0, 0.0, 1.0]);
        Mirror { centre: c, normal: n, right: [1.0, 0.0, 0.0], room: 0, lo: [0.0; 3], hi: [0.0; 3], reflect: reflection(c, n) }
    }

    #[test]
    fn reflection_flips_across_the_plane() {
        let m = wall_mirror();
        assert_eq!(mat::point(&m.reflect, [0.3, 1.0, 2.0]), [0.3, 1.0, -2.0]);
        let r = reflection([4.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(mat::point(&r, [3.0, 1.0, 5.0]), [5.0, 1.0, 5.0]);
    }

    #[test]
    fn flattened_body_keeps_screen_ray_and_sits_in_front_of_glass() {
        let m = wall_mirror();
        let eye = [0.5, 1.6, 3.0];
        let v = Vert { pos: [0.1, 1.4, -1.0], ..Vert::default() };
        let p = m.flatten(eye, &v);
        assert!(p.v.pos[2] > 0.0 && p.v.pos[2] < 0.05, "{:?}", p.v.pos);
        let (a, b) = (mat::norm(mat::sub(v.pos, eye)), mat::norm(mat::sub(p.v.pos, eye)));
        assert!(mat::dot(a, b) > 0.99999);
    }

    #[test]
    fn clipping_keeps_only_the_glass() {
        let m = wall_mirror();
        let eye = [0.0, 1.5, 3.0];
        let big = [[-1.0, 0.5, -0.5], [1.0, 0.5, -0.5], [0.0, 2.5, -0.5]].map(|pos| Vert { pos, ..Vert::default() });
        let mut out = Vec::new();
        m.emit(eye, &big, &[0, 1, 2], &mut out);
        assert!(!out.is_empty() && out.len() % 4 == 0);
        assert!(out.iter().all(|v| (v.pos[0] - m.centre[0]).abs() <= GLASS[0] + 1e-3 && (v.pos[1] - m.centre[1]).abs() <= GLASS[1] + 1e-3));
        let far = [[3.0, 0.5, -0.5], [4.0, 0.5, -0.5], [3.5, 2.0, -0.5]].map(|pos| Vert { pos, ..Vert::default() });
        out.clear();
        m.emit(eye, &far, &[0, 1, 2], &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn sobrado_has_the_banheiro_mirror_facing_the_room() {
        let m = map::sobrado();
        let all = mirrors(m);
        assert_eq!(all.len(), m.props.iter().filter(|p| p.kind == "espelho").count());
        all.iter().for_each(|r| {
            let inside = mat::add(r.centre, mat::scale(r.normal, 0.5));
            assert_eq!(lamps::room_near(m, inside[0].floor() as i32, inside[2].floor() as i32), Some(r.room));
        });
    }

    #[test]
    fn banheiro_camera_sees_an_echo_in_the_mirror_only_through_the_glass() {
        use super::super::camera::Camera;
        use super::super::mannequin::{self, Crowd};
        use crate::sim::Anim;
        let m = map::sobrado();
        let r = mirrors(m)[0];
        let eye = Camera::follow(m, 13.5, 16.5, 16.0 / 9.0).expect("banheiro cam").eye;
        let crowd = Crowd::new();
        let mesh = &crowd.get(false).mesh;
        let shown = |x: f32, z: f32| {
            let rig = mannequin::rig(x, z, std::f32::consts::FRAC_PI_2, Anim::Idle, 0, false, 0.0);
            let mut skin = vec![Vert::default(); mesh.verts.len()];
            crowd.get(false).skin(&rig.mats.map(|s| mat::mul(&r.reflect, &s)), |_| 0.5, [1; 4], &mut skin);
            let mut out = Vec::new();
            r.emit(eye, &skin, &mesh.idx, &mut out);
            out.len()
        };
        assert!(r.facing(eye));
        assert!(shown(14.9, 16.2) > 0);
        assert_eq!(shown(14.8, 13.2), 0);
    }
}
