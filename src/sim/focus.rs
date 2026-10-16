use super::tasks::HOLD_TICS;
use super::{Agent, Focus, TaskKind, World};
use crate::fx::{Fx, Vec2};
use crate::map::{Map, Tile};

const REACH: i64 = (Fx::ratio(12, 10).0 as i64) * (Fx::ratio(12, 10).0 as i64);
const NEAR_PROP: i64 = 4 << 32;
const SIDES: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

fn props(kind: TaskKind) -> &'static [&'static str] {
    match kind {
        TaskKind::Window => &["cortina"],
        TaskKind::Breaker => &["disjuntor"],
        TaskKind::Key => &["criado_mudo", "cristaleira", "estante", "geladeira", "guarda_roupa", "rack"],
        TaskKind::Tv => &["tv_tubo"],
        TaskKind::Gas => &["fogao"],
        TaskKind::Faucet => &["pia", "banheira", "filtro_barro"],
    }
}

fn centre(x: i32, y: i32) -> Vec2 {
    Vec2::new(Fx::int(x) + Fx::HALF, Fx::int(y) + Fx::HALF)
}

pub fn anchor(map: &Map, kind: TaskKind, x: i32, y: i32) -> Vec2 {
    let here = centre(x, y);
    let target = map
        .props
        .iter()
        .filter(|p| props(kind).contains(&p.kind))
        .map(|p| p.pos)
        .filter(|&p| (p - here).len_sq() < NEAR_PROP)
        .min_by_key(|&p| (p - here).len_sq());
    let preferred = |t: Tile| match kind {
        TaskKind::Window => t == Tile::Window,
        _ => t != Tile::Window,
    };
    SIDES
        .iter()
        .map(|&(dx, dy)| (dx, dy, map.tile(x + dx, y + dy)))
        .filter(|&(_, _, t)| !t.walkable() && t != Tile::Void)
        .min_by_key(|&(dx, dy, t)| (!preferred(t), target.map_or(0, |p| (centre(x + dx, y + dy) - p).len_sq())))
        .map_or(here, |(dx, dy, _)| here + Vec2::new(Fx::HALF * dx, Fx::HALF * dy))
}

pub fn facing(heading: Vec2, d: Vec2) -> bool {
    heading.dot(d).0 as i64 * 2 >= d.len().0 as i64
}

impl World {
    pub(super) fn reach_task(&self, i: usize) -> Option<usize> {
        let b = self.bodies[i];
        let worker = matches!(self.ctl[i].agent, Agent::Player(p) if self.living(p as usize));
        let ahead = Vec2::from_ang(b.heading);
        (0..self.tasks.len())
            .filter(|&t| !worker || (self.tasks[t].active && !self.tasks[t].done))
            .map(|t| (t, self.anchors[t] - b.pos))
            .filter(|&(_, d)| d.len_sq() <= REACH && facing(ahead, d))
            .min_by_key(|&(t, d)| (d.len_sq(), t))
            .map(|(t, _)| t)
    }

    pub(super) fn focus_of(&self, i: usize) -> Option<Focus> {
        let door = |d: usize| Focus::Door { door: d as u8, locked: self.doors[d].locked, open: self.door_dir[d] };
        let task = |t: usize| {
            let c = self.ctl[i];
            let held = match c.hold_task as usize == t {
                true => c.hold as u32,
                false => 0,
            };
            Focus::Task { task: t as u8, kind: self.tasks[t].kind, progress: (held * u16::MAX as u32 / HOLD_TICS as u32) as u16 }
        };
        self.door_ahead(i).map(door).or_else(|| self.reach_task(i).map(task))
    }
}

#[cfg(test)]
mod tests {
    use super::super::tank::A;
    use super::super::tests::{sobrado_world, tiny_world};
    use super::super::*;
    use super::*;
    use crate::fx::Ang;

    #[test]
    fn anchors_sit_on_the_object_face() {
        let w = sobrado_world(1, 30);
        let spot = |kind: TaskKind, x: u8, y: u8| {
            let t = w.tasks.iter().position(|t| (t.kind, t.x, t.y) == (kind, x, y)).unwrap();
            w.task_spot(t)
        };
        assert_eq!(spot(TaskKind::Window, 2, 4), Vec2::new(Fx::int(2) + Fx::HALF, Fx::int(4)));
        assert_eq!(spot(TaskKind::Tv, 2, 7), Vec2::new(Fx::int(2), Fx::int(7) + Fx::HALF));
        assert_eq!(spot(TaskKind::Breaker, 21, 13), Vec2::new(Fx::int(21) + Fx::HALF, Fx::int(14)));
        assert_eq!(spot(TaskKind::Faucet, 14, 15), Vec2::new(Fx::int(15), Fx::int(15) + Fx::HALF));
        assert!(w.tasks.iter().all(|t| t.room != NO_ROOM));
    }

    #[test]
    fn task_focus_needs_reach_and_facing() {
        let mut w = tiny_world(1, 30);
        while w.phase() != (Phase::Night { night: 1 }) {
            w.step(&[0; MAX_PLAYERS]);
        }
        let t = w.tasks.iter().position(|t| t.active && t.kind != TaskKind::Breaker).unwrap();
        w.tasks.iter_mut().enumerate().for_each(|(k, task)| task.active = k == t);
        let a = w.task_spot(t);
        let face = (a - centre(w.tasks[t].x as i32, w.tasks[t].y as i32)).ang();
        let deg = |d: i32| Ang::deg(Fx::int(d));
        let stand = |w: &mut World, back: Fx, turn: Ang| {
            w.bodies[0].pos = a - Vec2::from_ang(face).scale(back);
            w.bodies[0].heading = face + turn;
            w.focus(0)
        };
        assert!(matches!(stand(&mut w, Fx::ratio(11, 10), Ang::ZERO), Some(Focus::Task { progress: 0, .. })));
        assert!(matches!(stand(&mut w, Fx::ratio(5, 10), deg(55)), Some(Focus::Task { .. })));
        assert_eq!(stand(&mut w, Fx::ratio(5, 10), deg(70)), None);
        assert_eq!(stand(&mut w, Fx::ratio(13, 10), Ang::ZERO), None);
        stand(&mut w, Fx::ratio(9, 10), -deg(40));
        (0..HOLD_TICS - 1).for_each(|_| {
            w.step(&[A, 0, 0, 0]);
        });
        let nearly = matches!(w.focus(0), Some(Focus::Task { progress, .. }) if progress > 60000);
        w.step(&[A, 0, 0, 0]);
        assert!(nearly && w.tasks[t].done && w.focus(0).is_none());
    }
}
