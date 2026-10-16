use super::alloc;
use super::bed::INSECTS;
use super::cue::{self, Build, Floor, Stuff};
use super::legend;
use super::scene::{self, Far, Scene};
use super::space::{Ear, occlusion};
use super::voice::{Patch, Place};
use super::{Audio, VOICES};
use crate::assets::TEXTURES;
use crate::fx::Rng;
use crate::map::{self, HideKind, ItemKind, LockKind, Map, PortalKind};
use crate::sim::{Event, Setup, World};
use std::sync::LazyLock;
use std::time::Instant;

const RATE: u32 = 32000;
const FPS: usize = 60;
const SCENES: [(&str, &str, &str); 5] = [
    ("sobrado", "corposeco", ""),
    ("cidade", "cabra", ""),
    ("igreja", "mula", ""),
    ("mata", "matinta", ""),
    ("cemiterio", "corposeco", "lapide 15.5 2.5 0\n"),
];

const FIXTURE: &str = "[map] name=\"Audio NAME\" w=20 h=11 sky=moon fog=10,12,20 fog_near=3 fog_far=14 legend=LEGEND power=on
[solid]
####W#####||||||||||
#........#.........|
#.B......#.........|
#........D.........|
#.H......#......~~.|
#....P...W......~~.|
#........#.........|
##########||||||||||
#####               
#.P.#               
#####               
[rooms]
--------------------
-aaaaaaaa-bbbbbbbbb-
-aaaaaaaa-bbbbbbbbb-
-aaaaaaaaabbbbbbbbb-
-aaaaaaaa-bbbbbbbbb-
-aaaaaaaa-bbbbbbbbb-
-aaaaaaaa-bbbbbbbbb-
--------------------
-----               
-ccc-               
-----               
[area]
0 terreo
1 sotao
[room]
a sala area=0 floor=taco wall=reboco ceil=forro light=40
b quintal area=0 floor=paralelepipedo wall=tijolo ceil=- h=3 light=12 out=1
c sotao area=1 floor=assoalho wall=reboco ceil=forro light=20
[item]
k chave 3 1
al alicate 6.5 3.5
fu fusivel 3 6
cr cruz 12 3
bd balde 13 2
sl sal 14 6
[lock]
dl key k
cl code 317
ch tool alicate
pl place fu 3 2
sa all ch pl
[door]
9 3 v locked:dl
[portal]
5 5 -> 2 9 0 escada
2 9 -> 5 4 0 escada
[prop]
geladeira 2.5 2.5 0
relogio_parede 4.5 1.2 0
quadro_fusivel 7.5 1.2 0
bau 7.5 6.5 0 lock=cl item=cr
vela_castical 3.5 5.5 0
sino 1.5 6.5 0
poste 12.5 1.5 0
portao_ferro 18.5 3.5 90
EXTRA[hide]
2 4 guarda_roupa 90
[light]
12.5 1.5 3 255,200,120 6 flicker
[cam]
0 0 9 7 5 2.4 6 5 0 3 60
10 0 19 7 14 3 7 14 0 3 60
0 8 4 10 2 2.4 10 2 0 9 60
[wake]
0 3 3 0
1 4 3 0
2 11 3 90
3 12 4 90
[legend]
spawn 15 2 180
patrol 12 2
patrol 15 5
patrol 11 6
patrol 5 2
patrol 4 5
patrol 18 6
lair 17 2
[exit]
17 1 18 2 lock=sa
";

static SOURCES: LazyLock<[String; 5]> = LazyLock::new(|| SCENES.map(|(name, legend, extra)| FIXTURE.replace("NAME", name).replace("LEGEND", legend).replace("EXTRA", extra)));
static MAPS: LazyLock<Vec<Map<'static>>> =
    LazyLock::new(|| SOURCES.iter().zip(SCENES).map(|(src, (name, ..))| map::parse(src).unwrap_or_else(|e| panic!("{name}: {e}"))).collect());

fn fixture(scene: usize) -> &'static Map<'static> {
    &MAPS[scene]
}

fn stage(scene: usize) -> (&'static Map<'static>, &'static str) {
    let kind = Scene::of(fixture(scene));
    map::campaign().iter().find(|m| Scene::of(m) == kind).map_or((fixture(scene), "fixture"), |m| (*m, "campaign"))
}

fn world(map: &'static Map<'static>) -> World {
    World::new(map, Setup { seed: 7, phase: 0, players: 2, characters: [2, 1, 0, 3] })
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
    let floors = ["taco", "assoalho", "lajota", "marmore_xadrez", "paralelepipedo", "asfalto", "folhas_chao", "grama", "barro", "agua_rio"];
    let mut ev = Vec::new();
    if every(0.5, 0.2, 9.5) {
        let step = (t * 2.0) as usize;
        ev.push(Event::Step { player: (step % 2) as u8, surface: surface(floors[step / 2 % floors.len()]), run: step % 3 == 0 });
    }
    if every(0.6, 0.3, 9.6) {
        ev.push(Event::LegendStep { surface: surface(floors[(t * 3.0) as usize % floors.len()]) });
    }
    if every(0.1, 4.0, 4.3) {
        ev.push(Event::DialTick { player: 0 });
    }
    [
        (24, Event::Lantern { player: 0, on: true }),
        (48, Event::DoorMove { door: 0, opening: true }),
        (60, Event::LegendCry),
        (90, Event::Pickup { player: 0, item: 0 }),
        (120, Event::Pickup { player: 1, item: 2 }),
        (132, Event::Pickup { player: 0, item: 5 }),
        (150, Event::DoorMove { door: 0, opening: false }),
        (168, Event::Container { player: 0, prop: 3 }),
        (180, Event::DoorRattle { door: 0 }),
        (198, Event::Drop { player: 1, item: 1 }),
        (216, Event::Drop { player: 0, item: 4 }),
        (234, Event::PortalUse { player: 1, portal: 0 }),
        (264, Event::DialWrong { player: 0 }),
        (270, Event::LegendSpecial),
        (300, Event::Unlock { lock: 0 }),
        (324, Event::Unlock { lock: 1 }),
        (348, Event::Unlock { lock: 2 }),
        (360, Event::Used { player: 0, item: 5 }),
        (378, Event::Unlock { lock: 3 }),
        (396, Event::Used { player: 1, item: 4 }),
        (408, Event::Unlock { lock: 4 }),
        (420, Event::Stunned),
        (444, Event::Hide { player: 0 }),
        (468, Event::PowerOut),
        (480, Event::Unhide { player: 0 }),
        (498, Event::Grabbed { player: 1 }),
        (516, Event::Fainted { player: 1 }),
        (528, Event::PowerOn),
        (552, Event::Woke { player: 1 }),
        (564, Event::Grabbed { player: 0 }),
        (570, Event::Fainted { player: 0 }),
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

fn db(x: f64) -> f64 {
    20.0 * x.max(1e-9).log10()
}

fn render_scene(name: &str, map: &'static Map<'static>) -> (f32, f64, u64) {
    let mut world = world(map);
    let mut audio = Audio::new(RATE);
    let mut pcm = Vec::with_capacity(RATE as usize * 2 * 11);
    let mut buf = vec![0i16; 1068];
    let inputs = |frame: usize| [if frame % 120 < 80 { 1u16 << 4 } else { 1 << 6 }, 1 << 4 | 1 << 7, 0, 0];
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
            assert!(audio.voices.iter().all(|v| v.sane()), "{name}: NaN in voice state at frame {frame}");
            pcm.extend_from_slice(out);
            n
        })
        .sum();
    write_wav(&format!("/tmp/doppio-audio-{name}.wav"), &pcm);
    let peak = pcm.iter().map(|s| (*s as f32 / 32768.0).abs()).fold(0.0, f32::max);
    let rms = (pcm.iter().map(|s| (*s as f64 / 32768.0).powi(2)).sum::<f64>() / pcm.len() as f64).sqrt();
    assert_eq!(pcm.len(), RATE as usize * 10 * 2);
    (peak, rms, allocs)
}

#[test]
fn scripted_scene_per_map_renders_clean_wav() {
    SCENES.iter().enumerate().for_each(|(i, (name, ..))| {
        assert_eq!(format!("{:?}", Scene::of(fixture(i))).to_lowercase(), *name);
        let (map, from) = stage(i);
        let (peak, rms, allocs) = render_scene(name, map);
        println!("{name} ({from} map): peak {:.1} dBFS, rms {:.1} dBFS, allocs in fill {allocs}", db(peak as f64), db(rms));
        assert_eq!(allocs, 0, "{name}");
        assert!(peak < 1.0 && peak > 0.05, "{name}: peak {peak}");
        assert!((0.003..0.2).contains(&rms), "{name}: rms {rms}");
    });
}

#[test]
fn ambience_alone_is_audible_but_quiet() {
    SCENES.iter().enumerate().for_each(|(i, (name, ..))| {
        let (map, from) = stage(i);
        let world = world(map);
        let mut audio = Audio::new(RATE);
        let mut out = vec![0i16; 533 * 2];
        let pcm: Vec<i16> = (0..6 * FPS)
            .flat_map(|_| {
                audio.ambience(&world, 0);
                audio.fill(&mut out);
                out.clone()
            })
            .collect();
        let tail = &pcm[pcm.len() / 2..];
        let rms = (tail.iter().map(|s| (*s as f64 / 32768.0).powi(2)).sum::<f64>() / tail.len() as f64).sqrt();
        println!("{name} ({from} map) ambience: rms {:.1} dBFS", db(rms));
        assert!((0.0005..0.05).contains(&rms), "{name}: ambience rms {rms}");
    });
}

fn every_patch() -> Vec<Patch> {
    let mut rng = Rng::new(3);
    let floors = [Floor::Wood, Floor::Boards, Floor::Hard, Floor::Marble, Floor::Cobble, Floor::Grit, Floor::Leaves, Floor::Soft, Floor::Mud, Floor::Water];
    let builds = [Build::Old, Build::Light, Build::Heavy, Build::Plain];
    let stuffs = [Stuff::Metal, Stuff::Glass, Stuff::Cloth, Stuff::Wood];
    let locks = [
        LockKind::Key(0),
        LockKind::Code { digits: [3, 1, 7, 0], len: 3 },
        LockKind::Tool(ItemKind::Alicate),
        LockKind::Tool(ItemKind::PeDeCabra),
        LockKind::Tool(ItemKind::Martelo),
        LockKind::Tool(ItemKind::ChaveDeFenda),
        LockKind::Place { item: 0, x: 0, y: 0 },
        LockKind::All(vec![0, 1]),
    ];
    let portals = [PortalKind::Escada, PortalKind::EscadaMao, PortalKind::Bueiro, PortalKind::Portao, PortalKind::JanelaAlta];
    let hides = [
        HideKind::Cama,
        HideKind::GuardaRoupa,
        HideKind::Armario,
        HideKind::Tumulo,
        HideKind::CaixaDagua,
        HideKind::Confessionario,
        HideKind::Carro,
        HideKind::Cesto,
        HideKind::Mesa,
    ];
    let fars = [Far::Dogs, Far::Car, Far::Owl, Far::Frogs, Far::Frogs, Far::Crows, Far::BellCreak, Far::GateCreak];
    let mut all: Vec<Patch> = Vec::new();
    floors.iter().for_each(|&f| {
        builds.iter().for_each(|&b| all.push(cue::footstep(f, true, b, &mut rng)));
        (0..6).for_each(|_| all.extend(cue::underfoot(f, true, &mut rng)));
        all.extend(legend::shuffle(f, &mut rng));
        all.extend(legend::clop(f));
        all.extend(legend::gallop(f, true));
    });
    stuffs.iter().for_each(|&s| all.extend(cue::pickup(s).into_iter().chain(cue::drop(s))));
    locks.iter().for_each(|l| all.extend(cue::unlock(l)));
    portals.iter().for_each(|&p| all.extend(cue::portal(p)));
    hides.iter().for_each(|&h| all.extend(cue::hide(h, true).into_iter().chain(cue::hide(h, false))));
    fars.iter().for_each(|&f| all.extend(scene::far(f, &mut rng)));
    all.extend([cue::latch(0.0, 1.0), cue::creak(true, 1.15), cue::creak(false, 0.85), cue::slam(0.5), cue::lantern(true), cue::lantern(false)]);
    all.extend([cue::dial_tick(), cue::compressor(), cue::slump(), cue::pour()]);
    all.extend(cue::rattle(1.0).into_iter().chain(cue::clunk()).chain(cue::dial_wrong()).chain(cue::container(true)).chain(cue::container(false)));
    all.extend(cue::hit(true).into_iter().chain(cue::faint()).chain(cue::chime()));
    all.extend(cue::gasp(true).into_iter().chain(cue::breath(true)).chain(cue::breath(false)).chain(cue::heartbeat(1.0)).chain(cue::splash()));
    all.extend(legend::joints(&mut rng).into_iter().chain(legend::rattle_breath()).chain(legend::recoil()).chain(legend::wings()));
    all.extend(legend::rasping_cry().into_iter().chain(legend::fire_roar()));
    all.extend(legend::bleat_scream().into_iter().chain(legend::crash()));
    all.extend(legend::splash_sizzle().into_iter().chain(legend::chains(&mut rng)).chain(legend::whistle(&mut rng)).chain(legend::laugh()));
    all
}

#[test]
fn every_cue_is_finite_bounded_and_ends() {
    every_patch().iter().for_each(|p| {
        let mut audio = Audio::new(RATE);
        audio.play(&[*p], Place::CENTER);
        let mut out = vec![0i16; RATE as usize * 2];
        let mut peak = 0.0f32;
        let ended = (0..24).any(|_| {
            audio.fill(&mut out);
            assert!(audio.voices.iter().all(|v| v.sane()), "{p:?}");
            peak = out.iter().fold(peak, |m, s| m.max((*s as f32 / 32768.0).abs()));
            !audio.voices.iter().any(|v| v.live())
        });
        assert!(ended, "voice never ends: {p:?}");
        assert!(peak > 0.002 && peak < 0.71, "peak {peak} for {p:?}");
    });
}

#[test]
fn walls_occlude_floors_separate_and_hiding_muffles() {
    let map = fixture(0);
    let world = world(map);
    let ear = Ear { x: 4.5, y: 1.5, rx: 1.0, ry: 0.0, area: 0, room: map.room_index(4, 1), out: false, hidden: false };
    assert_eq!(occlusion(&world, (4.5, 1.5), (6.5, 4.5)), 0.0);
    assert!(occlusion(&world, (4.5, 1.5), (14.5, 1.5)) >= 1.0);
    let same = ear.place(&world, 6.5, 1.5, 0);
    let other = ear.place(&world, 14.5, 1.5, 0);
    let upstairs = ear.place(&world, 5.5, 1.5, 1);
    assert!(same.right > same.left);
    assert!(other.muffle > 0.8 && same.muffle < 0.1 && other.left + other.right < (same.left + same.right) * 0.3);
    assert!(upstairs.muffle > 0.9 && upstairs.left + upstairs.right < (same.left + same.right) * 0.3);
    let hidden = Ear { hidden: true, ..ear }.place(&world, 6.5, 1.5, 0);
    assert!(hidden.muffle >= 0.8 && hidden.left + hidden.right < same.left + same.right);
    let wide = ear.reach(&world, 18.5, 6.5, 1, 0.3);
    assert!(wide.left + wide.right >= 0.3 && wide.muffle <= 0.7);
    let outside = Ear { out: true, ..ear }.far(0.0, 1.0);
    let inside = ear.far(0.0, 1.0);
    assert!(inside.muffle > outside.muffle && inside.left < outside.left);
}

#[test]
fn voice_pool_steals_quiet_low_priority() {
    let mut audio = Audio::new(RATE);
    let step = cue::footstep(Floor::Wood, false, Build::Plain, &mut Rng::new(1));
    (0..VOICES).for_each(|_| audio.play(&[step], Place::CENTER));
    audio.play(&cue::faint(), Place::CENTER);
    assert_eq!(audio.voices.iter().filter(|v| v.prio() == cue::PRIO_SELF).count(), 3);
    audio.play(&[step], Place::CENTER);
    assert_eq!(audio.voices.iter().filter(|v| v.prio() == cue::PRIO_SELF).count(), 3);
    audio.play(&legend::whistle(&mut Rng::new(2)), Place::CENTER);
    assert_eq!(audio.voices.iter().filter(|v| v.prio() == cue::PRIO_LEGEND).count(), 2);
}

#[test]
fn map_change_retunes() {
    let mut audio = Audio::new(RATE);
    let mata = world(fixture(3));
    audio.ambience(&mata, 0);
    assert_eq!(audio.scene, Scene::Mata);
    assert!(audio.timers[..3].iter().all(|t| *t > 0 && *t < u32::MAX) && audio.timers[3] == u32::MAX);
    let mut out = vec![0i16; 533 * 2];
    (0..120).for_each(|_| {
        audio.ambience(&mata, 0);
        audio.fill(&mut out);
    });
    assert!(out.iter().any(|s| *s != 0));
    let cidade = world(fixture(1));
    audio.ambience(&cidade, 0);
    assert_eq!(audio.scene, Scene::Cidade);
    assert!(audio.timers[..2].iter().all(|t| *t > 0 && *t < u32::MAX) && audio.timers[2..].iter().all(|t| *t == u32::MAX));
}

fn bench(load: &[Patch]) -> (f64, f64, u64) {
    let mut audio = Audio::new(RATE);
    let place = Place { left: 0.5, right: 0.4, muffle: 0.3 };
    audio.bed.set_scene(INSECTS, 120.0, true, 0.7);
    [&mut audio.bed.wind, &mut audio.bed.tone, &mut audio.bed.drone, &mut audio.bed.chirr, &mut audio.bed.river, &mut audio.bed.candle, &mut audio.bed.blaze]
        .into_iter()
        .for_each(|l| l.aim(place, 0.03));
    audio.bed.set_tension(1.0);
    audio.verb.aim(0.3);
    load.iter().enumerate().for_each(|(i, p)| {
        let spot = Place { left: 0.1 + 0.06 * i as f32, right: 0.9 - 0.06 * i as f32, muffle: (i % 3) as f32 * 0.45 };
        audio.play(&[Patch { hold: 60.0, pulses: 1, delay: 0.0, ..*p }], spot)
    });
    let mut out = vec![0i16; 533 * 2];
    let mut batches = vec![0.0f64; 200];
    let allocs = alloc::count(|| {
        batches.iter_mut().for_each(|b| {
            let start = Instant::now();
            (0..10).for_each(|_| audio.fill(&mut out));
            *b = start.elapsed().as_secs_f64() * 1e5;
        })
    });
    assert_eq!(audio.voices.iter().filter(|v| v.live()).count(), load.len());
    batches.sort_by(f64::total_cmp);
    (batches[0], batches[batches.len() / 2], allocs)
}

#[test]
fn fill_bench() {
    let mut rng = Rng::new(5);
    let heavy = [
        cue::footstep(Floor::Wood, true, Build::Plain, &mut rng),
        legend::fire_roar()[2],
        legend::bleat_scream()[1],
        cue::creak(true, 1.0),
        legend::whistle(&mut rng)[1],
        legend::chains(&mut rng)[0],
    ];
    let load = |n: usize| (0..n).map(|v| heavy[v % heavy.len()]).collect::<Vec<_>>();
    [("worst: every voice busy", load(VOICES)), ("busy: 8 voices", load(8)), ("ambience only", Vec::new())].iter().for_each(|(name, load)| {
        let (min, median, allocs) = bench(load);
        println!("fill bench {name} + 7 beds + reverb: min {min:.1} us, median {median:.1} us per 533-frame fill, box estimate x25 {:.0} us", median * 25.0);
        assert_eq!(allocs, 0);
    });
}
