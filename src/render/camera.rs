use super::mat::{self, Frustum, M4, V3};
use crate::map::{Cam, Map};

pub const NEAR: f32 = 0.08;
pub const FAR: f32 = 22.0;
const WIDE: f32 = 16.0 / 9.0;
const CLOSE_FOV: f32 = 30.0;
const CLOSE_REACH: f32 = 0.45;
const CLOSE_DROP: f32 = 0.32;

#[derive(Clone, Copy)]
pub struct Camera {
    pub eye: V3,
    pub view_proj: M4,
    pub frustum: Frustum,
}

pub fn fovy(fov_deg: f32, aspect: f32) -> f32 {
    let t = (fov_deg.to_radians() * 0.5).tan();
    let t = if aspect < WIDE { t * (WIDE / aspect).sqrt() } else { t };
    2.0 * t.min(1.6).atan()
}

impl Camera {
    fn look(eye: V3, target: V3, fov_deg: f32, aspect: f32) -> Camera {
        let proj = mat::perspective(fovy(fov_deg, aspect), aspect, NEAR, FAR);
        let view_proj = mat::mul(&proj, &mat::look_at(eye, target));
        Camera { eye, view_proj, frustum: Frustum::new(&view_proj) }
    }

    pub fn new(cam: &Cam, aspect: f32) -> Camera {
        let v = |p: &[crate::fx::Fx; 3]| [p[0].to_f32(), p[1].to_f32(), p[2].to_f32()];
        Camera::look(v(&cam.eye), v(&cam.target), cam.fov.to_f32(), aspect)
    }

    pub fn follow(map: &Map, x: f32, z: f32, aspect: f32) -> Option<Camera> {
        map.cam_at(x.floor() as i32, z.floor() as i32).map(|c| Camera::new(c, aspect))
    }

    pub fn close(face: V3, forward: V3, aspect: f32) -> Camera {
        let flat = mat::norm([forward[0], 0.0, forward[2]]);
        let eye = mat::add(face, [flat[0] * CLOSE_REACH, -CLOSE_DROP, flat[2] * CLOSE_REACH]);
        Camera::look(eye, mat::add(face, [0.0, 0.02, 0.0]), CLOSE_FOV, aspect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{self, Tile};

    #[test]
    fn narrow_views_widen_fov() {
        assert!((fovy(60.0, WIDE) - 60f32.to_radians()).abs() < 1e-4);
        assert!(fovy(60.0, 8.0 / 9.0) > fovy(60.0, WIDE));
    }

    #[test]
    fn every_walkable_tile_sees_its_own_floor() {
        let m = map::sobrado();
        (0..m.h).flat_map(|y| (0..m.w).map(move |x| (x, y))).filter(|&(x, y)| m.tile(x, y) == Tile::Floor).for_each(|(x, y)| {
            let c = Camera::follow(m, x as f32 + 0.5, y as f32 + 0.5, WIDE).expect("cam");
            let p = [x as f32 + 0.5, 0.9, y as f32 + 0.5];
            assert!(c.frustum.aabb(mat::sub(p, [0.6; 3]), mat::add(p, [0.6; 3])), "tile {x},{y} not in its camera");
        });
    }

    #[test]
    fn close_up_looks_up_at_the_face() {
        let face = [3.0, 1.7, 4.0];
        let c = Camera::close(face, [0.0, -0.1, 1.0], WIDE);
        assert!(c.eye[1] < face[1] && c.eye[2] > face[2]);
        assert!(c.frustum.aabb(mat::sub(face, [0.05; 3]), mat::add(face, [0.05; 3])));
        assert!(!c.frustum.aabb([3.6, 1.0, 4.0], [3.7, 1.1, 4.1]));
    }
}
