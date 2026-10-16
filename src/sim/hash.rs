use super::echo::Mode;
use super::{Agent, Phase, Role, World};

#[derive(Clone, Copy)]
struct Fnv(u32);

impl Fnv {
    const START: Fnv = Fnv(0x811c_9dc5);

    fn word(self, v: u32) -> Fnv {
        v.to_le_bytes().iter().fold(self, |h, &b| Fnv((h.0 ^ b as u32).wrapping_mul(0x0100_0193)))
    }

    fn words(self, vs: &[u32]) -> Fnv {
        vs.iter().fold(self, |h, &v| h.word(v))
    }
}

fn phase(p: Phase) -> u32 {
    match p {
        Phase::Dusk { night } => 0x100 | night as u32,
        Phase::Night { night } => 0x200 | night as u32,
        Phase::Dawn { night } => 0x300 | night as u32,
        Phase::Won => 0x400,
        Phase::Lost => 0x500,
    }
}

fn agent(a: Agent) -> u32 {
    match a {
        Agent::Free => 0,
        Agent::Player(p) => 0x100 | p as u32,
        Agent::Echo => 0x200,
    }
}

fn mode(m: Mode) -> [u32; 2] {
    match m {
        Mode::Play => [0, 0],
        Mode::Splice(t) => [1, t as u32],
        Mode::Reverse(at) => [2, at],
        Mode::Stare => [3, 0],
        Mode::Idle(n) => [4, n as u32],
    }
}

pub fn world(w: &World) -> u32 {
    let h = Fnv::START.words(&[
        w.tic,
        phase(w.phase),
        w.phase_tic,
        w.rng.0,
        w.power as u32 | (w.exit_open as u32) << 1 | (w.extra as u32) << 8 | (w.still as u32) << 16,
    ]);
    let h = w.bodies.iter().fold(h, |h, b| {
        h.words(&[
            b.pos.x.0 as u32,
            b.pos.y.0 as u32,
            b.heading.0 as u32 | (b.anim as u32) << 16 | (b.lantern as u32) << 24 | (b.visible as u32) << 25,
            b.anim_tic as u32,
        ])
    });
    let h = w.ctl.iter().fold(h, |h, c| {
        h.words(&[
            agent(c.agent),
            c.tank.prev as u32 | (c.tank.quick as u32) << 16,
            c.tank.quick_from.0 as u32,
            c.battery as u32 | (c.step as u32) << 16 | (c.hold as u32) << 24,
            c.hold_task as u32 | (c.press as u32) << 8 | (c.press_busy as u32) << 16,
        ])
    });
    let h = w.echo.iter().fold(h, |h, e| {
        let [m, arg] = mode(e.mode);
        let flags = e.left as u32 | (e.hunt as u32) << 8 | (e.glitchy as u32) << 9 | (e.watched as u32) << 10 | (e.armed as u32) << 11;
        h.words(&[
            e.tape as u32,
            e.cursor,
            flags | (e.lure as u32) << 16 | (e.shadow as u32) << 24,
            m,
            arg,
            e.wait as u32 | (e.look as u16 as u32) << 16,
            e.chase as u32 | (e.due as u32) << 16 | (e.of as u32) << 24,
        ])
    });
    let h = w.captor.iter().fold(h, |h, &c| h.word(c as u32));
    let h = w.players.iter().zip(&w.capture).fold(h, |h, (p, &c)| {
        let role = match p.role {
            Role::Living => 0,
            Role::Duplo => 1,
        };
        h.words(&[p.body as u32 | role << 8 | (p.room as u32) << 16 | (p.fade as u32) << 24, p.battery as u32 | (c as u32) << 16])
    });
    let h = w.doors.iter().zip(&w.door_dir).fold(h, |h, (d, &dir)| h.words(&[d.open as u32 | (d.locked as u32) << 16 | (dir as u32) << 17]));
    w.tasks.iter().fold(h, |h, t| h.word(t.active as u32 | (t.done as u32) << 1)).0
}
