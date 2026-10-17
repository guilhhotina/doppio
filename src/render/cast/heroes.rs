use super::super::mat::{self, V3};
use super::kit::{self, Kit, Ring, Uv, egg, ring, swatch};
use super::skeleton::Frame;
use super::{ARM_L, ARM_R, FOOT_L, FOOT_R, FORE_L, FORE_R, HEAD, LAMP, PELVIS, SHIN_L, SHIN_R, SKIRT, THIGH_L, THIGH_R, TORSO, X};
use std::f32::consts::FRAC_PI_2;

pub type Mat = (&'static str, Uv);

const BIG: f32 = 1.18;
const BIG_HEAD: [f32; 16] = [BIG, 0.0, 0.0, 0.0, 0.0, BIG, 0.0, 0.0, 0.0, 0.0, BIG, 0.0, 0.0, 0.0, 0.0, 1.0];

pub const BENEDITA: Frame = Frame {
    hip_x: 0.08,
    waist: 0.06,
    chest: 0.37,
    shoulder_x: 0.215,
    neck: 0.42,
    upper: 0.26,
    fore: 0.22,
    thigh: 0.38,
    shin: 0.37,
    meta: 0.0,
    sole: 0.06,
    hem: 0.56,
    back: 0.2,
    x_at: (HEAD, [0.0, 0.2 * BIG, -0.09 * BIG]),
};

pub const TONHO: Frame = Frame {
    hip_x: 0.07,
    waist: 0.05,
    chest: 0.32,
    shoulder_x: 0.18,
    neck: 0.36,
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
    hip_x: 0.12,
    waist: 0.07,
    chest: 0.47,
    shoulder_x: 0.285,
    neck: 0.52,
    upper: 0.31,
    fore: 0.28,
    thigh: 0.46,
    shin: 0.45,
    meta: 0.0,
    sole: 0.07,
    hem: 0.0,
    back: 0.15,
    x_at: (TORSO, [0.0, 0.45, 0.1]),
};

pub const BIA: Frame = Frame {
    hip_x: 0.085,
    waist: 0.06,
    chest: 0.41,
    shoulder_x: 0.205,
    neck: 0.46,
    upper: 0.28,
    fore: 0.25,
    thigh: 0.42,
    shin: 0.41,
    meta: 0.0,
    sole: 0.06,
    hem: 0.0,
    back: 0.365,
    x_at: (HEAD, [0.0, 0.2 * BIG, -0.1 * BIG]),
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
    let rings = egg(top, bottom, rx, rz, dz, k.n(5, 3));
    let sides = k.n(8, 4);
    k.tex(tex).loft(&rings, sides, Uv::Wrap([0.0005, 0.0, 0.9995, 1.0]), (0, 0), false);
    if k.hi {
        let y = bottom + (top - bottom) * 0.48;
        let a = [-rx * 0.2, y - 0.025, dz + rz * 0.94];
        let b = [rx * 0.2, y - 0.025, dz + rz * 0.94];
        let c = [0.0, y + 0.045, dz + rz * 0.94];
        let tip = [0.0, y, dz + rz + 0.025];
        [[a, b, tip], [b, c, tip], [c, a, tip]].into_iter().for_each(|p| k.poly(&p, &[[0.48, 0.58], [0.52, 0.58], [0.5, 0.5]]));
    }
}

pub fn hand(k: &mut Kit, len: f32, w: f32, tone: u8) {
    if k.hi {
        cube(k, [-w * 0.45, -len - 0.08, -w * 0.3], [w * 0.45, -len + 0.01, w * 0.45], sw(tone));
    }
}

pub fn shoe(k: &mut Kit, f: &Frame, w: f32, h: f32, toe: f32, tone: u8) {
    let y = -f.sole;
    cube(k, [-w, y, -0.055], [w, y + h, toe], sw(tone));
}

pub fn neck(k: &mut Kit, h: f32, r: f32, tone: u8) {
    if k.hi {
        tube(k, -0.02, h, r, r * 0.9, 5, sw(tone));
    }
}

pub fn lantern(k: &mut Kit) {
    k.seg(LAMP);
    k.at(mat::mul(&mat::translate([0.0, 0.0, -0.04]), &mat::rot_x(FRAC_PI_2)));
    match k.hi {
        true => {
            tube(k, 0.0, 0.13, 0.022, 0.024, 6, sw(kit::GREY));
            loft(k, &[ring(0.13, 0.026, 0.026, 0.0), ring(0.17, 0.04, 0.04, 0.0)], 6, sw(kit::BRASS), (0, 0), true);
        }
        false => loft(k, &[ring(0.0, 0.025, 0.025, 0.0), ring(0.17, 0.04, 0.04, 0.0)], 3, sw(kit::BRASS), (0, 0), true),
    }
}

fn arms(k: &mut Kit, f: &Frame, upper: (Mat, f32, f32), fore: (Mat, f32, f32), tone: u8, palm: f32) {
    let s = k.n(6, 3);
    [ARM_L, ARM_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.upper + 0.02, upper.1, upper.2, s, upper.0);
    });
    [FORE_L, FORE_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.fore, fore.1, fore.2, s, fore.0);
        hand(k, f.fore, palm, tone);
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

fn grow(r: Ring, g: f32) -> Ring {
    ring(r.y, r.rx + g, r.rz + g, r.dz)
}

fn benedita(k: &mut Kit) {
    let f = &BENEDITA;
    let dress = tiled("cast_florido", 4.0);
    let s8 = k.n(8, 4);
    k.seg(SKIRT);
    garment(k, super::cloth::DRESS, s8, dress);
    k.seg(TORSO);
    loft(k, &[ring(-0.02, 0.14, 0.11, 0.0), ring(0.17, 0.16, 0.13, 0.025), ring(0.33, 0.165, 0.115, 0.0), ring(0.42, 0.07, 0.06, 0.0)], s8, dress, (0, 0), false);
    let shawl = tiled("cast_xale", 5.0);
    loft(k, &[ring(0.44, 0.09, 0.08, 0.0), ring(0.37, 0.215, 0.165, -0.005), ring(0.2, 0.228, 0.178, -0.015)], s8, shawl, (0, 0), false);
    k.tex("cast_xale").tri([[-0.2, 0.21, -0.2], [0.2, 0.21, -0.2], [0.0, -0.05, -0.17]], Uv::Wrap([0.0, 0.0, 0.6, 0.7]));
    if k.hi {
        let bead = |k: &mut Kit, p: V3, r: f32| {
            card(k, [[p[0] - r, p[1] - r, p[2]], [p[0] + r, p[1] - r, p[2]], [p[0] + r, p[1] + r, p[2]], [p[0] - r, p[1] + r, p[2]]], sw(kit::WOOD), false)
        };
        (0..5).for_each(|i| {
            let t = i as f32 / 4.0;
            [-1.0, 1.0].iter().for_each(|&side| bead(k, [side * 0.075 * (1.0 - t), 0.39 - 0.21 * t, 0.118 + 0.07 * t], 0.012));
        });
        card(k, [[-0.007, 0.06, 0.192], [0.007, 0.06, 0.192], [0.007, 0.165, 0.19], [-0.007, 0.165, 0.19]], sw(kit::BRASS), false);
        card(k, [[-0.028, 0.125, 0.193], [0.028, 0.125, 0.193], [0.028, 0.139, 0.193], [-0.028, 0.139, 0.193]], sw(kit::BRASS), false);
    }
    k.seg(HEAD).at(BIG_HEAD);
    neck(k, 0.08, 0.042, kit::SKIN_DARK);
    head(k, "cast_benedita", 0.04, 0.27, 0.09, 0.105, 0.012);
    if k.hi {
        k.seg(X).at(BIG_HEAD);
        loft(k, &egg(0.045, -0.04, 0.05, 0.042, 0.0, 3), 5, ("cast_benedita", Uv::Wrap([0.02, 0.05, 0.16, 0.2])), (0, 0), false);
    }
    arms(k, f, (dress, 0.05, 0.042), (sw(kit::SKIN_DARK), 0.036, 0.03), kit::SKIN_DARK, 0.07);
    legs(k, f, (sw(kit::SKIN_DARK), 0.07, 0.05), (sw(kit::SKIN_DARK), 0.045, 0.034), (0.045, 0.07, 0.13, kit::BLACK));
    lantern(k);
}

fn tonho(k: &mut Kit) {
    let f = &TONHO;
    let alva = tiled("cast_alva", 4.0);
    let s8 = k.n(8, 4);
    k.seg(SKIRT);
    garment(k, super::cloth::ROBE, s8, alva);
    k.seg(TORSO);
    let torso = [ring(-0.02, 0.13, 0.1, 0.0), ring(0.16, 0.135, 0.1, 0.01), ring(0.32, 0.155, 0.11, 0.0), ring(0.35, 0.065, 0.055, 0.0)];
    loft(k, if k.hi { &torso } else { &torso[..3] }, s8, alva, (0, 0), false);
    loft(k, &[ring(0.37, 0.07, 0.06, 0.0), ring(0.3, 0.165, 0.12, 0.0), ring(0.21, 0.178, 0.132, 0.0)], s8, sw(kit::RED), (0, 0), false);
    if k.hi {
        card(k, [[-0.008, 0.06, 0.138], [0.008, 0.06, 0.138], [0.008, 0.19, 0.134], [-0.008, 0.19, 0.134]], sw(kit::BRASS), false);
        card(k, [[-0.03, 0.14, 0.139], [0.03, 0.14, 0.139], [0.03, 0.156, 0.138], [-0.03, 0.156, 0.138]], sw(kit::BRASS), false);
    }
    k.seg(HEAD).at(BIG_HEAD);
    neck(k, 0.07, 0.038, kit::SKIN_DARK);
    head(k, "cast_tonho", 0.035, 0.265, 0.093, 0.108, 0.012);
    arms(k, f, (alva, 0.055, 0.05), (sw(kit::SKIN_DARK), 0.032, 0.027), kit::SKIN_DARK, 0.065);
    if k.hi {
        [FORE_L, FORE_R].iter().for_each(|&seg| {
            k.seg(seg);
            tube(k, 0.0, -0.08, 0.056, 0.064, 5, alva);
        });
    }
    legs(k, f, (sw(kit::BLUE), 0.072, 0.062), (sw(kit::SKIN_DARK), 0.04, 0.03), (0.04, 0.06, 0.12, kit::LEATHER));
    lantern(k);
}

fn jairo(k: &mut Kit) {
    let f = &JAIRO;
    let shirt = tiled("cast_xadrez", 4.0);
    let jeans = tiled("cast_jeans", 4.0);
    let s8 = k.n(8, 4);
    k.seg(PELVIS);
    loft(k, &[ring(-0.12, 0.19, 0.15, 0.0), ring(0.06, 0.215, 0.18, 0.03)], s8, jeans, (0, 0), false);
    if k.hi {
        loft(k, &[ring(0.04, 0.222, 0.186, 0.03), ring(0.09, 0.222, 0.19, 0.04)], s8, sw(kit::LEATHER), (0, 0), false);
    }
    k.seg(TORSO);
    let belly = [ring(0.0, 0.215, 0.18, 0.04), ring(0.14, 0.235, 0.24, 0.1), ring(0.3, 0.23, 0.18, 0.04)];
    loft(k, &belly, s8, sw(kit::SKIN_TAN), if k.hi { (6, 10) } else { (3, 5) }, false);
    let gap = if k.hi { (3, 5) } else { (1, 3) };
    loft(k, &belly.map(|r| grow(r, 0.012)), s8, shirt, gap, false);
    loft(k, &[grow(belly[2], 0.012), ring(0.47, 0.255, 0.16, 0.0), ring(0.52, 0.1, 0.09, 0.0)], s8, shirt, (0, 0), false);
    k.seg(HEAD).at(BIG_HEAD);
    neck(k, 0.08, 0.055, kit::SKIN_TAN);
    head(k, "cast_jairo", 0.04, 0.29, 0.1, 0.115, 0.015);
    loft(k, &[ring(0.2, 0.112, 0.13, 0.01), ring(0.27, 0.108, 0.124, 0.012), ring(0.315, 0.06, 0.07, 0.005)], s8, sw(kit::RED), (0, 0), true);
    card(k, [[-0.09, 0.205, 0.1], [0.09, 0.205, 0.1], [0.08, 0.215, 0.23], [-0.08, 0.215, 0.23]], sw(kit::RED), true);
    let s = k.n(6, 3);
    [ARM_L, ARM_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, if k.hi { 0.15 } else { f.upper }, 0.09, 0.082, s, shirt);
        if k.hi {
            tube(k, -0.14, -f.upper - 0.02, 0.07, 0.058, s, sw(kit::SKIN_TAN));
        }
    });
    [FORE_L, FORE_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.fore, 0.058, 0.046, s, sw(kit::SKIN_TAN));
        hand(k, f.fore, 0.1, kit::SKIN_TAN);
    });
    legs(k, f, (jeans, 0.115, 0.085), (jeans, 0.08, 0.064), (0.065, 0.1, 0.18, kit::LEATHER));
    lantern(k);
}

fn bia(k: &mut Kit) {
    let f = &BIA;
    let hoodie = tiled("cast_moletom", 4.0);
    let jeans = tiled("cast_jeans", 4.0);
    let s8 = k.n(8, 4);
    k.seg(PELVIS);
    loft(k, &[ring(-0.12, 0.14, 0.11, 0.0), ring(0.04, 0.155, 0.12, 0.0)], s8, jeans, (0, 0), false);
    k.seg(TORSO);
    loft(k, &[ring(-0.07, 0.165, 0.13, 0.01), ring(0.15, 0.16, 0.12, 0.015), ring(0.41, 0.19, 0.13, 0.0), ring(0.46, 0.08, 0.07, 0.0)], s8, hoodie, (0, 0), false);
    let hood: Vec<Ring> = egg(0.5, 0.36, 0.13, 0.07, 0.0, k.n(3, 2)).iter().map(|r| ring(r.y, r.rx, r.rz, r.dz - 0.07)).collect();
    loft(k, &hood, k.n(6, 4), hoodie, (0, 0), false);
    k.seg(SKIRT);
    cube(k, [-0.13, 0.03, -0.31], [0.13, 0.38, -0.16], sw(kit::OLIVE));
    if k.hi {
        cube(k, [-0.1, 0.06, -0.34], [0.1, 0.2, -0.3], sw(kit::OLIVE));
        k.seg(TORSO);
        [-1.0, 1.0].iter().for_each(|&sd| {
            card(k, [[sd * 0.1 - 0.02, 0.18, 0.145], [sd * 0.1 + 0.02, 0.18, 0.145], [sd * 0.11 + 0.02, 0.42, 0.1], [sd * 0.11 - 0.02, 0.42, 0.1]], sw(kit::BLACK), false)
        });
    }
    k.seg(HEAD).at(BIG_HEAD);
    neck(k, 0.08, 0.04, kit::SKIN_FAIR);
    head(k, "cast_bia", 0.035, 0.27, 0.09, 0.105, 0.012);
    if k.hi {
        k.seg(X).at(BIG_HEAD);
        let tail = [ring(0.02, 0.035, 0.03, 0.0), ring(-0.08, 0.04, 0.035, -0.02), ring(-0.2, 0.015, 0.015, -0.03)];
        loft(k, &tail, 5, ("cast_bia", Uv::Wrap([0.03, 0.05, 0.15, 0.25])), (0, 0), false);
    }
    arms(k, f, (hoodie, 0.058, 0.052), (hoodie, 0.05, 0.045), kit::SKIN_FAIR, 0.07);
    legs(k, f, (jeans, 0.08, 0.06), (jeans, 0.058, 0.045), (0.048, 0.08, 0.15, kit::WHITE));
    k.seg(LAMP);
    cube(k, [-0.04, -0.05, -0.08], [0.04, 0.05, 0.1], sw(kit::GREY));
    if k.hi {
        k.at(mat::mul(&mat::translate([0.0, 0.0, 0.1]), &mat::rot_x(FRAC_PI_2)));
        loft(k, &[ring(0.0, 0.032, 0.032, 0.0), ring(0.05, 0.036, 0.036, 0.0)], 6, sw(kit::BLACK), (0, 0), true);
        k.at(mat::IDENTITY);
        card(k, [[0.041, -0.035, -0.04], [0.041, -0.035, 0.05], [0.041, 0.035, 0.05], [0.041, 0.035, -0.04]], sw(kit::BLUE), false);
        card(k, [[-0.012, 0.051, -0.01], [0.012, 0.051, -0.01], [0.012, 0.051, 0.03], [-0.012, 0.051, 0.03]], sw(kit::RED), false);
    }
}

pub fn garment(k: &mut Kit, cloth: super::cloth::Cloth, sides: usize, material: Mat) {
    let rings = [cloth.top, -cloth.hem * 0.45, -cloth.hem].map(|y| {
        let (_, radius) = cloth.ring(y);
        ring(y, radius[0], radius[1], 0.0)
    });
    k.tex(material.0).loft(&rings, sides.max(8), material.1, (0, 0), false);
}
