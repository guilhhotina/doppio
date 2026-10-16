use super::clock::Clock;
use super::layout;
use super::pad::PLAY;
use crate::audio::Audio;
use crate::core::Frontend;
use crate::ffi::LOG_INFO;
use crate::net::{Net, Seat};
use crate::render::{CANVAS_H, CANVAS_W, Ui2d, View};
use crate::sim::{MAX_PLAYERS, Phase, Role, Setup, TICS_PER_SEC, World};
use crate::{map, ui};
use std::time::Instant;

const WHISPER_TICS: u32 = 8 * TICS_PER_SEC;
const END_FRAMES: u32 = 150;
const PAUSE_FADE: u8 = 170;
const END_FADE: u8 = 200;

pub struct Game {
    world: Box<World>,
    clock: Clock,
    tic: u32,
    seats: [Option<u8>; MAX_PLAYERS],
    online: bool,
    night: u8,
    duplo_at: [Option<u32>; MAX_PLAYERS],
    end_frames: u32,
    hashlog: u32,
    pub pause: Option<usize>,
}

pub fn local_seats(seats: &[Seat]) -> [Option<u8>; MAX_PLAYERS] {
    std::array::from_fn(|i| seats.get(i).and_then(|s| s.local_port))
}

impl Game {
    pub fn new(setup: Setup, seats: &[Seat], online: bool, hashlog: u32, now: Instant) -> Game {
        Game {
            world: Box::new(World::new(map::sobrado(), setup)),
            clock: Clock::new(now),
            tic: 0,
            seats: local_seats(seats),
            online,
            night: 1,
            duplo_at: [None; MAX_PLAYERS],
            end_frames: 0,
            hashlog,
            pause: None,
        }
    }

    pub fn online(&self) -> bool {
        self.online
    }

    fn local(&self) -> impl Iterator<Item = usize> + '_ {
        (0..MAX_PLAYERS).filter(|&s| self.seats[s].is_some())
    }

    fn finished(&self) -> bool {
        matches!(self.world.phase(), Phase::Won | Phase::Lost)
    }

    pub fn over(&self) -> bool {
        self.end_frames >= END_FRAMES
    }

    fn listener(&self) -> u8 {
        self.local().next().map_or(0, |s| self.world.player(s).body)
    }

    pub fn advance(&mut self, fe: &Frontend, net: &mut Net, audio: &mut Audio, input: &[u16; 4], now: Instant) {
        if self.pause.is_some() && !self.online {
            self.clock.hold(now);
            return;
        }
        let frozen = self.pause.is_some() || self.finished();
        let local: [u16; 4] = std::array::from_fn(|p| if frozen { 0 } else { input[p] & PLAY });
        let listener = self.listener();
        let (world, tic, hashlog) = (&mut self.world, &mut self.tic, self.hashlog);
        self.clock.run(now, || {
            net.push_local(&local);
            net.pop_tic()
                .map(|row| {
                    world.step(&row);
                    audio.events(world, world.events(), listener);
                    let h = world.hash();
                    net.report_hash(*tic, h);
                    if hashlog > 0 && *tic % hashlog == 0 {
                        fe.log(LOG_INFO, &format!("hash tic={} {h:08x}", *tic));
                    }
                    *tic += 1;
                })
                .is_some()
        });
        audio.ambience(&self.world, listener);
        self.track();
    }

    fn track(&mut self) {
        match self.world.phase() {
            Phase::Dusk { night } | Phase::Night { night } | Phase::Dawn { night } => self.night = night,
            Phase::Won | Phase::Lost => self.end_frames += 1,
        }
        let (world, tic) = (&self.world, self.tic);
        self.seats.iter().zip(self.duplo_at.iter_mut()).enumerate().filter(|(_, (port, _))| port.is_some()).for_each(|(s, (_, at))| {
            match (world.player(s).role, *at) {
                (Role::Duplo, None) => *at = Some(tic),
                (Role::Living, Some(_)) => *at = None,
                _ => {}
            }
        });
    }

    pub fn views(&self) -> ([View; 4], usize) {
        let n = self.local().count().max(1);
        let rects = layout::rects(n);
        let dim = match (self.pause.is_some(), self.over()) {
            (_, true) => END_FADE,
            (true, false) => PAUSE_FADE,
            (false, false) => 0,
        };
        let mut seats = self.local();
        let views = std::array::from_fn(|i| {
            let p = self.world.player(seats.next().filter(|_| i < n).unwrap_or(0));
            View { rect: rects[i], follow: p.body, fade: p.fade.max(dim), duplo: p.role == Role::Duplo }
        });
        (views, n)
    }

    fn whispering(&self, seat: usize) -> bool {
        let p = self.world.player(seat);
        self.duplo_at[seat].is_some_and(|at| p.fade == 0 && self.tic.wrapping_sub(at) < WHISPER_TICS)
    }

    pub fn draw(&self, ui: &mut Ui2d, frame: u32) {
        let (views, n) = self.views();
        let quiet = self.pause.is_some() || self.over();
        self.local().zip(views.iter().take(n)).filter(|_| !quiet).for_each(|(seat, v)| {
            match self.world.phase() {
                Phase::Dusk { night } => ui::dusk(ui, v.rect, night),
                Phase::Dawn { night } => ui::dawn(ui, v.rect, night),
                Phase::Night { .. } => ui::hud(ui, &self.world, v.rect, seat),
                Phase::Won | Phase::Lost => {}
            }
            if self.whispering(seat) {
                ui::whisper(ui, v.rect, frame);
            }
        });
        if n >= 2 {
            ui.rect(CANVAS_W as i16 / 2 - 1, 0, 2, CANVAS_H as i16, 0);
        }
        if n >= 3 {
            ui.rect(0, CANVAS_H as i16 / 2 - 1, CANVAS_W as i16, 2, 0);
        }
        if let Some(sel) = self.pause {
            ui::pause(ui, sel, self.online, frame);
        } else if self.over() {
            let duplo = self.local().all(|s| self.world.player(s).role == Role::Duplo);
            ui::ending(ui, self.world.phase() == Phase::Won, duplo, self.night);
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_seats_follow_ports() {
        let seat = |p: Option<u8>| Seat { nick: [0; 16], local_port: p, ready: true };
        let seats = [seat(None), seat(Some(0)), seat(Some(2))];
        assert_eq!(local_seats(&seats), [None, Some(0), Some(2), None]);
        let setup = Setup { seed: 3, players: 3, night_secs: 60 };
        let g = Game::new(setup, &seats, true, 0, Instant::now());
        let (views, n) = g.views();
        assert_eq!(n, 2);
        assert_eq!(views[0].follow, g.world().player(1).body);
        assert_eq!(views[1].follow, g.world().player(2).body);
    }
}
