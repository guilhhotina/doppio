use super::super::mat::{self, M4, V3};
use super::super::mesh::Mesh;
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy)]
pub enum Uv {
    Flat([f32; 2]),
    Wrap([f32; 4]),
    Tile(f32),
}

pub fn swatch(i: u8) -> Uv {
    Uv::Flat([(i % 4) as f32 * 0.25 + 0.125, (i / 4) as f32 * 0.25 + 0.125])
}

pub const BLACK: u8 = 0;
pub const LEATHER: u8 = 1;
pub const RED: u8 = 2;
pub const WHITE: u8 = 3;
pub const SKIN_DARK: u8 = 4;
pub const SKIN_TAN: u8 = 5;
pub const SKIN_FAIR: u8 = 6;
pub const BLUE: u8 = 8;
pub const OLIVE: u8 = 9;
pub const GREY: u8 = 10;
pub const BRASS: u8 = 11;
pub const WOOD: u8 = 12;
pub const BONE: u8 = 13;
pub const EMBER: u8 = 14;
pub const HOOF: u8 = 15;

#[derive(Clone, Copy)]
pub struct Ring {
    pub y: f32,
    pub rx: f32,
    pub rz: f32,
    pub dz: f32,
}

pub const fn ring(y: f32, rx: f32, rz: f32, dz: f32) -> Ring {
    Ring { y, rx, rz, dz }
}

pub fn egg(top: f32, bottom: f32, rx: f32, rz: f32, dz: f32, rings: usize) -> Vec<Ring> {
    (0..=rings)
        .map(|i| {
            let t = i as f32 / rings as f32 * PI;
            let s = t.sin();
            ring(top + (bottom - top) * (1.0 - t.cos()) * 0.5, rx * s, rz * s, dz * s)
        })
        .collect()
}

pub struct Kit<'a> {
    pub mesh: Mesh,
    pub rank: Vec<u8>,
    pub xf: M4,
    pub hi: bool,
    seg: u8,
    tile: [u8; 4],
    glow: u8,
    tiles: &'a dyn Fn(&str) -> [u8; 4],
}

impl<'a> Kit<'a> {
    pub fn new(hi: bool, tiles: &'a dyn Fn(&str) -> [u8; 4]) -> Kit<'a> {
        Kit { mesh: Mesh::default(), rank: Vec::new(), xf: mat::IDENTITY, hi, seg: 0, tile: [0; 4], glow: 0, tiles }
    }

    pub fn seg(&mut self, seg: usize) -> &mut Self {
        self.seg = seg as u8;
        self.xf = mat::IDENTITY;
        self.glow = 0;
        self
    }

    pub fn glow(&mut self, k: f32) -> &mut Self {
        self.glow = (k.clamp(0.0, 1.0) * 255.0) as u8;
        self
    }

    pub fn tex(&mut self, name: &str) -> &mut Self {
        self.tile = (self.tiles)(name);
        self
    }

    pub fn at(&mut self, xf: M4) -> &mut Self {
        self.xf = xf;
        self
    }

    pub fn n(&self, near: usize, far: usize) -> usize {
        if self.hi { near } else { far }
    }

    pub fn poly(&mut self, p: &[V3], uv: &[[f32; 2]]) {
        let w: Vec<V3> = p.iter().map(|&v| mat::point(&self.xf, v)).collect();
        let n = mat::norm(mat::cross(mat::sub(w[2], w[0]), mat::sub(w[w.len() - 1], w[1])));
        let base = self.mesh.verts.len();
        self.mesh.face(&w, uv, n);
        let (seg, tile, glow) = (self.seg as i8, self.tile, self.glow);
        self.mesh.verts[base..].iter_mut().for_each(|v| {
            *v = v.with_tile(tile).with_lit([0, 0, 0, glow]);
            v.nrm[3] = seg;
        });
        let rank = (base as u32).wrapping_mul(2654435761).rotate_right(13) as u8;
        self.rank.extend(std::iter::repeat_n(rank, p.len()));
    }

    pub fn card(&mut self, p: [V3; 4], uv: Uv, both: bool) {
        let t = corners(uv, [1.0, 1.0]);
        self.poly(&p, &t);
        if both {
            self.poly(&[p[3], p[2], p[1], p[0]], &[t[3], t[2], t[1], t[0]]);
        }
    }

    pub fn tri(&mut self, p: [V3; 3], uv: Uv) {
        let t = corners(uv, [1.0, 1.0]);
        self.poly(&p, &t[..3]);
    }

    pub fn cuboid(&mut self, lo: V3, hi: V3, uv: Uv) {
        let c = |x: usize, y: usize, z: usize| [[lo[0], hi[0]][x], [lo[1], hi[1]][y], [lo[2], hi[2]][z]];
        let (sx, sy, sz) = (hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]);
        let detail = self.hi;
        [
            ([c(1, 0, 1), c(1, 0, 0), c(1, 1, 0), c(1, 1, 1)], [sz, sy]),
            ([c(0, 0, 0), c(0, 0, 1), c(0, 1, 1), c(0, 1, 0)], [sz, sy]),
            ([c(0, 1, 1), c(1, 1, 1), c(1, 1, 0), c(0, 1, 0)], [sx, sz]),
            ([c(0, 0, 0), c(1, 0, 0), c(1, 0, 1), c(0, 0, 1)], [sx, sz]),
            ([c(0, 0, 1), c(1, 0, 1), c(1, 1, 1), c(0, 1, 1)], [sx, sy]),
            ([c(1, 0, 0), c(0, 0, 0), c(0, 1, 0), c(1, 1, 0)], [sx, sy]),
        ]
        .iter()
        .enumerate()
        .filter(|&(i, _)| detail || i != 3)
        .for_each(|(_, (p, size))| self.poly(p, &corners(uv, *size)));
    }

    pub fn lod(&self, rings: &[Ring]) -> Vec<Ring> {
        match (self.hi, rings.len()) {
            (true, _) | (false, 0..=2) => rings.to_vec(),
            (false, 3..=4) => vec![rings[0], rings[rings.len() - 1]],
            (false, n) => vec![rings[0], rings[n / 2], rings[n - 1]],
        }
    }

    pub fn loft(&mut self, rings: &[Ring], sides: usize, uv: Uv, gap: (usize, usize), cap: bool) {
        let pt = |r: &Ring, j: usize| {
            let a = -PI + TAU * j as f32 / sides as f32;
            [a.sin() * r.rx, r.y, a.cos() * r.rz + r.dz]
        };
        let lens: Vec<f32> = std::iter::once(0.0)
            .chain(rings.windows(2).scan(0.0, |acc, w| {
                *acc += (w[1].y - w[0].y).abs().max(((w[1].rx - w[0].rx).abs() + (w[1].rz - w[0].rz).abs()) * 0.5);
                Some(*acc)
            }))
            .collect();
        let girth = rings.iter().map(|r| r.rx.max(r.rz)).fold(0.0, f32::max) * TAU;
        let span = rings.last().map(|r| r.y - rings[0].y).filter(|d| d.abs() > 1e-4).unwrap_or(1.0);
        let at = |i: usize, j: usize| -> [f32; 2] {
            let s = j as f32 / sides as f32;
            match uv {
                Uv::Flat(c) => c,
                Uv::Wrap([u0, v0, u1, v1]) => [u0 + (u1 - u0) * s, v0 + (v1 - v0) * (rings[i].y - rings[0].y) / span],
                Uv::Tile(d) => [s * (girth * d).round().max(1.0), lens[i] * d],
            }
        };
        let up = rings.last().map(|r| r.y).unwrap_or(0.0) >= rings[0].y;
        (0..rings.len() - 1).for_each(|i| {
            (0..sides).filter(|j| (j + sides - gap.0) % sides >= gap.1 - gap.0).for_each(|j| {
                let quad = [(i, j), (i, j + 1), (i + 1, j + 1), (i + 1, j)];
                let quad = if up { quad } else { [quad[3], quad[2], quad[1], quad[0]] };
                let (p, t): (Vec<V3>, Vec<[f32; 2]>) = quad.iter().map(|&(r, c)| (pt(&rings[r], c), at(r, c))).unzip();
                let keep: Vec<usize> = (0..4).filter(|&k| mat::len(mat::sub(p[k], p[(k + 3) % 4])) > 1e-6).collect();
                if keep.len() >= 3 {
                    let p: Vec<V3> = keep.iter().map(|&k| p[k]).collect();
                    let t: Vec<[f32; 2]> = keep.iter().map(|&k| t[k]).collect();
                    self.poly(&p, &t);
                }
            })
        });
        if cap {
            let last = rings.len() - 1;
            let (r, flip) = (&rings[last], !up);
            let mut p: Vec<V3> = (0..sides).map(|j| pt(r, j)).collect();
            let c = match uv {
                Uv::Flat(c) => c,
                Uv::Wrap([u0, v0, u1, v1]) => [(u0 + u1) * 0.5, v0 + (v1 - v0) * if up { 0.98 } else { 0.02 }],
                Uv::Tile(_) => [0.0, 0.0],
            };
            if flip {
                p.reverse();
            }
            self.poly(&p, &vec![c; sides]);
        }
    }
}

fn corners(uv: Uv, size: [f32; 2]) -> [[f32; 2]; 4] {
    match uv {
        Uv::Flat(c) => [c; 4],
        Uv::Wrap([u0, v0, u1, v1]) => [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
        Uv::Tile(d) => {
            let (u, v) = (size[0] * d, size[1] * d);
            [[0.0, v], [u, v], [u, 0.0], [0.0, 0.0]]
        }
    }
}
