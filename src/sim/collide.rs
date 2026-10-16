use crate::fx::{isqrt, Fx, Vec2};

pub const RADIUS: Fx = Fx::ratio(3, 10);
const R2: i64 = RADIUS.0 as i64 * RADIUS.0 as i64;

fn near(p: Vec2) -> impl Iterator<Item = (i32, i32)> {
    let (x, y) = (p.x.floor(), p.y.floor());
    (y - 1..=y + 1).flat_map(move |ty| (x - 1..=x + 1).map(move |tx| (tx, ty)))
}

fn gap(p: Vec2, tx: i32, ty: i32) -> Vec2 {
    let c = Vec2::new(p.x.clamp(Fx::int(tx), Fx::int(tx + 1)), p.y.clamp(Fx::int(ty), Fx::int(ty + 1)));
    p - c
}

pub fn overlaps(p: Vec2, solid: &impl Fn(i32, i32) -> bool) -> bool {
    near(p).any(|(tx, ty)| solid(tx, ty) && gap(p, tx, ty).len_sq() < R2)
}

fn push_out(p: Vec2, solid: &impl Fn(i32, i32) -> bool) -> Vec2 {
    near(p).filter(|&(tx, ty)| solid(tx, ty)).fold(p, |p, (tx, ty)| {
        let d = gap(p, tx, ty);
        let l2 = d.len_sq();
        match l2 > 0 && l2 < R2 {
            true => {
                let l = isqrt(l2 as u64) as i64;
                let k = RADIUS.0 as i64 - l + 2;
                p + Vec2::new(Fx((d.x.0 as i64 * k / l) as i32), Fx((d.y.0 as i64 * k / l) as i32))
            }
            false => p,
        }
    })
}

pub fn slide(from: Vec2, delta: Vec2, solid: impl Fn(i32, i32) -> bool) -> Vec2 {
    let settle = |p: Vec2| std::iter::successors(Some(p), |&p| Some(push_out(p, &solid)).filter(|&q| q != p)).take(4).last().unwrap_or(p);
    let tries = [from + delta, Vec2::new(from.x + delta.x, from.y), Vec2::new(from.x, from.y + delta.y)];
    match delta == Vec2::ZERO {
        true => from,
        false => tries.into_iter().map(settle).chain([from]).find(|&p| !overlaps(p, &solid)).unwrap_or(from + delta),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::{Ang, Rng};

    fn walls(x: i32, y: i32) -> bool {
        !(1..=4).contains(&x) || !(1..=4).contains(&y) || (x, y) == (3, 2)
    }

    #[test]
    fn slides_along_wall_and_never_enters() {
        let start = Vec2::new(Fx::int(1) + Fx::HALF, Fx::int(1) + Fx::HALF);
        let p = slide(start, Vec2::new(Fx::ratio(1, 20), -Fx::ratio(1, 10)), walls);
        assert!(p.x > start.x && p.y >= Fx::int(1) + RADIUS);
        let mut rng = Rng::new(3);
        let end = (0..20000).fold(start, |p, _| {
            let d = Vec2::from_ang(Ang(rng.next_u32() as u16)).scale(Fx::ratio(26, 300));
            let q = slide(p, d, walls);
            assert!(!overlaps(q, &walls), "{q:?}");
            q
        });
        assert!(!walls(end.x.floor(), end.y.floor()));
    }
}
