use super::bed::{CRICKETS, Chirp, INSECTS};
use super::cue::{PRIO_AMB, gate};
use super::dsp::unit;
use super::voice::{BASE, Patch};
use crate::fx::Rng;
use crate::map::{LegendKind, Map};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scene {
    Sobrado,
    Cidade,
    Igreja,
    Mata,
    Cemiterio,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Far {
    Dogs,
    Car,
    Owl,
    Frogs,
    Crows,
    BellCreak,
    GateCreak,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spot {
    Away,
    Prop(&'static str),
}

#[derive(Clone, Copy, Debug)]
pub struct Sporadic {
    pub sound: Far,
    pub every: (f32, f32),
    pub spot: Spot,
}

#[derive(Clone, Copy, Debug)]
pub struct Beds {
    pub chirp: Chirp,
    pub hum_hz: f32,
    pub buzzy: bool,
    pub wind_bright: f32,
    pub wind: (f32, f32),
    pub chirr: f32,
    pub tone: (f32, f32),
    pub verb: f32,
}

const fn every(sound: Far, lo: f32, hi: f32, spot: Spot) -> Sporadic {
    Sporadic { sound, every: (lo, hi), spot }
}

const CIDADE: [Sporadic; 2] = [every(Far::Dogs, 9.0, 24.0, Spot::Away), every(Far::Car, 25.0, 55.0, Spot::Away)];
const IGREJA: [Sporadic; 2] = [every(Far::BellCreak, 16.0, 38.0, Spot::Prop("sino")), every(Far::Dogs, 30.0, 70.0, Spot::Away)];
const MATA: [Sporadic; 3] = [every(Far::Frogs, 1.2, 4.5, Spot::Away), every(Far::Frogs, 2.0, 6.0, Spot::Away), every(Far::Owl, 14.0, 35.0, Spot::Away)];
const CEMITERIO: [Sporadic; 2] = [every(Far::GateCreak, 14.0, 40.0, Spot::Prop("portao_ferro")), every(Far::Crows, 11.0, 28.0, Spot::Away)];

impl Scene {
    pub fn of(map: &Map<'_>) -> Scene {
        match map.legend {
            LegendKind::Cabra => Scene::Cidade,
            LegendKind::Mula => Scene::Igreja,
            LegendKind::Matinta => Scene::Mata,
            LegendKind::CorpoSeco if map.props.iter().any(|p| matches!(p.kind, "lapide" | "tumulo" | "cova_aberta" | "ossario")) => Scene::Cemiterio,
            LegendKind::CorpoSeco => Scene::Sobrado,
        }
    }

    pub fn sporadics(self) -> &'static [Sporadic] {
        match self {
            Scene::Sobrado => &[],
            Scene::Cidade => &CIDADE,
            Scene::Igreja => &IGREJA,
            Scene::Mata => &MATA,
            Scene::Cemiterio => &CEMITERIO,
        }
    }

    pub fn beds(self) -> Beds {
        let base = Beds { chirp: CRICKETS, hum_hz: 60.0, buzzy: false, wind_bright: 0.5, wind: (0.06, 0.05), chirr: 0.0, tone: (0.008, 0.02), verb: 0.0 };
        match self {
            Scene::Sobrado => base,
            Scene::Cidade => Beds { hum_hz: 120.0, buzzy: true, wind: (0.035, 0.03), chirr: 0.03, ..base },
            Scene::Igreja => Beds { wind: (0.04, 0.03), chirr: 0.025, tone: (0.008, 0.045), verb: 0.32, ..base },
            Scene::Mata => Beds { chirp: INSECTS, wind: (0.04, 0.03), chirr: 0.035, ..base },
            Scene::Cemiterio => Beds { wind_bright: 0.8, wind: (0.08, 0.05), chirr: 0.018, tone: (0.006, 0.025), verb: 0.14, ..base },
        }
    }
}

pub fn wait(s: &Sporadic, rng: &mut Rng) -> u32 {
    ((s.every.0 + (s.every.1 - s.every.0) * unit(rng)) * 60.0) as u32
}

pub fn far(sound: Far, rng: &mut Rng) -> [Patch; 2] {
    let r = unit(rng);
    let amb = Patch { prio: PRIO_AMB, ..BASE };
    match sound {
        Far::Dogs => {
            let bark = Patch {
                buzz: 1.0,
                noise: 0.4,
                freq: 300.0 + 120.0 * r,
                glide: -1.2,
                jitter: 0.3,
                cutoff: 900.0,
                damp: 0.4,
                bp: 1.2,
                attack: 0.01,
                hold: 0.06,
                decay: 0.12,
                pulses: 2 + (r * 3.0) as u8,
                gap: 0.3,
                gap_jitter: 0.25,
                vary: 0.3,
                gain: 0.5,
                ..amb
            };
            [bark, Patch { delay: 1.4 + r, freq: 380.0 - 80.0 * r, pulses: 2, gain: 0.3, ..bark }]
        }
        Far::Car => [
            Patch { buzz: 1.0, freq: 62.0, glide: 0.15, jitter: 0.5, cutoff: 300.0, damp: 0.6, lp: 1.0, bp: 0.3, attack: 2.5, hold: 0.8, decay: 3.0, gain: 0.35, ..amb },
            Patch { noise: 1.0, cutoff: 650.0, damp: 1.0, lp: 0.8, bp: 0.6, attack: 2.4, hold: 0.9, decay: 2.8, gain: 0.4, ..amb },
        ],
        Far::Owl => {
            let hoot = Patch { dry: 1.0, bp: 0.6, noise: 0.3, cutoff: 420.0, damp: 0.3, freq: 410.0, glide: -0.35, fm_ratio: 1.0, fm_index: 0.15, attack: 0.05, hold: 0.15, decay: 0.3, gain: 0.3, ..amb };
            [Patch { pulses: 2, gap: 0.5, ..hoot }, Patch { delay: 1.15, hold: 0.45, decay: 0.5, ..hoot }]
        }
        Far::Frogs => match r < 0.5 {
            true => {
                let croak = Patch { buzz: 1.0, freq: 150.0 + 80.0 * r, jitter: 0.3, cutoff: 600.0, damp: 0.3, bp: 1.4, attack: 0.005, hold: 0.03, decay: 0.05, gap: 0.08, gap_jitter: 0.2, vary: 0.3, gain: 1.2, ..amb };
                [Patch { pulses: 3, ..croak }, Patch { delay: 0.6, pulses: 2, freq: croak.freq * 0.9, gain: 0.8, ..croak }]
            }
            false => {
                let tink = Patch { dry: 1.0, bp: 0.0, freq: 1700.0 + 900.0 * r, glide: 5.0, attack: 0.002, decay: 0.05, gap: 0.22, gap_jitter: 0.2, vary: 0.4, gain: 0.45, ..amb };
                [Patch { pulses: 3, ..tink }, Patch { delay: 0.11, pulses: 2, freq: tink.freq * 1.12, gain: 0.3, ..tink }]
            }
        },
        Far::Crows => {
            let caw = Patch {
                buzz: 1.0,
                noise: 0.6,
                freq: 420.0 + 80.0 * r,
                glide: -0.6,
                jitter: 0.6,
                cutoff: 1300.0,
                damp: 0.25,
                bp: 1.2,
                hp: 0.3,
                attack: 0.02,
                hold: 0.2,
                decay: 0.15,
                pulses: 2 + (r * 2.0) as u8,
                gap: 0.42,
                gap_jitter: 0.2,
                vary: 0.3,
                gain: 0.3,
                ..amb
            };
            [caw, Patch { delay: 2.1, pulses: 2, freq: caw.freq * 1.1, gain: 0.3, ..caw }]
        }
        Far::BellCreak => [
            Patch { buzz: 1.0, freq: 22.0, glide: 0.3, jitter: 10.0, cutoff: 450.0, damp: 0.05, bp: 1.6, attack: 0.6, hold: 0.8, decay: 0.8, gain: 0.6, ..amb },
            Patch { delay: 0.5, dry: 1.0, bp: 0.0, freq: 196.0, fm_ratio: 1.4, fm_index: 1.2, attack: 0.4, decay: 4.0, gain: 0.04, ..amb },
        ],
        Far::GateCreak => gate(PRIO_AMB).map(|p| Patch { gain: p.gain * 0.8, ..p }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waits_are_in_range_and_fit_the_timers() {
        assert!([Scene::Sobrado, Scene::Cidade, Scene::Igreja, Scene::Mata, Scene::Cemiterio].iter().all(|s| s.sporadics().len() <= 4));
        let mut rng = Rng::new(4);
        [Scene::Sobrado, Scene::Cidade, Scene::Igreja, Scene::Mata, Scene::Cemiterio]
            .iter()
            .flat_map(|s| s.sporadics())
            .for_each(|s| (0..50).for_each(|_| assert!((s.every.0 * 60.0..=s.every.1 * 60.0).contains(&(wait(s, &mut rng) as f32)))));
    }
}
