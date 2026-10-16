use super::alloc;
use super::cue;
use super::space::{Ear, occlusion};
use super::voice::Place;
use super::{Audio, VOICES};
use crate::assets::TEXTURES;
use crate::map;
use crate::sim::{Event, Setup, World};
use std::time::Instant;

const RATE: u32 = 32000;
const FPS: usize = 60;

fn world() -> World {
    World::new(map::sobrado(), Setup { seed: 7, players: 2, night_secs: 60 })
}

fn surface(name: &str) -> u8 {
    TEXTURES.iter().position(|t| t.0 == name).unwrap_or(0) as u8
}

fn frames(n: usize) -> usize {
    RATE as usize * (n + 1) / FPS - RATE as usize * n / FPS
}

fn script(frame: usize) -> Vec<Event> {
    let t = frame as f32 / FPS as f32;
    let every = |period: f32, from: f32, to: f32| t >= from && t < to && frame % (period * FPS as f32) as usize == 0;
    let floors = ["taco", "lajota", "grama", "concreto", "ceramica", "terra"];
    let mut ev = Vec::new();
    if frame == 0 {
        ev.push(Event::NightBegins { night: 3 });
    }
    if every(0.5, 0.2, 9.5) {
        let step = (t * 2.0) as usize;
        ev.push(Event::Step { body: (step % 2) as u8, surface: surface(floors[step / 4 % floors.len()]) });
    }
    if every(0.7, 4.0, 7.5) {
        ev.push(Event::DoorRattle { door: 2 });
    }
    [
        (60, Event::DoorMove { door: 0, opening: true }),
        (150, Event::DoorMove { door: 0, opening: false }),
        (100, Event::Lantern { body: 0, on: true }),
        (130, Event::Lantern { body: 1, on: false }),
        (170, Event::PowerOut),
        (220, Event::TaskDone { task: 0 }),
        (300, Event::Glitch { body: 1 }),
        (360, Event::PowerOn),
        (450, Event::Captured { player: 0 }),
        (500, Event::DawnBreaks { night: 3 }),
    ]
    .iter()
    .filter(|(f, _)| *f == frame)
    .for_each(|(_, e)| ev.push(*e));
    ev
}

fn write_wav(path: &str, pcm: &[i16]) {
    let data = (pcm.len() * 2) as u32;
    let header = [
        b"RIFF".as_slice(),
        &(36 + data).to_le_bytes(),
        b"WAVEfmt ",
        &16u32.to_le_bytes(),
        &1u16.to_le_bytes(),
        &2u16.to_le_bytes(),
        &RATE.to_le_bytes(),
        &(RATE * 4).to_le_bytes(),
        &4u16.to_le_bytes(),
        &16u16.to_le_bytes(),
        b"data",
        &data.to_le_bytes(),
    ]
    .concat();
    let body = pcm.iter().flat_map(|s| s.to_le_bytes());
    std::fs::write(path, header.into_iter().chain(body).collect::<Vec<u8>>()).expect("write wav");
}

#[test]
fn scripted_scene_renders_clean_wav() {
    let mut world = world();
    let mut audio = Audio::new(RATE);
    let mut pcm = Vec::with_capacity(RATE as usize * 2 * 11);
    let mut buf = vec![0i16; 1068];
    let inputs = |frame: usize| [if frame % 120 < 80 { 1u16 << 4 } else { 1 << 6 }, 1 << 5, 0, 0];
    let allocs: u64 = (0..10 * FPS)
        .map(|frame| {
            if frame % 2 == 0 {
                world.step(&inputs(frame));
                audio.events(&world, world.events(), 0);
            }
            let ear = Ear::new(&world, 0);
            script(frame).into_iter().for_each(|e| audio.event(&world, ear.as_ref(), e, 0));
            audio.ambience(&world, 0);
            let out = &mut buf[..frames(frame) * 2];
            let n = alloc::count(|| audio.fill(out));
            assert!(audio.voices.iter().all(|v| v.sane()), "NaN in voice state at frame {frame}");
            pcm.extend_from_slice(out);
            n
        })
        .sum();
    write_wav("/tmp/doppio-audio.wav", &pcm);
    let peak = pcm.iter().map(|s| (*s as f32 / 32768.0).abs()).fold(0.0, f32::max);
    let rms = (pcm.iter().map(|s| (*s as f64 / 32768.0).powi(2)).sum::<f64>() / pcm.len() as f64).sqrt();
    let db = |x: f64| 20.0 * x.log10();
    println!("scene: {} frames, peak {:.1} dBFS, rms {:.1} dBFS, allocs in fill {allocs}", pcm.len() / 2, db(peak as f64), db(rms));
    assert_eq!(pcm.len(), RATE as usize * 10 * 2);
    assert_eq!(allocs, 0);
    assert!(peak < 1.0 && peak > 0.05, "peak {peak}");
    assert!((0.003..0.2).contains(&rms), "rms {rms}");
}

#[test]
fn every_cue_is_finite_bounded_and_ends() {
    let groups: Vec<Vec<super::voice::Patch>> = vec![
        vec![cue::footstep(cue::Floor::Wood, true, &mut crate::fx::Rng::new(3))],
        vec![cue::footstep(cue::Floor::Hard, true, &mut crate::fx::Rng::new(3))],
        vec![cue::footstep(cue::Floor::Soft, true, &mut crate::fx::Rng::new(3))],
        vec![cue::footstep(cue::Floor::Concrete, false, &mut crate::fx::Rng::new(3))],
        vec![cue::latch(0.0, 1.0), cue::creak(true, 1.15), cue::creak(false, 0.85), cue::slam(0.5)],
        cue::rattle(1.0).to_vec(),
        vec![cue::lantern(true), cue::lantern(false), cue::compressor(), cue::bell(5)],
        cue::chime().to_vec(),
        cue::clunk().to_vec(),
        cue::captured().to_vec(),
        cue::rooster().to_vec(),
        cue::birds().to_vec(),
        cue::drip(1.0).to_vec(),
    ];
    groups.iter().flatten().for_each(|p| {
        let mut audio = Audio::new(RATE);
        audio.play(&[*p], Place::CENTER);
        let mut out = vec![0i16; RATE as usize * 2];
        let mut peak = 0.0f32;
        let live = (0..24).any(|_| {
            audio.fill(&mut out);
            assert!(audio.voices.iter().all(|v| v.sane()), "{p:?}");
            peak = out.iter().fold(peak, |m, s| m.max((*s as f32 / 32768.0).abs()));
            !audio.voices.iter().any(|v| v.live())
        });
        assert!(live, "voice never ends: {p:?}");
        assert!(peak > 0.003 && peak < 0.71, "peak {peak} for {p:?}");
    });
}

#[test]
fn walls_occlude_and_rooms_muffle() {
    let world = world();
    let ear = Ear { x: 4.5, y: 5.5, rx: 1.0, ry: 0.0, room: Some(b's') };
    assert_eq!(occlusion(&world, (4.5, 5.5), (6.5, 8.5)), 0.0);
    assert!(occlusion(&world, (4.5, 5.5), (19.5, 5.5)) >= 1.0);
    let same = ear.place(&world, 6.5, 5.5);
    let other = ear.place(&world, 19.5, 5.5);
    assert!(same.right > same.left);
    assert!(other.muffle > 0.8 && same.muffle < 0.1 && other.left + other.right < (same.left + same.right) * 0.3);
}

#[test]
fn voice_pool_steals_quiet_low_priority() {
    let mut audio = Audio::new(RATE);
    let step = cue::footstep(cue::Floor::Wood, false, &mut crate::fx::Rng::new(1));
    (0..VOICES).for_each(|_| audio.play(&[step], Place::CENTER));
    audio.play(&cue::captured(), Place::CENTER);
    assert_eq!(audio.voices.iter().filter(|v| v.prio() == cue::PRIO_SELF).count(), 3);
    audio.play(&[step], Place::CENTER);
    assert_eq!(audio.voices.iter().filter(|v| v.prio() == cue::PRIO_SELF).count(), 3);
}

fn bench(load: &[super::voice::Patch]) -> (f64, f64, u64) {
    let mut audio = Audio::new(RATE);
    let place = Place { left: 0.5, right: 0.4, muffle: 0.3 };
    [&mut audio.bed.hum, &mut audio.bed.clock, &mut audio.bed.wind, &mut audio.bed.tv, &mut audio.bed.tone, &mut audio.bed.drone]
        .into_iter()
        .for_each(|l| l.aim(place, 0.03));
    audio.bed.set_tension(1.0);
    let mut out = vec![0i16; 533 * 2];
    let mut batches = vec![0.0f64; 200];
    let allocs = alloc::count(|| {
        batches.iter_mut().for_each(|b| {
            audio.warble.kick(0.8);
            load.iter().for_each(|p| audio.play(&[*p], place));
            let start = Instant::now();
            (0..10).for_each(|_| audio.fill(&mut out));
            *b = start.elapsed().as_secs_f64() * 1e5;
        })
    });
    batches.sort_by(f64::total_cmp);
    (batches[0], batches[batches.len() / 2], allocs)
}

#[test]
fn fill_bench() {
    let mut rng = crate::fx::Rng::new(5);
    let worst: Vec<_> = (0..VOICES).map(|v| [cue::creak(true, 1.0), cue::bell(3), cue::rattle(1.0)[0]][v % 3]).collect();
    let floors = [cue::Floor::Wood, cue::Floor::Hard, cue::Floor::Soft, cue::Floor::Concrete];
    let typical: Vec<_> = (0..8).map(|i| cue::footstep(floors[i % 4], i % 3 == 0, &mut rng)).chain(cue::rattle(1.0)).collect();
    [("worst: all voices heavy", worst), ("typical: 8 steps + rattle", typical)].iter().for_each(|(name, load)| {
        let (min, median, allocs) = bench(load);
        println!("fill bench {name} + all beds + warble: min {min:.1} us, median {median:.1} us per 533-frame fill");
        assert_eq!(allocs, 0);
    });
}
