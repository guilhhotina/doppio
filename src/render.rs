mod camera;
mod font;
mod lamps;
mod level;
mod mannequin;
mod mat;
mod mesh;
mod props;
mod shader;
mod texture;
mod ui2d;

use crate::gl::{self, Gl};
use crate::map::{Map, Tile};
use crate::sim::{Body, Door, MAX_BODIES, World};
use camera::Camera;
use lamps::Lamps;
use level::Level;
use mannequin::{Crowd, Rig};
use mat::V3;
use mesh::{Faces, Mesh, Pen, Vert};
use shader::{MAX_LIGHTS, Program, Shaders, blit, glow, world as wu};
use texture::Textures;

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
const LOD_DISTANCE: f32 = 5.5;

pub struct Scene<'a> {
    pub bodies: &'a [Body],
    pub doors: &'a [Door],
    pub power: bool,
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
    door_vbo: u32,
    door_ibo: u32,
    door_count: i32,
    screen_vbo: u32,
    screen_count: i32,
    body_vbo: u32,
    body_ibo: u32,
    cone_vbo: u32,
    cone_ibo: u32,
    cone_count: i32,
    quad_vbo: u32,
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

pub struct Renderer {
    map: &'static Map<'static>,
    shaders: Shaders,
    textures: Textures,
    buf: Buffers,
    target: Target,
    level: Level,
    lamps: Lamps,
    crowd: Crowd,
    skinned: Vec<Vert>,
    rigs: [Option<(V3, bool)>; MAX_BODIES],
    lanterns: [Option<Light>; MAX_BODIES],
    tvs: Vec<Light>,
    body_tex: u8,
    door_tex: u8,
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

fn door_mesh() -> Mesh {
    let mut m = Mesh::default();
    let mut p = Pen { mesh: &mut m, xf: mat::IDENTITY, density: 1.0 };
    p.cuboid([0.03, 0.0, -0.025], [0.97, level::HEAD - 0.02, 0.025], Faces(0x0f));
    p.panel([0.03, 0.0, 0.0], [0.97, level::HEAD - 0.02, 0.025]);
    p.xf = mat::rot_y(std::f32::consts::PI);
    p.panel([-0.97, 0.0, 0.0], [-0.03, level::HEAD - 0.02, 0.025]);
    m.verts.iter_mut().for_each(|v| *v = v.with_light(1.0));
    m
}

fn cone_mesh() -> Mesh {
    let (sides, len, r) = (10u16, 2.6f32, 0.95f32);
    let ring = |a: f32| ([a.sin() * r, a.cos() * r, len], mat::norm([a.sin(), a.cos(), -r / len]));
    let mut m = Mesh::default();
    (0..sides).for_each(|i| {
        let step = std::f32::consts::TAU / sides as f32;
        let (a0, a1) = (i as f32 * step, (i + 1) as f32 * step);
        let ((p0, n0), (p1, n1), (_, nm)) = (ring(a0), ring(a1), ring((a0 + a1) * 0.5));
        let base = m.verts.len() as u16;
        m.verts.extend([Vert::new([0.0; 3], [0.22, 0.0], nm), Vert::new(p0, [0.0, 0.0], n0), Vert::new(p1, [0.0, 0.0], n1)]);
        m.idx.extend([base, base + 1, base + 2]);
    });
    m
}

fn ui_indices() -> Vec<u16> {
    (0..ui2d::MAX_QUADS as u16).flat_map(|q| [0, 1, 2, 0, 2, 3].map(|i| q * 4 + i)).collect()
}

fn tv_lights(screens: &Mesh) -> Vec<Light> {
    screens
        .verts
        .chunks(4)
        .map(|q| {
            let c = mat::scale(q.iter().fold([0.0; 3], |a, v| mat::add(a, v.pos)), 0.25);
            let n = q[0].normal();
            Light { pos: mat::add(c, mat::scale(n, 0.25)), dir: n, power: 0.0, cutoff: -2.0 }
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

impl Renderer {
    pub fn new(gl: &Gl, map: &'static Map<'static>) -> Renderer {
        let shaders = Shaders::new(gl).unwrap_or_else(|e| panic!("doppio shaders: {e}"));
        let textures = Textures::new(gl);
        let lamps = Lamps::new(map);
        let level = Level::new(map, &lamps);
        let crowd = Crowd::new();
        let doors = door_mesh();
        let cone = cone_mesh();
        let buf = Buffers {
            level_vbo: static_buffer(gl, gl::ARRAY_BUFFER, &level.verts),
            level_ibo: static_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &level.idx),
            door_vbo: static_buffer(gl, gl::ARRAY_BUFFER, &doors.verts),
            door_ibo: static_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &doors.idx),
            door_count: doors.idx.len() as i32,
            screen_vbo: static_buffer(gl, gl::ARRAY_BUFFER, &level.screens.verts),
            screen_count: level.screens.verts.len() as i32,
            body_vbo: gl.gen_buffer(),
            body_ibo: static_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &crowd.indices(MAX_BODIES)),
            cone_vbo: static_buffer(gl, gl::ARRAY_BUFFER, &cone.verts),
            cone_ibo: static_buffer(gl, gl::ELEMENT_ARRAY_BUFFER, &cone.idx),
            cone_count: cone.idx.len() as i32,
            quad_vbo: static_buffer(gl, gl::ARRAY_BUFFER, &[0.0f32, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
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
        let nv = crowd.slot_verts();
        Renderer {
            map,
            shaders,
            textures,
            buf,
            target: Target { fbo, color, depth, font },
            tvs: tv_lights(&level.screens),
            level,
            lamps,
            skinned: vec![Vert::default(); nv * MAX_BODIES],
            crowd,
            rigs: [None; MAX_BODIES],
            lanterns: [None; MAX_BODIES],
            body_tex: texture::id(BODY_TEX),
            door_tex: texture::id("madeira_porta"),
            frame: 0,
            power_seen: true,
            flicker: 0,
            stats: Stats::default(),
        }
    }

    pub fn stats(&self) -> Stats {
        self.stats
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
        [b.level_vbo, b.level_ibo, b.door_vbo, b.door_ibo, b.screen_vbo, b.body_vbo, b.body_ibo, b.cone_vbo, b.cone_ibo, b.quad_vbo, b.ui_vbo, b.ui_ibo]
            .iter()
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

    fn body_light(&self, b: &Body) -> (impl Fn(V3) -> f32 + use<>, bool) {
        let (x, z) = (b.pos.x.to_f32(), b.pos.y.to_f32());
        let room = lamps::room_near(self.map, x.floor() as i32, z.floor() as i32).unwrap_or(0);
        let c = [x, 1.0, z];
        let mut l = [([0.0f32; 3], 0.0f32); 3];
        let rb = self.lamps.rooms[room as usize];
        match rb.outdoor {
            true => l[0] = (mat::norm([0.42, 0.78, 0.46]), rb.level * 2.2),
            false => self.lamps.lamps(room).take(3).zip(l.iter_mut()).for_each(|(p, slot)| {
                let d = mat::sub(p, c);
                let d2 = mat::dot(d, d);
                *slot = (mat::norm(d), rb.level * 2.6 / (1.0 + 0.3 * d2));
            }),
        }
        (move |n: V3| l.iter().map(|(d, w)| w * (mat::dot(n, *d).max(0.0) * 0.8 + 0.2)).sum::<f32>().min(1.0), rb.outdoor)
    }

    fn pose_bodies(&mut self, gl: &Gl, bodies: &[Body]) {
        let nv = self.crowd.slot_verts();
        let mut last = 0;
        (0..MAX_BODIES).for_each(|i| {
            self.rigs[i] = None;
            self.lanterns[i] = None;
            let Some(b) = bodies.get(i).filter(|b| b.visible) else { return };
            let (x, z) = (b.pos.x.to_f32(), b.pos.y.to_f32());
            let Rig { mats, lantern, beam } = mannequin::rig(x, z, b.heading.to_rad(), b.anim, b.anim_tic, b.lantern);
            let (light, outdoor) = self.body_light(b);
            self.crowd.skin(&mats, light, &mut self.skinned[i * nv..(i + 1) * nv]);
            self.rigs[i] = Some(([x, 0.9, z], outdoor));
            self.lanterns[i] = b.lantern.then_some(Light { pos: lantern, dir: beam, power: 2.4, cutoff: 0.74 });
            last = i + 1;
        });
        if last > 0 {
            gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.body_vbo);
            gl.buffer_data(gl::ARRAY_BUFFER, &self.skinned[..last * nv], gl::STREAM_DRAW);
        }
    }

    fn pick_lights(&self, focus: V3, frame: u32, power: f32) -> [Light; MAX_LIGHTS] {
        let mut out = [Light::default(); MAX_LIGHTS];
        let mut dist = [f32::MAX; MAX_LIGHTS];
        let tv = self.tvs.iter().enumerate().map(|(k, l)| Light {
            power: power * (0.22 + 0.2 * unit(frame / 2 + k as u32 * 977)),
            ..*l
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
        let scene = Scene { bodies: world.bodies(), doors: world.doors(), power: world.power_on() };
        self.draw(gl, fbo, width, height, &scene, views, ui);
    }

    pub fn draw(&mut self, gl: &Gl, fbo: u32, width: u32, height: u32, scene: &Scene, views: &[View], ui: &Ui2d) {
        self.frame = self.frame.wrapping_add(1);
        self.stats = Stats::default();
        let lamp = self.lamp_level(scene.power);
        self.pose_bodies(gl, scene.bodies);

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
        [0, 1, 2].iter().for_each(|&a| gl.enable_attrib(a));
        views.iter().take(4).enumerate().for_each(|(i, v)| self.draw_view(gl, i, v, scene, lamp));

        gl.disable(gl::DEPTH_TEST);
        gl.disable(gl::CULL_FACE);
        gl.disable_attrib(1);
        gl.disable_attrib(2);
        gl.bind_framebuffer(fbo);
        gl.viewport(0, 0, width as i32, height as i32);
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
    }

    fn camera(&self, body: Option<&Body>, aspect: f32) -> Camera {
        let (x, z, h) = body.map_or((1.5, 1.5, 0.0), |b| (b.pos.x.to_f32(), b.pos.y.to_f32(), b.heading.to_rad()));
        Camera::follow(self.map, x, z, aspect).unwrap_or_else(|| {
            let eye = [x - h.sin() * 2.5, 2.2, z - h.cos() * 2.5];
            let vp = mat::mul(&mat::perspective(1.0, aspect, camera::NEAR, camera::FAR), &mat::look_at(eye, [x, 1.0, z]));
            Camera { eye, view_proj: vp, frustum: mat::Frustum::new(&vp) }
        })
    }

    fn draw_view(&mut self, gl: &Gl, slot: usize, v: &View, scene: &Scene, lamp: f32) {
        let [x, y, w, h] = v.rect.map(|c| c as i32);
        if w <= 0 || h <= 0 {
            return;
        }
        gl.viewport(x, CANVAS_H as i32 - y - h, w, h);
        let bodies = scene.bodies;
        let follow = bodies.get(v.follow as usize);
        let cam = self.camera(follow, w as f32 / h as f32);
        let focus = follow.map_or(cam.eye, |b| [b.pos.x.to_f32(), 1.0, b.pos.y.to_f32()]);
        let lights = self.pick_lights(focus, self.frame, lamp);
        let dark = lamp < 0.5;
        let seeds = [focus, cam.eye]
            .iter()
            .filter_map(|p| lamps::room_near(self.map, p[0].floor() as i32, p[2].floor() as i32))
            .fold(0u32, |m, r| m | 1 << r);
        let ajar = |x: u8, y: u8| scene.doors.iter().any(|d| d.x == x && d.y == y && d.open > 0);
        let reach = self
            .level
            .portals
            .iter()
            .filter(|p| p.rooms & seeds != 0 && ajar(p.x, p.y) && cam.frustum.aabb(p.lo, p.hi))
            .fold(seeds, |m, p| m | p.rooms);
        let p = &self.shaders.world;
        gl.use_program(p.id);
        self.set_scene(gl, p, &cam, &lights, dark, [w as f32 * 0.5, h as f32 * 0.5]);
        gl.uniform_mat4(p.loc[wu::MODEL], &mat::IDENTITY);
        gl.uniform_mat4(p.loc[wu::MVP], &cam.view_proj);

        gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.level_vbo);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.level_ibo);
        Self::world_attribs(gl);
        let mut tris = 0u32;
        self.level.rooms.iter().enumerate().filter(|(i, r)| reach >> i & 1 != 0 && cam.frustum.aabb(r.lo, r.hi)).for_each(|(_, r)| {
            let c = if r.outdoor { MOON } else { mat::scale(WARM, lamp) };
            gl.uniform3f(p.loc[wu::LAMP], c[0], c[1], c[2]);
            self.level.sections[r.sections.clone()].iter().for_each(|s| {
                gl.bind_texture(0, self.textures.ids[s.tex as usize]);
                gl.draw_elements_u16(gl::TRIANGLES, s.count as i32, s.first * 2);
                tris += s.count as u32 / 3;
                self.stats.draws += 1;
            });
        });

        gl.bind_texture(0, self.textures.ids[self.door_tex as usize]);
        gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.door_vbo);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.door_ibo);
        Self::world_attribs(gl);
        scene.doors.iter().for_each(|d| {
            let (dx, dy) = (d.x as i32, d.y as i32);
            let centre = [dx as f32 + 0.5, 1.0, dy as f32 + 0.5];
            if !cam.frustum.sphere(centre, 1.2) {
                return;
            }
            let horizontal = matches!(self.map.tile(dx - 1, dy), Tile::Wall | Tile::Window);
            let swing = d.open as f32 / 65535.0 * 1.62;
            let (hinge, base) = match horizontal {
                true => ([dx as f32, 0.0, dy as f32 + 0.5], 0.0),
                false => ([dx as f32 + 0.5, 0.0, dy as f32], -std::f32::consts::FRAC_PI_2),
            };
            let model = mat::mul(&mat::translate(hinge), &mat::rot_y(base + swing));
            let room = lamps::room_near(self.map, dx, dy).unwrap_or(0);
            let l = self.lamps.light(room, centre, None) * lamp;
            gl.uniform3f(p.loc[wu::LAMP], WARM[0] * l, WARM[1] * l, WARM[2] * l);
            gl.uniform_mat4(p.loc[wu::MODEL], &model);
            gl.uniform_mat4(p.loc[wu::MVP], &mat::mul(&cam.view_proj, &model));
            gl.draw_elements_u16(gl::TRIANGLES, self.buf.door_count, 0);
            tris += self.buf.door_count as u32 / 3;
            self.stats.draws += 1;
        });

        gl.uniform_mat4(p.loc[wu::MODEL], &mat::IDENTITY);
        gl.uniform_mat4(p.loc[wu::MVP], &cam.view_proj);
        gl.bind_texture(0, self.textures.ids[self.body_tex as usize]);
        gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.body_vbo);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.body_ibo);
        Self::world_attribs(gl);
        let detail = LOD_DISTANCE * h as f32 / CANVAS_H as f32;
        self.rigs.iter().enumerate().for_each(|(i, r)| {
            let Some((c, outdoor)) = r.filter(|(c, _)| cam.frustum.sphere(*c, 1.0)) else { return };
            let k = if outdoor { MOON } else { mat::scale(WARM, lamp) };
            gl.uniform3f(p.loc[wu::LAMP], k[0], k[1], k[2]);
            let (offset, count) = self.crowd.range(i, mat::len(mat::sub(c, cam.eye)) > detail);
            gl.draw_elements_u16(gl::TRIANGLES, count as i32, offset);
            tris += count as u32 / 3;
            self.stats.draws += 1;
        });

        if self.buf.screen_count > 0 {
            let s = &self.shaders.screen;
            gl.use_program(s.id);
            self.set_scene(gl, s, &cam, &lights, dark, [w as f32 * 0.5, h as f32 * 0.5]);
            gl.uniform_mat4(s.loc[wu::MODEL], &mat::IDENTITY);
            gl.uniform_mat4(s.loc[wu::MVP], &cam.view_proj);
            gl.uniform3f(s.loc[wu::LAMP], 0.0, 0.0, 0.0);
            let f = self.frame;
            gl.uniform4f(s.loc[wu::FX], unit(f * 3 + 1), unit(f * 7 + 2), lamp, 0.015 + 0.03 * lamp);
            gl.bind_texture(0, self.textures.noise);
            gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.screen_vbo);
            Self::world_attribs(gl);
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.ui_ibo);
            gl.draw_elements_u16(gl::TRIANGLES, self.buf.screen_count / 4 * 6, 0);
            self.stats.draws += 1;
        }

        let g = &self.shaders.glow;
        let cones: u32 = self.lanterns.iter().flatten().filter(|l| cam.frustum.sphere(mat::add(l.pos, mat::scale(l.dir, 1.4)), 1.6)).count() as u32;
        if cones > 0 {
            gl.use_program(g.id);
            gl.uniform2f(g.loc[glow::SNAP], w as f32 * 0.5, h as f32 * 0.5);
            gl.uniform3f(g.loc[glow::COLOR], 0.26, 0.22, 0.15);
            gl.uniform3f(g.loc[glow::EYE], cam.eye[0], cam.eye[1], cam.eye[2]);
            gl.enable(gl::BLEND);
            gl.blend_func(gl::ONE, gl::ONE);
            gl.depth_mask(false);
            gl.disable(gl::CULL_FACE);
            gl.bind_buffer(gl::ARRAY_BUFFER, self.buf.cone_vbo);
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.buf.cone_ibo);
            Self::world_attribs(gl);
            self.lanterns.iter().flatten().filter(|l| cam.frustum.sphere(mat::add(l.pos, mat::scale(l.dir, 1.4)), 1.6)).for_each(|l| {
                let model = mat::mul(
                    &mat::mul(&mat::translate(l.pos), &mat::rot_y(l.dir[0].atan2(l.dir[2]))),
                    &mat::rot_x((-l.dir[1]).asin()),
                );
                gl.uniform_mat4(g.loc[glow::MVP], &mat::mul(&cam.view_proj, &model));
                gl.uniform_mat4(g.loc[glow::MODEL], &model);
                gl.draw_elements_u16(gl::TRIANGLES, self.buf.cone_count, 0);
                self.stats.draws += 1;
            });
            gl.depth_mask(true);
            gl.enable(gl::CULL_FACE);
            gl.disable(gl::BLEND);
        }
        self.stats.tris[slot] = tris;
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
        views.iter().take(4).for_each(|v| {
            let r = [v.rect[0] as f32, v.rect[1] as f32, (v.rect[0] + v.rect[2]) as f32, (v.rect[1] + v.rect[3]) as f32];
            match v.fade > 0 || v.duplo {
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
                    let tint = if v.duplo { [k * 1.08, k * 0.86, k * 0.84] } else { [k; 3] };
                    gl.uniform3f(p.loc[blit::TINT], tint[0], tint[1], tint[2]);
                    let grain = if v.duplo { 0.16 } else { 0.04 * (1.0 - k) };
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

    #[test]
    fn cone_and_door_meshes_are_small() {
        assert_eq!(cone_mesh().tris(), 10);
        assert!(door_mesh().tris() <= 12);
    }

    #[test]
    fn body_slots_fit_u16() {
        let c = Crowd::new();
        assert!(c.slot_verts() * MAX_BODIES <= u16::MAX as usize);
    }

    #[test]
    fn hash_spreads() {
        let v: Vec<f32> = (0..1000).map(unit).collect();
        let mean = v.iter().sum::<f32>() / v.len() as f32;
        assert!((0.4..0.6).contains(&mean));
    }
}
