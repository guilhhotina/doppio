use super::{MarkKind, Sink};
use crate::render::mat;
use crate::render::mesh::Faces;
use std::f32::consts::TAU;

const SIDES: Faces = Faces::NO_BOTTOM;

pub fn build(kind: &str, s: &mut Sink) -> bool {
    match kind {
        "arvore" => arvore(s),
        "bananeira" => {
            s.pen("casca").cylinder([0.0, 0.0, 0.0], 0.11, 0.08, 1.5, 5, false);
            s.cross("bananeira_a", [0.0, 0.65, 0.0], 0.95, 2.0, 2, 0.3);
        }
        "bambu" => {
            [-0.12f32, 0.1, 0.02].iter().enumerate().for_each(|(i, &x)| {
                let z = [-0.08f32, 0.06, 0.12][i];
                s.pen("madeira_crua").cylinder([x, 0.0, z], 0.04, 0.035, 2.5, 4, false);
            });
            s.cross("bambu_a", [0.0, 0.9, 0.0], 0.9, 1.8, 2, 0.2);
        }
        "cerca_madeira" => cerca(s),
        "tronco" => tronco(s),
        "pedra" => pedra(s),
        "canoa" => canoa(s),
        "forno_farinha" => forno(s),
        "poco" => poco(s),
        _ => return false,
    }
    true
}

fn arvore(s: &mut Sink) {
    let mut b = s.pen("casca");
    b.cylinder([0.0, 0.0, 0.0], 0.4, 0.23, 0.85, 6, false);
    b.cylinder([0.0, 0.85, 0.0], 0.23, 0.15, 1.65, 5, false);
    (0..4).for_each(|i| {
        let a = i as f32 * 0.25 * TAU + 0.4;
        let (sn, cs) = a.sin_cos();
        b.tri([[sn * 0.58, 0.0, cs * 0.58], [-cs * 0.1, 0.0, sn * 0.1], [sn * 0.18, 0.9, cs * 0.18]], [[0.0, 1.0], [0.5, 1.0], [0.25, 0.0]]);
        b.tri([[cs * 0.1, 0.0, -sn * 0.1], [sn * 0.58, 0.0, cs * 0.58], [sn * 0.18, 0.9, cs * 0.18]], [[0.0, 1.0], [0.5, 1.0], [0.25, 0.0]]);
    });
    [(0.52f32, 0.0f32, 2.05f32), (-0.48, 0.35, 2.15), (0.15, -0.5, 2.2)].iter().for_each(|&(rz, rx, y)| {
        let local = mat::mul(&mat::translate([0.0, y, 0.0]), &mat::mul(&mat::rot_x(rx), &mat::rot_z(rz)));
        s.pen_at("casca", &local).cylinder([0.0, 0.0, 0.0], 0.08, 0.04, 0.95, 4, false);
    });
    s.cross("folhas_a", [0.0, 2.4, 0.0], 1.25, 1.05, 2, 0.3);
    let mut c = s.pen("folhas_a");
    [(2.45f32, 3.05f32, 1.15f32), (2.85, 3.35, 0.82)].iter().for_each(|&(y0, y1, r)| {
        (0..6).for_each(|i| {
            let (a0, a1) = (i as f32 / 6.0 * TAU, (i + 1) as f32 / 6.0 * TAU);
            let (p0, p1, top) = ([a0.sin() * r, y0, a0.cos() * r], [a1.sin() * r, y0, a1.cos() * r], [0.0, y1, 0.0]);
            c.tri([top, p1, p0], [[0.5, 0.0], [1.0, 1.0], [0.0, 1.0]]);
            c.tri([top, p0, p1], [[0.5, 0.0], [0.0, 1.0], [1.0, 1.0]]);
        });
    });
}

fn cerca(s: &mut Sink) {
    let mut w = s.pen("madeira_crua");
    [-0.48f32, 0.0].iter().for_each(|&x| w.cuboid([x, 0.0, -0.05], [x + 0.09, 1.2, 0.05], Faces(0x37)));
    [0.35f32, 0.85].iter().for_each(|&y| w.cuboid([-0.5, y, 0.05], [0.5, y + 0.12, 0.08], Faces(0x3d)));
}

fn tronco(s: &mut Sink) {
    let local = mat::mul(&mat::translate([0.0, 0.25, 0.0]), &mat::rot_x(std::f32::consts::FRAC_PI_2));
    s.pen_at("casca", &local).cylinder([0.0, -0.9, 0.0], 0.25, 0.22, 1.8, 6, true);
    let local = mat::mul(&mat::translate([0.0, 0.25, 0.0]), &mat::rot_x(-std::f32::consts::FRAC_PI_2));
    s.pen_at("madeira_crua", &local).cylinder([0.0, 0.9, 0.0], 0.25, 0.25, 0.0, 6, true);
}

fn pedra(s: &mut Sink) {
    let mut r = s.pen("rocha");
    let top = [0.08, 0.75, -0.05];
    let base = |i: usize, y: f32, k: f32| {
        let a = i as f32 / 6.0 * TAU + 0.3;
        let w = [0.55, 0.42, 0.6, 0.38, 0.5, 0.45][i % 6] * k;
        [a.sin() * w, y, a.cos() * w * 0.8]
    };
    (0..6).for_each(|i| {
        r.quad([base(i, 0.0, 1.0), base(i + 1, 0.0, 1.0), base(i + 1, 0.45, 0.8), base(i, 0.45, 0.8)], [[0.0, 0.5], [0.5, 0.5], [0.5, 0.0], [0.0, 0.0]]);
        r.tri([base(i, 0.45, 0.8), base(i + 1, 0.45, 0.8), top], [[0.0, 0.5], [0.5, 0.5], [0.25, 0.0]]);
    });
}

fn canoa(s: &mut Sink) {
    let mut w = s.pen("madeira_ponte");
    let hull = |t: f32, side: f32, y: f32| {
        let z = -1.8 + 3.6 * t;
        let width = 0.42 * (1.0 - (2.0 * t - 1.0).powi(2)).sqrt();
        [side * width * if y > 0.1 { 1.0 } else { 0.55 }, y, z]
    };
    (0..6).for_each(|i| {
        let (a, b) = (i as f32 / 6.0, (i + 1) as f32 / 6.0);
        [-1.0f32, 1.0].iter().for_each(|&side| {
            let q = [hull(a, side, 0.05), hull(b, side, 0.05), hull(b, side, 0.4), hull(a, side, 0.4)];
            let uv = [[a * 3.0, 0.4], [b * 3.0, 0.4], [b * 3.0, 0.0], [a * 3.0, 0.0]];
            match side > 0.0 {
                true => w.quad([q[1], q[0], q[3], q[2]], [uv[1], uv[0], uv[3], uv[2]]),
                false => w.quad(q, uv),
            }
            match side > 0.0 {
                true => w.quad(q, uv),
                false => w.quad([q[1], q[0], q[3], q[2]], [uv[1], uv[0], uv[3], uv[2]]),
            }
        });
        w.quad([hull(a, 1.0, 0.05), hull(b, 1.0, 0.05), hull(b, -1.0, 0.05), hull(a, -1.0, 0.05)], [[0.0, 0.0], [0.0, 0.6], [0.8, 0.6], [0.8, 0.0]]);
    });
    w.cuboid([-0.36, 0.25, -0.05], [0.36, 0.3, 0.15], Faces::ALL);
}

fn forno(s: &mut Sink) {
    s.pen("tijolo").cuboid([-1.0, 0.0, -0.8], [1.0, 0.75, 0.8], SIDES);
    s.pen("ferrugem").cylinder([0.0, 0.75, 0.0], 0.75, 0.72, 0.06, 10, true);
    s.pen("concreto").panel([-0.35, 0.12, 0.0], [0.35, 0.5, 0.802]);
    s.pen("madeira_crua").cuboid([-0.05, 0.81, -0.6], [0.05, 0.85, 0.6], Faces::NO_BOTTOM);
    s.mark(MarkKind::Flame, [0.0, 0.18, 0.75]);
}

fn poco(s: &mut Sink) {
    let mut r = s.pen("rocha");
    r.cylinder([0.0, 0.0, 0.0], 0.62, 0.6, 0.8, 8, false);
    let ring = |i: usize, rad: f32, y: f32| {
        let a = i as f32 / 8.0 * TAU;
        [a.sin() * rad, y, a.cos() * rad]
    };
    (0..8).for_each(|i| r.quad([ring(i + 1, 0.46, 0.2), ring(i, 0.46, 0.2), ring(i, 0.46, 0.8), ring(i + 1, 0.46, 0.8)], [[0.4, 0.5], [0.0, 0.5], [0.0, 0.0], [0.4, 0.0]]));
    (0..8).for_each(|i| r.quad([ring(i, 0.46, 0.8), ring(i, 0.6, 0.8), ring(i + 1, 0.6, 0.8), ring(i + 1, 0.46, 0.8)], [[0.0, 0.1], [0.0, 0.0], [0.4, 0.0], [0.4, 0.1]]));
    s.pen("agua").cylinder([0.0, 0.0, 0.0], 0.46, 0.46, 0.2, 8, true);
    let mut w = s.pen("madeira_escura");
    [-0.62f32, 0.54].iter().for_each(|&x| w.cuboid([x, 0.8, -0.04], [x + 0.08, 1.9, 0.04], Faces(0x37)));
    w.cylinder([-0.55, 1.6, 0.0], 0.05, 0.05, 0.0, 5, false);
    let local = mat::mul(&mat::translate([-0.6, 1.6, 0.0]), &mat::rot_z(-std::f32::consts::FRAC_PI_2));
    s.pen_at("madeira_escura", &local).cylinder([0.0, 0.0, 0.0], 0.05, 0.05, 1.2, 5, false);
    let mut t = s.pen("telha");
    t.quad([[-0.8, 1.9, -0.55], [0.8, 1.9, -0.55], [0.8, 2.25, 0.0], [-0.8, 2.25, 0.0]], [[0.0, 0.6], [1.6, 0.6], [1.6, 0.0], [0.0, 0.0]]);
    t.quad([[0.8, 1.9, 0.55], [-0.8, 1.9, 0.55], [-0.8, 2.25, 0.0], [0.8, 2.25, 0.0]], [[0.0, 0.6], [1.6, 0.6], [1.6, 0.0], [0.0, 0.0]]);
    s.pen("ferrugem").cylinder([0.0, 0.85, 0.0], 0.12, 0.14, 0.22, 6, false);
}
