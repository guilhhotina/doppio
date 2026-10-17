use super::heroes::{fingers, loft, spike, sw, tiled, tube};
use super::kit::{self, Kit, Ring, Uv, ring};
use super::skeleton::{self, Frame, Pose, ease, hide, noise, snap};
use super::{ARM_L, ARM_R, FOOT_L, FOOT_R, FORE_L, FORE_R, HEAD, LAMP, PELVIS, Rig, SHIN_L, SHIN_R, SKIRT, THIGH_L, THIGH_R, TORSO, X, locomotion, rig_from};
use crate::sim::LegendAnim;
use std::f32::consts::TAU;

const JAW: [f32; 2] = [0.128, -0.01];

const SKULL: [Ring; 8] = [
    ring(0.345, 0.0, 0.0, -0.01),
    ring(0.325, 0.064, 0.074, -0.005),
    ring(0.27, 0.084, 0.1, 0.005),
    ring(0.215, 0.074, 0.098, 0.012),
    ring(0.165, 0.07, 0.092, 0.014),
    ring(0.125, 0.074, 0.088, 0.016),
    ring(0.075, 0.056, 0.074, 0.022),
    ring(0.045, 0.032, 0.045, 0.03),
];

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
    let suit = tiled("cast_preto", 3.0);
    let rags = tiled("cast_trapo", 3.5);
    let hide_tex = tiled("cast_couro", 4.0);
    let ribs = ("cast_costela", Uv::Wrap([0.0005, 1.0, 0.9995, 0.0]));
    let bone = sw(kit::BONE);
    k.seg(PELVIS);
    loft(k, &[ring(-0.17, 0.115, 0.088, 0.0), ring(0.07, 0.105, 0.075, 0.0)], s8, suit, (0, 0), false);
    k.seg(SKIRT);
    let strips = k.n(6, 0);
    (0..strips).for_each(|i| {
        let a = TAU * i as f32 / strips as f32 + 0.4;
        let (s, c) = a.sin_cos();
        let len = 0.22 + 0.14 * noise(i as i32 * 7 + 3).abs();
        let mat = if i % 2 == 0 { rags } else { suit };
        spike(k, [s * 0.105, 0.03, c * 0.08], [s * 0.14, -len, c * 0.11], 0.045, mat);
    });
    k.seg(TORSO);
    loft(
        k,
        &[
            ring(-0.02, 0.08, 0.056, 0.0),
            ring(0.08, 0.05, 0.038, -0.022),
            ring(0.16, 0.054, 0.042, -0.025),
            ring(0.24, 0.148, 0.11, 0.022),
            ring(0.41, 0.162, 0.102, 0.012),
            ring(0.51, 0.052, 0.046, 0.0),
        ],
        s8,
        ribs,
        (0, 0),
        false,
    );
    if k.hi {
        [-1.0, 1.0].iter().for_each(|&sd| spike(k, [sd * 0.012, 0.455, 0.075], [sd * 0.175, 0.475, 0.0], 0.016, bone));
        (0..4).for_each(|i| {
            let y = 0.16 + 0.08 * i as f32;
            let z = -0.065 - 0.012 * (i == 2) as u8 as f32;
            spike(k, [0.0, y, z + 0.02], [0.0, y - 0.03, z - 0.035], 0.014, bone);
        });
    }
    let gap = if k.hi { (2, 6) } else { (1, 3) };
    loft(
        k,
        &[
            ring(0.53, 0.072, 0.068, 0.0),
            ring(0.45, 0.19, 0.115, 0.0),
            ring(0.32, 0.175, 0.118, -0.008),
        ],
        s8,
        suit,
        gap,
        false,
    );
    if k.hi {
        (0..4).for_each(|i| {
            let a = -2.2 + 1.5 * i as f32;
            let (s, c) = a.sin_cos();
            let mat = if i % 2 == 0 { suit } else { rags };
            spike(k, [s * 0.16, 0.32, c * 0.11 - 0.01], [s * 0.18, 0.05 - 0.08 * noise(i * 3 + 1).abs(), c * 0.13 - 0.02], 0.038, mat);
        });
    }
    k.seg(HEAD);
    loft(k, &[ring(-0.02, 0.032, 0.03, 0.0), ring(0.1, 0.026, 0.024, 0.0)], k.n(5, 3), hide_tex, (0, 0), false);
    let (top, chin) = (SKULL[0].y, SKULL[7].y);
    let face = |v0: f32, v1: f32| ("cast_corpo", Uv::Wrap([0.0005, v0, 0.9995, v1]));
    match k.hi {
        true => {
            let split = (top - SKULL[5].y) / (top - chin);
            loft(k, &SKULL[..=5], 8, face(0.0, split), (0, 0), false);
            k.glow(1.0);
            [-1.0, 1.0].iter().for_each(|&sd| {
                let x = sd * 0.032;
                k.tex("cast_misc").card([[x - 0.01, 0.198, 0.095], [x + 0.01, 0.198, 0.095], [x + 0.01, 0.214, 0.095], [x - 0.01, 0.214, 0.095]], kit::swatch(kit::BRASS), false);
            });
            k.glow(0.0);
            k.seg(X);
            let jaw: Vec<Ring> = SKULL[5..].iter().map(|r| ring(r.y - JAW[0], r.rx, r.rz, r.dz - JAW[1])).collect();
            loft(k, &jaw, 8, face(split, 1.0), (0, 0), false);
        }
        false => loft(k, &[SKULL[0], SKULL[2], SKULL[5], SKULL[7]], 4, face(0.0, 1.0), (0, 0), false),
    }
    let s = k.n(5, 3);
    [ARM_L, ARM_R].iter().for_each(|&seg| {
        k.seg(seg);
        loft(k, &[ring(0.03, 0.055, 0.055, 0.0), ring(-0.12, 0.048, 0.048, 0.0), ring(-0.21, 0.044, 0.044, 0.0)], s, suit, (0, 0), false);
        tube(k, -0.18, -f.upper - 0.02, 0.026, 0.03, s, hide_tex);
    });
    [FORE_L, FORE_R].iter().for_each(|&seg| {
        k.seg(seg);
        loft(k, &[ring(0.02, 0.032, 0.03, -0.01), ring(-0.08, 0.022, 0.02, 0.0), ring(-f.fore, 0.018, 0.016, 0.0)], s, rags, (0, 0), false);
        tube(k, -f.fore + 0.01, -f.fore - 0.06, 0.024, 0.03, k.n(4, 3), hide_tex);
        fingers(k, -f.fore - 0.055, 0.21, 0.068, 0.46, 0.01, bone);
    });
    [THIGH_L, THIGH_R].iter().for_each(|&seg| {
        k.seg(seg);
        loft(k, &[ring(0.03, 0.068, 0.062, 0.0), ring(-0.3, 0.052, 0.052, 0.0), ring(-f.thigh + 0.02, 0.058, 0.052, 0.01)], s, suit, (0, 0), false);
        if k.hi {
            spike(k, [0.0, -f.thigh + 0.06, 0.042], [0.015, -f.thigh - 0.15, 0.055], 0.042, rags);
        }
    });
    [SHIN_L, SHIN_R].iter().for_each(|&seg| {
        k.seg(seg);
        loft(k, &[ring(0.03, 0.042, 0.04, 0.01), ring(-0.04, 0.03, 0.028, 0.0), ring(-f.shin, 0.022, 0.02, 0.0)], s, hide_tex, (0, 0), false);
    });
    [FOOT_L, FOOT_R].iter().for_each(|&seg| {
        k.seg(seg);
        loft(
            k,
            &[
                ring(-f.sole + 0.06, 0.026, 0.028, 0.0),
                ring(-f.sole + 0.012, 0.038, 0.055, 0.04),
                ring(-f.sole + 0.004, 0.036, 0.034, 0.13),
                ring(-f.sole, 0.028, 0.012, 0.2),
            ],
            k.n(4, 3),
            bone,
            (0, 0),
            false,
        );
    });
}

fn hunch(t: f32) -> Pose {
    let twitch = noise(snap(t, 23.0) as i32);
    let tilt = noise(snap(t + 9.0, 31.0) as i32 + 77);
    Pose {
        roll: 0.04 * (t * 0.05).sin(),
        pelvis: [-0.14, 0.0, 0.0],
        torso: [0.46, 0.05 * (t * 0.03).sin(), 0.0],
        head: [-0.34, 0.45 * twitch, 0.32 * tilt],
        arm_l: [-0.32, 0.1, 0.18],
        arm_r: [-0.32, -0.1, -0.18],
        elbow_l: -0.35,
        elbow_r: -0.3,
        hip_l: [-0.22, 0.05],
        hip_r: [-0.18, 0.06],
        knee_l: 0.36,
        knee_r: 0.42,
        x: [0.14 + 0.14 * noise(snap(t, 13.0) as i32 + 5).max(0.0), 0.0, 0.0],
        ..Pose::default()
    }
}

fn walk(t: f32, period: f32, amp: f32) -> Pose {
    let hitch = noise(snap(t, 5.0) as i32 + 31).max(0.0) * 3.0;
    let ph = snap(t - hitch, 2.0) * TAU / period;
    let w = ph.cos();
    let lurch = (1.0 - (t / period + 0.5).fract() / 0.18).max(0.0);
    let base = hunch(t);
    Pose {
        roll: -0.14 * lurch * amp + 0.05 * w,
        pelvis: [-0.12, -0.12 * w, 0.0],
        torso: [0.46 + 0.18 * lurch, 0.14 * w, -0.1 * lurch],
        head: [base.head[0] - 0.22 * lurch, base.head[1], base.head[2] + 0.2 * lurch],
        arm_l: [-0.45 + 0.28 * w, 0.12, 0.2],
        arm_r: [-0.35 + 0.35 * lurch, -0.12, -0.32],
        elbow_l: -0.3 - 0.2 * (-w).max(0.0),
        elbow_r: -0.15,
        skirt: [0.06 * lurch, -0.1 * w, 0.08 * lurch],
        x: [base.x[0] + 0.22 * lurch, 0.0, 0.0],
        ..base
    }
}

fn pose(anim: LegendAnim, tic: u16) -> Pose {
    let t = tic as f32;
    match anim {
        LegendAnim::Idle => hunch(t),
        LegendAnim::Walk => walk(t, PERIOD[0], 1.0),
        LegendAnim::Run => {
            let ph = snap(t, 2.0) * TAU / PERIOD[1];
            let w = ph.cos();
            let bob = (2.0 * ph).cos();
            Pose {
                pitch: 0.3,
                roll: 0.07 * w,
                torso: [0.28 + 0.07 * bob, -0.2 * w, 0.06 * w],
                head: [-0.52 - 0.06 * bob, 0.15 * noise(snap(t, 4.0) as i32), 0.2 * noise(snap(t, 6.0) as i32 + 3)],
                arm_l: [-1.25 + 0.5 * w, 0.2, 0.15],
                arm_r: [-1.2 - 0.5 * w, -0.2, -0.15],
                elbow_l: -0.25 - 0.35 * (-w).max(0.0),
                elbow_r: -0.25 - 0.35 * w.max(0.0),
                hip_l: [-0.7 * w, 0.03],
                hip_r: [0.7 * w, 0.03],
                knee_l: 0.15,
                knee_r: 0.15,
                x: [0.38 + 0.2 * bob.max(0.0), 0.0, 0.0],
                ..Pose::default()
            }
        }
        LegendAnim::Sniff => {
            let jerk = noise(snap(t, 7.0) as i32 + 11);
            Pose {
                torso: [0.65, 0.22 * jerk, 0.0],
                head: [-0.7, 0.62 * jerk, 0.4 * noise(snap(t, 11.0) as i32)],
                x: [0.05 + 0.22 * (t * 1.7).sin().max(0.0), 0.0, 0.0],
                knee_l: 0.35,
                knee_r: 0.3,
                hip_l: [-0.3, 0.06],
                hip_r: [-0.25, 0.06],
                ..hunch(t)
            }
        }
        LegendAnim::Grab => {
            let wind = ease(t / 2.0) * (1.0 - ease((t - 2.0) / 3.0));
            let k = ease((t - 1.5) / 5.0);
            Pose {
                torso: [0.1 - 0.2 * wind + 0.38 * k, 0.0, 0.0],
                head: [-0.35 + 0.2 * wind, 0.1 * noise(snap(t, 3.0) as i32), 0.0],
                arm_l: [-1.4 * wind - 1.65 * k, 0.45 * wind + 0.18 * (1.0 - k), 0.2 * k],
                arm_r: [-1.35 * wind - 1.6 * k, -0.45 * wind - 0.18 * (1.0 - k), -0.2 * k],
                elbow_l: -0.5 * wind - 0.15,
                elbow_r: -0.5 * wind - 0.15,
                hip_l: [-0.32 * k, 0.04],
                knee_l: 0.32 * k,
                x: [0.68 * (wind + k).min(1.0), 0.0, 0.0],
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
        LegendAnim::Alert => {
            let snapped = 1.0 - (t / 2.0).min(1.0);
            let tremor = noise(snap(t, 2.0) as i32 + 17);
            Pose {
                torso: [0.12 - 0.15 * snapped, 0.04 * tremor, 0.0],
                head: [-0.15 + 0.3 * snapped, 0.08 * tremor, 0.35 + 0.05 * tremor],
                arm_l: [-0.95 - 0.1 * tremor, 0.28, 0.3],
                arm_r: [-0.9 + 0.1 * tremor, -0.28, -0.3],
                elbow_l: -0.7,
                elbow_r: -0.75,
                hip_l: [-0.15, 0.08],
                hip_r: [0.05, 0.08],
                knee_l: 0.3,
                knee_r: 0.1,
                x: [0.55 + 0.15 * tremor.abs(), 0.0, 0.0],
                ..hunch(t)
            }
        }
        LegendAnim::Special => {
            let k = ease(t / 7.0);
            let shake = 0.07 * (t * 2.3).sin();
            Pose {
                torso: [-0.28 * k, 0.0, shake],
                head: [-0.58 * k, shake, shake],
                arm_l: [-1.55 * k - 0.2, 0.55 * k + 0.1, 0.25 * k],
                arm_r: [-1.5 * k - 0.2, -0.55 * k - 0.1, -0.25 * k],
                elbow_l: -0.65 * k - 0.2,
                elbow_r: -0.6 * k - 0.2,
                knee_l: 0.25,
                knee_r: 0.25,
                hip_l: [-0.12, 0.12],
                hip_r: [-0.12, 0.12],
                x: [0.8 * k, 0.0, 0.0],
                ..Pose::default()
            }
        }
    }
}

pub const PERIOD: [f32; 2] = [40.0, 22.0];

pub fn rig(x: f32, z: f32, heading: f32, anim: LegendAnim, tic: u16, travel: [f32; 2]) -> Rig {
    let mut p = pose(anim, tic);
    p.ankle_l = -p.hip_l[0] - p.knee_l - p.pitch;
    p.ankle_r = -p.hip_r[0] - p.knee_r - p.pitch;
    let mut mats = skeleton::biped(&FRAME, &p, x, z, heading);
    if matches!(anim, LegendAnim::Walk | LegendAnim::Run) {
        let gait = match anim {
            LegendAnim::Run => locomotion::Gait { period: PERIOD[1], duty: 0.46, clearance: 0.1, sink: 0.95, limp: 0.5 },
            _ => locomotion::Gait { period: PERIOD[0], duty: 0.64, clearance: 0.07, sink: 0.995, limp: 0.85 },
        };
        locomotion::biped(&mut mats, &FRAME, [x, z, heading], tic as f32, gait, travel);
    }
    hide(&mut mats[LAMP]);
    let hands = [mats[FORE_R], mats[FORE_L]];
    let pelvis = mats[PELVIS];
    let head = mats[HEAD];
    let r = rig_from(mats, hands, false, &pelvis, [0.0, 0.2, 0.1]);
    let eye = |side: f32| super::super::mat::point(&head, [side * 0.03, 0.205, 0.088]);
    Rig { eyes: Some([eye(1.0), eye(-1.0)]), eye_color: [0.6, 0.48, 0.18], ..r }
}
