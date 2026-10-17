use super::lamps::Lamps;
use super::mat::{self, M4, V3};
use super::mesh::{Faces, Mesh, Pen, Vert};
use super::props::{self, Hinge, Mark, Parts, Sink};
use super::texture;
use super::zones::{self, DIRS, Mask, Zones};
use crate::assets::TEXTURES;
use crate::map::{HideKind, Map, PortalKind, Tile};
use std::collections::BTreeMap;
use std::ops::Range;

pub const HEAD: f32 = 2.1;
pub const WATER_Y: f32 = -0.14;
const SILL: f32 = 0.9;
const BASE: f32 = 0.12;
const MARGIN: f32 = 0.18;
const RECESS: f32 = 0.42;
const CHUNK: f32 = 3.0;
const CELL: f32 = 4.0;
const BANK: f32 = 0.45;
const RAIL: f32 = 1.1;
const WELL: f32 = 1.3;
const PAGE: usize = 65535;
const NOTE_Y: f32 = 1.45;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mat {
    Opaque,
    Alpha,
    Water,
}

pub fn mat_of(tex: u8) -> Mat {
    match TEXTURES[tex as usize].0 {
        "agua" | "agua_rio" => Mat::Water,
        n if texture::alpha(n) => Mat::Alpha,
        _ => Mat::Opaque,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Chunk {
    pub lo: V3,
    pub hi: V3,
    pub page: u16,
    pub first: usize,
    pub count: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Group {
    pub rooms: Mask,
    pub mat: Mat,
    pub chunk: Chunk,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Drive {
    Prop(u16),
    Lock(u8),
}

pub struct Mover {
    pub drive: Drive,
    pub hinge: Hinge,
    pub xf: M4,
    pub opaque: Mesh,
    pub alpha: Mesh,
    pub rooms: Mask,
    pub lo: V3,
    pub hi: V3,
}

pub struct Note {
    pub pos: V3,
    pub rooms: Mask,
}

#[derive(Default)]
pub struct Page {
    pub verts: Vec<Vert>,
    pub idx: Vec<u16>,
}

pub struct Level {
    pub pages: Vec<Page>,
    pub groups: Vec<Group>,
    pub passes: [Range<usize>; 3],
    pub screens: Mesh,
    pub screen_rooms: Vec<Mask>,
    pub marks: Vec<Mark>,
    pub movers: Vec<Mover>,
    pub notes: Vec<Note>,
    pub supports: Vec<Support>,
}

#[derive(Clone, Copy, Debug)]
pub struct Support {
    pub c: [f32; 2],
    pub rot: f32,
    pub half: [f32; 2],
    pub top: f32,
    pub prop: Option<u16>,
}

impl Support {
    pub fn holds(&self, x: f32, z: f32) -> bool {
        let d = [x - self.c[0], z - self.c[1]];
        let (s, c) = self.rot.sin_cos();
        (d[0] * c - d[1] * s).abs() <= self.half[0] && (d[0] * s + d[1] * c).abs() <= self.half[1]
    }
}

pub fn hide_kind(k: HideKind) -> &'static str {
    match k {
        HideKind::Cama => "cama",
        HideKind::GuardaRoupa => "guarda_roupa",
        HideKind::Armario => "armario",
        HideKind::Tumulo => "tumulo",
        HideKind::CaixaDagua => "caixa_dagua",
        HideKind::Confessionario => "confessionario",
        HideKind::Carro => "carro",
        HideKind::Cesto => "cesto",
        HideKind::Mesa => "mesa",
    }
}

struct Builder<'a> {
    map: &'a Map<'a>,
    z: &'a Zones,
    parts: Parts,
    roofs: Parts,
    screens: Mesh,
    marks: Vec<Mark>,
}

fn open(t: Tile) -> bool {
    zones::see_through(t) || t == Tile::Door
}

impl<'a> Builder<'a> {
    fn part(&mut self, room: u8, tex: &str) -> &mut Mesh {
        self.parts.entry((room, texture::id(tex))).or_default()
    }

    fn vface(&mut self, room: u8, tex: &str, a: [f32; 2], f: [f32; 2], len: f32, h: [f32; 2], top: f32) {
        let r = [f[1], -f[0]];
        let b = [a[0] + r[0] * len, a[1] + r[1] * len];
        let u0 = (a[0] * r[0] + a[1] * r[1]).rem_euclid(1.0);
        let p = [[a[0], h[0], a[1]], [b[0], h[0], b[1]], [b[0], h[1], b[1]], [a[0], h[1], a[1]]];
        let uv = [[u0, top - h[0]], [u0 + len, top - h[0]], [u0 + len, top - h[1]], [u0, top - h[1]]];
        self.part(room, tex).quad_facing(p, uv, [f[0], 0.0, f[1]]);
    }

    fn hface_in(parts: &mut Parts, room: u8, tex: &str, lo: [f32; 2], hi: [f32; 2], y: f32, up: bool) {
        let p = [[lo[0], y, hi[1]], [hi[0], y, hi[1]], [hi[0], y, lo[1]], [lo[0], y, lo[1]]];
        let (u0, v0) = (lo[0].rem_euclid(1.0), lo[1].rem_euclid(1.0));
        let (du, dv) = (hi[0] - lo[0], hi[1] - lo[1]);
        let uv = [[u0, v0 + dv], [u0 + du, v0 + dv], [u0 + du, v0], [u0, v0]];
        parts.entry((room, texture::id(tex))).or_default().quad_facing(p, uv, [0.0, if up { 1.0 } else { -1.0 }, 0.0]);
    }

    fn hface(&mut self, room: u8, tex: &str, lo: [f32; 2], hi: [f32; 2], y: f32, up: bool) {
        Self::hface_in(&mut self.parts, room, tex, lo, hi, y, up);
    }

    fn style(&self, x: i32, y: i32, viewer: u8) -> (&'a str, f32, bool) {
        let r = &self.map.rooms[viewer as usize];
        if !r.out {
            return (r.wall, r.h.to_f32(), true);
        }
        let inside = (-1..=1)
            .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
            .filter_map(|(dx, dy)| self.z.room(x + dx, y + dy))
            .map(|i| &self.map.rooms[i as usize])
            .find(|i| !i.out);
        inside.map_or((r.wall, r.h.to_f32(), false), |i| (i.ext.unwrap_or(r.wall), i.h.to_f32(), false))
    }

    fn grid(&mut self, room: u8, tex: &str, base: bool, a: [f32; 2], f: [f32; 2], cols: &[f32], rows: &[f32], hole: [f32; 4], top: f32) {
        let r = [f[1], -f[0]];
        cols.windows(2).for_each(|c| {
            rows.windows(2)
                .filter(|h| h[1] > h[0])
                .filter(|h| !(c[0] >= hole[0] && c[1] <= hole[1] && h[0] >= hole[2] && h[1] <= hole[3]))
                .for_each(|h| {
                    let t = if base && h[1] <= BASE { "madeira_escura" } else { tex };
                    self.vface(room, t, [a[0] + r[0] * c[0], a[1] + r[1] * c[0]], f, c[1] - c[0], [h[0], h[1]], top);
                })
        });
    }

    fn wall_point(a: [f32; 2], f: [f32; 2], top: f32) -> impl Fn(f32, f32) -> (V3, [f32; 2]) {
        let r = [f[1], -f[0]];
        let u0 = (a[0] * r[0] + a[1] * r[1]).rem_euclid(1.0);
        move |k, h| ([a[0] + r[0] * k, h, a[1] + r[1] * k], [u0 + k, top - h])
    }

    fn fan(&mut self, room: u8, tex: &str, pts: &[(V3, [f32; 2])], n: V3) {
        let c = mat::cross(mat::sub(pts[1].0, pts[0].0), mat::sub(pts[2].0, pts[0].0));
        let order: Vec<(V3, [f32; 2])> = match mat::dot(c, n) >= 0.0 {
            true => pts.to_vec(),
            false => pts[..1].iter().chain(pts[1..].iter().rev()).copied().collect(),
        };
        let (p, uv): (Vec<V3>, Vec<[f32; 2]>) = order.into_iter().unzip();
        self.part(room, tex).face(&p, &uv, n);
    }

    fn frame(&mut self, room: u8, tex: &str, a: [f32; 2], f: [f32; 2], y0: f32, hole: [f32; 4], top: f32) {
        let at = Self::wall_point(a, f, top);
        let outer = [at(0.0, y0), at(1.0, y0), at(1.0, top), at(0.0, top)];
        let inner = [at(hole[0], hole[2]), at(hole[1], hole[2]), at(hole[1], hole[3]), at(hole[0], hole[3])];
        let touching = [false, hole[1] >= 1.0, false, hole[0] <= 0.0];
        (0..4).filter(|&i| !touching[i]).for_each(|i| {
            let j = (i + 1) % 4;
            self.fan(room, tex, &[outer[i], outer[j], inner[j], inner[i]], [f[0], 0.0, f[1]]);
        });
    }

    fn notched(&mut self, room: u8, tex: &str, a: [f32; 2], f: [f32; 2], side: f32, h: [f32; 3]) {
        let at = Self::wall_point(a, f, h[2]);
        let far = 1.0 - side;
        self.fan(room, tex, &[at(side, h[1]), at(side, h[0]), at(far, h[0]), at(far, h[2]), at(side, h[2])], [f[0], 0.0, f[1]]);
    }

    fn wall_side(&mut self, x: i32, y: i32, d: [i32; 2], window: bool) -> Option<(u8, f32)> {
        let room = self.z.room(x + d[0], y + d[1])?;
        let (tex, top, base) = self.style(x, y, room);
        let f = [d[0] as f32, d[1] as f32];
        let r = [f[1], -f[0]];
        let c = [x as f32 + 0.5 + f[0] * 0.5, y as f32 + 0.5 + f[1] * 0.5];
        let a = [c[0] - r[0] * 0.5, c[1] - r[1] * 0.5];
        let out = self.map.rooms[room as usize].out.then_some((room, top));
        let y0 = if base { BASE } else { 0.0 };
        if base {
            self.vface(room, "madeira_escura", a, f, 1.0, [0.0, BASE], top);
        }
        if !window {
            let door = |k: i32| self.map.tile(x + d[1] * k, y - d[0] * k) == Tile::Door;
            let jamb = self.map.tile(x + d[0], y + d[1]) == Tile::Door;
            match (door(-1), door(1)) {
                (left, right) if jamb || (left && right) => self.grid(room, tex, false, a, f, &[0.0, 1.0], &[y0, HEAD, top], [2.0; 4], top),
                (false, false) => self.vface(room, tex, a, f, 1.0, [y0, top], top),
                (left, _) => self.notched(room, tex, a, f, if left { 0.0 } else { 1.0 }, [y0, HEAD, top]),
            }
            return out;
        }
        let along = |k: i32| self.map.tile(x + d[1] * k, y - d[0] * k) == Tile::Window;
        let (ml, mr) = (if along(-1) { 0.0 } else { MARGIN }, if along(1) { 0.0 } else { MARGIN });
        let at = |k: f32, depth: f32| [a[0] + r[0] * k - f[0] * depth, a[1] + r[1] * k - f[1] * depth];
        self.frame(room, tex, a, f, y0, [ml, 1.0 - mr, SILL, HEAD], top);
        if ml > 0.0 {
            self.vface(room, tex, at(ml, 0.0), r, RECESS, [SILL, HEAD], top);
        }
        if mr > 0.0 {
            self.vface(room, tex, at(1.0 - mr, RECESS), [-r[0], -r[1]], RECESS, [SILL, HEAD], top);
        }
        let (p0, p1) = (at(ml, 0.0), at(1.0 - mr, RECESS));
        let (lo, hi) = ([p0[0].min(p1[0]), p0[1].min(p1[1])], [p0[0].max(p1[0]), p0[1].max(p1[1])]);
        self.hface(room, "madeira_escura", lo, hi, SILL, true);
        self.hface(room, tex, lo, hi, HEAD, false);
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
        self.vface(room, "madeira_escura", at(ml + post, RECESS - 0.012), f, 0.05, [SILL, HEAD], top);
        let bar = [rp(0.0, 0.012)(mid - 0.025), rp(w, 0.012)(mid - 0.025), rp(w, 0.012)(mid + 0.025), rp(0.0, 0.012)(mid + 0.025)];
        self.part(room, "madeira_escura").quad_facing(bar, [[0.0, 0.05], [w, 0.05], [w, 0.0], [0.0, 0.0]], [f[0], 0.0, f[1]]);
        out
    }

    fn roof_of(&self, room: u8) -> Option<u8> {
        let r = &self.map.rooms[room as usize];
        (!r.out && r.ceil.is_some() && self.map.rooms.iter().any(|o| o.out && o.area == r.area)).then_some(room)
    }

    fn roof(&mut self, room: u8, x: i32, y: i32, h: f32) {
        Self::hface_in(&mut self.roofs, room, "telha", [x as f32, y as f32], [x as f32 + 1.0, y as f32 + 1.0], h, true);
    }

    fn wall_tile(&mut self, x: i32, y: i32) {
        let window = self.map.tile(x, y) == Tile::Window;
        let faces: Vec<(u8, f32)> = DIRS.iter().filter(|d| open(self.map.tile(x + d[0], y + d[1]))).filter_map(|d| self.wall_side(x, y, *d, window)).collect();
        let lo = [x as f32, y as f32];
        let hi = [lo[0] + 1.0, lo[1] + 1.0];
        match faces.iter().copied().max_by(|a, b| a.1.total_cmp(&b.1)) {
            Some((room, top)) => {
                let (tex, _, _) = self.style(x, y, room);
                self.hface(room, tex, lo, hi, top, true);
            }
            None => {
                let inner = (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (dx, dy))).filter_map(|(dx, dy)| self.z.room(x + dx, y + dy)).filter_map(|r| self.roof_of(r));
                if let Some((r, h)) = inner.map(|r| (r, self.map.rooms[r as usize].h.to_f32())).max_by(|a, b| a.1.total_cmp(&b.1)) {
                    self.roof(r, x, y, h);
                }
            }
        }
    }

    fn door_tile(&mut self, x: i32, y: i32) {
        let horizontal = zones::solid(self.map.tile(x - 1, y)) && zones::solid(self.map.tile(x + 1, y));
        let dirs: [[i32; 2]; 2] = if horizontal { [[0, -1], [0, 1]] } else { [[-1, 0], [1, 0]] };
        dirs.iter().for_each(|d| {
            let Some(room) = self.z.room(x + d[0], y + d[1]) else { return };
            let (tex, top, _) = self.style(x, y, room);
            let f = [d[0] as f32, d[1] as f32];
            let r = [f[1], -f[0]];
            let c = [x as f32 + 0.5 + f[0] * 0.5, y as f32 + 0.5 + f[1] * 0.5];
            self.grid(room, tex, false, [c[0] - r[0] * 0.5, c[1] - r[1] * 0.5], f, &[0.0, 1.0], &[HEAD, top], [2.0; 4], top);
        });
        if let Some(room) = self.z.room(x, y) {
            let r = &self.map.rooms[room as usize];
            let (lo, hi) = ([x as f32, y as f32], [x as f32 + 1.0, y as f32 + 1.0]);
            self.hface(room, r.floor, lo, hi, 0.0, true);
            self.hface(room, r.wall, lo, hi, HEAD, false);
            if let Some(roof) = self.roof_of(room) {
                self.roof(roof, x, y, r.h.to_f32());
            }
        }
    }

    fn floor_tile(&mut self, x: i32, y: i32, floor: bool) {
        let Some(room) = self.z.room(x, y) else { return };
        let r = &self.map.rooms[room as usize];
        let (lo, hi) = ([x as f32, y as f32], [x as f32 + 1.0, y as f32 + 1.0]);
        if floor {
            self.hface(room, r.floor, lo, hi, 0.0, true);
        }
        r.ceil.into_iter().for_each(|c| self.hface(room, c, lo, hi, r.h.to_f32(), false));
        if let Some(roof) = self.roof_of(room) {
            self.roof(roof, x, y, r.h.to_f32());
        }
    }

    fn water_tile(&mut self, x: i32, y: i32) {
        let Some(room) = self.z.room(x, y) else { return };
        let (lo, hi) = ([x as f32, y as f32], [x as f32 + 1.0, y as f32 + 1.0]);
        self.hface(room, "agua_rio", lo, hi, WATER_Y, true);
        let bank = if self.map.rooms[room as usize].out { "barro" } else { self.map.rooms[room as usize].floor };
        DIRS.iter().filter(|d| !matches!(self.map.tile(x + d[0], y + d[1]), Tile::Water | Tile::Void) && !zones::solid(self.map.tile(x + d[0], y + d[1]))).for_each(|d| {
            let f = [-d[0] as f32, -d[1] as f32];
            let r = [f[1], -f[0]];
            let c = [x as f32 + 0.5 - f[0] * 0.5, y as f32 + 0.5 - f[1] * 0.5];
            self.vface(room, bank, [c[0] - r[0] * 0.5, c[1] - r[1] * 0.5], f, 1.0, [WATER_Y - BANK, 0.0], 0.0);
        });
        let r = &self.map.rooms[room as usize];
        r.ceil.into_iter().for_each(|c| self.hface(room, c, lo, hi, r.h.to_f32(), false));
    }

    fn fence_tile(&mut self, x: i32, y: i32) {
        self.floor_tile(x, y, true);
        let Some(room) = self.z.room(x, y) else { return };
        let joins = |dx: i32, dy: i32| matches!(self.map.tile(x + dx, y + dy), Tile::Fence | Tile::Wall | Tile::Window | Tile::Door);
        let c = [x as f32 + 0.5, y as f32 + 0.5];
        let mut segs: Vec<([f32; 2], [f32; 2])> = DIRS.iter().filter(|d| joins(d[0], d[1])).map(|d| (c, [c[0] + d[0] as f32 * 0.5, c[1] + d[1] as f32 * 0.5])).collect();
        if segs.is_empty() {
            segs.push(([c[0] - 0.5, c[1]], [c[0] + 0.5, c[1]]));
        }
        segs.iter().for_each(|(a, b)| {
            let len = ((b[0] - a[0]).abs() + (b[1] - a[1]).abs()).max(0.01);
            let u0 = (a[0] + a[1]).rem_euclid(1.0);
            let q = [[a[0], 0.0, a[1]], [b[0], 0.0, b[1]], [b[0], RAIL, b[1]], [a[0], RAIL, a[1]]];
            let uv = [[u0, 1.0], [u0 + len, 1.0], [u0 + len, 0.0], [u0, 0.0]];
            let m = self.part(room, "grade_a");
            let n = [b[1] - a[1], 0.0, a[0] - b[0]];
            m.quad_facing(q, uv, n);
        });
        let mut post = Pen { mesh: self.part(room, "ferrugem"), xf: mat::IDENTITY, density: 2.0 };
        post.cuboid([c[0] - 0.03, 0.0, c[1] - 0.03], [c[0] + 0.03, RAIL + 0.05, c[1] + 0.03], Faces::NO_BOTTOM);
    }

    fn back(&self, x: i32, y: i32) -> [i32; 2] {
        let walk = DIRS.iter().find(|d| self.map.tile(x + d[0], y + d[1]).walkable()).copied();
        walk.map(|d| [-d[0], -d[1]]).filter(|d| zones::solid(self.map.tile(x + d[0], y + d[1]))).or_else(|| DIRS.iter().find(|d| zones::solid(self.map.tile(x + d[0], y + d[1]))).copied()).or(walk.map(|d| [-d[0], -d[1]])).unwrap_or([0, -1])
    }

    fn portal_tile(&mut self, x: i32, y: i32, movers: &mut Vec<(usize, M4, u8)>) {
        let Some(pi) = self.map.portal_at(x, y) else { return self.floor_tile(x, y, true) };
        let p = self.map.portals[pi];
        let Some(room) = self.z.room(x, y) else { return };
        let up = self.map.area_at(p.to_x, p.to_y).zip(self.map.area_at(x, y)).is_none_or(|(to, from)| to > from);
        let b = self.back(x, y);
        let yaw = (b[0] as f32).atan2(b[1] as f32);
        let xf = mat::mul(&mat::translate([x as f32 + 0.5, 0.0, y as f32 + 0.5]), &mat::rot_y(yaw));
        let well = !up && matches!(p.kind, PortalKind::Escada | PortalKind::EscadaMao);
        self.floor_tile(x, y, !well);
        let tex = self.map.rooms[room as usize].wall;
        let floor = self.map.rooms[room as usize].floor;
        let h = self.map.rooms[room as usize].h.to_f32();
        let mut sink = Sink { parts: &mut self.parts, screens: &mut self.screens, marks: &mut self.marks, room, xf };
        match p.kind {
            PortalKind::Escada | PortalKind::EscadaMao if !up => stairwell(&mut sink, tex, floor, p.kind == PortalKind::Escada),
            PortalKind::Escada => {}
            PortalKind::EscadaMao => ladder(&mut sink, 0.0, h),
            PortalKind::Bueiro => sink.pen("ferrugem").cylinder([0.0, 0.0, 0.0], 0.36, 0.36, 0.015, 8, true),
            PortalKind::Portao => {
                sink.pen("ferrugem").cuboid([-0.52, 0.0, 0.42], [-0.46, 2.2, 0.48], Faces::NO_BOTTOM);
                sink.pen("ferrugem").cuboid([0.46, 0.0, 0.42], [0.52, 2.2, 0.48], Faces::NO_BOTTOM);
                movers.push((pi, mat::mul(&xf, &mat::translate([0.0, 0.0, 0.45])), room));
            }
            PortalKind::JanelaAlta => {
                let _ = props::build("engradado", &mut Sink { xf: mat::mul(&xf, &mat::translate([0.0, 0.0, 0.25])), ..sink }, false);
                let mut s2 = Sink { parts: &mut self.parts, screens: &mut self.screens, marks: &mut self.marks, room, xf };
                props::wall_panel(&mut s2, "madeira_escura", "vidro_noite", [0.35, 0.3], 2.05, 0.04);
                Pen { mesh: s2.parts.entry((room, texture::id("madeira_escura"))).or_default(), xf: mat::mul(&xf, &mat::translate([0.0, 0.0, 0.5])), density: 2.0 }.cuboid([-0.35, 1.72, -0.06], [0.35, 1.75, 0.0], Faces::NO_BOTTOM);
            }
        }
    }

    fn shell(&mut self, gates: &mut Vec<(usize, M4, u8)>) {
        let map = self.map;
        (0..map.h).flat_map(|y| (0..map.w).map(move |x| (x, y))).for_each(|(x, y)| match map.tile(x, y) {
            Tile::Floor | Tile::Blocker => self.floor_tile(x, y, true),
            Tile::Hide => self.hide_tile(x, y),
            Tile::Portal => self.portal_tile(x, y, gates),
            Tile::Door => self.door_tile(x, y),
            Tile::Wall | Tile::Window => self.wall_tile(x, y),
            Tile::Water => self.water_tile(x, y),
            Tile::Fence => self.fence_tile(x, y),
            Tile::Void => {}
        });
    }

    fn hide_tile(&mut self, x: i32, y: i32) {
        self.floor_tile(x, y, true);
        let (Some(hi), Some(room)) = (self.map.hide_at(x, y), self.z.room(x, y)) else { return };
        let h = self.map.hides[hi];
        let mut xf = mat::mul(&mat::translate([x as f32 + 0.5, 0.0, y as f32 + 0.5]), &mat::rot_y(h.rot.to_rad()));
        if h.kind == HideKind::Tumulo {
            xf = mat::mul(&xf, &mat::scaling([1.0, 1.0, 0.45]));
        }
        let mut sink = Sink { parts: &mut self.parts, screens: &mut self.screens, marks: &mut self.marks, room, xf };
        let _ = props::build(hide_kind(h.kind), &mut sink, false);
        props::part(hide_kind(h.kind), &mut sink);
    }
}

fn stairwell(s: &mut Sink, wall: &str, floor: &str, steps: bool) {
    let mut w = s.pen(wall);
    let d = -WELL;
    w.quad([[-0.5, d, 0.5], [0.5, d, 0.5], [0.5, 0.0, 0.5], [-0.5, 0.0, 0.5]], [[0.0, 1.3], [1.0, 1.3], [1.0, 0.0], [0.0, 0.0]]);
    w.quad([[0.5, d, -0.5], [-0.5, d, -0.5], [-0.5, 0.0, -0.5], [0.5, 0.0, -0.5]], [[0.0, 1.3], [1.0, 1.3], [1.0, 0.0], [0.0, 0.0]]);
    w.quad([[0.5, d, 0.5], [0.5, d, -0.5], [0.5, 0.0, -0.5], [0.5, 0.0, 0.5]], [[0.0, 1.3], [1.0, 1.3], [1.0, 0.0], [0.0, 0.0]]);
    w.quad([[-0.5, d, -0.5], [-0.5, d, 0.5], [-0.5, 0.0, 0.5], [-0.5, 0.0, -0.5]], [[0.0, 1.3], [1.0, 1.3], [1.0, 0.0], [0.0, 0.0]]);
    let mut f = s.pen(floor);
    match steps {
        true => (0..5).for_each(|i| {
            let (z0, y) = (-0.5 + i as f32 * 0.2, -(i as f32 + 1.0) * WELL / 5.0);
            f.cuboid([-0.45, y, z0], [0.45, y + 0.04, 0.5], Faces(0x24));
        }),
        false => f.quad([[-0.5, d, 0.5], [0.5, d, 0.5], [0.5, d, -0.5], [-0.5, d, -0.5]], [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]),
    }
    if !steps {
        ladder(s, d, 0.6);
    }
    let mut r = s.pen("madeira_escura");
    [[-0.5f32, -0.46], [0.46, 0.5]].iter().for_each(|x| r.cuboid([x[0], 0.0, -0.5], [x[1], 0.95, 0.5], Faces(0x37)));
}

fn ladder(s: &mut Sink, y0: f32, y1: f32) {
    let mut p = s.pen("madeira_crua");
    [-0.25f32, 0.21].iter().for_each(|&x| p.cuboid([x, y0, 0.42], [x + 0.04, y1, 0.47], Faces(0x37)));
    let n = ((y1 - y0) / 0.3) as usize;
    (1..=n).for_each(|i| {
        let y = y0 + i as f32 * 0.3;
        p.cuboid([-0.21, y, 0.43], [0.21, y + 0.03, 0.46], Faces(0x3c));
    });
}

fn chunked(m: &Mesh) -> Vec<Mesh> {
    let mut cells: BTreeMap<(i32, i32), (Mesh, Vec<u16>)> = BTreeMap::new();
    m.idx.chunks(3).for_each(|t| {
        let c = t.iter().fold([0.0f32; 3], |a, &i| mat::add(a, m.verts[i as usize].pos));
        let key = ((c[0] / 3.0 / CHUNK).floor() as i32, (c[2] / 3.0 / CHUNK).floor() as i32);
        let (mesh, remap) = cells.entry(key).or_insert_with(|| (Mesh::default(), vec![u16::MAX; m.verts.len()]));
        t.iter().for_each(|&i| {
            if remap[i as usize] == u16::MAX {
                remap[i as usize] = mesh.verts.len() as u16;
                mesh.verts.push(m.verts[i as usize]);
            }
            mesh.idx.push(remap[i as usize]);
        });
    });
    cells.into_values().map(|(mesh, _)| mesh).collect()
}

fn bounds(verts: &[Vert]) -> (V3, V3) {
    verts.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), v| (std::array::from_fn(|k| lo[k].min(v.pos[k])), std::array::from_fn(|k| hi[k].max(v.pos[k]))))
}

fn bake(m: &Mesh, lamps: &Lamps, room: u8, tiles: &[[u8; 4]], tex: u8) -> Vec<Vert> {
    m.verts
        .iter()
        .map(|v| {
            let (moon, lit) = lamps.bake(room, v.pos, v.normal());
            v.with_light(moon).with_lit(lit).with_tile(tiles[tex as usize])
        })
        .collect()
}

fn prop_xf(x: f32, y: f32, z: f32, rot: f32) -> M4 {
    mat::mul(&mat::translate([x, y, z]), &mat::rot_y(rot))
}

impl Level {
    pub fn new(map: &Map, z: &Zones, lamps: &Lamps, tiles: &[[u8; 4]]) -> Level {
        let mut b = Builder { map, z, parts: Parts::new(), roofs: Parts::new(), screens: Mesh::default(), marks: Vec::new() };
        let mut gates = Vec::new();
        b.shell(&mut gates);
        let supports = supports(map);
        let mut movers = Vec::new();
        let mut screen_rooms = Vec::new();
        map.props.iter().enumerate().for_each(|(i, p)| {
            let (x, zz) = (p.pos.x.to_f32(), p.pos.y.to_f32());
            let Some(room) = z.at([x, 0.0, zz]) else { return };
            let base = supports.iter().filter(|s| s.prop != Some(i as u16) && s.holds(x, zz)).map(|s| s.top).fold(None, |a: Option<f32>, t| Some(a.map_or(t, |a| a.max(t))));
            let xf = prop_xf(x, base.unwrap_or(0.0), zz, p.rot.to_rad());
            let screens = b.screens.verts.len();
            let mut sink = Sink { parts: &mut b.parts, screens: &mut b.screens, marks: &mut b.marks, room, xf };
            let _ = props::build(p.kind, &mut sink, base.is_some());
            screen_rooms.extend((screens..b.screens.verts.len()).step_by(4).map(|_| 1 << room));
            let hinge = props::hinge(p.kind).filter(|_| p.item.is_some() || p.lock.is_some());
            match hinge {
                Some(hinge) => {
                    let mut parts = Parts::new();
                    props::part(p.kind, &mut Sink { parts: &mut parts, screens: &mut b.screens, marks: &mut b.marks, room, xf: mat::IDENTITY });
                    movers.push(mover(parts, Drive::Prop(i as u16), hinge, xf, room, lamps, tiles));
                }
                None => props::part(p.kind, &mut Sink { parts: &mut b.parts, screens: &mut b.screens, marks: &mut b.marks, room, xf }),
            }
        });
        gates.iter().for_each(|&(pi, xf, room)| {
            let mut parts = Parts::new();
            props::part("portao_ferro", &mut Sink { parts: &mut parts, screens: &mut b.screens, marks: &mut b.marks, room, xf: mat::IDENTITY });
            let hinge = props::hinge("portao_ferro").expect("gate hinge");
            let drive = map.portals[pi].lock.map_or(Drive::Prop(u16::MAX), Drive::Lock);
            movers.push(mover(parts, drive, hinge, xf, room, lamps, tiles));
        });
        let notes = map
            .notes
            .iter()
            .filter_map(|n| {
                let c = [n.x as f32 + 0.5, 0.0, n.y as f32 + 0.5];
                let wall = zones::solid(map.tile(n.x, n.y));
                let face = DIRS.iter().find(|d| map.tile(n.x + d[0], n.y + d[1]).walkable() || open(map.tile(n.x + d[0], n.y + d[1]))).copied().unwrap_or([0, 1]);
                let room = if wall { z.room(n.x + face[0], n.y + face[1]) } else { z.room(n.x, n.y) }?;
                let pos = match wall {
                    true => [c[0] + face[0] as f32 * 0.51, NOTE_Y, c[2] + face[1] as f32 * 0.51],
                    false => [c[0], supports.iter().filter(|s| s.holds(c[0], c[2])).map(|s| s.top).fold(0.0, f32::max) + 0.012, c[2]],
                };
                let (a, up) = match wall {
                    true => ((face[0] as f32).atan2(face[1] as f32), false),
                    false => (0.4, true),
                };
                let xf = mat::mul(&mat::translate(pos), &mat::rot_y(a));
                let mut p = Pen { mesh: b.parts.entry((room, texture::id("bilhete"))).or_default(), xf, density: 1.0 };
                match up {
                    true => p.quad([[-0.1, 0.0, 0.14], [0.1, 0.0, 0.14], [0.1, 0.0, -0.14], [-0.1, 0.0, -0.14]], [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]),
                    false => p.quad([[-0.1, -0.14, 0.0], [0.1, -0.14, 0.0], [0.1, 0.14, 0.0], [-0.1, 0.14, 0.0]], [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]),
                }
                Some(Note { pos, rooms: 1 << room })
            })
            .collect();
        let mut level = Level {
            pages: vec![Page::default()],
            groups: Vec::new(),
            passes: [0..0, 0..0, 0..0],
            screens: b.screens,
            screen_rooms,
            marks: b.marks,
            movers,
            notes,
            supports,
        };
        level.screens.verts.iter_mut().for_each(|v| *v = v.with_light(1.0));
        let mut units: Vec<(Mat, Mask, Mesh)> = Vec::new();
        b.parts.iter().for_each(|(&(room, tex), part)| {
            let lit = Mesh { verts: bake(part, lamps, room, tiles, tex), idx: part.idx.clone() };
            units.extend(chunked(&lit).into_iter().map(|m| (mat_of(tex), 1 << room, m)));
        });
        b.roofs.iter().for_each(|(&(room, tex), part)| {
            let area = map.rooms[room as usize].area;
            let outside = map.rooms.iter().enumerate().filter(|(_, r)| r.out && r.area == area).fold(0 as Mask, |m, (i, _)| m | 1 << i);
            let sky = outside.trailing_zeros() as u8;
            let lit = Mesh { verts: bake(part, lamps, sky, tiles, tex), idx: part.idx.clone() };
            units.extend(chunked(&lit).into_iter().map(|m| (Mat::Opaque, outside, m)));
        });
        let merged = merge(units);
        merged.into_iter().for_each(|(mat, rooms, m)| {
            let chunk = level.push(&m);
            level.groups.push(Group { rooms, mat, chunk });
        });
        level.groups.sort_by_key(|g| (g.mat, g.chunk.page));
        let at = |m: Mat| level.groups.iter().position(|g| g.mat >= m).unwrap_or(level.groups.len());
        let (a, w) = (at(Mat::Alpha), at(Mat::Water));
        level.passes = [0..a, a..w, w..level.groups.len()];
        level
    }

    fn push(&mut self, m: &Mesh) -> Chunk {
        if self.pages.last().is_some_and(|p| p.verts.len() + m.verts.len() > PAGE) {
            self.pages.push(Page::default());
        }
        let page = self.pages.len() - 1;
        let p = &mut self.pages[page];
        let base = p.verts.len();
        let (lo, hi) = bounds(&m.verts);
        let chunk = Chunk { lo, hi, page: page as u16, first: p.idx.len(), count: m.idx.len() };
        p.verts.extend_from_slice(&m.verts);
        p.idx.extend(m.idx.iter().map(|&i| (base + i as usize) as u16));
        chunk
    }

    #[cfg(test)]
    pub fn tris(&self) -> usize {
        self.pages.iter().map(|p| p.idx.len() / 3).sum()
    }

    pub fn rest(&self, x: f32, z: f32, open: impl Fn(u16) -> bool) -> f32 {
        self.supports.iter().filter(|s| s.holds(x, z) && s.prop.is_none_or(&open)).map(|s| s.top).fold(0.0, f32::max)
    }
}

fn merge(units: Vec<(Mat, Mask, Mesh)>) -> Vec<(Mat, Mask, Mesh)> {
    let mut cells: BTreeMap<(Mat, u128, i32, i32), Mesh> = BTreeMap::new();
    units.into_iter().for_each(|(mat, rooms, m)| {
        let (lo, hi) = bounds(&m.verts);
        let key = (mat, rooms, ((lo[0] + hi[0]) * 0.5 / CELL).floor() as i32, ((lo[2] + hi[2]) * 0.5 / CELL).floor() as i32);
        cells.entry(key).or_default().append(&m);
    });
    cells.into_iter().map(|((mat, rooms, _, _), m)| (mat, rooms, m)).collect()
}

fn mover(parts: Parts, drive: Drive, hinge: Hinge, xf: M4, room: u8, lamps: &Lamps, tiles: &[[u8; 4]]) -> Mover {
    let c = mat::point(&xf, [0.0, 0.8, 0.0]);
    let s = lamps.sample(room, c, None);
    let lit = super::lamps::Sample { electric: s.electric.map(|v| v.min(2.0)), flame: s.flame.min(2.0), moon: s.moon }.lit();
    let (mut opaque, mut alpha) = (Mesh::default(), Mesh::default());
    parts.iter().for_each(|(&(_, tex), m)| {
        let dst = if mat_of(tex) == Mat::Alpha { &mut alpha } else { &mut opaque };
        let base = dst.verts.len() as u16;
        dst.verts.extend(m.verts.iter().map(|v| v.with_light(s.moon).with_lit(lit).with_tile(tiles[tex as usize])));
        dst.idx.extend(m.idx.iter().map(|i| i + base));
    });
    let all: Vec<V3> = opaque.verts.iter().chain(&alpha.verts).flat_map(|v| [mat::point(&xf, v.pos), mat::point(&mat::mul(&xf, &hinge.at(1.0)), v.pos)]).collect();
    let (lo, hi) = all.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), p| (std::array::from_fn(|k| lo[k].min(p[k])), std::array::from_fn(|k| hi[k].max(p[k]))));
    Mover { drive, hinge, xf, opaque, alpha, rooms: 1 << room, lo, hi }
}

fn supports(map: &Map) -> Vec<Support> {
    map.props
        .iter()
        .enumerate()
        .flat_map(|(i, p)| {
            let c = [p.pos.x.to_f32(), p.pos.y.to_f32()];
            let rot = p.rot.to_rad();
            let surface = props::surface(p.kind).map(|(half, top)| Support { c, rot, half, top, prop: None });
            let inside = (props::content(p.kind) > 0.0 && p.item.is_some()).then(|| Support { c, rot, half: [0.2, 0.2], top: props::content(p.kind), prop: Some(i as u16) });
            surface.into_iter().chain(inside)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    fn build(m: &Map) -> Level {
        let z = Zones::new(m);
        let l = Lamps::new(m, &z);
        Level::new(m, &z, &l, &texture::atlas().tiles)
    }

    fn check(l: &Level) {
        l.pages.iter().for_each(|p| {
            assert!(p.verts.len() <= PAGE);
            assert!(p.idx.iter().all(|&i| (i as usize) < p.verts.len()));
            assert!(p.verts.iter().all(|v| v.tile[2] > 0 && v.tile[3] > 0));
        });
        l.groups.iter().for_each(|g| {
            let c = g.chunk;
            let p = &l.pages[c.page as usize];
            assert!(g.rooms != 0);
            assert!(p.idx[c.first..c.first + c.count].iter().all(|&i| (0..3).all(|k| (c.lo[k]..=c.hi[k]).contains(&p.verts[i as usize].pos[k]))));
        });
        assert!(l.passes[0].end <= l.passes[1].start && l.passes[1].end <= l.passes[2].start);
        l.groups[l.passes[1].clone()].iter().for_each(|g| assert_eq!(g.mat, Mat::Alpha));
        l.groups[l.passes[2].clone()].iter().for_each(|g| assert_eq!(g.mat, Mat::Water));
    }

    #[test]
    fn street_builds_water_fence_roofs_and_movers() {
        let m = map::parse(include_str!("testdata/rua.map")).expect("rua");
        let l = build(&m);
        check(&l);
        assert!(!l.passes[2].is_empty(), "water");
        assert!(!l.passes[1].is_empty(), "alpha fence and trees");
        assert!(l.groups.iter().any(|g| g.rooms.count_ones() > 1), "roofs seen from outside");
        assert!(l.marks.iter().any(|m| m.kind == props::MarkKind::Flame));
        assert!(l.tris() < 60000, "{}", l.tris());
    }

    #[test]
    fn sobrado_builds_within_budget() {
        let m = map::campaign()[0];
        let l = build(m);
        check(&l);
        assert!(l.tris() > 1000, "{}", l.tris());
        assert_eq!(l.screens.tris(), 2 * m.props.iter().filter(|p| p.kind == "tv_tubo").count());
        assert_eq!(l.screen_rooms.len() * 2, l.screens.tris());
    }

    #[test]
    fn supports_hold_inside_their_footprint() {
        let s = Support { c: [2.0, 2.0], rot: std::f32::consts::FRAC_PI_2, half: [1.0, 0.2], top: 0.8, prop: None };
        assert!(s.holds(2.0, 2.9) && !s.holds(2.9, 2.0));
    }
}
