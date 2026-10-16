use super::dsp::{bipolar, unit};
use super::voice::{BASE, Patch};
use crate::fx::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Floor {
    Wood,
    Hard,
    Soft,
    Concrete,
}

impl Floor {
    pub fn of(texture: &str) -> Floor {
        match texture {
            t if t.starts_with("lajota") || t.starts_with("ceramica") || t.starts_with("azulejo") => Floor::Hard,
            t if t.starts_with("grama") || t.starts_with("terra") => Floor::Soft,
            t if t.starts_with("concreto") => Floor::Concrete,
            _ => Floor::Wood,
        }
    }
}

pub const PRIO_STEP: u8 = 1;
pub const PRIO_WORLD: u8 = 2;
pub const PRIO_EVENT: u8 = 3;
pub const PRIO_SELF: u8 = 4;

pub fn footstep(floor: Floor, run: bool, rng: &mut Rng) -> Patch {
    let pitch = 1.0 + 0.08 * bipolar(rng);
    let level = (0.85 + 0.15 * unit(rng)) * if run { 1.5 } else { 1.0 };
    let pace = if run { 0.75 } else { 1.0 };
    let base = Patch { pulses: 2, vary: 0.55, gap_jitter: 0.25, prio: PRIO_STEP, noise: 1.0, ..BASE };
    let p = match floor {
        Floor::Wood => Patch {
            cutoff: 230.0 * pitch,
            damp: 0.22,
            bp: 1.6,
            hp: 0.06,
            tone: 0.0,
            dry: 0.25,
            freq: 92.0 * pitch,
            glide: -2.0,
            decay: 0.12,
            gap: 0.045,
            gain: 0.55,
            ..base
        },
        Floor::Hard => Patch {
            cutoff: 3100.0 * pitch,
            damp: 0.45,
            bp: 0.7,
            hp: 0.25,
            decay: 0.035,
            gap: 0.03,
            gain: 0.5,
            ..base
        },
        Floor::Soft => Patch {
            cutoff: 1500.0 * pitch,
            damp: 1.0,
            grain: 0.12,
            noise: 1.5,
            bp: 1.6,
            lp: 0.8,
            attack: 0.012,
            decay: 0.16,
            gap: 0.06,
            gain: 0.4,
            ..base
        },
        Floor::Concrete => Patch {
            cutoff: 900.0 * pitch,
            damp: 0.8,
            grain: 0.45,
            noise: 1.5,
            bp: 1.0,
            hp: 0.15,
            dry: 0.2,
            freq: 80.0 * pitch,
            decay: 0.06,
            gap: 0.04,
            gain: 0.5,
            ..base
        },
    };
    Patch { gain: p.gain * level, decay: p.decay * pace, gap: p.gap * pace, cutoff: p.cutoff * if run { 1.15 } else { 1.0 }, ..p }
}

pub fn latch(delay: f32, pitch: f32) -> Patch {
    Patch {
        delay,
        noise: 1.0,
        cutoff: 3300.0 * pitch,
        damp: 0.08,
        bp: 0.5,
        hp: 0.2,
        decay: 0.04,
        pulses: 2,
        gap: 0.018,
        vary: 0.5,
        gain: 0.3,
        prio: PRIO_WORLD,
        ..BASE
    }
}

pub fn creak(opening: bool, character: f32) -> Patch {
    let (from, glide) = if opening { (35.0, 1.4) } else { (110.0, -1.6) };
    Patch {
        delay: if opening { 0.06 } else { 0.0 },
        buzz: 1.0,
        freq: from * character,
        glide,
        jitter: 9.0,
        cutoff: 1150.0 * character,
        sweep: if opening { 0.3 } else { -0.3 },
        damp: 0.06,
        bp: 1.8,
        lp: 0.1,
        attack: 0.12,
        hold: 0.35,
        decay: 0.45,
        gain: 0.8,
        prio: PRIO_WORLD,
        ..BASE
    }
}

pub fn slam(delay: f32) -> Patch {
    Patch {
        delay,
        noise: 1.0,
        cutoff: 180.0,
        damp: 0.3,
        bp: 1.4,
        dry: 0.5,
        freq: 70.0,
        glide: -1.5,
        decay: 0.18,
        gain: 0.6,
        prio: PRIO_WORLD,
        ..BASE
    }
}

pub fn rattle(character: f32) -> [Patch; 2] {
    let rhythm = Patch { pulses: 6, gap: 0.085, gap_jitter: 0.35, vary: 0.6, prio: PRIO_WORLD, noise: 1.0, ..BASE };
    [
        Patch { cutoff: 2400.0 * character, damp: 0.07, bp: 0.6, hp: 0.15, decay: 0.05, gain: 0.25, ..rhythm },
        Patch { cutoff: 140.0 * character, damp: 0.25, bp: 1.8, dry: 0.3, freq: 75.0, decay: 0.09, gain: 0.55, ..rhythm },
    ]
}

pub fn lantern(on: bool) -> Patch {
    Patch {
        noise: 1.0,
        cutoff: if on { 2600.0 } else { 2000.0 },
        damp: 0.12,
        bp: 0.6,
        hp: 0.3,
        decay: 0.025,
        pulses: 2,
        gap: 0.012,
        vary: 0.4,
        gain: 0.2,
        prio: PRIO_EVENT,
        ..BASE
    }
}

pub fn chime() -> [Patch; 2] {
    let note = Patch {
        tone: 0.6,
        fm_ratio: 3.0,
        fm_index: 0.5,
        cutoff: 1800.0,
        damp: 1.0,
        bp: 0.0,
        lp: 1.0,
        attack: 0.006,
        decay: 1.1,
        gain: 0.22,
        prio: PRIO_EVENT,
        ..BASE
    };
    [Patch { freq: 587.0, ..note }, Patch { freq: 784.0, delay: 0.14, ..note }]
}

pub fn clunk() -> [Patch; 2] {
    [
        Patch { dry: 1.0, freq: 75.0, glide: -1.2, decay: 0.2, gain: 0.5, prio: PRIO_EVENT, bp: 0.0, ..BASE },
        Patch {
            noise: 1.0,
            cutoff: 520.0,
            damp: 0.35,
            bp: 1.0,
            hp: 0.25,
            decay: 0.06,
            pulses: 2,
            gap: 0.022,
            vary: 0.6,
            gain: 0.55,
            prio: PRIO_EVENT,
            ..BASE
        },
    ]
}

pub fn compressor() -> Patch {
    Patch { dry: 1.0, freq: 52.0, glide: -0.6, attack: 0.01, decay: 0.35, gain: 0.3, bp: 0.0, prio: PRIO_WORLD, ..BASE }
}

pub fn captured() -> [Patch; 3] {
    [
        Patch { dry: 1.0, freq: 48.0, glide: -0.8, attack: 0.004, decay: 1.6, gain: 1.0, bp: 0.0, prio: PRIO_SELF, ..BASE },
        Patch { noise: 1.0, cutoff: 90.0, damp: 0.5, lp: 1.25, bp: 0.0, decay: 0.5, gain: 1.8, prio: PRIO_SELF, ..BASE },
        Patch {
            tone: 0.0,
            dry: 1.0,
            freq: 6200.0,
            fm_ratio: 0.0047,
            fm_index: 0.08,
            delay: 0.25,
            attack: 0.6,
            hold: 2.0,
            decay: 6.0,
            gain: 0.05,
            bp: 0.0,
            prio: PRIO_SELF,
            ..BASE
        },
    ]
}

pub fn bell(strikes: u8) -> Patch {
    Patch {
        tone: 0.0,
        dry: 1.0,
        freq: 196.0,
        fm_ratio: 1.4,
        fm_index: 2.6,
        attack: 0.003,
        decay: 5.0,
        pulses: strikes.max(1),
        gap: 2.4,
        gain: 0.22,
        bp: 0.0,
        prio: PRIO_EVENT,
        ..BASE
    }
}

pub fn rooster() -> [Patch; 4] {
    let syl = Patch {
        tone: 0.0,
        dry: 1.0,
        fm_ratio: 1.0,
        fm_index: 2.2,
        attack: 0.015,
        hold: 0.07,
        decay: 0.12,
        gain: 0.18,
        bp: 0.0,
        prio: PRIO_EVENT,
        ..BASE
    };
    [
        Patch { delay: 0.0, freq: 520.0, glide: 2.0, ..syl },
        Patch { delay: 0.22, freq: 640.0, glide: 0.5, ..syl },
        Patch { delay: 0.42, freq: 700.0, glide: 0.4, ..syl },
        Patch { delay: 0.64, freq: 760.0, glide: -0.6, hold: 0.55, decay: 0.35, ..syl },
    ]
}

pub fn birds() -> [Patch; 2] {
    let chirp = Patch { tone: 0.0, dry: 1.0, attack: 0.008, decay: 0.07, gap_jitter: 0.3, vary: 0.5, gain: 0.12, bp: 0.0, prio: PRIO_EVENT, ..BASE };
    [
        Patch { delay: 1.4, freq: 4300.0, glide: -6.0, pulses: 5, gap: 0.16, ..chirp },
        Patch { delay: 2.3, freq: 3100.0, glide: 5.0, pulses: 4, gap: 0.22, ..chirp },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floors_by_texture() {
        assert_eq!(Floor::of("taco"), Floor::Wood);
        assert_eq!(Floor::of("madeira_escura"), Floor::Wood);
        assert_eq!(Floor::of("lajota"), Floor::Hard);
        assert_eq!(Floor::of("ceramica_barro"), Floor::Hard);
        assert_eq!(Floor::of("azulejo"), Floor::Hard);
        assert_eq!(Floor::of("grama"), Floor::Soft);
        assert_eq!(Floor::of("terra"), Floor::Soft);
        assert_eq!(Floor::of("concreto"), Floor::Concrete);
    }
}
