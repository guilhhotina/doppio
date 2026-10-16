mod bench;
mod hud;
mod layout;

use crate::audio::Audio;
use crate::core::{Frontend, HEIGHT, SAMPLE_RATE, WIDTH};
use crate::ffi::LOG_INFO;
use crate::gl::{self, Gl};
use crate::map;
use crate::render::{Renderer, Ui2d, View};
use crate::sim::{Anim, MAX_BODIES, MAX_PLAYERS, Role, Setup, TICS_PER_SEC, World};
use bench::{Bench, Spec};
use std::ffi::CStr;
use std::time::{Duration, Instant};

const TIC: Duration = Duration::from_nanos(1_000_000_000 / TICS_PER_SEC as u64);
const MAX_CATCHUP: u32 = 5;
const START: u16 = 1 << 3;
const DEFAULT_NIGHT_SECS: u32 = 360;
const PERF_FRAMES: u32 = 300;

#[derive(Default)]
struct Perf {
    frames: u32,
    total: Duration,
    worst: Duration,
}

enum Mode {
    Play(Box<World>),
    Bench(Bench, usize),
}

pub struct App {
    mode: Mode,
    audio: Audio,
    gfx: Option<(Gl, Renderer)>,
    ui: Ui2d,
    ports: [Option<u8>; MAX_PLAYERS],
    night_secs: u32,
    seed: u32,
    last: Instant,
    acc: Duration,
    held: [u16; 4],
    perf: Perf,
}

fn number<T: std::str::FromStr>(fe: &Frontend, key: &CStr) -> Option<T> {
    fe.variable(key).and_then(|v| v.trim().parse().ok())
}

fn bench_spec(fe: &Frontend) -> Option<(Spec, usize)> {
    let bodies = number::<usize>(fe, c"doppio_bench")?;
    let spawn = map::sobrado().spawns[0];
    let at = fe
        .variable(c"doppio_at")
        .map(|v| v.split(',').filter_map(|s| s.trim().parse::<f32>().ok()).collect::<Vec<f32>>())
        .filter(|v| v.len() == 3)
        .map_or([spawn.pos.x.to_f32(), spawn.pos.y.to_f32(), spawn.rot.to_rad().to_degrees()], |v| [v[0], v[1], v[2]]);
    let anim = match fe.variable(c"doppio_anim").as_deref() {
        Some("walk") => Anim::Walk,
        Some("run") => Anim::Run,
        Some("reach") => Anim::Reach,
        Some("stagger") => Anim::Stagger,
        _ => Anim::Idle,
    };
    let spec = Spec {
        bodies: bodies.clamp(1, MAX_BODIES),
        at,
        power: number::<u8>(fe, c"doppio_power") != Some(0),
        lantern: number::<u8>(fe, c"doppio_lantern") != Some(0),
        anim,
    };
    Some((spec, number::<usize>(fe, c"doppio_views").unwrap_or(1).clamp(1, 4)))
}

fn world(seed: u32, players: u8, night_secs: u32) -> Box<World> {
    Box::new(World::new(map::sobrado(), Setup { seed, players, night_secs }))
}

impl App {
    pub fn new(fe: &Frontend) -> App {
        let night_secs = number(fe, c"doppio_night_seconds").unwrap_or(DEFAULT_NIGHT_SECS);
        let seed = number(fe, c"doppio_seed").unwrap_or(0x5eed_d0b0);
        let mode = match bench_spec(fe) {
            Some((s, views)) => Mode::Bench(Bench::new(map::sobrado(), &s), views),
            None => Mode::Play(world(seed, 1, night_secs)),
        };
        fe.log(LOG_INFO, &format!("{} loaded (night_seconds={night_secs})", env!("CARGO_PKG_VERSION")));
        let mut ports = [None; MAX_PLAYERS];
        ports[0] = Some(0);
        App {
            mode,
            audio: Audio::new(SAMPLE_RATE),
            gfx: None,
            ui: Ui2d::new(),
            ports,
            night_secs,
            seed,
            last: Instant::now(),
            acc: Duration::ZERO,
            held: [0; 4],
            perf: Perf::default(),
        }
    }

    fn players(&self) -> usize {
        self.ports.iter().flatten().count()
    }

    fn restart(&mut self) {
        let players = self.players() as u8;
        if let Mode::Play(w) = &mut self.mode {
            *w = world(self.seed, players, self.night_secs);
            self.audio = Audio::new(SAMPLE_RATE);
            self.acc = Duration::ZERO;
        }
    }

    pub fn reset(&mut self) {
        self.restart();
    }

    pub fn context_reset(&mut self, fe: &Frontend, gl: Gl) {
        fe.log(LOG_INFO, &format!("GL renderer: {}", gl.string(gl::RENDERER).unwrap_or("?")));
        let r = Renderer::new(&gl, map::sobrado());
        fe.log(
            LOG_INFO,
            &format!("level {} tris, mannequin {:?} tris, textures {} KB", r.level_tris(), r.body_tris(), r.texture_bytes() / 1024),
        );
        self.gfx = Some((gl, r));
    }

    pub fn context_destroy(&mut self) {
        self.gfx = None;
    }

    fn join(&mut self, input: &[u16; 4]) {
        let pressed = |p: u8| input[p as usize] & START != 0 && self.held[p as usize] & START == 0;
        let fresh = (1..4u8).find(|&p| pressed(p) && !self.ports.contains(&Some(p)));
        self.held = *input;
        if let (Some(port), Some(slot)) = (fresh, self.ports.iter().position(Option::is_none)) {
            self.ports[slot] = Some(port);
            self.restart();
        }
    }

    fn step(&mut self, input: &[u16; 4]) {
        let now = Instant::now();
        self.acc = (self.acc + (now - self.last)).min(TIC * MAX_CATCHUP);
        self.last = now;
        let inputs: [u16; MAX_PLAYERS] = std::array::from_fn(|s| self.ports[s].map_or(0, |p| input[p as usize]));
        let tics = (self.acc.as_nanos() / TIC.as_nanos()) as u32;
        self.acc -= TIC * tics;
        match &mut self.mode {
            Mode::Bench(b, _) => (0..tics).for_each(|_| b.step()),
            Mode::Play(w) => {
                (0..tics).for_each(|_| {
                    w.step(&inputs);
                    self.audio.events(w, w.events(), w.player(0).body);
                });
                self.audio.ambience(w, w.player(0).body);
            }
        }
    }

    fn views(&self) -> ([View; 4], usize) {
        match &self.mode {
            Mode::Bench(_, n) => {
                let rects = layout::rects(*n);
                (std::array::from_fn(|i| View { rect: rects[i], follow: (i * 5) as u8, fade: 0, duplo: false }), *n)
            }
            Mode::Play(w) => {
                let n = self.players();
                let rects = layout::rects(n);
                let views = std::array::from_fn(|i| {
                    let p = w.player(i);
                    View { rect: rects[i], follow: p.body, fade: p.fade, duplo: p.role == Role::Duplo }
                });
                (views, n)
            }
        }
    }

    fn report(&mut self, fe: &Frontend, spent: Duration) {
        let p = &mut self.perf;
        p.frames += 1;
        p.total += spent;
        p.worst = p.worst.max(spent);
        if p.frames == PERF_FRAMES {
            let stats = self.gfx.as_ref().map(|(_, r)| r.stats()).unwrap_or_default();
            fe.log(
                LOG_INFO,
                &format!(
                    "frame avg {:.2} ms max {:.2} ms, tris/view {:?}, draws {}",
                    p.total.as_secs_f64() * 1000.0 / p.frames as f64,
                    p.worst.as_secs_f64() * 1000.0,
                    stats.tris,
                    stats.draws
                ),
            );
            *p = Perf::default();
        }
    }

    pub fn frame(&mut self, fe: &Frontend, input: [u16; 4]) -> bool {
        let start = Instant::now();
        if matches!(self.mode, Mode::Play(_)) {
            self.join(&input);
        }
        self.step(&input);
        let (views, n) = self.views();
        self.ui.clear();
        if let Mode::Play(w) = &self.mode {
            (0..n).for_each(|i| hud::draw(&mut self.ui, w, views[i].rect, i, n == 1));
        }
        let Some((gl, r)) = &mut self.gfx else { return false };
        let target = gl.target_framebuffer();
        match &self.mode {
            Mode::Bench(b, _) => r.draw(gl, target, WIDTH, HEIGHT, &b.scene(), &views[..n], &self.ui),
            Mode::Play(w) => r.frame(gl, target, WIDTH, HEIGHT, w, &views[..n], &self.ui),
        }
        self.report(fe, start.elapsed());
        true
    }

    pub fn audio(&mut self, out: &mut [i16]) {
        self.audio.fill(out);
    }
}
