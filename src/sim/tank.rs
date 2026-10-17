use crate::fx::{Ang, Fx, Vec2};

pub const B: u16 = 1 << 0;
pub const Y: u16 = 1 << 1;
pub const UP: u16 = 1 << 4;
pub const DOWN: u16 = 1 << 5;
pub const LEFT: u16 = 1 << 6;
pub const RIGHT: u16 = 1 << 7;
pub const A: u16 = 1 << 8;
pub const X: u16 = 1 << 9;
pub const DROP: u16 = 1 << 10;

pub const WALK: Fx = Fx::ratio(13, 300);
pub const BACK: Fx = Fx::ratio(8, 300);
pub const RUN: Fx = Fx::ratio(26, 300);
pub const TURN: u16 = 910;
pub const QUICK_TICS: u8 = 12;
pub const DASH_WINDOW: u8 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gait {
    Still,
    Turn,
    Walk,
    Run,
    Quick,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tank {
    pub prev: u16,
    pub quick: u8,
    pub quick_from: Ang,
    pub dash: u8,
    pub sprint: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Motion {
    pub delta: Vec2,
    pub heading: Ang,
    pub gait: Gait,
}

pub fn pressed(prev: u16, input: u16, bit: u16) -> bool {
    input & bit != 0 && prev & bit == 0
}

impl Tank {
    pub const REST: Tank = Tank { prev: 0, quick: 0, quick_from: Ang::ZERO, dash: 0, sprint: false };

    pub fn drive(&mut self, heading: Ang, input: u16) -> Motion {
        let sprint = self.sprint(input);
        if self.quick == 0 && pressed(self.prev, input, Y) {
            self.quick = QUICK_TICS;
            self.quick_from = heading;
        }
        match self.quick {
            0 => walk(heading, input, sprint),
            q => {
                self.quick = q - 1;
                let done = (QUICK_TICS - self.quick) as u32;
                let turn = Ang((Ang::HALF.0 as u32 * done / QUICK_TICS as u32) as u16);
                Motion { delta: Vec2::ZERO, heading: self.quick_from + turn, gait: Gait::Quick }
            }
        }
    }

    fn sprint(&mut self, input: u16) -> bool {
        let (up, steer, side) = (input & UP != 0, input & (LEFT | RIGHT | DOWN) != 0, input & (LEFT | RIGHT) != 0);
        let (sprint, dash) = match (up, pressed(self.prev, input, UP)) {
            (true, true) => (self.dash > 0 && !side, 0),
            (true, false) => (self.sprint, 0),
            (false, _) if steer => (false, 0),
            (false, _) if self.prev & UP != 0 => (false, DASH_WINDOW),
            (false, _) => (false, self.dash.saturating_sub(1)),
        };
        self.sprint = sprint;
        self.dash = dash;
        sprint
    }
}

fn walk(heading: Ang, input: u16, sprint: bool) -> Motion {
    let held = |bit: u16| input & bit != 0;
    let turn = match (held(LEFT), held(RIGHT)) {
        (true, false) => Ang(TURN),
        (false, true) => -Ang(TURN),
        _ => Ang::ZERO,
    };
    let heading = heading + turn;
    let (speed, gait) = match (held(UP), held(DOWN), held(B) || sprint) {
        (true, false, true) => (RUN, Gait::Run),
        (true, false, false) => (WALK, Gait::Walk),
        (false, true, _) => (-BACK, Gait::Walk),
        _ if turn != Ang::ZERO => (Fx::ZERO, Gait::Turn),
        _ => (Fx::ZERO, Gait::Still),
    };
    Motion { delta: Vec2::from_ang(heading).scale(speed), heading, gait }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speeds_and_turns() {
        let mut t = Tank::REST;
        let m = t.drive(Ang::ZERO, UP);
        assert_eq!((m.delta.x, m.delta.y, m.gait), (Fx::ZERO, WALK, Gait::Walk));
        assert_eq!(t.drive(Ang::QUARTER, UP | B).delta.x, RUN);
        assert_eq!(t.drive(Ang::ZERO, DOWN | B).delta.y, -BACK);
        assert_eq!(t.drive(Ang::ZERO, LEFT).heading, Ang(TURN));
        assert_eq!(t.drive(Ang::ZERO, RIGHT).gait, Gait::Turn);
    }

    #[test]
    fn quick_turn_is_half_turn_in_twelve_tics() {
        let mut t = Tank::REST;
        let end = (0..QUICK_TICS).fold(Ang(1000), |h, i| {
            let m = t.drive(h, if i == 0 { Y } else { Y | UP });
            t.prev = Y;
            assert_eq!((m.gait, m.delta), (Gait::Quick, Vec2::ZERO));
            m.heading
        });
        assert_eq!(end, Ang(1000) + Ang::HALF);
        assert_eq!(t.drive(end, Y).gait, Gait::Still);
    }

    fn gaits(inputs: &[u16]) -> Vec<Gait> {
        let mut t = Tank::REST;
        inputs
            .iter()
            .map(|&i| {
                let g = t.drive(Ang::ZERO, i).gait;
                t.prev = i;
                g
            })
            .collect()
    }

    #[test]
    fn double_tap_up_runs_until_released() {
        let g = gaits(&[UP, UP, 0, 0, UP, UP | LEFT, UP, 0]);
        assert_eq!(&g[4..7], &[Gait::Run, Gait::Run, Gait::Run]);
        assert_eq!(g[7], Gait::Still);
        assert_eq!(gaits(&[UP, 0, 0, 0, 0, 0, 0, 0, 0, 0, UP])[10], Gait::Walk);
        assert_eq!(gaits(&[UP, 0, 0, 0, 0, 0, 0, 0, 0, UP])[9], Gait::Run);
        assert_eq!(gaits(&[UP, LEFT, UP])[2], Gait::Walk);
        assert_eq!(gaits(&[UP, 0, DOWN, 0, UP])[4], Gait::Walk);
        assert_eq!(gaits(&[UP, 0, UP | RIGHT])[2], Gait::Walk);
        assert_eq!(gaits(&[UP, UP, UP])[2], Gait::Walk);
    }
}
