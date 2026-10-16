mod bed;
mod cue;
mod dsp;
mod legend;
mod scene;
mod space;
mod verb;
mod voice;

#[cfg(test)]
pub(crate) mod alloc;
#[cfg(test)]
mod tests;

use crate::assets::TEXTURES;
use crate::fx::Rng;
use crate::map::{DoorState, HideKind, ItemKind, LegendKind, LightMode, Map, Tile};
use crate::sim::{Event, Events, LegendMode, PlayerState, World};
use bed::Bed;
use cue::{Build, Floor, Stuff};
use scene::{Scene, Spot};
use space::{Ear, area_near, point};
use std::cmp::Ordering;
use verb::Verb;
use voice::{Patch, Place, SLOTS, Voice, slot_gains};

pub const VOICES: usize = 10;
const BLOCK: usize = 1024;
const HALF: usize = BLOCK / 2;
const MUFFLE_HZ: f32 = 650.0;
const FAR: f32 = 0.35;
const INAUDIBLE: f32 = 0.003;
const TIMERS: usize = 4;
const NEAR: i32 = 10;

pub struct Audio {
    rate: f32,
    voices: [Voice; VOICES],
    bed: Bed,
    verb: Verb,
    rng: Rng,
    mix: [i32; BLOCK],
    bus: [i32; BLOCK],
    bus_lp: [i32; 2],
    bus_a: i32,
    slots: [[i32; HALF]; SLOTS],
    routes: [[i32; 4]; SLOTS],
    last: [i32; 2],
    spill: Option<[i32; 2]>,
    map: usize,
    scene: Scene,
    timers: [u32; TIMERS],
    presence: u32,
    strides: u32,
    heart: u32,
    breath: u32,
    chase: f32,
}

fn door_character(door: u16) -> f32 {
    0.85 + ((door as u32).wrapping_mul(2_654_435_761) >> 24) as f32 * (0.3 / 255.0)
}

fn closest(ear: &Ear, points: impl Iterator<Item = (f32, f32)>) -> Option<(f32, f32)> {
    points.min_by(|a, b| ear.distance(a.0, a.1).partial_cmp(&ear.distance(b.0, b.1)).unwrap_or(Ordering::Equal))
}

fn prop_near(map: &Map<'_>, ear: &Ear, kinds: &[&str], within: f32) -> Option<(f32, f32)> {
    closest(ear, map.props.iter().filter(|p| kinds.contains(&p.kind)).map(|p| point(p.pos))).filter(|&(x, y)| ear.distance(x, y) <= within)
}

fn tile_near(map: &Map<'_>, ear: &Ear, tile: Tile) -> Option<(f32, f32)> {
    let (cx, cy) = (ear.x.floor() as i32, ear.y.floor() as i32);
    closest(
        ear,
        (cy - NEAR..=cy + NEAR)
            .flat_map(|y| (cx - NEAR..=cx + NEAR).map(move |x| (x, y)))
            .filter(|&(x, y)| map.tile(x, y) == tile)
            .map(|(x, y)| (x as f32 + 0.5, y as f32 + 0.5)),
    )
}

fn tile_centre(x: i32, y: i32) -> (f32, f32) {
    (x as f32 + 0.5, y as f32 + 0.5)
}

fn lit(map: &Map<'_>, x: f32, y: f32) -> bool {
    map.lights.iter().filter(|l| l.mode != LightMode::Dead).map(|l| point(l.pos)).any(|(lx, ly)| (lx - x).abs() + (ly - y).abs() < 1.5)
}

fn lock_spot(map: &Map<'_>, lock: u8) -> Option<(f32, f32)> {
    map.doors
        .iter()
        .find(|d| d.state == DoorState::Locked(lock))
        .map(|d| tile_centre(d.x, d.y))
        .or_else(|| map.portals.iter().find(|p| p.lock == Some(lock)).map(|p| tile_centre(p.x, p.y)))
        .or_else(|| map.props.iter().find(|p| p.lock == Some(lock)).map(|p| point(p.pos)))
        .or_else(|| (map.exit.lock == lock).then(|| ((map.exit.rect.x0 + map.exit.rect.x1 + 1) as f32 * 0.5, (map.exit.rect.y0 + map.exit.rect.y1 + 1) as f32 * 0.5)))
}

impl Audio {
    pub fn new(rate: u32) -> Audio {
        let rate = rate.max(8000) as f32 * 0.5;
        Audio {
            rate,
            voices: [Voice::IDLE; VOICES],
            bed: Bed::new(rate),
            verb: Verb::new(),
            rng: Rng::new(0xD0_99_10),
            mix: [0; BLOCK],
            bus: [0; BLOCK],
            bus_lp: [0; 2],
            bus_a: dsp::q(dsp::one_pole(MUFFLE_HZ, rate)),
            slots: [[0; HALF]; SLOTS],
            routes: std::array::from_fn(slot_gains),
            last: [0; 2],
            spill: None,
            map: 0,
            scene: Scene::Sobrado,
            timers: [0; TIMERS],
            presence: 0,
            strides: 0,
            heart: 0,
            breath: 0,
            chase: 0.0,
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
        let level = place.left + place.right;
        patches.iter().filter(|p| p.gain * level > INAUDIBLE).for_each(|p| {
            if let Some(i) = self.free_voice(p.prio) {
                self.voices[i].start(p, place, seed, self.rate);
            }
        });
    }

    fn tune(&mut self, world: &World) {
        let map = world.map();
        let id = map as *const Map<'static> as usize;
        if id == self.map {
            return;
        }
        self.map = id;
        self.scene = Scene::of(map);
        let beds = self.scene.beds();
        self.bed.set_scene(beds.chirp, beds.hum_hz, beds.buzzy, beds.wind_bright);
        let mut rng = self.rng;
        self.timers = std::array::from_fn(|i| self.scene.sporadics().get(i).map_or(u32::MAX, |s| scene::wait(s, &mut rng) / 2));
        self.rng = rng;
    }

    fn at(world: &World, ear: Option<&Ear>, x: f32, y: f32, area: u8) -> Place {
        ear.map_or(Place::CENTER, |e| e.place(world, x, y, area))
    }

    fn wide(world: &World, ear: Option<&Ear>, x: f32, y: f32, area: u8, floor: f32) -> Place {
        ear.map_or(Place::CENTER, |e| e.reach(world, x, y, area, floor))
    }

    fn of_player(world: &World, ear: Option<&Ear>, player: u8, listener: u8) -> Option<Place> {
        world.players().get(player as usize).map(|p| match player == listener {
            true => ear.map_or(Place::CENTER, |e| e.own(0.7071)),
            false => {
                let (x, y) = point(p.pos);
                Self::at(world, ear, x, y, p.area)
            }
        })
    }

    fn legend_event(&mut self, world: &World, ear: Option<&Ear>, event: Event) {
        let l = *world.legend();
        let (x, y) = point(l.pos);
        let floor = |s: u8| TEXTURES.get(s as usize).map_or(Floor::Wood, |(name, _)| Floor::of(name));
        let near = Self::at(world, ear, x, y, l.area);
        let wide = |f: f32| Self::wide(world, ear, x, y, l.area, f);
        match (l.kind, event) {
            (LegendKind::CorpoSeco, Event::LegendStep { surface }) => {
                self.strides = self.strides.wrapping_add(1);
                let s = legend::shuffle(floor(surface), &mut self.rng);
                self.play(&s, near);
                if self.strides % 4 == 0 {
                    let j = legend::joints(&mut self.rng);
                    self.play(&j, near);
                }
            }
            (LegendKind::Cabra, Event::LegendStep { surface }) => self.play(&legend::clop(floor(surface)), near),
            (LegendKind::Mula, Event::LegendStep { surface }) => {
                self.strides = self.strides.wrapping_add(1);
                self.play(&legend::gallop(floor(surface), l.mode == LegendMode::Chase), wide(0.1));
                if self.strides % 2 == 0 {
                    let c = legend::chains(&mut self.rng);
                    self.play(&c, near);
                }
            }
            (LegendKind::Matinta, Event::LegendStep { .. }) => {
                self.strides = self.strides.wrapping_add(1);
                if self.strides % 5 == 0 {
                    self.play(&legend::wings(), near);
                }
            }
            (LegendKind::CorpoSeco, Event::LegendCry) => self.play(&legend::rasping_cry(), wide(0.06)),
            (LegendKind::Cabra, Event::LegendCry) => self.play(&legend::bleat_scream(), wide(0.3)),
            (LegendKind::Mula, Event::LegendCry | Event::LegendSpecial) | (LegendKind::Cabra, Event::LegendSpecial) => self.play(&legend::fire_roar(), wide(0.08)),
            (LegendKind::Matinta, Event::LegendCry) => {
                let w = legend::whistle(&mut self.rng);
                self.play(&w, wide(0.15));
            }
            (LegendKind::CorpoSeco, Event::LegendSpecial | Event::Stunned) => self.play(&legend::recoil(), near),
            (LegendKind::Matinta, Event::LegendSpecial) => self.play(&legend::laugh(), near),
            (LegendKind::Cabra, Event::Stunned) => self.play(&legend::splash_sizzle(), near),
            (LegendKind::Mula, Event::Stunned) => self.play(&legend::crash(), wide(0.1)),
            (LegendKind::Matinta, Event::Stunned) => self.play(&legend::wings(), near),
            _ => {}
        }
    }

    fn event(&mut self, world: &World, ear: Option<&Ear>, event: Event, listener: u8) {
        let map = world.map();
        let door = |d: u16| map.doors.get(d as usize).map(|d| (tile_centre(d.x, d.y), map.area_at(d.x, d.y).unwrap_or(0)));
        let pos = |p: u8| world.players().get(p as usize).map(|p| (point(p.pos), p.area));
        let own = |p: u8| p == listener;
        match event {
            Event::Step { player, surface, run } => {
                if let Some(((x, y), area)) = pos(player) {
                    let floor = TEXTURES.get(surface as usize).map_or(Floor::Wood, |(name, _)| Floor::of(name));
                    let build = Build::of(world.players()[player as usize].character);
                    let step = cue::footstep(floor, run, build, &mut self.rng);
                    let under = cue::underfoot(floor, run, &mut self.rng);
                    let place = Self::at(world, ear, x, y, area);
                    self.play(&[step], place);
                    if let Some(u) = under {
                        self.play(&[u], place);
                    }
                }
            }
            Event::LegendStep { .. } | Event::LegendCry | Event::LegendSpecial | Event::Stunned => self.legend_event(world, ear, event),
            Event::DoorMove { door: d, opening } => {
                if let Some(((x, y), area)) = door(d) {
                    let c = door_character(d);
                    let place = Self::at(world, ear, x, y, area);
                    match opening {
                        true => self.play(&[cue::latch(0.0, c), cue::creak(true, c)], place),
                        false => self.play(&[cue::creak(false, c), cue::slam(0.55), cue::latch(0.57, c)], place),
                    }
                }
            }
            Event::DoorRattle { door: d } => {
                if let Some(((x, y), area)) = door(d) {
                    self.play(&cue::rattle(door_character(d)), Self::at(world, ear, x, y, area));
                }
            }
            Event::Pickup { player, item } | Event::Drop { player, item } => {
                let stuff = world.items().get(item as usize).map(|i| Stuff::of(i.kind));
                if let Some((stuff, place)) = stuff.zip(Self::of_player(world, ear, player, listener)) {
                    match event {
                        Event::Pickup { .. } => self.play(&cue::pickup(stuff), place),
                        _ => self.play(&cue::drop(stuff), place),
                    }
                }
            }
            Event::Used { player, item } => {
                let kind = world.items().get(item as usize).map(|i| i.kind);
                if let Some((kind, place)) = kind.zip(Self::of_player(world, ear, player, listener)) {
                    match kind {
                        ItemKind::Sal => self.play(&[cue::pour()], place),
                        ItemKind::Balde => self.play(&cue::splash(), place),
                        ItemKind::Tabaco | ItemKind::Vela | ItemKind::Fosforo => self.play(&cue::pickup(Stuff::Cloth), place),
                        _ => {}
                    }
                }
            }
            Event::Unlock { lock } => {
                if let Some(l) = map.locks.get(lock as usize) {
                    let place = lock_spot(map, lock).map_or(Place::CENTER, |(x, y)| Self::at(world, ear, x, y, area_near(map, x, y).unwrap_or(0)));
                    self.play(&cue::unlock(&l.kind), place);
                }
            }
            Event::DialTick { player } => {
                if let Some(place) = Self::of_player(world, ear, player, listener) {
                    self.play(&[cue::dial_tick()], place);
                }
            }
            Event::DialWrong { player } => {
                if let Some(place) = Self::of_player(world, ear, player, listener) {
                    self.play(&cue::dial_wrong(), place);
                }
            }
            Event::Container { prop, .. } => {
                if let Some(p) = map.props.get(prop as usize) {
                    let (x, y) = point(p.pos);
                    let metal = matches!(p.kind, "cofre" | "geladeira" | "geladeira_bar" | "caixa_correio" | "quadro_fusivel" | "lixeira");
                    self.play(&cue::container(metal), Self::at(world, ear, x, y, area_near(map, x, y).unwrap_or(0)));
                }
            }
            Event::PortalUse { player, portal } => {
                if let Some((p, place)) = map.portals.get(portal as usize).zip(Self::of_player(world, ear, player, listener)) {
                    self.play(&cue::portal(p.kind), place);
                }
            }
            Event::Hide { player } | Event::Unhide { player } => {
                if let Some(((x, y), _)) = pos(player) {
                    let spot = match world.players()[player as usize].state {
                        PlayerState::Hidden { spot } => map.hides.get(spot as usize),
                        _ => None,
                    };
                    let kind = spot
                        .or_else(|| map.hides.iter().find(|h| (h.x as f32 + 0.5 - x).abs() + (h.y as f32 + 0.5 - y).abs() < 2.5))
                        .map_or(HideKind::GuardaRoupa, |h| h.kind);
                    if let Some(place) = Self::of_player(world, ear, player, listener) {
                        self.play(&cue::hide(kind, matches!(event, Event::Hide { .. })), place);
                    }
                }
            }
            Event::Grabbed { player } => {
                if let Some(place) = Self::of_player(world, ear, player, listener) {
                    self.play(&cue::hit(own(player)), if own(player) { Place::CENTER } else { place });
                }
            }
            Event::Fainted { player } => match own(player) {
                true => self.play(&cue::faint(), Place::CENTER),
                false => {
                    if let Some(place) = Self::of_player(world, ear, player, listener) {
                        self.play(&[cue::slump()], place);
                    }
                }
            },
            Event::Woke { player } => {
                if let Some(place) = Self::of_player(world, ear, player, listener) {
                    self.play(&cue::gasp(own(player)), place);
                }
            }
            Event::Lantern { player, on } => {
                if let Some(place) = Self::of_player(world, ear, player, listener) {
                    self.play(&[cue::lantern(on)], place);
                }
            }
            Event::PowerOut | Event::PowerOn => {
                let at = |kind: &str| map.props.iter().find(|p| p.kind == kind).map(|p| point(p.pos)).map(|(x, y)| Self::at(world, ear, x, y, area_near(map, x, y).unwrap_or(0)));
                self.play(&cue::clunk(), at("quadro_fusivel").unwrap_or(Place::CENTER));
                if matches!(event, Event::PowerOut) {
                    self.bed.hum.off();
                } else if let Some(place) = at("geladeira") {
                    self.play(&[cue::compressor()], place);
                }
            }
            Event::PhaseWon => self.play(&cue::chime(), Place::CENTER),
            Event::LegendSaw { .. }
            | Event::LegendLost
            | Event::Noise { .. }
            | Event::Portal { .. }
            | Event::Escaped { .. }
            | Event::PhaseRestart => {}
        }
    }

    pub fn events(&mut self, world: &World, events: &Events, listener: u8) {
        self.tune(world);
        let ear = Ear::new(world, listener);
        events.iter().for_each(|e| self.event(world, ear.as_ref(), *e, listener));
    }

    fn sporadic(&mut self, world: &World, ear: &Ear) {
        let sporadics = self.scene.sporadics();
        (0..sporadics.len().min(TIMERS)).for_each(|i| {
            self.timers[i] = self.timers[i].saturating_sub(1);
            if self.timers[i] > 0 {
                return;
            }
            let s = &sporadics[i];
            self.timers[i] = scene::wait(s, &mut self.rng).max(1);
            let place = match s.spot {
                Spot::Away => Some(ear.far(dsp::bipolar(&mut self.rng), FAR)),
                Spot::Prop(kind) => prop_near(world.map(), ear, &[kind], 60.0).map(|(x, y)| ear.place(world, x, y, area_near(world.map(), x, y).unwrap_or(ear.area))),
            };
            if let Some(place) = place {
                let p = scene::far(s.sound, &mut self.rng);
                self.play(&p, place);
            }
        });
    }

    fn presence(&mut self, world: &World, ear: &Ear) {
        let l = *world.legend();
        let (x, y) = point(l.pos);
        let d = ear.distance(x, y);
        self.presence = self.presence.saturating_sub(1);
        let place = ear.place(world, x, y, l.area);
        match l.kind {
            LegendKind::CorpoSeco if self.presence == 0 => {
                self.presence = 180 + self.rng.below(120);
                if d < 20.0 {
                    self.play(&legend::rattle_breath(), place);
                }
            }
            LegendKind::Matinta if self.presence == 0 => {
                self.presence = 720 + self.rng.below(780);
                if d < 14.0 {
                    self.play(&legend::laugh(), place);
                }
            }
            _ => {}
        }
        let blaze = match l.kind {
            LegendKind::Mula => (place, 0.07),
            _ => (Place::CENTER, 0.0),
        };
        self.bed.blaze.aim(blaze.0, blaze.1);
    }

    fn chase(&mut self, world: &World, ear: &Ear, listener: u8) {
        let l = world.legend();
        let (x, y) = point(l.pos);
        let hunted = l.mode == LegendMode::Chase && l.target == Some(listener) && !ear.hidden;
        let goal = if hunted { 1.0 } else { 0.0 };
        self.chase += (goal - self.chase) * if goal > self.chase { 0.05 } else { 0.008 };
        self.bed.set_tension(self.chase);
        self.bed.drone.aim(ear.own(1.0), 0.11 * self.chase * self.chase.sqrt());
        self.heart = self.heart.saturating_sub(1);
        if self.chase > 0.15 && self.heart == 0 {
            self.heart = (60.0 - 33.0 * self.chase) as u32;
            self.play(&cue::heartbeat(self.chase), Place::CENTER);
        }
        self.breath = match ear.hidden {
            true => self.breath.saturating_sub(1),
            false => 30,
        };
        if ear.hidden && self.breath == 0 {
            let fast = ear.distance(x, y) < 5.0 && l.area == ear.area;
            self.breath = if fast { 96 } else { 150 };
            self.play(&cue::breath(fast), Place::CENTER);
        }
    }

    pub fn ambience(&mut self, world: &World, listener: u8) {
        self.tune(world);
        let Some(ear) = Ear::new(world, listener) else {
            self.bed.fade();
            self.verb.aim(0.0);
            return;
        };
        let map = world.map();
        let beds = self.scene.beds();
        let sobrado = self.scene == Scene::Sobrado;
        let placed = |p: Option<(f32, f32)>, level: f32| p.map_or((Place::CENTER, 0.0), |(x, y)| (ear.place(world, x, y, area_near(map, x, y).unwrap_or(ear.area)), level));
        let hum = match self.scene {
            Scene::Sobrado => placed(prop_near(map, &ear, &["geladeira"], 40.0).filter(|_| world.power()), 0.05),
            Scene::Cidade => placed(prop_near(map, &ear, &["poste"], 10.0).filter(|&(x, y)| world.power() && lit(map, x, y)), 0.035),
            _ => (Place::CENTER, 0.0),
        };
        let clock = placed(prop_near(map, &ear, &["relogio_parede"], 12.0).filter(|&(x, y)| sobrado && ear.room.is_some() && space::room_near(map, x, y) == ear.room), 0.12);
        let wind = match ear.out {
            true => (ear.far(0.0, beds.wind.0), 1.0),
            false => placed(tile_near(map, &ear, Tile::Window), beds.wind.1),
        };
        let river = placed((self.scene == Scene::Mata).then(|| tile_near(map, &ear, Tile::Water)).flatten(), 0.08);
        let candle = placed(prop_near(map, &ear, &["vela_castical", "vela_chao", "oratorio", "fogao_lenha", "forno_farinha"], 6.0), 0.05);
        self.bed.hum.aim(hum.0, hum.1);
        self.bed.clock.aim(clock.0, clock.1);
        self.bed.wind.aim(wind.0, wind.1);
        self.bed.river.aim(river.0, river.1);
        self.bed.candle.aim(candle.0, candle.1);
        self.bed.chirr.aim(ear.far(0.25, beds.chirr), 1.0);
        self.bed.tone.aim(ear.around(1.0), if ear.out { beds.tone.0 } else { beds.tone.1 });
        self.verb.aim(if ear.out { 0.0 } else { beds.verb });
        self.sporadic(world, &ear);
        self.presence(world, &ear);
        self.chase(world, &ear, listener);
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
        let used = self.voices.iter().filter(|v| v.live()).fold(0u32, |m, v| m | 1 << v.slot());
        let slots = &mut self.slots;
        slots.iter_mut().enumerate().filter(|(i, _)| used & 1 << i != 0).for_each(|(_, s)| s[..frames].fill(0));
        self.voices.iter_mut().filter(|v| v.live()).for_each(|v| v.render(&mut slots[v.slot()][..frames]));
        let pan = |dst: &mut [i32], s: &[i32], l: i32, r: i32| {
            dst.chunks_exact_mut(2).zip(s).for_each(|(d, y)| {
                d[0] += dsp::mulq(*y, l);
                d[1] += dsp::mulq(*y, r);
            })
        };
        slots.iter().zip(&self.routes).enumerate().filter(|(i, _)| used & 1 << i != 0).for_each(|(_, (s, g))| {
            if g[0] != 0 || g[1] != 0 {
                pan(mix, &s[..frames], g[0], g[1]);
            }
            if g[2] != 0 || g[3] != 0 {
                pan(bus, &s[..frames], g[2], g[3]);
            }
        });
        self.verb.process(mix);
        self.bed.render(mix, bus);
        let (a, mut lp) = (self.bus_a, self.bus_lp);
        mix.chunks_exact_mut(2).zip(bus.chunks_exact(2)).for_each(|(d, m)| {
            lp = [lp[0] + dsp::mulq(m[0] - lp[0], a), lp[1] + dsp::mulq(m[1] - lp[1], a)];
            d[0] += lp[0];
            d[1] += lp[1];
        });
        self.bus_lp = lp;
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
