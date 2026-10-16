use super::dsp::UNIT;

const COMBS: [usize; 4] = [1117, 1277, 1423, 1559];
const PASSES: [usize; 3] = [347, 113, 131];
const CAP: usize = 1560;
const PASS_CAP: usize = 348;
const FEEDBACK: i32 = (0.8 * UNIT as f32) as i32;
const KEEP: i32 = (0.65 * UNIT as f32) as i32;
const PASS_G: i32 = UNIT / 2;
const TAIL: usize = 48_000;

fn m(a: i32, b: i32) -> i32 {
    ((a as i64 * b as i64) >> 15) as i32
}

#[derive(Clone, Copy)]
struct Line<const N: usize> {
    buf: [i32; N],
    len: usize,
    at: usize,
}

impl<const N: usize> Line<N> {
    fn new(len: usize) -> Line<N> {
        Line { buf: [0; N], len: len.min(N), at: 0 }
    }

    fn swap(&mut self, x: i32) -> i32 {
        let y = std::mem::replace(&mut self.buf[self.at], x);
        self.at = if self.at + 1 == self.len { 0 } else { self.at + 1 };
        y
    }

    fn peek(&self) -> i32 {
        self.buf[self.at]
    }

    fn pass(&mut self, x: i32) -> i32 {
        let y = self.peek() - m(x, PASS_G);
        self.swap(x + m(y, PASS_G));
        y
    }
}

pub struct Verb {
    combs: [Line<CAP>; 4],
    lows: [i32; 4],
    passes: [Line<PASS_CAP>; 3],
    wet: i32,
    goal: i32,
    live: bool,
    tail: usize,
}

impl Verb {
    pub fn new() -> Verb {
        Verb { combs: COMBS.map(Line::new), lows: [0; 4], passes: PASSES.map(Line::new), wet: 0, goal: 0, live: false, tail: 0 }
    }

    pub fn aim(&mut self, wet: f32) {
        self.goal = (wet.clamp(0.0, 1.0) * UNIT as f32) as i32;
        if self.goal > 0 && !self.live {
            self.combs.iter_mut().for_each(|c| c.buf.fill(0));
            self.passes.iter_mut().for_each(|p| p.buf.fill(0));
            self.lows = [0; 4];
            self.live = true;
        }
    }

    pub fn process(&mut self, mix: &mut [i32]) {
        if !self.live {
            return;
        }
        self.wet += (self.goal - self.wet).clamp(-256, 256);
        let wet = self.wet;
        let (combs, lows, passes) = (&mut self.combs, &mut self.lows, &mut self.passes);
        mix.chunks_exact_mut(2).for_each(|f| {
            let x = m((f[0] >> 1) + (f[1] >> 1), wet) >> 2;
            let sum = combs.iter_mut().zip(lows.iter_mut()).fold(0, |s, (c, lp)| {
                let y = c.peek();
                *lp = y + m(*lp - y, KEEP);
                c.swap(x + m(*lp, FEEDBACK));
                s + y
            });
            let [a, b, c] = passes;
            let common = a.pass(sum >> 1);
            f[0] += b.pass(common);
            f[1] += c.pass(common);
        });
        self.tail = if wet > 0 { TAIL } else { self.tail.saturating_sub(mix.len() / 2) };
        self.live = self.goal > 0 || self.wet > 0 || self.tail > 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_rings_decays_and_stays_off_when_dry() {
        let mut v = Verb::new();
        let mut dry = vec![0i32; 4000];
        dry[0] = UNIT;
        let mut idle = dry.clone();
        v.process(&mut idle);
        assert_eq!(idle, dry);
        v.aim(1.0);
        (0..600).for_each(|_| v.process(&mut [0; 64]));
        let mut s = vec![0i32; 32000 * 2];
        s[0] = UNIT;
        s[1] = UNIT;
        v.process(&mut s);
        let energy = |c: &[i32]| c.iter().map(|x| (*x as i64).pow(2)).sum::<i64>();
        let early = energy(&s[2..8000]);
        let late = energy(&s[56000..64000]);
        assert!(early > 0 && late < early / 20, "{early} {late}");
        assert!(s[2..].iter().all(|x| x.abs() <= UNIT / 2));
        v.aim(0.0);
        (0..2000).for_each(|_| v.process(&mut [0; 64]));
        let mut after = dry.clone();
        v.process(&mut after);
        assert_eq!(after, dry);
    }
}
