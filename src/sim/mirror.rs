use super::{Anim, Body, World};
use crate::fx::{Ang, Fx, Vec2};

const SHIFT: u32 = 5;
const TIC_BITS: u16 = 13;
const ANIMS: [Anim; 7] = [Anim::Idle, Anim::Walk, Anim::Run, Anim::Turn, Anim::QuickTurn, Anim::Reach, Anim::Stagger];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pose([u16; 4]);

impl Pose {
    pub const NONE: Pose = Pose([0; 4]);

    pub fn of(b: &Body) -> Pose {
        let q = |v: Fx| (v.0 >> SHIFT).clamp(0, u16::MAX as i32) as u16;
        let anim = ANIMS.iter().position(|&a| a == b.anim).unwrap_or(0) as u16;
        Pose([q(b.pos.x), q(b.pos.y), b.heading.0, anim << TIC_BITS | b.anim_tic.min((1 << TIC_BITS) - 1)])
    }

    pub fn decode(self) -> (Vec2, Ang, Anim, u16) {
        let [x, y, h, a] = self.0;
        let pos = Vec2::new(Fx((x as i32) << SHIFT), Fx((y as i32) << SHIFT));
        (pos, Ang(h), ANIMS[(a >> TIC_BITS) as usize % ANIMS.len()], a & ((1 << TIC_BITS) - 1))
    }
}

impl World {
    pub(super) fn recall(&mut self, t: u32) {
        let n = self.night_tics as usize;
        (0..self.players_n as usize).for_each(|p| self.mirror[p] = self.poses[p * n + t as usize]);
    }

    pub(super) fn remember(&mut self, t: u32) {
        let n = self.night_tics as usize;
        (0..self.players_n as usize).for_each(|p| self.poses[p * n + t as usize] = Pose::of(&self.bodies[self.players[p].body as usize]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pose_round_trips() {
        let b = Body {
            pos: Vec2::new(Fx::int(15) + Fx::ratio(1, 4), Fx::int(27) + Fx::HALF),
            heading: Ang(41000),
            anim: Anim::Reach,
            anim_tic: 77,
            ..super::super::HIDDEN
        };
        let (pos, h, a, t) = Pose::of(&b).decode();
        assert_eq!((pos, h, a, t), (b.pos, b.heading, b.anim, 77));
    }
}
