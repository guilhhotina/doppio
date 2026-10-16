use super::tank::{pressed, Gait, Tank, A, DROP, X};
use super::{collide, Anim, Dial, Event, ItemPlace, Outcome, PlayerState, World, BATTERY_FULL, TICS_PER_SEC};
use crate::fx::Vec2;

pub const BENEDITA: u8 = 0;
pub const TONHO: u8 = 1;
pub const JAIRO: u8 = 2;
pub const BIA: u8 = 3;

pub const HIDE_TICS: u8 = 15;
pub const GRAB_TICS: u8 = 20;
const REACH_TICS: u8 = 8;
const TAP_TICS: u8 = (TICS_PER_SEC / 2) as u8;
const DRAIN: u16 = 15;
const RECHARGE: u16 = 3;
const WALK_STEP: u8 = 15;
const RUN_STEP: u8 = 9;
const FIRST_STEP: u8 = 4;
const FADE_OUT: u8 = 17;

pub const WALK_NOISE: u8 = 20;
pub const RUN_NOISE: u8 = 70;
pub const DROP_NOISE: u8 = 60;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ctl {
    pub tank: Tank,
    pub press: u8,
    pub press_busy: bool,
    pub step: u8,
    pub reach: u8,
    pub hide_t: u8,
    pub spot: u8,
    pub grab: u8,
    pub dial: Option<Dial>,
    pub reading: Option<u16>,
}

impl Ctl {
    pub const REST: Ctl = Ctl {
        tank: Tank::REST,
        press: 0,
        press_busy: false,
        step: FIRST_STEP,
        reach: 0,
        hide_t: 0,
        spot: 0,
        grab: 0,
        dial: None,
        reading: None,
    };
}

pub fn wake_tics(character: u8) -> u16 {
    match character {
        BENEDITA => 3 * TICS_PER_SEC as u16,
        _ => 6 * TICS_PER_SEC as u16,
    }
}

pub fn slots(character: u8) -> usize {
    match character {
        JAIRO => 2,
        _ => 1,
    }
}

fn drain(character: u8) -> u16 {
    match character {
        BIA => DRAIN / 3,
        _ => DRAIN,
    }
}

fn step_noise(character: u8, run: bool) -> u8 {
    match (character, run) {
        (TONHO, false) => 0,
        (TONHO, true) => RUN_NOISE / 2,
        (_, false) => WALK_NOISE,
        (_, true) => RUN_NOISE,
    }
}

fn gait_anim(gait: Gait) -> Anim {
    match gait {
        Gait::Quick => Anim::QuickTurn,
        Gait::Run => Anim::Run,
        Gait::Walk => Anim::Walk,
        Gait::Turn => Anim::Turn,
        Gait::Still => Anim::Idle,
    }
}

impl World {
    pub(super) fn player_tic(&mut self, p: usize, input: u16) {
        let prev = self.ctl[p].tank.prev;
        let before = self.players[p];
        let anim = match before.state {
            PlayerState::Free => self.free_tic(p, prev, input),
            PlayerState::Hidden { .. } => self.hidden_tic(p, prev, input),
            PlayerState::Grabbed => self.grabbed_tic(p),
            PlayerState::Fainted { tics_left } => self.fainted_tic(p, tics_left),
            PlayerState::Escaped => Anim::Idle,
        };
        self.ctl[p].tank.prev = input;
        let grab = self.ctl[p].grab;
        let pl = &mut self.players[p];
        pl.anim_tic = if anim == before.anim { pl.anim_tic.saturating_add(1) } else { 0 };
        pl.anim = anim;
        pl.fade = match pl.state {
            PlayerState::Grabbed => ((GRAB_TICS - grab.min(GRAB_TICS)) as u32 * 255 / GRAB_TICS as u32) as u8,
            PlayerState::Fainted { .. } => 255,
            _ => pl.fade.saturating_sub(FADE_OUT),
        };
    }

    fn busy_hold(&mut self, p: usize, input: u16) {
        let c = &mut self.ctl[p];
        c.press_busy |= input & A != 0;
    }

    fn free_tic(&mut self, p: usize, prev: u16, input: u16) -> Anim {
        let c = self.ctl[p];
        if c.hide_t > 0 {
            self.busy_hold(p, input);
            self.ctl[p].hide_t -= 1;
            if c.hide_t == 1 {
                self.enter_hide(p);
            }
            return Anim::Crouch;
        }
        if c.dial.is_some() {
            self.busy_hold(p, input);
            self.dial_tic(p, prev, input);
            self.lantern(p, false);
            return Anim::Reach;
        }
        if c.reading.is_some() {
            self.busy_hold(p, input);
            if pressed(prev, input, A) {
                self.ctl[p].reading = None;
            }
            self.lantern(p, false);
            return Anim::Idle;
        }
        let pl = self.players[p];
        let mut tank = c.tank;
        let m = tank.drive(pl.heading, input);
        self.ctl[p].tank = tank;
        let pos = collide::slide(pl.pos, m.delta, |x, y| self.player_solid(x, y));
        self.place_player(p, pos, m.heading);
        let acted = pressed(prev, input, A) && self.press_a(p);
        if pressed(prev, input, DROP) {
            self.drop_held(p);
        }
        let tap = self.tap(p, prev, input, acted);
        self.lantern(p, pressed(prev, input, X) || tap);
        self.footstep(p, m.gait, pos != pl.pos);
        self.escape(p);
        self.ctl[p].reach = match acted {
            true => REACH_TICS,
            false => self.ctl[p].reach.saturating_sub(1),
        };
        match self.ctl[p].reach > 0 && m.gait == Gait::Still {
            true => Anim::Reach,
            false => gait_anim(m.gait),
        }
    }

    pub(super) fn place_player(&mut self, p: usize, pos: Vec2, heading: crate::fx::Ang) {
        let area = self.grid.area_of(pos);
        let pl = &mut self.players[p];
        pl.pos = pos;
        pl.heading = heading;
        if area != super::grid::NONE {
            pl.area = area;
        }
    }

    fn tap(&mut self, p: usize, prev: u16, input: u16, acted: bool) -> bool {
        let c = self.ctl[p];
        let (press, busy) = match (input & A != 0, pressed(prev, input, A)) {
            (true, true) => (1, acted),
            (true, false) => (c.press.saturating_add(1), c.press_busy),
            (false, _) => (0, false),
        };
        self.ctl[p].press = press;
        self.ctl[p].press_busy = busy;
        prev & A != 0 && input & A == 0 && c.press < TAP_TICS && !c.press_busy
    }

    pub(super) fn lantern(&mut self, p: usize, toggle: bool) {
        let pl = self.players[p];
        let want = (pl.lantern ^ toggle) && pl.battery > 0;
        let level = match want {
            true => pl.battery.saturating_sub(drain(pl.character)),
            false => pl.battery.saturating_add(RECHARGE).min(BATTERY_FULL),
        };
        self.set_lantern(p, want && level > 0);
        self.players[p].battery = level;
    }

    fn set_lantern(&mut self, p: usize, on: bool) {
        if self.players[p].lantern != on {
            self.players[p].lantern = on;
            self.events.push(Event::Lantern { player: p as u8, on });
        }
    }

    fn footstep(&mut self, p: usize, gait: Gait, moved: bool) {
        let run = gait == Gait::Run;
        let interval = if run { RUN_STEP } else { WALK_STEP };
        let walking = moved && matches!(gait, Gait::Walk | Gait::Run);
        self.ctl[p].step = match (walking, self.ctl[p].step) {
            (false, _) => FIRST_STEP,
            (true, 0) => {
                let pl = self.players[p];
                self.events.push(Event::Step { player: p as u8, surface: self.grid.surface_at(pl.pos), run });
                self.noise(pl.pos, pl.area, step_noise(pl.character, run));
                interval
            }
            (true, n) => n.min(interval) - 1,
        };
    }

    pub(super) fn noise(&mut self, pos: Vec2, area: u8, loud: u8) {
        if loud > 0 {
            self.events.push(Event::Noise { pos, area, loud });
        }
    }

    fn escape(&mut self, p: usize) {
        let pl = self.players[p];
        let exit = self.map.exit;
        if self.lock_open(Some(exit.lock)) && exit.rect.contains(pl.pos.x.floor(), pl.pos.y.floor()) {
            self.set_lantern(p, false);
            self.players[p].state = PlayerState::Escaped;
            self.events.push(Event::Escaped { player: p as u8 });
        }
    }

    fn hidden_tic(&mut self, p: usize, prev: u16, input: u16) -> Anim {
        self.busy_hold(p, input);
        self.lantern(p, false);
        let c = self.ctl[p];
        match (c.hide_t, pressed(prev, input, A)) {
            (1, _) => {
                self.ctl[p].hide_t = 0;
                self.leave_hide(p);
                Anim::Crouch
            }
            (0, true) if self.players[p].character == TONHO => {
                self.leave_hide(p);
                Anim::Idle
            }
            (0, true) => {
                self.ctl[p].hide_t = HIDE_TICS;
                Anim::Crouch
            }
            (0, false) => Anim::Idle,
            (_, _) => {
                self.ctl[p].hide_t -= 1;
                Anim::Crouch
            }
        }
    }

    pub(super) fn start_hide(&mut self, p: usize, spot: u8) {
        self.ctl[p].spot = spot;
        self.legend_watch_hide(p);
        match self.players[p].character {
            TONHO => self.enter_hide(p),
            _ => self.ctl[p].hide_t = HIDE_TICS,
        }
    }

    fn enter_hide(&mut self, p: usize) {
        self.ctl[p].hide_t = 0;
        self.players[p].state = PlayerState::Hidden { spot: self.ctl[p].spot };
        self.events.push(Event::Hide { player: p as u8 });
        self.legend_watch_hide(p);
    }

    fn leave_hide(&mut self, p: usize) {
        self.players[p].state = PlayerState::Free;
        self.events.push(Event::Unhide { player: p as u8 });
    }

    pub(super) fn grab_player(&mut self, p: usize) {
        if matches!(self.players[p].state, PlayerState::Hidden { .. }) {
            self.events.push(Event::Unhide { player: p as u8 });
        }
        let c = &mut self.ctl[p];
        c.grab = GRAB_TICS;
        c.dial = None;
        c.reading = None;
        c.hide_t = 0;
        self.players[p].state = PlayerState::Grabbed;
        self.set_lantern(p, false);
        self.events.push(Event::Grabbed { player: p as u8 });
    }

    fn grabbed_tic(&mut self, p: usize) -> Anim {
        self.ctl[p].grab = self.ctl[p].grab.saturating_sub(1);
        if self.ctl[p].grab == 0 {
            self.faint(p);
            return Anim::Faint;
        }
        Anim::Grabbed
    }

    fn faint(&mut self, p: usize) {
        (0..2).for_each(|_| {
            self.drop_held_quietly(p);
        });
        self.players[p].state = PlayerState::Fainted { tics_left: wake_tics(self.players[p].character) };
        self.events.push(Event::Fainted { player: p as u8 });
        match self.desmaios {
            0 => {
                self.outcome = Outcome::Restart;
                self.events.push(Event::PhaseRestart);
            }
            _ => self.desmaios -= 1,
        }
    }

    fn fainted_tic(&mut self, p: usize, left: u16) -> Anim {
        match left {
            0 | 1 => {
                let wake = self.map.wakes[self.players[p].character as usize];
                self.place_player(p, wake.pos, wake.rot);
                self.wake(p);
                Anim::Idle
            }
            _ => {
                self.players[p].state = PlayerState::Fainted { tics_left: left - 1 };
                Anim::Faint
            }
        }
    }

    pub(super) fn wake(&mut self, p: usize) {
        self.players[p].state = PlayerState::Free;
        self.ctl[p] = Ctl { tank: Tank { prev: self.ctl[p].tank.prev, ..Tank::REST }, press_busy: true, ..Ctl::REST };
        self.events.push(Event::Woke { player: p as u8 });
    }

    pub(super) fn drop_held(&mut self, p: usize) {
        if self.drop_held_quietly(p) {
            let pl = self.players[p];
            self.noise(pl.pos, pl.area, DROP_NOISE);
        }
    }

    fn drop_held_quietly(&mut self, p: usize) -> bool {
        let pl = self.players[p];
        let slot = (0..2).rev().find(|&s| pl.hands[s].is_some());
        slot.and_then(|s| pl.hands[s].map(|i| (s, i)))
            .map(|(s, i)| {
                self.players[p].hands[s] = None;
                self.items[i as usize].place = ItemPlace::Floor { pos: pl.pos, area: pl.area };
                self.events.push(Event::Drop { player: p as u8, item: i });
            })
            .is_some()
    }
}
