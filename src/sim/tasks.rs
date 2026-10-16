use super::{shuffle, Event, TaskKind, World, MAX_TASKS, NO_ROOM};
use crate::fx::{Fx, Vec2};

pub const HOLD_TICS: u8 = 45;
pub const DARK_LIGHT: u8 = 15;

impl World {
    fn task_center(&self, t: usize) -> Vec2 {
        Vec2::new(Fx::int(self.tasks[t].x as i32) + Fx::HALF, Fx::int(self.tasks[t].y as i32) + Fx::HALF)
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

    pub(super) fn dark_room(&self, room: u8) -> bool {
        self.map.rooms.get(room as usize).is_some_and(|r| r.light <= DARK_LIGHT)
    }

    fn fits(&self, t: usize, dark: bool) -> bool {
        let task = self.tasks[t];
        let fresh = !self.tasks.iter().any(|o| o.active && (o.kind == task.kind || (o.room == task.room && task.room != NO_ROOM)));
        fresh && (!dark || self.dark_room(task.room)) && self.reached(self.task_center(t))
    }

    fn pick_task(&mut self, order: &[u8], dark: bool) -> Option<usize> {
        let t = order.iter().map(|&t| t as usize).find(|&t| self.fits(t, dark))?;
        self.tasks[t].active = true;
        Some(t)
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
        let breaker = order[..n]
            .iter()
            .map(|&t| t as usize)
            .find(|&t| self.tasks[t].kind == TaskKind::Breaker && self.reached(self.task_center(t)));
        if let Some(b) = breaker {
            self.tasks[b].active = true;
        }
        let want = 2 + self.rng.below(2) as usize;
        let order = &order[..n];
        if self.pick_task(order, true).is_none() {
            self.pick_task(order, false);
        }
        (1..want).for_each(|_| {
            self.pick_task(order, false);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::sobrado_world;
    use super::super::Task;
    use super::*;

    #[test]
    fn nightly_tasks_spread_over_rooms_and_reach_the_dark() {
        (1..40u32).for_each(|seed| {
            let mut w = sobrado_world(2, 30);
            w.rng = crate::fx::Rng::new(seed);
            w.select_tasks(1 + (seed % 4) as u8);
            let active: Vec<Task> = w.tasks.iter().copied().filter(|t| t.active).collect();
            assert!((3..=4).contains(&active.len()), "{seed}: {active:?}");
            assert!(active.iter().enumerate().all(|(i, a)| active[i + 1..].iter().all(|b| a.room != b.room && a.kind != b.kind)));
            assert!(active.iter().filter(|t| t.kind != TaskKind::Breaker).any(|t| w.dark_room(t.room)), "{seed}: {active:?}");
        });
    }
}
