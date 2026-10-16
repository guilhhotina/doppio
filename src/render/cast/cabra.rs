use super::super::mat::{self, M4, V3};
use super::heroes::{card, cube, limb, loft, sw, tiled, tube};
use super::kit::{self, Kit, Uv, ring};
use super::skeleton::{self, Frame, Pose, ease, hide, noise, snap};
use super::{ARM_L, ARM_R, FOOT_L, FOOT_R, FORE_L, FORE_R, HEAD, LAMP, PELVIS, Rig, SHIN_L, SHIN_R, THIGH_L, THIGH_R, TORSO, X, rig_from};
use crate::sim::LegendAnim;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

const PIVOT: V3 = [0.0, 0.27, -0.02];

pub const FRAME: Frame = Frame {
    hip_x: 0.14,
    waist: 0.08,
    chest: 0.55,
    shoulder_x: 0.27,
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
    mat::mul(&mat::mul(&mat::translate([0.0, 0.27, -0.02]), &mat::rot_x(FRAC_PI_2 + 0.35)), &mat::scaling([1.3; 3]))
}

pub fn build(k: &mut Kit) {
    let f = &FRAME;
    let s8 = k.n(8, 4);
    let fur = tiled("cast_pelo", 4.0);
    k.seg(PELVIS);
    loft(k, &[ring(-0.17, 0.2, 0.16, 0.0), ring(0.1, 0.22, 0.17, 0.0)], s8, fur, (0, 0), false);
    if k.hi {
        k.at(mat::mul(&mat::translate([0.0, 0.0, -0.15]), &mat::rot_x(-0.6)));
        loft(k, &[ring(0.0, 0.04, 0.04, 0.0), ring(0.16, 0.0, 0.0, -0.02)], 4, fur, (0, 0), false);
        k.at(mat::IDENTITY);
    }
    k.seg(TORSO);
    loft(k, &[ring(-0.02, 0.2, 0.16, 0.0), ring(0.25, 0.26, 0.22, 0.04), ring(0.5, 0.3, 0.19, 0.0), ring(0.63, 0.13, 0.12, 0.0)], s8, fur, (0, 0), false);
    if k.hi {
        (0..6).for_each(|i| {
            let a = -1.2 + 2.4 * i as f32 / 5.0;
            let (s, c) = a.sin_cos();
            let p = |r: f32, y: f32, d: f32| [s * r + c * d, y, c * r * 0.75 - s * d];
            card(k, [p(0.26, 0.2, -0.06), p(0.26, 0.2, 0.06), p(0.24, 0.58, 0.08), p(0.24, 0.58, -0.08)], fur, true);
        });
    }
    k.seg(HEAD);
    if k.hi {
        tube(k, -0.05, 0.26, 0.13, 0.11, 6, fur);
    }
    let face = ("cast_cabra", Uv::Wrap([0.0005, 0.0, 0.9995, 1.0]));
    k.at(snout());
    let skull = [ring(-0.14, 0.0, 0.0, 0.0), ring(-0.11, 0.095, 0.105, 0.0), ring(0.0, 0.105, 0.115, 0.0), ring(0.13, 0.075, 0.09, 0.01), ring(0.27, 0.05, 0.062, 0.015), ring(0.32, 0.038, 0.045, 0.015)];
    loft(k, &skull, k.n(8, 4), face, (0, 0), true);
    let horn = |k: &mut Kit, side: f32| {
        let base = mat::point(&snout(), [side * 0.06, -0.07, -0.08]);
        k.at(mat::mul(&mat::translate(base), &mat::mul(&mat::rot_z(-side * 0.55), &mat::rot_x(-0.75))));
        let rings = [ring(0.0, 0.05, 0.055, 0.0), ring(0.17, 0.04, 0.044, -0.06), ring(0.32, 0.028, 0.03, -0.2), ring(0.42, 0.015, 0.016, -0.38), ring(0.44, 0.0, 0.0, -0.5)];
        loft(k, &rings, k.n(5, 3), sw(kit::BONE), (0, 0), false);
    };
    horn(k, 1.0);
    horn(k, -1.0);
    k.at(snout());
    let ears: &[f32] = if k.hi { &[-1.0, 1.0] } else { &[] };
    ears.iter().for_each(|&side| {
        card(k, [[side * 0.09, -0.02, -0.04], [side * 0.09, -0.1, -0.02], [side * 0.22, -0.08, 0.06], [side * 0.2, -0.04, 0.05]], fur, true);
    });
    if k.hi {
        card(k, [[0.0, 0.1, 0.08], [0.0, 0.2, 0.06], [0.0, 0.17, 0.2], [0.0, 0.12, 0.2]], sw(kit::BLACK), true);
        card(k, [[-0.035, 0.03, 0.08], [0.035, 0.03, 0.08], [0.035, 0.3, 0.05], [-0.035, 0.3, 0.05]], sw(kit::EMBER), false);
    }
    k.seg(X);
    k.at(mat::mul(&mat::rot_x(FRAC_PI_2 + 0.35), &mat::scaling([1.3; 3])));
    if k.hi {
        cube(k, [-0.05, -0.04, 0.05], [0.05, 0.27, 0.09], fur);
        card(k, [[-0.04, 0.0, 0.048], [0.04, 0.0, 0.048], [0.035, 0.26, 0.048], [-0.035, 0.26, 0.048]], sw(kit::EMBER), false);
    }
    let s = k.n(6, 3);
    [ARM_L, ARM_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.upper + 0.03, 0.08, 0.06, s, fur);
    });
    [FORE_L, FORE_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.fore, 0.06, 0.045, s, fur);
        if k.hi {
            cube(k, [-0.05, -f.fore - 0.12, -0.04], [0.05, -f.fore + 0.01, 0.06], sw(kit::HOOF));
        }
    });
    [THIGH_L, THIGH_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.thigh + 0.03, 0.13, 0.08, s, fur);
    });
    [SHIN_L, SHIN_R].iter().for_each(|&seg| {
        k.seg(seg);
        limb(k, f.shin + 0.02, 0.07, 0.045, s, fur);
    });
    [FOOT_L, FOOT_R].iter().for_each(|&seg| {
        k.seg(seg);
        match k.hi {
            true => {
                limb(k, f.meta, 0.045, 0.035, 5, fur);
                cube(k, [-0.045, -f.meta - f.sole, -0.05], [0.045, -f.meta + 0.03, 0.09], sw(kit::HOOF));
            }
            false => cube(k, [-0.04, -f.meta - f.sole, -0.04], [0.04, 0.0, 0.07], sw(kit::HOOF)),
        }
    });
}

fn stand(t: f32) -> Pose {
    let breathe = (t * 0.09).sin();
    Pose {
        torso: [0.18 + 0.04 * breathe, 0.0, 0.0],
        head: [0.0, 0.35 * noise(snap(t, 37.0) as i32), 0.15 * (t * 0.03).sin()],
        arm_l: [-0.1, 0.25, 0.1],
        arm_r: [-0.1, -0.25, -0.1],
        elbow_l: -0.35,
        elbow_r: -0.3 - 0.1 * breathe,
        hip_l: [-0.45, 0.08],
        hip_r: [-0.45, 0.08],
        knee_l: 1.05,
        knee_r: 1.05,
        ankle_l: -0.8,
        ankle_r: -0.8,
        x: [0.06 + 0.04 * breathe, 0.0, 0.0],
        ..Pose::default()
    }
}

fn biped_walk(t: f32, period: f32, amp: f32) -> Pose {
    let ph = t * TAU / period;
    let w = ph.sin();
    let lift = |p: f32| (p + 0.6).cos().max(0.0);
    Pose {
        pelvis: [0.0, 0.12 * w, 0.0],
        torso: [0.25, -0.18 * w, 0.0],
        arm_l: [0.35 * w * amp, 0.25, 0.1],
        arm_r: [-0.35 * w * amp, -0.25, -0.1],
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
    let ph = t * TAU / 15.0;
    let (w, v) = (ph.sin(), (ph - 0.7).sin());
    Pose {
        pelvis: [1.45, 0.0, 0.0],
        torso: [0.05 + 0.08 * (ph * 2.0).cos(), 0.0, 0.0],
        head: [-1.3, 0.0, 0.0],
        arm_l: [-1.35 + 0.55 * v, 0.1, 0.0],
        arm_r: [-1.35 + 0.55 * (v + 0.4).sin(), -0.1, 0.0],
        elbow_l: -0.2 - 0.6 * (ph + 1.2).cos().max(0.0),
        elbow_r: -0.2 - 0.6 * (ph + 1.6).cos().max(0.0),
        hip_l: [-1.45 - 0.5 * w, 0.06],
        hip_r: [-1.45 - 0.5 * (w + 0.3).sin(), 0.06],
        knee_l: 1.1 + 0.4 * (ph + 0.5).cos().max(0.0),
        knee_r: 1.1 + 0.4 * (ph + 0.9).cos().max(0.0),
        ankle_l: -0.8,
        ankle_r: -0.8,
        x: [0.25, 0.0, 0.0],
        ..Pose::default()
    }
}

fn pose(anim: LegendAnim, tic: u16) -> Pose {
    let t = tic as f32;
    match anim {
        LegendAnim::Idle => stand(t),
        LegendAnim::Walk => biped_walk(t, 36.0, 1.0),
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
            let (up, slam) = (ease(t / 5.0), ease((t - 5.0) / 4.0));
            Pose {
                torso: [0.18 - 0.35 * up + 0.6 * slam, 0.0, 0.0],
                head: [0.1 - 0.3 * slam, 0.0, 0.0],
                arm_l: [-2.6 * up + 1.2 * slam, 0.3, 0.0],
                arm_r: [-2.6 * up + 1.1 * slam, -0.3, 0.0],
                elbow_l: -0.4,
                elbow_r: -0.4,
                x: [0.5 * up, 0.0, 0.0],
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
        LegendAnim::Special => {
            let k = ease(t / 6.0);
            Pose {
                torso: [0.18 + 0.3 * k, 0.0, 0.0],
                head: [-0.45 * k, 0.08 * (t * 0.5).sin(), 0.0],
                arm_l: [0.3 * k, 0.5, 0.0],
                arm_r: [0.3 * k, -0.5, 0.0],
                hip_l: [-0.7, 0.1],
                hip_r: [-0.2, 0.1],
                knee_l: 1.3,
                knee_r: 0.9,
                x: [0.7 * k, 0.0, 0.0],
                ..stand(t)
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
    let skull = mat::mul(&mats[HEAD], &snout());
    let r = rig_from(mats, hands, false, &pelvis, mat::point(&snout(), [0.0, 0.1, -0.05]));
    let eye = |side: f32| mat::point(&skull, [side * 0.085, -0.02, -0.06]);
    let fire = matches!(anim, LegendAnim::Special).then(|| {
        let mouth = mat::point(&skull, [0.0, 0.3, 0.05]);
        let dir = mat::norm(mat::dir(&skull, [0.0, 1.0, 0.25]));
        (mouth, dir, ease((tic as f32 - 4.0) / 6.0) * (0.85 + 0.15 * (tic as f32 * 1.7).sin()))
    });
    Rig { eyes: Some([eye(1.0), eye(-1.0)]), eye_color: [1.0, 0.3, 0.08], fire, ..r }
}
