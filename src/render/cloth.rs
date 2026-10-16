use super::mat::V3;
use std::f32::consts::TAU;

pub const PANELS: [f32; 2] = [-0.72, 0.36];
pub const EDGES: usize = 5;
pub const ROWS: usize = 4;
const STRIP: f32 = 0.09;
const ROD: f32 = 2.3;
const HEM: f32 = 0.25;
const FLAP_HZ: f32 = 0.7;

pub fn quads(rows: usize, swing: impl Fn(usize, usize) -> f32, mut emit: impl FnMut([V3; 4], [[f32; 2]; 4])) {
    let drop = ROD - HEM;
    PANELS.iter().enumerate().for_each(|(panel, &x0)| {
        let at = |j: usize, k: usize| {
            let (d, a) = (drop * k as f32 / rows as f32, swing(panel, j));
            let z = if j % 2 == 0 { 0.05 } else { 0.12 };
            ([x0 + j as f32 * STRIP, ROD - d * a.cos(), z + d * a.sin()], [j as f32 * 0.2, 4.0 * k as f32 / rows as f32])
        };
        (0..EDGES - 1).flat_map(|j| (0..rows).map(move |k| (j, k))).for_each(|(j, k)| {
            let q = [at(j, k + 1), at(j + 1, k + 1), at(j + 1, k), at(j, k)];
            emit(q.map(|v| v.0), q.map(|v| v.1));
        });
    });
}

pub fn sway(t: f32, seed: f32, panel: usize, edge: usize) -> f32 {
    let gust = 0.65 + 0.35 * (t * 0.9 + seed).sin() * (t * 0.37 + seed * 2.0).sin();
    let wave = (t * TAU * FLAP_HZ - edge as f32 * 0.55 - panel as f32 * 1.3 + seed).sin();
    (0.15 + 0.11 * wave) * gust
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flapping_stays_in_front_of_the_wall_within_fifteen_degrees() {
        (0..600).map(|i| i as f32 / 60.0).for_each(|t| {
            (0..PANELS.len()).for_each(|p| {
                (0..EDGES).for_each(|e| {
                    let a = sway(t, 1.7, p, e);
                    assert!((0.0..=15f32.to_radians()).contains(&a), "{a}");
                })
            })
        });
        let mut n = 0;
        quads(ROWS, |p, e| sway(0.3, 0.0, p, e), |q, _| {
            n += 1;
            assert!(q.iter().all(|v| v[2] >= 0.05 && v[1] >= 0.2 && v[1] <= 2.3));
        });
        assert_eq!(n, PANELS.len() * (EDGES - 1) * ROWS);
    }
}
