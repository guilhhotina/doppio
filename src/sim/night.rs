use super::hunt::FIELD_EVERY;
use super::tank::BUTTONS;
use super::{Agent, Event, Phase, World, MAX_BODIES, MAX_PLAYERS, NIGHTS, NO_ROOM, TICS_PER_SEC};

pub const MIN_SECS: u32 = 30;
pub const MAX_SECS: u32 = 1200;
const DUSK_TICS: u32 = 4 * TICS_PER_SEC;
const DAWN_TICS: u32 = 5 * TICS_PER_SEC;
const DUSK_MINUTES: u32 = 22 * 60;
const NIGHT_MINUTES: u32 = 8 * 60;
const DAY_MINUTES: u32 = 24 * 60;

pub fn clock(phase: Phase, tic: u32, night_tics: u32) -> u16 {
    match phase {
        Phase::Dusk { .. } => DUSK_MINUTES as u16,
        Phase::Night { .. } => ((DUSK_MINUTES + NIGHT_MINUTES * tic / night_tics) % DAY_MINUTES) as u16,
        Phase::Dawn { .. } | Phase::Won | Phase::Lost => ((DUSK_MINUTES + NIGHT_MINUTES) % DAY_MINUTES) as u16,
    }
}

pub fn at_hour(night_tics: u32, hours_after_dusk: u32) -> u32 {
    night_tics * hours_after_dusk / 8
}

impl World {
    pub(super) fn begin_dusk(&mut self, night: u8) {
        self.phase = Phase::Dusk { night };
        self.phase_tic = 0;
        self.still = 0;
        self.exit_open = false;
        self.assign_spawns(night);
        self.relock(night);
        self.select_tasks(night);
        if !self.power {
            self.power = true;
            self.events.push(Event::PowerOn);
        }
        self.populate(night);
        self.sync_players();
    }

    pub(super) fn dusk_tic(&mut self, night: u8) {
        self.phase_tic += 1;
        if self.phase_tic >= DUSK_TICS {
            self.phase = Phase::Night { night };
            self.phase_tic = 0;
            self.events.push(Event::NightBegins { night });
            if night >= NIGHTS {
                self.power = false;
                self.events.push(Event::PowerOut);
            }
        }
    }

    pub(super) fn dawn_tic(&mut self, night: u8) {
        self.phase_tic += 1;
        if self.phase_tic >= DAWN_TICS {
            self.begin_dusk(night + 1);
        }
    }

    fn end_night(&mut self, night: u8) {
        self.settle_captures();
        self.cut_segments(night);
        self.extra = self.unfinished();
        self.events.push(Event::DawnBreaks { night });
        self.phase_tic = 0;
        self.phase = match (self.phase, night >= NIGHTS) {
            (Phase::Lost, _) | (_, true) => Phase::Lost,
            _ => Phase::Dawn { night },
        };
    }

    pub(super) fn night_tic(&mut self, night: u8, inputs: &[u16; MAX_PLAYERS]) {
        let t = self.phase_tic;
        if night < NIGHTS && t == at_hour(self.night_tics, 5) && self.power && !self.breaker_done() {
            self.power = false;
            self.events.push(Event::PowerOut);
        }
        if t % FIELD_EVERY == 0 {
            self.chase_field();
        }
        self.aim_cameras();
        self.recall(t);
        self.shadows();
        let mut drive = [0u16; MAX_BODIES];
        (0..self.players_n as usize).for_each(|p| {
            let input = match self.capture[p] {
                0 => inputs[p] & BUTTONS,
                _ => 0,
            };
            self.record(night, p, t, input);
            drive[self.players[p].body as usize] = input;
        });
        (0..MAX_BODIES).for_each(|i| {
            if self.ctl[i].agent == Agent::Echo && !self.stage(i) {
                drive[i] = self.copy_input(i);
            }
        });
        (0..MAX_BODIES).for_each(|i| {
            let staggered = match self.ctl[i].agent {
                Agent::Free => return,
                Agent::Echo if !self.bodies[i].visible => return,
                Agent::Player(p) => self.capture[p as usize] > 0,
                Agent::Echo => false,
            };
            self.drive_body(i, drive[i], staggered);
        });
        self.echo_steps();
        self.swing_doors();
        self.catch();
        self.fades();
        if night >= NIGHTS {
            self.finale(&drive);
        }
        self.sync_players();
        self.remember(t);
        self.phase_tic += 1;
        if self.phase == (Phase::Night { night }) && self.phase_tic >= self.night_tics {
            self.end_night(night);
        }
    }

    pub(super) fn sync_players(&mut self) {
        (0..self.players_n as usize).for_each(|p| {
            let b = self.players[p].body as usize;
            self.players[p].battery = self.ctl[b].battery;
            self.players[p].room = self
                .tile_index(self.bodies[b].pos)
                .and_then(|t| self.map.room_of[t])
                .unwrap_or(NO_ROOM);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_runs_ten_pm_to_six_am() {
        let n = 600;
        assert_eq!(clock(Phase::Night { night: 1 }, 0, n), 22 * 60);
        assert_eq!(clock(Phase::Night { night: 1 }, at_hour(n, 2), n), 0);
        assert_eq!(clock(Phase::Night { night: 1 }, at_hour(n, 5), n), 3 * 60);
        assert_eq!(clock(Phase::Night { night: 1 }, n - 1, n), 5 * 60 + 59);
        assert_eq!(clock(Phase::Dawn { night: 1 }, 0, n), 6 * 60);
    }
}
