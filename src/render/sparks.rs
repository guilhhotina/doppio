use super::mat::{self, V3};
use super::mesh::{UV_ONE, Vert};
use std::f32::consts::{PI, TAU};

pub const FLAME: [u8; 3] = [255, 150, 50];
pub const GOLD: [u8; 3] = [255, 236, 190];
pub const WARM: [u8; 3] = [255, 178, 96];
pub const FIRE: [u8; 3] = [255, 110, 30];
pub const STAR: [u8; 3] = [255, 240, 120];

fn vert(p: V3, a: f32, c: [u8; 3]) -> Vert {
    Vert { pos: p, uv: [(a * UV_ONE).clamp(0.0, 32767.0) as i16, UV_ONE as i16], nrm: [0; 4], tile: [c[0], c[1], c[2], 255], lit: [0; 4] }
}

pub fn quad(out: &mut Vec<Vert>, p: [V3; 4], a: [f32; 4], c: [u8; 3]) {
    out.extend((0..4).map(|i| vert(p[i], a[i], c)));
}

fn tri(out: &mut Vec<Vert>, p: [V3; 3], a: [f32; 3], c: [u8; 3]) {
    quad(out, [p[0], p[1], p[2], p[2]], [a[0], a[1], a[2], a[2]], c);
}

fn blob(out: &mut Vec<Vert>, c: V3, r: V3, u: V3, a: f32, col: [u8; 3], sides: usize) {
    let rim = |i: usize| {
        let (s, k) = (i as f32 / sides as f32 * TAU).sin_cos();
        mat::add(c, mat::add(mat::scale(r, k), mat::scale(u, s)))
    };
    (0..sides).for_each(|i| tri(out, [c, rim(i), rim(i + 1)], [a, 0.0, 0.0], col));
}

pub fn facing(p: V3, eye: V3) -> (V3, V3) {
    let f = mat::norm(mat::sub(p, eye));
    let r = mat::norm(mat::cross(f, [0.0, 1.0, 0.0]));
    (r, mat::cross(r, f))
}

pub fn spark(out: &mut Vec<Vert>, p: V3, eye: V3, size: f32, a: f32, c: [u8; 3]) {
    let to = mat::sub(eye, p);
    let k = (size / mat::len(to).max(size * 2.0)).min(0.5);
    let (q, s) = (mat::add(p, mat::scale(to, k)), size * (1.0 - k));
    let (r, u) = facing(q, eye);
    blob(out, q, mat::scale(r, s), mat::scale(u, s), a, c, 6);
}

pub fn flame(out: &mut Vec<Vert>, p: V3, eye: V3, t: f32, seed: f32, size: f32) {
    let (r, _) = facing(p, eye);
    (0..3).for_each(|i| {
        let a = i as f32 / 3.0 * TAU + seed;
        let base = mat::add(p, [a.sin() * 0.012 * size, 0.0, a.cos() * 0.012 * size]);
        let h = size * (0.05 + 0.035 * ((t * 19.0 + i as f32 * 1.7 + seed).sin() * 0.5 + 0.5));
        let w = mat::scale(r, 0.014 * size);
        tri(out, [mat::sub(base, w), mat::add(base, w), mat::add(base, [0.0, h, 0.0])], [1.6, 1.6, 0.0], FLAME);
    });
    spark(out, mat::add(p, [0.0, 0.03 * size, 0.0]), eye, 0.12 * size, 0.28, FLAME);
}

pub fn glint(out: &mut Vec<Vert>, p: V3, eye: V3, t: f32, seed: f32, k: f32) {
    const PERIOD: f32 = 1.6;
    const FLASH: f32 = 0.3;
    let ph = (t / PERIOD + seed).fract();
    let f = if ph < FLASH { (PI * ph / FLASH).sin() * k } else { 0.0 };
    spark(out, p, eye, 0.035, 0.9 * k, GOLD);
    if f <= 0.0 {
        return;
    }
    let q = mat::add(p, mat::scale(mat::norm(mat::sub(eye, p)), 0.06));
    let (r, u) = facing(q, eye);
    blob(out, q, mat::scale(r, 0.2 * f), mat::scale(u, 0.014), 1.8 * f, GOLD, 4);
    blob(out, q, mat::scale(r, 0.014), mat::scale(u, 0.2 * f), 1.8 * f, GOLD, 4);
    spark(out, p, eye, 0.06, 1.3 * f, GOLD);
}

pub fn halo(out: &mut Vec<Vert>, p: V3, eye: V3, size: f32, level: f32, c: [u8; 3]) {
    spark(out, p, eye, size, 0.32 * level, c);
    spark(out, p, eye, size * 0.4, 0.45 * level, c);
}

pub fn fire(out: &mut Vec<Vert>, p: V3, dir: V3, size: f32, eye: V3, t: f32) {
    (0..5).for_each(|i| {
        let ph = (t * 2.6 + i as f32 * 0.21).fract();
        let c = mat::add(p, mat::add(mat::scale(dir, ph * size * 1.6), [0.0, ph * size * 0.5, 0.0]));
        let s = size * (0.25 + 0.5 * ph) * (1.0 - ph * 0.5);
        spark(out, c, eye, s, 0.9 * (1.0 - ph), if i % 2 == 0 { FIRE } else { FLAME });
    });
}

pub fn stars(out: &mut Vec<Vert>, head: V3, eye: V3, t: f32) {
    (0..3).for_each(|i| {
        let a = t * 4.0 + i as f32 * TAU / 3.0;
        spark(out, mat::add(head, [a.sin() * 0.25, 0.22 + 0.04 * (a * 2.0).sin(), a.cos() * 0.25]), eye, 0.05, 1.4, STAR);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprites_emit_whole_quads_and_glints_blink() {
        let eye = [0.0, 1.6, -3.0];
        let mut out = Vec::new();
        flame(&mut out, [0.0, 0.9, 0.0], eye, 0.3, 0.0, 1.0);
        assert!(!out.is_empty() && out.len() % 4 == 0);
        let n = |t: f32| {
            let mut o = Vec::new();
            glint(&mut o, [0.0, 0.1, 0.0], eye, t, 0.0, 1.0);
            o.len()
        };
        assert!(n(0.2) > n(1.0) && n(1.0) > 0);
        out.clear();
        fire(&mut out, [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], 0.5, eye, 0.1);
        assert!(out.iter().all(|v| v.tile[3] == 255));
    }
}
