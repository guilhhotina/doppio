use super::super::mat::{self, M4};
use super::heroes;
use super::skeleton::{self, Pose, ease, hide};
use super::{FORE_L, FORE_R, LAMP, Rig, rig_from};
use crate::sim::Anim;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

#[derive(Clone, Copy)]
struct Style {
    hunch: f32,
    stride: f32,
    swing: f32,
    sway: f32,
    spread: f32,
    tempo: f32,
}

const STYLES: [Style; 4] = [
    Style { hunch: 0.2, stride: 0.75, swing: 0.2, sway: 0.03, spread: 0.06, tempo: 1.1 },
    Style { hunch: 0.0, stride: 1.05, swing: 0.45, sway: 0.02, spread: 0.04, tempo: 1.2 },
    Style { hunch: 0.04, stride: 0.85, swing: 0.28, sway: 0.09, spread: 0.2, tempo: 0.92 },
    Style { hunch: 0.05, stride: 0.95, swing: 0.32, sway: 0.03, spread: 0.07, tempo: 1.0 },
];

const BIA: u8 = 3;

fn gait(s: &Style, ph: f32, amp: f32, knee: f32, arm: f32) -> Pose {
    let w = ph.sin();
    let a = amp * s.stride;
    Pose {
        roll: (ph).cos() * s.sway * amp,
        pelvis: [0.0, w * 0.1 * a, 0.0],
        torso: [s.hunch, -w * 0.16 * a, 0.0],
        head: [-s.hunch * 0.6, w * 0.06 * a, 0.0],
        arm_l: [w * arm * s.swing / 0.3, s.spread, 0.0],
        arm_r: [-w * arm * s.swing / 0.3, -s.spread, 0.0],
        elbow_l: -0.25 - 0.35 * amp * w.max(0.0),
        elbow_r: -0.25 - 0.35 * amp * (-w).max(0.0),
        hip_l: [-w * 0.45 * a, 0.0],
        hip_r: [w * 0.45 * a, 0.0],
        knee_l: 0.06 + knee * (ph + 0.6).cos().max(0.0),
        knee_r: 0.06 + knee * (ph + 0.6 + PI).cos().max(0.0),
        skirt: [0.0, w * 0.08 * a, 0.0],
        x: [0.15 * amp * (2.0 * ph).sin().abs(), 0.0, 0.1 * w * amp],
        ..Pose::default()
    }
}

fn idle(c: u8, s: &Style, t: f32) -> Pose {
    let breathe = (t * 0.11).sin();
    let base = Pose {
        torso: [s.hunch + 0.02 * breathe, 0.0, 0.0],
        head: [0.04 - s.hunch * 0.5, 0.25 * (t * 0.021).sin() * (t * 0.0073).cos(), 0.03 * (t * 0.05).sin()],
        arm_l: [0.0, s.spread, 0.0],
        arm_r: [0.0, -s.spread, 0.0],
        elbow_l: -0.12,
        elbow_r: -0.12,
        knee_l: 0.04,
        knee_r: 0.04,
        x: [0.05 * breathe, 0.0, 0.0],
        ..Pose::default()
    };
    match c {
        0 => Pose { arm_l: [-0.25, 0.05, -0.55], arm_r: [-0.25, -0.05, 0.55], elbow_l: -1.25, elbow_r: -1.15 + 0.06 * (t * 0.4).sin(), ..base },
        1 => {
            let shift = (t * 0.045).sin();
            Pose { pelvis: [0.0, 0.0, 0.05 * shift], torso: [0.0, 0.1 * shift, -0.05 * shift], hip_l: [0.0, 0.05 + 0.05 * shift], hip_r: [0.0, 0.05 - 0.05 * shift], knee_l: 0.04 + 0.12 * shift.max(0.0), knee_r: 0.04 + 0.12 * (-shift).max(0.0), ..base }
        }
        2 => Pose { arm_l: [0.1, 0.3, 0.0], arm_r: [0.1, -0.3, 0.0], elbow_l: -0.3, elbow_r: -0.3, hip_l: [0.0, 0.08], hip_r: [0.0, 0.08], torso: [s.hunch - 0.05 + 0.03 * breathe, 0.0, 0.0], ..base },
        _ => base,
    }
}

fn pose(c: u8, anim: Anim, tic: u16) -> Pose {
    let s = &STYLES[c as usize];
    let t = tic as f32;
    let rest = idle(c, s, t);
    match anim {
        Anim::Idle => rest,
        Anim::Walk => gait(s, t * TAU * s.tempo / 32.0, 1.0, 0.55, 0.32),
        Anim::Run => Pose { pitch: 0.18, ..gait(s, t * TAU * s.tempo / 20.0, 1.6, 1.15, 0.8) },
        Anim::Turn => Pose { head: [0.0, -0.2, 0.0], ..gait(s, t * TAU / 18.0, 0.35, 0.35, 0.06) },
        Anim::QuickTurn => {
            let k = (1.0 - t / 12.0).max(0.0);
            Pose { pelvis: [0.0, 0.5 * k, 0.0], torso: [s.hunch, 0.4 * k, 0.0], head: [0.0, 0.6 * k, 0.0], knee_l: 0.5 * k, knee_r: 0.5 * k, hip_l: [-0.25 * k, 0.0], hip_r: [-0.25 * k, 0.0], ..rest }
        }
        Anim::Reach => {
            let k = ease(t / 8.0);
            Pose { arm_r: [-1.3 * k, -0.1, 0.0], elbow_r: -0.15, torso: [s.hunch + 0.18 * k, 0.15 * k, 0.0], head: [0.2 * k, 0.0, 0.0], ..rest }
        }
        Anim::Crouch => {
            let k = ease(t / 8.0);
            Pose {
                torso: [s.hunch + 0.45 * k, 0.0, 0.0],
                head: [-0.3 * k, 0.0, 0.0],
                arm_l: [-0.5 * k, 0.15, 0.0],
                arm_r: [-0.5 * k, -0.15, 0.0],
                elbow_l: -0.8 * k,
                elbow_r: -0.8 * k,
                hip_l: [-1.5 * k, 0.12 * k],
                hip_r: [-1.5 * k, 0.12 * k],
                knee_l: 2.1 * k,
                knee_r: 2.1 * k,
                skirt: [-0.9 * k, 0.0, 0.0],
                ..rest
            }
        }
        Anim::Grabbed => {
            let w = (t * 0.9).sin();
            let v = (t * 1.3).cos();
            Pose {
                pitch: -0.25,
                torso: [-0.2 + 0.1 * w, 0.3 * v, 0.15 * w],
                head: [-0.4, 0.5 * w, 0.3 * v],
                arm_l: [-2.4 + 0.5 * w, 0.4, 0.0],
                arm_r: [-2.2 - 0.5 * v, -0.4, 0.0],
                elbow_l: -0.9 + 0.5 * v,
                elbow_r: -0.7 + 0.5 * w,
                hip_l: [-0.3 + 0.4 * w, 0.1],
                hip_r: [-0.2 - 0.4 * w, 0.1],
                knee_l: 0.6 + 0.4 * v,
                knee_r: 0.5 - 0.3 * v,
                x: [0.3 * w, 0.0, 0.4 * v],
                ..Pose::default()
            }
        }
        Anim::Faint => {
            let k = ease(t / 18.0);
            let limp = (1.0 - (t / 9.0 - 1.0).abs()).max(0.0);
            Pose {
                pitch: -FRAC_PI_2 * k,
                roll: 0.15 * k,
                torso: [0.25 * limp - 0.1 * k, 0.0, 0.1 * k],
                head: [0.3 * limp, 0.6 * k, 0.0],
                arm_l: [-0.4 * limp + 0.2 * k, 0.5 * k, 0.0],
                arm_r: [-0.2 * limp + 0.1 * k, -0.9 * k, 0.0],
                elbow_l: -0.4 * k,
                elbow_r: -0.2,
                hip_l: [-0.5 * limp - 0.2 * k, 0.05],
                hip_r: [-0.3 * limp, 0.08],
                knee_l: 1.0 * limp + 0.5 * k,
                knee_r: 0.8 * limp + 0.1 * k,
                x: [0.3 * k, 0.0, 0.0],
                ..Pose::default()
            }
        }
    }
}

fn level(m: &M4, swing: f32, elbow: f32) -> M4 {
    mat::mul(m, &mat::rot_x(-(swing + elbow)))
}

pub fn rig(c: u8, x: f32, z: f32, heading: f32, anim: Anim, tic: u16, lantern: bool, look: f32) -> Rig {
    let f = heroes::frame(c);
    let mut p = pose(c, anim, tic);
    p.head[1] += look;
    let free = !matches!(anim, Anim::Reach | Anim::Grabbed | Anim::Faint | Anim::Crouch);
    let sway = p.arm_l[0] * 0.15;
    match (c == BIA, lantern, free) {
        (true, true, true) => (p.arm_l, p.elbow_l) = ([-1.05 + sway * 0.3, -0.12, -0.25], -1.55),
        (true, false, true) => (p.arm_l, p.elbow_l) = ([-0.25 + sway, 0.1, 0.0], -1.15),
        (false, true, true) => (p.arm_l, p.elbow_l) = ([-0.55 + sway, 0.14, 0.0], -0.85),
        _ => {}
    }
    let mut mats = skeleton::biped(f, &p, x, z, heading);
    let lamp = level(&mats[LAMP], p.arm_l[0], p.elbow_l);
    mats[LAMP] = lamp;
    let grip = |seg: usize, swing: f32, elbow: f32, dz: f32| level(&mat::mul(&mats[seg], &mat::translate([0.0, -f.hand(), dz])), swing, elbow);
    let hands = [grip(FORE_R, p.arm_r[0], p.elbow_r, 0.0), grip(FORE_L, p.arm_l[0], p.elbow_l, 0.05)];
    let shown = (lantern || c == BIA) && !matches!(anim, Anim::Faint);
    if !shown {
        hide(&mut mats[LAMP]);
    }
    rig_from(mats, hands, lantern && shown, &lamp, [0.0, 0.16, 0.1])
}
