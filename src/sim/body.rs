use super::tank::{pressed, Gait, A, X};
use super::{collide, Agent, Anim, Event, World, BATTERY_FULL};
use crate::fx::{Fx, Vec2};
use crate::map::Tile;

const DRAIN: u16 = 15;
const RECHARGE: u16 = 3;
const WALK_STEP: u8 = 15;
const RUN_STEP: u8 = 9;
const FIRST_STEP: u8 = 4;
const REACH: Fx = Fx::ratio(7, 10);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Reach {
    Nothing,
    Busy,
}

pub fn battery(on: bool, level: u16) -> u16 {
    match on {
        true => level.saturating_sub(DRAIN),
        false => level.saturating_add(RECHARGE).min(BATTERY_FULL),
    }
}

fn anim(staggered: bool, gait: Gait, reach: Reach) -> Anim {
    match (staggered, gait, reach) {
        (true, _, _) => Anim::Stagger,
        (_, Gait::Quick, _) => Anim::QuickTurn,
        (_, _, Reach::Busy) => Anim::Reach,
        (_, Gait::Run, _) => Anim::Run,
        (_, Gait::Walk, _) => Anim::Walk,
        (_, Gait::Turn, _) => Anim::Turn,
        (_, Gait::Still, _) => Anim::Idle,
    }
}

impl World {
    pub(super) fn solid(&self, x: i32, y: i32) -> bool {
        match self.map.tile(x, y) {
            Tile::Floor => false,
            Tile::Door => !self.door_passable(self.door_at[(y * self.map.w + x) as usize] as usize),
            _ => true,
        }
    }

    pub(super) fn drive_body(&mut self, i: usize, input: u16, staggered: bool) {
        let before = self.bodies[i];
        let mut tank = self.ctl[i].tank;
        let prev = tank.prev;
        let m = tank.drive(before.heading, input);
        let pos = collide::slide(before.pos, m.delta, |x, y| self.solid(x, y));
        tank.prev = input;
        self.ctl[i].tank = tank;
        self.bodies[i].pos = pos;
        self.bodies[i].heading = m.heading;
        let reach = self.interact(i, prev, input);
        self.lantern(i, pressed(prev, input, X) || (pressed(prev, input, A) && reach == Reach::Nothing));
        self.footstep(i, m.gait, pos != before.pos);
        let next = anim(staggered, m.gait, reach);
        self.bodies[i].anim_tic = match next == before.anim {
            true => before.anim_tic.saturating_add(1),
            false => 0,
        };
        self.bodies[i].anim = next;
    }

    pub(super) fn door_ahead(&self, i: usize) -> Option<usize> {
        let b = self.bodies[i];
        let ahead = b.pos + Vec2::from_ang(b.heading).scale(REACH);
        [ahead, b.pos].into_iter().find_map(|p| self.door_on(p))
    }

    fn interact(&mut self, i: usize, prev: u16, input: u16) -> Reach {
        if input & A == 0 {
            self.ctl[i].hold = 0;
            return Reach::Nothing;
        }
        match (self.door_ahead(i), self.reach_task(i)) {
            (Some(d), _) => {
                if pressed(prev, input, A) {
                    self.use_door(d);
                }
                Reach::Busy
            }
            (None, Some(t)) => {
                if matches!(self.ctl[i].agent, Agent::Player(p) if self.living(p as usize)) {
                    self.work(i, t);
                }
                Reach::Busy
            }
            (None, None) => Reach::Nothing,
        }
    }

    fn lantern(&mut self, i: usize, toggle: bool) {
        let on = self.bodies[i].lantern;
        let level = self.ctl[i].battery;
        let want = (on ^ toggle) && level > 0;
        let level = battery(want, level);
        let lit = want && level > 0;
        self.ctl[i].battery = level;
        self.bodies[i].lantern = lit;
        if lit != on {
            self.events.push(Event::Lantern { body: i as u8, on: lit });
        }
    }

    fn footstep(&mut self, i: usize, gait: Gait, moved: bool) {
        let interval = match gait {
            Gait::Run => RUN_STEP,
            _ => WALK_STEP,
        };
        let walking = moved && matches!(gait, Gait::Walk | Gait::Run);
        self.ctl[i].step = match (walking, self.ctl[i].step) {
            (false, _) => FIRST_STEP,
            (true, 0) => {
                if self.echo[i].shadow == 0 {
                    let surface = self.surface_at(self.bodies[i].pos);
                    self.events.push(Event::Step { body: i as u8, surface });
                }
                interval
            }
            (true, n) => n.min(interval) - 1,
        };
    }

    pub(super) fn surface_at(&self, p: Vec2) -> u8 {
        self.tile_index(p)
            .and_then(|t| self.map.room_of[t])
            .map_or(0, |r| self.room_surface[r as usize])
    }
}
