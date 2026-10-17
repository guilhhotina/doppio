use super::super::mat::{self, M4, V3};
use super::heroes::{cube, limb, loft, spike, sw, tiled, tube};
use super::kit::{self, Kit, Ring, Uv, ring};
use super::skeleton::{self, Frame, Pose, ease, hidden, noise, snap};
use super::{ARM_L, ARM_R, FOOT_L, FOOT_R, FORE_L, FORE_R, FX, HEAD, LAMP, PELVIS, Rig, SHIN_L, SHIN_R, SKIRT, THIGH_L, THIGH_R, TORSO, X, locomotion, rig_from};
use crate::sim::LegendAnim;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

const PIVOT: V3 = [0.0, 0.27, -0.02];

pub const FRAME: Frame = Frame {
    hip_x: 0.14,
    waist: 0.08,
    chest: 0.55,
    shoulder_x: 0.31,
    neck: 0.6,
    upper: 0.38,
    fore: 0.36,
    thigh: 0.44,
    shin: 0.44,
    meta: 0.34,
    sole: 0.04,
    hem: 0.0,
    back: 0.22,
    x_at: (HEAD, PIVOT),
};

fn snout() -> M4 {
    mat::mul(&mat::mul(&mat::translate([0.0, 0.27, 0.01]), &mat::rot_x(FRAC_PI_2 + 0.14)), &mat::scaling([1.35, 1.42, 1.35]))
}

const SKULL: [Ring; 8] = [
    ring(-0.13, 0.0, 0.0, -0.01),
    ring(-0.1, 0.088, 0.102, -0.01),
    ring(0.0, 0.098, 0.112, 0.0),
    ring(0.11, 0.072, 0.098, 0.016),
    ring(0.23, 0.056, 0.082, 0.028),
    ring(0.35, 0.048, 0.068, 0.034),
    ring(0.41, 0.034, 0.046, 0.034),
    ring(0.44, 0.012, 0.016, 0.032),
];

const TUFTS: [(V3, V3, f32); 7] = [
    ([0.18, 0.58, -0.06], [0.34, 0.82, -0.24], 0.075),
    ([-0.18, 0.58, -0.06], [-0.34, 0.82, -0.24], 0.075),
    ([0.0, 0.62, -0.12], [0.0, 0.88, -0.34], 0.08),
    ([0.1, 0.42, -0.18], [0.18, 0.6, -0.42], 0.07),
    ([-0.1, 0.42, -0.18], [-0.18, 0.6, -0.42], 0.07),
    ([0.22, 0.44, 0.08], [0.36, 0.32, 0.2], 0.065),
    ([-0.22, 0.44, 0.08], [-0.36, 0.32, 0.2], 0.065),
];

pub fn build(k: &mut Kit) {
    let f = &FRAME;
    let s8 = k.n(8, 4);
    let fur = tiled("cast_bode", 2.5);
    let pelt = tiled("cast_pelo", 3.0);
    let shag = tiled("cast_couro", 3.0);
    k.seg(PELVIS);
    loft(k, &[ring(-0.2, 0.2, 0.17, -0.03), ring(0.1, 0.235, 0.205, -0.01)], s8, fur, (0, 0), false);
    if k.hi {
        spike(k, [0.0, 0.0, -0.19], [0.0, 0.2, -0.38], 0.06, shag);
    }
    k.seg(TORSO);
    loft(
        k,
        &[
            ring(-0.02, 0.2, 0.17, 0.01),
            ring(0.35, 0.355, 0.285, 0.02),
            ring(0.55, 0.325, 0.235, -0.05),
            ring(0.66, 0.145, 0.135, 0.01),
        ],
        s8,
        pelt,
        (0, 0),
        false,
    );
    let tufts: &[(V3, V3, f32)] = if k.hi { &TUFTS } else { &[] };
    tufts.iter().for_each(|&(a, b, r)| spike(k, mat::scale(a, 1.12), mat::scale(b, 1.12), r * 1.25, shag));
    k.seg(HEAD);
    loft(k, &[ring(-0.1, 0.21, 0.195, -0.02), ring(0.06, 0.155, 0.145, 0.0), ring(0.24, 0.105, 0.105, 0.0)], k.n(5, 3), pelt, (0, 0), false);
    if k.hi {
        (0..3).for_each(|i| {
            let a = -1.4 + 1.4 * i as f32;
            let (s, c) = a.sin_cos();
            spike(k, [s * 0.16, 0.0, c * 0.15], [s * 0.32, -0.15, c * 0.3], 0.08, shag);
        });
    }
    let face = ("cast_cabra", Uv::Wrap([0.0005, 0.0, 0.9995, 1.0]));
    k.at(snout());
    loft(k, &SKULL, k.n(8, 4), face, (0, 0), true);
    if k.hi {
        k.glow(1.0);
        [-1.0, 1.0].iter().for_each(|&sd| {
            let (x, y, z) = (sd * 0.098, -0.02, -0.055);
            k.tex("cast_misc").card([[x, y - 0.032, z + 0.016], [x, y - 0.032, z - 0.016], [x, y + 0.032, z - 0.016], [x, y + 0.032, z + 0.016]], kit::swatch(kit::EMBER), false);
        });
        k.glow(0.0);
    }
    let horn = |k: &mut Kit, side: f32| {
        let base = mat::point(&snout(), [side * 0.062, -0.08, -0.085]);
        k.at(mat::mul(&mat::translate(base), &mat::mul(&mat::rot_y(side * 0.42), &mat::mul(&mat::rot_z(-side * 0.88), &mat::mul(&mat::rot_x(-0.32), &mat::scaling([1.52; 3]))))));
        if k.hi {
            loft(k, &[ring(-0.02, 0.056, 0.062, 0.0), ring(0.075, 0.05, 0.055, -0.014)], 4, sw(kit::LEATHER), (0, 0), false);
        }
        let rings = [
            ring(0.07, 0.05, 0.055, -0.012),
            ring(0.16, 0.044, 0.048, -0.045),
            ring(0.25, 0.038, 0.042, -0.095),
            ring(0.35, 0.028, 0.03, -0.18),
            ring(0.45, 0.0, 0.0, -0.37),
        ];
        let rings: &[Ring] = if k.hi { &rings } else { &[ring(-0.02, 0.055, 0.06, 0.0), rings[3], rings[4]] };
        let sides = k.n(4, 3);
        k.tex("cast_misc").loft(rings, sides, kit::swatch(kit::BONE), (0, 0), false);
    };
    horn(k, 1.0);
    horn(k, -1.0);
    k.at(snout());
    if k.hi {
        [-1.0, 1.0].iter().for_each(|&side| spike(k, [side * 0.085, -0.04, -0.03], [side * 0.28, -0.01, 0.04], 0.044, shag));
    }
    k.seg(X);
    k.at(mat::mul(&mat::rot_x(FRAC_PI_2 + 0.14), &mat::scaling([1.35, 1.42, 1.35])));
    if k.hi {
        loft(k, &[ring(-0.04, 0.046, 0.024, 0.065), ring(0.3, 0.036, 0.02, 0.065)], 4, shag, (0, 0), true);
        card_glow(k);
        k.at(mat::mul(&mat::rot_x(FRAC_PI_2 + 0.14), &mat::scaling([1.35, 1.42, 1.35])));
        spike(k, [0.0, 0.18, 0.08], [0.0, 0.04, 0.36], 0.052, sw(kit::BONE));
    }
    k.seg(FX);
    flame(k);
    let s = k.n(5, 3);
    [ARM_L, ARM_R].iter().for_each(|&seg| {
        k.seg(seg);
        loft(k, &[ring(0.08, 0.155, 0.145, -0.01), ring(-0.12, 0.125, 0.115, 0.0), ring(-f.upper - 0.03, 0.08, 0.075, 0.0)], s, pelt, (0, 0), false);
    });
    [(FORE_L, LAMP), (FORE_R, SKIRT)].iter().for_each(|&(seg, hoof)| {
        k.seg(seg);
        loft(k, &[ring(0.02, 0.082, 0.078, 0.0), ring(-0.12, 0.088, 0.082, 0.01), ring(-f.fore, 0.052, 0.048, 0.0)], k.n(4, 3), fur, (0, 0), false);
        k.seg(hoof);
        match k.hi {
            true => cube(k, [-0.055, -0.04, -0.045], [0.055, 0.085, 0.07], sw(kit::HOOF)),
            false => tube(k, 0.08, -0.04, 0.04, 0.05, 3, sw(kit::HOOF)),
        }
    });
    [THIGH_L, THIGH_R].iter().for_each(|&seg| {
        k.seg(seg);
        loft(k, &[ring(0.08, 0.165, 0.195, -0.04), ring(-0.14, 0.175, 0.205, 0.0), ring(-f.thigh - 0.03, 0.085, 0.09, 0.0)], s, pelt, (0, 0), false);
        if k.hi {
            spike(k, [0.0, -0.12, -0.16], [0.0, -0.34, -0.32], 0.075, shag);
        }
    });
    [SHIN_L, SHIN_R].iter().for_each(|&seg| {
        k.seg(seg);
        loft(k, &[ring(0.03, 0.088, 0.092, 0.0), ring(-0.1, 0.062, 0.068, 0.0), ring(-f.shin - 0.02, 0.052, 0.058, -0.01)], k.n(4, 3), fur, (0, 0), false);
    });
    [FOOT_L, FOOT_R].iter().for_each(|&seg| {
        k.seg(seg);
        match k.hi {
            true => {
                limb(k, f.meta, 0.048, 0.04, 4, fur);
                cube(k, [-0.058, -f.meta - f.sole, -0.055], [0.058, -f.meta + 0.045, 0.115], sw(kit::LEATHER));
            }
            false => tube(k, 0.0, -f.meta - f.sole, 0.035, 0.045, 3, sw(kit::HOOF)),
        }
    });
}

fn card_glow(k: &mut Kit) {
    k.glow(0.95).tex("cast_misc").card([[-0.038, 0.0, 0.048], [0.038, 0.0, 0.048], [0.032, 0.26, 0.048], [-0.032, 0.26, 0.048]], kit::swatch(kit::EMBER), false);
    k.glow(0.0);
}

pub fn flame(k: &mut Kit) {
    let tongue = |k: &mut Kit, a: f32, w: f32, l: f32, seed: i32, tone: u8| {
        let (s, c) = a.sin_cos();
        let p = |x: f32, z: f32| [x * w * c, x * w * s, z * l];
        let j = |i: i32| 1.0 + 0.24 * noise(seed * 7 + i);
        let uv = kit::swatch(tone);
        k.tex("cast_misc");
        k.card([p(-0.15, 0.0), p(0.15, 0.0), p(0.21, 0.35), p(-0.21, 0.35)], uv, false);
        k.tri([p(-0.21, 0.35), p(-0.05, 0.35), p(-0.17 * j(1), 0.72 * j(2))], uv);
        k.tri([p(-0.08, 0.35), p(0.09, 0.35), p(0.02 * j(3), 0.96 * j(4))], uv);
        k.tri([p(0.05, 0.35), p(0.21, 0.35), p(0.18 * j(5), 0.68 * j(6))], uv);
    };
    let n = k.n(3, 1);
    k.glow(0.75);
    (0..n).for_each(|i| tongue(k, i as f32 * std::f32::consts::PI / n as f32, 1.15, 0.96, i as i32, kit::EMBER));
    k.glow(1.0);
    (0..n).for_each(|i| tongue(k, (i as f32 + 0.5) * std::f32::consts::PI / n as f32, 0.68, 0.65, i as i32 + 9, kit::BRASS));
    k.glow(0.0);
}

fn stand(t: f32) -> Pose {
    let breathe = (t * 0.09).sin();
    Pose {
        torso: [0.4 + 0.04 * breathe, 0.0, 0.0],
        head: [-0.22, 0.35 * noise(snap(t, 37.0) as i32), 0.15 * (t * 0.03).sin()],
        arm_l: [-0.4, 0.17, 0.15],
        arm_r: [-0.4, -0.17, -0.15],
        elbow_l: -0.55,
        elbow_r: -0.5 - 0.1 * breathe,
        hip_l: [-0.6, 0.1],
        hip_r: [-0.6, 0.1],
        knee_l: 1.05,
        knee_r: 1.05,
        ankle_l: -0.8,
        ankle_r: -0.8,
        x: [0.34 + 0.08 * breathe, 0.0, 0.0],
        ..Pose::default()
    }
}

fn biped_walk(t: f32, period: f32, amp: f32) -> Pose {
    let ph = t * TAU / period;
    let w = ph.sin();
    let bob = (2.0 * ph).cos();
    let lift = |p: f32| (p + 0.6).cos().max(0.0);
    Pose {
        roll: 0.045 * w * amp,
        pelvis: [0.03 * bob, 0.14 * w, 0.0],
        torso: [0.46 + 0.04 * bob, -0.2 * w, -0.04 * w],
        head: [-0.28 - 0.04 * bob, -0.08 * w, 0.06 * w],
        arm_l: [-0.35 + 0.42 * w * amp, 0.3, 0.12],
        arm_r: [-0.35 - 0.42 * w * amp, -0.3, -0.12],
        elbow_l: -0.6 - 0.25 * (-w).max(0.0),
        elbow_r: -0.6 - 0.25 * w.max(0.0),
        hip_l: [-0.45 - 0.45 * w * amp, 0.08],
        hip_r: [-0.45 + 0.45 * w * amp, 0.08],
        knee_l: 1.05 + 0.45 * amp * lift(ph),
        knee_r: 1.05 + 0.45 * amp * lift(ph + PI),
        ankle_l: -0.8 - 0.3 * amp * lift(ph),
        ankle_r: -0.8 - 0.3 * amp * lift(ph + PI),
        ..stand(t)
    }
}

fn gallop(t: f32) -> Pose {
    let ph = t * TAU / PERIOD[1];
    let flex = (ph - 0.6).sin();
    Pose {
        pelvis: [1.44 + 0.16 * flex, 0.0, 0.0],
        torso: [0.05 - 0.14 * flex, 0.0, 0.0],
        head: [-1.34 + 0.18 * flex, 0.0, 0.0],
        x: [0.22 + 0.18 * (ph + 1.0).sin(), 0.0, 0.0],
        ..Pose::default()
    }
}

fn pose(anim: LegendAnim, tic: u16) -> Pose {
    let t = tic as f32;
    match anim {
        LegendAnim::Idle => stand(t),
        LegendAnim::Walk => biped_walk(t, PERIOD[0], 1.0),
        LegendAnim::Run => gallop(t),
        LegendAnim::Sniff => {
            let sweep = (t * 0.07).sin();
            Pose {
                pelvis: [0.45, 0.0, 0.0],
                torso: [0.3, 0.2 * sweep, 0.0],
                head: [0.2, 0.6 * sweep, 0.0],
                hip_l: [-0.95, 0.1],
                hip_r: [-0.95, 0.1],
                knee_l: 1.4,
                knee_r: 1.4,
                ankle_l: -0.9,
                ankle_r: -0.9,
                x: [0.1 * (t * 0.8).sin().max(0.0), 0.0, 0.0],
                ..stand(t)
            }
        }
        LegendAnim::Grab => {
            let (up, slam) = (ease(t / 4.0), ease((t - 4.0) / 4.0));
            Pose {
                torso: [0.18 - 0.42 * up + 0.72 * slam, 0.0, 0.0],
                head: [0.15 * up - 0.35 * slam, 0.0, 0.0],
                arm_l: [-2.65 * up + 1.35 * slam, 0.35 * (1.0 - 0.5 * slam), 0.0],
                arm_r: [-2.65 * up + 1.25 * slam, -0.35 * (1.0 - 0.5 * slam), 0.0],
                elbow_l: -0.45,
                elbow_r: -0.45,
                x: [0.6 * up, 0.0, 0.0],
                ..stand(t)
            }
        }
        LegendAnim::Stagger => {
            let w = (t * 1.6).sin();
            Pose {
                torso: [-0.15, 0.25 * w, 0.1 * w],
                head: [-0.3, 0.8 * w, 0.4 * w],
                arm_l: [-1.6, 0.6, 0.0],
                arm_r: [-1.2, -0.8, 0.0],
                elbow_l: -1.4,
                elbow_r: -0.9,
                hip_l: [-0.7, 0.15],
                hip_r: [-0.2, 0.15],
                x: [0.4, 0.0, 0.0],
                ..stand(t)
            }
        }
        LegendAnim::Alert => {
            let rear = ease(t / 4.0);
            let inhale = ease(t / 12.0);
            let quiver = 0.03 * (t * 2.1).sin() * inhale;
            Pose {
                torso: [0.18 - 0.4 * rear - 0.1 * inhale, quiver, 0.0],
                head: [0.25 * rear - 0.55 * inhale, 0.0, quiver],
                arm_l: [-0.9 * rear, 0.7 * inhale, 0.2],
                arm_r: [-0.9 * rear, -0.7 * inhale, -0.2],
                elbow_l: -0.9 * rear,
                elbow_r: -0.9 * rear,
                hip_l: [-0.6, 0.1],
                hip_r: [-0.3, 0.1],
                knee_l: 1.2,
                knee_r: 0.95,
                x: [0.05, 0.0, 0.0],
                ..stand(t)
            }
        }
        LegendAnim::Special => {
            let k = ease(t / 3.0);
            let shake = 0.05 * (t * 1.3).sin();
            Pose {
                torso: [-0.22 + 0.55 * k, shake, 0.0],
                head: [-0.55 + 0.1 * k, 0.12 * (t * 0.35).sin(), 0.0],
                arm_l: [-0.9 + 1.1 * k, 0.7 - 0.2 * k, 0.2],
                arm_r: [-0.9 + 1.1 * k, -0.7 + 0.2 * k, -0.2],
                elbow_l: -0.9 + 0.5 * k,
                elbow_r: -0.9 + 0.5 * k,
                hip_l: [-0.7, 0.1],
                hip_r: [-0.2, 0.1],
                knee_l: 1.3,
                knee_r: 0.9,
                x: [0.75 * k, 0.0, 0.0],
                ..stand(t)
            }
        }
    }
}

pub const PERIOD: [f32; 2] = [28.0, 18.0];

pub fn rig(x: f32, z: f32, heading: f32, anim: LegendAnim, tic: u16, travel: [f32; 2]) -> Rig {
    let p = pose(anim, tic);
    let mut mats = skeleton::biped(&FRAME, &p, x, z, heading);
    if matches!(anim, LegendAnim::Run) {
        let f = locomotion::Quadruped { hind: [FRAME.thigh, FRAME.shin, FRAME.meta], front: [FRAME.upper, FRAME.fore], sole: FRAME.sole, height: 0.72, spread: [0.27, 0.14], z: [0.6, 0.0] };
        locomotion::quadruped(&mut mats, &f, [x, z, heading], tic as f32, PERIOD[1], travel[0], true);
    } else {
        if matches!(anim, LegendAnim::Walk) {
            locomotion::biped(&mut mats, &FRAME, [x, z, heading], tic as f32, locomotion::Gait { period: PERIOD[0], duty: 0.6, clearance: 0.12, sink: 0.97, limp: 0.0 }, travel);
        }
        mats[LAMP] = skeleton::at(&mats[FORE_L], [0.0, -FRAME.fore - 0.08, 0.0], &mat::IDENTITY);
        mats[SKIRT] = skeleton::at(&mats[FORE_R], [0.0, -FRAME.fore - 0.08, 0.0], &mat::IDENTITY);
    }
    let t = tic as f32;
    let flame = match anim {
        LegendAnim::Special => ease((t - 1.0) / 3.0) * (0.85 + 0.15 * (t * 1.7).sin()),
        LegendAnim::Alert => 0.3 * ease((t - 6.0) / 9.0) * (0.7 + 0.3 * (t * 2.3).sin()),
        _ => 0.0,
    };
    let skull = mat::mul(&mats[HEAD], &snout());
    let mouth = mat::point(&skull, [0.0, 0.38, 0.05]);
    let dir = mat::norm(mat::dir(&skull, [0.0, 1.0, 0.25]));
    mats[FX] = match flame > 0.0 {
        true => jet(mouth, dir, flame, flame * (1.2 + 0.15 * (t * 2.9).sin()), t * 0.9),
        false => hidden(&mats[HEAD]),
    };
    let hands = [mats[FORE_R], mats[FORE_L]];
    let pelvis = mats[PELVIS];
    let r = rig_from(mats, hands, false, &pelvis, mat::point(&snout(), [0.0, 0.1, -0.05]));
    let eye = |side: f32| mat::point(&skull, [side * 0.08, -0.02, -0.06]);
    let fire = (flame > 0.0).then_some((mouth, dir, flame * 1.3));
    Rig { eyes: Some([eye(1.0), eye(-1.0)]), eye_color: [1.0, 0.45, 0.08], fire, ..r }
}

pub fn jet(at: V3, dir: V3, width: f32, len: f32, spin: f32) -> M4 {
    let aim = mat::mul(&mat::rot_y(dir[0].atan2(dir[2])), &mat::rot_x(-dir[1].clamp(-1.0, 1.0).asin()));
    mat::mul(&mat::mul(&mat::translate(at), &aim), &mat::mul(&mat::rot_z(spin), &mat::scaling([width, width, len])))
}
