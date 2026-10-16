use crate::fx::{Fx, Vec2};

pub fn clear(a: Vec2, b: Vec2, blocked: impl Fn(i32, i32) -> bool) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: i32, y: i32) -> Vec2 {
        Vec2::new(Fx::int(x) + Fx::HALF, Fx::int(y) + Fx::HALF)
    }

    #[test]
    fn grid_walk_stops_at_walls_only_between_ends() {
        let wall = |x: i32, y: i32| (x, y) == (3, 2);
        assert!(!clear(at(1, 2), at(5, 2), wall));
        assert!(clear(at(1, 1), at(5, 1), wall));
        assert!(clear(at(3, 2), at(5, 3), wall));
        assert!(clear(at(1, 3), at(5, 3), wall));
        assert!(!clear(at(2, 1), at(4, 3), wall));
        assert!(clear(at(2, 2), at(2, 2), wall));
    }
}
