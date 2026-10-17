use super::super::mat::{self, M4, V3};
use super::{ARM_L, ARM_R, FOOT_L, FOOT_R, FORE_L, FORE_R, FX, HEAD, LAMP, PELVIS, SEGS, SHIN_L, SHIN_R, SKIRT, THIGH_L, THIGH_R, TORSO, X};

#[derive(Clone, Copy)]
pub struct Frame {
    pub hip_x: f32,
    pub waist: f32,
    pub chest: f32,
    pub shoulder_x: f32,
    pub neck: f32,
    pub upper: f32,
    pub fore: f32,
    pub thigh: f32,
    pub shin: f32,
    pub meta: f32,
    pub sole: f32,
    pub hem: f32,
    pub back: f32,
    pub x_at: (usize, V3),
}

impl Frame {
    pub fn hand(&self) -> f32 {
        self.fore + 0.07
    }
}

#[derive(Clone, Copy, Default)]
pub struct Pose {
    pub lift: f32,
    pub pitch: f32,
    pub roll: f32,
    pub slide: f32,
    pub pelvis: [f32; 3],
    pub torso: [f32; 3],
    pub head: [f32; 3],
    pub arm_l: [f32; 3],
    pub arm_r: [f32; 3],
    pub elbow_l: f32,
    pub elbow_r: f32,
    pub hip_l: [f32; 2],
    pub hip_r: [f32; 2],
    pub knee_l: f32,
    pub knee_r: f32,
    pub ankle_l: f32,
    pub ankle_r: f32,
    pub skirt: [f32; 3],
    pub x: [f32; 3],
}

pub fn rot(e: [f32; 3]) -> M4 {
    mat::mul(&mat::mul(&mat::rot_y(e[1]), &mat::rot_x(e[0])), &mat::rot_z(e[2]))
}

pub fn at(parent: &M4, offset: V3, r: &M4) -> M4 {
    mat::mul(&mat::mul(parent, &mat::translate(offset)), r)
}

pub fn ease(k: f32) -> f32 {
    let k = k.clamp(0.0, 1.0);
    k * k * (3.0 - 2.0 * k)
}

pub fn biped(f: &Frame, p: &Pose, x: f32, z: f32, heading: f32) -> [M4; SEGS] {
    let place = mat::mul(&mat::translate([x, 0.0, z]), &mat::rot_y(heading));
    let root = mat::mul(&mat::mul(&place, &mat::translate([0.0, 0.0, p.slide])), &mat::mul(&mat::rot_x(p.pitch), &mat::rot_z(p.roll)));
    let pelvis = at(&root, [0.0; 3], &rot(p.pelvis));
    let torso = at(&pelvis, [0.0, f.waist, 0.0], &rot(p.torso));
    let head = at(&torso, [0.0, f.neck, 0.0], &rot(p.head));
    let arm = |side: f32, a: [f32; 3]| at(&torso, [side * f.shoulder_x, f.chest, 0.0], &rot([a[0], a[2], a[1]]));
    let (arm_l, arm_r) = (arm(1.0, p.arm_l), arm(-1.0, p.arm_r));
    let fore_l = at(&arm_l, [0.0, -f.upper, 0.0], &mat::rot_x(p.elbow_l));
    let fore_r = at(&arm_r, [0.0, -f.upper, 0.0], &mat::rot_x(p.elbow_r));
    let thigh = |side: f32, a: [f32; 2]| at(&pelvis, [side * f.hip_x, 0.0, 0.0], &mat::mul(&mat::rot_z(side * a[1]), &mat::rot_x(a[0])));
    let (thigh_l, thigh_r) = (thigh(1.0, p.hip_l), thigh(-1.0, p.hip_r));
    let shin_l = at(&thigh_l, [0.0, -f.thigh, 0.0], &mat::rot_x(p.knee_l));
    let shin_r = at(&thigh_r, [0.0, -f.thigh, 0.0], &mat::rot_x(p.knee_r));
    let foot_l = at(&shin_l, [0.0, -f.shin, 0.0], &mat::rot_x(p.ankle_l));
    let foot_r = at(&shin_r, [0.0, -f.shin, 0.0], &mat::rot_x(p.ankle_r));
    let lamp = at(&fore_l, [0.0, -f.hand(), 0.0], &mat::IDENTITY);
    let skirt = at(&pelvis, [0.0; 3], &rot(p.skirt));
    let parent = |s: usize| [pelvis, torso, head][s.min(2)];
    let xseg = at(&parent(f.x_at.0), f.x_at.1, &rot(p.x));
    let mut m = [mat::IDENTITY; SEGS];
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
        (LAMP, lamp),
        (SKIRT, skirt),
        (FOOT_L, foot_l),
        (FOOT_R, foot_r),
        (X, xseg),
        (FX, hidden(&pelvis)),
    ]
    .iter()
    .for_each(|&(s, v)| m[s] = v);
    let probes = contacts(f);
    let low = probes.iter().map(|&(s, p)| mat::point(&m[s], p)[1]).fold(f32::MAX, f32::min);
    lift(&mut m, p.lift - low);
    m
}

fn contacts(f: &Frame) -> [(usize, V3); 13] {
    let (foot_seg_l, foot_seg_r, len) = (FOOT_L, FOOT_R, f.meta);
    let sole = |s: usize, z: f32| (s, [0.0, -len - f.sole, z]);
    [
        sole(foot_seg_l, 0.15),
        sole(foot_seg_l, -0.06),
        sole(foot_seg_r, 0.15),
        sole(foot_seg_r, -0.06),
        (SHIN_L, [0.0, 0.0, 0.07]),
        (SHIN_R, [0.0, 0.0, 0.07]),
        (PELVIS, [0.0, -0.05, -0.13]),
        (TORSO, [0.0, f.chest, -f.back]),
        (TORSO, [0.0, f.chest * 0.4, -f.back]),
        (HEAD, [0.0, 0.14, -0.12]),
        (HEAD, [0.0, 0.14, 0.13]),
        (FORE_L, [0.0, -f.hand(), 0.0]),
        (FORE_R, [0.0, -f.hand(), 0.0]),
    ]
}

pub fn lift(m: &mut [M4], dy: f32) {
    m.iter_mut().for_each(|s| s[13] += dy);
}

pub fn hide(m: &mut M4) {
    m[..12].fill(0.0);
}

pub fn hidden(m: &M4) -> M4 {
    let mut m = *m;
    hide(&mut m);
    m
}

pub fn noise(i: i32) -> f32 {
    let h = (i as u32).wrapping_mul(0x9E37_79B9).rotate_left(11).wrapping_mul(0x85EB_CA6B);
    (h >> 8) as f32 / (1u32 << 23) as f32 - 1.0
}

pub fn snap(t: f32, step: f32) -> f32 {
    (t / step).floor() * step
}
