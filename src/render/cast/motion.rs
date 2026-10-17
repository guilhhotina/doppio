use super::super::mat::{self, M4};
use super::locomotion::{self, Gait};
use super::skeleton::{self, Pose, ease, hide};
use super::{FORE_L, FORE_R, LAMP, Rig, SKIRT, TORSO, heroes, rig_from};
use crate::sim::Anim;
use std::f32::consts::{FRAC_PI_2, TAU};

#[derive(Clone, Copy)]
pub struct Style {
    hunch: f32,
    swing: f32,
    sway: f32,
    spread: f32,
    pub walk: f32,
    pub run: f32,
    duty: [f32; 2],
    clearance: f32,
}

pub const STYLES: [Style; 4] = [
    Style { hunch: 0.16, swing: 0.22, sway: 0.035, spread: 0.18, walk: 24.0, run: 15.0, duty: [0.6, 0.42], clearance: 0.07 },
    Style { hunch: 0.0, swing: 0.45, sway: 0.02, spread: 0.12, walk: 18.0, run: 13.0, duty: [0.6, 0.42], clearance: 0.1 },
    Style { hunch: 0.04, swing: 0.3, sway: 0.05, spread: 0.24, walk: 28.0, run: 20.0, duty: [0.62, 0.45], clearance: 0.12 },
    Style { hunch: 0.05, swing: 0.34, sway: 0.03, spread: 0.14, walk: 26.0, run: 18.0, duty: [0.6, 0.42], clearance: 0.13 },
];

const BIA: u8 = 3;
const SKIRTED: u8 = 2;

fn gait(s: &Style, ph: f32, amp: f32, lean: f32) -> Pose {
    let w = ph.cos();
    let bob = (2.0 * ph).cos();
    let pump = (amp - 1.0).max(0.0);
    let spread = s.spread * (1.0 - 0.5 * pump);
    Pose {
        roll: w * s.sway * amp,
        pelvis: [0.0, -w * 0.12 * amp, 0.0],
        torso: [s.hunch + lean + 0.02 * amp * bob, w * 0.14 * amp, -w * 0.5 * s.sway * amp],
        head: [-s.hunch * 0.6 - lean * 0.7, -w * 0.05 * amp, w * 0.4 * s.sway * amp],
        arm_l: [w * amp * s.swing * 1.6, spread, 0.0],
        arm_r: [-w * amp * s.swing * 1.6, -spread, 0.0],
        elbow_l: -0.25 - 1.2 * pump - 0.45 * amp * (-w).max(0.0),
        elbow_r: -0.25 - 1.2 * pump - 0.45 * amp * w.max(0.0),
        skirt: [0.05 * lean.max(0.0) * 4.0 + 0.03 * amp * bob, -w * 0.06 * amp, w * 0.05 * amp],
        x: [0.12 * amp * bob.max(0.0), 0.0, 0.08 * w * amp],
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
        skirt: [0.0, 0.02 * (t * 0.05).sin(), 0.0],
        x: [0.05 * breathe, 0.0, 0.0],
        ..Pose::default()
    };
    match c {
        0 => Pose { arm_l: [-0.28, 0.12, -0.45], arm_r: [-0.28, -0.12, 0.45], elbow_l: -1.2, elbow_r: -1.12 + 0.05 * (t * 0.4).sin(), ..base },
        1 => {
            let shift = (t * 0.045).sin();
            Pose { pelvis: [0.0, 0.0, 0.05 * shift], torso: [0.0, 0.1 * shift, -0.05 * shift], hip_l: [0.0, 0.05 + 0.05 * shift], hip_r: [0.0, 0.05 - 0.05 * shift], knee_l: 0.04 + 0.12 * shift.max(0.0), knee_r: 0.04 + 0.12 * (-shift).max(0.0), ..base }
        }
        2 => Pose { arm_l: [0.1, 0.3, 0.0], arm_r: [0.1, -0.3, 0.0], elbow_l: -0.3, elbow_r: -0.3, hip_l: [0.0, 0.08], hip_r: [0.0, 0.08], torso: [s.hunch - 0.05 + 0.03 * breathe, 0.0, 0.0], ..base },
        _ => base,
    }
}

fn crouch(c: u8, s: &Style, k: f32, rest: Pose) -> Pose {
    match c < SKIRTED {
        true => Pose {
            torso: [s.hunch + 0.5 * k, 0.0, 0.0],
            head: [-0.35 * k, 0.0, 0.0],
            arm_l: [-0.55 * k, 0.15, -0.1],
            arm_r: [-0.55 * k, -0.15, 0.1],
            elbow_l: -0.9 * k,
            elbow_r: -0.9 * k,
            hip_l: [-0.12 * k, 0.04],
            hip_r: [-0.12 * k, 0.04],
            knee_l: 1.75 * k,
            knee_r: 1.75 * k,
            skirt: [0.25 * k, 0.0, 0.0],
            ..rest
        },
        false => Pose {
            torso: [s.hunch + 0.45 * k, 0.0, 0.0],
            head: [-0.3 * k, 0.0, 0.0],
            arm_l: [-0.8 * k, 0.2, 0.0],
            arm_r: [-0.8 * k, -0.2, 0.0],
            elbow_l: -1.0 * k,
            elbow_r: -1.0 * k,
            hip_l: [-1.5 * k, 0.12 * k],
            hip_r: [-1.5 * k, 0.12 * k],
            knee_l: 2.1 * k,
            knee_r: 2.1 * k,
            ..rest
        },
    }
}

fn pose(c: u8, anim: Anim, tic: u16, travel: [f32; 2]) -> Pose {
    let s = &STYLES[c as usize];
    let t = tic as f32;
    let rest = idle(c, s, t);
    let lean = (travel[0] * 0.035).clamp(-0.06, 0.1);
    match anim {
        Anim::Idle => rest,
        Anim::Walk => gait(s, t * TAU / s.walk, (travel[0].abs() / 1.3).clamp(0.4, 1.0), lean),
        Anim::Run => gait(s, t * TAU / s.run, 1.6 * (travel[0].abs() / 2.6).clamp(0.6, 1.0), lean + 0.12),
        Anim::Turn => Pose { head: [0.0, -0.25 * travel[1].signum(), 0.0], ..gait(s, t * TAU / 18.0, 0.3, 0.0) },
        Anim::QuickTurn => {
            let k = (1.0 - t / 12.0).max(0.0);
            Pose { pelvis: [0.0, -0.25 * k, 0.0], torso: [s.hunch + 0.1 * k, 0.4 * k, 0.0], head: [0.0, 0.6 * k, 0.0], ..gait(s, t * TAU / 12.0, 0.4, 0.0) }
        }
        Anim::Reach => {
            let k = ease(t / 8.0);
            Pose { arm_r: [-1.3 * k, -0.1, 0.0], elbow_r: -0.15, torso: [s.hunch + 0.18 * k, 0.15 * k, 0.0], head: [0.2 * k, 0.0, 0.0], ..rest }
        }
        Anim::Crouch => crouch(c, s, ease(t / 8.0), rest),
        Anim::Grabbed => {
            let w = (t * 0.9).sin();
            let v = (t * 1.3).cos();
            let kick = if c < SKIRTED { 0.45 } else { 1.0 };
            Pose {
                pitch: -0.25,
                torso: [-0.2 + 0.1 * w, 0.3 * v, 0.15 * w],
                head: [-0.4, 0.5 * w, 0.3 * v],
                arm_l: [-2.4 + 0.5 * w, 0.4, 0.0],
                arm_r: [-2.2 - 0.5 * v, -0.4, 0.0],
                elbow_l: -0.9 + 0.5 * v,
                elbow_r: -0.7 + 0.5 * w,
                hip_l: [(-0.3 + 0.4 * w) * kick, 0.1],
                hip_r: [(-0.2 - 0.4 * w) * kick, 0.1],
                knee_l: (0.6 + 0.4 * v) * kick,
                knee_r: (0.5 - 0.3 * v) * kick,
                skirt: [0.1 * v, 0.1 * w, 0.06 * v],
                x: [0.3 * w, 0.0, 0.4 * v],
                ..Pose::default()
            }
        }
        Anim::Faint => {
            let buckle = ease(t / 7.0);
            let k = ease((t - 5.0) / 12.0);
            let sag = buckle * (1.0 - k);
            let settle = ease((t - 17.0) / 14.0);
            let bounce = 0.07 * (1.0 - settle) * ((t - 17.0) * 0.8).sin() * (t > 17.0) as u8 as f32;
            Pose {
                pitch: -FRAC_PI_2 * k + bounce,
                roll: 0.06 * k,
                torso: [0.3 * sag - 0.05 * k, 0.1 * sag, 0.04 * k],
                head: [0.45 * sag - 0.15 * bounce, 0.6 * k, 0.1 * sag],
                arm_l: [-0.2 * sag + 0.2 * k, 0.15 + 0.35 * k, 0.0],
                arm_r: [-0.1 * sag + 0.1 * k, -0.15 - 0.75 * k, 0.0],
                elbow_l: -0.3 - 0.2 * k,
                elbow_r: -0.2,
                hip_l: [-0.55 * sag - 0.05 * k, 0.03],
                hip_r: [-0.45 * sag, 0.04],
                knee_l: 1.0 * sag + 0.15 * k,
                knee_r: 0.85 * sag + 0.08 * k,
                skirt: [0.1 * bounce, 0.0, 0.0],
                x: [0.3 * k, 0.0, 0.0],
                ..Pose::default()
            }
        }
    }
}

fn level(m: &M4, swing: f32, elbow: f32) -> M4 {
    mat::mul(m, &mat::rot_x(-(swing + elbow)))
}

pub fn gait_of(c: u8, anim: Anim) -> Option<Gait> {
    let s = STYLES[c as usize % 4];
    match anim {
        Anim::Walk => Some(Gait { period: s.walk, duty: s.duty[0], clearance: s.clearance, sink: 0.975, limp: 0.0 }),
        Anim::Run => Some(Gait { period: s.run, duty: s.duty[1], clearance: s.clearance * 1.6, sink: 0.94, limp: 0.0 }),
        Anim::Turn => Some(Gait { period: 18.0, duty: 0.6, clearance: s.clearance * 0.6, sink: 0.98, limp: 0.0 }),
        Anim::QuickTurn => Some(Gait { period: 12.0, duty: 0.55, clearance: s.clearance * 0.8, sink: 0.96, limp: 0.0 }),
        _ => None,
    }
}

pub fn rig(c: u8, x: f32, z: f32, heading: f32, anim: Anim, tic: u16, lantern: bool, look: f32, travel: [f32; 2]) -> Rig {
    let f = heroes::frame(c);
    let mut p = pose(c, anim, tic, travel);
    p.head[1] += look;
    let free = !matches!(anim, Anim::Reach | Anim::Grabbed | Anim::Faint | Anim::Crouch);
    let sway = p.arm_l[0] * 0.15;
    match (c == BIA, lantern, free) {
        (true, true, true) => (p.arm_l, p.elbow_l) = ([-0.95 + sway * 0.3, 0.25, -0.15], -1.4),
        (true, false, true) => (p.arm_l, p.elbow_l) = ([-0.25 + sway, 0.1, 0.0], -1.15),
        (false, true, true) => (p.arm_l, p.elbow_l) = ([-0.55 + sway, 0.14, 0.0], -0.85),
        _ => {}
    }
    p.ankle_l = -p.hip_l[0] - p.knee_l - p.pitch;
    p.ankle_r = -p.hip_r[0] - p.knee_r - p.pitch;
    let mut mats = skeleton::biped(f, &p, x, z, heading);
    if let Some(g) = gait_of(c, anim) {
        locomotion::biped(&mut mats, f, [x, z, heading], tic as f32, g, travel);
    }
    if c == BIA {
        let bounce = 0.012 * (tic as f32 * TAU / STYLES[BIA as usize].run * 2.0 - 0.6).sin();
        mats[SKIRT] = skeleton::at(&mats[TORSO], [0.0, bounce, -0.015], &mat::rot_x(p.x[0] * 0.08));
        mats[LAMP] = mat::mul(&mats[LAMP], &mat::translate([0.06, -0.02, 0.025]));
    }
    let lamp = level(&mats[LAMP], p.arm_l[0], p.elbow_l);
    mats[LAMP] = lamp;
    let grip = |seg: usize, swing: f32, elbow: f32, dz: f32| level(&mat::mul(&mats[seg], &mat::translate([0.0, -f.fore - 0.035, dz])), swing, elbow);
    let hands = [grip(FORE_R, p.arm_r[0], p.elbow_r, 0.03), grip(FORE_L, p.arm_l[0], p.elbow_l, 0.035)];
    let shown = (lantern || c == BIA) && !matches!(anim, Anim::Faint);
    if !shown {
        hide(&mut mats[LAMP]);
    }
    rig_from(mats, hands, lantern && shown, &lamp, [0.0, 0.16, 0.1])
}
