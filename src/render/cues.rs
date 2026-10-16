use crate::fx::{Ang, Vec2};
use crate::sim::{Anim, MAX_BODIES, MAX_PLAYERS, Task, World};

pub const MAX_TELLS: usize = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tell {
    #[default]
    Idle,
    Pending,
    Done,
}

impl Tell {
    pub fn of(t: &Task) -> Tell {
        match (t.active, t.done) {
            (_, true) => Tell::Done,
            (true, false) => Tell::Pending,
            (false, false) => Tell::Idle,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Echo {
    pub pos: Vec2,
    pub heading: Ang,
    pub anim: Anim,
    pub tic: u16,
}

#[derive(Clone, Copy)]
pub struct Cues {
    pub tells: [Tell; MAX_TELLS],
    pub gaze: [Option<Ang>; MAX_BODIES],
    pub caught: [Option<(u8, u8)>; MAX_BODIES],
    pub echo: [Option<Echo>; MAX_BODIES],
}

impl Cues {
    pub const NONE: Cues = Cues { tells: [Tell::Idle; MAX_TELLS], gaze: [None; MAX_BODIES], caught: [None; MAX_BODIES], echo: [None; MAX_BODIES] };

    pub fn of(w: &World) -> Cues {
        let mut c = Cues::NONE;
        w.tasks().iter().zip(c.tells.iter_mut()).for_each(|(t, s)| *s = Tell::of(t));
        (0..w.bodies().len().min(MAX_BODIES)).for_each(|b| c.gaze[b] = w.gaze(b));
        (0..MAX_PLAYERS).for_each(|p| {
            let b = w.player(p).body as usize;
            if b < MAX_BODIES {
                c.caught[b] = c.caught[b].or(w.captor(p));
                c.echo[b] = c.echo[b].or(w.echo_pose(p).map(|(pos, heading, anim, tic)| Echo { pos, heading, anim, tic }));
            }
        });
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::TaskKind;

    #[test]
    fn tell_follows_task_flags() {
        let t = |active, done| Tell::of(&Task { kind: TaskKind::Gas, x: 0, y: 0, active, done, room: 0 });
        assert_eq!([t(false, false), t(true, false), t(true, true)], [Tell::Idle, Tell::Pending, Tell::Done]);
    }
}
