use super::echo::Echo;
use super::tape::tape_id;
use super::{Agent, Body, Event, Phase, Role, World, MAX_BODIES, TICS_PER_SEC};
use crate::fx::{Fx, Vec2};

pub const FADE_TICS: u8 = 90;
pub const SWAP_AT: u8 = 45;
const TOUCH: i64 = 1 << 30;
pub const GRACE_TICS: u32 = 10 * TICS_PER_SEC;

pub fn grabs(victim: &Body, hunter: &Body) -> bool {
    let d = hunter.pos - victim.pos;
    d.len_sq() < TOUCH && (!victim.lantern || Vec2::from_ang(victim.heading).dot(d) < Fx::ZERO)
}

pub fn fade(t: u8) -> u8 {
    let t = t.min(FADE_TICS) as u32;
    let half = SWAP_AT as u32;
    (match t <= half {
        true => 255 * t / half,
        false => 255 * (FADE_TICS as u32 - t) / half,
    }) as u8
}

impl World {
    fn hostile(&self, i: usize) -> bool {
        self.bodies[i].visible
            && match self.ctl[i].agent {
                Agent::Echo => self.echo[i].hostile(),
                Agent::Player(q) => self.players[q as usize].role == Role::Duplo && self.capture[q as usize] == 0,
                Agent::Free => false,
            }
    }

    pub(super) fn catch(&mut self) {
        if self.phase_tic < GRACE_TICS {
            return;
        }
        for p in 0..self.players_n as usize {
            let me = self.players[p].body as usize;
            let victim = self.bodies[me];
            let by = (0..MAX_BODIES).find(|&j| j != me && self.hostile(j) && grabs(&victim, &self.bodies[j]));
            if let (true, Some(j)) = (self.living(p), by) {
                self.capture[p] = 1;
                self.captor[p] = j as u8;
                self.events.push(Event::Captured { player: p as u8 });
                self.events.push(Event::CaughtBy { player: p as u8, body: j as u8 });
            }
        }
    }

    pub(super) fn fades(&mut self) {
        for p in 0..self.players_n as usize {
            let t = match self.capture[p] {
                0 => continue,
                t => t + 1,
            };
            if t == SWAP_AT {
                self.turn(p);
            }
            self.players[p].fade = fade(t);
            self.capture[p] = if t >= FADE_TICS { 0 } else { t };
        }
        self.check_lost();
    }

    pub(super) fn settle_captures(&mut self) {
        for p in 0..self.players_n as usize {
            if self.capture[p] == 0 {
                continue;
            }
            if self.capture[p] < SWAP_AT {
                self.turn(p);
            }
            self.capture[p] = 0;
            self.players[p].fade = 0;
        }
        self.check_lost();
    }

    fn check_lost(&mut self) {
        let alive = (0..self.players_n as usize).any(|p| self.players[p].role == Role::Living);
        if !alive {
            self.phase = Phase::Lost;
        }
    }

    pub(super) fn night(&self) -> u8 {
        match self.phase {
            Phase::Dusk { night } | Phase::Night { night } | Phase::Dawn { night } => night,
            Phase::Won | Phase::Lost => 5,
        }
    }

    fn turn(&mut self, p: usize) {
        let night = self.night();
        let old = self.players[p].body as usize;
        let free = (0..MAX_BODIES).find(|&j| self.ctl[j].agent == Agent::Free);
        let far_echo = (0..MAX_BODIES)
            .filter(|&j| self.ctl[j].agent == Agent::Echo && self.bodies[j].visible)
            .max_by_key(|&j| (self.distance_at(self.bodies[j].pos), j));
        let Some(slot) = free.or(far_echo) else { return };
        if free.is_some() {
            let spot = self
                .map
                .spawns
                .iter()
                .max_by_key(|s| self.distance_at(s.pos))
                .map_or((self.bodies[old].pos, self.bodies[old].heading), |s| (s.pos, s.rot));
            self.place(slot, spot, Agent::Player(p as u8));
        }
        self.ctl[slot].agent = Agent::Player(p as u8);
        self.echo[slot] = Echo::NONE;
        self.ctl[old].agent = Agent::Echo;
        self.ctl[old].hold = 0;
        self.echo[old] = Echo::new(tape_id(night.max(2) - 1, p), night, false);
        self.players[p].body = slot as u8;
        self.players[p].role = Role::Duplo;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::Ang;

    fn at(x: i32, y: i32, heading: Ang, lantern: bool) -> Body {
        Body { pos: Vec2::new(Fx::int(x), Fx::int(y)), heading, lantern, ..super::super::HIDDEN }
    }

    #[test]
    fn touch_rule() {
        let copy = at(5, 5, Ang::ZERO, false);
        let near = |dx: Fx, dy: Fx, h: Ang, lit: bool| {
            let mut v = at(5, 5, h, lit);
            v.pos = v.pos - Vec2::new(dx, dy);
            grabs(&v, &copy)
        };
        let close = Fx::ratio(4, 10);
        assert!(near(Fx::ZERO, close, Ang::ZERO, false));
        assert!(!near(Fx::ZERO, close, Ang::ZERO, true));
        assert!(near(Fx::ZERO, close, Ang::HALF, true));
        assert!(!near(Fx::ZERO, Fx::ratio(6, 10), Ang::HALF, false));
        assert!(!near(close, Fx::ZERO, Ang::QUARTER, true));
        assert_eq!((fade(0), fade(SWAP_AT), fade(FADE_TICS)), (0, 255, 0));
    }
}
