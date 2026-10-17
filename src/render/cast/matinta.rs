use super::super::mat;
use super::heroes::{card, hand, head, limb, loft, sw, tiled, tube};
use super::kit::{self, Kit, Uv, ring};
use super::skeleton::{self, Frame, Pose, ease, hide, noise, snap};
use super::{ARM_L, ARM_R, FOOT_L, FOOT_R, FORE_L, FORE_R, HEAD, LAMP, PELVIS, Rig, SHIN_L, SHIN_R, SKIRT, TORSO, X, locomotion, rig_from};
use crate::sim::LegendAnim;
use std::f32::consts::TAU;

pub const FRAME: Frame = Frame {
    hip_x: 0.075,
    waist: 0.05,
    chest: 0.36,
    shoulder_x: 0.22,
    neck: 0.4,
    upper: 0.28,
    fore: 0.26,
    thigh: 0.38,
    shin: 0.37,
    meta: 0.0,
    sole: 0.05,
    hem: 0.79,
    back: 0.21,
    x_at: (HEAD, [0.0, 0.2, 0.0]),
};

pub fn build(k: &mut Kit) {
    let f = &FRAME;
    let s8 = k.n(8, 4);
    let black = tiled("cast_preto", 4.0);
    k.seg(SKIRT);
    super::heroes::garment(k, super::cloth::SKIRTS, s8, black);
    k.seg(TORSO);
    loft(k, &[ring(-0.02, 0.13, 0.1, 0.0), ring(0.2, 0.15, 0.12, -0.01), ring(0.34, 0.16, 0.11, -0.02), ring(0.41, 0.07, 0.06, 0.0)], s8, black, (0, 0), false);
    loft(k, &[ring(0.43, 0.1, 0.09, -0.02), ring(0.35, 0.22, 0.18, -0.03), ring(0.12, 0.21, 0.2, -0.05)], s8, black, (0, 0), false);
    k.seg(HEAD);
    tube(k, -0.02, 0.08, 0.036, 0.032, k.n(5, 3), sw(kit::SKIN_PALE));
    head(k, "cast_matinta", 0.035, 0.26, 0.086, 0.1, 0.012);
    let gap = if k.hi { (3, 5) } else { (1, 3) };
    loft(k, &[ring(0.29, 0.0, 0.0, -0.01), ring(0.27, 0.075, 0.085, -0.015), ring(0.18, 0.115, 0.13, -0.025), ring(0.08, 0.12, 0.13, -0.045)], s8, black, gap, false);
    k.seg(X);
    let hair = ("cast_matinta", Uv::Wrap([0.02, 0.1, 0.2, 0.9]));
    [-1.0, 1.0].iter().for_each(|&sd| {
        card(k, [[sd * 0.1, -0.68, 0.14], [sd * 0.2, -0.64, 0.08], [sd * 0.13, 0.0, -0.01], [sd * 0.07, 0.0, 0.07]], hair, true);
    });
    if k.hi {
        card(k, [[-0.07, -0.42, 0.155], [0.07, -0.42, 0.155], [0.06, -0.02, 0.09], [-0.06, -0.02, 0.09]], hair, true);
    }
    let s = k.n(6, 3);
    [ARM_L, ARM_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.upper + 0.02, 0.05, 0.045, s, black);
    });
    [FORE_L, FORE_R].iter().for_each(|&seg| {
        k.seg(seg);
        if k.hi {
            limb(k, 0.08, 0.047, 0.05, s, black);
        }
        tube(k, -0.06, -f.fore, 0.026, 0.02, s, sw(kit::SKIN_PALE));
        hand(k, f.fore, 0.06, kit::SKIN_PALE);
    });
    [SHIN_L, SHIN_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.shin, 0.03, 0.025, k.n(4, 3), sw(kit::SKIN_PALE));
    });
    [FOOT_L, FOOT_R].iter().for_each(|&seg| {
        k.seg(seg);
        if k.hi {
            super::heroes::cube(k, [-0.035, -f.sole, -0.05], [0.035, 0.02, 0.14], sw(kit::HOOF));
        }
    });
}

fn hunched(t: f32) -> Pose {
    let rock = (t * 0.06).sin();
    let fidget = noise(snap(t, 17.0) as i32 + 4);
    Pose {
        torso: [0.55 + 0.03 * rock, 0.0, 0.04 * rock],
        head: [-0.55, 0.3 * fidget, 0.25 * noise(snap(t + 5.0, 29.0) as i32)],
        arm_l: [-0.35, 0.1, -0.3],
        arm_r: [-0.4, -0.1, 0.3],
        elbow_l: -0.7 + 0.08 * (t * 0.7).sin(),
        elbow_r: -0.8 + 0.08 * (t * 0.9).cos(),
        hip_l: [-0.15, 0.05],
        hip_r: [-0.15, 0.05],
        knee_l: 0.3,
        knee_r: 0.3,
        x: [0.25, 0.0, 0.05 * rock],
        ..Pose::default()
    }
}

fn shuffle(t: f32, period: f32, amp: f32, low: f32) -> Pose {
    let ph = t * TAU / period;
    let w = ph.sin();
    let base = hunched(t);
    Pose {
        roll: 0.05 * w,
        torso: [0.55 + low, 0.12 * w, 0.0],
        head: [-0.55 - low, -0.1 * w, base.head[2]],
        hip_l: [-0.15 - 0.3 * w * amp, 0.05],
        hip_r: [-0.15 + 0.3 * w * amp, 0.05],
        knee_l: 0.3 + 0.45 * amp * (ph + 0.6).cos().max(0.0),
        knee_r: 0.3 + 0.45 * amp * (ph + 0.6 + std::f32::consts::PI).cos().max(0.0),
        skirt: [0.05 * amp, 0.1 * w, 0.06 * w],
        x: [0.3 + 0.1 * amp, 0.0, 0.1 * w],
        ..base
    }
}

fn pose(anim: LegendAnim, tic: u16) -> Pose {
    let t = tic as f32;
    match anim {
        LegendAnim::Idle => hunched(t),
        LegendAnim::Walk => shuffle(t, PERIOD[0], 0.6, 0.0),
        LegendAnim::Run => Pose { arm_l: [0.6, 0.35, 0.0], arm_r: [0.6, -0.35, 0.0], elbow_l: -0.3, elbow_r: -0.3, ..shuffle(t, PERIOD[1], 1.0, 0.2) },
        LegendAnim::Sniff => {
            let tilt = (t * 0.08).sin();
            Pose { head: [-0.45, 0.4 * noise(snap(t, 19.0) as i32), 0.6 * tilt], torso: [0.7, 0.0, 0.1 * tilt], ..hunched(t) }
        }
        LegendAnim::Grab => {
            let k = ease(t / 6.0);
            Pose {
                torso: [0.55 - 0.3 * k, 0.0, 0.0],
                head: [-0.5 + 0.2 * k, 0.0, 0.0],
                arm_l: [-0.4 - 1.5 * k, 0.25, 0.0],
                arm_r: [-0.4 - 1.6 * k, -0.25, 0.0],
                elbow_l: -0.7 + 0.5 * k,
                elbow_r: -0.7 + 0.5 * k,
                x: [0.3, 0.0, 0.0],
                ..hunched(t)
            }
        }
        LegendAnim::Stagger => {
            let w = (t * 0.9).sin();
            Pose {
                torso: [0.85, 0.3 * w, 0.0],
                head: [0.0, -0.5, 0.2],
                arm_l: [-1.6, 0.2, 0.7],
                arm_r: [-1.7, -0.2, -0.7],
                elbow_l: -1.9,
                elbow_r: -2.0,
                knee_l: 0.6,
                knee_r: 0.5,
                hip_l: [-0.45, 0.05],
                hip_r: [-0.4, 0.05],
                skirt: [0.05, 0.0, 0.06 * w],
                x: [0.4, 0.0, 0.0],
                ..Pose::default()
            }
        }
        LegendAnim::Special => {
            let k = ease(t / 10.0);
            let trill = 0.03 * (t * 1.9).sin();
            Pose {
                torso: [0.55 - 0.3 * k, 0.0, 0.0],
                head: [-0.55 - 0.35 * k, trill, 0.3 * k],
                arm_r: [-1.2 * k - 0.3, -0.05, 0.5 * k],
                elbow_r: -0.8 - 1.5 * k,
                x: [0.2 - 0.15 * k, 0.0, 0.0],
                ..hunched(t)
            }
        }
        LegendAnim::Alert => {
            let k = ease(t / 3.0);
            let twitch = noise(snap(t, 4.0) as i32 + 61);
            Pose {
                torso: [0.55 - 0.25 * k, 0.0, 0.0],
                head: [-0.35 - 0.2 * k, 0.12 * twitch * k, 0.45 * k],
                arm_l: [-0.7 * k, 0.55 * k, -0.3],
                arm_r: [-0.7 * k, -0.55 * k, 0.3],
                elbow_l: -0.7 - 0.6 * k,
                elbow_r: -0.8 - 0.6 * k,
                x: [0.1 + 0.15 * k, 0.0, 0.08 * twitch],
                ..hunched(t)
            }
        }
    }
}

pub const PERIOD: [f32; 2] = [16.0, 12.0];

pub fn rig(x: f32, z: f32, heading: f32, anim: LegendAnim, tic: u16, travel: [f32; 2]) -> Rig {
    let mut p = pose(anim, tic);
    p.ankle_l = -p.hip_l[0] - p.knee_l;
    p.ankle_r = -p.hip_r[0] - p.knee_r;
    let mut mats = skeleton::biped(&FRAME, &p, x, z, heading);
    if matches!(anim, LegendAnim::Walk | LegendAnim::Run) {
        let run = matches!(anim, LegendAnim::Run);
        let gait = if run { locomotion::Gait { period: PERIOD[1], duty: 0.45, clearance: 0.05, sink: 0.93, limp: 0.0 } } else { locomotion::Gait { period: PERIOD[0], duty: 0.55, clearance: 0.03, sink: 0.95, limp: 0.0 } };
        locomotion::biped(&mut mats, &FRAME, [x, z, heading], tic as f32, gait, travel);
    }
    hide(&mut mats[LAMP]);
    let hands = [mats[FORE_R], mats[FORE_L]];
    let pelvis = mats[PELVIS];
    let head = mats[HEAD];
    let r = rig_from(mats, hands, false, &pelvis, [0.0, 0.16, 0.1]);
    let eye = |side: f32| mat::point(&head, [side * 0.03, 0.155, 0.09]);
    Rig { eyes: Some([eye(1.0), eye(-1.0)]), eye_color: [0.55, 0.75, 0.6], ..r }
}
