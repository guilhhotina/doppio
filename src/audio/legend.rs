use super::cue::{Floor, PRIO_LEGEND};
use super::dsp::unit;
use super::voice::{BASE, Patch};
use crate::fx::Rng;

const L: Patch = Patch { prio: PRIO_LEGEND, ..BASE };

pub fn shuffle(floor: Floor, rng: &mut Rng) -> [Patch; 2] {
    let b = floor.bright() * (0.9 + 0.2 * unit(rng));
    [
        Patch { noise: 1.0, grain: 0.5, cutoff: 650.0 * b, sweep: -1.5, damp: 0.9, bp: 1.2, lp: 0.4, attack: 0.05, hold: 0.1, decay: 0.2, gain: 0.5, ..L },
        Patch { delay: 0.12, noise: 1.0, cutoff: 1800.0 * b, damp: 0.15, bp: 0.8, hp: 0.2, decay: 0.03, gain: 0.3, ..L },
    ]
}

pub fn joints(rng: &mut Rng) -> [Patch; 2] {
    let r = unit(rng);
    [
        Patch { buzz: 1.0, freq: 24.0 + 10.0 * r, glide: 1.0, jitter: 12.0, cutoff: 1500.0 + 400.0 * r, damp: 0.05, bp: 1.6, attack: 0.05, hold: 0.15, decay: 0.2, gain: 0.45, ..L },
        Patch { delay: 0.2, noise: 1.0, cutoff: 2500.0, damp: 0.1, bp: 0.9, hp: 0.2, pulses: 3, gap: 0.06, gap_jitter: 0.5, vary: 0.5, decay: 0.015, gain: 0.4, ..L },
    ]
}

pub fn rattle_breath() -> [Patch; 2] {
    let inhale = Patch {
        noise: 0.5,
        buzz: 0.6,
        freq: 24.0,
        jitter: 6.0,
        cutoff: 700.0,
        sweep: 0.6,
        damp: 0.6,
        bp: 1.0,
        lp: 0.3,
        attack: 0.45,
        hold: 0.2,
        decay: 0.35,
        gain: 0.4,
        ..L
    };
    [inhale, Patch { delay: 1.1, freq: 19.0, cutoff: 480.0, sweep: -0.5, attack: 0.2, hold: 0.4, decay: 0.6, gain: 0.35, ..inhale }]
}

pub fn rasping_cry() -> [Patch; 3] {
    [
        Patch { buzz: 1.0, noise: 0.4, freq: 175.0, glide: -0.3, jitter: 4.0, cutoff: 1300.0, sweep: -0.2, damp: 0.3, bp: 1.2, hp: 0.2, attack: 0.3, hold: 1.6, decay: 1.2, gain: 0.6, ..L },
        Patch { dry: 1.0, bp: 0.0, freq: 220.0, glide: -0.25, jitter: 0.3, fm_ratio: 1.5, fm_index: 0.6, attack: 0.4, hold: 1.4, decay: 1.0, gain: 0.16, ..L },
        Patch { delay: 0.2, noise: 1.0, grain: 0.35, cutoff: 2600.0, damp: 0.6, bp: 0.6, hp: 0.5, attack: 0.2, hold: 1.4, decay: 0.8, gain: 0.25, ..L },
    ]
}

pub fn recoil() -> [Patch; 2] {
    [
        Patch { noise: 1.0, cutoff: 3500.0, sweep: -1.0, damp: 0.7, bp: 0.5, hp: 0.6, attack: 0.02, hold: 0.3, decay: 0.6, gain: 0.25, ..L },
        Patch { buzz: 1.0, freq: 140.0, glide: -1.0, jitter: 3.0, cutoff: 900.0, damp: 0.3, bp: 1.2, attack: 0.02, hold: 0.2, decay: 0.4, gain: 0.25, ..L },
    ]
}

pub fn clop(floor: Floor) -> [Patch; 2] {
    let b = floor.bright();
    let beat = Patch { pulses: 2, gap: 0.11, gap_jitter: 0.1, vary: 0.3, ..L };
    [
        Patch { noise: 1.0, cutoff: 1100.0 * b, damp: 0.08, bp: 1.4, decay: 0.05, gain: 0.35, ..beat },
        Patch { dry: 1.0, bp: 0.0, freq: 330.0 * b, glide: -3.0, attack: 0.001, decay: 0.05, gain: 0.18, ..beat },
    ]
}

pub fn bleat_scream() -> [Patch; 4] {
    [
        Patch { buzz: 1.0, freq: 390.0, jitter: 0.25, cutoff: 1100.0, damp: 0.25, bp: 1.4, hp: 0.2, attack: 0.01, hold: 0.09, decay: 0.07, pulses: 4, gap: 0.14, gap_jitter: 0.1, vary: 0.2, gain: 0.5, ..L },
        Patch { delay: 0.55, buzz: 1.0, noise: 0.5, freq: 520.0, glide: 0.6, jitter: 0.35, cutoff: 1800.0, sweep: 0.5, damp: 0.2, bp: 1.4, hp: 0.3, attack: 0.25, hold: 1.1, decay: 0.5, gain: 0.4, ..L },
        Patch { delay: 0.6, dry: 1.0, bp: 0.0, freq: 1040.0, glide: 0.6, jitter: 0.06, fm_ratio: 0.5, fm_index: 0.7, attack: 0.3, hold: 1.0, decay: 0.4, gain: 0.12, ..L },
        Patch { delay: 0.65, noise: 1.0, cutoff: 3200.0, damp: 0.5, bp: 0.6, hp: 0.6, attack: 0.3, hold: 0.9, decay: 0.5, gain: 0.15, ..L },
    ]
}

pub fn fire_roar() -> [Patch; 3] {
    [
        Patch { noise: 1.0, cutoff: 300.0, sweep: 2.0, damp: 0.7, lp: 1.0, bp: 0.6, attack: 0.15, hold: 0.6, decay: 0.7, gain: 0.8, ..L },
        Patch { buzz: 1.0, freq: 45.0, jitter: 3.0, cutoff: 200.0, damp: 0.5, lp: 1.0, bp: 0.0, attack: 0.1, hold: 0.7, decay: 0.6, gain: 0.5, ..L },
        Patch { noise: 3.0, grain: 0.03, cutoff: 2500.0, damp: 0.6, bp: 0.4, hp: 0.8, attack: 0.05, hold: 1.0, decay: 0.8, gain: 0.4, ..L },
    ]
}

pub fn splash_sizzle() -> [Patch; 2] {
    [
        Patch { noise: 1.0, cutoff: 1200.0, sweep: -1.0, damp: 0.8, bp: 1.0, lp: 0.4, attack: 0.01, decay: 0.3, gain: 0.7, ..L },
        Patch { delay: 0.1, noise: 1.0, grain: 0.3, cutoff: 5000.0, damp: 0.8, hp: 1.0, bp: 0.2, attack: 0.05, hold: 0.8, decay: 1.0, gain: 0.3, ..L },
    ]
}

pub fn gallop(floor: Floor, charging: bool) -> [Patch; 2] {
    let b = floor.bright();
    let g = if charging { 1.3 } else { 1.0 };
    let beat = Patch { pulses: 3, gap: if charging { 0.07 } else { 0.09 }, gap_jitter: 0.15, vary: 0.35, ..L };
    [
        Patch { noise: 1.0, cutoff: 120.0, damp: 0.4, lp: 1.2, bp: 0.6, dry: 0.6, freq: 60.0, glide: -1.5, decay: 0.12, gain: 0.8 * g, ..beat },
        Patch { noise: 1.0, cutoff: 900.0 * b, damp: 0.2, bp: 1.0, decay: 0.04, gain: 0.3 * g, ..beat },
    ]
}

pub fn chains(rng: &mut Rng) -> [Patch; 2] {
    let p = 0.9 + 0.2 * unit(rng);
    [
        Patch { dry: 1.0, bp: 0.0, freq: 2900.0 * p, fm_ratio: 1.41, fm_index: 0.8, attack: 0.001, decay: 0.08, pulses: 5, gap: 0.06, gap_jitter: 0.6, vary: 0.7, gain: 0.12, ..L },
        Patch { noise: 1.0, cutoff: 6000.0 * p, damp: 0.1, bp: 0.7, hp: 0.2, decay: 0.03, pulses: 5, gap: 0.06, gap_jitter: 0.6, vary: 0.7, gain: 0.18, ..L },
    ]
}

pub fn crash() -> [Patch; 4] {
    [
        Patch { noise: 1.0, cutoff: 400.0, damp: 0.6, lp: 1.2, bp: 0.6, attack: 0.002, decay: 0.6, gain: 0.8, ..L },
        Patch { dry: 1.0, bp: 0.0, freq: 42.0, glide: -0.8, attack: 0.002, decay: 0.8, gain: 0.7, ..L },
        Patch { delay: 0.08, noise: 1.0, grain: 0.5, cutoff: 1800.0, damp: 0.3, bp: 1.2, pulses: 5, gap: 0.09, gap_jitter: 0.6, vary: 0.6, decay: 0.07, gain: 0.5, ..L },
        Patch { delay: 0.05, pulses: 4, gap: 0.08, gap_jitter: 0.5, vary: 0.6, dry: 1.0, bp: 0.0, freq: 2400.0, fm_ratio: 1.41, fm_index: 0.9, decay: 0.15, gain: 0.12, ..L },
    ]
}

pub fn whistle(rng: &mut Rng) -> [Patch; 2] {
    let f = 2300.0 + 150.0 * unit(rng);
    [
        Patch { dry: 1.0, bp: 0.0, freq: f, glide: -0.06, jitter: 0.012, attack: 0.25, hold: 1.5, decay: 0.6, gain: 0.35, ..L },
        Patch { noise: 1.0, cutoff: f, damp: 0.04, bp: 1.0, attack: 0.25, hold: 1.5, decay: 0.6, gain: 0.08, ..L },
    ]
}

pub fn laugh() -> [Patch; 2] {
    let syl = Patch { pulses: 6, gap: 0.13, gap_jitter: 0.15, vary: 0.4, attack: 0.01, hold: 0.04, decay: 0.08, ..L };
    [
        Patch { noise: 1.0, cutoff: 1500.0, sweep: -0.4, damp: 0.35, bp: 1.0, hp: 0.3, gain: 0.18, ..syl },
        Patch { noise: 1.0, cutoff: 3200.0, damp: 0.5, bp: 0.5, hp: 0.5, gain: 0.1, ..syl },
    ]
}

pub fn wings() -> [Patch; 2] {
    let flap = Patch { pulses: 5, gap: 0.14, gap_jitter: 0.1, vary: 0.3, attack: 0.03, decay: 0.1, ..L };
    [
        Patch { noise: 1.0, cutoff: 500.0, damp: 1.0, lp: 1.0, bp: 0.6, gain: 0.5, ..flap },
        Patch { noise: 1.0, cutoff: 1800.0, damp: 0.9, bp: 0.6, hp: 0.3, gain: 0.15, ..flap },
    ]
}
