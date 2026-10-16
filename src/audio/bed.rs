use super::dsp::{UNIT, bipolar, cycles, decay_mul, hum, mul15, noise, one_pole, sine, sine_f, svf_coef};
use super::voice::Place;
use crate::fx::Rng;

const CONTROL: usize = 32;
const SILENT: f32 = 1e-5;
const LOOP: usize = 1 << 15;
const LOOP_RMS: f32 = 0.3;
const GAIN_BITS: u32 = 12;
const GAIN_ONE: i32 = 1 << GAIN_BITS;

#[derive(Clone, Copy, Default, Debug)]
pub struct Layer {
    gl: f32,
    gr: f32,
    tl: f32,
    tr: f32,
    rise: f32,
    fall: f32,
    muffle: f32,
    on: bool,
}

impl Layer {
    fn new(rise: f32, fall: f32) -> Layer {
        Layer { rise, fall, ..Layer::default() }
    }

    pub fn aim(&mut self, place: Place, level: f32) {
        self.tl = place.left * level;
        self.tr = place.right * level;
        self.muffle = place.muffle.clamp(0.0, 1.0);
    }

    fn glide(&mut self) {
        let k = |cur: f32, to: f32| if to > cur { self.rise } else { self.fall };
        self.gl += (self.tl - self.gl) * k(self.gl, self.tl);
        self.gr += (self.tr - self.gr) * k(self.gr, self.tr);
        self.on = self.gl + self.gr + self.tl + self.tr > SILENT;
    }

    fn gains(&self) -> [i32; 4] {
        let g = |x: f32| (x * GAIN_ONE as f32) as i32;
        match self.on {
            true => [g(self.gl * (1.0 - self.muffle)), g(self.gr * (1.0 - self.muffle)), g(self.gl * self.muffle), g(self.gr * self.muffle)],
            false => [0; 4],
        }
    }

    fn mix(&self, main: &mut [i32], bus: &mut [i32], mut source: impl FnMut() -> i32) {
        let g = self.gains();
        main.chunks_exact_mut(2).zip(bus.chunks_exact_mut(2)).for_each(|(d, m)| {
            let x = source();
            d[0] += (x * g[0]) >> GAIN_BITS;
            d[1] += (x * g[1]) >> GAIN_BITS;
            m[0] += (x * g[2]) >> GAIN_BITS;
            m[1] += (x * g[3]) >> GAIN_BITS;
        });
    }
}

fn noise_loop(seed: u32, mut filter: impl FnMut(f32) -> f32) -> Box<[i16; LOOP]> {
    let mut rng = Rng::new(seed);
    let white: Vec<f32> = (0..LOOP).map(|_| bipolar(&mut rng)).collect();
    let mut raw = vec![0.0f32; LOOP];
    (0..2).for_each(|_| raw.iter_mut().zip(&white).for_each(|(o, x)| *o = filter(*x)));
    let rms = (raw.iter().map(|x| x * x).sum::<f32>() / LOOP as f32).sqrt().max(1e-9);
    let pcm: Box<[i16]> = raw.iter().map(|x| (x * LOOP_RMS / rms * UNIT as f32).clamp(-32767.0, 32767.0) as i16).collect();
    pcm.try_into().unwrap_or_else(|_| unreachable!())
}

fn band(hz: f32, damp: f32, rate: f32) -> impl FnMut(f32) -> f32 {
    let f = svf_coef(hz, rate, damp);
    let (mut lp, mut bp) = (0.0, 0.0);
    move |x| {
        lp += f * bp;
        bp += f * (x - lp - damp * bp);
        bp
    }
}

fn high(hz: f32, rate: f32) -> impl FnMut(f32) -> f32 {
    let a = one_pole(hz, rate);
    let mut lp = 0.0;
    move |x| {
        lp += a * (x - lp);
        x - lp
    }
}

fn low2(hz: f32, rate: f32) -> impl FnMut(f32) -> f32 {
    let a = one_pole(hz, rate);
    let (mut l0, mut l1) = (0.0, 0.0);
    move |x| {
        l0 += a * (x - l0);
        l1 += a * (l0 - l1);
        l1
    }
}

pub struct Bed {
    rate: f32,
    rng: Rng,
    pub hum: Layer,
    pub clock: Layer,
    pub wind: Layer,
    pub tv: Layer,
    pub tone: Layer,
    pub drone: Layer,
    tension: f32,
    wind_lo: Box<[i16; LOOP]>,
    wind_hi: Box<[i16; LOOP]>,
    static_: Box<[i16; LOOP]>,
    rumble: Box<[i16; LOOP]>,
    pos: [usize; 3],
    hum_phase: u32,
    hum_inc: u32,
    tick_t: u32,
    tick_n: u32,
    tick_left: usize,
    tick_env: i32,
    tick_decay: i32,
    tick_f: i32,
    tick_lp: i32,
    tick_bp: i32,
    tock: bool,
    gust: [u32; 2],
    gust_inc: [u32; 2],
    wind_mix: [i32; 2],
    drone_phase: [u32; 5],
    drone_inc: [u32; 5],
    drone_lfo: u32,
    lfo_inc: u32,
    swell: i32,
}

impl Bed {
    pub fn new(rate: f32) -> Bed {
        let per_control = |secs: f32| 1.0 - (-(CONTROL as f32) / (secs * rate)).exp();
        let drone_hz = [41.2, 41.7, 61.9, 87.3, 92.5];
        let q15 = |x: f32| (x * UNIT as f32) as i32;
        Bed {
            rate,
            rng: Rng::new(0x5EED_A0D1),
            hum: Layer::new(per_control(0.4), per_control(0.006)),
            clock: Layer::new(per_control(0.3), per_control(0.3)),
            wind: Layer::new(per_control(0.8), per_control(0.8)),
            tv: Layer::new(per_control(0.5), per_control(0.006)),
            tone: Layer::new(per_control(1.0), per_control(1.0)),
            drone: Layer::new(per_control(2.5), per_control(4.0)),
            tension: 0.0,
            wind_lo: noise_loop(11, band(300.0, 0.35, rate)),
            wind_hi: noise_loop(12, band(760.0, 0.35, rate)),
            static_: noise_loop(13, high(2500.0, rate)),
            rumble: noise_loop(14, low2(140.0, rate)),
            pos: [0, LOOP / 3, 2 * LOOP / 3],
            hum_phase: 0,
            hum_inc: cycles(60.0, rate),
            tick_t: 0,
            tick_n: rate as u32,
            tick_left: 0,
            tick_env: 0,
            tick_decay: q15(decay_mul(0.012, rate)),
            tick_f: 0,
            tick_lp: 0,
            tick_bp: 0,
            tock: false,
            gust: [0, 0x5EB8_51EC],
            gust_inc: [cycles(0.071 * CONTROL as f32, rate), cycles(0.13 * CONTROL as f32, rate)],
            wind_mix: [0; 2],
            drone_phase: [0; 5],
            drone_inc: drone_hz.map(|f| cycles(f, rate)),
            drone_lfo: 0,
            lfo_inc: cycles(0.11 * CONTROL as f32, rate),
            swell: 0,
        }
    }

    pub fn set_tension(&mut self, t: f32) {
        self.tension = t;
    }

    fn control(&mut self) {
        [&mut self.hum, &mut self.clock, &mut self.wind, &mut self.tv, &mut self.tone, &mut self.drone]
            .into_iter()
            .for_each(Layer::glide);
        let q15 = |x: f32| (x * UNIT as f32) as i32;
        self.gust = [self.gust[0].wrapping_add(self.gust_inc[0]), self.gust[1].wrapping_add(self.gust_inc[1])];
        let g = 0.5 + 0.3 * sine_f(self.gust[0]) + 0.2 * sine_f(self.gust[1]);
        let level = 0.6 + 0.4 * sine_f(self.gust[0]);
        self.wind_mix = [q15((1.0 - g) * level), q15(g * level)];
        self.drone_lfo = self.drone_lfo.wrapping_add(self.lfo_inc);
        self.swell = q15(0.75 + 0.25 * sine_f(self.drone_lfo));
        self.tick_t += CONTROL as u32;
        if self.tick_t >= self.tick_n {
            self.tick_t -= self.tick_n;
            self.tock = !self.tock;
            self.tick_env = UNIT;
            self.tick_left = (self.rate * 0.06) as usize;
            self.tick_f = q15(svf_coef(if self.tock { 2100.0 } else { 2800.0 }, self.rate, 0.12));
        }
    }

    fn block(&mut self, block: &mut [i32], bus: &mut [i32]) {
        let frames = block.len() / 2;
        if self.clock.on && self.tick_left > 0 {
            let n = frames.min(self.tick_left);
            self.tick_left -= n;
            let rng = &mut self.rng;
            let (f, decay, damp) = (self.tick_f, self.tick_decay, 3932);
            let (mut env, mut lp, mut bp) = (self.tick_env, self.tick_lp, self.tick_bp);
            self.clock.mix(&mut block[..n * 2], &mut bus[..n * 2], || {
                let x = (noise(rng) * (env >> 8)) >> 10;
                env = mul15(env, decay);
                lp += mul15(f, bp);
                bp += mul15(f, x - lp - mul15(damp, bp));
                bp << 3
            });
            (self.tick_env, self.tick_lp, self.tick_bp) = (env, lp, bp);
        }
        let (hg, wg, tg, rg, dg) = (self.hum.gains(), self.wind.gains(), self.tv.gains(), self.tone.gains(), self.drone.gains());
        let (hum_on, wind_on, tv_on, tone_on, drone_on) = (self.hum.on, self.wind.on, self.tv.on, self.tone.on, self.drone.on);
        if !(hum_on || wind_on || tv_on || tone_on || drone_on) {
            return;
        }
        let ([lo, hi], swell, rub) = (self.wind_mix, self.swell, (self.tension * 0.18 * UNIT as f32) as i32);
        let (lo_loop, hi_loop, static_, rumble) = (&*self.wind_lo, &*self.wind_hi, &*self.static_, &*self.rumble);
        let (hum_inc, drone_inc) = (self.hum_inc, self.drone_inc);
        let (mut hp, mut dp, mut pos) = (self.hum_phase, self.drone_phase, self.pos);
        block.chunks_exact_mut(2).zip(bus.chunks_exact_mut(2)).for_each(|(d, m)| {
            let mut acc = [0i32; 4];
            let mut add = |x: i32, g: &[i32; 4]| acc.iter_mut().zip(g).for_each(|(a, g)| *a += x * g);
            pos = pos.map(|p| (p + 1) & (LOOP - 1));
            if hum_on {
                hp = hp.wrapping_add(hum_inc);
                add(hum(hp), &hg);
            }
            if wind_on {
                add(mul15(lo_loop[pos[0]] as i32, lo) + mul15(hi_loop[pos[0]] as i32, hi), &wg);
            }
            if tv_on {
                add(static_[pos[1]] as i32, &tg);
            }
            if tone_on {
                add(rumble[pos[2]] as i32, &rg);
            }
            if drone_on {
                let low = (sine(dp[0]) * 13 + sine(dp[1]) * 13 + sine(dp[2]) * 8) >> 5;
                dp = std::array::from_fn(|i| dp[i].wrapping_add(drone_inc[i]));
                add(mul15(low + mul15(sine(dp[3]) + sine(dp[4]), rub), swell), &dg);
            }
            d[0] += acc[0] >> GAIN_BITS;
            d[1] += acc[1] >> GAIN_BITS;
            m[0] += acc[2] >> GAIN_BITS;
            m[1] += acc[3] >> GAIN_BITS;
        });
        (self.hum_phase, self.drone_phase, self.pos) = (hp, dp, pos);
    }

    pub fn render(&mut self, mix: &mut [i32], bus: &mut [i32]) {
        for (block, bus) in mix.chunks_mut(CONTROL * 2).zip(bus.chunks_mut(CONTROL * 2)) {
            self.control();
            self.block(block, bus);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_loops_are_normalized_and_seamless() {
        let l = noise_loop(3, low2(140.0, 16000.0));
        let rms = (l.iter().map(|x| (*x as f32).powi(2)).sum::<f32>() / LOOP as f32).sqrt() / UNIT as f32;
        assert!((rms - LOOP_RMS).abs() < 0.02, "{rms}");
        let step = |a: i16, b: i16| (a as i32 - b as i32).abs();
        let worst = l.windows(2).map(|w| step(w[0], w[1])).max().unwrap_or(0);
        assert!(step(l[LOOP - 1], l[0]) <= worst);
    }

    #[test]
    fn layers_fade_in_and_stay_bounded() {
        let mut bed = Bed::new(16000.0);
        bed.set_tension(1.0);
        [&mut bed.hum, &mut bed.clock, &mut bed.wind, &mut bed.tv, &mut bed.tone, &mut bed.drone]
            .into_iter()
            .for_each(|l| l.aim(Place { muffle: 0.5, ..Place::CENTER }, 0.1));
        let mut mix = vec![0; 32000 * 2];
        let mut bus = vec![0; 32000 * 2];
        bed.render(&mut mix, &mut bus);
        assert!(bus.iter().all(|x| x.abs() < UNIT / 2));
        assert!(mix.iter().all(|x| x.abs() < UNIT / 2));
        assert!(mix[50_000..].iter().any(|x| x.abs() > UNIT / 100));
    }

    #[test]
    fn hum_cuts_fast() {
        let mut bed = Bed::new(16000.0);
        bed.hum.aim(Place::CENTER, 0.2);
        let mut mix = vec![0; 16000 * 2];
        let mut bus = vec![0; 16000 * 2];
        bed.render(&mut mix, &mut bus);
        assert!(mix.iter().any(|x| x.abs() > UNIT / 100));
        bed.hum.aim(Place::CENTER, 0.0);
        let mut tail = vec![0; 1600 * 2];
        let mut bus = vec![0; 1600 * 2];
        bed.render(&mut tail, &mut bus);
        assert!(tail[1600..].iter().all(|x| x.abs() < UNIT / 1000));
    }
}
