use super::dsp::{bipolar, unit};
use super::voice::{BASE, Patch};
use crate::fx::Rng;
use crate::map::{HideKind, ItemKind, LockKind, PortalKind};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Floor {
    Wood,
    Boards,
    Hard,
    Marble,
    Cobble,
    Grit,
    Leaves,
    Soft,
    Mud,
    Water,
}

impl Floor {
    pub fn of(texture: &str) -> Floor {
        let has = |p: &[&str]| p.iter().any(|p| texture.starts_with(p));
        match () {
            _ if has(&["assoalho", "madeira_ponte"]) => Floor::Boards,
            _ if has(&["lajota", "ceramica", "azulejo", "ladrilho", "calcada"]) => Floor::Hard,
            _ if has(&["marmore"]) => Floor::Marble,
            _ if has(&["paralelepipedo", "pedra", "rocha"]) => Floor::Cobble,
            _ if has(&["asfalto", "concreto", "meio_fio"]) => Floor::Grit,
            _ if has(&["folhas"]) => Floor::Leaves,
            _ if has(&["grama", "terra", "musgo"]) => Floor::Soft,
            _ if has(&["barro"]) => Floor::Mud,
            _ if has(&["agua"]) => Floor::Water,
            _ => Floor::Wood,
        }
    }

    pub fn bright(self) -> f32 {
        match self {
            Floor::Hard | Floor::Marble | Floor::Cobble => 1.3,
            Floor::Wood | Floor::Boards | Floor::Grit => 1.0,
            Floor::Water => 0.8,
            Floor::Leaves | Floor::Soft | Floor::Mud => 0.6,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Build {
    Old,
    Light,
    Heavy,
    Plain,
}

impl Build {
    pub fn of(character: u8) -> Build {
        match character {
            0 => Build::Old,
            1 => Build::Light,
            2 => Build::Heavy,
            _ => Build::Plain,
        }
    }

    fn shape(self) -> (f32, f32, f32) {
        match self {
            Build::Old => (0.8, 0.95, 1.1),
            Build::Light => (0.5, 1.25, 0.8),
            Build::Heavy => (1.35, 0.75, 1.25),
            Build::Plain => (1.0, 1.0, 1.0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stuff {
    Metal,
    Glass,
    Cloth,
    Wood,
}

impl Stuff {
    pub fn of(kind: ItemKind) -> Stuff {
        match kind {
            ItemKind::Chave
            | ItemKind::Alicate
            | ItemKind::Martelo
            | ItemKind::PeDeCabra
            | ItemKind::ChaveDeFenda
            | ItemKind::Balde
            | ItemKind::Freio
            | ItemKind::Manivela
            | ItemKind::LanternaPilha => Stuff::Metal,
            ItemKind::Fusivel => Stuff::Glass,
            ItemKind::Ossos | ItemKind::Cruz | ItemKind::Remo | ItemKind::Vela => Stuff::Wood,
            ItemKind::Sal | ItemKind::Fosforo | ItemKind::Tabaco | ItemKind::BilheteOnibus | ItemKind::Foto => Stuff::Cloth,
        }
    }
}

pub const PRIO_AMB: u8 = 1;
pub const PRIO_STEP: u8 = 2;
pub const PRIO_WORLD: u8 = 3;
pub const PRIO_EVENT: u8 = 4;
pub const PRIO_LEGEND: u8 = 5;
pub const PRIO_SELF: u8 = 6;

pub fn footstep(floor: Floor, run: bool, build: Build, rng: &mut Rng) -> Patch {
    let pitch = 1.0 + 0.08 * bipolar(rng);
    let level = (0.85 + 0.15 * unit(rng)) * if run { 1.5 } else { 1.0 };
    let pace = if run { 0.75 } else { 1.0 };
    let base = Patch { pulses: 2, vary: 0.55, gap_jitter: 0.25, prio: PRIO_STEP, noise: 1.0, ..BASE };
    let wood = Patch { cutoff: 230.0, damp: 0.22, bp: 1.6, hp: 0.06, dry: 0.25, freq: 92.0, glide: -2.0, decay: 0.12, gap: 0.045, gain: 0.55, ..base };
    let p = match floor {
        Floor::Wood => wood,
        Floor::Boards => Patch { cutoff: 190.0, damp: 0.15, decay: 0.16, dry: 0.3, freq: 80.0, gain: 0.6, ..wood },
        Floor::Hard => Patch { cutoff: 3100.0, damp: 0.45, bp: 0.7, hp: 0.25, decay: 0.035, gap: 0.03, gain: 0.5, ..base },
        Floor::Marble => Patch { cutoff: 3600.0, damp: 0.1, bp: 0.6, hp: 0.3, decay: 0.07, gap: 0.03, gain: 0.45, ..base },
        Floor::Cobble => Patch { cutoff: 2000.0, damp: 0.3, grain: 0.7, bp: 1.0, hp: 0.2, dry: 0.12, freq: 140.0, decay: 0.05, gap: 0.035, gain: 0.5, ..base },
        Floor::Grit => Patch { cutoff: 950.0, damp: 0.8, grain: 0.45, noise: 1.5, bp: 1.0, hp: 0.2, dry: 0.2, freq: 80.0, decay: 0.06, gap: 0.04, gain: 0.5, ..base },
        Floor::Leaves => Patch { cutoff: 2600.0, damp: 1.0, grain: 0.15, noise: 1.8, bp: 0.8, hp: 0.5, attack: 0.02, hold: 0.06, decay: 0.16, gap: 0.07, gain: 0.22, ..base },
        Floor::Soft => Patch { cutoff: 1500.0, damp: 1.0, grain: 0.12, noise: 1.5, bp: 1.6, lp: 0.8, attack: 0.012, decay: 0.16, gap: 0.06, gain: 0.4, ..base },
        Floor::Mud => Patch { cutoff: 380.0, sweep: 2.0, damp: 0.35, bp: 1.3, lp: 0.5, attack: 0.02, decay: 0.15, gap: 0.08, gain: 0.5, ..base },
        Floor::Water => Patch { cutoff: 1800.0, sweep: -1.5, damp: 0.9, grain: 0.6, bp: 1.0, hp: 0.3, attack: 0.01, decay: 0.2, gap: 0.09, gain: 0.5, ..base },
    };
    let (g, f, d) = build.shape();
    Patch {
        gain: p.gain * level * g,
        decay: p.decay * pace * d,
        gap: p.gap * pace * d,
        cutoff: p.cutoff * pitch * f * if run { 1.15 } else { 1.0 },
        freq: p.freq * pitch * f,
        ..p
    }
}

pub fn underfoot(floor: Floor, run: bool, rng: &mut Rng) -> Option<Patch> {
    let roll = unit(rng);
    let level = if run { 1.4 } else { 1.0 };
    match floor {
        Floor::Boards if roll < 0.3 => Some(Patch {
            delay: 0.03,
            buzz: 1.0,
            freq: 45.0 + 30.0 * roll,
            glide: 1.2,
            jitter: 8.0,
            cutoff: 900.0,
            damp: 0.07,
            bp: 1.5,
            attack: 0.05,
            hold: 0.12,
            decay: 0.15,
            gain: 0.35 * level,
            prio: PRIO_STEP,
            ..BASE
        }),
        Floor::Water => Some(Patch { dry: 0.6, bp: 0.0, freq: 350.0 + 200.0 * roll, glide: 4.0, attack: 0.004, decay: 0.05, gain: 0.25 * level, prio: PRIO_STEP, ..BASE }),
        Floor::Mud if roll < 0.6 => Some(Patch { delay: 0.1, dry: 0.5, bp: 0.0, freq: 140.0, glide: 3.5, attack: 0.01, decay: 0.07, gain: 0.3 * level, prio: PRIO_STEP, ..BASE }),
        _ => None,
    }
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
    Patch { delay, noise: 1.0, cutoff: 180.0, damp: 0.3, bp: 1.4, dry: 0.5, freq: 70.0, glide: -1.5, decay: 0.18, gain: 0.6, prio: PRIO_WORLD, ..BASE }
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

fn ring(freq: f32, ratio: f32, index: f32, decay: f32, gain: f32) -> Patch {
    Patch { dry: 1.0, bp: 0.0, freq, fm_ratio: ratio, fm_index: index, attack: 0.002, decay, gain, prio: PRIO_EVENT, ..BASE }
}

fn rustle(gain: f32) -> Patch {
    Patch { noise: 1.0, grain: 0.4, cutoff: 2200.0, damp: 1.0, bp: 0.8, hp: 0.4, attack: 0.02, hold: 0.05, decay: 0.08, gain, prio: PRIO_EVENT, ..BASE }
}

pub fn pickup(stuff: Stuff) -> [Patch; 2] {
    let tap = match stuff {
        Stuff::Metal => Patch { pulses: 2, gap: 0.05, gap_jitter: 0.4, vary: 0.5, delay: 0.04, ..ring(1800.0, 1.41, 0.6, 0.12, 0.14) },
        Stuff::Glass => Patch { pulses: 2, gap: 0.06, vary: 0.6, delay: 0.04, ..ring(3200.0, 2.76, 0.3, 0.15, 0.1) },
        Stuff::Cloth => rustle(0.2),
        Stuff::Wood => Patch { delay: 0.04, noise: 1.0, cutoff: 650.0, damp: 0.2, bp: 1.2, dry: 0.3, freq: 240.0, decay: 0.06, gain: 0.3, prio: PRIO_EVENT, ..BASE },
    };
    [rustle(0.15), tap]
}

pub fn drop(stuff: Stuff) -> [Patch; 2] {
    let bounce = Patch { pulses: 3, gap: 0.12, gap_jitter: 0.3, vary: 0.65, ..BASE };
    match stuff {
        Stuff::Metal => [
            Patch { pulses: 3, gap: 0.12, gap_jitter: 0.3, vary: 0.65, ..ring(700.0, 2.41, 1.2, 0.5, 0.22) },
            Patch { noise: 1.0, cutoff: 300.0, damp: 0.4, bp: 1.2, decay: 0.07, gain: 0.35, prio: PRIO_EVENT, ..bounce },
        ],
        Stuff::Glass => [
            Patch { pulses: 3, gap: 0.1, gap_jitter: 0.3, vary: 0.6, ..ring(2600.0, 2.76, 0.5, 0.25, 0.14) },
            Patch { noise: 1.0, cutoff: 4000.0, damp: 0.15, bp: 0.6, hp: 0.3, decay: 0.02, gain: 0.2, prio: PRIO_EVENT, ..bounce },
        ],
        Stuff::Cloth => [
            Patch { noise: 1.0, cutoff: 300.0, damp: 0.7, lp: 1.0, bp: 0.6, decay: 0.08, gain: 0.45, prio: PRIO_EVENT, ..BASE },
            rustle(0.12),
        ],
        Stuff::Wood => [
            Patch { noise: 1.0, cutoff: 520.0, damp: 0.15, bp: 1.4, dry: 0.4, freq: 180.0, decay: 0.08, gain: 0.45, prio: PRIO_EVENT, ..bounce },
            Patch { noise: 1.0, cutoff: 1500.0, damp: 0.3, bp: 0.6, decay: 0.03, gain: 0.2, prio: PRIO_EVENT, ..bounce },
        ],
    }
}

pub fn clunk() -> [Patch; 2] {
    [
        Patch { dry: 1.0, freq: 75.0, glide: -1.2, decay: 0.2, gain: 0.5, prio: PRIO_EVENT, bp: 0.0, ..BASE },
        Patch { noise: 1.0, cutoff: 520.0, damp: 0.35, bp: 1.0, hp: 0.25, decay: 0.06, pulses: 2, gap: 0.022, vary: 0.6, gain: 0.55, prio: PRIO_EVENT, ..BASE },
    ]
}

fn delayed(p: Patch, by: f32) -> Patch {
    Patch { delay: p.delay + by, ..p }
}

pub fn unlock(kind: &LockKind) -> [Patch; 4] {
    let [thud, clack] = clunk();
    let click = |at: f32, pitch: f32| latch(at, pitch);
    let jingle = Patch { pulses: 6, gap: 0.07, gap_jitter: 0.5, vary: 0.7, ..ring(2900.0, 1.41, 0.8, 0.09, 0.12) };
    match kind {
        LockKind::Key(_) => [click(0.0, 0.8), click(0.18, 1.0), delayed(thud, 0.32), delayed(clack, 0.32)],
        LockKind::Code { .. } => [click(0.0, 1.3), Patch { pulses: 3, gap: 0.04, ..ring(1500.0, 1.5, 0.4, 0.1, 0.12) }, delayed(clack, 0.1), delayed(thud, 0.1)],
        LockKind::Tool(ItemKind::Alicate) => [
            Patch { noise: 1.0, cutoff: 4200.0, damp: 0.1, bp: 0.8, hp: 0.6, dry: 0.4, freq: 3800.0, decay: 0.05, gain: 0.3, prio: PRIO_EVENT, ..BASE },
            delayed(jingle, 0.12),
            delayed(Patch { pulses: 4, gap: 0.09, gap_jitter: 0.4, vary: 0.5, ..ring(900.0, 2.41, 1.0, 0.25, 0.15) }, 0.2),
            delayed(thud, 0.55),
        ],
        LockKind::Tool(ItemKind::PeDeCabra) => [
            Patch { buzz: 1.0, freq: 28.0, glide: 1.6, jitter: 10.0, cutoff: 800.0, sweep: 0.5, damp: 0.06, bp: 1.6, attack: 0.3, hold: 0.4, decay: 0.15, gain: 0.7, prio: PRIO_EVENT, ..BASE },
            Patch { delay: 0.8, noise: 1.0, cutoff: 1200.0, damp: 0.3, bp: 1.4, dry: 0.4, freq: 110.0, glide: -2.0, decay: 0.25, gain: 0.7, prio: PRIO_EVENT, ..BASE },
            Patch { delay: 0.95, buzz: 0.8, freq: 520.0, glide: 0.8, jitter: 0.6, cutoff: 2600.0, damp: 0.05, bp: 1.0, attack: 0.05, hold: 0.25, decay: 0.15, gain: 0.25, prio: PRIO_EVENT, ..BASE },
            Patch { delay: 1.25, noise: 1.0, cutoff: 400.0, damp: 0.4, bp: 1.2, pulses: 3, gap: 0.1, vary: 0.5, decay: 0.09, gain: 0.5, prio: PRIO_EVENT, ..BASE },
        ],
        LockKind::Tool(ItemKind::Martelo) => [
            Patch { pulses: 3, gap: 0.35, gap_jitter: 0.1, ..ring(620.0, 2.41, 1.6, 0.3, 0.3) },
            Patch { noise: 1.0, cutoff: 900.0, damp: 0.3, bp: 1.2, pulses: 3, gap: 0.35, gap_jitter: 0.1, decay: 0.08, gain: 0.6, prio: PRIO_EVENT, ..BASE },
            delayed(jingle, 0.9),
            delayed(thud, 0.95),
        ],
        LockKind::Tool(_) => [
            Patch { buzz: 0.8, freq: 260.0, jitter: 0.8, cutoff: 1800.0, damp: 0.06, bp: 1.0, pulses: 4, gap: 0.2, gap_jitter: 0.2, attack: 0.03, hold: 0.08, decay: 0.05, gain: 0.25, prio: PRIO_EVENT, ..BASE },
            click(0.85, 1.2),
            delayed(thud, 0.9),
            delayed(clack, 0.9),
        ],
        LockKind::Place { .. } => [
            Patch { noise: 1.0, cutoff: 500.0, damp: 0.5, bp: 1.2, dry: 0.3, freq: 120.0, decay: 0.08, gain: 0.4, prio: PRIO_EVENT, ..BASE },
            click(0.15, 1.0),
            delayed(thud, 0.3),
            delayed(clack, 0.3),
        ],
        LockKind::All(_) => [
            Patch { dry: 1.0, bp: 0.0, freq: 52.0, glide: -0.6, attack: 0.01, decay: 0.9, gain: 0.55, prio: PRIO_EVENT, ..BASE },
            Patch { noise: 1.0, cutoff: 260.0, damp: 0.4, bp: 1.4, pulses: 3, gap: 0.16, vary: 0.4, decay: 0.12, gain: 0.55, prio: PRIO_EVENT, ..BASE },
            click(0.05, 0.7),
            delayed(Patch { pulses: 3, gap: 0.12, vary: 0.4, ..ring(400.0, 2.41, 1.0, 0.5, 0.12) }, 0.1),
        ],
    }
}

pub fn dial_tick() -> Patch {
    Patch { noise: 1.0, cutoff: 4200.0, damp: 0.1, bp: 0.7, hp: 0.3, decay: 0.012, gain: 0.18, prio: PRIO_SELF, ..BASE }
}

pub fn dial_wrong() -> [Patch; 2] {
    [
        Patch { buzz: 1.0, freq: 92.0, cutoff: 900.0, damp: 0.5, lp: 1.0, bp: 0.5, attack: 0.005, hold: 0.16, decay: 0.04, pulses: 2, gap: 0.22, gain: 1.1, prio: PRIO_SELF, ..BASE },
        Patch { pulses: 4, gap: 0.05, gap_jitter: 0.4, vary: 0.5, prio: PRIO_SELF, ..ring(2100.0, 1.41, 0.5, 0.05, 0.22) },
    ]
}

pub fn container(metal: bool) -> [Patch; 2] {
    match metal {
        true => [
            latch(0.0, 0.7),
            Patch { buzz: 1.0, freq: 120.0, glide: 0.6, jitter: 2.0, cutoff: 2400.0, damp: 0.04, bp: 1.0, attack: 0.08, hold: 0.25, decay: 0.2, gain: 0.3, prio: PRIO_WORLD, ..BASE },
        ],
        false => [latch(0.0, 1.1), Patch { hold: 0.2, decay: 0.3, gain: 0.6, ..creak(true, 1.3) }],
    }
}

pub fn portal(kind: PortalKind) -> [Patch; 2] {
    match kind {
        PortalKind::Escada => [
            Patch { noise: 1.0, cutoff: 210.0, damp: 0.2, bp: 1.6, dry: 0.3, freq: 85.0, glide: -2.0, decay: 0.12, pulses: 5, gap: 0.24, gap_jitter: 0.15, vary: 0.4, gain: 0.55, prio: PRIO_WORLD, ..BASE },
            Patch { buzz: 1.0, freq: 38.0, glide: 1.0, jitter: 9.0, cutoff: 1000.0, damp: 0.06, bp: 1.6, attack: 0.08, hold: 0.2, decay: 0.2, pulses: 2, gap: 0.5, gain: 0.4, prio: PRIO_WORLD, ..BASE },
        ],
        PortalKind::EscadaMao => [
            Patch { pulses: 6, gap: 0.2, gap_jitter: 0.2, vary: 0.4, prio: PRIO_WORLD, ..ring(820.0, 2.41, 0.9, 0.15, 0.14) },
            Patch { noise: 1.0, cutoff: 380.0, damp: 0.3, bp: 1.3, decay: 0.06, pulses: 6, gap: 0.2, gap_jitter: 0.2, gain: 0.4, prio: PRIO_WORLD, ..BASE },
        ],
        PortalKind::Bueiro => [
            Patch { noise: 1.0, buzz: 0.6, freq: 42.0, jitter: 6.0, grain: 0.7, cutoff: 700.0, damp: 0.35, bp: 1.4, lp: 0.4, attack: 0.1, hold: 0.7, decay: 0.2, gain: 0.6, prio: PRIO_WORLD, ..BASE },
            Patch { delay: 1.0, prio: PRIO_WORLD, ..ring(310.0, 2.41, 1.8, 0.8, 0.3) },
        ],
        PortalKind::Portao => gate(PRIO_WORLD),
        PortalKind::JanelaAlta => [
            Patch { noise: 1.0, grain: 0.6, cutoff: 900.0, damp: 0.6, bp: 1.0, attack: 0.06, hold: 0.35, decay: 0.15, gain: 0.4, prio: PRIO_WORLD, ..BASE },
            delayed(slam(0.0), 0.55),
        ],
    }
}

pub fn gate(prio: u8) -> [Patch; 2] {
    [
        Patch { buzz: 1.0, freq: 55.0, glide: 0.5, jitter: 6.0, cutoff: 2300.0, sweep: 0.4, damp: 0.025, bp: 1.4, attack: 0.1, hold: 0.55, decay: 0.3, gain: 0.45, prio, ..BASE },
        Patch { delay: 1.0, pulses: 2, gap: 0.14, vary: 0.5, prio, ..ring(380.0, 2.41, 1.5, 0.6, 0.2) },
    ]
}

pub fn hide(kind: HideKind, entering: bool) -> [Patch; 2] {
    let close = Patch { noise: 1.0, cutoff: 230.0, damp: 0.3, bp: 1.4, dry: 0.3, freq: 90.0, glide: -1.5, decay: 0.1, gain: 0.4, prio: PRIO_EVENT, ..BASE };
    let wood_door = [Patch { hold: 0.12, decay: 0.2, gain: 0.5, prio: PRIO_EVENT, ..creak(entering, 1.25) }, delayed(close, 0.4)];
    match kind {
        HideKind::GuardaRoupa | HideKind::Armario | HideKind::Confessionario => wood_door,
        HideKind::Cama => [
            rustle(0.3),
            Patch { buzz: 1.0, freq: 340.0, jitter: 1.5, cutoff: 1900.0, damp: 0.05, bp: 1.0, attack: 0.02, hold: 0.06, decay: 0.1, pulses: 3, gap: 0.1, gain: 0.18, prio: PRIO_EVENT, ..BASE },
        ],
        HideKind::Tumulo => [
            Patch { noise: 1.0, grain: 0.6, cutoff: 500.0, damp: 0.5, bp: 1.4, attack: 0.1, hold: 0.5, decay: 0.15, gain: 0.55, prio: PRIO_EVENT, ..BASE },
            delayed(close, 0.65),
        ],
        HideKind::CaixaDagua => [Patch { prio: PRIO_EVENT, ..ring(140.0, 1.5, 1.0, 0.6, 0.35) }, rustle(0.2)],
        HideKind::Carro => [latch(0.0, 0.8), Patch { delay: 0.25, noise: 1.0, cutoff: 160.0, damp: 0.3, bp: 1.4, dry: 0.5, freq: 60.0, glide: -1.0, decay: 0.15, gain: 0.55, prio: PRIO_EVENT, ..BASE }],
        HideKind::Cesto => [rustle(0.3), Patch { noise: 1.0, grain: 0.1, cutoff: 1400.0, damp: 0.4, bp: 1.2, hold: 0.25, decay: 0.15, gain: 0.5, prio: PRIO_EVENT, ..BASE }],
        HideKind::Mesa => [rustle(0.3), Patch { noise: 1.0, grain: 0.5, cutoff: 700.0, damp: 0.4, bp: 1.2, attack: 0.03, hold: 0.15, decay: 0.1, gain: 0.35, prio: PRIO_EVENT, ..BASE }],
    }
}

pub fn hit(own: bool) -> [Patch; 3] {
    let prio = if own { PRIO_SELF } else { PRIO_EVENT };
    [
        Patch { noise: 1.0, cutoff: 200.0, damp: 0.5, lp: 1.3, bp: 0.4, decay: 0.3, gain: 1.4, prio, ..BASE },
        Patch { dry: 1.0, bp: 0.0, freq: 62.0, glide: -1.2, attack: 0.002, decay: 0.45, gain: 0.9, prio, ..BASE },
        Patch { noise: 1.0, cutoff: 1100.0, damp: 0.4, bp: 1.2, hp: 0.3, grain: 0.5, decay: 0.12, gain: 0.6, prio, ..BASE },
    ]
}

pub fn faint() -> [Patch; 3] {
    [
        Patch { dry: 1.0, freq: 48.0, glide: -0.8, attack: 0.004, decay: 1.6, gain: 1.0, bp: 0.0, prio: PRIO_SELF, ..BASE },
        Patch { noise: 1.0, cutoff: 90.0, damp: 0.5, lp: 1.25, bp: 0.0, decay: 0.5, gain: 1.8, prio: PRIO_SELF, ..BASE },
        Patch {
            dry: 1.0,
            freq: 6200.0,
            glide: -0.05,
            fm_ratio: 0.0047,
            fm_index: 0.08,
            delay: 0.25,
            attack: 0.6,
            hold: 2.0,
            decay: 4.0,
            gain: 0.05,
            bp: 0.0,
            prio: PRIO_SELF,
            ..BASE
        },
    ]
}

pub fn slump() -> Patch {
    Patch { noise: 1.0, cutoff: 160.0, damp: 0.5, lp: 1.2, bp: 0.6, dry: 0.4, freq: 70.0, glide: -1.0, pulses: 2, gap: 0.15, vary: 0.5, decay: 0.15, gain: 0.7, prio: PRIO_EVENT, ..BASE }
}

pub fn gasp(own: bool) -> [Patch; 2] {
    let prio = if own { PRIO_SELF } else { PRIO_EVENT };
    let gain = if own { 1.0 } else { 0.7 };
    [
        Patch { noise: 1.0, cutoff: 900.0, sweep: 1.4, damp: 0.9, bp: 1.2, hp: 0.4, attack: 0.08, hold: 0.12, decay: 0.25, gain: 0.45 * gain, prio, ..BASE },
        Patch { buzz: 0.7, freq: 150.0, glide: 0.8, jitter: 0.5, cutoff: 750.0, damp: 0.5, bp: 1.0, attack: 0.06, hold: 0.1, decay: 0.15, gain: 0.25 * gain, prio, ..BASE },
    ]
}

pub fn breath(fast: bool) -> [Patch; 2] {
    let pace = if fast { 0.6 } else { 1.0 };
    [
        Patch { noise: 1.0, cutoff: 1100.0, sweep: 0.5, damp: 0.9, bp: 1.0, hp: 0.3, attack: 0.7 * pace, hold: 0.1, decay: 0.25, gain: 0.14, prio: PRIO_SELF, ..BASE },
        Patch { delay: 1.1 * pace, noise: 1.0, cutoff: 750.0, sweep: -0.6, damp: 0.9, bp: 1.0, lp: 0.3, attack: 0.12, hold: 0.2, decay: 0.7 * pace, gain: 0.11, prio: PRIO_SELF, ..BASE },
    ]
}

pub fn pour() -> Patch {
    Patch { noise: 1.0, grain: 0.3, cutoff: 3000.0, damp: 0.8, bp: 0.8, hp: 0.4, attack: 0.1, hold: 0.6, decay: 0.2, gain: 0.15, prio: PRIO_EVENT, ..BASE }
}

pub fn splash() -> [Patch; 2] {
    [
        Patch { noise: 1.0, cutoff: 900.0, sweep: 1.5, damp: 0.9, bp: 1.0, lp: 0.4, attack: 0.08, hold: 0.1, decay: 0.15, gain: 0.4, prio: PRIO_EVENT, ..BASE },
        Patch { delay: 0.25, noise: 1.0, cutoff: 1400.0, sweep: -1.2, damp: 0.8, bp: 1.0, hp: 0.3, attack: 0.01, decay: 0.35, gain: 0.6, prio: PRIO_EVENT, ..BASE },
    ]
}

pub fn heartbeat(strength: f32) -> [Patch; 2] {
    let beat = Patch { dry: 1.0, bp: 0.0, lp: 0.6, noise: 0.6, cutoff: 90.0, damp: 0.6, freq: 58.0, glide: -1.2, attack: 0.008, decay: 0.16, prio: PRIO_SELF, ..BASE };
    [Patch { gain: 0.55 * strength, ..beat }, Patch { delay: 0.17, gain: 0.38 * strength, freq: 52.0, ..beat }]
}

pub fn compressor() -> Patch {
    Patch { dry: 1.0, freq: 52.0, glide: -0.6, attack: 0.01, decay: 0.35, gain: 0.3, bp: 0.0, prio: PRIO_WORLD, ..BASE }
}

pub fn chime() -> [Patch; 2] {
    let note = Patch { tone: 0.6, fm_ratio: 3.0, fm_index: 0.5, cutoff: 1800.0, damp: 1.0, bp: 0.0, lp: 1.0, attack: 0.006, decay: 1.1, gain: 0.22, prio: PRIO_EVENT, ..BASE };
    [Patch { freq: 587.0, ..note }, Patch { freq: 784.0, delay: 0.14, ..note }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floors_by_texture() {
        [
            ("taco", Floor::Wood),
            ("madeira_escura", Floor::Wood),
            ("assoalho", Floor::Boards),
            ("madeira_ponte", Floor::Boards),
            ("lajota", Floor::Hard),
            ("ceramica_barro", Floor::Hard),
            ("calcada_portuguesa", Floor::Hard),
            ("ladrilho_hidraulico", Floor::Hard),
            ("marmore_xadrez", Floor::Marble),
            ("paralelepipedo", Floor::Cobble),
            ("pedra", Floor::Cobble),
            ("asfalto", Floor::Grit),
            ("concreto", Floor::Grit),
            ("folhas_chao", Floor::Leaves),
            ("grama", Floor::Soft),
            ("terra_cova", Floor::Soft),
            ("musgo", Floor::Soft),
            ("barro", Floor::Mud),
            ("agua_rio", Floor::Water),
        ]
        .iter()
        .for_each(|(name, floor)| assert_eq!(Floor::of(name), *floor, "{name}"));
    }

    #[test]
    fn builds_shape_steps() {
        let step = |b: u8| footstep(Floor::Wood, false, Build::of(b), &mut Rng::new(9));
        assert!(step(1).gain < step(3).gain && step(3).gain < step(2).gain);
        assert!(step(2).freq < step(1).freq);
    }
}
