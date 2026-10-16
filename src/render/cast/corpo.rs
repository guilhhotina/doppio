use super::heroes::{card, cube, hand, limb, loft, sw, tiled};
use super::kit::{self, Kit, Ring, Uv, egg, ring};
use super::skeleton::{self, Frame, Pose, ease, hide, noise, snap};
use super::{ARM_L, ARM_R, FORE_L, FORE_R, HEAD, LAMP, PELVIS, Rig, SHIN_L, SHIN_R, SKIRT, THIGH_L, THIGH_R, TORSO, X, rig_from};
use crate::sim::LegendAnim;
use std::f32::consts::{PI, TAU};

const JAW: [f32; 2] = [0.128, -0.01];

pub const FRAME: Frame = Frame {
    hip_x: 0.075,
    waist: 0.06,
    chest: 0.44,
    shoulder_x: 0.165,
    neck: 0.49,
    upper: 0.34,
    fore: 0.31,
    thigh: 0.47,
    shin: 0.46,
    meta: 0.0,
    sole: 0.05,
    hem: 0.42,
    back: 0.12,
    x_at: (HEAD, [0.0, JAW[0], JAW[1]]),
};

pub fn build(k: &mut Kit) {
    let f = &FRAME;
    let s8 = k.n(8, 4);
    let rag = tiled("cast_trapo", 3.0);
    let hide_tex = tiled("cast_couro", 4.0);
    k.seg(PELVIS);
    loft(k, &[ring(-0.17, 0.12, 0.09, 0.0), ring(0.07, 0.12, 0.085, 0.0)], s8, rag, (0, 0), false);
    k.seg(SKIRT);
    let strips = k.n(7, 3);
    (0..strips).for_each(|i| {
        let a = TAU * i as f32 / strips as f32 + 0.3;
        let (s, c) = a.sin_cos();
        let len = 0.32 + 0.14 * noise(i as i32 * 7 + 3).abs();
        let (w, r0, r1) = (0.075, 0.13, 0.16);
        let p = |r: f32, y: f32, d: f32| [s * r + c * d, y, c * r * 0.8 - s * d];
        card(k, [p(r1, -len, -w * 0.7), p(r1, -len, w * 0.7), p(r0, 0.05, w), p(r0, 0.05, -w)], ("cast_trapo", Uv::Wrap([0.1 * i as f32, 0.0, 0.1 * i as f32 + 0.35, 0.9])), true);
    });
    k.seg(TORSO);
    loft(k, &[ring(-0.02, 0.1, 0.07, 0.0), ring(0.2, 0.145, 0.1, 0.012), ring(0.42, 0.155, 0.085, 0.0), ring(0.51, 0.055, 0.05, 0.0)], s8, hide_tex, (0, 0), false);
    let gap = if k.hi { (3, 5) } else { (1, 3) };
    loft(k, &[ring(0.52, 0.07, 0.065, 0.0), ring(0.43, 0.172, 0.1, 0.0), ring(0.17, 0.162, 0.117, -0.005)], s8, rag, gap, false);
    k.seg(HEAD);
    let s5 = k.n(5, 3);
    loft(k, &[ring(-0.02, 0.032, 0.03, 0.0), ring(0.12, 0.028, 0.026, 0.0)], s5, sw(kit::LEATHER), (0, 0), false);
    let (top, chin) = (0.34, 0.05);
    let full = egg(top, chin, 0.082, 0.098, 0.012, k.n(6, 3));
    let face = |v0: f32, v1: f32| ("cast_corpo", Uv::Wrap([0.0005, v0, 0.9995, v1]));
    match k.hi {
        true => {
            let split = (top - full[4].y) / (top - chin);
            loft(k, &full[..=4], 8, face(0.0, split), (0, 0), false);
            card(k, [[-0.04, 0.06, 0.07], [0.04, 0.06, 0.07], [0.04, 0.135, 0.06], [-0.04, 0.135, 0.06]], sw(kit::BLACK), false);
            k.seg(X);
            let jaw: Vec<Ring> = full[4..].iter().map(|r| ring(r.y - JAW[0], r.rx, r.rz, r.dz - JAW[1])).collect();
            loft(k, &jaw, 8, face(split, 1.0), (0, 0), false);
        }
        false => loft(k, &full, 4, face(0.0, 1.0), (0, 0), false),
    }
    let s = k.n(6, 3);
    [ARM_L, ARM_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.upper + 0.02, 0.036, 0.027, s, hide_tex);
    });
    [FORE_L, FORE_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.fore, 0.028, 0.02, s, hide_tex);
        hand(k, f.fore, 0.06, kit::LEATHER);
        if k.hi {
            cube(k, [-0.022, -f.fore - 0.19, -0.005], [0.022, -f.fore - 0.07, 0.015], sw(kit::HOOF));
        }
    });
    [THIGH_L, THIGH_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.thigh + 0.02, 0.048, 0.034, s, hide_tex);
    });
    [SHIN_L, SHIN_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.shin, 0.034, 0.025, s, hide_tex);
        cube(k, [-0.035, -f.shin - f.sole, -0.05], [0.035, -f.shin + 0.02, 0.16], sw(kit::LEATHER));
    });
}

fn hunch(t: f32) -> Pose {
    let twitch = noise(snap(t, 23.0) as i32);
    let tilt = noise(snap(t + 9.0, 31.0) as i32 + 77);
    Pose {
        roll: 0.04 * (t * 0.05).sin(),
        torso: [0.28, 0.05 * (t * 0.03).sin(), 0.0],
        head: [-0.25, 0.45 * twitch, 0.32 * tilt],
        arm_l: [-0.15, 0.1, 0.2],
        arm_r: [-0.15, -0.1, -0.2],
        elbow_l: -0.25,
        elbow_r: -0.2,
        knee_l: 0.12,
        knee_r: 0.18,
        x: [0.08 + 0.12 * noise(snap(t, 13.0) as i32 + 5).max(0.0), 0.0, 0.0],
        ..Pose::default()
    }
}

fn walk(t: f32, period: f32, amp: f32) -> Pose {
    let hitch = noise(snap(t, 5.0) as i32 + 31).max(0.0) * 3.0;
    let ph = snap(t - hitch, 2.0) * TAU / period;
    let w = ph.sin();
    let base = hunch(t);
    Pose {
        roll: 0.09 * w.max(0.0) * amp,
        pelvis: [0.0, 0.08 * w, 0.0],
        torso: [0.3 + 0.05 * (2.0 * ph).cos(), -0.1 * w, 0.06 * w],
        arm_l: [-0.35 + 0.15 * w, 0.12, 0.2],
        arm_r: [-0.3 - 0.1 * w, -0.12, -0.2],
        hip_l: [-0.42 * w * amp, 0.04],
        hip_r: [0.25 * w * amp, 0.04],
        knee_l: 0.1 + 0.5 * amp * (ph + 0.6).cos().max(0.0),
        knee_r: 0.05,
        ankle_l: 0.0,
        skirt: [0.0, -0.1 * w, 0.05 * w],
        ..base
    }
}

fn pose(anim: LegendAnim, tic: u16) -> Pose {
    let t = tic as f32;
    match anim {
        LegendAnim::Idle => hunch(t),
        LegendAnim::Walk => walk(t, 44.0, 1.0),
        LegendAnim::Run => {
            let ph = snap(t, 2.0) * TAU / 22.0;
            let w = ph.sin();
            Pose {
                pitch: 0.32,
                torso: [0.25, -0.15 * w, 0.0],
                head: [-0.55, 0.15 * noise(snap(t, 4.0) as i32), 0.2 * noise(snap(t, 6.0) as i32 + 3)],
                arm_l: [-1.35 + 0.25 * w, 0.15, 0.1],
                arm_r: [-1.3 - 0.25 * w, -0.15, -0.1],
                elbow_l: -0.15,
                elbow_r: -0.2,
                hip_l: [-0.7 * w, 0.03],
                hip_r: [0.7 * w, 0.03],
                knee_l: 0.15 + 1.0 * (ph + 0.6).cos().max(0.0),
                knee_r: 0.15 + 1.0 * (ph + 0.6 + PI).cos().max(0.0),
                x: [0.35, 0.0, 0.0],
                ..Pose::default()
            }
        }
        LegendAnim::Sniff => {
            let jerk = noise(snap(t, 7.0) as i32 + 11);
            Pose {
                torso: [0.65, 0.2 * jerk, 0.0],
                head: [-0.7, 0.6 * jerk, 0.4 * noise(snap(t, 11.0) as i32)],
                x: [0.05 + 0.2 * (t * 1.7).sin().max(0.0), 0.0, 0.0],
                knee_l: 0.35,
                knee_r: 0.3,
                hip_l: [-0.3, 0.06],
                hip_r: [-0.25, 0.06],
                ..hunch(t)
            }
        }
        LegendAnim::Grab => {
            let k = ease(t / 6.0);
            Pose {
                torso: [0.3 * k + 0.1, 0.0, 0.0],
                head: [-0.35, 0.1 * noise(snap(t, 3.0) as i32), 0.0],
                arm_l: [-1.55 * k, 0.15 - 0.1 * k, 0.0],
                arm_r: [-1.5 * k, -0.15 + 0.1 * k, 0.0],
                elbow_l: -0.1,
                elbow_r: -0.1,
                hip_l: [-0.3 * k, 0.04],
                knee_l: 0.3 * k,
                x: [0.6 * k, 0.0, 0.0],
                ..hunch(t)
            }
        }
        LegendAnim::Stagger => {
            let w = (t * 1.1).sin();
            Pose {
                torso: [-0.35 + 0.05 * w, 0.2, 0.12 * w],
                head: [0.3, -0.6, 0.3 * w],
                arm_l: [-2.1, 0.3, 0.6],
                arm_r: [-1.9, -0.4, -0.5],
                elbow_l: -1.9,
                elbow_r: -1.7,
                hip_l: [-0.5, 0.1],
                hip_r: [0.2, 0.1],
                knee_l: 0.7,
                knee_r: 0.3,
                x: [0.45 + 0.1 * w, 0.0, 0.0],
                ..Pose::default()
            }
        }
        LegendAnim::Special => {
            let k = ease(t / 8.0);
            let shake = 0.06 * (t * 2.3).sin();
            Pose {
                torso: [-0.25 * k, 0.0, shake],
                head: [-0.6 * k, shake, shake],
                arm_l: [-0.3, 1.3 * k, 0.0],
                arm_r: [-0.3, -1.3 * k, 0.0],
                elbow_l: -0.4,
                elbow_r: -0.4,
                knee_l: 0.25,
                knee_r: 0.25,
                hip_l: [-0.1, 0.12],
                hip_r: [-0.1, 0.12],
                x: [0.75 * k, 0.0, 0.0],
                ..Pose::default()
            }
        }
    }
}

pub fn rig(x: f32, z: f32, heading: f32, anim: LegendAnim, tic: u16) -> Rig {
    let p = pose(anim, tic);
    let mut mats = skeleton::biped(&FRAME, &p, x, z, heading);
    hide(&mut mats[LAMP]);
    let hands = [mats[FORE_R], mats[FORE_L]];
    let pelvis = mats[PELVIS];
    let head = mats[HEAD];
    let r = rig_from(mats, hands, false, &pelvis, [0.0, 0.2, 0.1]);
    let eye = |side: f32| super::super::mat::point(&head, [side * 0.03, 0.205, 0.088]);
    Rig { eyes: Some([eye(1.0), eye(-1.0)]), eye_color: [0.95, 0.85, 0.4], ..r }
}
