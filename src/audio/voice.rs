use super::dsp::{UNIT, bipolar, decay_mul, mulc, mulq, octave_mul, q, sine, svf_coef, svf_max, unit, unq, white_at};
use crate::fx::Rng;

#[derive(Clone, Copy, Debug)]
pub struct Patch {
    pub delay: f32,
    pub attack: f32,
    pub hold: f32,
    pub decay: f32,
    pub gain: f32,
    pub noise: f32,
    pub grain: f32,
    pub tone: f32,
    pub dry: f32,
    pub buzz: f32,
    pub freq: f32,
    pub glide: f32,
    pub jitter: f32,
    pub fm_ratio: f32,
    pub fm_index: f32,
    pub cutoff: f32,
    pub sweep: f32,
    pub damp: f32,
    pub lp: f32,
    pub bp: f32,
    pub hp: f32,
    pub pulses: u8,
    pub gap: f32,
    pub gap_jitter: f32,
    pub vary: f32,
    pub prio: u8,
}

pub const BASE: Patch = Patch {
    delay: 0.0,
    attack: 0.0,
    hold: 0.0,
    decay: 0.1,
    gain: 1.0,
    noise: 0.0,
    grain: 1.0,
    tone: 0.0,
    dry: 0.0,
    buzz: 0.0,
    freq: 100.0,
    glide: 0.0,
    jitter: 0.0,
    fm_ratio: 1.0,
    fm_index: 0.0,
    cutoff: 1000.0,
    sweep: 0.0,
    damp: 1.0,
    lp: 0.0,
    bp: 1.0,
    hp: 0.0,
    pulses: 1,
    gap: 0.1,
    gap_jitter: 0.0,
    vary: 0.0,
    prio: 1,
};

#[derive(Clone, Copy, Debug)]
pub struct Place {
    pub left: f32,
    pub right: f32,
    pub muffle: f32,
}

pub const PANS: usize = 5;
pub const SLOTS: usize = PANS * 3;

impl Place {
    pub const CENTER: Place = Place { left: 0.7071, right: 0.7071, muffle: 0.0 };

    pub fn slot(&self) -> (usize, f32) {
        let (l2, r2) = (self.left * self.left, self.right * self.right);
        let level = (l2 + r2).sqrt();
        let pan = if level > 1e-6 { (r2 - l2) / (l2 + r2) } else { 0.0 };
        let p = (((pan + 1.0) * 0.5 * (PANS - 1) as f32).round() as usize).min(PANS - 1);
        let wet = match self.muffle {
            m if m < 0.25 => 0,
            m if m < 0.75 => 1,
            _ => 2,
        };
        (wet * PANS + p, level)
    }
}

pub fn slot_gains(slot: usize) -> [i32; 4] {
    let p = (slot % PANS) as f32 / (PANS - 1) as f32 * 2.0 - 1.0;
    let (l, r) = (((1.0 - p) * 0.5).sqrt(), ((1.0 + p) * 0.5).sqrt());
    let wet = [0.0, 0.5, 1.0][slot / PANS];
    [l * (1.0 - wet), r * (1.0 - wet), l * wet, r * wet].map(q)
}

#[derive(Clone, Copy)]
pub struct Voice {
    live: bool,
    prio: u8,
    gain: f32,
    noise: i32,
    grain: u32,
    tone: i32,
    dry: i32,
    buzz: i32,
    fm_ratio: f32,
    fm_scale: f32,
    lpg: i32,
    bpg: i32,
    hpg: i32,
    damp: f32,
    jitter: f32,
    wait: u32,
    t: u32,
    attack_n: u32,
    sustain_n: u32,
    decay: i32,
    decay_sub: i32,
    env: i32,
    peak: f32,
    vary: f32,
    inc0: f32,
    inc: f32,
    inc_mul: f32,
    phase: u32,
    mphase: u32,
    jit: f32,
    f0: f32,
    f: f32,
    f_mul: f32,
    f_max: f32,
    lp: i32,
    bp: i32,
    pulses: u8,
    next: u32,
    gap_n: f32,
    gap_jitter: f32,
    rng: Rng,
    tap: u32,
    fm: bool,
    slot: u8,
}

const GRAIN_TAP: u32 = 0x2000;
const TWO32: f32 = 4_294_967_296.0;
const SUB: usize = 32;

impl Voice {
    pub const IDLE: Voice = Voice {
        live: false,
        prio: 0,
        gain: 0.0,
        noise: 0,
        grain: u32::MAX,
        tone: 0,
        dry: 0,
        buzz: 0,
        fm_ratio: 1.0,
        fm_scale: 0.0,
        lpg: 0,
        bpg: 0,
        hpg: 0,
        damp: 1.0,
        jitter: 0.0,
        wait: 0,
        t: 0,
        attack_n: 1,
        sustain_n: 1,
        decay: 0,
        decay_sub: 0,
        env: 0,
        peak: 1.0,
        vary: 0.0,
        inc0: 0.0,
        inc: 0.0,
        inc_mul: 1.0,
        phase: 0,
        mphase: 0,
        jit: 0.0,
        f0: 0.1,
        f: 0.1,
        f_mul: 1.0,
        f_max: 1.0,
        lp: 0,
        bp: 0,
        pulses: 0,
        next: 0,
        gap_n: 0.0,
        gap_jitter: 0.0,
        rng: Rng::new(1),
        tap: 0,
        fm: false,
        slot: 0,
    };

    pub fn start(&mut self, p: &Patch, place: Place, seed: u32, rate: f32) {
        let samples = |secs: f32| (secs.max(0.0) * rate) as u32;
        let attack_n = samples(p.attack).max(1);
        let mut rng = Rng::new(seed);
        let peak = 1.0 - p.vary * unit(&mut rng);
        let inc0 = (p.freq / rate).clamp(1e-6, 0.45);
        let f0 = svf_coef(p.cutoff, rate, p.damp);
        let control = rate / SUB as f32;
        let (slot, level) = place.slot();
        let gain = p.gain * level;
        *self = Voice {
            live: true,
            prio: p.prio,
            gain,
            noise: q(p.noise),
            grain: (p.grain.clamp(0.0, 1.0) as f64 * u32::MAX as f64) as u32,
            tone: q(p.tone),
            dry: q(p.dry),
            buzz: (p.buzz * UNIT as f32) as i32,
            fm_ratio: p.fm_ratio,
            fm_scale: p.fm_index / gain.max(1e-6),
            lpg: q(p.lp),
            bpg: q(p.bp),
            hpg: q(p.hp),
            damp: p.damp,
            jitter: p.jitter,
            wait: samples(p.delay),
            t: 0,
            attack_n,
            sustain_n: attack_n + samples(p.hold),
            decay: q(decay_mul(p.decay, rate)),
            decay_sub: q(decay_mul(p.decay, rate).powi(SUB as i32)),
            env: 0,
            peak,
            vary: p.vary,
            inc0,
            inc: inc0,
            inc_mul: octave_mul(p.glide, control),
            phase: 0,
            mphase: 0,
            jit: 0.0,
            f0,
            f: f0,
            f_mul: octave_mul(p.sweep, control),
            f_max: svf_max(p.damp),
            lp: 0,
            bp: 0,
            pulses: p.pulses.saturating_sub(1),
            next: 0,
            gap_n: p.gap * rate,
            gap_jitter: p.gap_jitter,
            tap: rng.next_u32(),
            rng,
            fm: p.fm_index != 0.0,
            slot: slot as u8,
        };
        self.next = self.gap();
    }

    fn gap(&mut self) -> u32 {
        (self.gap_n * (1.0 + self.gap_jitter * bipolar(&mut self.rng))).max(1.0) as u32
    }

    pub fn live(&self) -> bool {
        self.live
    }

    pub fn prio(&self) -> u8 {
        self.prio
    }

    #[cfg(test)]
    pub fn sane(&self) -> bool {
        [self.inc, self.f, self.jit, self.peak, self.gain].iter().all(|x| x.is_finite())
    }

    pub fn loudness(&self) -> f32 {
        let level = match self.wait > 0 || self.t < self.sustain_n {
            true => self.peak * self.gain,
            false => unq(self.env),
        };
        level
    }

    fn finished(&self) -> bool {
        self.pulses == 0 && self.t > self.sustain_n && unq(self.env) < 2e-5
    }

    fn stage(&mut self, len: usize) -> i32 {
        if self.pulses > 0 && self.t >= self.next {
            self.pulses -= 1;
            self.t = 0;
            self.env = 0;
            self.peak = 1.0 - self.vary * unit(&mut self.rng);
            self.inc = self.inc0;
            self.f = self.f0;
            self.next = self.gap();
        }
        let top = q(self.peak * self.gain);
        match self.t {
            t if t < self.attack_n => (top - self.env) / (self.attack_n - t).max(len as u32) as i32,
            t if t < self.sustain_n => {
                self.env = top;
                0
            }
            _ => {
                let end = match len == SUB {
                    true => mulc(self.env, self.decay_sub),
                    false => (0..len).fold(self.env, |e, _| mulc(e, self.decay)),
                };
                (end - self.env) / len as i32
            }
        }
    }

    fn bend(&mut self) -> (u32, u32) {
        let now = match self.jitter != 0.0 {
            true => {
                self.jit += 0.128 * (bipolar(&mut self.rng) - self.jit);
                (self.inc * (1.0 + self.jitter * self.jit)).clamp(1e-6, 0.45)
            }
            false => self.inc,
        };
        self.inc = (self.inc * self.inc_mul).clamp(1e-6, 0.45);
        self.f = (self.f * self.f_mul).clamp(1e-4, self.f_max);
        ((now * TWO32) as u32, ((now * self.fm_ratio).min(0.45) * TWO32) as u32)
    }

    pub fn slot(&self) -> usize {
        self.slot as usize
    }

    pub fn render(&mut self, out: &mut [i32]) {
        let skip = (self.wait as usize).min(out.len());
        self.wait -= skip as u32;
        let out = &mut out[skip..];
        let mut v = *self;
        let noisy = v.noise != 0;
        let tonal = v.tone != 0 || v.dry != 0;
        let filtered = v.lpg != 0 || v.bpg != 0 || v.hpg != 0;
        match (noisy, tonal, filtered) {
            (true, true, true) => v.run::<true, true, true>(out),
            (true, true, false) => v.run::<true, true, false>(out),
            (true, false, true) => v.run::<true, false, true>(out),
            (true, false, false) => v.run::<true, false, false>(out),
            (false, true, true) => v.run::<false, true, true>(out),
            (false, true, false) => v.run::<false, true, false>(out),
            (false, false, true) => v.run::<false, false, true>(out),
            (false, false, false) => v.run::<false, false, false>(out),
        }
        v.live = !v.finished();
        *self = v;
    }

    fn run<const NOISE: bool, const TONE: bool, const FILTER: bool>(&mut self, mono: &mut [i32]) {
        let grainy = self.grain < u32::MAX;
        let fm = self.fm;
        let buzzes = self.buzz != 0;
        let (gn, gt, gd, gb) = (self.noise, self.tone, self.dry, self.buzz);
        let (glp, gbp, ghp) = (self.lpg, self.bpg, self.hpg);
        let (has_lp, has_hp) = (glp != 0, ghp != 0);
        for chunk in mono.chunks_mut(SUB) {
            let step = self.stage(chunk.len());
            let (inc, minc) = self.bend();
            let f = q(self.f);
            let fd = q(self.f + self.damp);
            let c1 = q(1.0 - self.f * (self.f + self.damp));
            let fm_amt = (self.fm_scale * unq(self.env) * 131_072.0) as i32;
            let (mut env, mut lp, mut bp) = (self.env, self.lp, self.bp);
            let (mut phase, mut mphase, mut tap) = (self.phase, self.mphase, self.tap);
            for out in chunk.iter_mut() {
                env += step;
                let mut src = 0;
                if NOISE {
                    let n = white_at(tap);
                    tap = tap.wrapping_add(1);
                    src = match grainy && (white_at(tap ^ GRAIN_TAP) as u32) << 16 > self.grain {
                        true => 0,
                        false => mulq(n, gn),
                    };
                }
                let mut y = 0;
                if TONE {
                    let offset = match fm {
                        true => sine(mphase).wrapping_mul(fm_amt) as u32,
                        false => 0,
                    };
                    let s = sine(phase.wrapping_add(offset));
                    src += mulq(s, gt);
                    y = mulq(s, gd);
                    mphase = mphase.wrapping_add(minc);
                }
                if TONE || buzzes {
                    let (next, wrapped) = phase.overflowing_add(inc);
                    phase = next;
                    if buzzes && wrapped {
                        src += gb;
                    }
                }
                if FILTER {
                    let high = match has_hp {
                        true => mulq(src - lp - mulq(bp, fd), ghp),
                        false => 0,
                    };
                    let bp_next = mulq(bp, c1) + mulq(src - lp, f);
                    lp += mulq(bp, f);
                    bp = bp_next;
                    y += mulq(bp, gbp) + high;
                    if has_lp {
                        y += mulq(lp, glp);
                    }
                }
                *out += mulq(y, env);
            }
            (self.env, self.lp, self.bp, self.phase, self.mphase, self.tap) = (env, lp, bp, phase, mphase, tap);
            self.t += chunk.len() as u32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(p: &Patch, secs: f32) -> (Vec<i32>, bool) {
        let mut v = Voice::IDLE;
        v.start(p, Place::CENTER, 99, 32000.0);
        let mut mono = vec![0; (secs * 32000.0) as usize];
        for c in mono.chunks_mut(1024) {
            if v.live() {
                v.render(c);
            }
        }
        (mono.iter().flat_map(|y| [*y, *y]).collect(), v.live())
    }

    #[test]
    fn click_decays_and_ends() {
        let (mix, live) = run(&Patch { noise: 1.0, cutoff: 3000.0, damp: 0.5, decay: 0.03, ..BASE }, 1.0);
        assert!(!live);
        assert!(mix.iter().any(|x| x.abs() > UNIT / 100));
        assert!(mix[mix.len() - 2000..].iter().all(|x| *x == 0));
    }

    #[test]
    fn delay_holds_silence_then_sounds() {
        let (mix, _) = run(&Patch { delay: 0.1, tone: 1.0, freq: 440.0, decay: 0.2, ..BASE }, 0.5);
        assert!(mix[..6000].iter().all(|x| *x == 0));
        assert!(mix[6400..9000].iter().any(|x| x.abs() > UNIT / 20));
    }

    #[test]
    fn pulses_retrigger() {
        let p = Patch { noise: 1.0, decay: 0.01, pulses: 3, gap: 0.2, cutoff: 2000.0, ..BASE };
        let (mix, live) = run(&p, 1.0);
        assert!(!live);
        let burst = |at: f32| mix[(at * 32000.0) as usize * 2..][..400].iter().any(|x| x.abs() > UNIT / 100);
        assert!(burst(0.0) && burst(0.2) && burst(0.4) && !burst(0.6));
    }

    #[test]
    fn resonant_filter_stays_bounded() {
        let p = Patch { noise: 1.0, cutoff: 7000.0, damp: 0.05, bp: 1.0, hold: 1.0, decay: 0.1, ..BASE };
        let (mix, _) = run(&p, 1.0);
        assert!(mix.iter().all(|x| x.abs() < 64 * UNIT));
    }
}
