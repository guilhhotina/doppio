use super::dsp::{cycles, decay_mul, sine_f};

const LEN: usize = 1024;
const MASK: usize = LEN - 1;
const SUB: usize = 16;

pub struct Warble {
    buf: [[i32; 2]; LEN],
    w: usize,
    goal: f32,
    depth: f32,
    fall: f32,
    rise: f32,
    span: f32,
    delay: f32,
    lfo: [u32; 2],
    inc: [u32; 2],
}

impl Warble {
    pub fn new(rate: f32) -> Warble {
        Warble {
            buf: [[0; 2]; LEN],
            w: 0,
            goal: 0.0,
            depth: 0.0,
            fall: decay_mul(1.6, rate / SUB as f32),
            rise: 1.0 - decay_mul(0.03, rate / SUB as f32),
            span: (0.007 * rate).min((LEN / 2) as f32 - 2.0),
            delay: 0.0,
            lfo: [0; 2],
            inc: [cycles(6.3 * SUB as f32, rate), cycles(0.9 * SUB as f32, rate)],
        }
    }

    pub fn kick(&mut self, amount: f32) {
        self.goal = self.goal.max(amount.clamp(0.0, 1.0));
    }

    fn tap(&self, delay: u32) -> [i32; 2] {
        let i = (delay >> 16) as usize;
        let frac = (delay & 0xFFFF) as i64;
        let a = self.buf[self.w.wrapping_sub(i) & MASK];
        let b = self.buf[self.w.wrapping_sub(i + 1) & MASK];
        let lerp = |a: i32, b: i32| a + (((b - a) as i64 * frac) >> 16) as i32;
        [lerp(a[0], b[0]), lerp(a[1], b[1])]
    }

    fn target(&mut self) -> f32 {
        self.depth += (self.goal - self.depth) * self.rise;
        self.goal *= self.fall;
        let wobble = 1.0 + 0.6 * sine_f(self.lfo[0]) + 0.4 * sine_f(self.lfo[1]);
        self.lfo = [self.lfo[0].wrapping_add(self.inc[0]), self.lfo[1].wrapping_add(self.inc[1])];
        self.depth * self.span * wobble
    }

    pub fn process(&mut self, mix: &mut [i32]) {
        if self.goal + self.depth < 1e-4 {
            mix.chunks_exact(2).for_each(|frame| {
                self.w = (self.w + 1) & MASK;
                self.buf[self.w] = [frame[0], frame[1]];
            });
            return;
        }
        for block in mix.chunks_mut(SUB * 2) {
            let to = self.target();
            let n = (block.len() / 2) as f32;
            let mut d = (self.delay * 65536.0) as i64;
            let step = ((to - self.delay) * 65536.0 / n) as i64;
            self.delay = to;
            block.chunks_exact_mut(2).for_each(|frame| {
                self.w = (self.w + 1) & MASK;
                self.buf[self.w] = [frame[0], frame[1]];
                d += step;
                let out = self.tap(d.max(0) as u32);
                frame[0] = out[0];
                frame[1] = out[1];
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::dsp::{UNIT, sine};
    use super::*;

    #[test]
    fn transparent_when_idle_and_recovers() {
        let mut w = Warble::new(16000.0);
        let inc = cycles(220.0, 16000.0);
        let src: Vec<i32> = (0..32000u32).map(|i| sine(i.wrapping_mul(inc)) / 2).collect();
        let mut a = src.clone();
        w.process(&mut a);
        assert_eq!(a, src);
        w.kick(1.0);
        let mut b = src.clone();
        w.process(&mut b);
        assert!(b.iter().all(|x| x.abs() <= UNIT / 2 + 1));
        assert!(b[..10000].iter().zip(&src).any(|(x, y)| (x - y).abs() > UNIT / 20));
        let mut c = src.clone();
        (0..8).for_each(|_| w.process(&mut c));
        let mut d = src.clone();
        w.process(&mut d);
        assert_eq!(d, src);
    }
}
