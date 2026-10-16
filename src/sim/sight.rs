use super::{World, MAX_PLAYERS, NO_CAM};
use crate::fx::{Ang, Fx, Vec2};
use crate::map::{Cam, Tile};

const RANGE: i64 = (16i64 << 16) * (16i64 << 16);
const FEET: Fx = Fx::ratio(2, 10);
const CROWN: Fx = Fx::ratio(17, 10);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct View {
    pub eye: Vec2,
    height: Fx,
    yaw: Ang,
    half: i32,
    pitch: i32,
    vhalf: i32,
}

pub fn signed(a: Ang) -> i32 {
    a.0 as i16 as i32
}

pub fn view(c: &Cam) -> View {
    let eye = Vec2::new(c.eye[0], c.eye[2]);
    let flat = Vec2::new(c.target[0], c.target[2]) - eye;
    let half_v = Ang::deg(c.fov / 2);
    let wide = half_v.sin().div(half_v.cos()) * 16 / 9;
    View {
        eye,
        height: c.eye[1],
        yaw: flat.ang(),
        half: signed(Ang::atan2(wide, Fx::ONE)),
        pitch: signed(Ang::atan2(c.target[1] - c.eye[1], flat.len())),
        vhalf: signed(half_v),
    }
}

pub fn frames(v: &View, p: Vec2) -> bool {
    let d = p - v.eye;
    let r = d.len();
    let elev = |h: Fx| signed(Ang::atan2(h - v.height, r));
    r.0 > 0
        && d.len_sq() < RANGE
        && signed(d.ang() - v.yaw).abs() <= v.half
        && elev(CROWN) >= v.pitch - v.vhalf
        && elev(FEET) <= v.pitch + v.vhalf
}

pub fn walk(a: Vec2, b: Vec2, blocked: impl Fn(i32, i32) -> bool) -> bool {
    let one = Fx::ONE.0 as i64;
    let frac = |v: Fx| (v.0 & 0xFFFF) as i64;
    let d = b - a;
    let (sx, sy) = (d.x.0.signum(), d.y.0.signum());
    let (ax, ay) = (d.x.0.unsigned_abs() as i64, d.y.0.unsigned_abs() as i64);
    let first = |s: i32, f: i64| match s {
        1 => one - f,
        -1 => f,
        _ => 0,
    };
    let start = (a.x.floor(), a.y.floor(), first(sx, frac(a.x)), first(sy, frac(a.y)));
    let end = (b.x.floor(), b.y.floor());
    let n = (end.0 - start.0).abs() + (end.1 - start.1).abs();
    (0..n)
        .try_fold(start, |(x, y, nx, ny), _| {
            let along_x = ay == 0 || (ax != 0 && nx * ay < ny * ax);
            let next = match along_x {
                true => (x + sx, y, nx + one, ny),
                false => (x, y + sy, nx, ny + one),
            };
            match (next.0, next.1) == end || !blocked(next.0, next.1) {
                true => Some(next),
                false => None,
            }
        })
        .is_some()
}

impl World {
    fn opaque(&self, x: i32, y: i32) -> bool {
        match self.map.tile(x, y) {
            Tile::Floor | Tile::Blocker => false,
            Tile::Door => !self.door_passable(self.door_at[(y * self.map.w + x) as usize] as usize),
            _ => true,
        }
    }

    pub(super) fn clear(&self, a: Vec2, b: Vec2) -> bool {
        walk(a, b, |x, y| self.opaque(x, y))
    }

    pub(super) fn aim_cameras(&mut self) {
        self.eyes = std::array::from_fn(|p| match self.living(p) {
            true => self.tile_index(self.bodies[self.players[p].body as usize].pos).map_or(NO_CAM, |t| self.cam_of[t]),
            false => NO_CAM,
        });
    }

    pub(super) fn watcher(&self, p: Vec2, far: i64) -> Option<View> {
        (0..MAX_PLAYERS)
            .filter(|&q| self.eyes[q] != NO_CAM)
            .filter(|&q| (self.bodies[self.players[q].body as usize].pos - p).len_sq() > far)
            .map(|q| self.views[self.eyes[q] as usize])
            .find(|v| frames(v, p) && self.clear(v.eye, p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Rect;

    fn at(x: i32, y: i32) -> Vec2 {
        Vec2::new(Fx::int(x) + Fx::HALF, Fx::int(y) + Fx::HALF)
    }

    #[test]
    fn grid_walk_stops_at_walls_only_between_ends() {
        let wall = |x: i32, y: i32| (x, y) == (3, 2);
        assert!(!walk(at(1, 2), at(5, 2), wall));
        assert!(walk(at(1, 1), at(5, 1), wall));
        assert!(walk(at(3, 2), at(5, 3), wall));
        assert!(walk(at(1, 3), at(5, 3), wall));
        assert!(!walk(at(2, 1), at(4, 3), wall));
        assert!(walk(at(2, 2), at(2, 2), wall));
    }

    #[test]
    fn camera_frames_ahead_not_behind() {
        let cam = Cam {
            area: Rect { x0: 0, y0: 0, x1: 9, y1: 9 },
            eye: [Fx::int(1), Fx::ratio(24, 10), Fx::int(1)],
            target: [Fx::int(8), Fx::ratio(5, 10), Fx::int(8)],
            fov: Fx::int(60),
        };
        let v = view(&cam);
        assert!(frames(&v, at(6, 6)));
        assert!(frames(&v, at(8, 2)));
        assert!(!frames(&v, Vec2::new(Fx::ZERO, Fx::ZERO)));
        assert!(!frames(&v, at(9, 0) - Vec2::new(Fx::int(12), Fx::ZERO)));
        assert!(!frames(&v, at(0, 9)));
    }
}
