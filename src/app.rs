mod autopilot;
mod backdrop;
mod clock;
mod flow;
mod game;
mod layout;
mod pad;
mod save;

use crate::audio::Audio;
use crate::core::{Frontend, HEIGHT, SAMPLE_RATE, WIDTH};
use crate::ffi::{LOG_INFO, LOG_WARN};
use crate::gl::{self, Gl};
use crate::map::{self, Map};
use crate::net::{self, Link, Net};
use crate::render::{Renderer, Ui2d, View};
use crate::sim::MAX_PLAYERS;
use crate::ui;
use autopilot::Autopilot;
use backdrop::Backdrop;
use flow::{Look, Notice, Screen};
use game::{Game, PauseAction, Stage, VIEWS};
use pad::Pad;
use std::path::PathBuf;
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
    maps: &'static [&'static Map<'static>],
    net: Net,
    audio: Audio,
    gfx: Option<(Gl, Renderer)>,
    ui: Ui2d,
    pad: Pad,
    screen: Screen,
    game: Option<Box<Game>>,
    backdrop: Backdrop,
    ports: [Option<u8>; MAX_PLAYERS],
    save_dir: Option<PathBuf>,
    unlocked: u8,
    tips: u32,
    focus: usize,
    hashlog: u32,
    stage: Option<Stage>,
    frame: u32,
    perf: Perf,
    autopilot: Option<Autopilot>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Direct {
    phase: u8,
    characters: [u8; MAX_PLAYERS],
    players: usize,
}

fn direct(phase: Option<&str>, chars: Option<&str>, phases: usize) -> Option<Direct> {
    let phase = phase.and_then(|p| p.trim().parse::<usize>().ok()).filter(|&p| p < phases);
    let list: Vec<u8> = chars.map_or(vec![], |c| c.split(',').filter_map(|s| s.trim().parse().ok()).filter(|&c: &u8| (c as usize) < MAX_PLAYERS).collect());
    (phase.is_some() || !list.is_empty()).then(|| Direct {
        phase: phase.unwrap_or(0) as u8,
        characters: std::array::from_fn(|i| list.get(i).copied().unwrap_or(i as u8)),
        players: list.len().clamp(1, MAX_PLAYERS),
    })
}

fn ports_list(ports: &[Option<u8>; MAX_PLAYERS]) -> ([u8; MAX_PLAYERS], usize) {
    let mut it = ports.iter().flatten().copied();
    let list = std::array::from_fn(|_| it.next().unwrap_or(0));
    (list, ports.iter().flatten().count())
}

fn side(bits: u16) -> Option<i8> {
    match (bits & pad::LEFT != 0, bits & pad::RIGHT != 0) {
        (true, false) => Some(-1),
        (false, true) => Some(1),
        _ => None,
    }
}

fn source(input: [u16; 4], bot: Option<u16>) -> [u16; 4] {
    bot.map_or(input, |b| [b, input[1], input[2], input[3]])
}

impl App {
    pub fn new(fe: &Frontend) -> App {
        net::register(fe);
        let maps = map::campaign();
        let save_dir = fe.save_dir();
        let save::Save { unlocked, tips } = save::load(save_dir.as_deref(), maps.len() as u8);
        let start = direct(fe.variable(c"doppio_phase").as_deref(), fe.variable(c"doppio_chars").as_deref(), maps.len());
        let bot = fe.variable(c"doppio_bot").is_some_and(|v| v.trim() == "1");
        fe.log(
            LOG_INFO,
            &format!(
                "{} loaded ({} phases, {unlocked} unlocked, save dir {:?}, direct {start:?}, bot {bot})",
                env!("CARGO_PKG_VERSION"),
                maps.len(),
                save_dir
            ),
        );
        let mut app = App {
            maps,
            net: Net::new(fe),
            audio: Audio::new(SAMPLE_RATE),
            gfx: None,
            ui: Ui2d::new(),
            pad: Pad::default(),
            screen: Screen::Title,
            game: None,
            backdrop: Backdrop::new(maps, Instant::now()),
            ports: [Some(0), None, None, None],
            save_dir,
            unlocked,
            tips,
            focus: 0,
            hashlog: fe.variable(c"doppio_hashlog").and_then(|v| v.trim().parse().ok()).unwrap_or(0),
            stage: fe.variable(c"doppio_stage").as_deref().and_then(game::stage_named),
            frame: 0,
            perf: Perf::default(),
            autopilot: bot.then(Autopilot::new),
        };
        start.iter().for_each(|d| {
            app.ports = std::array::from_fn(|i| (i < d.players).then_some(i as u8));
            app.sync_ports();
            d.characters.iter().take(d.players).enumerate().for_each(|(i, &c)| app.net.set_character(i, c));
            app.net.set_phase(d.phase);
            app.net.start();
        });
        app
    }

    pub fn reset(&mut self) {
        if self.game.as_ref().is_some_and(|g| !g.online()) {
            self.quit_to(Screen::main());
        }
    }

    pub fn context_reset(&mut self, fe: &Frontend, gl: Gl) {
        fe.log(LOG_INFO, &format!("GL renderer: {}", gl.string(gl::RENDERER).unwrap_or("?")));
        let r = Renderer::new(&gl, self.maps[0]);
        fe.log(LOG_INFO, &format!("level {} tris, textures {} KB", r.level_tris(), r.texture_bytes() / 1024));
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

    fn sync_ports(&mut self) {
        let (list, n) = ports_list(&self.ports);
        self.net.set_local_players(&list[..n]);
    }

    fn begin(&mut self) {
        self.sync_ports();
        self.net.start();
    }

    fn persist(&self, fe: &Frontend) {
        if let Err(e) = save::store(self.save_dir.as_deref(), save::Save { unlocked: self.unlocked, tips: self.tips }) {
            fe.log(LOG_WARN, &format!("could not save progress: {e}"));
        }
    }

    fn unlock(&mut self, fe: &Frontend, won: u8) {
        let next = (won as usize + 2).min(self.maps.len()) as u8;
        if next > self.unlocked {
            self.unlocked = next;
            self.persist(fe);
        }
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
                    fe.log(LOG_INFO, &format!("game begins: {setup:?}, online {online}"));
                    let mut g = Box::new(Game::new(self.maps, setup, self.net.seats(), online, self.hashlog, Instant::now()));
                    g.teach(self.tips);
                    self.stage.take().into_iter().for_each(|s| g.force(s));
                    self.game = Some(g);
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
            self.sync_ports();
        }
        changed
    }

    fn pick(&mut self, local: usize, dir: i8) {
        let (list, n) = ports_list(&self.ports);
        let seats = self.net.seats();
        let Some(i) = (local < n).then(|| seats.iter().position(|s| s.local_port == Some(list[local]))).flatten() else { return };
        let taken = seats.iter().enumerate().filter(|&(k, _)| k != i).fold(0u8, |m, (_, s)| m | 1 << s.character);
        let c = flow::next_free(seats[i].character, dir, taken);
        self.net.set_character(local, c);
        self.focus = i;
    }

    fn menus(&mut self, edges: &[u16; 4]) {
        let screen = self.screen;
        let guest = screen.joining() && self.join_ports(edges);
        if screen.picking() {
            (1..4usize).for_each(|p| {
                let local = self.ports.iter().flatten().position(|&q| q as usize == p);
                if let (Some(d), Some(local)) = (side(edges[p]), local) {
                    self.pick(local, d);
                }
            });
        }
        let mask = pad::START | pad::B | if screen.picking() { pad::LEFT | pad::RIGHT } else { 0 };
        let own = if guest { 0 } else { edges[0] | edges[1..].iter().fold(0, |m, e| m | e & !mask) };
        let ctx = flow::Ctx {
            lobby: self.net.lobby_supported(),
            rooms: self.net.rooms().len(),
            link: self.net.link(),
            players: self.ports.iter().flatten().count(),
            remote: self.net.seats().iter().any(|s| s.local_port.is_none()),
            frame: self.frame,
            phases: self.maps.len(),
            unlocked: self.unlocked as usize,
        };
        let (next, action) = flow::step(screen, pad::key(own), &ctx);
        self.screen = next;
        match action {
            flow::Action::None => {}
            flow::Action::Solo | flow::Action::Couch => {
                self.ports = [Some(0), None, None, None];
                self.focus = 0;
                self.sync_ports();
            }
            flow::Action::Phase(p) => self.net.set_phase(p),
            flow::Action::PhaseStep(d) => {
                let p = (self.net.phase() as i16 + d as i16).rem_euclid(self.unlocked.max(1) as i16);
                self.net.set_phase(p as u8);
            }
            flow::Action::Character(d) => self.pick(0, d),
            flow::Action::Begin => self.begin(),
            flow::Action::Host { public, relay } => {
                self.sync_ports();
                self.net.host(public, flow::RELAYS[relay].1);
                self.net.set_phase(self.net.phase().min(self.unlocked.saturating_sub(1)));
            }
            flow::Action::Refresh => self.net.refresh_rooms(),
            flow::Action::Join(i) => {
                self.sync_ports();
                self.net.join(i);
            }
            flow::Action::Start => {
                self.net.start();
            }
            flow::Action::Ready => {
                let ready = !self.net.ready();
                self.net.set_ready(ready);
            }
            flow::Action::Leave => self.quit_to(self.screen),
        }
    }

    fn in_game(&mut self, fe: &Frontend, edges: &[u16; 4], input: &[u16; 4]) {
        let Some(g) = self.game.as_mut() else { return };
        let now = Instant::now();
        let key = pad::key(pad::merged(edges));
        let held = g.held_to_pause(input, now);
        let start = (0..4).find(|&p| edges[p] & pad::PAUSE != 0).and_then(|p| g.seat_of(p));
        match (g.over(), g.pause) {
            (true, _) if key == Some(pad::Key::Confirm) => return self.quit_to(Screen::main()),
            (true, _) => {}
            (false, None) => held.or(start).into_iter().for_each(|seat| g.open_pause(seat)),
            (false, Some(p)) => match (key, g.pause_choice()) {
                (Some(pad::Key::Confirm), Some(PauseAction::Quit)) => return self.quit_to(Screen::main()),
                (Some(pad::Key::Confirm), Some(PauseAction::Drop)) => g.drop_item(input),
                (Some(pad::Key::Confirm | pad::Key::Back), _) => g.resume(input),
                (k, _) => g.pause = Some(game::Pause { sel: pad::list(p.sel, g.pause_len(), k), ..p }),
            },
        }
        g.advance(fe, &mut self.net, &mut self.audio, input, now);
        let (won, seen) = (g.take_won(), g.take_seen());
        if let Some(won) = won {
            self.unlock(fe, won);
        }
        if let Some(tips) = seen.filter(|&t| t != self.tips) {
            self.tips = tips;
            self.persist(fe);
        }
    }

    fn focus_character(&self) -> u8 {
        let seats = self.net.seats();
        seats.get(self.focus).or_else(|| seats.iter().find(|s| s.local_port.is_some())).map_or(0, |s| s.character)
    }

    fn draw_menus(&mut self) {
        let t = self.frame;
        match self.screen {
            Screen::Title => ui::title(&mut self.ui, t, concat!("v", env!("CARGO_PKG_VERSION"))),
            Screen::Game => {}
            s => {
                let look = Look { net: &self.net, maps: self.maps, unlocked: self.unlocked as usize, focus: self.focus_character(), t };
                flow::draw(&mut self.ui, s, &look);
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

    fn menu_views(&self) -> ([View; VIEWS], usize) {
        let dim = if self.screen == Screen::Title { TITLE_FADE } else { MENU_FADE };
        let (portraits, n) = flow::portraits(self.screen, &self.net);
        let views = std::array::from_fn(|i| match i.checked_sub(1).filter(|&k| k < n) {
            Some(k) => View { rect: portraits[k].0, follow: 0, fade: 0, portrait: Some(portraits[k].1) },
            None => self.backdrop.view(dim),
        });
        (views, n + 1)
    }

    fn steer(&mut self, fe: &Frontend, input: [u16; 4]) -> [u16; 4] {
        let (screen, game, frame) = (self.screen, self.game.as_deref(), self.frame);
        let bot = self.autopilot.as_mut().map(|a| a.drive(screen, game, frame, &mut |l| fe.log(LOG_INFO, &l)));
        source(input, bot)
    }

    fn upcoming(&self) -> Option<&'static Map<'static>> {
        match (self.screen, self.game.as_deref()) {
            (Screen::Game, Some(g)) if g.over() => Some(self.backdrop.world().map()),
            (Screen::Game, Some(g)) => g.upcoming(),
            (s, _) if s.picking() => self.maps.get(self.net.phase() as usize).copied(),
            _ => self.backdrop.upcoming(self.unlocked as usize),
        }
    }

    pub fn frame(&mut self, fe: &Frontend, input: [u16; 4]) -> bool {
        let start = Instant::now();
        self.frame = self.frame.wrapping_add(1);
        self.net.poll(fe);
        let input = self.steer(fe, input);
        let edges = self.pad.edges(&input);
        self.watch_net(fe);
        match self.screen {
            Screen::Game => self.in_game(fe, &edges, &input),
            _ => self.menus(&edges),
        }
        self.watch_net(fe);
        let playing = self.game.as_ref().filter(|_| self.screen == Screen::Game);
        if playing.is_none() {
            self.backdrop.tick(start, self.unlocked as usize);
        }
        let (views, n) = match playing {
            Some(g) => g.views(),
            None => self.menu_views(),
        };
        let upcoming = self.upcoming();
        upcoming.zip(self.gfx.as_mut()).into_iter().for_each(|(m, (_, r))| r.prepare(m));
        if !self.gfx.as_mut().is_some_and(|(_, r)| r.due(n)) {
            self.report(fe, start.elapsed());
            return false;
        }
        self.ui.clear();
        self.draw_menus();
        let Some((gl, r)) = &mut self.gfx else { return false };
        let target = gl.target_framebuffer();
        match self.game.as_ref().filter(|_| self.screen == Screen::Game) {
            Some(g) => {
                g.draw(&mut self.ui, self.frame);
                r.frame(gl, target, WIDTH, HEIGHT, g.world(), &views[..n], &self.ui);
            }
            None => r.frame(gl, target, WIDTH, HEIGHT, self.backdrop.world(), &views[..n], &self.ui),
        }
        self.report(fe, start.elapsed());
        true
    }

    pub fn audio(&mut self, out: &mut [i16]) {
        self.audio.fill(out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::LazyLock;

    static SAMPLE: LazyLock<Map<'static>> = LazyLock::new(|| map::parse(include_str!("ui/sample.map")).unwrap_or_else(|e| panic!("{e}")));
    static MAPS: LazyLock<[&'static Map<'static>; 3]> = LazyLock::new(|| [&*SAMPLE, &*SAMPLE, &*SAMPLE]);

    pub fn sample_maps() -> &'static [&'static Map<'static>] {
        &*MAPS
    }

    #[test]
    fn direct_start_reads_phase_and_characters() {
        assert_eq!(direct(None, None, 5), None);
        assert_eq!(direct(Some("5"), None, 5), None);
        assert_eq!(direct(Some("2"), None, 5), Some(Direct { phase: 2, characters: [0, 1, 2, 3], players: 1 }));
        assert_eq!(direct(None, Some("3, 1"), 5), Some(Direct { phase: 0, characters: [3, 1, 2, 3], players: 2 }));
        assert_eq!(direct(Some("4"), Some("2,0,1,3,1"), 5), Some(Direct { phase: 4, characters: [2, 0, 1, 3], players: 4 }));
        assert_eq!(direct(Some("x"), Some("7,z"), 5), None);
    }

    #[test]
    fn sides_and_ports() {
        assert_eq!(side(pad::LEFT), Some(-1));
        assert_eq!(side(pad::RIGHT | pad::A), Some(1));
        assert_eq!(side(pad::LEFT | pad::RIGHT), None);
        assert_eq!(ports_list(&[None, Some(2), None, Some(1)]), ([2, 1, 0, 0], 2));
    }

    #[test]
    fn bot_replaces_only_the_first_port() {
        assert_eq!(source([1, 2, 3, 4], None), [1, 2, 3, 4]);
        assert_eq!(source([1, 2, 3, 4], Some(pad::A)), [pad::A, 2, 3, 4]);
    }
}
