use super::super::mat::{self, M4, V3};
use super::super::mesh::Vert;
use super::locomotion::unpoint;
use super::{FOOT_L, FOOT_R, PELVIS, SEGS, SHIN_L, SHIN_R, SKIRT, THIGH_L, THIGH_R};

#[derive(Clone, Copy)]
pub struct Cloth {
    pub top: f32,
    pub hem: f32,
    pub waist: [f32; 2],
    pub edge: [f32; 2],
}

pub const DRESS: Cloth = Cloth { top: 0.08, hem: 0.56, waist: [0.17, 0.14], edge: [0.29, 0.26] };
pub const ROBE: Cloth = Cloth { top: 0.06, hem: 0.24, waist: [0.15, 0.12], edge: [0.2, 0.17] };
pub const SKIRTS: Cloth = Cloth { top: 0.07, hem: 0.79, waist: [0.16, 0.14], edge: [0.31, 0.29] };

const MIDDLE: f32 = 0.45;
const FLARE: [f32; 3] = [0.1, 0.5, 1.0];
const MAX_FLARE: f32 = 0.9;
const ROOM: f32 = 0.9;
const PRIOR: f32 = 2.0;
const SHIFT: [f32; 2] = [0.06, 0.25];
const CLEAR: f32 = 0.14;

pub fn leg(segment: usize) -> bool {
    matches!(segment, THIGH_L | THIGH_R | SHIN_L | SHIN_R | FOOT_L | FOOT_R)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    pub levels: [f32; 3],
    pub centre: [[f32; 2]; 3],
    pub radius: [[f32; 2]; 3],
}

impl Shape {
    pub fn at(&self, y: f32) -> ([f32; 2], [f32; 2]) {
        let (i, t) = span(&self.levels, y);
        let mix = |a: [f32; 2], b: [f32; 2]| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        (mix(self.centre[i], self.centre[i + 1]), mix(self.radius[i], self.radius[i + 1]))
    }
}

fn span(levels: &[f32; 3], y: f32) -> (usize, f32) {
    let i = (y < levels[1]) as usize;
    (i, ((levels[i] - y) / (levels[i] - levels[i + 1])).clamp(0.0, 1.0))
}

impl Cloth {
    pub fn ring(self, y: f32) -> (f32, [f32; 2]) {
        let w = ((self.top - y) / (self.top + self.hem)).clamp(0.0, 1.0);
        (w, std::array::from_fn(|i| self.waist[i] + (self.edge[i] - self.waist[i]) * w))
    }

    pub fn rest(self) -> [f32; 3] {
        [self.top, -self.hem * MIDDLE, -self.hem]
    }

    pub fn shape(self, mats: &[M4; SEGS], legs: impl Iterator<Item = V3> + Clone) -> Shape {
        let pelvis = &mats[PELVIS];
        let hem = if pelvis[5] > 0.8 { self.hem.min((pelvis[13] - CLEAR) / pelvis[5]).max(0.1) } else { self.hem };
        let levels = [self.top, -hem * MIDDLE, -hem];
        let base = self.rest().map(|y| self.ring(y).1);
        let sway = unpoint(pelvis, mat::point(&mats[SKIRT], [0.0, -hem, 0.0]));
        let inside = legs.filter(|p| p[1] < levels[0] && p[1] > levels[2]);
        let centre: [[f32; 2]; 3] = std::array::from_fn(|i| {
            let [x, z, w] = inside.clone().fold([0.0; 3], |[x, z, w], p| {
                let (j, t) = span(&levels, p[1]);
                let weight = if j + 1 == i { t } else if j == i { 1.0 - t } else { 0.0 };
                [x + p[0] * weight, z + p[2] * weight, w + weight]
            });
            let follow = (i > 0) as u8 as f32 / (w + PRIOR);
            let mean = [x * follow + sway[0] * FLARE[i], z * follow + sway[2] * FLARE[i]];
            [mean[0].clamp(-SHIFT[0], SHIFT[0]), mean[1].clamp(-SHIFT[1], SHIFT[1])]
        });
        let rigid = Shape { levels, centre, radius: base };
        let flare = |axis: usize, need: &dyn Fn(V3, [f32; 2], [f32; 2]) -> f32| {
            inside.clone().fold(0.0f32, |a, p| {
                let (i, t) = span(&levels, p[1]);
                let (c, r) = rigid.at(p[1]);
                let grow = (1.0 - t) * base[i][axis] * FLARE[i] + t * base[i + 1][axis] * FLARE[i + 1];
                a.max((need(p, c, r) - r[axis]) / grow)
            })
        };
        let grown = |a: f32, axis: usize, radius: [[f32; 2]; 3]| -> [[f32; 2]; 3] {
            std::array::from_fn(|i| {
                let mut r = radius[i];
                r[axis] = base[i][axis] * (1.0 + a.clamp(0.0, MAX_FLARE) * FLARE[i]);
                r
            })
        };
        let wide = flare(0, &|p, c, _| (p[0] - c[0]).abs() / (ROOM * 0.8));
        let radius: [[f32; 2]; 3] = grown(wide, 0, base);
        let sideways = Shape { radius, ..rigid };
        let deep = flare(1, &|p, _, _| {
            let (c, r) = sideways.at(p[1]);
            let x = ((p[0] - c[0]) / r[0]).min(ROOM * 0.8);
            (p[2] - c[1]).abs() / (ROOM * ROOM - x * x).sqrt()
        });
        Shape { radius: grown(deep, 1, radius), ..sideways }
    }

    pub fn drape(self, mats: &[M4; SEGS], source: &[Vert], out: &mut [Vert], rank: &[u8], cut: u32) {
        let pelvis = &mats[PELVIS];
        let legs = source.iter().zip(out.iter()).filter(|(v, _)| leg(v.nrm[3] as usize)).map(|(_, v)| unpoint(pelvis, v.pos));
        let shape = self.shape(mats, legs);
        let rest = self.rest();
        source.iter().zip(out.iter_mut()).zip(rank).filter(|((v, _), rank)| v.nrm[3] as usize == SKIRT && **rank as u32 >= cut).for_each(|((v, out), _)| {
            let (i, t) = span(&rest, v.pos[1]);
            let y = shape.levels[i] + (shape.levels[i + 1] - shape.levels[i]) * t;
            let (c, r) = shape.at(y);
            let (_, b) = self.ring(v.pos[1]);
            let p: V3 = [c[0] + v.pos[0] * r[0] / b[0], y, c[1] + v.pos[2] * r[1] / b[1]];
            out.pos = mat::point(pelvis, p);
            out.pos[1] = out.pos[1].max(0.003);
        });
    }
}
