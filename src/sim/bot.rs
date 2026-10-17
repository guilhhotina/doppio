use super::{World, MAX_PLAYERS, TICS_PER_SEC};
use crate::fx::Rng;

mod nav;
mod pilot;
mod plan;
#[cfg(test)]
mod tests;

use pilot::{Pilot, Shared};

#[cfg(test)]
pub const LIMIT: u32 = 25 * 60 * TICS_PER_SEC;
const START_DELAY: u32 = 10 * TICS_PER_SEC;

pub struct Bot {
    pilots: Vec<Pilot>,
    shared: Shared,
}

impl Bot {
    pub fn new(w: &World, seed: u32) -> Bot {
        let mut rng = Rng::new(seed ^ 0x5bd1_e995);
        let pilots = (0..w.count).map(|p| Pilot::new(w, p, rng.below(START_DELAY))).collect();
        Bot { pilots, shared: Shared { known: vec![false; w.locks.len()], jobs: [None; MAX_PLAYERS], rng, log: Vec::new(), trace: Vec::new() } }
    }

    pub fn inputs(&mut self, w: &World) -> [u16; MAX_PLAYERS] {
        let mut out = [0; MAX_PLAYERS];
        self.pilots.iter_mut().enumerate().for_each(|(p, pilot)| {
            out[p] = pilot.tick(w, &mut self.shared);
            if self.shared.jobs[p] != pilot.job() {
                let pl = w.players[p];
                let line = format!("t{} p{} {:?} at {},{} {:?}", w.tic, p, pilot.job(), pl.pos.x.floor(), pl.pos.y.floor(), w.legend.mode);
                self.shared.trace.push(line);
            }
            self.shared.jobs[p] = pilot.job();
        });
        out
    }

    pub fn drain(&mut self) -> impl Iterator<Item = String> + '_ {
        self.shared.log.drain(..).chain(self.shared.trace.drain(..))
    }
}
