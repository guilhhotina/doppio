mod backdrop;
mod bench;
mod clock;
mod flow;
mod game;
mod layout;
mod pad;

use crate::audio::Audio;
use crate::core::{Frontend, HEIGHT, SAMPLE_RATE, WIDTH};
use crate::ffi::LOG_INFO;
use crate::gl::{self, Gl};
use crate::map;
use crate::net::{self, Link, Net};
use crate::render::{Renderer, Tell, Ui2d, View};
use crate::sim::{Anim, MAX_BODIES, MAX_PLAYERS};
use crate::ui;
use backdrop::Backdrop;
use bench::{Bench, Spec};
use clock::Clock;
use flow::{Notice, Screen};
use game::Game;
use pad::Pad;
use std::ffi::CStr;
use std::time::{Duration, Instant};

const PERF_FRAMES: u32 = 300;
const MENU_FADE: u8 = 175;
const TITLE_FADE: u8 = 60;

#[derive(Default)]
struct Perf {
    frames: u32,
    total: Duration,
    worst: Duration,
}

pub struct App {
    net: Net,
    audio: Audio,
    gfx: Option<(Gl, Renderer)>,
    ui: Ui2d,
    pad: Pad,
    screen: Screen,
    game: Option<Box<Game>>,
    backdrop: Backdrop,
    bench: Option<(Bench, usize, Clock)>,
    ports: [Option<u8>; MAX_PLAYERS],
    night_secs: u32,
    hashlog: u32,
    frame: u32,
    perf: Perf,
}

fn number<T: std::str::FromStr>(fe: &Frontend, key: &CStr) -> Option<T> {
    fe.variable(key).and_then(|v| v.trim().parse().ok())
}

fn triple(fe: &Frontend, key: &CStr) -> Option<[f32; 3]> {
    fe.variable(key)
        .map(|v| v.split(',').filter_map(|s| s.trim().parse::<f32>().ok()).collect::<Vec<f32>>())
        .filter(|v| v.len() == 3)
        .map(|v| [v[0], v[1], v[2]])
}

fn bench_spec(fe: &Frontend) -> Option<(Spec, usize)> {
    let bodies = number::<usize>(fe, c"doppio_bench")?;
    let spawn = map::sobrado().spawns[0];
    let at = triple(fe, c"doppio_at").unwrap_or([spawn.pos.x.to_f32(), spawn.pos.y.to_f32(), spawn.rot.to_rad().to_degrees()]);
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
        tells: match fe.variable(c"doppio_tells").as_deref() {
            Some("pending") => Tell::Pending,
            Some("done") => Tell::Done,
            _ => Tell::Idle,
        },
        gaze: number::<f32>(fe, c"doppio_gaze"),
        caught: number::<u8>(fe, c"doppio_caught").is_some_and(|c| c != 0),
        echo: triple(fe, c"doppio_echo"),
    };
    Some((spec, number::<usize>(fe, c"doppio_views").unwrap_or(1).clamp(1, 4)))
}

fn ports_list(ports: &[Option<u8>; MAX_PLAYERS]) -> ([u8; MAX_PLAYERS], usize) {
    let mut it = ports.iter().flatten().copied();
    let list = std::array::from_fn(|_| it.next().unwrap_or(0));
    (list, ports.iter().flatten().count())
}

impl App {
    pub fn new(fe: &Frontend) -> App {
        net::register(fe);
        let net = Net::new(fe);
        let night_secs = flow::night_choice(number(fe, c"doppio_night_seconds"));
        let bench = bench_spec(fe).map(|(s, views)| (Bench::new(map::sobrado(), &s), views, Clock::new(Instant::now())));
        fe.log(
            LOG_INFO,
            &format!("{} loaded (night_seconds={night_secs}, lobby={})", env!("CARGO_PKG_VERSION"), net.lobby_supported()),
        );
        App {
            net,
            audio: Audio::new(SAMPLE_RATE),
            gfx: None,
            ui: Ui2d::new(),
            pad: Pad::default(),
            screen: Screen::Title,
            game: None,
            backdrop: Backdrop::new(map::sobrado()),
            bench,
            ports: [Some(0), None, None, None],
            night_secs,
            hashlog: number(fe, c"doppio_hashlog").unwrap_or(0),
            frame: 0,
            perf: Perf::default(),
        }
    }

    pub fn reset(&mut self) {
        if self.game.as_ref().is_some_and(|g| !g.online()) {
            self.quit_to(Screen::main());
        }
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

    fn quit_to(&mut self, screen: Screen) {
        self.net.leave();
        self.game = None;
        self.audio = Audio::new(SAMPLE_RATE);
        self.screen = screen;
    }

    fn begin(&mut self, solo: bool) {
        if solo {
            self.ports = [Some(0), None, None, None];
        }
        let (list, n) = ports_list(&self.ports);
        self.net.set_local_players(&list[..n]);
        self.net.set_night_secs(self.night_secs);
        self.net.start();
    }

    fn watch_net(&mut self, fe: &Frontend) {
        let link = self.net.link();
        let ended = self.game.as_ref().is_some_and(|g| g.over());
        match (link, &self.game) {
            (Link::Lost, _) if !ended => {
                let notice = flow::lost_notice(self.net.lost_reason());
                self.quit_to(Screen::Notice(notice));
            }
            (_, Some(g)) if g.online() && !ended && self.net.desynced() => self.quit_to(Screen::Notice(Notice::Desync)),
            (Link::Hosting | Link::Joined, Some(g)) if g.online() && self.net.started().is_none() => {
                self.game = None;
                self.screen = Screen::Lobby { since: self.frame, hosting: link == Link::Hosting, sel: 0 };
            }
            (_, None) => match self.net.started() {
                Some(setup) => {
                    let online = link != Link::Offline;
                    fe.log(
                        LOG_INFO,
                        &format!("game begins: seed {:08x}, {} players, {} s nights, online {online}", setup.seed, setup.players, setup.night_secs),
                    );
                    self.game = Some(Box::new(Game::new(setup, self.net.seats(), online, self.hashlog, Instant::now())));
                    self.audio = Audio::new(SAMPLE_RATE);
                    self.screen = Screen::Game;
                }
                None if link != Link::Offline && !matches!(self.screen, Screen::Lobby { .. }) => {
                    self.screen = Screen::Lobby { since: self.frame, hosting: link == Link::Hosting, sel: 0 };
                }
                None => {}
            },
            _ => {}
        }
    }

    fn join_ports(&mut self, edges: &[u16; 4]) -> bool {
        let fresh = (1..4u8).find(|&p| edges[p as usize] & pad::START != 0 && !self.ports.contains(&Some(p)));
        let left = (1..4u8).find(|&p| edges[p as usize] & pad::B != 0 && self.ports.contains(&Some(p)));
        let changed = match (fresh, left, self.ports.iter().position(Option::is_none)) {
            (Some(p), _, Some(slot)) => {
                self.ports[slot] = Some(p);
                true
            }
            (None, Some(p), _) => {
                self.ports.iter_mut().filter(|s| **s == Some(p)).for_each(|s| *s = None);
                let (list, n) = ports_list(&self.ports);
                self.ports = std::array::from_fn(|i| (i < n).then(|| list[i]));
                true
            }
            _ => false,
        };
        if changed {
            let (list, n) = ports_list(&self.ports);
            self.net.set_local_players(&list[..n]);
        }
        changed
    }

    fn menus(&mut self, edges: &[u16; 4]) {
        let guest = matches!(self.screen, Screen::Couch { .. } | Screen::Lobby { .. }) && self.join_ports(edges);
        let own = if guest { 0 } else { edges[0] | edges[1..].iter().fold(0, |m, e| m | e & !(pad::START | pad::B)) };
        let key = pad::key(own);
        let ctx = flow::Ctx {
            lobby: self.net.lobby_supported(),
            rooms: self.net.rooms().len(),
            link: self.net.link(),
            players: self.ports.iter().flatten().count(),
            remote: self.net.seats().iter().any(|s| s.local_port.is_none()),
            frame: self.frame,
        };
        let (next, action) = flow::step(self.screen, key, &ctx);
        self.screen = next;
        match action {
            flow::Action::None => {}
            flow::Action::Solo => self.begin(true),
            flow::Action::Couch => self.begin(false),
            flow::Action::Night(dir) => self.night_secs = flow::next_night(self.night_secs, dir),
            flow::Action::Host { public, relay } => {
                let (list, n) = ports_list(&self.ports);
                self.net.set_local_players(&list[..n]);
                self.net.host(public, flow::RELAYS[relay].1);
            }
            flow::Action::Refresh => self.net.refresh_rooms(),
            flow::Action::Join(i) => {
                let (list, n) = ports_list(&self.ports);
                self.net.set_local_players(&list[..n]);
                self.net.join(i);
            }
            flow::Action::Start => {
                self.net.set_night_secs(self.night_secs);
                self.net.start();
            }
            flow::Action::Ready => {
                let ready = !self.net.ready();
                self.net.set_ready(ready);
            }
            flow::Action::Leave => self.quit_to(self.screen),
            flow::Action::CouchReset => self.ports = [Some(0), None, None, None],
        }
    }

    fn in_game(&mut self, fe: &Frontend, edges: &[u16; 4], input: &[u16; 4]) {
        let Some(g) = self.game.as_mut() else { return };
        let now = Instant::now();
        let key = pad::key(pad::merged(edges));
        let held = g.held_to_pause(input, now);
        match (g.over(), g.pause) {
            (true, _) if key == Some(pad::Key::Confirm) => return self.quit_to(Screen::main()),
            (true, _) => {}
            (false, None) if held || pad::merged(edges) & pad::PAUSE != 0 => g.pause = Some(0),
            (false, None) => {}
            (false, Some(sel)) => match key {
                Some(pad::Key::Confirm) if sel == 1 => return self.quit_to(Screen::main()),
                Some(pad::Key::Confirm | pad::Key::Back) => g.resume(input),
                k => g.pause = Some(pad::list(sel, 2, k)),
            },
        }
        g.advance(fe, &mut self.net, &mut self.audio, input, now);
    }

    fn draw_menus(&mut self) {
        let ui = &mut self.ui;
        let t = self.frame;
        match self.screen {
            Screen::Title => ui::title(ui, t, concat!("v", env!("CARGO_PKG_VERSION"))),
            Screen::Game => {}
            s => flow::draw(ui, s, &self.net, &self.ports, self.night_secs, t),
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

    fn bench_frame(&mut self) -> bool {
        let Some((b, n, clock)) = &mut self.bench else { return false };
        clock.run(Instant::now(), || {
            b.step();
            true
        });
        let rects = layout::rects(*n);
        let views: [View; 4] = std::array::from_fn(|i| View { rect: rects[i], follow: (i * 5) as u8, fade: 0, duplo: false });
        self.ui.clear();
        let Some((gl, r)) = &mut self.gfx else { return false };
        r.draw(gl, gl.target_framebuffer(), WIDTH, HEIGHT, &b.scene(), &views[..*n], &self.ui);
        true
    }

    pub fn frame(&mut self, fe: &Frontend, input: [u16; 4]) -> bool {
        let start = Instant::now();
        self.frame = self.frame.wrapping_add(1);
        if self.bench.is_some() {
            let drawn = self.bench_frame();
            self.report(fe, start.elapsed());
            return drawn;
        }
        self.net.poll(fe);
        let edges = self.pad.edges(&input);
        self.watch_net(fe);
        match self.screen {
            Screen::Game => self.in_game(fe, &edges, &input),
            _ => self.menus(&edges),
        }
        self.watch_net(fe);
        if self.game.is_none() {
            self.backdrop.tick(self.frame);
        }
        let views = self.game.as_ref().filter(|_| self.screen == Screen::Game).map_or(1, |g| g.views().1);
        if !self.gfx.as_mut().is_some_and(|(_, r)| r.due(views)) {
            self.report(fe, start.elapsed());
            return false;
        }
        self.ui.clear();
        self.draw_menus();
        let Some((gl, r)) = &mut self.gfx else { return false };
        let target = gl.target_framebuffer();
        match &self.game {
            Some(g) if self.screen == Screen::Game => {
                g.draw(&mut self.ui, self.frame);
                let (views, n) = g.views();
                r.frame(gl, target, WIDTH, HEIGHT, g.world(), &views[..n], &self.ui);
            }
            _ => {
                let dim = if self.screen == Screen::Title { TITLE_FADE } else { MENU_FADE };
                r.draw(gl, target, WIDTH, HEIGHT, &self.backdrop.scene(), &[self.backdrop.view(dim)], &self.ui);
            }
        }
        self.report(fe, start.elapsed());
        true
    }

    pub fn audio(&mut self, out: &mut [i16]) {
        self.audio.fill(out);
    }
}
