use super::lamps::{self, Lamps};
use super::mat::{self, V3};
use super::mesh::{Mesh, Vert};
use super::props::{self, Parts, Sink};
use super::texture;
use crate::map::{Map, Tile};
use std::ops::Range;

pub const H: f32 = 2.6;
pub const HEAD: f32 = 2.1;
const SILL: f32 = 0.9;
const BASE: f32 = 0.12;
const MARGIN: f32 = 0.18;
const RECESS: f32 = 0.42;
const DIRS: [[i32; 2]; 4] = [[0, -1], [0, 1], [1, 0], [-1, 0]];
const ROWS: [f32; 5] = [0.0, BASE, SILL, HEAD, H];

pub struct Section {
    pub tex: u8,
    pub first: usize,
    pub count: usize,
}

pub struct RoomGeo {
    pub lo: V3,
    pub hi: V3,
    pub outdoor: bool,
    pub sections: Range<usize>,
}

pub struct Portal {
    pub x: u8,
    pub y: u8,
    pub rooms: u32,
    pub lo: V3,
    pub hi: V3,
}

pub struct Level {
    pub verts: Vec<Vert>,
    pub idx: Vec<u16>,
    pub rooms: Vec<RoomGeo>,
    pub sections: Vec<Section>,
    pub portals: Vec<Portal>,
    pub screens: Mesh,
}

fn solid(t: Tile) -> bool {
    matches!(t, Tile::Wall | Tile::Window)
}

fn open(t: Tile) -> bool {
    matches!(t, Tile::Floor | Tile::Door | Tile::Blocker)
}

fn portals(map: &Map) -> Vec<Portal> {
    (0..map.h)
        .flat_map(|y| (0..map.w).map(move |x| (x, y)))
        .filter(|&(x, y)| map.tile(x, y) == Tile::Door)
        .map(|(x, y)| {
            let rooms = DIRS
                .iter()
                .filter(|d| open(map.tile(x + d[0], y + d[1])))
                .filter_map(|d| lamps::room_near(map, x + d[0], y + d[1]))
                .fold(0u32, |m, r| m | 1 << r);
            let (fx, fy) = (x as f32, y as f32);
            Portal { x: x as u8, y: y as u8, rooms, lo: [fx, 0.0, fy], hi: [fx + 1.0, HEAD, fy + 1.0] }
        })
        .collect()
}

struct Builder<'a> {
    map: &'a Map<'a>,
    parts: Parts,
    screens: Mesh,
}

impl<'a> Builder<'a> {
    fn part(&mut self, room: u8, tex: &str) -> &mut Mesh {
        self.parts.entry((room, texture::id(tex))).or_default()
    }

    fn vface(&mut self, room: u8, tex: &str, a: [f32; 2], f: [f32; 2], len: f32, h: [f32; 2]) {
        let r = [f[1], -f[0]];
        let b = [a[0] + r[0] * len, a[1] + r[1] * len];
        let u0 = (a[0] * r[0] + a[1] * r[1]).rem_euclid(1.0);
        let p = [[a[0], h[0], a[1]], [b[0], h[0], b[1]], [b[0], h[1], b[1]], [a[0], h[1], a[1]]];
        let uv = [[u0, H - h[0]], [u0 + len, H - h[0]], [u0 + len, H - h[1]], [u0, H - h[1]]];
        self.part(room, tex).quad_facing(p, uv, [f[0], 0.0, f[1]]);
    }

    fn hface(&mut self, room: u8, tex: &str, lo: [f32; 2], hi: [f32; 2], y: f32, up: bool) {
        let p = [[lo[0], y, hi[1]], [hi[0], y, hi[1]], [hi[0], y, lo[1]], [lo[0], y, lo[1]]];
        let (u0, v0) = (lo[0].rem_euclid(1.0), lo[1].rem_euclid(1.0));
        let (du, dv) = (hi[0] - lo[0], hi[1] - lo[1]);
        let uv = [[u0, v0 + dv], [u0 + du, v0 + dv], [u0 + du, v0], [u0, v0]];
        self.part(room, tex).quad_facing(p, uv, [0.0, if up { 1.0 } else { -1.0 }, 0.0]);
    }

    fn room_tex(&self, room: u8) -> (&'a str, &'a str, Option<&'a str>, bool) {
        let r = &self.map.rooms[room as usize];
        (r.floor, r.wall, r.ceil, r.ceil.is_none())
    }

    fn grid(&mut self, room: u8, a: [f32; 2], f: [f32; 2], cols: &[f32], rows: &[f32], hole: [f32; 4]) {
        let (_, wall, _, outdoor) = self.room_tex(room);
        let r = [f[1], -f[0]];
        cols.windows(2).for_each(|c| {
            rows.windows(2)
                .filter(|h| !(c[0] >= hole[0] && c[1] <= hole[1] && h[0] >= hole[2] && h[1] <= hole[3]))
                .for_each(|h| {
                    let tex = if !outdoor && h[1] <= BASE { "madeira_escura" } else { wall };
                    self.vface(room, tex, [a[0] + r[0] * c[0], a[1] + r[1] * c[0]], f, c[1] - c[0], [h[0], h[1]]);
                })
        });
    }

    fn wall_side(&mut self, x: i32, y: i32, d: [i32; 2], window: bool) {
        let Some(room) = lamps::room_near(self.map, x + d[0], y + d[1]) else { return };
        let f = [d[0] as f32, d[1] as f32];
        let r = [f[1], -f[0]];
        let c = [x as f32 + 0.5 + f[0] * 0.5, y as f32 + 0.5 + f[1] * 0.5];
        let a = [c[0] - r[0] * 0.5, c[1] - r[1] * 0.5];
        if !window {
            return self.grid(room, a, f, &[0.0, 1.0], &ROWS, [2.0; 4]);
        }
        let (_, wall, _, _) = self.room_tex(room);
        let along = |k: i32| self.map.tile(x + d[1] * k, y - d[0] * k) == Tile::Window;
        let (ml, mr) = (if along(-1) { 0.0 } else { MARGIN }, if along(1) { 0.0 } else { MARGIN });
        let at = |k: f32, depth: f32| [a[0] + r[0] * k - f[0] * depth, a[1] + r[1] * k - f[1] * depth];
        let mut cols = vec![0.0, ml, 1.0 - mr, 1.0];
        cols.dedup();
        self.grid(room, a, f, &cols, &ROWS, [ml, 1.0 - mr, SILL, HEAD]);
        if ml > 0.0 {
            self.vface(room, wall, at(ml, 0.0), r, RECESS, [SILL, HEAD]);
        }
        if mr > 0.0 {
            self.vface(room, wall, at(1.0 - mr, RECESS), [-r[0], -r[1]], RECESS, [SILL, HEAD]);
        }
        let (p0, p1) = (at(ml, 0.0), at(1.0 - mr, RECESS));
        let (lo, hi) = ([p0[0].min(p1[0]), p0[1].min(p1[1])], [p0[0].max(p1[0]), p0[1].max(p1[1])]);
        self.hface(room, "madeira_escura", lo, hi, SILL, true);
        self.hface(room, wall, lo, hi, HEAD, false);
        let w = 1.0 - ml - mr;
        let rp = |k: f32, depth: f32| {
            let p = at(ml + k, RECESS - depth);
            move |h: f32| [p[0], h, p[1]]
        };
        let glass = [rp(0.0, 0.0)(SILL), rp(w, 0.0)(SILL), rp(w, 0.0)(HEAD), rp(0.0, 0.0)(HEAD)];
        let u = |k: f32| (ml + k) * 0.5;
        self.part(room, "vidro_noite").quad_facing(glass, [[u(0.0), 1.0], [u(w), 1.0], [u(w), 0.0], [u(0.0), 0.0]], [f[0], 0.0, f[1]]);
        let mid = (SILL + HEAD) * 0.5;
        let post = 0.5 - ml - 0.025;
        self.vface(room, "madeira_escura", at(ml + post, RECESS - 0.012), f, 0.05, [SILL, HEAD]);
        let bar = [rp(0.0, 0.012)(mid - 0.025), rp(w, 0.012)(mid - 0.025), rp(w, 0.012)(mid + 0.025), rp(0.0, 0.012)(mid + 0.025)];
        self.part(room, "madeira_escura").quad_facing(bar, [[0.0, 0.05], [w, 0.05], [w, 0.0], [0.0, 0.0]], [f[0], 0.0, f[1]]);
    }

    fn door_tile(&mut self, x: i32, y: i32) {
        let horizontal = solid(self.map.tile(x - 1, y)) && solid(self.map.tile(x + 1, y));
        let dirs: [[i32; 2]; 2] = if horizontal { [[0, -1], [0, 1]] } else { [[-1, 0], [1, 0]] };
        dirs.iter().for_each(|d| {
            let Some(room) = lamps::room_near(self.map, x + d[0], y + d[1]) else { return };
            let f = [d[0] as f32, d[1] as f32];
            let r = [f[1], -f[0]];
            let c = [x as f32 + 0.5 + f[0] * 0.5, y as f32 + 0.5 + f[1] * 0.5];
            self.grid(room, [c[0] - r[0] * 0.5, c[1] - r[1] * 0.5], f, &[0.0, 1.0], &[HEAD, H], [2.0; 4]);
        });
        if let Some(room) = lamps::room_near(self.map, x, y) {
            let (floor, wall, _, _) = self.room_tex(room);
            let (lo, hi) = ([x as f32, y as f32], [x as f32 + 1.0, y as f32 + 1.0]);
            self.hface(room, floor, lo, hi, 0.0, true);
            self.hface(room, wall, lo, hi, HEAD, false);
        }
    }

    fn floor_tile(&mut self, x: i32, y: i32) {
        let Some(room) = lamps::room_near(self.map, x, y) else { return };
        let (floor, _, ceil, _) = self.room_tex(room);
        let (lo, hi) = ([x as f32, y as f32], [x as f32 + 1.0, y as f32 + 1.0]);
        self.hface(room, floor, lo, hi, 0.0, true);
        ceil.into_iter().for_each(|c| self.hface(room, c, lo, hi, H, false));
    }

    fn wall_tile(&mut self, x: i32, y: i32) {
        let window = self.map.tile(x, y) == Tile::Window;
        DIRS.iter().filter(|d| open(self.map.tile(x + d[0], y + d[1]))).for_each(|d| self.wall_side(x, y, *d, window));
        let outdoor = (-1..=1)
            .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
            .filter_map(|(dx, dy)| self.map.room_at(x + dx, y + dy))
            .find(|r| r.ceil.is_none())
            .and_then(|r| self.map.rooms.iter().position(|q| q.id == r.id));
        outdoor.into_iter().for_each(|room| {
            let (_, wall, _, _) = self.room_tex(room as u8);
            self.hface(room as u8, wall, [x as f32, y as f32], [x as f32 + 1.0, y as f32 + 1.0], H, true);
        });
    }

    fn props(&mut self) {
        let map = self.map;
        let support = |i: usize, x: f32, z: f32| {
            map.props
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .filter_map(|(_, p)| props::surface(p.kind).map(|s| (p, s)))
                .filter(|(p, (half, _))| {
                    let (a, c) = (p.rot.to_rad(), [p.pos.x.to_f32(), p.pos.y.to_f32()]);
                    let d = [x - c[0], z - c[1]];
                    let lx = d[0] * a.cos() - d[1] * a.sin();
                    let lz = d[0] * a.sin() + d[1] * a.cos();
                    lx.abs() <= half[0] && lz.abs() <= half[1]
                })
                .map(|(_, (_, top))| top)
                .fold(None, |a: Option<f32>, t| Some(a.map_or(t, |a| a.max(t))))
        };
        map.props.iter().enumerate().for_each(|(i, p)| {
            let (x, z) = (p.pos.x.to_f32(), p.pos.y.to_f32());
            let Some(room) = lamps::room_near(map, x.floor() as i32, z.floor() as i32) else { return };
            let base = support(i, x, z);
            let xf = mat::mul(&mat::translate([x, base.unwrap_or(0.0), z]), &mat::rot_y(p.rot.to_rad()));
            let mut sink = Sink { parts: &mut self.parts, screens: &mut self.screens, room, xf };
            props::build(p.kind, &mut sink, base.is_some());
        });
    }
}


impl Level {
    pub fn new<'a>(map: &'a Map<'a>, lamps: &Lamps) -> Level {
        let mut b = Builder { map, parts: Parts::new(), screens: Mesh::default() };
        (0..map.h).flat_map(|y| (0..map.w).map(move |x| (x, y))).for_each(|(x, y)| match map.tile(x, y) {
            Tile::Floor | Tile::Blocker => b.floor_tile(x, y),
            Tile::Door => b.door_tile(x, y),
            Tile::Wall | Tile::Window => b.wall_tile(x, y),
            Tile::Void => {}
        });
        b.props();
        let lit = |room: u8, m: &Mesh| -> Vec<Vert> {
            m.verts.iter().map(|v| v.with_light(lamps.light(room, v.pos, Some(v.normal())))).collect()
        };
        let mut level = Level {
            verts: Vec::new(),
            idx: Vec::new(),
            rooms: Vec::with_capacity(map.rooms.len()),
            sections: Vec::with_capacity(b.parts.len()),
            portals: portals(map),
            screens: b.screens,
        };
        level.screens.verts.iter_mut().for_each(|v| *v = v.with_light(1.0));
        (0..map.rooms.len() as u8).for_each(|room| {
            let start = level.sections.len();
            let (lo, hi) = b.parts.range((room, 0)..=(room, u8::MAX)).fold(([f32::MAX; 3], [f32::MIN; 3]), |acc, ((_, tex), m)| {
                let base = level.verts.len();
                level.sections.push(Section { tex: *tex, first: level.idx.len(), count: m.idx.len() });
                level.verts.extend(lit(room, m));
                level.idx.extend(m.idx.iter().map(|&i| (base + i as usize) as u16));
                m.verts.iter().fold(acc, |(lo, hi), v| {
                    (std::array::from_fn(|k| lo[k].min(v.pos[k])), std::array::from_fn(|k| hi[k].max(v.pos[k])))
                })
            });
            let outdoor = map.rooms[room as usize].ceil.is_none();
            level.rooms.push(RoomGeo { lo, hi, outdoor, sections: start..level.sections.len() });
        });
        assert!(level.verts.len() <= u16::MAX as usize, "level has {} vertices", level.verts.len());
        level
    }

    pub fn tris(&self) -> usize {
        self.idx.len() / 3
    }

    #[cfg(test)]
    pub fn room_tris(&self, room: usize) -> usize {
        self.sections[self.rooms[room].sections.clone()].iter().map(|s| s.count / 3).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    #[test]
    fn sobrado_builds_within_budget() {
        let m = map::sobrado();
        let l = Level::new(m, &Lamps::new(m));
        assert!(l.tris() > 1000 && l.tris() < 20000, "{} tris", l.tris());
        assert!(l.idx.iter().all(|&i| (i as usize) < l.verts.len()));
        assert_eq!(l.screens.tris(), 2 * m.props.iter().filter(|p| p.kind == "tv_tubo").count());
        (0..l.rooms.len()).for_each(|r| assert!(l.room_tris(r) < 2600, "room {r}: {}", l.room_tris(r)));
    }
}
