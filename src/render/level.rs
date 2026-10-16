use super::lamps::{self, Lamps};
use super::mat::{self, M4, V3};
use super::mesh::{Mesh, Vert};
use super::props::{self, Parts, Sink};
use super::texture;
use crate::map::{Map, TaskKind, Tile};
use std::collections::BTreeMap;
use std::ops::Range;

pub const H: f32 = 2.6;
pub const HEAD: f32 = 2.1;
const SILL: f32 = 0.9;
const BASE: f32 = 0.12;
const MARGIN: f32 = 0.18;
const RECESS: f32 = 0.42;
const DIRS: [[i32; 2]; 4] = [[0, -1], [0, 1], [1, 0], [-1, 0]];
const ROWS: [f32; 5] = [0.0, BASE, SILL, HEAD, H];
const CHUNK: f32 = 3.0;
const DRAPE_REACH: f32 = 1.5;

pub struct Chunk {
    pub lo: V3,
    pub hi: V3,
    pub first: usize,
    pub count: usize,
}

pub struct RoomGeo {
    pub outdoor: bool,
    pub chunks: Range<usize>,
}

pub struct Portal {
    pub x: u8,
    pub y: u8,
    pub rooms: u32,
    pub lo: V3,
    pub hi: V3,
}

pub struct Shutter {
    pub run: u8,
    pub room: u8,
    pub chunk: Chunk,
}

pub struct Drape {
    pub room: u8,
    pub xf: M4,
    pub chunk: Chunk,
}

pub struct Level {
    pub verts: Vec<Vert>,
    pub idx: Vec<u16>,
    pub rooms: Vec<RoomGeo>,
    pub chunks: Vec<Chunk>,
    pub portals: Vec<Portal>,
    pub screens: Mesh,
    pub shutters: Vec<Shutter>,
    pub drapes: Vec<Drape>,
    pub glows: Vec<(V3, u8)>,
    runs: Vec<i16>,
    width: i32,
}

fn solid(t: Tile) -> bool {
    matches!(t, Tile::Wall | Tile::Window)
}

fn open(t: Tile) -> bool {
    matches!(t, Tile::Floor | Tile::Door | Tile::Blocker)
}

fn window_runs(map: &Map) -> Vec<i16> {
    let mut runs = vec![-1i16; (map.w * map.h) as usize];
    let mut next = 0i16;
    (0..map.h).flat_map(|y| (0..map.w).map(move |x| (x, y))).for_each(|(x, y)| {
        if map.tile(x, y) != Tile::Window || runs[(y * map.w + x) as usize] >= 0 {
            return;
        }
        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            let at = (cy * map.w + cx) as usize;
            if map.tile(cx, cy) != Tile::Window || runs[at] >= 0 {
                continue;
            }
            runs[at] = next;
            stack.extend(DIRS.iter().map(|d| (cx + d[0], cy + d[1])));
        }
        next += 1;
    });
    assert!(next <= 64, "{next} window runs");
    runs
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
    shutters: Vec<(i32, i32, u8, Mesh)>,
    drapes: Vec<(u8, M4, Mesh)>,
    glows: Vec<(V3, u8)>,
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
        let mut leaves = Mesh::default();
        [(0.0, w * 0.5 - 0.01), (w * 0.5 + 0.01, w)].iter().for_each(|&(k0, k1)| {
            let q = [rp(k0, 0.03)(SILL + 0.02), rp(k1, 0.03)(SILL + 0.02), rp(k1, 0.03)(HEAD - 0.02), rp(k0, 0.03)(HEAD - 0.02)];
            let v = (HEAD - SILL) * 2.0;
            leaves.quad_facing(q, [[k0, v], [k1, v], [k1, 0.0], [k0, 0.0]], [f[0], 0.0, f[1]]);
        });
        self.shutters.push((x, y, room, leaves));
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
        let windowed = |x: f32, z: f32| {
            map.tasks.iter().filter(|t| t.kind == TaskKind::Window).any(|t| {
                let (dx, dz) = (t.pos.x.floor() as f32 + 0.5 - x, t.pos.y.floor() as f32 + 0.5 - z);
                dx * dx + dz * dz < DRAPE_REACH * DRAPE_REACH
            })
        };
        map.props.iter().enumerate().for_each(|(i, p)| {
            let (x, z) = (p.pos.x.to_f32(), p.pos.y.to_f32());
            let Some(room) = lamps::room_near(map, x.floor() as i32, z.floor() as i32) else { return };
            let base = support(i, x, z);
            let xf = mat::mul(&mat::translate([x, base.unwrap_or(0.0), z]), &mat::rot_y(p.rot.to_rad()));
            let mut sink = Sink { parts: &mut self.parts, screens: &mut self.screens, room, xf };
            match p.kind {
                "cortina" if windowed(x, z) => {
                    props::rod(&mut sink);
                    let mut cloth = Parts::new();
                    props::cloth(&mut Sink { parts: &mut cloth, screens: &mut self.screens, room, xf });
                    self.drapes.extend(cloth.into_values().map(|m| (room, xf, m)));
                }
                "abajur" => {
                    props::build(p.kind, &mut sink, base.is_some());
                    self.glows.push((mat::point(&xf, props::shade_centre(base.is_some())), room));
                }
                kind => props::build(kind, &mut sink, base.is_some()),
            }
        });
    }
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

impl Level {
    pub fn new<'a>(map: &'a Map<'a>, lamps: &Lamps, tiles: &[[u8; 4]]) -> Level {
        let mut b = Builder { map, parts: Parts::new(), screens: Mesh::default(), shutters: Vec::new(), drapes: Vec::new(), glows: Vec::new() };
        (0..map.h).flat_map(|y| (0..map.w).map(move |x| (x, y))).for_each(|(x, y)| match map.tile(x, y) {
            Tile::Floor | Tile::Blocker => b.floor_tile(x, y),
            Tile::Door => b.door_tile(x, y),
            Tile::Wall | Tile::Window => b.wall_tile(x, y),
            Tile::Void => {}
        });
        b.props();
        let mut level = Level {
            verts: Vec::new(),
            idx: Vec::new(),
            rooms: Vec::with_capacity(map.rooms.len()),
            chunks: Vec::new(),
            portals: portals(map),
            screens: b.screens,
            shutters: Vec::with_capacity(b.shutters.len()),
            drapes: Vec::with_capacity(b.drapes.len()),
            glows: b.glows,
            runs: window_runs(map),
            width: map.w,
        };
        level.screens.verts.iter_mut().for_each(|v| *v = v.with_light(1.0));
        (0..map.rooms.len() as u8).for_each(|room| {
            let whole = b.parts.range((room, 0)..=(room, u8::MAX)).fold(Mesh::default(), |mut acc, ((_, tex), part)| {
                let lit = part.verts.iter().map(|v| v.with_light(lamps.light(room, v.pos, Some(v.normal()))).with_tile(tiles[*tex as usize]));
                let base = acc.verts.len() as u16;
                acc.verts.extend(lit);
                acc.idx.extend(part.idx.iter().map(|i| i + base));
                acc
            });
            let start = level.chunks.len();
            chunked(&whole).iter().for_each(|c| {
                let chunk = level.push(c);
                level.chunks.push(chunk);
            });
            let outdoor = map.rooms[room as usize].ceil.is_none();
            level.rooms.push(RoomGeo { outdoor, chunks: start..level.chunks.len() });
        });
        let wood = tiles[texture::id("madeira_escura") as usize];
        b.shutters.iter().for_each(|(x, y, room, m)| {
            let lit = m.verts.iter().map(|v| v.with_light(lamps.light(*room, v.pos, Some(v.normal()))).with_tile(wood)).collect();
            let chunk = level.push(&Mesh { verts: lit, idx: m.idx.clone() });
            let run = level.runs[(y * map.w + x) as usize] as u8;
            level.shutters.push(Shutter { run, room: *room, chunk });
        });
        let cloth = tiles[texture::id("colcha") as usize];
        b.drapes.iter().for_each(|(room, xf, m)| {
            let lit = m.verts.iter().map(|v| v.with_light(lamps.light(*room, v.pos, Some(v.normal()))).with_tile(cloth)).collect();
            let chunk = level.push(&Mesh { verts: lit, idx: m.idx.clone() });
            level.drapes.push(Drape { room: *room, xf: *xf, chunk });
        });
        assert!(level.verts.len() <= u16::MAX as usize, "level has {} vertices", level.verts.len());
        level
    }

    fn push(&mut self, m: &Mesh) -> Chunk {
        let base = self.verts.len();
        let (lo, hi) = m.verts.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), v| {
            (std::array::from_fn(|k| lo[k].min(v.pos[k])), std::array::from_fn(|k| hi[k].max(v.pos[k])))
        });
        let chunk = Chunk { lo, hi, first: self.idx.len(), count: m.idx.len() };
        self.verts.extend_from_slice(&m.verts);
        self.idx.extend(m.idx.iter().map(|&i| (base + i as usize) as u16));
        chunk
    }

    pub fn window_run(&self, x: i32, y: i32) -> Option<u8> {
        [(0, 0), (0, -1), (0, 1), (-1, 0), (1, 0)]
            .iter()
            .map(|(dx, dy)| (x + dx, y + dy))
            .filter(|&(x, y)| x >= 0 && y >= 0 && x < self.width && (y * self.width + x) < self.runs.len() as i32)
            .find_map(|(x, y)| u8::try_from(self.runs[(y * self.width + x) as usize]).ok())
    }

    pub fn tris(&self) -> usize {
        self.idx.len() / 3
    }

    #[cfg(test)]
    pub fn room_tris(&self, room: usize) -> usize {
        self.chunks[self.rooms[room].chunks.clone()].iter().map(|c| c.count / 3).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    #[test]
    fn sobrado_builds_within_budget() {
        let m = map::sobrado();
        let l = Level::new(m, &Lamps::new(m), &texture::atlas().tiles);
        assert!(l.tris() > 1000 && l.tris() < 20000, "{} tris", l.tris());
        assert!(l.idx.iter().all(|&i| (i as usize) < l.verts.len()));
        assert_eq!(l.screens.tris(), 2 * m.props.iter().filter(|p| p.kind == "tv_tubo").count());
        (0..l.rooms.len()).for_each(|r| assert!(l.room_tris(r) < 2600, "room {r}: {}", l.room_tris(r)));
        assert!(l.verts.iter().all(|v| v.tile[2] > 0 && v.tile[3] > 0));
        l.chunks.iter().for_each(|c| {
            assert!(c.hi[0] - c.lo[0] < CHUNK + 3.0 && c.hi[2] - c.lo[2] < CHUNK + 3.0, "chunk {:?}..{:?}", c.lo, c.hi);
            assert!(l.idx[c.first..c.first + c.count].iter().all(|&i| (0..3).all(|k| (c.lo[k]..=c.hi[k]).contains(&l.verts[i as usize].pos[k]))));
        });
    }

    #[test]
    fn every_window_task_has_shutters() {
        let m = map::sobrado();
        let l = Level::new(m, &Lamps::new(m), &texture::atlas().tiles);
        m.tasks.iter().filter(|t| t.kind == map::TaskKind::Window).for_each(|t| {
            let (x, y) = (t.pos.x.to_f32().floor() as i32, t.pos.y.to_f32().floor() as i32);
            let run = l.window_run(x, y).unwrap_or_else(|| panic!("window task {x},{y} has no window"));
            assert!(l.shutters.iter().any(|s| s.run == run && s.chunk.count > 0));
        });
    }
}
