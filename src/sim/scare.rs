use super::capture::GRACE_TICS;
use super::sight::signed;
use super::tank::{A, MOVE, X};
use super::{Agent, Event, World, MAX_BODIES, MAX_PLAYERS, TICS_PER_SEC};
use crate::fx::{Ang, Fx, Vec2};
use crate::map::Tile;

const fn metres_sq(num: i64, den: i64) -> i64 {
    let r = (num << 16) / den;
    r * r
}

const STAGE_LATE: u32 = 20 * TICS_PER_SEC;
const WATCH_FAR: i64 = metres_sq(3, 1);
const HEAD_SLOW: i32 = 160;
const HEAD_BACK: i32 = 1800;
const SHADOW_RANGE: i64 = metres_sq(6, 1);
pub const ECHO_DELAY: u8 = 6;
const CHASE_RANGE: i64 = metres_sq(7, 1);
const NOTICE_CLOSE: i64 = metres_sq(2, 1);
pub const CHASE_TICS: u16 = 6 * TICS_PER_SEC as u16;
pub const LURE_TICS: u8 = 150;
pub const LURE_PERCENT: u32 = 35;
const LURE_REACH: Fx = Fx::ratio(8, 10);

pub fn stage_tics(night: u8) -> u16 {
    let secs = match night {
        2 => 20,
        3 => 12,
        4 => 10,
        _ => 0,
    };
    secs * TICS_PER_SEC as u16
}

pub fn head_step(look: i16, target: i32, watched: bool) -> i16 {
    let rate = if watched { HEAD_SLOW } else { HEAD_BACK };
    (look as i32 + (target - look as i32).clamp(-rate, rate)) as i16
}

impl World {
    pub(super) fn stage(&mut self, i: usize) -> bool {
        if self.bodies[i].visible {
            return false;
        }
        if self.echo[i].wait > 0 {
            self.echo[i].wait -= 1;
            return true;
        }
        let late = self.phase_tic >= stage_tics(self.night()) as u32 + STAGE_LATE;
        let seen = self.watcher(self.bodies[i].pos, -1).is_some();
        self.bodies[i].visible = late || !seen;
        !self.bodies[i].visible
    }

    pub(super) fn copy_input(&mut self, i: usize) -> u16 {
        let night = self.night();
        self.notice(i);
        let pos = self.bodies[i].pos;
        let hold = self.ctl[i].tank.prev & !MOVE;
        let watcher = match night >= 3 && self.echo[i].chase == 0 {
            true => self.watcher(pos, WATCH_FAR),
            false => None,
        };
        let target = watcher.map_or(0, |v| signed((v.eye - pos).ang() - self.bodies[i].heading));
        self.echo[i].look = head_step(self.echo[i].look, target, watcher.is_some());
        self.echo[i].watched = watcher.is_some();
        if watcher.is_some() {
            return hold;
        }
        if self.echo[i].lure > 0 {
            self.echo[i].lure -= 1;
            return self.lure_input(i, hold);
        }
        let input = self.echo_input(i);
        match self.echo[i].armed && self.lures(i) {
            true => {
                self.echo[i].armed = false;
                self.echo[i].lure = LURE_TICS;
                self.lure_input(i, hold)
            }
            false => input,
        }
    }

    fn lure_input(&self, i: usize, hold: u16) -> u16 {
        let quiet = hold & !(X | A);
        match self.bodies[i].lantern || hold & X != 0 {
            true => quiet,
            false => quiet | X,
        }
    }

    fn lures(&self, i: usize) -> bool {
        let b = self.bodies[i];
        let ahead = b.pos + Vec2::from_ang(b.heading).scale(LURE_REACH);
        let dark = !self.power || self.tile_index(b.pos).and_then(|t| self.map.room_of[t]).is_some_and(|r| self.dark_room(r));
        dark && matches!(self.map.tile(ahead.x.floor(), ahead.y.floor()), Tile::Wall | Tile::Window)
    }

    pub(super) fn notice(&mut self, i: usize) {
        let e = self.echo[i];
        if !e.hunt || self.phase_tic < GRACE_TICS {
            return;
        }
        let b = self.bodies[i];
        let ahead = Vec2::from_ang(b.heading);
        let spotted = self.living_bodies().any(|l| {
            let d = l.pos - b.pos;
            let sensed = e.chase > 0 || d.len_sq() <= NOTICE_CLOSE || ahead.dot(d) >= Fx::ZERO;
            d.len_sq() <= CHASE_RANGE && sensed && self.clear(b.pos, l.pos)
        });
        self.echo[i].chase = match (spotted, e.chase) {
            (true, _) => CHASE_TICS,
            (false, c) => c.saturating_sub(1),
        };
        if spotted && e.chase == 0 {
            self.echo[i].left = 0;
        }
    }

    pub(super) fn head_of(&self, i: usize) -> Option<Ang> {
        let e = self.echo.get(i)?;
        let b = self.bodies[i];
        (self.ctl[i].agent == Agent::Echo && b.visible && (e.watched || e.look != 0)).then(|| b.heading + Ang(e.look as u16))
    }

    pub(super) fn shadows(&mut self) {
        self.echo.iter_mut().for_each(|e| e.shadow = 0);
        if self.night() < 2 {
            return;
        }
        (0..self.players_n as usize).for_each(|p| {
            if !self.living(p) {
                return;
            }
            let me = self.bodies[self.players[p].body as usize];
            let behind = Vec2::from_ang(me.heading);
            let best = (0..MAX_BODIES)
                .filter(|&j| self.ctl[j].agent == Agent::Echo && self.bodies[j].visible && self.echo[j].shadow == 0)
                .map(|j| (j, self.bodies[j].pos - me.pos))
                .filter(|&(_, d)| d.len_sq() <= SHADOW_RANGE && behind.dot(d) < Fx::ZERO)
                .filter(|&(j, _)| !self.clear(me.pos, self.bodies[j].pos))
                .min_by_key(|&(j, d)| (d.len_sq(), j));
            if let Some((j, _)) = best {
                self.echo[j].shadow = p as u8 + 1;
            }
        });
    }

    pub(super) fn echo_steps(&mut self) {
        let stepped = self.events.iter().fold(0u8, |m, e| match e {
            Event::Step { body, .. } => (0..MAX_PLAYERS).filter(|&p| self.living(p) && self.players[p].body == *body).fold(m, |m, p| m | 1 << p),
            _ => m,
        });
        (0..MAX_BODIES).for_each(|j| {
            let e = self.echo[j];
            if e.due > 0 {
                self.echo[j].due = e.due - 1;
                if e.due == 1 && self.bodies[j].visible {
                    let surface = self.surface_at(self.bodies[j].pos);
                    self.events.push(Event::EchoStep { body: j as u8, surface, of: e.of });
                }
            }
            if e.shadow > 0 && stepped & 1 << (e.shadow - 1) != 0 && self.echo[j].due == 0 {
                self.echo[j].due = ECHO_DELAY;
                self.echo[j].of = e.shadow - 1;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn head_turns_slowly_toward_camera_and_snaps_back() {
        let watched = (0..10).fold(0i16, |l, _| head_step(l, 16000, true));
        assert_eq!(watched, 1600);
        assert_eq!(head_step(1600, 0, false), 0);
        assert_eq!(head_step(-5000, 0, false), -3200);
        assert_eq!(stage_tics(2), 600);
        assert_eq!(stage_tics(5), 0);
    }
}
