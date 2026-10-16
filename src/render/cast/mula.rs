use super::super::mat::{self, M4, V3};
use super::heroes::{card, cube, limb, loft, sw, tiled, tube};
use super::kit::{self, Kit, ring};
use super::skeleton::{Pose, at, ease, lift, noise, rot, snap};
use super::{ARM_L, ARM_R, FOOT_L, FOOT_R, FORE_L, FORE_R, HEAD, PELVIS, Rig, SEGS, SHIN_L, SHIN_R, THIGH_L, THIGH_R, TORSO, X, rig_from};
use crate::sim::LegendAnim;
use std::f32::consts::{FRAC_PI_2, TAU};

const THIGH: f32 = 0.42;
const GASKIN: f32 = 0.38;
const CANNON: f32 = 0.3;
const ARM: f32 = 0.45;
const SHANK: f32 = 0.42;
const HOOF: f32 = 0.06;
const NECK: f32 = 0.52;
const NECK_TILT: f32 = 1.05;

pub fn build(k: &mut Kit) {
    let hide_tex = tiled("cast_pelo", 3.0);
    let s8 = k.n(8, 4);
    let along = mat::rot_x(FRAC_PI_2);
    k.seg(PELVIS).at(along);
    let rump = [ring(-0.82, 0.0, 0.0, -0.05), ring(-0.76, 0.2, 0.25, -0.02), ring(-0.6, 0.29, 0.34, 0.0), ring(-0.3, 0.3, 0.36, 0.02), ring(0.0, 0.29, 0.35, 0.04)];
    loft(k, &rump, s8, hide_tex, (0, 0), false);
    k.seg(TORSO).at(along);
    let chest = [ring(0.0, 0.29, 0.35, 0.04), ring(0.35, 0.29, 0.35, 0.02), ring(0.62, 0.25, 0.32, -0.03), ring(0.76, 0.12, 0.17, -0.1)];
    loft(k, &chest, s8, hide_tex, (0, 0), true);
    k.seg(HEAD);
    let neck = [ring(-0.15, 0.15, 0.26, 0.04), ring(0.25, 0.125, 0.18, -0.01), ring(NECK, 0.11, 0.13, 0.0)];
    loft(k, &neck, s8, hide_tex, (0, 0), false);
    loft(k, &[ring(NECK - 0.01, 0.112, 0.132, 0.0), ring(NECK + 0.02, 0.08, 0.09, 0.0), ring(NECK + 0.03, 0.0, 0.0, 0.0)], s8, sw(kit::EMBER), (0, 0), false);
    if k.hi {
        loft(k, &[ring(NECK - 0.1, 0.12, 0.14, 0.0), ring(NECK - 0.05, 0.12, 0.14, 0.0)], 8, sw(kit::GREY), (0, 0), false);
        (0..3).for_each(|i| {
            let y = 0.05 + 0.18 * i as f32;
            card(k, [[0.0, y, -0.13], [0.0, y + 0.2, -0.11], [0.0, y + 0.16, -0.2], [0.0, y - 0.04, -0.24]], sw(kit::BLACK), true);
        });
        [-1.0, 1.0].iter().for_each(|&sd| {
            card(k, [[sd * 0.11, NECK - 0.08, 0.06], [sd * 0.11, NECK - 0.08, 0.0], [sd * 0.14, 0.05, 0.25], [sd * 0.14, 0.07, 0.3]], sw(kit::LEATHER), true);
        });
    }
    k.seg(X);
    loft(k, &[ring(0.0, 0.04, 0.04, 0.0), ring(-0.25, 0.05, 0.045, -0.03), ring(-0.65, 0.07, 0.03, -0.06), ring(-0.75, 0.0, 0.0, -0.06)], k.n(5, 3), sw(kit::BLACK), (0, 0), false);
    let s = k.n(6, 3);
    [THIGH_L, THIGH_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, THIGH + 0.03, 0.17, 0.1, s, hide_tex);
    });
    [SHIN_L, SHIN_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, GASKIN + 0.02, 0.085, 0.055, s, hide_tex);
    });
    [FOOT_L, FOOT_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, CANNON, 0.05, 0.045, k.n(5, 3), hide_tex);
        hoof(k, CANNON);
    });
    [ARM_L, ARM_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, ARM + 0.02, 0.11, 0.065, s, hide_tex);
    });
    [FORE_L, FORE_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, SHANK, 0.052, 0.045, k.n(5, 3), hide_tex);
        hoof(k, SHANK);
    });
}

fn hoof(k: &mut Kit, len: f32) {
    match k.hi {
        true => cube(k, [-0.05, -len - HOOF, -0.055], [0.05, -len + 0.01, 0.07], sw(kit::HOOF)),
        false => tube(k, -len, -len - HOOF, 0.038, 0.05, 3, sw(kit::HOOF)),
    }
}

fn contacts() -> [(usize, V3); 6] {
    [
        (FOOT_L, [0.0, -CANNON - HOOF, 0.0]),
        (FOOT_R, [0.0, -CANNON - HOOF, 0.0]),
        (FORE_L, [0.0, -SHANK - HOOF, 0.0]),
        (FORE_R, [0.0, -SHANK - HOOF, 0.0]),
        (PELVIS, [0.0, -0.33, -0.3]),
        (TORSO, [0.0, -0.33, 0.3]),
    ]
}

fn assemble(p: &Pose, x: f32, z: f32, heading: f32) -> [M4; SEGS] {
    let place = mat::mul(&mat::translate([x, 0.0, z]), &mat::rot_y(heading));
    let root = mat::mul(&mat::mul(&place, &mat::translate([0.0, 0.0, p.slide])), &mat::mul(&mat::rot_x(p.pitch), &mat::rot_z(p.roll)));
    let pelvis = at(&root, [0.0; 3], &rot(p.pelvis));
    let torso = at(&root, [0.0; 3], &rot(p.torso));
    let head = at(&torso, [0.0, 0.14, 0.6], &rot([NECK_TILT + p.head[0], p.head[1], p.head[2]]));
    let tail = at(&pelvis, [0.0, 0.16, -0.74], &rot([0.35 + p.x[0], p.x[1], p.x[2]]));
    let hind = |side: f32, hip: [f32; 2], knee: f32, ankle: f32| {
        let thigh = at(&pelvis, [side * 0.17, 0.0, -0.5], &mat::mul(&mat::rot_z(side * hip[1]), &mat::rot_x(hip[0] - 0.25)));
        let shin = at(&thigh, [0.0, -THIGH, 0.0], &mat::rot_x(knee + 0.6));
        let foot = at(&shin, [0.0, -GASKIN, 0.0], &mat::rot_x(ankle - 0.35));
        (thigh, shin, foot)
    };
    let front = |side: f32, a: [f32; 3], knee: f32| {
        let arm = at(&torso, [side * 0.16, -0.04, 0.5], &mat::mul(&mat::rot_z(side * a[1]), &mat::rot_x(a[0])));
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
        (X, tail),
    ]
    .iter()
    .for_each(|&(s, v)| m[s] = v);
    let low = contacts().iter().map(|&(s, q)| mat::point(&m[s], q)[1]).fold(f32::MAX, f32::min);
    lift(&mut m, p.lift - low);
    m
}

fn legs(ph: f32, offsets: [f32; 4], amp: f32) -> Pose {
    let swing = |o: f32| ((ph + o * TAU).sin(), (ph + o * TAU + 0.8).cos().max(0.0));
    let (lh, rh, lf, rf) = (swing(offsets[0]), swing(offsets[1]), swing(offsets[2]), swing(offsets[3]));
    Pose {
        hip_l: [-0.4 * amp * lh.0, 0.02],
        hip_r: [-0.4 * amp * rh.0, 0.02],
        knee_l: 0.5 * amp * lh.1,
        knee_r: 0.5 * amp * rh.1,
        ankle_l: -0.3 * amp * lh.1,
        ankle_r: -0.3 * amp * rh.1,
        arm_l: [-0.38 * amp * lf.0, 0.02, 0.0],
        arm_r: [-0.38 * amp * rf.0, 0.02, 0.0],
        elbow_l: 1.1 * amp * lf.1,
        elbow_r: 1.1 * amp * rf.1,
        ..Pose::default()
    }
}

fn pose(anim: LegendAnim, tic: u16) -> Pose {
    let t = tic as f32;
    let blind = 0.4 * (t * 0.031).sin() + 0.2 * noise(snap(t, 41.0) as i32);
    match anim {
        LegendAnim::Idle => {
            let paw = ((t % 70.0) / 70.0 * TAU * 2.0).sin().max(0.0) * (noise(snap(t, 70.0) as i32) > 0.0) as u8 as f32;
            Pose { head: [0.05 * (t * 0.07).sin(), blind, 0.1 * blind], arm_l: [-0.5 * paw, 0.0, 0.0], elbow_l: 1.4 * paw, x: [0.1 * (t * 0.05).sin(), 0.2 * (t * 0.043).sin(), 0.0], ..Pose::default() }
        }
        LegendAnim::Walk => {
            let ph = t * TAU / 40.0;
            Pose { head: [0.08 * (ph * 2.0).sin(), blind * 0.5, 0.0], pelvis: [0.0, 0.04 * ph.sin(), 0.0], x: [0.1, 0.25 * ph.sin(), 0.0], ..legs(ph, [0.0, 0.5, 0.25, 0.75], 1.0) }
        }
        LegendAnim::Run => {
            let ph = t * TAU / 16.0;
            let rock = (ph - 0.9).sin();
            Pose {
                pitch: 0.07 * rock,
                torso: [0.05 * rock, 0.0, 0.0],
                head: [-0.25 + 0.2 * rock, 0.0, 0.0],
                x: [0.9 + 0.2 * rock, 0.0, 0.0],
                ..legs(ph, [0.0, 0.08, 0.5, 0.58], 1.8)
            }
        }
        LegendAnim::Sniff => {
            let sweep = (t * 0.05).sin();
            Pose { head: [1.05, 0.7 * sweep, 0.2 * sweep], torso: [0.05, 0.0, 0.0], arm_l: [-0.15, 0.05, 0.0], arm_r: [-0.1, -0.05, 0.0], x: [0.05, 0.0, 0.0], ..Pose::default() }
        }
        LegendAnim::Grab | LegendAnim::Special => {
            let k = ease(t / 8.0);
            let kick = (t * 0.5).sin();
            Pose {
                pitch: -0.75 * k,
                head: [-0.5 * k, 0.15 * kick, 0.0],
                hip_l: [0.75 * k, 0.05],
                hip_r: [0.7 * k, 0.05],
                knee_l: 0.3 * k,
                knee_r: 0.35 * k,
                arm_l: [(-0.6 - 0.5 * kick) * k, 0.05, 0.0],
                arm_r: [(-0.6 + 0.5 * kick) * k, 0.05, 0.0],
                elbow_l: 1.5 * k,
                elbow_r: 1.4 * k,
                x: [1.0 * k, 0.0, 0.0],
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

pub fn rig(x: f32, z: f32, heading: f32, anim: LegendAnim, tic: u16) -> Rig {
    let p = pose(anim, tic);
    let mats = assemble(&p, x, z, heading);
    let neck = mats[HEAD];
    let pelvis = mats[PELVIS];
    let hands = [mats[FORE_R], mats[FORE_L]];
    let r = rig_from(mats, hands, false, &pelvis, [0.0, NECK, 0.0]);
    let t = tic as f32;
    let roar = match anim {
        LegendAnim::Special => 1.5,
        LegendAnim::Run | LegendAnim::Grab => 1.15,
        _ => 0.8,
    };
    let flicker = 0.85 + 0.1 * (t * 1.9).sin() + 0.05 * noise(tic as i32);
    let fire = (mat::point(&neck, [0.0, NECK + 0.03, 0.0]), mat::norm(mat::dir(&neck, [0.0, 1.0, 0.0])), roar * flicker);
    Rig { fire: Some(fire), ..r }
}
