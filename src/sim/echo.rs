use super::capture::GRACE_TICS;
use super::scare::LURE_PERCENT;
use super::tank::{mirrored, A, LEFT, RIGHT};
use super::tape::SEG;
use super::{Event, World, MAX_PLAYERS};

pub const IDLE_TICS: u8 = 90;
const GLITCH_PERCENT: u32 = 30;
const STARE_SLACK: i16 = 1200;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Play,
    Splice(u8),
    Reverse(u32),
    Stare,
    Idle(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Echo {
    pub tape: u8,
    pub cursor: u32,
    pub left: u8,
    pub mode: Mode,
    pub hunt: bool,
    pub glitchy: bool,
    pub wait: u16,
    pub look: i16,
    pub watched: bool,
    pub chase: u16,
    pub lure: u8,
    pub armed: bool,
    pub shadow: u8,
    pub due: u8,
    pub of: u8,
}

impl Echo {
    pub const NONE: Echo = Echo {
        tape: 0,
        cursor: 0,
        left: 0,
        mode: Mode::Play,
        hunt: false,
        glitchy: false,
        wait: 0,
        look: 0,
        watched: false,
        chase: 0,
        lure: 0,
        armed: false,
        shadow: 0,
        due: 0,
        of: 0,
    };

    pub fn new(tape: u8, night: u8, extra: bool) -> Echo {
        let hunt = extra || night >= 4;
        Echo { tape, hunt, glitchy: night == 3 && !hunt, ..Echo::NONE }
    }

    pub fn hostile(&self) -> bool {
        self.hunt || (self.glitchy && matches!(self.mode, Mode::Splice(_) | Mode::Reverse(_) | Mode::Stare))
    }
}

pub fn stare(diff: i16) -> u16 {
    match diff {
        d if d > STARE_SLACK => LEFT,
        d if d < -STARE_SLACK => RIGHT,
        _ => 0,
    }
}

impl World {
    pub(super) fn echo_input(&mut self, i: usize) -> u16 {
        if let Mode::Idle(n) = self.echo[i].mode {
            self.echo[i] = match n {
                0 => Echo { mode: Mode::Play, cursor: 0, left: 0, ..self.echo[i] },
                _ => Echo { mode: Mode::Idle(n - 1), ..self.echo[i] },
            };
            return 0;
        }
        if self.echo[i].left == 0 {
            self.next_segment(i);
        }
        self.echo[i].left -= 1;
        let e = self.echo[i];
        let len = self.night_tics;
        let (input, next) = match e.mode {
            Mode::Play | Mode::Splice(_) if e.cursor >= len => (0, Echo { mode: Mode::Idle(IDLE_TICS), ..e }),
            Mode::Play => (self.tape_at(e.tape, e.cursor), Echo { cursor: e.cursor + 1, ..e }),
            Mode::Splice(t) => (self.tape_at(t, e.cursor), Echo { cursor: e.cursor + 1, ..e }),
            Mode::Reverse(at) => {
                let at = at.saturating_sub(1);
                (mirrored(self.tape_at(e.tape, at)), Echo { mode: Mode::Reverse(at), ..e })
            }
            Mode::Stare => (self.stare_input(i), e),
            Mode::Idle(_) => (0, e),
        };
        self.echo[i] = next;
        match self.hunting(e) {
            true => input | self.knock(i),
            false => input,
        }
    }

    fn hunting(&self, e: Echo) -> bool {
        e.hunt && self.phase_tic >= GRACE_TICS
    }

    fn knock(&self, i: usize) -> u16 {
        let shut = |d: usize| !self.door_dir[d] && !self.doors[d].locked;
        match self.door_ahead(i).is_some_and(shut) && self.ctl[i].tank.prev & A == 0 {
            true => A,
            false => 0,
        }
    }

    fn stare_input(&self, i: usize) -> u16 {
        let b = self.bodies[i];
        self.living_bodies()
            .map(|l| l.pos - b.pos)
            .min_by_key(|d| d.len_sq())
            .map_or(0, |d| stare((d.ang() - b.heading).0 as i16))
    }

    fn next_segment(&mut self, i: usize) {
        let e = self.echo[i];
        let cursor = match e.mode {
            Mode::Reverse(at) => at,
            _ => e.cursor,
        };
        let (mode, cursor) = match (self.hunting(e), e.glitchy) {
            (true, _) => (Mode::Play, self.pick_segment(i) * SEG),
            (false, true) if self.rng.below(100) < GLITCH_PERCENT => {
                self.events.push(Event::Glitch { body: i as u8 });
                (self.glitch(e.tape, cursor), cursor)
            }
            _ => (Mode::Play, cursor),
        };
        let armed = match e.armed || self.night() < 3 {
            true => e.armed,
            false => self.rng.below(100) < LURE_PERCENT,
        };
        self.echo[i] = Echo { mode, cursor, armed, left: SEG as u8, ..e };
    }

    fn glitch(&mut self, tape: u8, cursor: u32) -> Mode {
        let n = self.players_n as u32;
        match (self.rng.below(3), n) {
            (0, 2..) => {
                let p = tape as u32 % MAX_PLAYERS as u32;
                let other = (p + 1 + self.rng.below(n - 1)) % n;
                Mode::Splice((tape as u32 - p + other) as u8)
            }
            (0 | 1, _) => Mode::Reverse(cursor),
            _ => Mode::Stare,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stare_turns_toward_target() {
        assert_eq!(stare(8000), LEFT);
        assert_eq!(stare(-8000), RIGHT);
        assert_eq!(stare(300), 0);
        assert!(!Echo::new(0, 2, false).hostile());
        assert!(Echo::new(0, 2, true).hostile());
        assert!(Echo::new(0, 4, false).hostile());
        let night3 = Echo::new(0, 3, false);
        assert!(!night3.hostile() && Echo { mode: Mode::Stare, ..night3 }.hostile());
    }
}
