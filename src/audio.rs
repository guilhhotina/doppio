mod bed;
mod cue;
mod dsp;
mod space;
mod voice;
mod warble;

#[cfg(test)]
pub(crate) mod alloc;
#[cfg(test)]
mod tests;

use crate::assets::TEXTURES;
use crate::fx::Rng;
use crate::map::{Map, Tile};
use crate::sim::{Anim, Event, Events, MAX_PLAYERS, Phase, Role, World};
use bed::Bed;
use cue::Floor;
use space::{Ear, point, room_near};
use std::cmp::Ordering;
use voice::{Patch, Place, SLOTS, Voice, slot_gains};
use warble::Warble;

pub const VOICES: usize = 12;
const BLOCK: usize = 1024;
const HALF: usize = BLOCK / 2;
const BELL: Place = Place { left: 0.3, right: 0.42, muffle: 0.85 };
const ROOSTER: Place = Place { left: 0.45, right: 0.3, muffle: 0.6 };
const BIRDS: Place = Place { left: 0.35, right: 0.5, muffle: 0.25 };
const MUFFLE_HZ: f32 = 650.0;

pub struct Audio {
    rate: f32,
    voices: [Voice; VOICES],
    bed: Bed,
    warble: Warble,
    rng: Rng,
    mix: [i32; BLOCK],
    bus: [i32; BLOCK],
    bus_lp: [i32; 2],
    bus_a: i32,
    slots: [[i32; HALF]; SLOTS],
    routes: [[i32; 4]; SLOTS],
    last: [i32; 2],
    spill: Option<[i32; 2]>,
}

fn prop(map: &Map<'_>, kind: &str) -> Option<(f32, f32)> {
    map.props.iter().find(|p| p.kind == kind).map(|p| point(p.pos))
}

fn door_character(door: u8) -> f32 {
    0.85 + ((door as u32).wrapping_mul(2_654_435_761) >> 24) as f32 * (0.3 / 255.0)
}

fn nearest_window(map: &Map<'_>, ear: &Ear) -> Option<(f32, f32)> {
    (0..map.h)
        .flat_map(|y| (0..map.w).map(move |x| (x, y)))
        .filter(|&(x, y)| map.tile(x, y) == Tile::Window)
        .map(|(x, y)| (x as f32 + 0.5, y as f32 + 0.5))
        .min_by(|a, b| ear.distance(a.0, a.1).partial_cmp(&ear.distance(b.0, b.1)).unwrap_or(Ordering::Equal))
}

fn nearest_copy(world: &World, listener: u8, ear: &Ear) -> Option<f32> {
    let living = |i: usize| (0..MAX_PLAYERS).map(|p| world.player(p)).any(|p| p.role == Role::Living && p.body as usize == i);
    world
        .bodies()
        .iter()
        .enumerate()
        .filter(|&(i, b)| b.visible && i != listener as usize && !living(i))
        .map(|(_, b)| point(b.pos))
        .map(|(x, y)| ear.distance(x, y))
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
}

impl Audio {
    pub fn new(rate: u32) -> Audio {
        let rate = rate.max(8000) as f32 * 0.5;
        Audio {
            rate,
            voices: [Voice::IDLE; VOICES],
            bed: Bed::new(rate),
            warble: Warble::new(rate),
            rng: Rng::new(0xD0_99_10),
            mix: [0; BLOCK],
            bus: [0; BLOCK],
            bus_lp: [0; 2],
            bus_a: dsp::q(dsp::one_pole(MUFFLE_HZ, rate)),
            slots: [[0; HALF]; SLOTS],
            routes: std::array::from_fn(slot_gains),
            last: [0; 2],
            spill: None,
        }
    }

    fn free_voice(&self, prio: u8) -> Option<usize> {
        self.voices.iter().position(|v| !v.live()).or_else(|| {
            self.voices
                .iter()
                .enumerate()
                .filter(|(_, v)| v.prio() <= prio)
                .min_by(|a, b| (a.1.prio(), a.1.loudness()).partial_cmp(&(b.1.prio(), b.1.loudness())).unwrap_or(Ordering::Equal))
                .map(|(i, _)| i)
        })
    }

    fn play(&mut self, patches: &[Patch], place: Place) {
        let seed = self.rng.next_u32();
        patches.iter().for_each(|p| {
            if let Some(i) = self.free_voice(p.prio) {
                self.voices[i].start(p, place, seed, self.rate);
            }
        });
    }

    fn at(world: &World, ear: Option<&Ear>, x: f32, y: f32) -> Place {
        ear.map_or(Place::CENTER, |e| e.place(world, x, y))
    }

    fn event(&mut self, world: &World, ear: Option<&Ear>, event: Event, listener: u8) {
        let body = |b: u8| world.bodies().get(b as usize).map(|b| (point(b.pos), b.anim));
        let door = |d: u8| world.doors().get(d as usize).map(|d| (d.x as f32 + 0.5, d.y as f32 + 0.5));
        match event {
            Event::Step { body: b, surface } => {
                if let Some(((x, y), anim)) = body(b) {
                    let floor = TEXTURES.get(surface as usize).map_or(Floor::Wood, |(name, _)| Floor::of(name));
                    let step = cue::footstep(floor, matches!(anim, Anim::Run), &mut self.rng);
                    self.play(&[step], Self::at(world, ear, x, y));
                }
            }
            Event::DoorMove { door: d, opening } => {
                if let Some((x, y)) = door(d) {
                    let c = door_character(d);
                    let place = Self::at(world, ear, x, y);
                    match opening {
                        true => self.play(&[cue::latch(0.0, c), cue::creak(true, c)], place),
                        false => self.play(&[cue::creak(false, c), cue::slam(0.55), cue::latch(0.57, c)], place),
                    }
                }
            }
            Event::DoorRattle { door: d } => {
                if let Some((x, y)) = door(d) {
                    self.play(&cue::rattle(door_character(d)), Self::at(world, ear, x, y));
                }
            }
            Event::Lantern { body: b, on } => {
                if let Some(((x, y), _)) = body(b) {
                    self.play(&[cue::lantern(on)], Self::at(world, ear, x, y));
                }
            }
            Event::TaskDone { .. } => self.play(&cue::chime(), Place::CENTER),
            Event::PowerOut => {
                self.bed.hum.aim(Place::CENTER, 0.0);
                self.bed.tv.aim(Place::CENTER, 0.0);
                self.breaker(world, ear);
            }
            Event::PowerOn => {
                self.breaker(world, ear);
                if let Some((x, y)) = prop(world.map(), "geladeira") {
                    self.play(&[cue::compressor()], Self::at(world, ear, x, y));
                }
            }
            Event::Captured { player } => {
                if (player as usize) < MAX_PLAYERS && world.player(player as usize).body == listener {
                    self.play(&cue::captured(), Place::CENTER);
                }
            }
            Event::NightBegins { night } => self.play(&[cue::bell(night.clamp(1, 5))], BELL),
            Event::DawnBreaks { .. } => {
                self.play(&cue::rooster(), ROOSTER);
                self.play(&cue::birds(), BIRDS);
            }
            Event::Glitch { body: b } => {
                if let Some(((x, y), _)) = body(b) {
                    let d = ear.map_or(0.0, |e| e.distance(x, y));
                    self.warble.kick(1.0 / (1.0 + 0.25 * d));
                }
            }
        }
    }

    fn breaker(&mut self, world: &World, ear: Option<&Ear>) {
        let place = prop(world.map(), "disjuntor").map_or(Place::CENTER, |(x, y)| Self::at(world, ear, x, y));
        self.play(&cue::clunk(), place);
    }

    pub fn events(&mut self, world: &World, events: &Events, listener: u8) {
        let ear = Ear::new(world, listener);
        events.iter().for_each(|e| self.event(world, ear.as_ref(), *e, listener));
    }

    pub fn ambience(&mut self, world: &World, listener: u8) {
        let Some(ear) = Ear::new(world, listener) else {
            [&mut self.bed.hum, &mut self.bed.clock, &mut self.bed.wind, &mut self.bed.tv, &mut self.bed.tone, &mut self.bed.drone]
                .into_iter()
                .for_each(|l| l.aim(Place::CENTER, 0.0));
            return;
        };
        let map = world.map();
        let power = world.power_on();
        let near = |kind: &str| prop(map, kind).map(|(x, y)| (x, y, ear.room.is_some() && room_near(map, x, y) == ear.room));
        let outdoor = ear.room.and_then(|id| map.rooms.iter().find(|r| r.id == id)).is_some_and(|r| r.ceil.is_none());
        let hum = prop(map, "geladeira").filter(|_| power).map_or((Place::CENTER, 0.0), |(x, y)| (ear.place(world, x, y), 0.05));
        let clock = near("relogio_parede").filter(|c| c.2).map_or((Place::CENTER, 0.0), |(x, y, _)| (ear.place(world, x, y), 0.12));
        let tv = near("tv_tubo").filter(|t| t.2 && power).map_or((Place::CENTER, 0.0), |(x, y, _)| (ear.place(world, x, y), 0.035));
        let wind = match outdoor {
            true => (Place::CENTER, 0.06),
            false => nearest_window(map, &ear).map_or((Place::CENTER, 0.0), |(x, y)| (ear.place(world, x, y), 0.07)),
        };
        let tone = (Place::CENTER, if outdoor { 0.008 } else { 0.02 });
        let tension = match world.phase() {
            Phase::Night { night } if night >= 3 => nearest_copy(world, listener, &ear).map_or(0.0, |d| (1.0 - (d - 1.5) / 12.0).clamp(0.0, 1.0)),
            _ => 0.0,
        };
        self.bed.set_tension(tension);
        self.bed.hum.aim(hum.0, hum.1);
        self.bed.clock.aim(clock.0, clock.1);
        self.bed.tv.aim(tv.0, tv.1);
        self.bed.wind.aim(wind.0, wind.1);
        self.bed.tone.aim(tone.0, tone.1);
        self.bed.drone.aim(Place::CENTER, 0.11 * tension * tension.sqrt());
    }

    pub fn fill(&mut self, out: &mut [i16]) {
        out.chunks_mut(BLOCK * 2).for_each(|chunk| self.block(chunk));
    }

    fn block(&mut self, out: &mut [i16]) {
        let emit = |o: &mut [i16], s: [i32; 2]| {
            o[0] = dsp::pcm(s[0] + (s[0] >> 2));
            o[1] = dsp::pcm(s[1] + (s[1] >> 2));
        };
        let held = match (self.spill.take(), out.len() >= 2) {
            (Some(s), true) => {
                emit(&mut out[..2], s);
                2
            }
            (s, _) => {
                self.spill = s;
                0
            }
        };
        let out = &mut out[held..];
        let frames = (out.len() / 2).div_ceil(2);
        let mix = &mut self.mix[..frames * 2];
        let bus = &mut self.bus[..frames * 2];
        mix.fill(0);
        bus.fill(0);
        self.bed.render(mix, bus);
        let used = self.voices.iter().filter(|v| v.live()).fold(0u32, |m, v| m | 1 << v.slot());
        let slots = &mut self.slots;
        slots.iter_mut().enumerate().filter(|(i, _)| used & 1 << i != 0).for_each(|(_, s)| s[..frames].fill(0));
        self.voices.iter_mut().filter(|v| v.live()).for_each(|v| v.render(&mut slots[v.slot()][..frames]));
        slots.iter().zip(&self.routes).enumerate().filter(|(i, _)| used & 1 << i != 0).for_each(|(_, (s, g))| {
            mix.chunks_exact_mut(2).zip(bus.chunks_exact_mut(2)).zip(&s[..frames]).for_each(|((d, m), y)| {
                d[0] += dsp::mulq(*y, g[0]);
                d[1] += dsp::mulq(*y, g[1]);
                m[0] += dsp::mulq(*y, g[2]);
                m[1] += dsp::mulq(*y, g[3]);
            })
        });
        let (a, mut lp) = (self.bus_a, self.bus_lp);
        mix.chunks_exact_mut(2).zip(bus.chunks_exact(2)).for_each(|(d, m)| {
            lp = [lp[0] + dsp::mulq(m[0] - lp[0], a), lp[1] + dsp::mulq(m[1] - lp[1], a)];
            d[0] += lp[0];
            d[1] += lp[1];
        });
        self.bus_lp = lp;
        self.warble.process(mix);
        let (mut prev, mut spill) = (self.last, None);
        mix.chunks_exact(2).zip(out.chunks_mut(4)).for_each(|(m, o)| {
            let cur = [m[0], m[1]];
            emit(&mut o[..2], [(prev[0] + cur[0]) >> 1, (prev[1] + cur[1]) >> 1]);
            match o.len() >= 4 {
                true => emit(&mut o[2..4], cur),
                false => spill = Some(cur),
            }
            prev = cur;
        });
        (self.last, self.spill) = (prev, spill.or(self.spill));
    }
}
