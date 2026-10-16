use crate::sim::TICS_PER_SEC;
use std::time::{Duration, Instant};

pub const TIC: Duration = Duration::from_nanos(1_000_000_000 / TICS_PER_SEC as u64);
const MAX_CATCHUP: u32 = 5;

pub struct Clock {
    last: Instant,
    acc: Duration,
}

impl Clock {
    pub fn new(now: Instant) -> Clock {
        Clock { last: now, acc: Duration::ZERO }
    }

    pub fn hold(&mut self, now: Instant) {
        self.last = now;
        self.acc = Duration::ZERO;
    }

    pub fn run(&mut self, now: Instant, mut step: impl FnMut() -> bool) -> u32 {
        self.acc = (self.acc + now.saturating_duration_since(self.last)).min(TIC * MAX_CATCHUP);
        self.last = now;
        let mut done = 0;
        while self.acc >= TIC {
            if !step() {
                self.acc = self.acc.min(TIC);
                break;
            }
            self.acc -= TIC;
            done += 1;
        }
        done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_due_tics_and_caps_on_stall() {
        let t0 = Instant::now();
        let mut c = Clock::new(t0);
        assert_eq!(c.run(t0 + TIC * 2, || true), 2);
        assert_eq!(c.run(t0 + TIC * 20, || true), MAX_CATCHUP);
        assert_eq!(c.run(t0 + TIC * 40, || false), 0);
        assert_eq!(c.run(t0 + TIC * 40, || true), 1);
        c.hold(t0 + TIC * 90);
        assert_eq!(c.run(t0 + TIC * 90, || true), 0);
    }
}
