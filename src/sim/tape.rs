use super::tank::Tank;
use super::{World, MAX_PLAYERS};
use crate::fx::{Ang, Vec2};

pub const SEG: u32 = 60;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Seg {
    pub d: Vec2,
    pub turn: Ang,
}

impl Seg {
    pub const ZERO: Seg = Seg { d: Vec2::ZERO, turn: Ang::ZERO };
}

pub fn tape_id(night: u8, player: usize) -> u8 {
    ((night.max(1) as usize - 1) * MAX_PLAYERS + player) as u8
}

pub fn measure(inputs: &[u16], prev: u16) -> Seg {
    let start = (Tank { prev, ..Tank::REST }, Seg::ZERO);
    inputs
        .iter()
        .fold(start, |(mut tank, seg), &input| {
            let m = tank.drive(seg.turn, input);
            tank.prev = input;
            (tank, Seg { d: seg.d + m.delta, turn: m.heading })
        })
        .1
}

impl World {
    fn tape_base(&self, id: u8) -> usize {
        id as usize * self.night_tics as usize
    }

    pub(super) fn tape_at(&self, id: u8, t: u32) -> u16 {
        self.tape[self.tape_base(id) + t as usize]
    }

    pub(super) fn record(&mut self, night: u8, player: usize, t: u32, input: u16) {
        let at = self.tape_base(tape_id(night, player)) + t as usize;
        self.tape[at] = input;
    }

    pub(super) fn cut_segments(&mut self, night: u8) {
        (0..self.players_n as usize).for_each(|p| {
            let id = tape_id(night, p);
            let base = self.tape_base(id);
            (0..self.nseg as usize).for_each(|s| {
                let start = base + s * SEG as usize;
                let prev = match s {
                    0 => 0,
                    _ => self.tape[start - 1],
                };
                let seg = measure(&self.tape[start..start + SEG as usize], prev);
                self.segs[id as usize * self.nseg as usize + s] = seg;
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::tank::{LEFT, UP, WALK};
    use super::*;
    use crate::fx::Fx;

    #[test]
    fn measure_matches_kinematics() {
        let straight = measure(&[UP; SEG as usize], 0);
        assert_eq!(straight, Seg { d: Vec2::new(Fx::ZERO, WALK * SEG as i32), turn: Ang::ZERO });
        let spin = measure(&[LEFT; 10], 0);
        assert_eq!((spin.d, spin.turn), (Vec2::ZERO, Ang(9100)));
    }
}
