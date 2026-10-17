use super::super::mat;
use super::heroes::{card, cube, fingers, head, limb, loft, spike, sw, tiled, tube};
use super::kit::{self, Kit, Uv, egg, ring};
use super::skeleton::{self, Frame, Pose, ease, hide, noise, snap};
use super::{ARM_L, ARM_R, FOOT_L, FOOT_R, FORE_L, FORE_R, HEAD, LAMP, PELVIS, Rig, SHIN_L, SHIN_R, SKIRT, TORSO, locomotion, rig_from};
use crate::sim::LegendAnim;
use std::f32::consts::TAU;

pub const FRAME: Frame = Frame {
    hip_x: 0.075,
    waist: 0.05,
    chest: 0.36,
    shoulder_x: 0.2,
    neck: 0.4,
    upper: 0.28,
    fore: 0.26,
    thigh: 0.38,
    shin: 0.37,
    meta: 0.0,
    sole: 0.05,
    hem: 0.79,
    back: 0.24,
    x_at: (TORSO, [0.19, 0.42, -0.03]),
};

pub fn build(k: &mut Kit) {
    let f = &FRAME;
    let s8 = k.n(8, 4);
    let shawl = tiled("cast_preto", 4.0);
    let rags = tiled("cast_trapo", 3.5);
    let dress = tiled("cast_chita", 4.0);
    let skin = tiled("cast_pele", 6.0);
    let gap = if k.hi { (3, 5) } else { (1, 3) };
    k.seg(SKIRT);
    super::heroes::garment(k, super::cloth::SKIRTS, s8, dress);
    k.seg(TORSO);
    loft(k, &[ring(-0.02, 0.13, 0.1, 0.0), ring(0.18, 0.135, 0.115, -0.01), ring(0.34, 0.125, 0.125, -0.04), ring(0.42, 0.052, 0.052, 0.02)], k.n(6, 4), dress, (0, 0), false);
    loft(
        k,
        &[
            ring(0.47, 0.065, 0.075, -0.01),
            ring(0.44, 0.16, 0.15, -0.04),
            ring(0.32, 0.25, 0.21, -0.06),
            ring(0.1, 0.26, 0.23, -0.07),
            ring(-0.08, 0.235, 0.21, -0.075),
        ],
        s8,
        shawl,
        gap,
        false,
    );
    k.tex("cast_preto").tri([[-0.22, 0.02, -0.26], [0.22, 0.02, -0.26], [0.0, -0.46, -0.21]], Uv::Wrap([0.0, 0.0, 0.8, 1.0]));
    if k.hi {
        (0..5).for_each(|i| {
            let x = -0.17 + 0.085 * i as f32;
            let mat = if i % 2 == 0 { rags } else { sw(kit::BLACK) };
            spike(k, [x, -0.04, -0.24 + 0.02 * (i % 2) as f32], [x * 1.15, -0.22 - 0.06 * noise(i * 5 + 1).abs(), -0.26], 0.032, mat);
        });
    }
    k.seg(HEAD);
    super::heroes::neck_textured(k, "cast_matinta", 0.024, 0.042);
    head(k, "cast_matinta", 0.008, 0.25, 0.092, 0.104, 0.01);
    loft(
        k,
        &[
            ring(0.34, 0.0, 0.0, -0.02),
            ring(0.3, 0.095, 0.105, -0.01),
            ring(0.2, 0.132, 0.138, -0.02),
            ring(0.04, 0.145, 0.148, -0.04),
            ring(-0.11, 0.168, 0.158, -0.07),
        ],
        s8,
        shawl,
        gap,
        false,
    );
    let grey = ("cast_matinta", Uv::Wrap([0.03, 0.12, 0.16, 0.9]));
    let dark = ("cast_trapo", Uv::Wrap([0.1, 0.0, 0.8, 1.0]));
    let n = k.n(14, 4);
    (0..n).for_each(|i| {
        let u = i as f32 / (n - 1) as f32 * 2.0 - 1.0;
        if u.abs() < 0.42 {
            return;
        }
        let a = u * 1.45;
        let (s, c) = a.sin_cos();
        let top = [s * 0.102, 0.18 + 0.06 * c, 0.008 + c * 0.095];
        let w = 0.015 + 0.009 * noise(i as i32 * 5 + 1).abs();
        let side = [c * w, 0.0, -s * w];
        let len = 0.34 + 0.3 * noise(i as i32 * 3 + 7).abs();
        let tip = [top[0] + s * 0.06 + 0.02 * noise(i as i32 + 30), top[1] - len, top[2] + 0.02 + 0.02 * c];
        let m = if i % 2 == 1 { dark } else { grey };
        k.tex(m.0).tri([mat::sub(top, side), mat::add(top, side), tip], m.1);
    });
    let s = k.n(5, 3);
    [ARM_L, ARM_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.upper + 0.02, 0.064, 0.054, s, shawl);
    });
    [FORE_L, FORE_R].iter().for_each(|&seg| {
        k.seg(seg);
        if k.hi {
            limb(k, 0.08, 0.058, 0.065, s, shawl);
        }
        tube(k, -0.05, -f.fore, 0.025, 0.019, k.n(4, 3), skin);
        match k.hi {
            true => cube(k, [-0.028, -f.fore - 0.06, -0.014], [0.028, -f.fore + 0.005, 0.02], skin),
            false => tube(k, -f.fore, -f.fore - 0.14, 0.02, 0.0, 3, skin),
        }
        fingers(k, -f.fore - 0.055, 0.12, 0.054, 0.4, 0.009, sw(kit::BONE));
    });
    [SHIN_L, SHIN_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.shin + 0.02, 0.032, 0.023, k.n(4, 3), skin);
    });
    [FOOT_L, FOOT_R].iter().for_each(|&seg| {
        k.seg(seg);
        loft(
            k,
            &[ring(-0.05, 0.028, 0.03, 0.0), ring(0.0, 0.036, 0.052, 0.04), ring(0.0, 0.032, 0.042, 0.11)]
                .map(|r| ring(r.y - f.sole + 0.05, r.rx, r.rz, r.dz)),
            k.n(4, 3),
            skin,
            (0, 0),
            false,
        );
        if k.hi {
            (0..3).for_each(|i| {
                let x = (i as f32 - 1.0) * 0.02;
                spike(k, [x, -f.sole + 0.012, 0.1], [x * 1.35, -f.sole, 0.17], 0.01, sw(kit::BONE));
            });
        }
    });
    k.seg(super::X);
    owl(k);
}

fn owl(k: &mut Kit) {
    let body = tiled("cast_alva", 3.0);
    let wing = tiled("cast_trapo", 3.0);
    k.at(mat::mul(&mat::mul(&mat::translate([0.0, 0.0, -0.04]), &mat::rot_x(0.28)), &mat::scaling([1.42; 3])));
    loft(k, &egg(0.165, 0.0, 0.052, 0.054, 0.0, k.n(4, 2)), k.n(6, 4), body, (0, 0), false);
    if k.hi {
        card(k, [[0.0, 0.072, 0.05], [0.036, 0.138, 0.042], [0.0, 0.128, 0.048], [-0.036, 0.138, 0.042]], sw(kit::WHITE), false);
        k.glow(0.95);
        [-1.0, 1.0].iter().for_each(|&sd| card(k, [[sd * 0.018 - 0.008, 0.105, 0.051], [sd * 0.018 + 0.008, 0.105, 0.051], [sd * 0.018 + 0.008, 0.12, 0.05], [sd * 0.018 - 0.008, 0.12, 0.05]], sw(kit::BRASS), false));
        k.glow(0.0);
        spike(k, [0.0, 0.102, 0.048], [0.0, 0.08, 0.066], 0.007, sw(kit::HOOF));
        spike(k, [0.0, 0.03, -0.038], [0.0, -0.055, -0.08], 0.024, wing);
        [-1.0, 1.0].iter().for_each(|&sd| spike(k, [sd * 0.046, 0.112, -0.008], [sd * 0.068, -0.025, -0.04], 0.025, wing));
    }
}


fn hunched(t: f32) -> Pose {
    let rock = (t * 0.06).sin();
    let fidget = noise(snap(t, 17.0) as i32 + 4);
    Pose {
        torso: [0.78 + 0.03 * rock, 0.0, 0.04 * rock],
        head: [-0.8, 0.3 * fidget, 0.22 * noise(snap(t + 5.0, 29.0) as i32)],
        arm_l: [-0.55, 0.12, -0.25],
        arm_r: [-0.45, -0.12, 0.25],
        elbow_l: -0.55 + 0.08 * (t * 0.7).sin(),
        elbow_r: -0.7 + 0.08 * (t * 0.9).cos(),
        hip_l: [-0.2, 0.05],
        hip_r: [-0.2, 0.05],
        knee_l: 0.35,
        knee_r: 0.35,
        ..Pose::default()
    }
}

fn shuffle(t: f32, period: f32, amp: f32, low: f32) -> Pose {
    let ph = t * TAU / period;
    let w = ph.sin();
    let base = hunched(t);
    Pose {
        roll: 0.05 * w,
        torso: [0.78 + low, 0.12 * w, 0.0],
        head: [-0.8 - low, -0.1 * w, base.head[2]],
        hip_l: [-0.2 - 0.3 * w * amp, 0.05],
        hip_r: [-0.2 + 0.3 * w * amp, 0.05],
        knee_l: 0.35 + 0.45 * amp * (ph + 0.6).cos().max(0.0),
        knee_r: 0.35 + 0.45 * amp * (ph + 0.6 + std::f32::consts::PI).cos().max(0.0),
        skirt: [0.05 * amp, 0.1 * w, 0.06 * w],
        ..base
    }
}

fn pose(anim: LegendAnim, tic: u16) -> Pose {
    let t = tic as f32;
    match anim {
        LegendAnim::Idle => hunched(t),
        LegendAnim::Walk => shuffle(t, PERIOD[0], 0.65, 0.0),
        LegendAnim::Run => {
            let ph = t * TAU / PERIOD[1];
            let w = ph.sin();
            Pose {
                arm_l: [-0.9 + 0.35 * w, 0.25, -0.15],
                arm_r: [-0.9 - 0.35 * w, -0.25, 0.15],
                elbow_l: -0.55,
                elbow_r: -0.55,
                ..shuffle(t, PERIOD[1], 1.0, 0.18)
            }
        }
        LegendAnim::Sniff => {
            let tilt = (t * 0.08).sin();
            Pose { head: [-0.6, 0.4 * noise(snap(t, 19.0) as i32), 0.6 * tilt], torso: [0.9, 0.0, 0.1 * tilt], ..hunched(t) }
        }
        LegendAnim::Grab => {
            let wind = ease(t / 2.0) * (1.0 - ease((t - 2.0) / 3.0));
            let k = ease((t - 1.5) / 5.0);
            Pose {
                torso: [0.78 + 0.15 * wind - 0.4 * k, 0.0, 0.0],
                head: [-0.75 + 0.3 * k, 0.0, 0.0],
                arm_l: [-0.5 - 1.45 * k, 0.28 + 0.2 * wind, 0.0],
                arm_r: [-0.5 - 1.55 * k, -0.28 - 0.2 * wind, 0.0],
                elbow_l: -0.6 + 0.42 * k,
                elbow_r: -0.6 + 0.42 * k,
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
                ..Pose::default()
            }
        }
        LegendAnim::Special => {
            let k = ease(t / 8.0);
            let trill = 0.04 * (t * 2.1).sin();
            Pose {
                torso: [0.78 - 0.42 * k, 0.0, 0.0],
                head: [-0.75 - 0.28 * k, trill, 0.32 * k],
                arm_l: [-0.95 * k - 0.45, 0.35 * k, -0.2],
                arm_r: [-1.15 * k - 0.45, -0.05, 0.55 * k],
                elbow_l: -0.8 * k - 0.4,
                elbow_r: -0.7 - 1.55 * k,
                ..hunched(t)
            }
        }
        LegendAnim::Alert => {
            let k = ease(t / 3.0);
            let twitch = noise(snap(t, 4.0) as i32 + 61);
            Pose {
                torso: [0.78 - 0.32 * k, 0.0, 0.0],
                head: [-0.6 - 0.2 * k, 0.12 * twitch * k, 0.45 * k],
                arm_l: [-0.55 - 0.55 * k, 0.62 * k, -0.3],
                arm_r: [-0.45 - 0.55 * k, -0.62 * k, 0.3],
                elbow_l: -0.6 - 0.6 * k,
                elbow_r: -0.7 - 0.6 * k,
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
    let t = tic as f32;
    let flutter = match anim {
        LegendAnim::Special | LegendAnim::Alert | LegendAnim::Grab => 0.22 * (t * 2.4).sin(),
        _ => 0.0,
    };
    p.x = [-p.torso[0] - p.pitch + flutter, 0.25 * noise(snap(t + 3.0, 23.0) as i32), -p.torso[2] - p.roll];
    let mut mats = skeleton::biped(&FRAME, &p, x, z, heading);
    if matches!(anim, LegendAnim::Walk | LegendAnim::Run) {
        let run = matches!(anim, LegendAnim::Run);
        let gait = if run { locomotion::Gait { period: PERIOD[1], duty: 0.45, clearance: 0.06, sink: 0.93, limp: 0.0 } } else { locomotion::Gait { period: PERIOD[0], duty: 0.55, clearance: 0.04, sink: 0.95, limp: 0.0 } };
        locomotion::biped(&mut mats, &FRAME, [x, z, heading], tic as f32, gait, travel);
    }
    hide(&mut mats[LAMP]);
    let hands = [mats[FORE_R], mats[FORE_L]];
    let pelvis = mats[PELVIS];
    let head = mats[HEAD];
    let r = rig_from(mats, hands, false, &pelvis, [0.0, 0.16, 0.1]);
    let eye = |side: f32| mat::point(&head, [side * 0.03, 0.155, 0.09]);
    Rig { eyes: Some([eye(1.0), eye(-1.0)]), eye_color: [0.32, 0.42, 0.34], ..r }
}
