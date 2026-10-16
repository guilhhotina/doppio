use super::clock::Clock;
use super::layout;
use super::pad::{A, PLAY};
use crate::audio::Audio;
use crate::core::{Frontend, SAMPLE_RATE};
use crate::ffi::LOG_INFO;
use crate::map::Map;
use crate::net::{Net, Seat};
use crate::render::{CANVAS_H, CANVAS_W, Ui2d, View};
use crate::sim::{DROP, MAX_PLAYERS, Outcome, PlayerState, Setup, TICS_PER_SEC, World};
use crate::ui::{self, Card, Item};
use std::time::{Duration, Instant};

pub const INTRO_TICS: u32 = 5 * TICS_PER_SEC;
pub const OUTRO_TICS: u32 = 5 * TICS_PER_SEC;
pub const RESTART_TICS: u32 = 4 * TICS_PER_SEC;
const PAUSE_FADE: u8 = 170;
const CARD_FADE: u8 = 190;
const PAUSE_HOLD: Duration = Duration::from_secs(1);
pub const VIEWS: usize = MAX_PLAYERS + 1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Intro { until: u32 },
    Play,
    Outro { until: u32 },
    Restart { until: u32 },
    Ending,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Change {
    None,
    Load(u8),
    Won(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pause {
    pub sel: usize,
    pub seat: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PauseAction {
    Resume,
    Drop,
    Quit,
}

pub fn next(stage: Stage, outcome: Outcome, tic: u32, phase: u8, phases: u8) -> (Stage, Change) {
    match stage {
        Stage::Intro { until } if tic >= until => (Stage::Play, Change::None),
        Stage::Play => match outcome {
            Outcome::Escaped => (Stage::Outro { until: tic + OUTRO_TICS }, Change::Won(phase)),
            Outcome::Restart => (Stage::Restart { until: tic + RESTART_TICS }, Change::None),
            Outcome::Playing => (stage, Change::None),
        },
        Stage::Outro { until } if tic >= until && phase + 1 < phases => (Stage::Intro { until: tic + INTRO_TICS }, Change::Load(phase + 1)),
        Stage::Outro { until } if tic >= until => (Stage::Ending, Change::None),
        Stage::Restart { until } if tic >= until => (Stage::Intro { until: tic + INTRO_TICS }, Change::Load(phase)),
        _ => (stage, Change::None),
    }
}

pub fn pause_items(holding: bool) -> ([PauseAction; 3], usize) {
    match holding {
        true => ([PauseAction::Resume, PauseAction::Drop, PauseAction::Quit], 3),
        false => ([PauseAction::Resume, PauseAction::Quit, PauseAction::Quit], 2),
    }
}

fn pause_label(a: PauseAction) -> &'static str {
    match a {
        PauseAction::Resume => "Continuar",
        PauseAction::Drop => "Largar item",
        PauseAction::Quit => "Sair para o menu",
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stepped {
    Idle,
    Played,
    Fresh,
}

struct Run {
    maps: &'static [&'static Map<'static>],
    world: Box<World>,
    setup: Setup,
    stage: Stage,
    tic: u32,
    won: Option<u8>,
}

impl Run {
    fn step(&mut self, row: &[u16; MAX_PLAYERS]) -> Stepped {
        let played = self.stage == Stage::Play;
        if played {
            self.world.step(row);
        }
        let (stage, change) = next(self.stage, self.world.outcome(), self.tic, self.setup.phase, self.maps.len() as u8);
        self.stage = stage;
        self.tic += 1;
        match change {
            Change::Load(p) => {
                self.setup.phase = p;
                self.world = Box::new(World::new(self.maps[p as usize], self.setup));
                Stepped::Fresh
            }
            Change::Won(p) => {
                self.won = Some(p);
                Stepped::Played
            }
            Change::None if played => Stepped::Played,
            Change::None => Stepped::Idle,
        }
    }
}

pub struct Game {
    run: Run,
    clock: Clock,
    seats: [Option<u8>; MAX_PLAYERS],
    online: bool,
    hashlog: u32,
    hold: [Option<Instant>; MAX_PLAYERS],
    quiet: [u16; 4],
    drop: [bool; 4],
    pub pause: Option<Pause>,
}

pub fn local_seats(seats: &[Seat]) -> [Option<u8>; MAX_PLAYERS] {
    std::array::from_fn(|i| seats.get(i).and_then(|s| s.local_port))
}

pub fn stage_named(name: &str) -> Option<Stage> {
    match name.trim() {
        "play" => Some(Stage::Play),
        "outro" => Some(Stage::Outro { until: OUTRO_TICS }),
        "restart" => Some(Stage::Restart { until: RESTART_TICS }),
        "ending" => Some(Stage::Ending),
        _ => None,
    }
}

impl Game {
    pub fn new(maps: &'static [&'static Map<'static>], setup: Setup, seats: &[Seat], online: bool, hashlog: u32, now: Instant) -> Game {
        let setup = Setup { phase: setup.phase.min(maps.len().saturating_sub(1) as u8), ..setup };
        Game {
            run: Run {
                maps,
                world: Box::new(World::new(maps[setup.phase as usize], setup)),
                setup,
                stage: Stage::Intro { until: INTRO_TICS },
                tic: 0,
                won: None,
            },
            clock: Clock::new(now),
            seats: local_seats(seats),
            online,
            hashlog,
            hold: [None; MAX_PLAYERS],
            quiet: [0; 4],
            drop: [false; 4],
            pause: None,
        }
    }

    pub fn online(&self) -> bool {
        self.online
    }

    pub fn force(&mut self, stage: Stage) {
        self.run.stage = stage;
    }

    pub fn take_won(&mut self) -> Option<u8> {
        self.run.won.take()
    }

    fn local(&self) -> impl Iterator<Item = usize> + '_ {
        (0..MAX_PLAYERS).filter(|&s| self.seats[s].is_some())
    }

    pub fn over(&self) -> bool {
        self.run.stage == Stage::Ending
    }

    pub fn holding(&self, seat: usize) -> bool {
        self.run.world.players().get(seat).is_some_and(|p| p.hands.iter().any(Option::is_some))
    }

    pub fn seat_of(&self, port: usize) -> Option<usize> {
        self.seats.iter().position(|&p| p == Some(port as u8))
    }

    fn idle_hands(&self, seat: usize) -> bool {
        let w = &self.run.world;
        w.focus(seat).is_none()
            && w.dial(seat).is_none()
            && w.reading(seat).is_none()
            && w.players().get(seat).is_some_and(|p| p.state == PlayerState::Free)
    }

    pub fn held_to_pause(&mut self, input: &[u16; 4], now: Instant) -> Option<usize> {
        self.quiet = std::array::from_fn(|p| self.quiet[p] & input[p]);
        let free = |s: usize| self.seats[s].is_some_and(|p| input[p as usize] & !self.quiet[p as usize] & A != 0) && self.idle_hands(s);
        let hold: [Option<Instant>; MAX_PLAYERS] = std::array::from_fn(|s| free(s).then(|| self.hold[s].unwrap_or(now)));
        self.hold = hold;
        hold.iter().position(|h| h.is_some_and(|since| now.duration_since(since) >= PAUSE_HOLD))
    }

    pub fn open_pause(&mut self, seat: usize) {
        self.pause = Some(Pause { sel: 0, seat });
        self.hold = [None; MAX_PLAYERS];
    }

    pub fn pause_choice(&self) -> Option<PauseAction> {
        let p = self.pause?;
        let (items, n) = pause_items(self.holding(p.seat));
        Some(items[p.sel.min(n - 1)])
    }

    pub fn pause_len(&self) -> usize {
        self.pause.map_or(0, |p| pause_items(self.holding(p.seat)).1)
    }

    pub fn drop_item(&mut self, input: &[u16; 4]) {
        let port = self.pause.and_then(|p| self.seats[p.seat]);
        port.iter().for_each(|&p| self.drop[p as usize] = true);
        self.resume(input);
    }

    pub fn resume(&mut self, input: &[u16; 4]) {
        self.pause = None;
        self.quiet = input.map(|b| b & A);
    }

    pub fn advance(&mut self, fe: &Frontend, net: &mut Net, audio: &mut Audio, input: &[u16; 4], now: Instant) {
        if self.pause.is_some() && !self.online {
            self.clock.hold(now);
            return;
        }
        let frozen = self.pause.is_some() || self.run.stage != Stage::Play;
        let listener = self.local().next().unwrap_or(0) as u8;
        let (run, hashlog, drop, quiet) = (&mut self.run, self.hashlog, &mut self.drop, self.quiet);
        self.clock.run(now, || {
            let local: [u16; 4] = std::array::from_fn(|p| match frozen {
                true => 0,
                false => input[p] & PLAY & !quiet[p] | if drop[p] { DROP } else { 0 },
            });
            net.push_local(&local);
            if !frozen {
                *drop = [false; 4];
            }
            net.pop_tic()
                .map(|row| {
                    let tic = run.tic;
                    match run.step(&row) {
                        Stepped::Played => audio.events(&run.world, run.world.events(), listener),
                        Stepped::Fresh => *audio = Audio::new(SAMPLE_RATE),
                        Stepped::Idle => {}
                    }
                    let h = run.world.hash();
                    net.report_hash(tic, h);
                    if hashlog > 0 && tic % hashlog == 0 {
                        fe.log(LOG_INFO, &format!("hash tic={tic} {h:08x}"));
                    }
                })
                .is_some()
        });
        audio.ambience(&self.run.world, listener);
    }

    fn dim(&self) -> u8 {
        match (self.pause.is_some(), self.run.stage) {
            (true, _) => PAUSE_FADE,
            (false, Stage::Play) => 0,
            (false, _) => CARD_FADE,
        }
    }

    pub fn views(&self) -> ([View; VIEWS], usize) {
        let n = self.local().count().max(1);
        let rects = layout::rects(n);
        let dim = self.dim();
        let mut seats = self.local();
        let mut views: [View; VIEWS] = std::array::from_fn(|i| {
            let seat = seats.next().filter(|_| i < n).unwrap_or(0);
            let fade = self.run.world.players().get(seat).map_or(0, |p| p.fade);
            View { rect: if i < n { rects[i] } else { [0; 4] }, follow: seat as u8, fade: fade.max(dim), portrait: None }
        });
        let intro = matches!(self.run.stage, Stage::Intro { .. }) && self.pause.is_none();
        if intro {
            views[n] = View { rect: ui::INTRO_PORTRAIT, follow: 0, fade: 0, portrait: Some(4 + self.run.world.map().legend as u8) };
        }
        (views, n + intro as usize)
    }

    pub fn draw(&self, ui: &mut Ui2d, frame: u32) {
        let n = self.local().count().max(1);
        let rects = layout::rects(n);
        let world = &self.run.world;
        if self.pause.is_none() && self.run.stage == Stage::Play {
            self.local().zip(rects.iter()).for_each(|(seat, &r)| ui::hud(ui, world, r, seat, frame));
        }
        if n >= 2 {
            ui.rect(CANVAS_W as i16 / 2 - 1, 0, 2, CANVAS_H as i16, 0);
        }
        if n >= 3 {
            ui.rect(0, CANVAS_H as i16 / 2 - 1, CANVAS_W as i16, 2, 0);
        }
        let phases = self.run.maps.len() as u8;
        let card = match (self.pause, self.run.stage) {
            (Some(p), _) => {
                let (acts, k) = pause_items(self.holding(p.seat));
                let items: [Item; 3] = std::array::from_fn(|i| Item::new(pause_label(acts[i])));
                ui::pause(ui, &items[..k], p.sel, self.online, frame);
                None
            }
            (None, Stage::Intro { .. }) => Some(Card::Intro),
            (None, Stage::Outro { .. }) => Some(Card::Outro),
            (None, Stage::Restart { .. }) => Some(Card::Restart),
            (None, Stage::Ending) => Some(Card::Ending),
            (None, Stage::Play) => None,
        };
        card.iter().for_each(|&c| ui::card(ui, c, world.map(), self.run.setup.phase, phases));
    }

    pub fn world(&self) -> &World {
        &self.run.world
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::sample_maps;

    fn seat(p: Option<u8>) -> Seat {
        Seat { nick: [0; 16], local_port: p, ready: true, character: 0 }
    }

    fn setup(players: u8) -> Setup {
        Setup { seed: 3, phase: 0, players, characters: [0, 1, 2, 3] }
    }

    #[test]
    fn stages_follow_outcomes() {
        let s = Stage::Intro { until: 10 };
        assert_eq!(next(s, Outcome::Playing, 9, 0, 5), (s, Change::None));
        assert_eq!(next(s, Outcome::Playing, 10, 0, 5), (Stage::Play, Change::None));
        assert_eq!(next(Stage::Play, Outcome::Playing, 50, 0, 5), (Stage::Play, Change::None));
        assert_eq!(next(Stage::Play, Outcome::Escaped, 50, 1, 5), (Stage::Outro { until: 50 + OUTRO_TICS }, Change::Won(1)));
        assert_eq!(next(Stage::Play, Outcome::Restart, 50, 1, 5), (Stage::Restart { until: 50 + RESTART_TICS }, Change::None));
        let o = Stage::Outro { until: 80 };
        assert_eq!(next(o, Outcome::Escaped, 79, 1, 5), (o, Change::None));
        assert_eq!(next(o, Outcome::Escaped, 80, 1, 5), (Stage::Intro { until: 80 + INTRO_TICS }, Change::Load(2)));
        assert_eq!(next(o, Outcome::Escaped, 80, 4, 5), (Stage::Ending, Change::None));
        let r = Stage::Restart { until: 70 };
        assert_eq!(next(r, Outcome::Restart, 70, 3, 5), (Stage::Intro { until: 70 + INTRO_TICS }, Change::Load(3)));
        assert_eq!(next(Stage::Ending, Outcome::Escaped, 999, 4, 5), (Stage::Ending, Change::None));
    }

    #[test]
    fn pause_offers_drop_only_when_holding() {
        assert_eq!(pause_items(false).1, 2);
        assert_eq!(pause_items(false).0[1], PauseAction::Quit);
        assert_eq!(pause_items(true).0[..3], [PauseAction::Resume, PauseAction::Drop, PauseAction::Quit]);
        assert_eq!(stage_named(" ending"), Some(Stage::Ending));
        assert_eq!(stage_named("outro"), Some(Stage::Outro { until: OUTRO_TICS }));
        assert_eq!(stage_named("nada"), None);
    }

    #[test]
    fn local_seats_follow_ports() {
        let seats = [seat(None), seat(Some(0)), seat(Some(2))];
        assert_eq!(local_seats(&seats), [None, Some(0), Some(2), None]);
        let g = Game::new(sample_maps(), setup(3), &seats, true, 0, Instant::now());
        let (views, n) = g.views();
        assert_eq!(n, 3);
        assert_eq!((views[0].follow, views[1].follow), (1, 2));
        assert_eq!(views[2].portrait, Some(4));
        assert!(views[0].fade >= CARD_FADE);
        assert_eq!(g.seat_of(2), Some(2));
    }

    #[test]
    fn restart_and_intro_run_on_lockstep_tics() {
        let mut run = Run {
            maps: sample_maps(),
            world: Box::new(World::new(sample_maps()[0], setup(1))),
            setup: setup(1),
            stage: Stage::Intro { until: 3 },
            tic: 0,
            won: None,
        };
        let row = [1 << 4, 0, 0, 0];
        assert_eq!((0..4).map(|_| run.step(&row)).collect::<Vec<_>>(), [Stepped::Idle; 4]);
        assert_eq!(run.stage, Stage::Play);
        assert_eq!(run.world.tic(), 0);
        assert_eq!(run.step(&row), Stepped::Played);
        assert_eq!(run.world.tic(), 1);
        run.stage = Stage::Restart { until: 5 };
        assert_eq!(run.step(&row), Stepped::Fresh);
        assert_eq!((run.world.tic(), run.stage), (0, Stage::Intro { until: 5 + INTRO_TICS }));
    }

    #[test]
    fn holding_a_with_nothing_in_reach_pauses_after_a_second() {
        let seats = [seat(Some(2))];
        let t0 = Instant::now();
        let at = |ms: u64| t0 + Duration::from_millis(ms);
        let mut g = Game::new(sample_maps(), Setup { characters: [3, 0, 1, 2], ..setup(1) }, &seats, false, 0, t0);
        assert_eq!(g.world().focus(0), None);
        let a = [0, 0, A, 0];
        assert_eq!(g.held_to_pause(&[A, 0, 0, 0], t0), None);
        assert_eq!(g.held_to_pause(&a, at(100)), None);
        assert_eq!(g.held_to_pause(&a, at(1000)), None);
        assert_eq!(g.held_to_pause(&[0; 4], at(1050)), None);
        assert_eq!(g.held_to_pause(&a, at(1100)), None);
        assert_eq!(g.held_to_pause(&a, at(2100)), Some(0));
        g.open_pause(0);
        assert_eq!(g.pause_choice(), Some(PauseAction::Resume));
        g.resume(&a);
        assert!(g.held_to_pause(&a, at(2200)).is_none() && g.held_to_pause(&a, at(4000)).is_none());
        assert_eq!(g.held_to_pause(&[0; 4], at(4100)), None);
        assert_eq!(g.held_to_pause(&a, at(4200)), None);
        assert_eq!(g.held_to_pause(&a, at(5200)), Some(0));
    }
}
