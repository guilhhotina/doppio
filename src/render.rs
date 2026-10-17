mod camera;
mod cast;
mod cloth;
mod doors;
mod eye;
mod font;
mod glow;
mod hide;
mod items;
mod lamps;
mod level;
mod mat;
mod mesh;
mod prefetch;
mod props;
mod shader;
mod sky;
mod sparks;
mod texture;
mod ui2d;
mod zones;

use crate::gl::{self, Gl};
use crate::map::{LegendKind, LightMode, LockKind, Map, Sky};
use crate::sim::{Anim, DoorState, ItemPlace, ItemState, Legend, LegendAnim, LegendMode, LockState, MAX_PLAYERS, Player, PlayerState, PropState, World};
use camera::Camera;
use cast::{Cast, Rig, Who};
use eye::Eye;
use lamps::{Lamps, Source};
use level::{Drive, Level};
use mat::{M4, V3};
use mesh::{Mesh, Vert};
use prefetch::Prefetch;
use shader::{MAX_LIGHTS, Shaders, blit, glow as gu, sky as su, world as wu};
use std::f32::consts::{PI, TAU};
use std::time::Instant;
use texture::Textures;
use zones::{Mask, Zones};

pub use font::{Face, width as text_width};
pub use ui2d::{MAX_QUADS as MAX_UI_QUADS, Ui2d, UiVert, WHITE, rgb, rgb565};

#[cfg(test)]
pub fn atlas_pixels() -> Vec<u16> {
    texture::atlas().px
}

pub const CANVAS_W: u16 = 384;
pub const CANVAS_H: u16 = 216;
pub const MAX_VIEWS: usize = 10;

const AMBIENT: V3 = [0.052, 0.058, 0.085];
const DARK: V3 = [0.058, 0.064, 0.095];
const BODY_FLOOR: V3 = [0.11, 0.115, 0.16];
const MOON: V3 = [0.55, 0.68, 1.0];
const CLOUD: V3 = [0.6, 0.64, 0.74];
const FLICKER: [u8; 24] = [1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1];
const BODIES: usize = 5;
const LOD_DISTANCE: f32 = 4.0;
const LOOK_EASE: f32 = 0.2;
const MATINTA_NEAR: f32 = 4.0;
const CAMERA_CLEAR: f32 = 0.9;
const MATINTA_BEAM: f32 = 7.0;
const LANTERN: f32 = 2.4;
const SPOT: f32 = 0.74;
const FIRE: V3 = [1.6, 0.75, 0.25];
const MAX_FX: usize = ui2d::MAX_QUADS;
const DYN_VERTS: usize = 16384;
const ATTRIBS: [u32; 5] = [0, 1, 2, 3, 4];
const GATE_EASE: f32 = 0.08;
const HIDE_EASE: f32 = 0.45;
const PEEK_FOV: f32 = 66.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct View {
    pub rect: [u16; 4],
    pub follow: u8,
    pub fade: u8,
    pub portrait: Option<u8>,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Stats {
    pub tris: [u32; MAX_VIEWS],
    pub draws: u32,
}

struct Scene<'a> {
    map: &'static Map<'static>,
    players: &'a [Player],
    legend: &'a Legend,
    doors: &'a [DoorState],
    props: &'a [PropState],
    items: &'a [ItemState],
    locks: &'a [LockState],
    power: bool,
    tic: u32,
    hiding: [Option<(u8, u8)>; MAX_PLAYERS],
}

#[derive(Clone, Copy, Default)]
struct Light {
    pos: V3,
    dir: V3,
    power: f32,
    cutoff: f32,
    color: V3,
    falloff: f32,
}

#[derive(Clone, Copy)]
struct Body {
    who: Who,
    rig: Rig,
    centre: V3,
    half: V3,
    rooms: Mask,
    area: u8,
    color: V3,
    key: V3,
    fade: f32,
    stunned: bool,
    down: bool,
}

struct DoorInfo {
    gate: bool,
    rooms: Mask,
    moon: f32,
    lit: [u8; 4],
}

struct Glow {
    pos: V3,
    color: [u8; 3],
    size: f32,
    rooms: Mask,
    lamp: Option<usize>,
}

struct Stage {
    map: &'static Map<'static>,
    zones: Zones,
    lamps: Lamps,
    level: Level,
    pages: Vec<(u32, u32)>,
    sky: Option<(u32, u32, i32)>,
    screen_vbo: u32,
    doors: Vec<DoorInfo>,
    glows: Vec<Glow>,
    gates: Vec<f32>,
    outdoor: Mask,
    fog: V3,
    fog_range: [f32; 2],
    far: f32,
    moon: V3,
}

struct Built {
    map: &'static Map<'static>,
    zones: Zones,
    lamps: Lamps,
    level: Level,
}

impl Built {
    fn new(map: &'static Map<'static>, tiles: &[[u8; 4]]) -> Built {
        let zones = Zones::new(map);
        let lamps = Lamps::new(map, &zones);
        let level = Level::new(map, &zones, &lamps, tiles);
        Built { map, zones, lamps, level }
    }
}

struct Buffers {
    body_vbo: u32,
    body_ibo: [u32; MAX_VIEWS],
    dyn_vbo: [u32; MAX_VIEWS],
    dyn_ibo: [u32; MAX_VIEWS],
    alpha_vbo: [u32; MAX_VIEWS],
    alpha_ibo: [u32; MAX_VIEWS],
    cone_vbo: [u32; MAX_VIEWS],
    cone_ibo: u32,
    fx_vbo: [u32; MAX_VIEWS],
    quad_vbo: u32,
    ui_vbo: u32,
    overlay_vbo: u32,
    ui_ibo: u32,
}

struct Target {
    fbo: u32,
    color: u32,
    depth: u32,
}

#[derive(Default)]
struct Stream {
    verts: Vec<Vert>,
    idx: Vec<u16>,
}

impl Stream {
    fn clear(&mut self) {
        self.verts.clear();
        self.idx.clear();
    }

    fn add(&mut self, mesh: &Mesh, xf: &M4, moon: f32, lit: [u8; 4]) {
        if self.verts.len() + mesh.verts.len() > DYN_VERTS {
            return;
        }
        let base = self.verts.len() as u16;
        doors::emit(mesh, xf, moon, lit, &mut self.verts);
        self.idx.extend(mesh.idx.iter().map(|i| i + base));
    }
}

pub struct Renderer {
    stage: Stage,
    shaders: Shaders,
    textures: Textures,
    buf: Buffers,
    target: Target,
    cast: Cast,
    items: Vec<Mesh>,
    door: Mesh,
    knob: Mesh,
    gate: Mesh,
    padlock: Mesh,
    cone: Mesh,
    bodies: [Option<Body>; BODIES],
    skin: [Option<(u16, bool)>; BODIES],
    skinned: Vec<Vert>,
    body_idx: Vec<u16>,
    look: [f32; BODIES],
    travel: [cast::Travel; BODIES],
    hides: [(hide::Track, hide::Phase); MAX_PLAYERS],
    lanterns: [Option<Light>; BODIES],
    opaque: Stream,
    alpha: Stream,
    fx: Vec<Vert>,
    cones: Vec<Vert>,
    overlay: Ui2d,
    item_rest: Vec<(i32, i32, f32)>,
    start: Instant,
    time: f32,
    tick: u32,
    frame: u32,
    power_seen: bool,
    flicker: u8,
    stats: Stats,
    prefetch: Prefetch<Built>,
}

fn static_buffer<T: Copy>(gl: &Gl, target: gl::GLenum, data: &[T]) -> u32 {
    let b = gl.gen_buffer();
    gl.bind_buffer(target, b);
    gl.buffer_data(target, data, gl::STATIC_DRAW);
    b
}

fn repeated(m: &Mesh, copies: usize) -> Vec<u16> {
    let n = m.verts.len() as u16;
    (0..copies as u16).flat_map(|c| m.idx.iter().map(move |i| i + c * n)).collect()
}

fn ui_indices() -> Vec<u16> {
    (0..ui2d::MAX_QUADS as u16).flat_map(|q| [0, 1, 2, 0, 2, 3].map(|i| q * 4 + i)).collect()
}

fn hash(x: u32) -> u32 {
    let x = (x ^ 61) ^ (x >> 16);
    let x = x.wrapping_mul(9);
    let x = x ^ (x >> 4);
    let x = x.wrapping_mul(0x27d4eb2d);
    x ^ (x >> 15)
}

fn unit(x: u32) -> f32 {
    (hash(x) & 0xffff) as f32 / 65536.0
}

fn paced(tick: u32, views: usize) -> bool {
    views < 2 || tick & 1 == 0
}

fn turn(a: f32) -> f32 {
    (a + PI).rem_euclid(TAU) - PI
}

fn flicker_on(frame: u32, seed: usize) -> bool {
    unit(frame / 4 + seed as u32 * 7919) > 0.28
}

fn hue(c: V3) -> V3 {
    mat::scale(c, 1.0 / c.iter().fold(1e-3f32, |a, &b| a.max(b)))
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn crowds(eye: V3, centre: V3, half: V3) -> bool {
    let gap = |k: usize| ((eye[k] - centre[k]).abs() - half[k]).max(0.0);
    gap(0).hypot(gap(2)) < CAMERA_CLEAR
}

impl Body {
    fn shows(&self, e: &Eye, close: bool) -> bool {
        (close || !crowds(e.cam.eye, self.centre, self.half)) && e.sees(self.rooms, mat::sub(self.centre, self.half), mat::add(self.centre, self.half))
    }
}

fn matinta_fade(legend: V3, players: impl Iterator<Item = (V3, Option<(V3, V3)>)>) -> f32 {
    let seen = players.map(|(p, beam)| {
        let near = smooth(MATINTA_NEAR + 1.0, MATINTA_NEAR, mat::len(mat::sub(p, legend)));
        let lit = beam.map_or(0.0, |(at, dir)| {
            let d = mat::sub(legend, at);
            let l = mat::len(d);
            (l < MATINTA_BEAM && mat::dot(d, dir) > l * SPOT) as u8 as f32
        });
        near.max(lit)
    });
    1.0 - seen.fold(0.0, f32::max)
}

fn who_of(portrait: u8) -> Who {
    match portrait {
        0..=3 => Who::Hero(portrait),
        4 => Who::Legend(LegendKind::CorpoSeco),
        5 => Who::Legend(LegendKind::Cabra),
        6 => Who::Legend(LegendKind::Mula),
        _ => Who::Legend(LegendKind::Matinta),
    }
}

fn v2(p: crate::fx::Vec2) -> (f32, f32) {
    (p.x.to_f32(), p.y.to_f32())
}

impl Stage {
    fn new(gl: &Gl, built: Built, textures: &Textures) -> Stage {
        let Built { map, zones, lamps, mut level } = built;
        let pages = level.pages.iter().map(|p| (static_buffer(gl, gl::ARRAY_BUFFER, &p.verts), static_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &p.idx))).collect();
        level.pages.iter_mut().for_each(|p| *p = level::Page::default());
        let sky = sky::mesh(map.sky, &textures.tiles).map(|m| (static_buffer(gl, gl::ARRAY_BUFFER, &m.verts), static_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &m.idx), m.idx.len() as i32));
        let screen_vbo = static_buffer(gl, gl::ARRAY_BUFFER, &level.screens.verts);
        gl.bind_buffer(gl::ARRAY_BUFFER, 0);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        let doors = map
            .doors
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let rooms = zones.portals.iter().find(|p| p.door == Some(i as u16)).map_or(0, |p| p.rooms);
                let room = zones.near(d.x, d.y).unwrap_or(0);
                let s = lamps.sample(room, [d.x as f32 + 0.5, 1.0, d.y as f32 + 0.5], None);
                let lit = lamps::Sample { electric: s.electric.map(|v| v.min(2.0)), flame: s.flame.min(2.0), moon: s.moon }.lit();
                DoorInfo { gate: doors::outdoor(map, &zones, d), rooms, moon: s.moon, lit }
            })
            .collect();
        let lamp_glows = lamps.lamps.iter().enumerate().filter(|(_, l)| l.mode != LightMode::Dead).map(|(i, l)| {
            let color = hue(l.rgb).map(|c| (c * 255.0) as u8);
            let size = match (l.source, l.open) {
                (Source::Flame, _) => 0.0,
                (Source::Electric, true) => 0.25 + l.radius * 0.06,
                (Source::Electric, false) => 0.24,
            };
            Glow { pos: l.pos, color, size, rooms: 1 << l.room, lamp: Some(i) }
        });
        let marks = level.marks.iter().filter(|m| m.kind == props::MarkKind::Abajur).map(|m| Glow { pos: m.pos, color: sparks::WARM, size: 0.34, rooms: 1 << m.room, lamp: None });
        let glows = lamp_glows.chain(marks).filter(|g| g.size > 0.0).collect();
        let outdoor = map.rooms.iter().enumerate().filter(|(_, r)| r.out).fold(0 as Mask, |m, (i, _)| m | 1 << i);
        let fog = map.fog.map(|c| c as f32 / 255.0);
        let (near, far) = (map.fog_near.to_f32(), map.fog_far.to_f32().max(map.fog_near.to_f32() + 1.0));
        let moon = mat::scale(if map.sky == Sky::Overcast { CLOUD } else { MOON }, lamps::moon_strength(map.sky));
        let gates = vec![0.0; level.movers.len()];
        Stage { map, zones, lamps, level, pages, sky, screen_vbo, doors, glows, gates, outdoor, fog, fog_range: [near, far], far: (far + 3.0).clamp(14.0, 40.0), moon }
    }

    fn delete(&self, gl: &Gl) {
        self.pages.iter().for_each(|&(v, i)| {
            gl.delete_buffer(v);
            gl.delete_buffer(i);
        });
        self.sky.iter().for_each(|&(v, i, _)| {
            gl.delete_buffer(v);
            gl.delete_buffer(i);
        });
        gl.delete_buffer(self.screen_vbo);
    }
}

impl Renderer {
    pub fn new(gl: &Gl, map: &'static Map<'static>) -> Renderer {
        let shaders = Shaders::new(gl).unwrap_or_else(|e| panic!("doppio shaders: {e}"));
        let textures = Textures::new(gl);
        let stage = Stage::new(gl, Built::new(map, &textures.tiles), &textures);
        let cast = Cast::new(|n| textures.tile(n));
        let items = items::meshes(&textures.tiles);
        let door = doors::mesh(textures.tile("madeira_porta"));
        let knob = doors::knob(textures.tile("dourado"), textures.tile("ferrugem"));
        let gate = doors::gate(textures.tile("portao_a"));
        let padlock = doors::padlock(textures.tile("bronze"), textures.tile("metal"));
        let cone = glow::cone();
        let gens = |gl: &Gl| -> [u32; MAX_VIEWS] { std::array::from_fn(|_| gl.gen_buffer()) };
        let buf = Buffers {
            body_vbo: gl.gen_buffer(),
            body_ibo: gens(gl),
            dyn_vbo: gens(gl),
            dyn_ibo: gens(gl),
            alpha_vbo: gens(gl),
            alpha_ibo: gens(gl),
            cone_vbo: gens(gl),
            cone_ibo: static_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &repeated(&cone, BODIES)),
            fx_vbo: gens(gl),
            quad_vbo: static_buffer(gl, gl::ARRAY_BUFFER, &[0.0f32, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ui_vbo: gl.gen_buffer(),
            overlay_vbo: gl.gen_buffer(),
            ui_ibo: static_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &ui_indices()),
        };
        gl.bind_buffer(gl::ARRAY_BUFFER, 0);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        let color = texture::target(gl, CANVAS_W as i32, CANVAS_H as i32);
        let depth = gl.gen_renderbuffer(gl::DEPTH_COMPONENT16, CANVAS_W as i32, CANVAS_H as i32);
        let fbo = gl.gen_framebuffer();
        gl.bind_framebuffer(fbo);
        gl.framebuffer_texture(gl::COLOR_ATTACHMENT0, color);
        gl.framebuffer_renderbuffer(gl::DEPTH_ATTACHMENT, depth);
        let status = gl.framebuffer_status();
        gl.bind_target();
        gl.bind_texture(0, 0);
        assert_eq!(status, gl::FRAMEBUFFER_COMPLETE, "low-res framebuffer incomplete");
        let max_verts = cast.max_verts();
        let max_idx = cast.max_idx();
        Renderer {
            stage,
            shaders,
            textures,
            buf,
            target: Target { fbo, color, depth },
            cast,
            items,
            door,
            knob,
            gate,
            padlock,
            cone,
            bodies: [None; BODIES],
            skin: [None; BODIES],
            skinned: vec![Vert::default(); max_verts * BODIES],
            body_idx: Vec::with_capacity(max_idx * BODIES),
            look: [0.0; BODIES],
            travel: [cast::Travel::default(); BODIES],
            hides: [(hide::Track::default(), hide::Phase::Out); MAX_PLAYERS],
            lanterns: [None; BODIES],
            opaque: Stream { verts: Vec::with_capacity(DYN_VERTS), idx: Vec::with_capacity(DYN_VERTS * 2) },
            alpha: Stream { verts: Vec::with_capacity(DYN_VERTS), idx: Vec::with_capacity(DYN_VERTS * 2) },
            fx: Vec::with_capacity(MAX_FX * 4),
            cones: Vec::with_capacity(BODIES * 32),
            overlay: Ui2d::new(),
            item_rest: Vec::with_capacity(crate::sim::MAX_ITEMS),
            start: Instant::now(),
            time: 0.0,
            tick: 0,
            frame: 0,
            power_seen: true,
            flicker: 0,
            stats: Stats::default(),
            prefetch: Prefetch::new(),
        }
    }

    pub fn prepare(&mut self, map: &'static Map<'static>) {
        if std::ptr::eq(self.stage.map, map) || !self.prefetch.idle_for(map) {
            return;
        }
        let tiles = self.textures.tiles.clone();
        self.prefetch.want(map, move || Built::new(map, &tiles));
    }

    pub fn load(&mut self, gl: &Gl, map: &'static Map<'static>) {
        if std::ptr::eq(self.stage.map, map) {
            return;
        }
        let built = self.prefetch.take(map).unwrap_or_else(|| Built::new(map, &self.textures.tiles));
        self.stage.delete(gl);
        self.stage = Stage::new(gl, built, &self.textures);
        self.item_rest.clear();
        self.travel.fill(cast::Travel::default());
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    pub fn due(&mut self, views: usize) -> bool {
        self.tick = self.tick.wrapping_add(1);
        paced(self.tick, views)
    }

    pub fn level_tris(&self) -> usize {
        self.stage.level.groups.iter().map(|g| g.chunk.count / 3).sum()
    }

    pub fn texture_bytes(&self) -> usize {
        self.textures.bytes + CANVAS_W as usize * CANVAS_H as usize * 4
    }

    pub fn delete(&self, gl: &Gl) {
        self.stage.delete(gl);
        self.shaders.delete(gl);
        self.textures.delete(gl);
        let b = &self.buf;
        [b.body_vbo, b.cone_ibo, b.quad_vbo, b.ui_vbo, b.overlay_vbo, b.ui_ibo]
            .iter()
            .chain(&b.body_ibo)
            .chain(&b.dyn_vbo)
            .chain(&b.dyn_ibo)
            .chain(&b.alpha_vbo)
            .chain(&b.alpha_ibo)
            .chain(&b.cone_vbo)
            .chain(&b.fx_vbo)
            .for_each(|&x| gl.delete_buffer(x));
        gl.delete_framebuffer(self.target.fbo);
        gl.delete_texture(self.target.color);
        gl.delete_renderbuffer(self.target.depth);
    }

    fn lamp_level(&mut self, power: bool) -> f32 {
        if power != self.power_seen {
            self.power_seen = power;
            self.flicker = FLICKER.len() as u8;
        }
        self.flicker = self.flicker.saturating_sub(1);
        match (self.flicker, power) {
            (0, true) => 1.0,
            (0, false) => 0.0,
            (f, _) => FLICKER[f as usize] as f32 * 0.8,
        }
    }

    fn rooms_at(&self, p: V3) -> Mask {
        let z = &self.stage.zones;
        let (x, y) = (p[0].floor() as i32, p[2].floor() as i32);
        let own = z.near(x, y).map_or(0, |r| 1 << r);
        z.portals.iter().filter(|q| q.door.is_some() && q.lo[0] as i32 == x && q.lo[2] as i32 == y).fold(own, |m, q| m | q.rooms)
    }

    fn body_light(&self, c: V3, lamp: f32) -> (V3, V3) {
        let st = &self.stage;
        let room = st.zones.at(c).unwrap_or(0);
        let colour = mat::add(st.lamps.sample(room, c, None).colour(lamp, st.moon), BODY_FLOOR);
        let key = st
            .lamps
            .lamps
            .iter()
            .filter(|l| l.room == room && l.mode == LightMode::Steady)
            .min_by(|a, b| mat::len(mat::sub(a.pos, c)).total_cmp(&mat::len(mat::sub(b.pos, c))))
            .map_or(lamps::MOON_DIR, |l| mat::norm(mat::sub(l.pos, c)));
        (colour, key)
    }

    fn pose(&mut self, scene: &Scene, lamp: f32) {
        (0..BODIES).for_each(|i| {
            self.bodies[i] = None;
            self.lanterns[i] = None;
        });
        scene.players.iter().enumerate().take(BODIES - 1).for_each(|(i, p)| {
            if matches!(p.state, PlayerState::Hidden { .. } | PlayerState::Escaped) {
                return;
            }
            let (x, z) = v2(p.pos);
            let phase = self.hides.get(i).map_or(hide::Phase::Out, |h| h.1);
            let spot = match phase {
                hide::Phase::Entering { spot, .. } | hide::Phase::Emerging { spot, .. } => scene.map.hides.get(spot as usize),
                _ => None,
            };
            let acting = spot.and_then(|h| hide::pose(h, phase, [x, z], p.heading.to_rad()));
            let (x, z, heading, anim, anim_tic) = acting.map_or((x, z, p.heading.to_rad(), p.anim, p.anim_tic), |a| (a.at[0], a.at[1], a.heading, a.anim, a.tic));
            self.look[i] = turn(self.look[i] * (1.0 - LOOK_EASE));
            let travel = self.travel[i].sample(scene.tic, p.area, [x, z, heading]);
            let rig = cast::hero(p.character, x, z, heading, anim, anim_tic, p.lantern, self.look[i], travel);
            let who = Who::Hero(p.character);
            let half = cast::extent(who);
            let centre = [x, half[1], z];
            let (color, key) = self.body_light(centre, lamp);
            self.lanterns[i] = p.lantern.then_some(Light { pos: rig.lantern, dir: rig.beam, power: LANTERN, cutoff: SPOT, color: [1.0, 0.88, 0.66], falloff: 0.16 });
            let down = scene.players.len() > 1 && matches!(p.state, PlayerState::Fainted { .. });
            self.bodies[i] = Some(Body { who, rig, centre, half, rooms: self.rooms_at(centre), area: p.area, color, key, fade: 0.0, stunned: false, down });
        });
        let l = scene.legend;
        let (x, z) = v2(l.pos);
        let travel = self.travel[BODIES - 1].sample(scene.tic, l.area, [x, z, l.heading.to_rad()]);
        let rig = cast::legend(l.kind, x, z, l.heading.to_rad(), l.anim, l.anim_tic, travel);
        let who = Who::Legend(l.kind);
        let half = cast::extent(who);
        let centre = [x, half[1], z];
        let fade = match l.kind {
            LegendKind::Matinta => {
                let seen = scene.players.iter().enumerate().filter(|(_, p)| p.area == l.area && !matches!(p.state, PlayerState::Escaped)).map(|(i, p)| {
                    let (px, pz) = v2(p.pos);
                    ([px, 1.0, pz], self.lanterns.get(i).copied().flatten().map(|b| (b.pos, b.dir)))
                });
                matinta_fade(centre, seen)
            }
            _ => 0.0,
        };
        let (color, key) = self.body_light(centre, lamp);
        let stunned = l.mode == LegendMode::Stunned || matches!(l.anim, LegendAnim::Stagger);
        self.bodies[BODIES - 1] = Some(Body { who, rig, centre, half, rooms: self.rooms_at(centre), area: l.area, color, key, fade, stunned, down: false });
    }

    fn skin_bodies(&mut self, gl: &Gl, eyes: &[Option<(Eye, u8, bool)>]) {
        let mut base = 0;
        (0..BODIES).for_each(|i| {
            self.skin[i] = None;
            let Some(b) = self.bodies[i] else { return };
            if b.fade >= 1.0 {
                return;
            }
            let seen = eyes.iter().flatten().filter(|(e, area, _)| *area == b.area && e.sees(b.rooms, mat::sub(b.centre, b.half), mat::add(b.centre, b.half)));
            let Some(near) = seen.map(|(e, _, _)| mat::len(mat::sub(b.centre, e.cam.eye)) < e.detail).reduce(|a, c| a || c) else { return };
            let model = self.cast.model(b.who, !near);
            let n = model.mesh.verts.len();
            let key = b.key;
            model.skin(&b.rig.mats, |n| mat::dot(n, key).max(0.0) * 0.7 + 0.3, b.fade, &mut self.skinned[base..base + n]);
            self.skin[i] = Some((base as u16, !near));
            base += n;
        });
        if base > 0 {
            gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.body_vbo);
            gl.buffer_data(gl::ARRAY_BUFFER, &self.skinned[..base], gl::STREAM_DRAW);
        }
    }

    fn scene_lights(&self, scene: &Scene, focus: V3, area: u8, lamp: f32) -> [Light; MAX_LIGHTS] {
        let st = &self.stage;
        let mut out = [Light::default(); MAX_LIGHTS];
        let mut dist = [f32::MAX; MAX_LIGHTS];
        let frame = self.frame;
        let flick = st.lamps.lamps.iter().enumerate().filter(|(_, l)| l.mode == LightMode::Flicker && st.zones.rooms[l.room as usize].area == area).filter_map(|(i, l)| {
            let on = flicker_on(frame, i) && (l.source == Source::Flame || lamp > 0.0);
            let k = if l.source == Source::Flame { 0.75 + 0.25 * unit(frame / 2 + i as u32) } else { lamp };
            on.then_some(Light { pos: l.pos, dir: [0.0, -1.0, 0.0], power: k * 1.2, cutoff: -2.0, color: hue(l.rgb), falloff: 1.6 / (l.radius * l.radius).max(1.0) })
        });
        let tvs = st.level.screens.verts.chunks(4).enumerate().filter(|_| scene.power && lamp > 0.0).map(|(k, q)| {
            let c = mat::scale(q.iter().fold([0.0; 3], |a, v| mat::add(a, v.pos)), 0.25);
            let n = q[0].normal();
            Light { pos: mat::add(c, mat::scale(n, 0.25)), dir: n, power: lamp * (0.22 + 0.2 * unit(frame / 2 + k as u32 * 977)), cutoff: -2.0, color: [0.78, 0.86, 1.0], falloff: 0.16 }
        });
        let fire = self.bodies[BODIES - 1].filter(|b| b.area == area && b.fade < 1.0).and_then(|b| b.rig.fire).map(|(p, d, s)| Light {
            pos: mat::add(p, mat::scale(d, s)),
            dir: d,
            power: 1.4 + 0.4 * unit(frame),
            cutoff: -2.0,
            color: FIRE,
            falloff: 0.25,
        });
        let lanterns = self.bodies.iter().zip(&self.lanterns).filter(|(b, _)| b.is_some_and(|b| b.area == area)).filter_map(|(_, l)| *l);
        lanterns.chain(fire).chain(flick).chain(tvs).for_each(|l| {
            let d = mat::sub(l.pos, focus);
            let d2 = mat::dot(d, d) + if l.cutoff < -1.0 { 4.0 } else { 0.0 };
            if let Some(slot) = (0..MAX_LIGHTS).max_by(|&a, &b| dist[a].total_cmp(&dist[b])).filter(|&s| d2 < dist[s]) {
                out[slot] = l;
                dist[slot] = d2;
            }
        });
        out
    }

    pub fn frame(&mut self, gl: &Gl, fbo: u32, width: u32, height: u32, world: &World, views: &[View], ui: &Ui2d) {
        self.load(gl, world.map());
        let scene = Scene {
            map: world.map(),
            players: world.players(),
            legend: world.legend(),
            doors: world.doors(),
            props: world.props(),
            items: world.items(),
            locks: world.locks(),
            power: world.power(),
            tic: world.tic(),
            hiding: std::array::from_fn(|i| world.hiding(i)),
        };
        self.render(gl, [fbo, width, height], &scene, views, ui);
    }

    fn camera(&self, scene: &Scene, p: &Player, aspect: f32) -> (Camera, bool) {
        let st = &self.stage;
        let (x, z) = v2(p.pos);
        let grabbed = p.state == PlayerState::Grabbed;
        let close = self.bodies[BODIES - 1].filter(|b| grabbed && b.area == p.area).map(|b| Camera::close(b.rig.head.0, b.rig.head.1, aspect, st.far));
        let hidden = match p.state {
            PlayerState::Hidden { spot } => scene.map.hides.get(spot as usize),
            _ => None,
        };
        let peek = hidden.map(|h| {
            let (eye, target) = hide::peek(h, [x, z], (self.time * 1.3).sin());
            Camera::look(eye, target, PEEK_FOV, aspect, st.far)
        });
        let cam = close.or(peek).or_else(|| Camera::follow(scene.map, x, z, aspect, st.far)).unwrap_or_else(|| {
            let h = p.heading.to_rad();
            Camera::look([x - h.sin() * 2.5, 2.2, z - h.cos() * 2.5], [x, 1.0, z], 57.0, aspect, st.far)
        });
        (cam, close.is_some())
    }

    fn eye(&self, scene: &Scene, v: &View) -> Option<(Eye, u8, bool)> {
        let rect = v.rect.map(|c| c as i32);
        if rect[2] <= 0 || rect[3] <= 0 || v.portrait.is_some() {
            return None;
        }
        let p = scene.players.get(v.follow as usize)?;
        let aspect = rect[2] as f32 / rect[3] as f32;
        let (cam, close) = self.camera(scene, p, aspect);
        let (x, z) = v2(p.pos);
        let focus = [x, 1.0, z];
        let z_ = &self.stage.zones;
        let allowed = z_.area_mask(p.area);
        let seeds = [focus, cam.eye].iter().filter_map(|q| z_.at(*q)).fold(0 as Mask, |m, r| m | 1 << r);
        let doors = scene.doors;
        let open = |q: &zones::Portal| q.door.is_none_or(|d| doors.get(d as usize).is_some_and(|s| s.open > 0));
        let detail = LOD_DISTANCE * rect[3] as f32 / CANVAS_H as f32;
        let mut e = Eye::new(cam, focus, seeds, z_, open, allowed, detail, rect);
        e.far = self.stage.fog_range[1] + 0.5;
        Some((e, p.area, close))
    }

    fn render(&mut self, gl: &Gl, out: [u32; 3], scene: &Scene, views: &[View], ui: &Ui2d) {
        self.frame = self.frame.wrapping_add(1);
        self.time = self.start.elapsed().as_secs_f32();
        self.stats = Stats::default();
        let lamp = self.lamp_level(scene.power);
        self.track_hides(scene);
        self.pose(scene, lamp);
        self.ease_gates(scene);
        let views = &views[..views.len().min(MAX_VIEWS)];
        let eyes: [Option<(Eye, u8, bool)>; MAX_VIEWS] = std::array::from_fn(|i| views.get(i).and_then(|v| self.eye(scene, v)));
        self.skin_bodies(gl, &eyes);

        gl.bind_framebuffer(self.target.fbo);
        gl.viewport(0, 0, CANVAS_W as i32, CANVAS_H as i32);
        gl.disable(gl::SCISSOR_TEST);
        gl.disable(gl::BLEND);
        gl.color_mask(true, true, true, true);
        gl.depth_mask(true);
        let f = self.stage.fog;
        gl.clear_color(f[0], f[1], f[2], 1.0);
        gl.clear_depth(1.0);
        gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl.enable(gl::DEPTH_TEST);
        gl.depth_func(gl::LEQUAL);
        gl.enable(gl::CULL_FACE);
        gl.cull_face(gl::BACK);
        gl.front_face(gl::CCW);
        gl.bind_texture(1, self.textures.dither);
        gl.bind_texture(2, self.textures.noise);
        ATTRIBS.iter().for_each(|&a| gl.enable_attrib(a));
        self.overlay.clear();
        views.iter().enumerate().for_each(|(i, v)| match &eyes[i] {
            Some((e, area, close)) => self.draw_view(gl, i, e, *area, *close, scene, v, lamp),
            None => {
                if let Some(c) = v.portrait {
                    self.draw_portrait(gl, i, v, c);
                }
            }
        });

        gl.disable(gl::DEPTH_TEST);
        gl.disable(gl::CULL_FACE);
        ATTRIBS[1..].iter().for_each(|&a| gl.disable_attrib(a));
        if self.overlay.quads() > 0 {
            gl.viewport(0, 0, CANVAS_W as i32, CANVAS_H as i32);
            self.draw_quads(gl, self.buf.overlay_vbo, &self.overlay);
            [1, 2].iter().for_each(|&a| gl.disable_attrib(a));
        }
        gl.bind_framebuffer(out[0]);
        gl.viewport(0, 0, out[1] as i32, out[2] as i32);
        gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        let closes: [bool; MAX_VIEWS] = std::array::from_fn(|i| eyes[i].as_ref().is_some_and(|e| e.2));
        self.upscale(gl, views, &closes);
        if ui.quads() > 0 {
            self.draw_quads(gl, self.buf.ui_vbo, ui);
        }
        gl.disable(gl::BLEND);
        [0, 1, 2].iter().for_each(|&a| gl.disable_attrib(a));
        gl.bind_buffer(gl::ARRAY_BUFFER, 0);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        [2, 1, 0].iter().for_each(|&u| gl.bind_texture(u, 0));
        gl.use_program(0);
    }

    fn track_hides(&mut self, scene: &Scene) {
        self.hides.iter_mut().zip(scene.players).zip(scene.hiding).for_each(|((h, p), hiding)| *h = hide::step(h.0, p.state, hiding, scene.tic));
    }

    fn ease_gates(&mut self, scene: &Scene) {
        let hides = self.hides;
        let st = &mut self.stage;
        st.level.movers.iter().zip(st.gates.iter_mut()).for_each(|(m, g)| {
            let target = match m.drive {
                Drive::Prop(i) => match scene.props.get(i as usize) {
                    Some(p) if p.open > 0 => p.open as f32 / 65535.0,
                    _ => scene.map.props.get(i as usize).and_then(|p| p.lock).filter(|&l| scene.map.props[i as usize].item.is_none() && scene.locks.get(l as usize).is_some_and(|s| s.open)).map_or(0.0, |_| 1.0),
                },
                Drive::Lock(l) => scene.locks.get(l as usize).map_or(0.0, |s| s.open as u8 as f32),
                Drive::Hide(spot) => hides.iter().filter_map(|h| hide::door(h.1)).filter(|d| d.0 == spot).map(|d| d.1).fold(0.0, f32::max),
            };
            let ease = match m.drive {
                Drive::Prop(_) => 1.0,
                Drive::Hide(_) => HIDE_EASE,
                Drive::Lock(_) => GATE_EASE,
            };
            *g += (target - *g) * ease;
        });
    }

    fn world_attribs(gl: &Gl) {
        let stride = size_of::<Vert>();
        gl.attrib_pointer(0, 3, gl::FLOAT, false, stride, 0);
        gl.attrib_pointer(1, 2, gl::SHORT, false, stride, 12);
        gl.attrib_pointer(2, 4, gl::BYTE, true, stride, 16);
        gl.attrib_pointer(3, 4, gl::UNSIGNED_BYTE, false, stride, 20);
        gl.attrib_pointer(4, 4, gl::UNSIGNED_BYTE, true, stride, 24);
    }

    fn set_scene(&self, gl: &Gl, p: &shader::World, e: &Eye, lights: &[Light; MAX_LIGHTS], lamp: f32, moon: V3) {
        let st = &self.stage;
        let dark = lamp < 0.5 && st.outdoor & e.reach == 0;
        let amb = if dark { DARK } else { AMBIENT };
        let [n, f] = st.fog_range;
        let (n, f) = if dark { (n * 0.6, f * 0.75) } else { (n, f) };
        let [_, _, w, h] = e.rect;
        gl.uniform_mat4(p.loc[wu::MVP], &e.cam.view_proj);
        gl.uniform3f(p.loc[wu::AMB], amb[0], amb[1], amb[2]);
        gl.uniform3f(p.loc[wu::EYE], e.cam.eye[0], e.cam.eye[1], e.cam.eye[2]);
        gl.uniform2f(p.loc[wu::FOG], n, 1.0 / (f - n));
        gl.uniform2f(p.loc[wu::SNAP], w as f32 * 0.5, h as f32 * 0.5);
        gl.uniform3f(p.loc[wu::FOGC], st.fog[0], st.fog[1], st.fog[2]);
        gl.uniform3f(p.loc[wu::LAMP], lamp, lamp, lamp);
        gl.uniform3f(p.loc[wu::MOON], moon[0], moon[1], moon[2]);
        let k = 0.85 + 0.15 * unit(self.frame / 3);
        gl.uniform3f(p.loc[wu::FLAME], lamps::FLAME[0] * k, lamps::FLAME[1] * k, lamps::FLAME[2] * k);
        lights.iter().enumerate().for_each(|(i, l)| {
            gl.uniform4f(p.loc[wu::LPOS + i], l.pos[0], l.pos[1], l.pos[2], l.power);
            gl.uniform4f(p.loc[wu::LDIR + i], l.dir[0], l.dir[1], l.dir[2], l.cutoff);
            gl.uniform4f(p.loc[wu::LCOL + i], l.color[0], l.color[1], l.color[2], l.falloff);
        });
    }

    fn draw_groups(&mut self, gl: &Gl, e: &Eye, pass: usize) -> u32 {
        let st = &self.stage;
        let mut bound = u16::MAX;
        let mut tris = 0;
        st.level.groups[st.level.passes[pass].clone()].iter().filter(|g| e.sees(g.rooms, g.chunk.lo, g.chunk.hi)).for_each(|g| {
            if g.chunk.page != bound {
                bound = g.chunk.page;
                let (v, i) = st.pages[bound as usize];
                gl.bind_buffer(gl::ARRAY_BUFFER, v);
                gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, i);
                Self::world_attribs(gl);
            }
            gl.draw_elements_u16(gl::TRIANGLES, g.chunk.count as i32, g.chunk.first * 2);
            self.stats.draws += 1;
            tris += g.chunk.count as u32 / 3;
        });
        tris
    }

    fn draw_sky(&mut self, gl: &Gl, e: &Eye) -> u32 {
        let st = &self.stage;
        let Some((vbo, ibo, count)) = st.sky.filter(|_| st.outdoor & e.reach != 0) else { return 0 };
        let p = &self.shaders.sky;
        let [_, _, w, h] = e.rect;
        gl.use_program(p.id);
        gl.uniform_mat4(p.loc[su::MVP], &e.cam.view_proj);
        gl.uniform3f(p.loc[su::EYE], e.cam.eye[0], e.cam.eye[1], e.cam.eye[2]);
        gl.uniform2f(p.loc[su::SNAP], w as f32 * 0.5, h as f32 * 0.5);
        gl.uniform3f(p.loc[su::FOGC], st.fog[0], st.fog[1], st.fog[2]);
        let k = lamps::moon_strength(st.map.sky);
        gl.uniform3f(p.loc[su::TINT], k, k, k);
        gl.bind_texture(0, self.textures.atlas);
        gl.bind_buffer(gl::ARRAY_BUFFER, vbo);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        Self::world_attribs(gl);
        gl.depth_mask(false);
        gl.draw_elements_u16(gl::TRIANGLES, count, 0);
        gl.depth_mask(true);
        self.stats.draws += 1;
        count as u32 / 3
    }

    fn fill_dynamic(&mut self, e: &Eye, area: u8, scene: &Scene, viewer: &Player) {
        self.opaque.clear();
        self.alpha.clear();
        let st = &self.stage;
        let eye = e.cam.eye;
        scene.map.doors.iter().zip(scene.doors).zip(&st.doors).filter(|((d, _), info)| {
            let (lo, hi) = ([d.x as f32 - 0.5, 0.0, d.y as f32 - 0.5], [d.x as f32 + 1.5, level::HEAD, d.y as f32 + 1.5]);
            e.sees(info.rooms, lo, hi)
        }).for_each(|((d, s), info)| {
            let xf = doors::model(scene.map, d, s.open, eye);
            match info.gate {
                true => self.alpha.add(&self.gate, &xf, info.moon, info.lit),
                false => {
                    self.opaque.add(&self.door, &xf, info.moon, info.lit);
                    self.opaque.add(&self.knob, &xf, info.moon, info.lit);
                    if self.fx.len() < MAX_FX * 4 - 64 {
                        let kp = mat::point(&xf, [0.8, 0.96, 0.0]);
                        sparks::glint(&mut self.fx, kp, eye, self.time, (d.x * 13 + d.y * 7) as f32 * 0.41, 0.52);
                    }
                }
            }
            if s.locked {
                self.opaque.add(&self.padlock, &xf, info.moon, info.lit);
            }
        });
        st.level.movers.iter().zip(&st.gates).filter(|(m, _)| e.sees(m.rooms, m.lo, m.hi)).for_each(|(m, &f)| {
            let xf = mat::mul(&m.xf, &m.hinge.at(f));
            let lit = m.opaque.verts.first().or(m.alpha.verts.first()).map_or(([0; 4], 0.0), |v| (v.lit, v.nrm[3] as f32 / 127.0));
            self.opaque.add(&m.opaque, &xf, lit.1, lit.0);
            self.alpha.add(&m.alpha, &xf, lit.1, lit.0);
        });
        let held = scene.items.iter().enumerate().filter_map(|(i, it)| match it.place {
            ItemPlace::Held { player } => Some((i, it.kind, player)),
            _ => None,
        });
        held.for_each(|(_, kind, player)| {
            let Some(b) = self.bodies.get(player as usize).copied().flatten().filter(|b| b.area == area && self.skin[player as usize].is_some()) else { return };
            let p = &scene.players[player as usize];
            let slot = p.hands.iter().position(|h| h.is_some_and(|h| scene.items.get(h as usize).is_some_and(|x| x.kind == kind))).unwrap_or(0);
            let s = st.lamps.sample(st.zones.at(b.centre).unwrap_or(0), b.centre, None);
            let lit = lamps::Sample { electric: s.electric.map(|v| v.min(2.0)), flame: s.flame.min(2.0), moon: s.moon }.lit();
            let mesh = &self.items[kind as usize];
            let dst = if items::alpha(kind) { &mut self.alpha } else { &mut self.opaque };
            dst.add(mesh, &cast::held(kind, &b.rig.hands[slot.min(1)]), s.moon, lit);
        });
        let range = items::glint_range(viewer.character);
        let (vx, vz) = v2(viewer.pos);
        let placed = scene.map.locks.iter().zip(scene.locks).filter(|(_, s)| s.open).filter_map(|(l, _)| match l.kind {
            LockKind::Place { item, x, y } => scene.map.items.get(item as usize).map(|it| (it.kind, [x as f32 + 0.5, y as f32 + 0.5], area)),
            _ => None,
        });
        let floor = scene.items.iter().filter_map(|it| match it.place {
            ItemPlace::Floor { pos, area } => Some((it.kind, [pos.x.to_f32(), pos.y.to_f32()], area)),
            _ => None,
        });
        floor.map(|f| (f, true)).chain(placed.map(|p| (p, false))).enumerate().filter(|(_, ((_, _, a), _))| *a == area).for_each(|(i, ((kind, at, _), shine))| {
            let rest = st.level.rest(at[0], at[1], |p| scene.props.get(p as usize).is_some_and(|s| s.open > 0));
            let pos = [at[0], rest, at[1]];
            let room = st.zones.at(pos).unwrap_or(0);
            if !e.sees(1 << room, mat::sub(pos, [0.4; 3]), mat::add(pos, [0.4; 3])) {
                return;
            }
            let yaw = unit(i as u32 * 31 + kind as u32) * TAU;
            let xf = mat::mul(&mat::translate(pos), &mat::rot_y(yaw));
            let s = st.lamps.sample(room, mat::add(pos, [0.0, 0.3, 0.0]), None);
            let lit = lamps::Sample { electric: s.electric.map(|v| v.min(2.0)), flame: s.flame.min(2.0), moon: s.moon }.lit();
            let dst = if items::alpha(kind) { &mut self.alpha } else { &mut self.opaque };
            dst.add(&self.items[kind as usize], &xf, (s.moon + 0.25).min(1.0), lit);
            let d = ((at[0] - vx).powi(2) + (at[1] - vz).powi(2)).sqrt();
            if shine && d < range && self.fx.len() < MAX_FX * 4 - 64 {
                sparks::glint(&mut self.fx, mat::add(pos, [0.0, 0.08, 0.0]), eye, self.time, i as f32 * 0.37, 1.0 - d / range * 0.5);
            }
        });
        st.level.notes.iter().enumerate().filter(|(_, n)| {
            let d = ((n.pos[0] - vx).powi(2) + (n.pos[2] - vz).powi(2)).sqrt();
            d < range && e.sees(n.rooms, mat::sub(n.pos, [0.3; 3]), mat::add(n.pos, [0.3; 3]))
        }).for_each(|(i, n)| {
            if self.fx.len() < MAX_FX * 4 - 64 {
                sparks::glint(&mut self.fx, n.pos, eye, self.time + 0.7, i as f32 * 0.61, 0.6);
            }
        });
    }

    fn draw_stream(&mut self, gl: &Gl, alpha: bool, slot: usize) -> u32 {
        let (s, vbo, ibo) = match alpha {
            true => (&self.alpha, self.buf.alpha_vbo[slot], self.buf.alpha_ibo[slot]),
            false => (&self.opaque, self.buf.dyn_vbo[slot], self.buf.dyn_ibo[slot]),
        };
        if s.idx.is_empty() {
            return 0;
        }
        gl.bind_buffer(gl::ARRAY_BUFFER, vbo);
        gl.buffer_data(gl::ARRAY_BUFFER, &s.verts, gl::STREAM_DRAW);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        gl.buffer_data(gl::ELEMENT_ARRAY_BUFFER, &s.idx, gl::STREAM_DRAW);
        Self::world_attribs(gl);
        gl.draw_elements_u16(gl::TRIANGLES, s.idx.len() as i32, 0);
        self.stats.draws += 1;
        s.idx.len() as u32 / 3
    }

    fn draw_bodies(&mut self, gl: &Gl, slot: usize, e: &Eye, area: u8, close: bool) -> u32 {
        self.body_idx.clear();
        let mut runs = [(0usize, 0usize, [0.0f32; 3]); BODIES];
        (0..BODIES).for_each(|i| {
            let (Some(b), Some((base, far))) = (self.bodies[i], self.skin[i]) else { return };
            if b.area != area || !b.shows(e, close) {
                return;
            }
            let start = self.body_idx.len();
            self.body_idx.extend(self.cast.model(b.who, far).mesh.idx.iter().map(|k| k + base));
            runs[i] = (start, self.body_idx.len() - start, b.color);
        });
        if self.body_idx.is_empty() {
            return 0;
        }
        let p = &self.shaders.world;
        gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.body_vbo);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.body_ibo[slot]);
        gl.buffer_data(gl::ELEMENT_ARRAY_BUFFER, &self.body_idx, gl::STREAM_DRAW);
        Self::world_attribs(gl);
        gl.disable(gl::CULL_FACE);
        runs.iter().filter(|r| r.1 > 0).for_each(|&(start, n, c)| {
            gl.uniform3f(p.loc[wu::MOON], c[0] + 0.04, c[1] + 0.04, c[2] + 0.05);
            gl.draw_elements_u16(gl::TRIANGLES, n as i32, start * 2);
            self.stats.draws += 1;
        });
        gl.enable(gl::CULL_FACE);
        self.body_idx.len() as u32 / 3
    }

    fn draw_screens(&mut self, gl: &Gl, e: &Eye, lights: &[Light; MAX_LIGHTS], lamp: f32, power: bool) -> u32 {
        let st = &self.stage;
        if !power || lamp <= 0.0 || st.level.screen_rooms.is_empty() {
            return 0;
        }
        let visible = |k: usize| {
            let c = st.level.screens.verts[k * 4].pos;
            e.sees(st.level.screen_rooms[k], mat::sub(c, [0.5; 3]), mat::add(c, [0.5; 3]))
        };
        let count = st.level.screen_rooms.len();
        if !(0..count).any(visible) {
            return 0;
        }
        let s = &self.shaders.screen;
        gl.use_program(s.id);
        self.set_scene(gl, s, e, lights, lamp, [0.0; 3]);
        gl.uniform3f(s.loc[wu::LAMP], 0.0, 0.0, 0.0);
        let f = self.frame;
        gl.uniform4f(s.loc[wu::FX], unit(f * 3 + 1), unit(f * 7 + 2), lamp, 0.015 + 0.03 * lamp);
        gl.bind_texture(0, self.textures.noise);
        gl.bind_buffer(gl::ARRAY_BUFFER, st.screen_vbo);
        Self::world_attribs(gl);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.ui_ibo);
        let drawn = (0..count).filter(|&k| visible(k)).inspect(|&k| gl.draw_elements_u16(gl::TRIANGLES, 6, k * 12)).count() as u32;
        self.stats.draws += drawn;
        gl.bind_texture(0, self.textures.atlas);
        drawn * 2
    }

    fn emit_fx(&mut self, e: &Eye, area: u8, lamp: f32, close: bool) {
        let st = &self.stage;
        let (t, eye) = (self.time, e.cam.eye);
        let frame = self.frame;
        st.level.marks.iter().filter(|m| m.kind == props::MarkKind::Flame && st.zones.rooms[m.room as usize].area == area).enumerate().for_each(|(i, m)| {
            if self.fx.len() < MAX_FX * 4 - 64 && e.sees(1 << m.room, mat::sub(m.pos, [0.2; 3]), mat::add(m.pos, [0.2; 3])) {
                sparks::flame(&mut self.fx, m.pos, eye, t, i as f32 * 1.7, 1.0);
            }
        });
        st.glows.iter().filter(|g| e.sees(g.rooms, mat::sub(g.pos, [g.size; 3]), mat::add(g.pos, [g.size; 3]))).for_each(|g| {
            let on = g.lamp.map_or(lamp, |i| {
                let l = &st.lamps.lamps[i];
                let k = if l.source == Source::Flame { 1.0 } else { lamp };
                if l.mode == LightMode::Flicker && !flicker_on(frame, i) { 0.0 } else { k }
            });
            if on > 0.0 && self.fx.len() < MAX_FX * 4 - 64 {
                sparks::halo(&mut self.fx, g.pos, eye, g.size, on, g.color);
            }
        });
        self.bodies.iter().flatten().filter(|b| b.area == area && b.fade < 0.9 && b.shows(e, close)).for_each(|b| {
            if let Some((p, d, s)) = b.rig.fire {
                sparks::fire(&mut self.fx, p, d, s, eye, t);
            }
            if let Some(eyes) = b.rig.eyes {
                let c = b.rig.eye_color.map(|v| (v * 255.0).clamp(0.0, 255.0) as u8);
                eyes.iter().for_each(|&p| sparks::spark(&mut self.fx, p, eye, 0.035, 1.6 * (1.0 - b.fade), c));
            }
            if b.stunned {
                sparks::stars(&mut self.fx, b.rig.head.0, eye, t);
            }
            if b.down {
                let beat = 0.6 + 0.4 * (t * 6.0).sin().abs();
                let mark = mat::add(b.rig.head.0, [0.0, 0.7, 0.0]);
                sparks::halo(&mut self.fx, mark, eye, 0.28 * beat, beat, sparks::GOLD);
                sparks::spark(&mut self.fx, mark, eye, 0.07, 1.5, sparks::STAR);
            }
        });
        self.fx.truncate(MAX_FX * 4);
    }

    fn draw_glow(&mut self, gl: &Gl, slot: usize, e: &Eye, area: u8) -> u32 {
        self.cones.clear();
        (0..BODIES).for_each(|i| {
            let (Some(l), Some(b)) = (self.lanterns[i], self.bodies[i]) else { return };
            let tip = mat::add(l.pos, mat::scale(l.dir, 0.9));
            if b.area == area && e.sees(b.rooms, mat::sub(tip, [1.0; 3]), mat::add(tip, [1.0; 3])) {
                glow::emit(&self.cone, &glow::model(l.pos, l.dir), &mut self.cones);
            }
        });
        let sparks = self.fx.len() / 4;
        if self.cones.is_empty() && sparks == 0 {
            return 0;
        }
        let g = &self.shaders.glow;
        let [_, _, w, h] = e.rect;
        gl.use_program(g.id);
        gl.uniform2f(g.loc[gu::SNAP], w as f32 * 0.5, h as f32 * 0.5);
        gl.uniform3f(g.loc[gu::EYE], e.cam.eye[0], e.cam.eye[1], e.cam.eye[2]);
        gl.uniform_mat4(g.loc[gu::MVP], &e.cam.view_proj);
        gl.enable(gl::BLEND);
        gl.blend_func(gl::ONE, gl::ONE);
        gl.depth_mask(false);
        gl.disable(gl::CULL_FACE);
        let mut tris = 0;
        if !self.cones.is_empty() {
            gl.uniform3f(g.loc[gu::COLOR], 0.26, 0.22, 0.15);
            gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.cone_vbo[slot]);
            gl.buffer_data(gl::ARRAY_BUFFER, &self.cones, gl::STREAM_DRAW);
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.cone_ibo);
            Self::world_attribs(gl);
            let count = self.cones.len() / self.cone.verts.len() * self.cone.idx.len();
            gl.draw_elements_u16(gl::TRIANGLES, count as i32, 0);
            self.stats.draws += 1;
            tris += count as u32 / 3;
        }
        if sparks > 0 {
            gl.uniform3f(g.loc[gu::COLOR], 1.0, 1.0, 1.0);
            gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.fx_vbo[slot]);
            gl.buffer_data(gl::ARRAY_BUFFER, &self.fx, gl::STREAM_DRAW);
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.ui_ibo);
            Self::world_attribs(gl);
            gl.draw_elements_u16(gl::TRIANGLES, (sparks * 6) as i32, 0);
            self.stats.draws += 1;
            tris += sparks as u32 * 2;
        }
        gl.depth_mask(true);
        gl.enable(gl::CULL_FACE);
        gl.disable(gl::BLEND);
        tris
    }

    fn draw_view(&mut self, gl: &Gl, slot: usize, e: &Eye, area: u8, close: bool, scene: &Scene, view: &View, lamp: f32) {
        let [x, y, w, h] = e.rect;
        gl.viewport(x, CANVAS_H as i32 - y - h, w, h);
        let viewer = scene.players[view.follow as usize];
        let mut lights = self.scene_lights(scene, e.focus, area, lamp);
        if let Some(face) = self.bodies[BODIES - 1].filter(|_| close).map(|b| b.rig.head.0) {
            lights[0] = Light { pos: e.cam.eye, dir: mat::norm(mat::sub(face, e.cam.eye)), power: 1.6, cutoff: 0.8, color: [1.0, 0.85, 0.7], falloff: 0.16 };
        }
        let moon = self.stage.moon;
        self.fx.clear();
        let mut tris = self.draw_sky(gl, e);

        let p = &self.shaders.world;
        let moon_loc = p.loc[wu::MOON];
        gl.use_program(p.id);
        self.set_scene(gl, p, e, &lights, lamp, moon);
        gl.bind_texture(0, self.textures.atlas);
        tris += self.draw_groups(gl, e, 0);
        self.fill_dynamic(e, area, scene, &viewer);
        tris += self.draw_stream(gl, false, slot);
        tris += self.draw_bodies(gl, slot, e, area, close);
        gl.uniform3f(moon_loc, moon[0], moon[1], moon[2]);

        let a = &self.shaders.alpha;
        gl.use_program(a.id);
        self.set_scene(gl, a, e, &lights, lamp, moon);
        gl.disable(gl::CULL_FACE);
        tris += self.draw_groups(gl, e, 1);
        tris += self.draw_stream(gl, true, slot);
        gl.enable(gl::CULL_FACE);

        if !self.stage.level.passes[2].is_empty() {
            let wp = &self.shaders.water;
            gl.use_program(wp.id);
            self.set_scene(gl, wp, e, &lights, lamp, moon);
            let t = self.time;
            gl.uniform4f(wp.loc[wu::FX], t * 0.031, t * 0.017, t * 1.3, 0.6 + 0.4 * lamps::moon_strength(self.stage.map.sky));
            tris += self.draw_groups(gl, e, 2);
        }

        tris += self.draw_screens(gl, e, &lights, lamp, scene.power);
        self.emit_fx(e, area, lamp, close);
        tris += self.draw_glow(gl, slot, e, area);
        if let PlayerState::Hidden { spot } = viewer.state {
            if let Some(hd) = scene.map.hides.get(spot as usize) {
                let phase = self.hides.get(view.follow as usize).map_or(hide::Phase::Out, |h| h.1);
                hide::overlay(hd.kind, phase, e.rect, self.time, &mut self.overlay);
            }
        }
        if slot < MAX_VIEWS {
            self.stats.tris[slot] = tris;
        }
    }

    fn draw_portrait(&mut self, gl: &Gl, slot: usize, v: &View, c: u8) {
        let rect = v.rect.map(|c| c as i32);
        if rect[2] <= 0 || rect[3] <= 0 {
            return;
        }
        let [x, y, w, h] = rect;
        let gy = CANVAS_H as i32 - y - h;
        gl.viewport(x, gy, w, h);
        gl.enable(gl::SCISSOR_TEST);
        gl.scissor(x, gy, w, h);
        gl.clear_color(0.02, 0.018, 0.03, 1.0);
        gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl.disable(gl::SCISSOR_TEST);
        let who = who_of(c);
        let tic = (self.time * 30.0) as u16;
        let spin = (self.time * 0.4).sin() * 0.5;
        let rig = match who {
            Who::Hero(ch) => cast::hero(ch, 0.0, 0.0, spin, Anim::Idle, tic, false, 0.0, [0.0; 2]),
            Who::Legend(k) => cast::legend(k, 0.0, 0.0, spin, LegendAnim::Idle, tic, [0.0; 2]),
        };
        let (eye, target, fov) = cast::portrait_cam(who);
        let cam = Camera::look(eye, target, fov, w as f32 / h as f32, 20.0);
        let model = self.cast.model(who, false);
        let n = model.mesh.verts.len();
        let key = mat::norm([-0.5, 0.7, 0.6]);
        model.skin(&rig.mats, |n| mat::dot(n, key).max(0.0) * 0.75 + 0.25, 0.0, &mut self.skinned[..n]);
        let p = &self.shaders.world;
        gl.use_program(p.id);
        let e = Eye { cam, focus: target, reach: 0, clip: [eye::FULL; zones::ROOMS], detail: 0.0, rect, far: f32::MAX };
        let lights = [Light { pos: [-1.2, 2.4, 1.6], dir: [0.0, -1.0, 0.0], power: 0.9, cutoff: -2.0, color: [1.0, 0.82, 0.6], falloff: 0.2 }, Light::default(), Light::default(), Light::default()];
        self.set_scene(gl, p, &e, &lights, 0.0, [0.25, 0.27, 0.36]);
        gl.uniform2f(p.loc[wu::FOG], 50.0, 0.01);
        gl.bind_texture(0, self.textures.atlas);
        gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.body_vbo);
        gl.buffer_data(gl::ARRAY_BUFFER, &self.skinned[..n], gl::STREAM_DRAW);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.body_ibo[slot]);
        gl.buffer_data(gl::ELEMENT_ARRAY_BUFFER, &model.mesh.idx, gl::STREAM_DRAW);
        Self::world_attribs(gl);
        gl.disable(gl::CULL_FACE);
        gl.draw_elements_u16(gl::TRIANGLES, model.mesh.idx.len() as i32, 0);
        gl.enable(gl::CULL_FACE);
        self.stats.draws += 1;
        if slot < MAX_VIEWS {
            self.stats.tris[slot] = model.mesh.tris() as u32;
        }
        let f = self.stage.fog;
        gl.clear_color(f[0], f[1], f[2], 1.0);
        self.skin.iter_mut().for_each(|s| *s = None);
    }

    fn upscale(&self, gl: &Gl, views: &[View], closes: &[bool]) {
        gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.quad_vbo);
        gl.attrib_pointer(0, 2, gl::FLOAT, false, 8, 0);
        gl.bind_texture(0, self.target.color);
        let canvas = [CANVAS_W as f32, CANVAS_H as f32];
        views.iter().enumerate().for_each(|(i, v)| {
            let r = [v.rect[0] as f32, v.rect[1] as f32, (v.rect[0] + v.rect[2]) as f32, (v.rect[1] + v.rect[3]) as f32];
            let close = closes.get(i).copied().unwrap_or(false);
            match close || v.fade > 0 {
                false => {
                    let p = &self.shaders.blit;
                    gl.use_program(p.id);
                    gl.uniform4f(p.loc[blit::RECT], r[0], r[1], r[2], r[3]);
                    gl.uniform2f(p.loc[blit::CANVAS], canvas[0], canvas[1]);
                }
                true => {
                    let p = &self.shaders.blit_fx;
                    gl.use_program(p.id);
                    gl.uniform4f(p.loc[blit::RECT], r[0], r[1], r[2], r[3]);
                    gl.uniform2f(p.loc[blit::CANVAS], canvas[0], canvas[1]);
                    let k = 1.0 - v.fade as f32 / 255.0;
                    let tint = if close { [1.05 * k, 0.9 * k, 0.86 * k] } else { [k; 3] };
                    gl.uniform3f(p.loc[blit::TINT], tint[0], tint[1], tint[2]);
                    let grain = if close { 0.14 } else { 0.04 * (1.0 - k) };
                    let f = self.frame;
                    gl.uniform4f(p.loc[blit::GRAIN], unit(f * 5 + 3), unit(f * 11 + 4), grain, CANVAS_W as f32 / 64.0);
                }
            }
            gl.draw_arrays(gl::TRIANGLE_STRIP, 0, 4);
        });
    }

    fn draw_quads(&self, gl: &Gl, vbo: u32, ui: &Ui2d) {
        let p = &self.shaders.ui;
        gl.use_program(p.id);
        gl.uniform2f(p.loc[shader::ui::SCALE], 2.0 / CANVAS_W as f32, -2.0 / CANVAS_H as f32);
        gl.bind_texture(0, self.textures.atlas);
        gl.bind_buffer(gl::ARRAY_BUFFER, vbo);
        gl.buffer_data(gl::ARRAY_BUFFER, ui.verts(), gl::STREAM_DRAW);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.ui_ibo);
        let stride = size_of::<ui2d::UiVert>();
        gl.attrib_pointer(0, 2, gl::SHORT, false, stride, 0);
        gl.attrib_pointer(1, 2, gl::UNSIGNED_SHORT, false, stride, 4);
        gl.attrib_pointer(2, 4, gl::UNSIGNED_BYTE, true, stride, 8);
        gl.enable_attrib(1);
        gl.enable_attrib(2);
        gl.enable(gl::BLEND);
        gl.blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl.draw_elements_u16(gl::TRIANGLES, (ui.quads() * 6) as i32, 0);
        gl.disable(gl::BLEND);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_views_render_every_other_frame() {
        assert!((0..8).all(|t| paced(t, 1)));
        [2, 4].iter().for_each(|&n| {
            let due: Vec<bool> = (0..8).map(|t| paced(t, n)).collect();
            assert_eq!(due, [true, false, true, false, true, false, true, false]);
        });
    }

    #[test]
    fn bodies_hugging_the_camera_are_dropped() {
        let half = [0.3, 0.9, 0.3];
        assert!(crowds([0.0, 1.5, 0.0], [0.6, 0.9, 0.5], half));
        assert!(crowds([0.0, 3.0, 0.0], [0.0, 0.9, 1.0], half));
        assert!(!crowds([0.0, 1.5, 0.0], [0.0, 0.9, 2.0], half));
        assert!(!crowds([0.0, 1.5, 0.0], [1.5, 0.9, 1.5], half));
    }

    #[test]
    fn hash_spreads() {
        let v: Vec<f32> = (0..1000).map(unit).collect();
        let mean = v.iter().sum::<f32>() / v.len() as f32;
        assert!((0.4..0.6).contains(&mean));
    }

    #[test]
    fn matinta_shows_only_near_or_in_the_beam() {
        let legend = [5.0, 1.0, 5.0];
        assert_eq!(matinta_fade(legend, [([15.0, 1.0, 5.0], None)].into_iter()), 1.0);
        assert_eq!(matinta_fade(legend, [([7.0, 1.0, 5.0], None)].into_iter()), 0.0);
        assert_eq!(matinta_fade(legend, [([11.0, 1.0, 5.0], Some(([11.0, 1.2, 5.0], [-1.0, 0.0, 0.0])))].into_iter()), 0.0);
        assert_eq!(matinta_fade(legend, [([11.0, 1.0, 5.0], Some(([11.0, 1.2, 5.0], [1.0, 0.0, 0.0])))].into_iter()), 1.0);
        assert!(matinta_fade(legend, [([9.5, 1.0, 5.0], None)].into_iter()) > 0.0);
    }

    #[test]
    fn portraits_cover_heroes_and_legends() {
        assert_eq!(who_of(2), Who::Hero(2));
        assert_eq!(who_of(4), Who::Legend(LegendKind::CorpoSeco));
        assert_eq!(who_of(7), Who::Legend(LegendKind::Matinta));
    }

    #[test]
    fn flicker_spends_some_time_dark() {
        let on = (0..400).filter(|&f| flicker_on(f, 3)).count();
        assert!((150..390).contains(&on), "{on}");
    }
}
