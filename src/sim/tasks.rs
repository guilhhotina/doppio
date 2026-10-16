use super::{shuffle, Event, TaskKind, World, MAX_TASKS};
use crate::fx::{Fx, Vec2};

pub const HOLD_TICS: u8 = 45;
const REACH: i64 = (Fx::ratio(8, 10).0 as i64) * (Fx::ratio(8, 10).0 as i64);

impl World {
    fn task_center(&self, t: usize) -> Vec2 {
        Vec2::new(Fx::int(self.tasks[t].x as i32) + Fx::HALF, Fx::int(self.tasks[t].y as i32) + Fx::HALF)
    }

    pub(super) fn task_near(&self, p: Vec2) -> Option<usize> {
        (0..self.tasks.len()).find(|&t| (self.task_center(t) - p).len_sq() < REACH)
    }

    pub(super) fn work(&mut self, i: usize, t: usize) {
        let open = self.tasks[t].active && !self.tasks[t].done;
        let c = self.ctl[i];
        let hold = match (open, c.hold_task as usize == t) {
            (false, _) => 0,
            (true, true) => c.hold.saturating_add(1),
            (true, false) => 1,
        };
        self.ctl[i].hold_task = t as u8;
        self.ctl[i].hold = match hold >= HOLD_TICS {
            true => {
                self.finish_task(t);
                0
            }
            false => hold,
        };
    }

    fn finish_task(&mut self, t: usize) {
        self.tasks[t].done = true;
        self.events.push(Event::TaskDone { task: t as u8 });
        if self.tasks[t].kind == TaskKind::Breaker && !self.power {
            self.power = true;
            self.events.push(Event::PowerOn);
        }
    }

    pub(super) fn breaker_done(&self) -> bool {
        self.tasks.iter().any(|t| t.kind == TaskKind::Breaker && t.active && t.done)
    }

    pub(super) fn unfinished(&self) -> u8 {
        self.tasks.iter().filter(|t| t.active && !t.done).count() as u8
    }

    pub(super) fn select_tasks(&mut self, night: u8) {
        self.tasks.iter_mut().for_each(|t| {
            t.active = false;
            t.done = false;
        });
        if night >= 5 {
            return;
        }
        let n = self.tasks.len();
        let mut order = [0u8; MAX_TASKS];
        order.iter_mut().enumerate().for_each(|(i, o)| *o = i as u8);
        shuffle(&mut self.rng, &mut order[..n]);
        let reachable = |w: &World, t: usize| w.reached(w.task_center(t));
        let breaker = order[..n]
            .iter()
            .map(|&t| t as usize)
            .find(|&t| self.tasks[t].kind == TaskKind::Breaker && reachable(self, t));
        if let Some(b) = breaker {
            self.tasks[b].active = true;
        }
        let want = 2 + self.rng.below(2) as usize;
        order[..n].iter().map(|&t| t as usize).fold(0, |picked, t| {
            let kind = self.tasks[t].kind;
            let fresh = !self.tasks.iter().any(|o| o.active && o.kind == kind);
            match picked < want && fresh && reachable(self, t) {
                true => {
                    self.tasks[t].active = true;
                    picked + 1
                }
                false => picked,
            }
        });
    }
}
