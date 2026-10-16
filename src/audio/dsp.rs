use crate::fx::{Ang, Rng};
use std::f32::consts::{PI, TAU};

pub const UNIT: i32 = 1 << 15;
pub const CEIL: f32 = 0.708;
const KNEE: i32 = 19661;
const ROOM: i32 = UNIT - KNEE;
const DRIVE: i32 = 4 * UNIT;
const CEIL_Q15: i32 = (CEIL * 32767.0) as i32;
const ONE_Q: f32 = (1u32 << 30) as f32;
const TABLE: usize = 1024;

const fn wave(harmonics: [i32; 6]) -> [i16; TABLE] {
    let mut t = [0i16; TABLE];
    let mut i = 0;
    while i < TABLE {
        let mut v = 0;
        let mut k = 0;
        while k < 6 {
            v += (Ang(((i << 6) * (k + 1)) as u16).sin().0 >> 1) * harmonics[k];
            k += 1;
        }
        v >>= 5;
        t[i] = if v > 32767 { 32767 } else if v < -32767 { -32767 } else { v as i16 };
        i += 1;
    }
    t
}

static SINE: [i16; TABLE] = wave([32, 0, 0, 0, 0, 0]);
static HUM: [i16; TABLE] = wave([16, 11, 0, 0, 0, 0]);
static BUZZ: [i16; TABLE] = wave([9, 8, 6, 4, 3, 2]);

pub const WHITE_LEN: usize = 1 << 14;

const fn white() -> [i16; WHITE_LEN] {
    let mut t = [0i16; WHITE_LEN];
    let mut x: u32 = 0x2545_F491;
    let mut i = 0;
    while i < WHITE_LEN {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        t[i] = (x >> 16) as i16;
        i += 1;
    }
    t
}

static WHITE: [i16; WHITE_LEN] = white();

pub fn white_at(i: u32) -> i32 {
    WHITE[i as usize & (WHITE_LEN - 1)] as i32
}

pub fn bipolar(rng: &mut Rng) -> f32 {
    (rng.next_u32() as i32) as f32 * (1.0 / 2_147_483_648.0)
}

pub fn unit(rng: &mut Rng) -> f32 {
    (rng.next_u32() >> 8) as f32 * (1.0 / 16_777_216.0)
}

pub fn q(x: f32) -> i32 {
    (x * ONE_Q) as i32
}

pub fn unq(x: i32) -> f32 {
    x as f32 * (1.0 / ONE_Q)
}

pub fn mulq(signal: i32, coef: i32) -> i32 {
    (((signal << 2) as i64 * coef as i64) >> 32) as i32
}

pub fn mulc(a: i32, b: i32) -> i32 {
    ((a as i64 * b as i64) >> 30) as i32
}

pub fn cycles(hz: f32, rate: f32) -> u32 {
    ((hz as f64 / rate as f64).clamp(0.0, 0.45) * 4_294_967_296.0) as u32
}

pub fn sine(ph: u32) -> i32 {
    SINE[(ph >> 22) as usize] as i32
}

pub fn hum(ph: u32) -> i32 {
    HUM[(ph >> 22) as usize] as i32
}

pub fn buzz(ph: u32) -> i32 {
    BUZZ[(ph >> 22) as usize] as i32
}

pub fn mul15(a: i32, b: i32) -> i32 {
    (a * b) >> 15
}

pub fn sine_f(ph: u32) -> f32 {
    sine(ph) as f32 * (1.0 / UNIT as f32)
}

pub fn pcm(x: i32) -> i16 {
    let a = x.clamp(-DRIVE, DRIVE).abs();
    let f = match a < KNEE {
        true => a,
        false => KNEE + ROOM * (a - KNEE) / (ROOM + a - KNEE),
    };
    let y = (f * CEIL_Q15) >> 15;
    (if x < 0 { -y } else { y }) as i16
}

pub fn svf_coef(hz: f32, rate: f32, damp: f32) -> f32 {
    (2.0 * (PI * hz.max(1.0) / rate).min(PI / 2.0).sin()).min(svf_max(damp))
}

pub fn svf_max(damp: f32) -> f32 {
    (0.9 * (2.0 - damp)).clamp(0.05, 1.2)
}

pub fn one_pole(hz: f32, rate: f32) -> f32 {
    1.0 - (-TAU * hz / rate).exp()
}

pub fn decay_mul(secs: f32, rate: f32) -> f32 {
    (-6.9078 / (secs.max(1e-4) * rate)).exp()
}

pub fn octave_mul(octaves_per_sec: f32, rate: f32) -> f32 {
    (octaves_per_sec * std::f32::consts::LN_2 / rate).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_tracks_std() {
        let err = (0..1000u32)
            .map(|i| (i.wrapping_mul(4_294_967), i as f32 * 4_294_967.0 / 4_294_967_296.0))
            .map(|(ph, p)| (sine_f(ph) - (TAU * p).sin()).abs())
            .fold(0.0, f32::max);
        assert!(err < 0.008, "{err}");
    }

    #[test]
    fn fixed_point_round_trips() {
        assert_eq!(mulq(1000, q(0.5)), 500);
        assert_eq!(mulq(4000, q(-0.25)), -1000);
        assert_eq!(mulc(q(1.5), q(0.5)), q(0.75));
        assert!((unq(q(1.37)) - 1.37).abs() < 1e-6);
        assert_eq!(cycles(8000.0, 32000.0), 1 << 30);
        assert_eq!(q(f32::NAN), 0);
    }

    #[test]
    fn pcm_is_monotonic_bounded_and_linear_when_quiet() {
        let ys: Vec<i16> = (-400..=400).map(|i| pcm(i * UNIT / 40)).collect();
        assert!(ys.windows(2).all(|w| w[1] >= w[0]));
        assert!(ys.iter().all(|y| (*y as f32).abs() <= CEIL * 32767.0 + 1.0));
        assert_eq!(pcm(0), 0);
        assert!((pcm(UNIT / 20) as f32 / (0.05 * CEIL * 32767.0) - 1.0).abs() < 0.01);
        assert_eq!(pcm(i32::MAX), pcm(4 * UNIT));
        assert_eq!(pcm(i32::MIN), -pcm(4 * UNIT));
    }

    #[test]
    fn noise_ranges() {
        let mut r = Rng::new(7);
        assert!((0..10_000).all(|_| (-1.0..1.0).contains(&bipolar(&mut r)) && (0.0..1.0).contains(&unit(&mut r))));
        let mean = (0..WHITE_LEN as u32).map(white_at).map(|x| x as i64).sum::<i64>() / WHITE_LEN as i64;
        let rms = ((0..WHITE_LEN as u32).map(white_at).map(|x| (x as f64).powi(2)).sum::<f64>() / WHITE_LEN as f64).sqrt() / UNIT as f64;
        assert!(mean.abs() < 600 && (0.5..0.65).contains(&rms), "{mean} {rms}");
        assert_eq!(white_at(WHITE_LEN as u32), white_at(0));
    }
}
