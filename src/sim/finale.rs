use super::night::at_hour;
use super::tank::MOVE;
use super::{Phase, World, MAX_BODIES};

pub const HOLD_TICS: u16 = 150;
pub const OPENS_AFTER_HOURS: u32 = 6;

impl World {
    fn inside_final(&self, i: usize) -> bool {
        let p = self.bodies[i].pos;
        self.map.final_area.contains(p.x.floor(), p.y.floor())
    }

    fn vigil(&self, drive: &[u16; MAX_BODIES]) -> bool {
        let mut living = (0..self.players_n as usize).filter(|&p| self.living(p)).map(|p| self.players[p].body as usize).peekable();
        let late = self.phase_tic >= at_hour(self.night_tics, OPENS_AFTER_HOURS);
        late && living.peek().is_some() && living.all(|b| self.inside_final(b) && !self.bodies[b].lantern && drive[b] & MOVE == 0)
    }

    pub(super) fn finale(&mut self, drive: &[u16; MAX_BODIES]) {
        if !self.exit_open {
            self.still = match self.vigil(drive) {
                true => self.still + 1,
                false => 0,
            };
            if self.still >= HOLD_TICS {
                self.exit_open = true;
                self.open_door(self.exit_door as usize);
            }
        }
        let exit = (self.map.exit.x.floor(), self.map.exit.y.floor());
        let out = self.living_bodies().any(|b| (b.pos.x.floor(), b.pos.y.floor()) == exit);
        if self.exit_open && out {
            self.phase = Phase::Won;
        }
    }
}
