mod cabra;
mod corpo;
mod heroes;
mod kit;
mod matinta;
mod motion;
mod mula;
mod skeleton;

use super::mat::{self, M4, V3};
use super::mesh::{Mesh, Vert};
use crate::map::LegendKind;
use crate::sim::{Anim, LegendAnim};

pub const SEGS: usize = 16;
pub const PELVIS: usize = 0;
pub const TORSO: usize = 1;
pub const HEAD: usize = 2;
pub const ARM_L: usize = 3;
pub const FORE_L: usize = 4;
pub const ARM_R: usize = 5;
pub const FORE_R: usize = 6;
pub const THIGH_L: usize = 7;
pub const SHIN_L: usize = 8;
pub const THIGH_R: usize = 9;
pub const SHIN_R: usize = 10;
pub const LAMP: usize = 11;
pub const SKIRT: usize = 12;
pub const FOOT_L: usize = 13;
pub const FOOT_R: usize = 14;
pub const X: usize = 15;

pub const HEROES: u8 = 4;
const LEGENDS: [LegendKind; 4] = [LegendKind::CorpoSeco, LegendKind::Cabra, LegendKind::Mula, LegendKind::Matinta];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Who {
    Hero(u8),
    Legend(LegendKind),
}

impl Who {
    fn index(self) -> usize {
        match self {
            Who::Hero(c) => (c % HEROES) as usize,
            Who::Legend(k) => HEROES as usize + LEGENDS.iter().position(|&l| l == k).unwrap_or(0),
        }
    }
}

pub struct Model {
    pub mesh: Mesh,
    rest: Vec<V3>,
    rank: Vec<u8>,
}

impl Model {
    fn new(kit: kit::Kit) -> Model {
        let rest = kit.mesh.verts.iter().map(|v| mat::norm(v.normal())).collect();
        Model { mesh: kit.mesh, rest, rank: kit.rank }
    }

    pub fn skin(&self, mats: &[M4; SEGS], light: impl Fn(V3) -> f32, fade: f32, out: &mut [Vert]) {
        let q = |v: f32| (v * 127.0).round().clamp(-127.0, 127.0) as i8;
        let cut = (fade.clamp(0.0, 1.0) * 256.0) as u32;
        out.iter_mut().zip(&self.mesh.verts).zip(self.rest.iter().zip(&self.rank)).for_each(|((o, v), (&n, &r))| {
            let m = &mats[v.nrm[3] as usize];
            let n = mat::dir(m, n);
            let pos = match (r as u32) < cut {
                true => [m[12], m[13], m[14]],
                false => mat::point(m, v.pos),
            };
            *o = Vert { pos, nrm: [q(n[0]), q(n[1]), q(n[2]), q(light(n).clamp(0.0, 1.0))], ..*v };
        });
    }
}

pub struct Cast {
    models: Vec<[Model; 2]>,
}

impl Cast {
    pub fn new(tile: impl Fn(&str) -> [u8; 4]) -> Cast {
        let tiles: &dyn Fn(&str) -> [u8; 4] = &tile;
        let build = |who: Who| [true, false].map(|hi| Model::new(model(who, hi, tiles)));
        Cast { models: (0..HEROES).map(Who::Hero).chain(LEGENDS.iter().map(|&k| Who::Legend(k))).map(build).collect() }
    }

    pub fn model(&self, who: Who, far: bool) -> &Model {
        &self.models[who.index()][far as usize]
    }

    pub fn max_verts(&self) -> usize {
        self.models.iter().flatten().map(|m| m.mesh.verts.len()).max().unwrap_or(0)
    }

    pub fn max_idx(&self) -> usize {
        self.models.iter().flatten().map(|m| m.mesh.idx.len()).max().unwrap_or(0)
    }
}

fn model<'a>(who: Who, hi: bool, tiles: &'a dyn Fn(&str) -> [u8; 4]) -> kit::Kit<'a> {
    let mut k = kit::Kit::new(hi, tiles);
    match who {
        Who::Hero(c) => heroes::build(c % HEROES, &mut k),
        Who::Legend(LegendKind::CorpoSeco) => corpo::build(&mut k),
        Who::Legend(LegendKind::Cabra) => cabra::build(&mut k),
        Who::Legend(LegendKind::Mula) => mula::build(&mut k),
        Who::Legend(LegendKind::Matinta) => matinta::build(&mut k),
    }
    k
}

#[derive(Clone, Copy)]
pub struct Rig {
    pub mats: [M4; SEGS],
    pub hands: [M4; 2],
    pub lantern: V3,
    pub beam: V3,
    pub head: (V3, V3),
    pub eyes: Option<[V3; 2]>,
    pub eye_color: [f32; 3],
    pub fire: Option<(V3, V3, f32)>,
}

pub fn hero(character: u8, x: f32, z: f32, heading: f32, anim: Anim, tic: u16, lantern: bool, look: f32) -> Rig {
    motion::rig(character % HEROES, x, z, heading, anim, tic, lantern, look)
}

pub fn legend(kind: LegendKind, x: f32, z: f32, heading: f32, anim: LegendAnim, tic: u16) -> Rig {
    match kind {
        LegendKind::CorpoSeco => corpo::rig(x, z, heading, anim, tic),
        LegendKind::Cabra => cabra::rig(x, z, heading, anim, tic),
        LegendKind::Mula => mula::rig(x, z, heading, anim, tic),
        LegendKind::Matinta => matinta::rig(x, z, heading, anim, tic),
    }
}

pub fn extent(who: Who) -> V3 {
    match who {
        Who::Hero(0) => [0.5, 0.8, 0.5],
        Who::Hero(1) => [0.45, 0.72, 0.45],
        Who::Hero(2) => [0.55, 0.98, 0.55],
        Who::Hero(_) => [0.55, 0.87, 0.55],
        Who::Legend(LegendKind::CorpoSeco) => [0.5, 1.0, 0.5],
        Who::Legend(LegendKind::Cabra) => [0.8, 1.2, 1.0],
        Who::Legend(LegendKind::Mula) => [0.9, 1.1, 1.4],
        Who::Legend(LegendKind::Matinta) => [0.5, 0.8, 0.5],
    }
}

pub fn portrait_cam(who: Who) -> (V3, V3, f32) {
    let rest = match who {
        Who::Hero(c) => hero(c, 0.0, 0.0, 0.0, Anim::Idle, 0, false, 0.0),
        Who::Legend(k) => legend(k, 0.0, 0.0, 0.0, LegendAnim::Idle, 0),
    };
    let face = rest.head.0;
    match who {
        Who::Legend(LegendKind::Mula) => ([-2.4, 1.5, 1.6], [0.0, 0.95, 0.1], 40.0),
        Who::Legend(LegendKind::Cabra) => ([-0.5, face[1] - 0.1, face[2] + 1.6], [face[0], face[1] - 0.3, face[2]], 40.0),
        _ => ([-0.3, face[1] + 0.02, face[2] + 1.05], [face[0], face[1] - 0.14, face[2]], 40.0),
    }
}

fn rig_from(mats: [M4; SEGS], hands: [M4; 2], lantern: bool, beam_from: &M4, head_off: V3) -> Rig {
    let lamp = &mats[LAMP];
    let head = &mats[HEAD];
    Rig {
        mats,
        hands,
        lantern: mat::point(lamp, [0.0, 0.0, 0.12]),
        beam: match lantern {
            true => mat::norm(mat::dir(beam_from, [0.0, -0.3, 1.0])),
            false => [0.0, 0.0, 1.0],
        },
        head: (mat::point(head, head_off), mat::norm(mat::dir(head, [0.0, 0.0, 1.0]))),
        eyes: None,
        eye_color: [0.0; 3],
        fire: None,
    }
}

#[cfg(test)]
mod tests;
