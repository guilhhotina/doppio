use super::flow::Screen;
use super::game::Game;
use super::pad::{A, B, DOWN, UP};
use crate::sim::{Bot, Event, Outcome, TICS_PER_SEC, World};

const MENU_BEAT: u32 = 90;
const IDLE_TICS: u32 = 60 * TICS_PER_SEC;

pub struct Autopilot {
    bot: Option<Bot>,
    phase: u8,
    watched: u32,
    asked: Option<u32>,
    bits: u16,
    left: u8,
    outcome: Outcome,
    spot: ((i32, i32), u32, bool),
}

fn menu_key(screen: Screen) -> u16 {
    match screen {
        Screen::Title | Screen::Main { sel: 0 } | Screen::Campaign { sel: 1, .. } | Screen::Party { sel: 1, .. } | Screen::Notice(_) => A,
        Screen::Campaign { sel: 0, .. } | Screen::Party { sel: 0, .. } => DOWN,
        Screen::Main { .. } | Screen::Campaign { .. } | Screen::Party { .. } => UP,
        Screen::Game => 0,
        _ => B,
    }
}

fn pulse(frame: u32, bits: u16) -> u16 {
    if frame % MENU_BEAT == 0 { bits } else { 0 }
}

fn beat(e: &Event) -> Option<&'static str> {
    match e {
        Event::Pickup { .. } => Some("item"),
        Event::LegendSaw { .. } => Some("sighting"),
        Event::Hide { .. } => Some("hide"),
        Event::Unlock { .. } => Some("unlock"),
        _ => None,
    }
}

fn tile(w: &World, seat: usize) -> (i32, i32) {
    w.players().get(seat).map_or((0, 0), |p| (p.pos.x.floor(), p.pos.y.floor()))
}

impl Autopilot {
    pub fn new() -> Autopilot {
        Autopilot { bot: None, phase: 0, watched: 0, asked: None, bits: 0, left: 0, outcome: Outcome::Playing, spot: ((0, 0), 0, false) }
    }

    pub fn drive(&mut self, screen: Screen, game: Option<&Game>, frame: u32, note: &mut impl FnMut(String)) -> u16 {
        match game.filter(|_| screen == Screen::Game) {
            None => pulse(frame, menu_key(screen)),
            Some(g) if g.pause.is_some() => pulse(frame, B),
            Some(g) => self.steer(g.world(), g.playing(), g.seat_of(0).unwrap_or(0), note),
        }
    }

    fn steer(&mut self, w: &World, playing: bool, seat: usize, note: &mut impl FnMut(String)) -> u16 {
        let phase = w.setup().phase;
        if self.bot.is_none() || w.tic() < self.watched || phase != self.phase {
            *self = Autopilot {
                bot: Some(Bot::new(w, w.setup().seed)),
                phase,
                watched: w.tic(),
                left: w.desmaios_left(),
                spot: (tile(w, seat), w.tic(), false),
                ..Autopilot::new()
            };
            note(format!("bot world phase {phase} seed {}", w.setup().seed));
        }
        if w.tic() != self.watched {
            self.watch(w, seat, note);
        }
        match (playing, self.asked, self.bot.as_mut()) {
            (true, asked, Some(bot)) if asked != Some(w.tic()) => {
                if asked.is_none() {
                    note(format!("bot beat wake phase {phase} t{}", w.tic()));
                }
                self.bits = bot.inputs(w)[seat];
                self.asked = Some(w.tic());
                bot.drain().for_each(|l| note(format!("bot: {l}")));
                self.bits
            }
            (true, _, _) => self.bits,
            (false, _, _) => 0,
        }
    }

    fn watch(&mut self, w: &World, seat: usize, note: &mut impl FnMut(String)) {
        let (phase, tic) = (self.phase, w.tic());
        self.watched = tic;
        w.events().iter().filter_map(beat).for_each(|b| note(format!("bot beat {b} phase {phase} t{tic}")));
        if w.desmaios_left() < self.left {
            note(format!("bot faint phase {phase} t{tic} left {}", w.desmaios_left()));
        }
        self.left = w.desmaios_left();
        if w.outcome() != self.outcome {
            note(format!("bot outcome {:?} phase {phase} t{tic} secs {}", w.outcome(), tic / TICS_PER_SEC));
            self.outcome = w.outcome();
        }
        let (at, since, told) = self.spot;
        self.spot = match tile(w, seat) {
            t if t != at => (t, tic, false),
            _ if !told && tic - since >= IDLE_TICS => {
                let state = w.players().get(seat).map(|p| p.state);
                note(format!("bot idle 60s phase {phase} t{tic} at {at:?} {state:?}"));
                (at, since, true)
            }
            _ => self.spot,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::{self, Action, Ctx};
    use crate::app::pad;
    use crate::map;
    use crate::net::Link;
    use crate::sim::Setup;

    fn ctx(unlocked: usize) -> Ctx {
        Ctx { lobby: true, rooms: 0, link: Link::Offline, players: 1, remote: false, frame: 0, phases: 5, unlocked }
    }

    fn menu_run(from: Screen, unlocked: usize) -> Vec<Action> {
        let c = ctx(unlocked);
        let mut s = from;
        let mut acts = vec![];
        while acts.last() != Some(&Action::Begin) && acts.len() < 40 {
            let (next, a) = flow::step(s, pad::key(menu_key(s)), &c);
            s = next;
            acts.push(a);
        }
        acts
    }

    #[test]
    fn menus_start_the_first_phase_solo() {
        [Screen::Title, Screen::Credits { page: 1 }, Screen::Main { sel: 3 }, Screen::Notice(flow::Notice::Lost)]
            .into_iter()
            .flat_map(|s| [(s, 1), (s, 5)])
            .for_each(|(s, unlocked)| {
                let acts = menu_run(s, unlocked);
                let phase = acts.iter().rev().find_map(|a| if let Action::Phase(p) = a { Some(*p) } else { None });
                assert_eq!(acts.last(), Some(&Action::Begin), "{s:?} unlocked {unlocked}: {acts:?}");
                assert_eq!(phase, Some(0), "{s:?} unlocked {unlocked}: {acts:?}");
            });
        assert_eq!(pulse(1, A), 0);
        assert_eq!(pulse(MENU_BEAT, A), A);
    }

    #[test]
    fn pilot_ticks_once_per_world_tic() {
        let m = map::campaign()[0];
        let setup = Setup { seed: 11, phase: 0, players: 1, characters: [0, 1, 2, 3] };
        let (mut a, mut b) = (World::new(m, setup), World::new(m, setup));
        let mut bot = Bot::new(&b, setup.seed);
        let mut auto = Autopilot::new();
        let mut notes = vec![];
        (0..1500).for_each(|_| {
            let bits = [0, 1].map(|_| auto.steer(&a, true, 0, &mut |l| notes.push(l)));
            assert_eq!(bits[0], bits[1]);
            a.step(&[bits[0], 0, 0, 0]);
            b.step(&bot.inputs(&b));
        });
        assert_eq!(a.hash(), b.hash());
        assert_ne!(tile(&a, 0), tile(&World::new(m, setup), 0));
        assert!(notes.iter().any(|n| n.starts_with("bot beat wake")));
        assert_eq!(auto.steer(&a, false, 0, &mut |_| {}), 0);
    }
}
