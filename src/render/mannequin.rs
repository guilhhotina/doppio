use super::mat::{self, M4, V3};
use super::mesh::{Faces, Mesh, Pen, Vert};
use crate::sim::Anim;
use std::f32::consts::{PI, TAU};

pub const SEGS: usize = 12;
const PELVIS: usize = 0;
const TORSO: usize = 1;
const HEAD: usize = 2;
const ARM_L: usize = 3;
const FORE_L: usize = 4;
const ARM_R: usize = 5;
const FORE_R: usize = 6;
const THIGH_L: usize = 7;
const SHIN_L: usize = 8;
const THIGH_R: usize = 9;
const SHIN_R: usize = 10;

const HIP_Y: f32 = 0.97;
const UPPER_ARM: f32 = 0.29;
const FOREARM: f32 = 0.26;
const THIGH: f32 = 0.44;
const SHIN: f32 = 0.43;

pub struct Mannequin {
    pub mesh: Mesh,
    normals: Vec<V3>,
}

fn limb(p: &mut Pen, len: f32, r0: f32, r1: f32, sides: usize) {
    p.cylinder([0.0, -len, 0.0], r1, r0, len, sides, false);
}

fn egg(p: &mut Pen, c: V3, r: V3, rings: usize, sides: usize) {
    let pt = |i: usize, j: usize| {
        let (t, a) = (i as f32 / rings as f32 * PI, j as f32 / sides as f32 * TAU);
        let s = t.sin() * (1.0 - 0.18 * (t / PI));
        [c[0] + a.sin() * s * r[0], c[1] - t.cos() * r[1], c[2] + a.cos() * s * r[2]]
    };
    (0..rings).for_each(|i| {
        (0..sides).for_each(|j| {
            let uv = [[0.0, 0.3], [0.3, 0.3], [0.3, 0.0], [0.0, 0.0]];
            match i {
                0 => p.tri([pt(0, 0), pt(1, j + 1), pt(1, j)], [uv[0], uv[2], uv[1]]),
                i if i == rings - 1 => p.tri([pt(i, j), pt(i, j + 1), pt(i + 1, 0)], [uv[0], uv[1], uv[2]]),
                _ => p.quad([pt(i, j), pt(i, j + 1), pt(i + 1, j + 1), pt(i + 1, j)], uv),
            }
        })
    });
}

fn build_segment(seg: usize, p: &mut Pen, hi: bool) {
    let (n, wide) = if hi { (6, 8) } else { (3, 4) };
    match seg {
        PELVIS => {
            p.xf = mat::scaling([1.0, 1.0, 0.68]);
            p.cylinder([0.0, -0.13, 0.0], 0.13, 0.16, 0.19, n, false);
        }
        TORSO => {
            p.xf = mat::scaling([1.0, 1.0, 0.62]);
            p.cylinder([0.0, 0.0, 0.0], 0.145, 0.2, 0.34, wide, false);
            p.cylinder([0.0, 0.34, 0.0], 0.2, 0.1, 0.1, wide, true);
        }
        HEAD if hi => {
            p.cylinder([0.0, 0.0, 0.0], 0.05, 0.045, 0.09, 6, false);
            egg(p, [0.0, 0.22, 0.01], [0.105, 0.135, 0.12], 5, 8);
        }
        HEAD => egg(p, [0.0, 0.2, 0.01], [0.11, 0.15, 0.12], 2, 4),
        ARM_L | ARM_R => limb(p, UPPER_ARM, 0.058, 0.044, n),
        FORE_L | FORE_R => {
            limb(p, FOREARM + if hi { 0.0 } else { 0.08 }, 0.044, 0.034, n);
            if hi {
                p.cuboid([-0.025, -FOREARM - 0.11, -0.04], [0.025, -FOREARM, 0.04], Faces::ALL);
            }
        }
        THIGH_L | THIGH_R => limb(p, THIGH, 0.085, 0.058, n),
        SHIN_L | SHIN_R => {
            limb(p, SHIN, 0.058, 0.04, n);
            let faces = if hi { Faces::NO_BOTTOM } else { Faces(0x14) };
            p.cuboid([-0.045, -SHIN - 0.065, -0.05], [0.045, -SHIN, 0.17], faces);
        }
        _ => {
            p.xf = mat::rot_x(PI / 2.0);
            p.cylinder([0.0, 0.0, 0.0], 0.04, 0.05, 0.16, n, true);
            if hi {
                p.cylinder([0.0, -0.05, 0.0], 0.03, 0.04, 0.05, 6, false);
            }
        }
    }
}

impl Mannequin {
    pub fn new(hi: bool) -> Mannequin {
        let mut mesh = Mesh::default();
        (0..SEGS).for_each(|s| {
            let mut part = Mesh::default();
            build_segment(s, &mut Pen { mesh: &mut part, xf: mat::IDENTITY, density: 4.0 }, hi);
            part.verts.iter_mut().for_each(|v| v.nrm[3] = s as i8);
            mesh.append(&part);
        });
        let normals = mesh.verts.iter().map(|v| mat::norm(v.normal())).collect();
        Mannequin { mesh, normals }
    }

    pub fn skin(&self, mats: &[M4; SEGS], light: impl Fn(V3) -> f32, tile: [u8; 4], out: &mut [Vert]) {
        let q = |v: f32| (v * 127.0) as i8;
        out.iter_mut().zip(&self.mesh.verts).zip(&self.normals).for_each(|((o, v), &n)| {
            let m = &mats[v.nrm[3] as usize];
            let n = mat::dir(m, n);
            *o = Vert { pos: mat::point(m, v.pos), uv: v.uv, nrm: [q(n[0]), q(n[1]), q(n[2]), q(light(n))], tile };
        });
    }
}

pub struct Crowd {
    pub lod: [Mannequin; 2],
}

impl Crowd {
    pub fn new() -> Crowd {
        Crowd { lod: [Mannequin::new(true), Mannequin::new(false)] }
    }

    pub fn get(&self, far: bool) -> &Mannequin {
        &self.lod[far as usize]
    }
}

#[derive(Clone, Copy, Default)]
struct Pose {
    bob: f32,
    lean: f32,
    twist: f32,
    torso: [f32; 3],
    head: [f32; 3],
    arm_l: [f32; 2],
    arm_r: [f32; 2],
    elbow_l: f32,
    elbow_r: f32,
    hip_l: f32,
    hip_r: f32,
    knee_l: f32,
    knee_r: f32,
}

fn gait(ph: f32, amp: f32, knee: f32, arm: f32) -> Pose {
    let s = ph.sin();
    Pose {
        bob: 0.035 * amp * (2.0 * ph).cos().abs() - 0.01 * amp,
        twist: s * 0.08 * amp,
        torso: [0.0, -s * 0.12 * amp, 0.0],
        arm_l: [s * arm, 0.06],
        arm_r: [-s * arm, -0.06],
        elbow_l: -0.25 - 0.3 * amp * (s.max(0.0)),
        elbow_r: -0.25 - 0.3 * amp * ((-s).max(0.0)),
        hip_l: -s * 0.42 * amp,
        hip_r: s * 0.42 * amp,
        knee_l: 0.08 + knee * (ph + 0.6).cos().max(0.0),
        knee_r: 0.08 + knee * (ph + 0.6 + PI).cos().max(0.0),
        ..Pose::default()
    }
}

fn pose(anim: Anim, tic: u16) -> Pose {
    let t = tic as f32;
    let idle = Pose {
        torso: [0.025 * (t * 0.11).sin(), 0.0, 0.0],
        head: [0.04, 0.12 * (t * 0.031).sin(), 0.03 * (t * 0.05).sin()],
        arm_l: [0.0, 0.07],
        arm_r: [0.0, -0.07],
        elbow_l: -0.12,
        elbow_r: -0.12,
        knee_l: 0.04,
        knee_r: 0.04,
        ..Pose::default()
    };
    match anim {
        Anim::Idle => idle,
        Anim::Walk => gait(t * TAU / 32.0, 1.0, 0.55, 0.32),
        Anim::Run => Pose { lean: 0.2, ..gait(t * TAU / 20.0, 1.7, 1.1, 0.75) },
        Anim::Turn => Pose { head: [0.0, -0.15, 0.0], ..gait(t * TAU / 18.0, 0.35, 0.3, 0.06) },
        Anim::QuickTurn => {
            let k = (1.0 - t / 12.0).max(0.0);
            Pose { bob: -0.07 * k, twist: 0.7 * k, head: [0.0, 0.5 * k, 0.0], knee_l: 0.45 * k, knee_r: 0.45 * k, hip_l: -0.2 * k, hip_r: -0.2 * k, ..idle }
        }
        Anim::Reach => {
            let k = (t / 8.0).min(1.0);
            Pose { arm_r: [-1.35 * k, -0.1], elbow_r: -0.15, torso: [0.12 * k, 0.15 * k, 0.0], head: [0.15 * k, 0.0, 0.0], ..idle }
        }
        Anim::Stagger => {
            let w = (t * 0.7).sin();
            Pose {
                bob: -0.04,
                lean: -0.22 + 0.08 * w,
                torso: [0.0, 0.0, 0.15 * w],
                head: [-0.3, 0.2 * w, 0.25 * w],
                arm_l: [-0.4, 0.7 + 0.2 * w],
                arm_r: [-0.3, -0.7 + 0.2 * w],
                elbow_l: -0.6,
                elbow_r: -0.6,
                knee_l: 0.35,
                knee_r: 0.2,
                hip_l: -0.25,
                hip_r: 0.1,
                ..Pose::default()
            }
        }
    }
}

fn rot(e: [f32; 3]) -> M4 {
    mat::mul(&mat::mul(&mat::rot_y(e[1]), &mat::rot_x(e[0])), &mat::rot_z(e[2]))
}

fn at(parent: &M4, offset: V3, r: &M4) -> M4 {
    mat::mul(&mat::mul(parent, &mat::translate(offset)), r)
}

pub struct Rig {
    pub mats: [M4; SEGS],
    pub lantern: V3,
    pub beam: V3,
}

pub fn rig(x: f32, z: f32, heading: f32, anim: Anim, tic: u16, lantern: bool, look: f32) -> Rig {
    let mut p = pose(anim, tic);
    p.head[1] += look;
    if lantern && !matches!(anim, Anim::Reach | Anim::Stagger) {
        p.arm_r = [-0.55 + p.arm_r[0] * 0.15, -0.12];
        p.elbow_r = -0.85;
    }
    let root = mat::mul(&mat::translate([x, HIP_Y + p.bob, z]), &mat::rot_y(heading));
    let pelvis = at(&root, [0.0; 3], &mat::mul(&mat::rot_y(-p.twist), &mat::rot_x(p.lean * 0.4)));
    let torso = at(&pelvis, [0.0, 0.06, 0.0], &rot([p.torso[0] + p.lean * 0.6, p.torso[1] + p.twist * 2.0, p.torso[2]]));
    let head = at(&torso, [0.0, 0.45, 0.0], &rot(p.head));
    let arm = |side: f32, a: [f32; 2]| at(&torso, [side * 0.215, 0.4, 0.0], &mat::mul(&mat::rot_x(a[0]), &mat::rot_z(a[1])));
    let (arm_l, arm_r) = (arm(1.0, p.arm_l), arm(-1.0, p.arm_r));
    let fore_l = at(&arm_l, [0.0, -UPPER_ARM, 0.0], &mat::rot_x(p.elbow_l));
    let fore_r = at(&arm_r, [0.0, -UPPER_ARM, 0.0], &mat::rot_x(p.elbow_r));
    let thigh = |side: f32, a: f32| at(&pelvis, [side * 0.09, -0.06, 0.0], &mat::rot_x(a));
    let (thigh_l, thigh_r) = (thigh(1.0, p.hip_l), thigh(-1.0, p.hip_r));
    let shin_l = at(&thigh_l, [0.0, -THIGH, 0.0], &mat::rot_x(p.knee_l));
    let shin_r = at(&thigh_r, [0.0, -THIGH, 0.0], &mat::rot_x(p.knee_r));
    let hold = at(&fore_r, [0.0, -FOREARM - 0.06, 0.0], &mat::rot_x(-p.elbow_r - p.arm_r[0]));
    let lamp = if lantern { hold } else { [0.0; 16] };
    let lantern_pos = mat::point(&hold, [0.0, 0.0, 0.17]);
    let beam = mat::norm(mat::dir(&root, [0.0, -0.32, 1.0]));
    Rig {
        mats: [pelvis, torso, head, arm_l, fore_l, arm_r, fore_r, thigh_l, shin_l, thigh_r, shin_r, lamp],
        lantern: lantern_pos,
        beam,
    }
}

pub fn face(mats: &[M4; SEGS]) -> (V3, V3) {
    let h = &mats[HEAD];
    (mat::point(h, [0.0, 0.22, 0.01]), mat::norm(mat::dir(h, [0.0, 0.0, 1.0])))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mannequin_budget() {
        let c = Crowd::new();
        let (hi, lo) = (c.get(false).mesh.tris(), c.get(true).mesh.tris());
        assert!((250..=400).contains(&hi), "{hi} tris");
        assert!(lo <= 110, "{lo} tris");
        assert!(c.lod.iter().all(|m| m.mesh.verts.iter().all(|v| (v.nrm[3] as usize) < SEGS)));
        assert!(c.get(false).mesh.verts.len() * crate::sim::MAX_BODIES <= u16::MAX as usize);
    }

    #[test]
    fn feet_touch_floor_and_head_on_top() {
        [true, false].iter().for_each(|&detail| {
            let m = Mannequin::new(detail);
            let r = rig(3.0, 4.0, 0.0, Anim::Idle, 0, false, 0.0);
            let mut out = vec![Vert::default(); m.mesh.verts.len()];
            m.skin(&r.mats, |_| 0.5, [1; 4], &mut out);
            let body: Vec<&Vert> = out.iter().zip(&m.mesh.verts).filter(|(_, v)| v.nrm[3] as usize != SEGS - 1).map(|(o, _)| o).collect();
            let lo = body.iter().map(|v| v.pos[1]).fold(f32::MAX, f32::min);
            let hi = body.iter().map(|v| v.pos[1]).fold(f32::MIN, f32::max);
            assert!(lo.abs() < 0.06, "feet at {lo}");
            assert!((1.6..1.85).contains(&hi), "head top at {hi}");
            assert!(body.iter().all(|v| (v.pos[0] - 3.0).abs() < 0.5 && (v.pos[2] - 4.0).abs() < 0.5));
            assert!(out.iter().all(|v| v.tile == [1; 4] && v.nrm[3] == 63));
        });
    }

    #[test]
    fn lantern_is_in_front_at_chest_height() {
        let r = rig(0.0, 0.0, 0.0, Anim::Walk, 7, true, 0.0);
        assert!(r.lantern[2] > 0.15 && (0.8..1.5).contains(&r.lantern[1]), "{:?}", r.lantern);
        assert!(r.beam[2] > 0.8);
        let r = rig(0.0, 0.0, PI / 2.0, Anim::Idle, 0, true, 0.0);
        assert!(r.lantern[0] > 0.15, "{:?}", r.lantern);
    }

    #[test]
    fn all_anims_pose_finite() {
        [Anim::Idle, Anim::Walk, Anim::Run, Anim::Turn, Anim::QuickTurn, Anim::Reach, Anim::Stagger].iter().for_each(|&a| {
            (0..60).for_each(|t| assert!(rig(1.0, 1.0, 0.3, a, t, true, 2.0).mats.iter().flatten().all(|v| v.is_finite())));
        });
    }

    #[test]
    fn look_turns_only_the_head() {
        let (still, turned) = (rig(0.0, 0.0, 0.0, Anim::Idle, 0, false, 0.0), rig(0.0, 0.0, 0.0, Anim::Idle, 0, false, PI));
        assert!(face(&still.mats).1[2] > 0.9 && face(&turned.mats).1[2] < -0.9);
        assert!((0..SEGS).filter(|&s| s != HEAD).all(|s| still.mats[s] == turned.mats[s]));
    }
}
