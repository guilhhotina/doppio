mod camera;
mod cloth;
mod cues;
mod doors;
mod eye;
mod font;
mod glow;
mod lamps;
mod level;
mod mannequin;
mod mat;
mod mesh;
mod mirror;
mod props;
mod shader;
mod tells;
mod texture;
mod ui2d;

use crate::fx::Ang;
use crate::gl::{self, Gl};
use crate::map::Map;
use crate::sim::{Body, Door, MAX_BODIES, World};
use camera::Camera;
use mirror::Mirror;
use std::f32::consts::{PI, TAU};
use std::time::Instant;
use tells::{Fixture, Site};
use eye::{Eye, ROOMS};
use lamps::Lamps;
use level::{Chunk, Level};
use mannequin::{Crowd, Rig, SEGS};
use mat::{M4, V3};
use mesh::{Mesh, Vert};
use shader::{MAX_LIGHTS, Program, Shaders, blit, glow as gu, world as wu};
use texture::Textures;

pub use cues::{Cues, Echo, MAX_TELLS, Tell};
pub use ui2d::{Ui2d, rgb565};

pub const CANVAS_W: u16 = 384;
pub const CANVAS_H: u16 = 216;

const WARM: V3 = [1.45, 1.12, 0.78];
const MOON: V3 = [0.55, 0.68, 1.0];
const AMBIENT: V3 = [0.045, 0.05, 0.075];
const DARK: V3 = [0.026, 0.028, 0.046];
const FOG: V3 = [0.012, 0.012, 0.02];
const FLICKER: [u8; 24] = [1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1];
const BODY_TEX: &str = "metal";
const LOD_DISTANCE: f32 = 4.0;
const NEAR_BODIES: usize = 3;
const BODY_HALF: V3 = [0.45, 0.9, 0.45];
const LOOK_EASE: f32 = 0.2;
const CLOSE_TICS: u8 = 10;
const CLOTH_MOON: f32 = 1.3;
const MIRROR_GLEAM: f32 = 0.25;

pub struct Scene<'a> {
    pub bodies: &'a [Body],
    pub doors: &'a [Door],
    pub power: bool,
    pub cues: &'a Cues,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct View {
    pub rect: [u16; 4],
    pub follow: u8,
    pub fade: u8,
    pub duplo: bool,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Stats {
    pub tris: [u32; 4],
    pub draws: u32,
}

struct Buffers {
    level_vbo: u32,
    level_ibo: u32,
    door_vbo: [u32; 4],
    door_ibo: u32,
    screen_vbo: u32,
    body_vbo: u32,
    body_ibo: [u32; 4],
    cone_vbo: u32,
    cone_ibo: u32,
    quad_vbo: u32,
    fx_vbo: [u32; 4],
    ui_vbo: u32,
    ui_ibo: u32,
}

struct Target {
    fbo: u32,
    color: u32,
    depth: u32,
    font: u32,
}

#[derive(Clone, Copy, Default)]
struct Light {
    pos: V3,
    dir: V3,
    power: f32,
    cutoff: f32,
}

#[derive(Clone, Copy)]
struct Shade([(V3, f32); 3]);

impl Shade {
    fn at(&self, n: V3) -> f32 {
        self.0.iter().map(|(d, w)| w * (mat::dot(n, *d).max(0.0) * 0.8 + 0.2)).sum::<f32>().min(1.0)
    }
}

#[derive(Clone, Copy)]
struct Posed {
    mats: [M4; SEGS],
    shade: Shade,
    centre: V3,
    rooms: u32,
    outdoor: bool,
}

impl Posed {
    fn seen_by(&self, e: &Eye) -> bool {
        e.sees(self.rooms, mat::sub(self.centre, BODY_HALF), mat::add(self.centre, BODY_HALF))
    }
}

struct Tv {
    light: Light,
    lo: V3,
    hi: V3,
    rooms: u32,
    on: bool,
}

pub struct Renderer {
    map: &'static Map<'static>,
    shaders: Shaders,
    textures: Textures,
    buf: Buffers,
    target: Target,
    level: Level,
    lamps: Lamps,
    crowd: Crowd,
    door: Mesh,
    cone: Mesh,
    scratch: Vec<Vert>,
    skinned: Vec<Vert>,
    body_idx: Vec<u16>,
    posed: [Option<Posed>; MAX_BODIES],
    skin: [Option<(u16, bool)>; MAX_BODIES],
    lanterns: [Option<Light>; MAX_BODIES],
    body_tile: [u8; 4],
    cloth_tile: [u8; 4],
    tvs: Vec<Tv>,
    sites: Vec<Site>,
    mirrors: Vec<Mirror>,
    fx: Vec<Vert>,
    fx_quads: [usize; 3],
    mirror_skin: Vec<Vert>,
    look: [f32; MAX_BODIES],
    close: [bool; 4],
    shut: u64,
    flapping: u64,
    start: Instant,
    time: f32,
    tick: u32,
    frame: u32,
    power_seen: bool,
    flicker: u8,
    stats: Stats,
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

fn tvs(map: &Map, screens: &Mesh) -> Vec<Tv> {
    screens
        .verts
        .chunks(4)
        .map(|q| {
            let c = mat::scale(q.iter().fold([0.0; 3], |a, v| mat::add(a, v.pos)), 0.25);
            let n = q[0].normal();
            let rooms = lamps::room_near(map, c[0].floor() as i32, c[2].floor() as i32).map_or(0, |r| 1 << r);
            let (lo, hi) = (mat::sub(c, [0.4; 3]), mat::add(c, [0.4; 3]));
            Tv { light: Light { pos: mat::add(c, mat::scale(n, 0.25)), dir: n, power: 0.0, cutoff: -2.0 }, lo, hi, rooms, on: true }
        })
        .collect()
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

fn tv_lit(k: usize, sites: &[Site], tells: &[Tell], power: bool) -> bool {
    power && sites.iter().zip(tells).find(|(s, _)| s.fixture == Fixture::Tv(Some(k as u8))).is_none_or(|(_, t)| *t == Tell::Pending)
}

fn turn(a: f32) -> f32 {
    (a + PI).rem_euclid(TAU) - PI
}

fn task_bits(sites: &[Site], tells: &[Tell], want: Tell, pick: impl Fn(&Fixture) -> Option<u8>) -> u64 {
    sites.iter().zip(tells).filter(|(_, t)| **t == want).filter_map(|(s, _)| pick(&s.fixture)).fold(0u64, |m, r| m | 1 << r)
}

impl Renderer {
    pub fn new(gl: &Gl, map: &'static Map<'static>) -> Renderer {
        assert!(map.rooms.len() <= ROOMS, "{} rooms", map.rooms.len());
        let shaders = Shaders::new(gl).unwrap_or_else(|e| panic!("doppio shaders: {e}"));
        let textures = Textures::new(gl);
        let lamps = Lamps::new(map);
        let level = Level::new(map, &lamps, &textures.tiles);
        let crowd = Crowd::new();
        let door = doors::mesh(textures.tile("madeira_porta"));
        let cone = glow::cone();
        let body_tile = textures.tile(BODY_TEX);
        let (hi_verts, hi_idx) = (crowd.get(false).mesh.verts.len(), crowd.get(false).mesh.idx.len());
        let buf = Buffers {
            level_vbo: static_buffer(gl, gl::ARRAY_BUFFER, &level.verts),
            level_ibo: static_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &level.idx),
            door_vbo: std::array::from_fn(|_| gl.gen_buffer()),
            door_ibo: static_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &repeated(&door, map.doors.len())),
            screen_vbo: static_buffer(gl, gl::ARRAY_BUFFER, &level.screens.verts),
            body_vbo: gl.gen_buffer(),
            body_ibo: std::array::from_fn(|_| gl.gen_buffer()),
            cone_vbo: gl.gen_buffer(),
            cone_ibo: static_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &repeated(&cone, MAX_BODIES)),
            quad_vbo: static_buffer(gl, gl::ARRAY_BUFFER, &[0.0f32, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            fx_vbo: std::array::from_fn(|_| gl.gen_buffer()),
            ui_vbo: gl.gen_buffer(),
            ui_ibo: static_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &ui_indices()),
        };
        gl.bind_buffer(gl::ARRAY_BUFFER, 0);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        let font = texture::upload_luminance(gl, font::ATLAS as i32, font::ATLAS as i32, &font::atlas());
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
        let scratch = Vec::with_capacity((map.doors.len() * door.verts.len()).max(MAX_BODIES * cone.verts.len()));
        assert!(level.drapes.len() <= 64, "{} drapes", level.drapes.len());
        let tvs = tvs(map, &level.screens);
        let sites = tells::sites(map, &level, &tvs.iter().map(|t| t.light.pos).collect::<Vec<V3>>());
        let cloth_tile = textures.tile("colcha");
        Renderer {
            map,
            shaders,
            textures,
            buf,
            target: Target { fbo, color, depth, font },
            tvs,
            sites,
            mirrors: mirror::mirrors(map),
            fx: Vec::with_capacity(ui2d::MAX_QUADS * 4),
            fx_quads: [0; 3],
            mirror_skin: vec![Vert::default(); crowd.get(true).mesh.verts.len()],
            look: [0.0; MAX_BODIES],
            close: [false; 4],
            cloth_tile,
            level,
            lamps,
            crowd,
            door,
            cone,
            scratch,
            skinned: vec![Vert::default(); hi_verts * MAX_BODIES],
            body_idx: Vec::with_capacity(hi_idx * MAX_BODIES),
            body_tile,
            posed: [None; MAX_BODIES],
            skin: [None; MAX_BODIES],
            lanterns: [None; MAX_BODIES],
            shut: 0,
            flapping: 0,
            start: Instant::now(),
            time: 0.0,
            tick: 0,
            frame: 0,
            power_seen: true,
            flicker: 0,
            stats: Stats::default(),
        }
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    pub fn due(&mut self, views: usize) -> bool {
        self.tick = self.tick.wrapping_add(1);
        paced(self.tick, views)
    }

    pub fn level_tris(&self) -> usize {
        self.level.tris()
    }

    pub fn body_tris(&self) -> [usize; 2] {
        self.crowd.lod.each_ref().map(|m| m.mesh.tris())
    }

    pub fn texture_bytes(&self) -> usize {
        self.textures.bytes + font::ATLAS * font::ATLAS + CANVAS_W as usize * CANVAS_H as usize * 4
    }

    pub fn delete(&self, gl: &Gl) {
        self.shaders.delete(gl);
        self.textures.delete(gl);
        let b = &self.buf;
        [b.level_vbo, b.level_ibo, b.door_ibo, b.screen_vbo, b.body_vbo, b.cone_vbo, b.cone_ibo, b.quad_vbo, b.ui_vbo, b.ui_ibo]
            .iter()
            .chain(&b.door_vbo)
            .chain(&b.fx_vbo)
            .chain(&b.body_ibo)
            .for_each(|&x| gl.delete_buffer(x));
        gl.delete_framebuffer(self.target.fbo);
        gl.delete_texture(self.target.color);
        gl.delete_texture(self.target.font);
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

    fn shade(&self, b: &Body) -> (Shade, bool) {
        let (x, z) = (b.pos.x.to_f32(), b.pos.y.to_f32());
        let room = lamps::room_near(self.map, x.floor() as i32, z.floor() as i32).unwrap_or(0);
        let c = [x, 1.0, z];
        let mut l = [([0.0f32; 3], 0.0f32); 3];
        let rb = self.lamps.rooms[room as usize];
        match rb.outdoor {
            true => l[0] = (mat::norm([0.42, 0.78, 0.46]), rb.level * 2.2),
            false => self.lamps.lamps(room).take(3).zip(l.iter_mut()).for_each(|(lamp, slot)| {
                let d = mat::sub(lamp.pos, c);
                *slot = (mat::norm(d), lamp.gain * 0.65 / (1.0 + lamp.falloff * 1.5 * mat::dot(d, d)));
            }),
        }
        (Shade(l), rb.outdoor)
    }

    fn rooms_at(&self, x: i32, z: i32) -> u32 {
        let own = lamps::room_near(self.map, x, z).map_or(0, |r| 1 << r);
        self.level.portals.iter().filter(|p| p.x as i32 == x && p.y as i32 == z).fold(own, |m, p| m | p.rooms)
    }

    fn pose(&self, b: &Body, look: f32) -> (Posed, Option<Light>) {
        let (x, z) = (b.pos.x.to_f32(), b.pos.y.to_f32());
        let Rig { mats, lantern, beam } = mannequin::rig(x, z, b.heading.to_rad(), b.anim, b.anim_tic, b.lantern, look);
        let (shade, outdoor) = self.shade(b);
        let rooms = self.rooms_at(x.floor() as i32, z.floor() as i32);
        let light = b.lantern.then_some(Light { pos: lantern, dir: beam, power: 2.4, cutoff: 0.74 });
        (Posed { mats, shade, centre: [x, 0.9, z], rooms, outdoor }, light)
    }

    fn pose_bodies(&mut self, bodies: &[Body], gaze: &[Option<Ang>; MAX_BODIES]) {
        (0..MAX_BODIES).for_each(|i| {
            let body = bodies.get(i).filter(|b| b.visible);
            let target = body.zip(gaze[i]).map_or(0.0, |(b, g)| turn(g.to_rad() - b.heading.to_rad()));
            self.look[i] = body.map_or(0.0, |_| turn(self.look[i] + turn(target - self.look[i]) * LOOK_EASE));
            let (posed, light) = body.map(|b| self.pose(b, self.look[i])).unzip();
            self.posed[i] = posed;
            self.lanterns[i] = light.flatten();
        });
    }

    fn skin_bodies(&mut self, gl: &Gl, eyes: &[Option<Eye>; 4]) {
        let mut seen = [false; MAX_BODIES];
        let mut near = [false; MAX_BODIES];
        eyes.iter().flatten().for_each(|e| {
            let mut close = [(f32::MAX, 0u8); MAX_BODIES];
            let mut n = 0;
            self.posed.iter().enumerate().for_each(|(i, p)| {
                let Some(p) = p.as_ref().filter(|p| p.seen_by(e)) else { return };
                seen[i] = true;
                let d = mat::len(mat::sub(p.centre, e.cam.eye));
                if d < e.detail {
                    close[n] = (d, i as u8);
                    n += 1;
                }
            });
            close[..n].sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
            close[..n.min(NEAR_BODIES)].iter().for_each(|&(_, i)| near[i as usize] = true);
        });
        let mut base = 0;
        (0..MAX_BODIES).for_each(|i| {
            self.skin[i] = None;
            let Some(p) = self.posed[i].as_ref().filter(|_| seen[i]) else { return };
            let m = self.crowd.get(!near[i]);
            let n = m.mesh.verts.len();
            let shade = p.shade;
            m.skin(&p.mats, |n| shade.at(n), self.body_tile, &mut self.skinned[base..base + n]);
            self.skin[i] = Some((base as u16, !near[i]));
            base += n;
        });
        if base > 0 {
            gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.body_vbo);
            gl.buffer_data(gl::ARRAY_BUFFER, &self.skinned[..base], gl::STREAM_DRAW);
        }
    }

    fn task_state(&mut self, tells: &[Tell], power: bool) {
        self.shut = task_bits(&self.sites, tells, Tell::Done, |f| match f {
            Fixture::Window { run, .. } => *run,
            _ => None,
        });
        self.flapping = task_bits(&self.sites, tells, Tell::Pending, |f| match f {
            Fixture::Window { drape, .. } => *drape,
            _ => None,
        });
        self.tvs.iter_mut().enumerate().for_each(|(k, tv)| tv.on = tv_lit(k, &self.sites, tells, power));
    }

    fn pick_lights(&self, focus: V3, frame: u32, power: f32) -> [Light; MAX_LIGHTS] {
        let mut out = [Light::default(); MAX_LIGHTS];
        let mut dist = [f32::MAX; MAX_LIGHTS];
        let tv = self.tvs.iter().enumerate().filter(|(_, t)| t.on).map(|(k, t)| Light {
            power: power * (0.22 + 0.2 * unit(frame / 2 + k as u32 * 977)),
            ..t.light
        });
        self.lanterns
            .iter()
            .flatten()
            .copied()
            .chain(tv.filter(|l| l.power > 0.0))
            .for_each(|l| {
                let d = mat::sub(l.pos, focus);
                let d2 = mat::dot(d, d) + if l.cutoff < -1.0 { 9.0 } else { 0.0 };
                if let Some(slot) = (0..MAX_LIGHTS).max_by(|&a, &b| dist[a].total_cmp(&dist[b])).filter(|&s| d2 < dist[s]) {
                    out[slot] = l;
                    dist[slot] = d2;
                }
            });
        out
    }

    pub fn frame(&mut self, gl: &Gl, fbo: u32, width: u32, height: u32, world: &World, views: &[View], ui: &Ui2d) {
        let cues = Cues::of(world);
        let scene = Scene { bodies: world.bodies(), doors: world.doors(), power: world.power_on(), cues: &cues };
        self.render(gl, [fbo, width, height], &scene, views, ui);
    }

    pub fn draw(&mut self, gl: &Gl, fbo: u32, width: u32, height: u32, scene: &Scene, views: &[View], ui: &Ui2d) {
        self.render(gl, [fbo, width, height], scene, views, ui);
    }

    fn render(&mut self, gl: &Gl, out: [u32; 3], scene: &Scene, views: &[View], ui: &Ui2d) {
        self.frame = self.frame.wrapping_add(1);
        self.time = self.start.elapsed().as_secs_f32();
        self.stats = Stats::default();
        let lamp = self.lamp_level(scene.power);
        self.task_state(&scene.cues.tells, scene.power);
        self.pose_bodies(scene.bodies, &scene.cues.gaze);
        self.close = std::array::from_fn(|i| views.get(i).is_some_and(|v| self.captor_face(v, scene).is_some()));
        let eyes: [Option<Eye>; 4] = std::array::from_fn(|i| views.get(i).and_then(|v| self.eye(v, scene)));
        let echoes: [Option<Echo>; 4] = std::array::from_fn(|i| views.get(i).and_then(|v| scene.cues.echo.get(v.follow as usize).copied().flatten()));
        self.skin_bodies(gl, &eyes);

        gl.bind_framebuffer(self.target.fbo);
        gl.viewport(0, 0, CANVAS_W as i32, CANVAS_H as i32);
        gl.disable(gl::SCISSOR_TEST);
        gl.disable(gl::BLEND);
        gl.color_mask(true, true, true, true);
        gl.depth_mask(true);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear_depth(1.0);
        gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl.enable(gl::DEPTH_TEST);
        gl.depth_func(gl::LEQUAL);
        gl.enable(gl::CULL_FACE);
        gl.cull_face(gl::BACK);
        gl.front_face(gl::CCW);
        gl.bind_texture(1, self.textures.dither);
        gl.bind_texture(2, self.textures.noise);
        [0, 1, 2, 3].iter().for_each(|&a| gl.enable_attrib(a));
        eyes.iter().enumerate().for_each(|(i, e)| e.iter().for_each(|e| self.draw_view(gl, i, e, scene, echoes[i], lamp)));

        gl.disable(gl::DEPTH_TEST);
        gl.disable(gl::CULL_FACE);
        [1, 2, 3].iter().for_each(|&a| gl.disable_attrib(a));
        gl.bind_framebuffer(out[0]);
        gl.viewport(0, 0, out[1] as i32, out[2] as i32);
        gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        self.upscale(gl, views);
        self.draw_ui(gl, ui);

        gl.disable(gl::BLEND);
        [0, 1, 2].iter().for_each(|&a| gl.disable_attrib(a));
        gl.bind_buffer(gl::ARRAY_BUFFER, 0);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        [2, 1, 0].iter().for_each(|&u| gl.bind_texture(u, 0));
        gl.use_program(0);
    }

    fn world_attribs(gl: &Gl) {
        let stride = size_of::<Vert>();
        gl.attrib_pointer(0, 3, gl::FLOAT, false, stride, 0);
        gl.attrib_pointer(1, 2, gl::SHORT, false, stride, 12);
        gl.attrib_pointer(2, 4, gl::BYTE, true, stride, 16);
        gl.attrib_pointer(3, 4, gl::UNSIGNED_BYTE, false, stride, 20);
    }

    fn camera(&self, body: Option<&Body>, aspect: f32) -> Camera {
        let (x, z, h) = body.map_or((1.5, 1.5, 0.0), |b| (b.pos.x.to_f32(), b.pos.y.to_f32(), b.heading.to_rad()));
        Camera::follow(self.map, x, z, aspect).unwrap_or_else(|| {
            let eye = [x - h.sin() * 2.5, 2.2, z - h.cos() * 2.5];
            let vp = mat::mul(&mat::perspective(1.0, aspect, camera::NEAR, camera::FAR), &mat::look_at(eye, [x, 1.0, z]));
            Camera { eye, view_proj: vp, frustum: mat::Frustum::new(&vp) }
        })
    }

    fn captor_face(&self, v: &View, scene: &Scene) -> Option<(V3, V3)> {
        let (body, tics) = scene.cues.caught.get(v.follow as usize).copied().flatten()?;
        let posed = self.posed.get(body as usize).copied().flatten().filter(|_| tics < CLOSE_TICS)?;
        Some(mannequin::face(&posed.mats))
    }

    fn eye(&self, v: &View, scene: &Scene) -> Option<Eye> {
        let rect = v.rect.map(|c| c as i32);
        if rect[2] <= 0 || rect[3] <= 0 {
            return None;
        }
        let aspect = rect[2] as f32 / rect[3] as f32;
        let follow = scene.bodies.get(v.follow as usize);
        let close = self.captor_face(v, scene);
        let cam = close.map_or_else(|| self.camera(follow, aspect), |(face, fwd)| Camera::close(face, fwd, aspect));
        let focus = close.map(|c| c.0).or_else(|| follow.map(|b| [b.pos.x.to_f32(), 1.0, b.pos.y.to_f32()])).unwrap_or(cam.eye);
        let seeds = [focus, cam.eye]
            .iter()
            .filter_map(|p| lamps::room_near(self.map, p[0].floor() as i32, p[2].floor() as i32))
            .fold(0u32, |m, r| m | 1 << r);
        let ajar = |x: u8, y: u8| scene.doors.iter().any(|d| d.x == x && d.y == y && d.open > 0);
        let open = self.level.portals.iter().filter(|p| ajar(p.x, p.y));
        let detail = LOD_DISTANCE * rect[3] as f32 / CANVAS_H as f32;
        Some(Eye::new(cam, focus, seeds, open, detail, rect))
    }

    fn draw_view(&mut self, gl: &Gl, slot: usize, e: &Eye, scene: &Scene, echo: Option<Echo>, lamp: f32) {
        let [x, y, w, h] = e.rect;
        gl.viewport(x, CANVAS_H as i32 - y - h, w, h);
        let cam = &e.cam;
        let snap = [w as f32 * 0.5, h as f32 * 0.5];
        let mut lights = self.pick_lights(e.focus, self.frame, lamp);
        if self.close[slot] {
            lights[0] = Light { pos: e.cam.eye, dir: mat::norm(mat::sub(e.focus, e.cam.eye)), power: 1.4, cutoff: 0.8 };
        }
        let dark = lamp < 0.5;
        let warm = mat::scale(WARM, lamp);
        let mut tris = 0u32;

        let p = self.shaders.world.loc;
        gl.use_program(self.shaders.world.id);
        self.set_scene(gl, &self.shaders.world, cam, &lights, dark, snap);
        gl.uniform_mat4(p[wu::MVP], &cam.view_proj);
        gl.bind_texture(0, self.textures.atlas);
        gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.level_vbo);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.level_ibo);
        Self::world_attribs(gl);
        let level = &self.level;
        let draws = &mut self.stats.draws;
        let mut draw = |c: &Chunk| {
            gl.draw_elements_u16(gl::TRIANGLES, c.count as i32, c.first * 2);
            *draws += 1;
            tris += c.count as u32 / 3;
        };
        (0..level.rooms.len()).filter(|r| e.reach >> r & 1 != 0).for_each(|r| {
            let c = if level.rooms[r].outdoor { MOON } else { warm };
            gl.uniform3f(p[wu::LAMP], c[0], c[1], c[2]);
            level.chunks[level.rooms[r].chunks.clone()].iter().filter(|c| e.sees(1 << r, c.lo, c.hi)).for_each(&mut draw);
        });
        gl.uniform3f(p[wu::LAMP], warm[0], warm[1], warm[2]);
        level
            .shutters
            .iter()
            .filter(|s| self.shut >> s.run & 1 != 0 && e.sees(1 << s.room, s.chunk.lo, s.chunk.hi))
            .for_each(|s| draw(&s.chunk));
        level
            .drapes
            .iter()
            .enumerate()
            .filter(|(d, s)| self.flapping >> d & 1 == 0 && e.sees(1 << s.room, s.chunk.lo, s.chunk.hi))
            .for_each(|(_, s)| draw(&s.chunk));

        tris += self.draw_doors(gl, slot, e, scene.doors);

        tris += self.draw_bodies(gl, slot, e, warm);

        tris += self.draw_fx(gl, slot, e, scene, echo, lamp);

        tris += self.draw_screens(gl, e, &lights, dark, lamp);
        tris += self.draw_glow(gl, slot, e);
        self.stats.tris[slot] = tris;
    }

    fn draw_doors(&mut self, gl: &Gl, slot: usize, e: &Eye, doors: &[Door]) -> u32 {
        self.scratch.clear();
        doors.iter().for_each(|d| {
            let (dx, dy) = (d.x as i32, d.y as i32);
            let rooms = self.level.portals.iter().find(|p| p.x == d.x && p.y == d.y).map_or(0, |p| p.rooms);
            let (lo, hi) = ([dx as f32 - 0.5, 0.0, dy as f32 - 0.5], [dx as f32 + 1.5, level::HEAD, dy as f32 + 1.5]);
            if !e.sees(rooms, lo, hi) {
                return;
            }
            let room = lamps::room_near(self.map, dx, dy).unwrap_or(0);
            let light = self.lamps.light(room, [dx as f32 + 0.5, 1.0, dy as f32 + 0.5], None);
            doors::emit(&self.door, &doors::model(self.map, d, e.cam.eye), light, &mut self.scratch);
        });
        if self.scratch.is_empty() {
            return 0;
        }
        gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.door_vbo[slot]);
        gl.buffer_data(gl::ARRAY_BUFFER, &self.scratch, gl::STREAM_DRAW);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.door_ibo);
        Self::world_attribs(gl);
        let count = self.scratch.len() / self.door.verts.len() * self.door.idx.len();
        gl.draw_elements_u16(gl::TRIANGLES, count as i32, 0);
        self.stats.draws += 1;
        count as u32 / 3
    }

    fn draw_bodies(&mut self, gl: &Gl, slot: usize, e: &Eye, warm: V3) -> u32 {
        self.body_idx.clear();
        let groups = [false, true].map(|outdoor| {
            let start = self.body_idx.len();
            (0..MAX_BODIES).for_each(|i| {
                let (Some(p), Some((base, far))) = (self.posed[i].as_ref(), self.skin[i]) else { return };
                if p.outdoor == outdoor && p.seen_by(e) {
                    self.body_idx.extend(self.crowd.get(far).mesh.idx.iter().map(|k| k + base));
                }
            });
            (start, self.body_idx.len() - start)
        });
        if self.body_idx.is_empty() {
            return 0;
        }
        let lamp = self.shaders.world.loc[wu::LAMP];
        gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.body_vbo);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.body_ibo[slot]);
        gl.buffer_data(gl::ELEMENT_ARRAY_BUFFER, &self.body_idx, gl::STREAM_DRAW);
        Self::world_attribs(gl);
        [warm, MOON].iter().zip(groups).filter(|(_, (_, n))| *n > 0).for_each(|(k, (start, n))| {
            gl.uniform3f(lamp, k[0], k[1], k[2]);
            gl.draw_elements_u16(gl::TRIANGLES, n as i32, start * 2);
            self.stats.draws += 1;
        });
        self.body_idx.len() as u32 / 3
    }

    fn draw_screens(&mut self, gl: &Gl, e: &Eye, lights: &[Light; MAX_LIGHTS], dark: bool, lamp: f32) -> u32 {
        let visible = |t: &Tv| t.on && e.sees(t.rooms, t.lo, t.hi);
        if !self.tvs.iter().any(visible) {
            return 0;
        }
        let s = &self.shaders.screen;
        let [_, _, w, h] = e.rect;
        gl.use_program(s.id);
        self.set_scene(gl, s, &e.cam, lights, dark, [w as f32 * 0.5, h as f32 * 0.5]);
        gl.uniform_mat4(s.loc[wu::MVP], &e.cam.view_proj);
        gl.uniform3f(s.loc[wu::LAMP], 0.0, 0.0, 0.0);
        let f = self.frame;
        gl.uniform4f(s.loc[wu::FX], unit(f * 3 + 1), unit(f * 7 + 2), lamp, 0.015 + 0.03 * lamp);
        gl.bind_texture(0, self.textures.noise);
        gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.screen_vbo);
        Self::world_attribs(gl);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.ui_ibo);
        let drawn: u32 = (0..self.tvs.len())
            .filter(|&k| visible(&self.tvs[k]))
            .map(|k| {
                gl.draw_elements_u16(gl::TRIANGLES, 6, k * 12);
                2
            })
            .sum();
        self.stats.draws += drawn / 2;
        gl.bind_texture(0, self.textures.atlas);
        drawn
    }

    fn emit_drapes(&mut self, e: &Eye) {
        let t = self.time;
        (0..self.level.drapes.len()).filter(|&d| self.flapping >> d & 1 != 0).for_each(|d| {
            let s = &self.level.drapes[d];
            if e.sees(1 << s.room, mat::sub(s.chunk.lo, [0.6; 3]), mat::add(s.chunk.hi, [0.6; 3])) {
                tells::drape(&s.xf, t, d as f32 * 1.37, self.cloth_tile, &mut self.fx);
            }
        });
    }

    fn emit_mirror(&mut self, e: &Eye, echo: Option<Echo>) {
        let Some(echo) = echo else { return };
        let (x, z) = (echo.pos.x.to_f32(), echo.pos.y.to_f32());
        let room = lamps::room_near(self.map, x.floor() as i32, z.floor() as i32);
        let eye = e.cam.eye;
        let body = Body { pos: echo.pos, heading: echo.heading, anim: echo.anim, anim_tic: echo.tic, lantern: false, visible: true };
        let (shade, _) = self.shade(&body);
        let rig = mannequin::rig(x, z, echo.heading.to_rad(), echo.anim, echo.tic, false, 0.0);
        for m in self.mirrors.iter().filter(|m| Some(m.room) == room && m.facing(eye) && e.sees(1 << m.room, m.lo, m.hi)) {
            let mats = rig.mats.map(|s| mat::mul(&m.reflect, &s));
            let mesh = self.crowd.get(true);
            let n = mesh.mesh.verts.len();
            mesh.skin(&mats, |nn| (shade.at(nn) * 1.4 + MIRROR_GLEAM).min(1.0), self.body_tile, &mut self.mirror_skin[..n]);
            m.emit(eye, &self.mirror_skin[..n], &mesh.mesh.idx, &mut self.fx);
        }
    }

    fn emit_glows(&mut self, e: &Eye, tells: &[Tell], power: bool, lamp: f32) {
        let (t, eye) = (self.time, e.cam.eye);
        self.sites.iter().zip(tells).filter(|(s, _)| e.sees(s.rooms, s.lo, s.hi)).for_each(|(s, &tell)| tells::emit(s, tell, power, t, eye, &mut self.fx));
        if lamp > 0.0 {
            self.level
                .glows
                .iter()
                .filter(|(p, room)| e.sees(1 << room, mat::sub(*p, [0.35; 3]), mat::add(*p, [0.35; 3])))
                .for_each(|&(p, _)| tells::abajur(&mut self.fx, p, eye, lamp));
        }
    }

    fn draw_fx(&mut self, gl: &Gl, slot: usize, e: &Eye, scene: &Scene, echo: Option<Echo>, lamp: f32) -> u32 {
        self.fx.clear();
        self.emit_drapes(e);
        let cloth = self.fx.len() / 4;
        self.emit_mirror(e, echo);
        let mirror = self.fx.len() / 4 - cloth;
        self.emit_glows(e, &scene.cues.tells, scene.power, lamp);
        self.fx.truncate(ui2d::MAX_QUADS * 4);
        let total = self.fx.len() / 4;
        let mirror = mirror.min(total - cloth.min(total));
        let cloth = cloth.min(total);
        self.fx_quads = [cloth, mirror, total - cloth - mirror];
        if total == 0 {
            return 0;
        }
        gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.fx_vbo[slot]);
        gl.buffer_data(gl::ARRAY_BUFFER, &self.fx, gl::STREAM_DRAW);
        if cloth + mirror == 0 {
            return 0;
        }
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.ui_ibo);
        Self::world_attribs(gl);
        let p = self.shaders.world.loc[wu::LAMP];
        if cloth > 0 {
            gl.disable(gl::CULL_FACE);
            gl.uniform3f(p, MOON[0] * CLOTH_MOON, MOON[1] * CLOTH_MOON, MOON[2] * CLOTH_MOON);
            gl.draw_elements_u16(gl::TRIANGLES, (cloth * 6) as i32, 0);
            gl.enable(gl::CULL_FACE);
            self.stats.draws += 1;
        }
        if mirror > 0 {
            gl.front_face(gl::CW);
            gl.uniform3f(p, WARM[0] * lamp, WARM[1] * lamp, WARM[2] * lamp);
            gl.draw_elements_u16(gl::TRIANGLES, (mirror * 6) as i32, cloth * 12);
            gl.front_face(gl::CCW);
            self.stats.draws += 1;
        }
        (cloth + mirror) as u32 * 2
    }

    fn draw_glow(&mut self, gl: &Gl, slot: usize, e: &Eye) -> u32 {
        self.scratch.clear();
        (0..MAX_BODIES).for_each(|i| {
            let Some(l) = self.lanterns[i] else { return };
            let rooms = self.posed[i].map_or(0, |p| p.rooms);
            let tip = mat::add(l.pos, mat::scale(l.dir, 0.9));
            if e.sees(rooms, mat::sub(tip, [1.0; 3]), mat::add(tip, [1.0; 3])) {
                glow::emit(&self.cone, &glow::model(l.pos, l.dir), &mut self.scratch);
            }
        });
        let [cloth, mirror, sparks] = self.fx_quads;
        if self.scratch.is_empty() && sparks == 0 {
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
        if !self.scratch.is_empty() {
            gl.uniform3f(g.loc[gu::COLOR], 0.26, 0.22, 0.15);
            gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.cone_vbo);
            gl.buffer_data(gl::ARRAY_BUFFER, &self.scratch, gl::STREAM_DRAW);
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.cone_ibo);
            Self::world_attribs(gl);
            let count = self.scratch.len() / self.cone.verts.len() * self.cone.idx.len();
            gl.draw_elements_u16(gl::TRIANGLES, count as i32, 0);
            self.stats.draws += 1;
            tris += count as u32 / 3;
        }
        if sparks > 0 {
            gl.uniform3f(g.loc[gu::COLOR], 1.0, 1.0, 1.0);
            gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.fx_vbo[slot]);
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.ui_ibo);
            Self::world_attribs(gl);
            gl.draw_elements_u16(gl::TRIANGLES, (sparks * 6) as i32, (cloth + mirror) * 12);
            self.stats.draws += 1;
            tris += sparks as u32 * 2;
        }
        gl.depth_mask(true);
        gl.enable(gl::CULL_FACE);
        gl.disable(gl::BLEND);
        tris
    }

    fn set_scene<const N: usize>(&self, gl: &Gl, p: &Program<N>, cam: &Camera, lights: &[Light; MAX_LIGHTS], dark: bool, snap: [f32; 2]) {
        let amb = if dark { DARK } else { AMBIENT };
        let (fog0, fog1) = if dark { (1.5, 9.0) } else { (4.0, 16.0) };
        gl.uniform3f(p.loc[wu::AMB], amb[0], amb[1], amb[2]);
        gl.uniform3f(p.loc[wu::EYE], cam.eye[0], cam.eye[1], cam.eye[2]);
        gl.uniform2f(p.loc[wu::FOG], fog0, 1.0 / (fog1 - fog0));
        gl.uniform2f(p.loc[wu::SNAP], snap[0], snap[1]);
        gl.uniform3f(p.loc[wu::FOGC], FOG[0], FOG[1], FOG[2]);
        lights.iter().enumerate().for_each(|(i, l)| {
            gl.uniform4f(p.loc[wu::LPOS + i], l.pos[0], l.pos[1], l.pos[2], l.power);
            gl.uniform4f(p.loc[wu::LDIR + i], l.dir[0], l.dir[1], l.dir[2], l.cutoff);
        });
    }

    fn upscale(&self, gl: &Gl, views: &[View]) {
        gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.quad_vbo);
        gl.attrib_pointer(0, 2, gl::FLOAT, false, 8, 0);
        gl.bind_texture(0, self.target.color);
        let canvas = [CANVAS_W as f32, CANVAS_H as f32];
        views.iter().take(4).enumerate().for_each(|(i, v)| {
            let r = [v.rect[0] as f32, v.rect[1] as f32, (v.rect[0] + v.rect[2]) as f32, (v.rect[1] + v.rect[3]) as f32];
            let close = self.close.get(i).copied().unwrap_or(false);
            match close || v.fade > 0 || v.duplo {
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
                    let k = if close { 1.0 } else { 1.0 - v.fade as f32 / 255.0 };
                    let tint = match (close, v.duplo) {
                        (true, _) => [1.05, 0.9, 0.86],
                        (false, true) => [k * 1.08, k * 0.86, k * 0.84],
                        (false, false) => [k; 3],
                    };
                    gl.uniform3f(p.loc[blit::TINT], tint[0], tint[1], tint[2]);
                    let grain = if close { 0.14 } else if v.duplo { 0.16 } else { 0.04 * (1.0 - k) };
                    let f = self.frame;
                    gl.uniform4f(p.loc[blit::GRAIN], unit(f * 5 + 3), unit(f * 11 + 4), grain, CANVAS_W as f32 / 64.0);
                }
            }
            gl.draw_arrays(gl::TRIANGLE_STRIP, 0, 4);
        });
    }

    fn draw_ui(&self, gl: &Gl, ui: &Ui2d) {
        let n = ui.quads();
        if n == 0 {
            return;
        }
        let p = &self.shaders.ui;
        gl.use_program(p.id);
        gl.uniform2f(p.loc[shader::ui::SCALE], 2.0 / CANVAS_W as f32, -2.0 / CANVAS_H as f32);
        gl.bind_texture(0, self.target.font);
        gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.ui_vbo);
        gl.buffer_data(gl::ARRAY_BUFFER, ui.verts(), gl::STREAM_DRAW);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.ui_ibo);
        let stride = size_of::<ui2d::UiVert>();
        gl.attrib_pointer(0, 2, gl::SHORT, false, stride, 0);
        gl.attrib_pointer(1, 2, gl::UNSIGNED_BYTE, false, stride, 4);
        gl.attrib_pointer(2, 4, gl::UNSIGNED_BYTE, true, stride, 8);
        gl.enable_attrib(1);
        gl.enable_attrib(2);
        gl.enable(gl::BLEND);
        gl.blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl.draw_elements_u16(gl::TRIANGLES, (n * 6) as i32, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    #[test]
    fn split_views_render_every_other_frame() {
        assert!((0..8).all(|t| paced(t, 1)));
        [2, 4].iter().for_each(|&n| {
            let due: Vec<bool> = (0..8).map(|t| paced(t, n)).collect();
            assert_eq!(due, [true, false, true, false, true, false, true, false]);
        });
    }

    #[test]
    fn hash_spreads() {
        let v: Vec<f32> = (0..1000).map(unit).collect();
        let mean = v.iter().sum::<f32>() / v.len() as f32;
        assert!((0.4..0.6).contains(&mean));
    }

    #[test]
    fn tv_follows_its_task() {
        let m = map::sobrado();
        let l = Level::new(m, &Lamps::new(m), &texture::atlas().tiles);
        let sets = tvs(m, &l.screens);
        let sites = tells::sites(m, &l, &sets.iter().map(|t| t.light.pos).collect::<Vec<V3>>());
        let task = m.tasks.iter().position(|t| t.kind == map::TaskKind::Tv).expect("tv task");
        let Fixture::Tv(Some(k)) = sites[task].fixture else { panic!("tv task without tv") };
        let with = |s: Tell| {
            let mut t = [Tell::Idle; MAX_TELLS];
            t[task] = s;
            t
        };
        let k = k as usize;
        assert!(!tv_lit(k, &sites, &with(Tell::Pending), false));
        assert!(!tv_lit(k, &sites, &with(Tell::Idle), true));
        assert!(tv_lit(k, &sites, &with(Tell::Pending), true));
        assert!(!tv_lit(k, &sites, &with(Tell::Done), true));
        assert!(sets.len() == 1 || (0..sets.len()).filter(|&j| j != k).all(|j| sites.iter().any(|s| s.fixture == Fixture::Tv(Some(j as u8))) || tv_lit(j, &sites, &with(Tell::Done), true)));
    }

    #[test]
    fn head_turn_eases_the_short_way() {
        assert!((turn(1.5 * PI) + 0.5 * PI).abs() < 1e-5);
        assert!((turn(-1.5 * PI) - 0.5 * PI).abs() < 1e-5);
        assert!((turn(0.3) - 0.3).abs() < 1e-5);
    }
}
