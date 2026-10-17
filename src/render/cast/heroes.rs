use super::super::mat::{self, V3};
use super::kit::{self, Kit, Ring, Uv, egg, ring, swatch};
use super::skeleton::Frame;
use super::{ARM_L, ARM_R, FOOT_L, FOOT_R, FORE_L, FORE_R, HEAD, LAMP, PELVIS, SHIN_L, SHIN_R, SKIRT, THIGH_L, THIGH_R, TORSO, X};
use std::f32::consts::FRAC_PI_2;

pub type Mat = (&'static str, Uv);

const BIG: f32 = 1.14;
const BIG_HEAD: [f32; 16] = [BIG, 0.0, 0.0, 0.0, 0.0, BIG, 0.0, 0.0, 0.0, 0.0, BIG, 0.0, 0.0, 0.0, 0.0, 1.0];

pub const BENEDITA: Frame = Frame {
    hip_x: 0.08,
    waist: 0.06,
    chest: 0.34,
    shoulder_x: 0.146,
    neck: 0.375,
    upper: 0.25,
    fore: 0.22,
    thigh: 0.38,
    shin: 0.37,
    meta: 0.0,
    sole: 0.06,
    hem: 0.56,
    back: 0.2,
    x_at: (HEAD, [0.0, 0.16 * BIG, -0.095 * BIG]),
};

pub const TONHO: Frame = Frame {
    hip_x: 0.07,
    waist: 0.05,
    chest: 0.3,
    shoulder_x: 0.15,
    neck: 0.332,
    upper: 0.22,
    fore: 0.19,
    thigh: 0.34,
    shin: 0.33,
    meta: 0.0,
    sole: 0.05,
    hem: 0.27,
    back: 0.14,
    x_at: (TORSO, [0.0, 0.3, 0.1]),
};

pub const JAIRO: Frame = Frame {
    hip_x: 0.11,
    waist: 0.07,
    chest: 0.43,
    shoulder_x: 0.188,
    neck: 0.468,
    upper: 0.3,
    fore: 0.27,
    thigh: 0.46,
    shin: 0.45,
    meta: 0.0,
    sole: 0.07,
    hem: 0.0,
    back: 0.15,
    x_at: (TORSO, [0.0, 0.45, 0.1]),
};

pub const BIA: Frame = Frame {
    hip_x: 0.082,
    waist: 0.06,
    chest: 0.38,
    shoulder_x: 0.144,
    neck: 0.415,
    upper: 0.27,
    fore: 0.24,
    thigh: 0.42,
    shin: 0.41,
    meta: 0.0,
    sole: 0.06,
    hem: 0.0,
    back: 0.34,
    x_at: (HEAD, [0.0, 0.16 * BIG, -0.1 * BIG]),
};
pub fn frame(c: u8) -> &'static Frame {
    [&BENEDITA, &TONHO, &JAIRO, &BIA][c as usize % 4]
}

pub fn build(c: u8, k: &mut Kit) {
    match c {
        0 => benedita(k),
        1 => tonho(k),
        2 => jairo(k),
        _ => bia(k),
    }
}

pub fn sw(i: u8) -> Mat {
    ("cast_misc", swatch(i))
}

pub fn tiled(name: &'static str, d: f32) -> Mat {
    (name, Uv::Tile(d))
}

pub fn loft(k: &mut Kit, rings: &[Ring], sides: usize, m: Mat, gap: (usize, usize), cap: bool) {
    let rings = k.lod(rings);
    k.tex(m.0).loft(&rings, sides, m.1, gap, cap);
}

pub fn tube(k: &mut Kit, y0: f32, y1: f32, r0: f32, r1: f32, sides: usize, m: Mat) {
    loft(k, &[ring(y0, r0, r0 * 0.9, 0.0), ring(y1, r1, r1 * 0.9, 0.0)], sides, m, (0, 0), false);
}

pub fn limb(k: &mut Kit, len: f32, r0: f32, r1: f32, sides: usize, m: Mat) {
    tube(k, 0.0, -len, r0, r1, sides, m);
}

pub fn cube(k: &mut Kit, lo: V3, hi: V3, m: Mat) {
    k.tex(m.0).cuboid(lo, hi, m.1);
}

pub fn card(k: &mut Kit, p: [V3; 4], m: Mat, both: bool) {
    k.tex(m.0).card(p, m.1, both);
}

pub fn head(k: &mut Kit, tex: &str, bottom: f32, top: f32, rx: f32, rz: f32, dz: f32) {
    let h = top - bottom;
    let profile: [(f32, f32, f32, f32, f32); 6] = [
        (0.0, 0.0, 0.0, 0.0, 0.0),
        (0.095, 0.59, 0.61, 0.006, 0.095),
        (0.345, 0.95, 0.96, 0.01, 0.345),
        (0.655, 0.95, 0.96, 0.01, 0.655),
        (0.905, 0.59, 0.62, 0.006, 0.905),
        (1.0, 0.36, 0.4, 0.0, 0.95),
    ];
    let rows: Vec<(f32, f32, f32, f32, f32)> = match k.hi {
        true => profile.to_vec(),
        false => vec![profile[0], profile[2], profile[3], profile[5]],
    };
    let sides = k.n(8, 4);
    let pt = |r: &(f32, f32, f32, f32, f32), j: usize| -> (V3, [f32; 2]) {
        let s = j as f32 / sides as f32;
        let a = -std::f32::consts::PI + std::f32::consts::TAU * s;
        let (sn, cs) = a.sin_cos();
        let d = (j as isize - (sides / 2) as isize).unsigned_abs();
        let (x, z, u) = match (sides == 8, d) {
            (true, 0) => (0.0, rz * r.2 + dz + r.3, 0.5),
            (true, 1) => (sn.signum() * rx * r.1 * 0.72, rz * r.2 * 0.86 + dz + r.3, 0.5 + sn.signum() * 0.125),
            (true, 2) => (sn.signum() * rx * r.1, rz * r.2 * 0.12 + dz + r.3 * 0.3, 0.5 + sn.signum() * 0.24),
            _ => (sn * rx * r.1, cs * rz * r.2 + dz, 0.0005 + s * 0.999),
        };
        let y = top - r.0 * h;
        let v = if cs < 0.05 && r.4 > 0.68 { 0.62 } else { r.4 };
        ([x, y, z], [u.clamp(0.001, 0.999), v.clamp(0.001, 0.96)])
    };
    k.tex(tex);
    (0..rows.len() - 1).for_each(|i| {
        (0..sides).for_each(|j| {
            let quad = [(i + 1, j), (i + 1, j + 1), (i, j + 1), (i, j)];
            let (p, t): (Vec<V3>, Vec<[f32; 2]>) = quad.iter().map(|&(r, c)| pt(&rows[r], c)).unzip();
            let keep: Vec<usize> = (0..4).filter(|&idx| mat::len(mat::sub(p[idx], p[(idx + 3) % 4])) > 1e-6).collect();
            if keep.len() >= 3 {
                let p: Vec<V3> = keep.iter().map(|&idx| p[idx]).collect();
                let t: Vec<[f32; 2]> = keep.iter().map(|&idx| t[idx]).collect();
                k.poly(&p, &t);
            }
        });
    });
}

pub fn hand(k: &mut Kit, len: f32, w: f32, tone: u8) {
    if k.hi {
        let m = sw(tone);
        loft(
            k,
            &[
                ring(-len + 0.01, w * 0.38, w * 0.22, 0.0),
                ring(-len - 0.045, w * 0.48, w * 0.22, w * 0.05),
                ring(-len - 0.092, w * 0.38, w * 0.15, -w * 0.12),
            ],
            6,
            m,
            (0, 0),
            true,
        );
        spike(k, [w * 0.34, -len - 0.015, w * 0.08], [w * 0.72, -len - 0.068, w * 0.48], 0.016, m);
    }
}

pub fn spike(k: &mut Kit, base: V3, tip: V3, r: f32, m: Mat) {
    let d = mat::norm(mat::sub(tip, base));
    let side = mat::norm(mat::cross(d, if d[1].abs() > 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] }));
    let up = mat::cross(side, d);
    let rim: Vec<V3> = (0..3).map(|i| {
        let (s, c) = (i as f32 * std::f32::consts::TAU / 3.0).sin_cos();
        mat::add(base, mat::add(mat::scale(side, c * r), mat::scale(up, s * r)))
    }).collect();
    k.tex(m.0);
    let uv = match m.1 {
        Uv::Flat(c) => [c; 3],
        _ => [[0.3, 0.0], [0.7, 0.0], [0.5, 1.0]],
    };
    (0..3).for_each(|i| k.poly(&[rim[i], rim[(i + 1) % 3], tip], &uv));
}

pub fn fingers(k: &mut Kit, y: f32, len: f32, width: f32, curl: f32, r: f32, m: Mat) {
    if k.hi {
        (0..4).for_each(|i| {
            let x = (i as f32 / 3.0 - 0.5) * width;
            let l = len * [0.85, 1.0, 0.95, 0.8][i];
            spike(k, [x, y, 0.0], [x * 1.4, y - l, curl * l], r, m);
        });
        spike(k, [width * 0.5, y + 0.02, r * 2.0], [width * 0.7, y - len * 0.45, r * 2.0 + curl * len], r, m);
    }
}

pub fn shoe(k: &mut Kit, f: &Frame, w: f32, h: f32, toe: f32, tone: u8) {
    let y = -f.sole;
    match k.hi {
        true => {
            let sole = if tone == kit::WHITE { kit::GREY } else { kit::HOOF };
            cube(k, [-w * 1.02, y, -0.058], [w * 1.02, y + h * 0.28, toe * 1.02], sw(sole));
            loft(
                k,
                &[
                    ring(y + h * 0.26, w * 0.98, (toe + 0.058) * 0.52, (toe - 0.058) * 0.5),
                    ring(y + h * 0.68, w * 0.9, (toe + 0.052) * 0.44, (toe - 0.07) * 0.38),
                    ring(y + h, w * 0.76, 0.048, -0.01),
                ],
                4,
                sw(tone),
                (0, 0),
                true,
            );
        }
        false => cube(k, [-w, y, -0.055], [w, y + h, toe], sw(tone)),
    }
}

pub fn neck_textured(k: &mut Kit, tex: &'static str, h: f32, r: f32) {
    if k.hi {
        let m = (tex, Uv::Wrap([0.45, 0.3, 0.55, 0.35]));
        loft(
            k,
            &[
                ring(-0.025, r * 1.15, r * 1.1, -0.004),
                ring(h * 0.5, r, r * 0.96, 0.0),
                ring(h + 0.015, r * 0.98, r * 0.95, 0.002),
            ],
            6,
            m,
            (0, 0),
            false,
        );
    }
}

pub fn lantern(k: &mut Kit) {
    k.seg(LAMP);
    k.at(mat::mul(&mat::translate([0.0, 0.0, -0.04]), &mat::rot_x(FRAC_PI_2)));
    match k.hi {
        true => {
            tube(k, 0.0, 0.13, 0.022, 0.024, 6, sw(kit::GREY));
            k.glow(0.9);
            loft(k, &[ring(0.13, 0.026, 0.026, 0.0), ring(0.17, 0.04, 0.04, 0.0)], 6, sw(kit::BRASS), (0, 0), true);
            k.glow(0.0);
        }
        false => loft(k, &[ring(0.0, 0.025, 0.025, 0.0), ring(0.17, 0.04, 0.04, 0.0)], 3, sw(kit::BRASS), (0, 0), true),
    }
}

fn sculpted_arms(k: &mut Kit, f: &Frame, cap_mat: Mat, sleeve: (Mat, f32, f32, f32), cuff_mat: Mat, fore: (Mat, f32, f32), wrist_cuff: Option<Mat>, tone: u8, palm: f32) {
    let s = k.n(6, 3);
    let (s_mat, r0, r1, s_len) = sleeve;
    [ARM_R, ARM_L].iter().for_each(|&seg| {
        k.seg(seg);
        match k.hi {
            true => {
                loft(
                    k,
                    &[
                        ring(0.032, r0 * 0.62, r0 * 0.62, -0.004),
                        ring(-0.018, r0 * 1.05, r0 * 1.0, 0.0),
                        ring(-s_len * 0.55, r0 * 0.98, r0 * 0.95, 0.0),
                        ring(-s_len, r1 * 1.08, r1 * 1.04, 0.0),
                    ],
                    s,
                    cap_mat,
                    (0, 0),
                    true,
                );
                loft(k, &[ring(-s_len + 0.006, r1 * 1.14, r1 * 1.1, 0.0), ring(-s_len - 0.024, r1 * 1.04, r1 * 1.0, 0.0)], s, cuff_mat, (0, 0), false);
                if s_len < f.upper - 0.01 {
                    loft(
                        k,
                        &[
                            ring(-s_len - 0.014, fore.1 * 1.08, fore.1 * 1.05, 0.0),
                            ring(-f.upper - 0.018, fore.1 * 0.96, fore.1 * 1.02, -0.008),
                        ],
                        s,
                        fore.0,
                        (0, 0),
                        false,
                    );
                }
            }
            false => limb(k, f.upper + 0.02, r0, r1, s, s_mat),
        }
    });
    [FORE_L, FORE_R].iter().for_each(|&seg| {
        k.seg(seg);
        let (f_mat, fr0, fr1) = fore;
        match k.hi {
            true => {
                loft(
                    k,
                    &[
                        ring(0.022, fr0 * 1.02, fr0 * 1.08, -0.01),
                        ring(-0.048, fr0 * 1.12, fr0 * 1.06, 0.004),
                        ring(-f.fore * 0.65, (fr0 + fr1) * 0.52, (fr0 + fr1) * 0.48, 0.002),
                        ring(-f.fore, fr1 * 0.92, fr1 * 0.82, 0.0),
                    ],
                    s,
                    f_mat,
                    (0, 0),
                    false,
                );
                if let Some(cm) = wrist_cuff {
                    loft(k, &[ring(-f.fore + 0.038, fr1 * 1.12, fr1 * 1.05, 0.0), ring(-f.fore + 0.004, fr1 * 0.98, fr1 * 0.9, 0.0)], s, cm, (0, 0), false);
                }
                hand(k, f.fore, palm, tone);
            }
            false => limb(k, f.fore, fr0, fr1, s, f_mat),
        }
    });
}

fn legs(k: &mut Kit, f: &Frame, thigh: (Mat, f32, f32), shin: (Mat, f32, f32), boot: (f32, f32, f32, u8)) {
    let s = k.n(6, 3);
    [THIGH_L, THIGH_R].iter().filter(|_| f.hem < f.thigh).for_each(|&seg| {
        k.seg(seg);
        tube(k, -f.hem, -f.thigh - 0.02, thigh.1, thigh.2, s, thigh.0);
    });
    [SHIN_L, SHIN_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.shin, shin.1, shin.2, s, shin.0);
    });
    [FOOT_L, FOOT_R].iter().for_each(|&seg| {
        k.seg(seg);
        shoe(k, f, boot.0, boot.1, boot.2, boot.3);
    });
}

fn benedita(k: &mut Kit) {
    let f = &BENEDITA;
    let dress = tiled("cast_florido", 4.0);
    let shawl = tiled("cast_xale", 5.0);
    let s8 = k.n(8, 4);
    k.seg(SKIRT);
    garment(k, super::cloth::DRESS, s8, dress);
    k.seg(TORSO);
    let body = [
        ring(-0.03, 0.142, 0.115, 0.0),
        ring(0.07, 0.154, 0.14, 0.026),
        ring(0.15, 0.132, 0.112, 0.012),
        ring(0.25, 0.15, 0.13, 0.01),
        ring(0.33, 0.158, 0.118, -0.002),
        ring(0.362, 0.108, 0.088, -0.005),
        ring(0.378, 0.058, 0.054, -0.005),
    ];
    loft(k, &body, s8, dress, (0, 0), false);
    let wrap = [
        ring(0.275, 0.156, 0.134, 0.004),
        ring(0.335, 0.164, 0.124, -0.004),
        ring(0.368, 0.092, 0.082, -0.006),
    ];
    loft(k, &wrap, s8, shawl, (3, 5), false);
    if k.hi {
        loft(k, &[ring(0.14, 0.138, 0.116, 0.014), ring(0.165, 0.14, 0.118, 0.014)], s8, shawl, (0, 0), false);
        k.tex("cast_xale").tri([[-0.15, 0.28, -0.128], [0.15, 0.28, -0.128], [0.0, 0.12, -0.132]], Uv::Wrap([0.0, 0.0, 0.6, 0.7]));
        let bead = |k: &mut Kit, p: V3, r: f32| {
            card(k, [[p[0] - r, p[1] - r, p[2]], [p[0] + r, p[1] - r, p[2]], [p[0] + r, p[1] + r, p[2]], [p[0] - r, p[1] + r, p[2]]], sw(kit::WOOD), false)
        };
        (0..5).for_each(|i| {
            let t = i as f32 / 4.0;
            [-1.0, 1.0].iter().for_each(|&side| bead(k, [side * 0.058 * (1.0 - t), 0.36 - 0.17 * t, 0.108 + 0.04 * t], 0.011));
        });
        card(k, [[-0.008, 0.08, 0.154], [0.008, 0.08, 0.154], [0.008, 0.18, 0.152], [-0.008, 0.18, 0.152]], sw(kit::BRASS), false);
        card(k, [[-0.03, 0.14, 0.155], [0.03, 0.14, 0.155], [0.03, 0.155, 0.155], [-0.03, 0.155, 0.155]], sw(kit::BRASS), false);
    }
    k.seg(HEAD).at(BIG_HEAD);
    neck_textured(k, "cast_benedita", 0.024, 0.044);
    head(k, "cast_benedita", 0.008, 0.242, 0.095, 0.105, 0.008);
    if k.hi {
        k.seg(X).at(BIG_HEAD);
        loft(k, &[ring(0.015, 0.048, 0.042, 0.005), ring(-0.005, 0.056, 0.05, 0.0)], 5, sw(kit::RED), (0, 0), false);
        loft(k, &egg(0.062, -0.055, 0.068, 0.062, -0.018, 3), 6, ("cast_benedita", Uv::Wrap([0.02, 0.05, 0.16, 0.2])), (0, 0), false);
    }
    sculpted_arms(k, f, dress, (dress, 0.058, 0.052, 0.17), shawl, (sw(kit::SKIN_DARK), 0.042, 0.034), None, kit::SKIN_DARK, 0.074);
    legs(k, f, (sw(kit::SKIN_DARK), 0.068, 0.05), (sw(kit::SKIN_DARK), 0.046, 0.036), (0.046, 0.072, 0.135, kit::BLACK));
    lantern(k);
}

fn tonho(k: &mut Kit) {
    let f = &TONHO;
    let alva = tiled("cast_alva", 4.0);
    let s8 = k.n(8, 4);
    k.seg(SKIRT);
    garment(k, super::cloth::ROBE, s8, alva);
    k.seg(TORSO);
    let lower = [
        ring(-0.03, 0.128, 0.102, 0.0),
        ring(0.08, 0.122, 0.098, 0.006),
        ring(0.2, 0.15, 0.118, 0.01),
    ];
    loft(k, &lower, s8, alva, (0, 0), false);
    let mozetta = [
        ring(0.195, 0.16, 0.124, 0.008),
        ring(0.285, 0.184, 0.128, 0.004),
        ring(0.32, 0.118, 0.092, 0.0),
        ring(0.336, 0.058, 0.054, 0.0),
    ];
    loft(k, &mozetta, s8, sw(kit::RED), (0, 0), false);
    if k.hi {
        loft(k, &[ring(0.324, 0.068, 0.062, 0.002), ring(0.342, 0.06, 0.056, 0.002)], s8, sw(kit::WHITE), (0, 0), false);
        loft(k, &[ring(0.19, 0.163, 0.126, 0.008), ring(0.205, 0.164, 0.127, 0.008)], s8, sw(kit::BRASS), (0, 0), false);
        card(k, [[-0.008, 0.06, 0.13], [0.008, 0.06, 0.13], [0.008, 0.18, 0.134], [-0.008, 0.18, 0.134]], sw(kit::BRASS), false);
        card(k, [[-0.028, 0.135, 0.135], [0.028, 0.135, 0.135], [0.028, 0.15, 0.135], [-0.028, 0.15, 0.135]], sw(kit::BRASS), false);
    }
    k.seg(HEAD).at(BIG_HEAD);
    neck_textured(k, "cast_tonho", 0.022, 0.043);
    head(k, "cast_tonho", 0.006, 0.24, 0.092, 0.102, 0.008);
    sculpted_arms(k, f, sw(kit::RED), (alva, 0.058, 0.066, f.upper - 0.01), sw(kit::WHITE), (sw(kit::SKIN_DARK), 0.038, 0.03), None, kit::SKIN_DARK, 0.068);
    if k.hi {
        [SHIN_L, SHIN_R].iter().for_each(|&seg| {
            k.seg(seg);
            tube(k, -f.shin + 0.09, -f.shin, 0.036, 0.034, 5, sw(kit::WHITE));
        });
    }
    legs(k, f, (sw(kit::BLUE), 0.072, 0.062), (sw(kit::SKIN_DARK), 0.042, 0.033), (0.042, 0.062, 0.125, kit::LEATHER));
    lantern(k);
}

fn jairo(k: &mut Kit) {
    let f = &JAIRO;
    let shirt = tiled("cast_xadrez", 4.0);
    let jeans = tiled("cast_jeans", 4.0);
    let s8 = k.n(8, 4);
    k.seg(PELVIS);
    loft(k, &[ring(-0.12, 0.162, 0.128, 0.0), ring(0.06, 0.168, 0.142, 0.016)], s8, jeans, (0, 0), false);
    if k.hi {
        loft(k, &[ring(0.035, 0.172, 0.148, 0.018), ring(0.078, 0.175, 0.154, 0.022)], s8, sw(kit::LEATHER), (0, 0), false);
        card(k, [[-0.032, 0.038, 0.18], [0.032, 0.038, 0.18], [0.032, 0.08, 0.182], [-0.032, 0.08, 0.182]], sw(kit::BRASS), false);
    }
    k.seg(TORSO);
    let shell = [
        ring(-0.01, 0.162, 0.135, 0.015),
        ring(0.09, 0.174, 0.172, 0.052),
        ring(0.18, 0.176, 0.184, 0.066),
        ring(0.28, 0.184, 0.148, 0.022),
        ring(0.36, 0.198, 0.142, 0.012),
        ring(0.425, 0.206, 0.128, 0.004),
        ring(0.452, 0.132, 0.096, -0.002),
        ring(0.47, 0.066, 0.06, 0.0),
    ];
    loft(k, &shell, k.n(10, 4), shirt, (0, 0), false);
    if k.hi {
        loft(k, &[ring(0.454, 0.082, 0.076, 0.002), ring(0.482, 0.074, 0.07, -0.002)], s8, shirt, (3, 5), false);
        let front = |r: &Ring, x: f32, dz: f32| [x, r.y, r.rz + r.dz + dz];
        shell[..6].windows(2).for_each(|w| {
            card(k, [front(&w[0], -0.016, 0.005), front(&w[0], 0.016, 0.005), front(&w[1], 0.016, 0.005), front(&w[1], -0.016, 0.005)], sw(kit::RED), false);
            let mid = [0.0, (w[0].y + w[1].y) * 0.5, (w[0].rz + w[0].dz + w[1].rz + w[1].dz) * 0.5 + 0.008];
            card(k, [[mid[0] - 0.006, mid[1] - 0.006, mid[2]], [mid[0] + 0.006, mid[1] - 0.006, mid[2]], [mid[0] + 0.006, mid[1] + 0.006, mid[2]], [mid[0] - 0.006, mid[1] + 0.006, mid[2]]], sw(kit::WHITE), false);
        });
    }
    k.seg(HEAD).at(BIG_HEAD);
    neck_textured(k, "cast_jairo", 0.026, 0.048);
    head(k, "cast_jairo", 0.008, 0.258, 0.098, 0.11, 0.01);
    loft(k, &[ring(0.205, 0.102, 0.116, 0.006), ring(0.255, 0.096, 0.11, 0.008), ring(0.288, 0.048, 0.056, 0.002)], s8, sw(kit::RED), (0, 0), true);
    if k.hi {
        card(k, [[-0.048, 0.208, 0.126], [0.048, 0.208, 0.126], [0.04, 0.26, 0.12], [-0.04, 0.26, 0.12]], sw(kit::WHITE), false);
    }
    card(k, [[-0.09, 0.205, 0.088], [0.09, 0.205, 0.088], [0.078, 0.196, 0.236], [-0.078, 0.196, 0.236]], sw(kit::RED), true);
    sculpted_arms(k, f, shirt, (shirt, 0.076, 0.07, 0.16), sw(kit::RED), (sw(kit::SKIN_TAN), 0.06, 0.046), None, kit::SKIN_TAN, 0.094);
    legs(k, f, (jeans, 0.112, 0.084), (jeans, 0.078, 0.062), (0.064, 0.1, 0.18, kit::LEATHER));
    lantern(k);
}

fn bia(k: &mut Kit) {
    let f = &BIA;
    let hoodie = tiled("cast_moletom", 4.0);
    let jeans = tiled("cast_jeans", 4.0);
    let s8 = k.n(8, 4);
    k.seg(PELVIS);
    loft(k, &[ring(-0.12, 0.134, 0.104, 0.0), ring(0.04, 0.142, 0.112, 0.0)], s8, jeans, (0, 0), false);
    k.seg(TORSO);
    let body = [
        ring(-0.06, 0.142, 0.112, 0.006),
        ring(-0.01, 0.146, 0.116, 0.01),
        ring(0.1, 0.12, 0.096, 0.008),
        ring(0.2, 0.142, 0.116, 0.016),
        ring(0.28, 0.154, 0.132, 0.022),
        ring(0.365, 0.168, 0.116, 0.004),
        ring(0.398, 0.112, 0.086, -0.004),
        ring(0.418, 0.056, 0.052, -0.004),
    ];
    loft(k, &body, s8, hoodie, (0, 0), false);
    let hood_collar = [
        ring(0.395, 0.082, 0.08, -0.008),
        ring(0.422, 0.076, 0.084, -0.022),
        ring(0.34, 0.098, 0.072, -0.088),
        ring(0.28, 0.048, 0.038, -0.108),
    ];
    loft(k, &hood_collar, s8, hoodie, (0, 0), false);
    k.seg(SKIRT);
    cube(k, [-0.145, 0.04, -0.28], [0.145, 0.38, -0.09], sw(kit::OLIVE));
    if k.hi {
        cube(k, [-0.115, 0.05, -0.33], [0.115, 0.21, -0.28], sw(kit::OLIVE));
        card(k, [[-0.13, 0.382, -0.27], [0.13, 0.382, -0.27], [0.14, 0.26, -0.285], [-0.14, 0.26, -0.285]], sw(kit::LEATHER), false);
        k.seg(TORSO);
        [-1.0, 1.0].iter().for_each(|&sd| {
            let path = [[sd * 0.118, 0.12, 0.115], [sd * 0.122, 0.365, 0.112], [sd * 0.102, 0.405, -0.01], [sd * 0.102, 0.32, -0.11]];
            path.windows(2).for_each(|w| card(k, [[w[0][0] - 0.014, w[0][1], w[0][2]], [w[0][0] + 0.014, w[0][1], w[0][2]], [w[1][0] + 0.014, w[1][1], w[1][2]], [w[1][0] - 0.014, w[1][1], w[1][2]]], sw(kit::LEATHER), false));
        });
        card(k, [[-0.005, 0.04, 0.138], [0.005, 0.04, 0.138], [0.005, 0.395, 0.118], [-0.005, 0.395, 0.118]], sw(kit::GREY), false);
        [-1.0, 1.0].iter().for_each(|&sd| spike(k, [sd * 0.032, 0.395, 0.095], [sd * 0.036, 0.28, 0.14], 0.005, sw(kit::WHITE)));
    }
    k.seg(HEAD).at(BIG_HEAD);
    neck_textured(k, "cast_bia", 0.024, 0.043);
    head(k, "cast_bia", 0.008, 0.248, 0.092, 0.104, 0.008);
    if k.hi {
        k.seg(X).at(BIG_HEAD);
        let tail = [ring(0.035, 0.042, 0.038, -0.005), ring(-0.07, 0.048, 0.042, -0.025), ring(-0.21, 0.018, 0.018, -0.04)];
        loft(k, &tail, 6, ("cast_bia", Uv::Wrap([0.03, 0.05, 0.15, 0.25])), (0, 0), false);
    }
    sculpted_arms(k, f, hoodie, (hoodie, 0.058, 0.052, f.upper), hoodie, (hoodie, 0.05, 0.038), Some(hoodie), kit::SKIN_FAIR, 0.072);
    legs(k, f, (jeans, 0.076, 0.058), (jeans, 0.055, 0.043), (0.046, 0.078, 0.145, kit::WHITE));
    cube(k, [-0.042, -0.05, -0.085], [0.042, 0.052, 0.105], sw(kit::GREY));
    if k.hi {
        cube(k, [-0.02, 0.052, -0.07], [0.02, 0.082, 0.06], sw(kit::BLACK));
        k.at(mat::mul(&mat::translate([0.0, 0.0, 0.105]), &mat::rot_x(FRAC_PI_2)));
        loft(k, &[ring(0.0, 0.034, 0.034, 0.0), ring(0.055, 0.042, 0.042, 0.0)], 6, sw(kit::BLACK), (0, 0), true);
        k.at(mat::IDENTITY);
        k.glow(0.85);
        card(k, [[0.044, -0.035, -0.04], [0.044, -0.035, 0.05], [0.044, 0.035, 0.05], [0.044, 0.035, -0.04]], sw(kit::BLUE), true);
        card(k, [[-0.012, 0.054, 0.07], [0.012, 0.054, 0.07], [0.012, 0.054, 0.095], [-0.012, 0.054, 0.095]], sw(kit::RED), false);
        k.glow(0.0);
    }
}

pub fn garment(k: &mut Kit, cloth: super::cloth::Cloth, sides: usize, material: Mat) {
    let rings = [cloth.top, -cloth.hem * 0.45, -cloth.hem].map(|y| {
        let (_, radius) = cloth.ring(y);
        ring(y, radius[0], radius[1], 0.0)
    });
    k.tex(material.0).loft(&rings, sides.max(8), material.1, (0, 0), false);
}
