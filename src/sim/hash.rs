use super::{ItemPlace, PlayerState, World};
use crate::fx::Vec2;

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

    fn vec(self, v: Vec2) -> Fnv {
        self.words(&[v.x.0 as u32, v.y.0 as u32])
    }
}

fn state(s: PlayerState) -> u32 {
    match s {
        PlayerState::Free => 0,
        PlayerState::Hidden { spot } => 0x100 | spot as u32,
        PlayerState::Grabbed => 0x200,
        PlayerState::Fainted { tics_left } => 0x30000 | tics_left as u32,
        PlayerState::Escaped => 0x400,
    }
}

fn place(p: ItemPlace) -> [u32; 3] {
    match p {
        ItemPlace::Floor { pos, area } => [area as u32, pos.x.0 as u32, pos.y.0 as u32],
        ItemPlace::Held { player } => [0x100 | player as u32, 0, 0],
        ItemPlace::Inside { prop } => [0x10000 | prop as u32, 0, 0],
        ItemPlace::Used => [0x20000, 0, 0],
    }
}

pub fn world(w: &World) -> u32 {
    let h = Fnv::START.words(&[w.tic, w.rng.0, w.outcome as u32 | (w.desmaios as u32) << 8 | (w.power as u32) << 16]);
    let h = w.players[..w.count].iter().zip(&w.ctl).fold(h, |h, (p, c)| {
        let hands = p.hands.map(|i| i.map_or(0xff, |i| i as u32));
        let dial = c.dial.map_or([0; 3], |d| {
            [d.lock as u32 | (d.len as u32) << 8 | (d.cursor as u32) << 16 | (d.wrong as u32) << 24, u32::from_le_bytes(d.digits), 1]
        });
        h.vec(p.pos).words(&[
            p.character as u32 | (p.area as u32) << 8 | (p.anim as u32) << 16 | (p.lantern as u32) << 24,
            p.heading.0 as u32 | (p.anim_tic as u32) << 16,
            p.battery as u32 | (p.fade as u32) << 16,
            hands[0] | hands[1] << 8,
            state(p.state),
            c.tank.prev as u32 | (c.tank.quick as u32) << 16 | (c.press as u32) << 24,
            c.tank.quick_from.0 as u32 | (c.press_busy as u32) << 16 | (c.step as u32) << 24,
            c.reach as u32 | (c.hide_t as u32) << 8 | (c.spot as u32) << 16 | (c.grab as u32) << 24,
            dial[0],
            dial[1],
            dial[2] | c.reading.map_or(0xffff, |n| n as u32) << 8,
        ])
    });
    let l = w.legend;
    let b = w.brain;
    let h = h.vec(l.pos).words(&[
        l.heading.0 as u32 | (l.area as u32) << 16 | (l.anim as u32) << 24,
        l.anim_tic as u32 | (l.mode as u32) << 16 | l.target.map_or(0xff, |t| t as u32) << 24,
        l.stunned as u32,
    ]);
    let h = h.vec(b.goal).vec(b.origin).vec(b.charge_dir).words(&[
        b.dirty as u32 | (b.seen_hide as u32) << 1 | (b.waypoint as u32) << 8 | (b.special as u32) << 16 | (b.step as u32) << 24,
        b.field_goal as u32 | (b.field_complete as u32) << 16,
        b.hear_cell as u32 | (b.hear_age as u32) << 16,
        b.timer as u32 | (b.pause as u32) << 16,
        b.cry as u32 | (b.breath as u32) << 16,
        b.charge as u32 | (b.charge_cool as u32) << 16,
        b.stall as u32 | (b.best as u32) << 16,
        b.portal_wait as u32,
    ]);
    let h = w.doors.iter().zip(&w.door_dir).zip(&w.salt).fold(h, |h, ((d, &dir), &s)| {
        h.words(&[d.open as u32 | (d.locked as u32) << 16 | (d.stuck as u32) << 17 | (dir as u32) << 18, s as u32])
    });
    let h = w.props.iter().fold(h, |h, s| h.word(s.open as u32));
    let h = w.items.iter().fold(h, |h, s| h.words(&place(s.place)));
    w.locks.iter().fold(h, |h, s| h.word(s.open as u32)).0
}
