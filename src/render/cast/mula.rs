use super::super::mat::{self, M4, V3};
use super::heroes::{card, loft, sw, tiled, tube};
use super::kit::{self, Kit, ring};
use super::skeleton::{Pose, at, ease, lift, noise, rot, snap};
use super::{ARM_L, ARM_R, FOOT_L, FOOT_R, FORE_L, FORE_R, FX, HEAD, LAMP, PELVIS, Rig, SEGS, SHIN_L, SHIN_R, SKIRT, THIGH_L, THIGH_R, TORSO, X, locomotion, rig_from};
use crate::sim::LegendAnim;
use std::f32::consts::{FRAC_PI_2, TAU};

const THIGH: f32 = 0.46;
const GASKIN: f32 = 0.44;
const CANNON: f32 = 0.36;
const ARM: f32 = 0.58;
const SHANK: f32 = 0.58;
const HOOF: f32 = 0.07;
const NECK: f32 = 0.52;
const NECK_TILT: f32 = 0.88;

pub fn build(k: &mut Kit) {
    let hide_tex = tiled("cast_pelo", 3.0);
    let mane_tex = tiled("cast_preto", 3.0);
    let s8 = k.n(8, 4);
    let along = mat::rot_x(FRAC_PI_2);
    k.seg(PELVIS).at(along);
    let rump = [
        ring(-0.88, 0.0, 0.0, -0.09),
        ring(-0.78, 0.18, 0.215, -0.06),
        ring(-0.56, 0.25, 0.27, -0.08),
        ring(-0.24, 0.21, 0.235, 0.01),
        ring(0.0, 0.235, 0.27, 0.02),
    ];
    loft(k, &rump, s8, hide_tex, (0, 0), false);
    k.seg(TORSO).at(along);
    let chest = [
        ring(0.0, 0.225, 0.26, 0.02),
        ring(0.32, 0.265, 0.345, 0.01),
        ring(0.58, 0.225, 0.285, 0.02),
        ring(0.78, 0.13, 0.165, 0.04),
    ];
    k.glow(0.35);
    loft(k, &chest, s8, hide_tex, (0, 0), true);
    k.glow(0.0);
    if k.hi {
        loft(k, &[ring(0.18, 0.258, 0.335, 0.015), ring(0.24, 0.262, 0.342, 0.012)], s8, sw(kit::LEATHER), (0, 0), false);
    }
    k.seg(HEAD);
    let neck = [
        ring(-0.14, 0.16, 0.26, 0.02),
        ring(0.22, 0.135, 0.195, -0.015),
        ring(NECK, 0.115, 0.145, 0.0),
    ];
    k.glow(0.45);
    loft(k, &neck, s8, hide_tex, (0, 0), false);
    loft(
        k,
        &[
            ring(NECK - 0.03, 0.125, 0.152, 0.0),
            ring(NECK + 0.025, 0.095, 0.115, 0.0),
            ring(NECK + 0.05, 0.0, 0.0, 0.0),
        ],
        s8,
        sw(kit::EMBER),
        (0, 0),
        false,
    );
    k.glow(0.0);
    if k.hi {
        k.glow(0.6);
        loft(k, &[ring(NECK - 0.11, 0.125, 0.15, 0.0), ring(NECK - 0.04, 0.125, 0.15, 0.0)], 8, sw(kit::BRASS), (0, 0), false);
        (0..4).for_each(|i| {
            let y = -0.02 + 0.13 * i as f32;
            card(k, [[0.0, y, -0.11], [0.0, y + 0.18, -0.08], [0.0, y + 0.15, -0.26], [0.0, y - 0.03, -0.3]], mane_tex, true);
        });
        [-1.0, 1.0].iter().for_each(|&sd| chain(k, [sd * 0.125, NECK - 0.07, 0.04], [sd * 0.19, -0.04, 0.38], 0.15, 7));
        chain(k, [0.115, NECK - 0.02, 0.09], [-0.115, NECK - 0.02, 0.09], 0.15, 4);
        k.glow(0.0);
    }
    k.seg(FX);
    super::cabra::flame(k);
    k.seg(X);
    loft(
        k,
        &[
            ring(0.0, 0.042, 0.042, 0.0),
            ring(-0.22, 0.068, 0.055, -0.06),
            ring(-0.55, 0.095, 0.05, -0.14),
            ring(-0.84, 0.035, 0.02, -0.18),
            ring(-0.94, 0.0, 0.0, -0.2),
        ],
        k.n(5, 3),
        mane_tex,
        (0, 0),
        false,
    );
    let s = k.n(6, 3);
    [THIGH_L, THIGH_R].iter().for_each(|&seg| {
        k.seg(seg);
        loft(
            k,
            &[
                ring(0.06, 0.185, 0.225, -0.02),
                ring(-THIGH * 0.55, 0.145, 0.175, 0.02),
                ring(-THIGH - 0.02, 0.108, 0.128, 0.0),
            ],
            s,
            hide_tex,
            (0, 0),
            false,
        );
    });
    [SHIN_L, SHIN_R].iter().for_each(|&seg| {
        k.seg(seg);
        loft(
            k,
            &[
                ring(0.02, 0.11, 0.128, 0.0),
                ring(-GASKIN * 0.45, 0.092, 0.11, -0.012),
                ring(-GASKIN + 0.06, 0.062, 0.072, -0.015),
                ring(-GASKIN - 0.02, 0.076, 0.096, -0.032),
            ],
            s,
            hide_tex,
            (0, 0),
            false,
        );
    });
    [FOOT_L, FOOT_R].iter().for_each(|&seg| {
        k.seg(seg);
        loft(
            k,
            &[
                ring(0.0, 0.065, 0.078, -0.015),
                ring(-CANNON * 0.65, 0.048, 0.056, 0.0),
                ring(-CANNON + 0.065, 0.064, 0.072, -0.008),
                ring(-CANNON, 0.058, 0.064, 0.018),
            ],
            k.n(5, 3),
            mane_tex,
            (0, 0),
            false,
        );
        hoof(k, CANNON);
    });
    [ARM_L, ARM_R].iter().for_each(|&seg| {
        k.seg(seg);
        loft(
            k,
            &[
                ring(0.05, 0.145, 0.175, 0.01),
                ring(-ARM * 0.42, 0.112, 0.132, 0.0),
                ring(-ARM * 0.78, 0.074, 0.086, 0.0),
                ring(-ARM - 0.02, 0.08, 0.09, 0.014),
            ],
            s,
            hide_tex,
            (0, 0),
            false,
        );
    });
    [(FORE_L, LAMP), (FORE_R, SKIRT)].iter().for_each(|&(seg, foot)| {
        k.seg(seg);
        loft(
            k,
            &[
                ring(0.0, 0.074, 0.084, 0.01),
                ring(-SHANK * 0.68, 0.052, 0.06, 0.0),
                ring(-SHANK + 0.065, 0.064, 0.072, -0.008),
                ring(-SHANK, 0.058, 0.064, 0.018),
            ],
            k.n(5, 3),
            mane_tex,
            (0, 0),
            false,
        );
        k.seg(foot);
        hoof(k, 0.0);
    });
}

fn chain(k: &mut Kit, a: V3, b: V3, sag: f32, n: usize) {
    let at = |t: f32| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t + sag * 4.0 * t * (1.0 - t)];
    (0..n).for_each(|i| {
        let (p, q) = (at(i as f32 / n as f32), at((i + 1) as f32 / n as f32));
        let d = mat::sub(q, p);
        let w = mat::scale(mat::norm(mat::cross(d, if i % 2 == 0 { [1.0, 0.0, 0.0] } else { [0.0, 0.0, 1.0] })), 0.022);
        card(k, [mat::sub(p, w), mat::add(p, w), mat::add(q, w), mat::sub(q, w)], sw(if i % 2 == 0 { kit::BRASS } else { kit::WHITE }), false);
    });
}

fn hoof(k: &mut Kit, len: f32) {
    match k.hi {
        true => {
            loft(
                k,
                &[
                    ring(-len + 0.01, 0.052, 0.058, 0.018),
                    ring(-len - HOOF * 0.72, 0.066, 0.076, 0.032),
                ],
                5,
                sw(kit::HOOF),
                (0, 0),
                false,
            );
            k.glow(1.0);
            loft(
                k,
                &[
                    ring(-len - HOOF * 0.7, 0.068, 0.078, 0.032),
                    ring(-len - HOOF, 0.07, 0.08, 0.034),
                ],
                5,
                sw(kit::EMBER),
                (0, 0),
                true,
            );
            k.glow(0.0);
        }
        false => tube(k, -len, -len - HOOF, 0.038, 0.05, 3, sw(kit::HOOF)),
    }
}

fn contacts() -> [(usize, V3); 6] {
    [
        (FOOT_L, [0.0, -CANNON - HOOF, 0.0]),
        (FOOT_R, [0.0, -CANNON - HOOF, 0.0]),
        (LAMP, [0.0, -HOOF, 0.0]),
        (SKIRT, [0.0, -HOOF, 0.0]),
        (PELVIS, [0.0, -0.3, -0.3]),
        (TORSO, [0.0, -0.3, 0.3]),
    ]
}

fn assemble(p: &Pose, x: f32, z: f32, heading: f32) -> [M4; SEGS] {
    let place = mat::mul(&mat::translate([x, 0.0, z]), &mat::rot_y(heading));
    let root = mat::mul(&mat::mul(&place, &mat::translate([0.0, 0.0, p.slide])), &mat::mul(&mat::rot_x(p.pitch), &mat::rot_z(p.roll)));
    let pelvis = at(&root, [0.0; 3], &rot(p.pelvis));
    let torso = at(&root, [0.0; 3], &rot(p.torso));
    let head = at(&torso, [0.0, 0.2, 0.52], &rot([NECK_TILT + p.head[0], p.head[1], p.head[2]]));
    let tail = at(&pelvis, [0.0, 0.24, -0.8], &rot([0.08 + p.x[0], p.x[1], p.x[2]]));
    let hind = |side: f32, hip: [f32; 2], knee: f32, ankle: f32| {
        let thigh = at(&pelvis, [side * 0.16, 0.02, -0.54], &mat::mul(&mat::rot_z(side * hip[1]), &mat::rot_x(hip[0] - 0.32)));
        let shin = at(&thigh, [0.0, -THIGH, 0.0], &mat::rot_x(knee + 0.68));
        let foot = at(&shin, [0.0, -GASKIN, 0.0], &mat::rot_x(ankle - 0.36));
        (thigh, shin, foot)
    };
    let front = |side: f32, a: [f32; 3], knee: f32| {
        let arm = at(&torso, [side * 0.16, -0.02, 0.5], &mat::mul(&mat::rot_z(side * a[1]), &mat::rot_x(a[0])));
        let fore = at(&arm, [0.0, -ARM, 0.0], &mat::rot_x(knee));
        (arm, fore)
    };
    let (thigh_l, shin_l, foot_l) = hind(1.0, p.hip_l, p.knee_l, p.ankle_l);
    let (thigh_r, shin_r, foot_r) = hind(-1.0, p.hip_r, p.knee_r, p.ankle_r);
    let (arm_l, fore_l) = front(1.0, p.arm_l, p.elbow_l);
    let (arm_r, fore_r) = front(-1.0, p.arm_r, p.elbow_r);
    let mut m = [[0.0; 16]; SEGS];
    [
        (PELVIS, pelvis),
        (TORSO, torso),
        (HEAD, head),
        (ARM_L, arm_l),
        (FORE_L, fore_l),
        (ARM_R, arm_r),
        (FORE_R, fore_r),
        (THIGH_L, thigh_l),
        (SHIN_L, shin_l),
        (THIGH_R, thigh_r),
        (SHIN_R, shin_r),
        (FOOT_L, foot_l),
        (FOOT_R, foot_r),
        (LAMP, at(&fore_l, [0.0, -SHANK, 0.0], &mat::IDENTITY)),
        (SKIRT, at(&fore_r, [0.0, -SHANK, 0.0], &mat::IDENTITY)),
        (X, tail),
    ]
    .iter()
    .for_each(|&(s, v)| m[s] = v);
    let low = contacts().iter().map(|&(s, q)| mat::point(&m[s], q)[1]).fold(f32::MAX, f32::min);
    lift(&mut m, p.lift - low);
    m
}

fn pose(anim: LegendAnim, tic: u16) -> Pose {
    let t = tic as f32;
    let blind = 0.4 * (t * 0.031).sin() + 0.2 * noise(snap(t, 41.0) as i32);
    match anim {
        LegendAnim::Idle => {
            let paw = ((t % 70.0) / 70.0 * TAU * 2.0).sin().max(0.0) * (noise(snap(t, 70.0) as i32) > 0.0) as u8 as f32;
            Pose { head: [0.05 * (t * 0.07).sin(), blind, 0.1 * blind], arm_l: [-0.5 * paw, 0.0, 0.0], elbow_l: 1.4 * paw, x: [0.08 * (t * 0.05).sin(), 0.22 * (t * 0.043).sin(), 0.0], ..Pose::default() }
        }
        LegendAnim::Walk => {
            let ph = t * TAU / 36.0;
            Pose {
                roll: 0.035 * ph.cos(),
                pelvis: [0.05 * (ph * 2.0).cos(), 0.04 * ph.sin(), 0.0],
                torso: [-0.04 * (ph * 2.0).cos(), 0.0, 0.0],
                head: [0.15 * (ph * 2.0).sin(), blind * 0.45, 0.0],
                x: [0.18 + 0.1 * (ph * 2.0).sin(), 0.32 * ph.sin(), 0.0],
                ..Pose::default()
            }
        }
        LegendAnim::Run => {
            let ph = t * TAU / PERIOD[1];
            let rock = (ph - 0.9).sin();
            Pose {
                pitch: 0.14 * rock,
                pelvis: [-0.1 * rock, 0.0, 0.0],
                torso: [0.12 * rock, 0.0, 0.0],
                head: [-0.22 + 0.32 * rock, 0.0, 0.0],
                x: [0.75 + 0.3 * rock, 0.0, 0.0],
                ..Pose::default()
            }
        }
        LegendAnim::Sniff => {
            let sweep = (t * 0.05).sin();
            Pose { head: [1.05, 0.7 * sweep, 0.2 * sweep], torso: [0.05, 0.0, 0.0], arm_l: [-0.15, 0.05, 0.0], arm_r: [-0.1, -0.05, 0.0], x: [0.05, 0.0, 0.0], ..Pose::default() }
        }
        LegendAnim::Alert => {
            let land = ease((t - 8.0) / 5.0);
            let k = ease(t / 4.0) * (1.0 - land);
            let kick = (t * 0.9).sin();
            Pose {
                pitch: -0.65 * k,
                head: [-0.45 * k + 0.35 * land, 0.1 * kick * k, 0.0],
                hip_l: [0.65 * k, 0.05],
                hip_r: [0.6 * k, 0.05],
                knee_l: 0.25 * k,
                knee_r: 0.3 * k,
                arm_l: [(-0.75 - 0.55 * kick) * k - 0.25 * land, 0.05, 0.0],
                arm_r: [(-0.75 + 0.55 * kick) * k, 0.05, 0.0],
                elbow_l: 1.5 * k + 0.4 * land,
                elbow_r: 1.4 * k,
                x: [0.75 * k + 0.4 * land, 0.3 * kick * land, 0.0],
                ..Pose::default()
            }
        }
        LegendAnim::Grab | LegendAnim::Special => {
            let k = ease(t / 8.0);
            let kick = (t * 0.5).sin();
            Pose {
                pitch: -0.78 * k,
                head: [-0.5 * k, 0.15 * kick, 0.0],
                hip_l: [0.78 * k, 0.05],
                hip_r: [0.72 * k, 0.05],
                knee_l: 0.3 * k,
                knee_r: 0.35 * k,
                arm_l: [(-0.65 - 0.55 * kick) * k, 0.05, 0.0],
                arm_r: [(-0.65 + 0.55 * kick) * k, 0.05, 0.0],
                elbow_l: 1.5 * k,
                elbow_r: 1.4 * k,
                x: [0.8 * k, 0.0, 0.0],
                ..Pose::default()
            }
        }
        LegendAnim::Stagger => {
            let w = (t * 1.3).sin();
            Pose {
                roll: 0.12 * w,
                head: [0.6, 0.9 * w, 0.4 * w],
                hip_l: [-0.2, 0.25],
                hip_r: [0.15, 0.25],
                arm_l: [-0.3, 0.3, 0.0],
                arm_r: [0.25, 0.3, 0.0],
                knee_l: 0.3,
                elbow_r: 0.4,
                x: [0.3, 0.6 * w, 0.0],
                ..Pose::default()
            }
        }
    }
}
pub const PERIOD: [f32; 2] = [36.0, 20.0];

pub fn rig(x: f32, z: f32, heading: f32, anim: LegendAnim, tic: u16, travel: [f32; 2]) -> Rig {
    let p = pose(anim, tic);
    let mut mats = assemble(&p, x, z, heading);
    if matches!(anim, LegendAnim::Walk | LegendAnim::Run) {
        let run = matches!(anim, LegendAnim::Run);
        let f = locomotion::Quadruped { hind: [THIGH, GASKIN, CANNON], front: [ARM, SHANK], sole: HOOF, height: 1.16, spread: [0.15, 0.15], z: [0.5, -0.5] };
        locomotion::quadruped(&mut mats, &f, [x, z, heading], tic as f32, PERIOD[run as usize], travel[0], run);
    }
    let neck = mats[HEAD];
    let t = tic as f32;
    let roar = match anim {
        LegendAnim::Special | LegendAnim::Alert => 1.45,
        LegendAnim::Run | LegendAnim::Grab => 1.25,
        _ => 1.05,
    };
    let flicker = 0.88 + 0.1 * (t * 2.3).sin() + 0.06 * noise(tic as i32);
    let stump = mat::point(&neck, [0.0, NECK + 0.01, 0.0]);
    let up = mat::norm(mat::add(mat::dir(&neck, [0.0, 1.0, 0.0]), [0.0, 1.15, 0.0]));
    let s = roar * flicker;
    let jitter = |i: i32| 1.0 + 0.16 * noise(tic as i32 * 3 + i);
    mats[FX] = super::cabra::jet(stump, up, 1.18 * s * jitter(1), 0.82 * s * jitter(2), t * 0.7 + 0.4 * noise(tic as i32 + 5));
    let pelvis = mats[PELVIS];
    let hands = [mats[FORE_R], mats[FORE_L]];
    let r = rig_from(mats, hands, false, &pelvis, [0.0, NECK, 0.0]);
    Rig { fire: Some((stump, up, s * 1.25)), ..r }
}
