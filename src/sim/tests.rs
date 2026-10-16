use super::capture::{FADE_TICS, GRACE_TICS, SWAP_AT};
use super::echo::Echo;
use super::finale::{HOLD_TICS, OPENS_AFTER_HOURS};
use super::night::at_hour;
use super::tank::{A, B, DOWN, LEFT, RIGHT, UP, X};
use super::tape::tape_id;
use super::*;
use crate::fx::Fx;
use std::sync::LazyLock;

const TINY: &str = "[solid]
###########
#....D...##
#....#...D#
#B...#...##
###########
[rooms]
-----------
-aaaa-bbb--
-aaaa-bbb--
-aaaa-bbb--
-----------
[room]
a sala floor=taco wall=reboco ceil=forro light=50
b despensa floor=concreto wall=reboco ceil=forro light=20
[door]
5 1 v open
9 2 v locked
[cam]
0 0 10 4 5 2.4 2 5 0.5 2 60
[spot]
spawn 1.5 1.5 0
spawn 3.5 1.5 90
spawn 2.5 2.5 180
spawn 3.5 3.5 270
task breaker 7 1
task window 2 1
task key 7 3
task tv 4 3
final 6 1 8 3
exit 9 2
";

static TINY_MAP: LazyLock<Map<'static>> = LazyLock::new(|| map::parse(TINY).unwrap_or_else(|e| panic!("{e}")));

const GOLDEN: (u32, u32) = (0x5450_6562, 0xd03b_434a);

pub fn tiny_world(players: u8, night_secs: u32) -> World {
    World::new(&TINY_MAP, Setup { seed: 7, players, night_secs })
}
fn sobrado_world(players: u8, night_secs: u32) -> World {
    World::new(map::sobrado(), Setup { seed: 0xD099_105E, players, night_secs })
}

struct Script {
    rng: Rng,
    held: [u16; MAX_PLAYERS],
    left: [u32; MAX_PLAYERS],
}

impl Script {
    fn new(seed: u32) -> Script {
        Script { rng: Rng::new(seed), held: [0; MAX_PLAYERS], left: [0; MAX_PLAYERS] }
    }

    fn next(&mut self) -> [u16; MAX_PLAYERS] {
        (0..MAX_PLAYERS).for_each(|p| {
            if self.left[p] == 0 {
                let r = self.rng.below(100);
                let base = match r {
                    0..=44 => UP,
                    45..=54 => UP | B,
                    55..=64 => LEFT,
                    65..=74 => RIGHT,
                    75..=79 => DOWN,
                    80..=87 => A,
                    88..=91 => X,
                    92..=94 => super::tank::Y,
                    _ => 0,
                };
                self.held[p] = base | if self.rng.below(4) == 0 { LEFT } else { 0 };
                self.left[p] = 5 + self.rng.below(40);
            }
            self.left[p] -= 1;
        });
        self.held
    }
}

fn run(w: &mut World, script: &mut Script, tics: u32, mut each: impl FnMut(&World)) {
    (0..tics).for_each(|_| {
        let inputs = script.next();
        w.step(&inputs);
        each(w);
    });
}

fn night_of(p: Phase) -> u8 {
    match p {
        Phase::Dusk { night } | Phase::Night { night } | Phase::Dawn { night } => night,
        Phase::Won | Phase::Lost => 0,
    }
}

fn deep_step(w: &mut World, script: &mut Script, deepest: &mut u8) {
    let inputs = script.next();
    w.step(&inputs);
    *deepest = (*deepest).max(night_of(w.phase()));
    if matches!(w.phase(), Phase::Dawn { .. }) {
        w.extra = 0;
    }
    if w.phase() == Phase::Lost && *deepest < NIGHTS {
        (0..w.players_n as usize).for_each(|p| {
            w.players[p].role = Role::Living;
            w.players[p].fade = 0;
            w.capture[p] = 0;
        });
        w.begin_dusk(*deepest + 1);
    }
}

#[test]
fn deterministic_with_golden_hash() {
    let mut a = sobrado_world(4, 30);
    let mut b = sobrado_world(4, 30);
    let (mut sa, mut sb) = (Script::new(99), Script::new(99));
    let (mut da, mut db) = (0, 0);
    let digest = (0..7000).fold(0u32, |d, _| {
        deep_step(&mut a, &mut sa, &mut da);
        deep_step(&mut b, &mut sb, &mut db);
        assert_eq!(a.hash(), b.hash());
        (d ^ a.hash()).wrapping_mul(0x0100_0193)
    });
    assert_eq!(da, NIGHTS);
    assert_eq!((digest, a.hash()), GOLDEN, "golden changed: ({digest:#x}, {:#x})", a.hash());
}

#[test]
fn bodies_never_overlap_walls() {
    let mut w = sobrado_world(4, 30);
    let mut s = Script::new(5);
    let mut deepest = 0;
    (0..7000).for_each(|_| {
        deep_step(&mut w, &mut s, &mut deepest);
        w.bodies().iter().enumerate().filter(|(_, b)| b.visible).for_each(|(i, b)| {
            let solid = |x, y| w.solid(x, y);
            assert!(!collide::overlaps(b.pos, &solid), "body {i} in wall at {:?} phase {:?}", b.pos, w.phase());
            assert!(w.map.tile(b.pos.x.floor(), b.pos.y.floor()).walkable());
        });
    });
    assert_eq!(deepest, NIGHTS);
}

#[test]
fn copy_replays_original_trajectory() {
    let mut w = tiny_world(1, 30);
    let script = |t: u32| match (t / 20) % 5 {
        0 => UP,
        1 => LEFT,
        2 => UP | B,
        3 => RIGHT | DOWN,
        _ => X | LEFT,
    };
    let mut original = Vec::new();
    let mut copy = Vec::new();
    (0..2300).for_each(|_| {
        let night = w.phase();
        let t = w.phase_tic;
        let input = match night {
            Phase::Night { night: 1 } => script(t),
            _ => 0,
        };
        w.step(&[input, 0, 0, 0]);
        match night {
            Phase::Night { night: 1 } => original.push((w.bodies[0].pos, w.bodies[0].heading, w.bodies[0].lantern)),
            Phase::Night { night: 2 } => copy.push((w.bodies[1].pos, w.bodies[1].heading, w.bodies[1].lantern)),
            Phase::Dawn { night: 1 } => w.extra = 0,
            _ => {}
        }
    });
    assert_eq!(original.len(), w.night_tics as usize);
    assert_eq!(copy.len(), original.len());
    let moved = original.iter().filter(|(p, _, _)| *p != original[0].0).count();
    assert!(moved > 100);
    assert!(original == copy);
}

fn to_night(w: &mut World, night: u8, quiet: bool) {
    while w.phase() != (Phase::Night { night }) {
        assert!(!matches!(w.phase(), Phase::Won | Phase::Lost), "ended at {:?}", w.phase());
        w.extra = 0;
        if quiet {
            clear_echoes(w);
        }
        w.step(&[0; MAX_PLAYERS]);
    }
}

fn clear_echoes(w: &mut World) {
    (w.players_n as usize..MAX_BODIES).for_each(|i| {
        w.bodies[i] = HIDDEN;
        w.ctl[i] = Ctl::FREE;
        w.echo[i] = Echo::NONE;
    });
}

fn put(w: &mut World, i: usize, x: Fx, y: Fx, heading: Ang, lantern: bool) {
    w.bodies[i].pos = Vec2::new(x, y);
    w.bodies[i].heading = heading;
    w.bodies[i].lantern = lantern;
}

fn captured(e: &Events) -> Option<u8> {
    e.iter().find_map(|e| match e {
        Event::Captured { player } => Some(*player),
        _ => None,
    })
}

#[test]
fn capture_turns_living_into_duplo() {
    let mut w = tiny_world(2, 30);
    to_night(&mut w, 2, true);
    clear_echoes(&mut w);
    (0..GRACE_TICS).for_each(|_| {
        w.step(&[0; MAX_PLAYERS]);
    });
    assert_eq!(captured(w.events()), None);
    let (x, y) = (Fx::int(2) + Fx::HALF, Fx::int(2) + Fx::HALF);
    put(&mut w, 0, x, y, Ang::ZERO, true);
    put(&mut w, 1, Fx::int(7) + Fx::HALF, y, Ang::ZERO, false);
    w.place(2, (Vec2::new(x, y + Fx::ratio(4, 10)), Ang::HALF), Agent::Echo);
    w.echo[2] = Echo::new(tape_id(1, 0), 2, true);
    w.segs.iter_mut().for_each(|s| *s = tape::Seg::ZERO);
    assert_eq!(captured(w.step(&[0; MAX_PLAYERS])), None);
    w.bodies[0].heading = Ang::HALF;
    assert_eq!(captured(w.step(&[0; MAX_PLAYERS])), Some(0));
    assert_eq!(w.player(0).role, Role::Living);
    (2..SWAP_AT).for_each(|_| assert_eq!(captured(w.step(&[UP; MAX_PLAYERS])), None));
    let duplo = w.player(0).body as usize;
    assert_eq!((w.player(0).role, w.player(0).fade), (Role::Duplo, 255));
    assert!(duplo != 0 && w.bodies[duplo].visible && w.ctl[0].agent == Agent::Echo);
    (SWAP_AT..FADE_TICS).for_each(|_| {
        w.step(&[0; MAX_PLAYERS]);
    });
    assert_eq!(w.player(0).fade, 0);
    assert_eq!(w.phase(), Phase::Night { night: 2 });
    clear_echoes(&mut w);
    w.place(duplo, (Vec2::new(Fx::int(7) + Fx::HALF, y - Fx::ratio(4, 10)), Ang::ZERO), Agent::Player(0));
    let b1 = w.player(1).body as usize;
    w.bodies[b1].pos = Vec2::new(Fx::int(7) + Fx::HALF, y);
    w.bodies[b1].heading = Ang::ZERO;
    assert_eq!(captured(w.step(&[0; MAX_PLAYERS])), Some(1));
    (0..FADE_TICS).for_each(|_| {
        w.step(&[0; MAX_PLAYERS]);
    });
    assert_eq!(w.phase(), Phase::Lost);
}

#[test]
fn finale_opens_only_for_a_still_dark_vigil() {
    let mut w = tiny_world(1, 30);
    to_night(&mut w, 5, true);
    clear_echoes(&mut w);
    assert!(!w.power_on());
    let exit = w.exit_door as usize;
    let inside = (Fx::int(7) + Fx::HALF, Fx::int(2) + Fx::HALF);
    put(&mut w, 0, inside.0, inside.1, Ang::QUARTER, false);
    let gate = at_hour(w.night_tics, OPENS_AFTER_HOURS);
    (0..gate + 20).for_each(|_| {
        w.step(&[0; MAX_PLAYERS]);
    });
    assert_eq!(w.phase(), Phase::Night { night: 5 });
    assert!(w.doors[exit].locked);
    w.step(&[LEFT, 0, 0, 0]);
    w.step(&[RIGHT, 0, 0, 0]);
    (0..HOLD_TICS - 1).for_each(|_| {
        w.step(&[0; MAX_PLAYERS]);
    });
    assert!(w.doors[exit].locked && !w.exit_open);
    let opened = w.step(&[0; MAX_PLAYERS]).iter().any(|e| *e == Event::DoorMove { door: exit as u8, opening: true });
    assert!(opened && !w.doors[exit].locked && w.exit_open);
    let mut tics = 0;
    while w.phase() != Phase::Won && tics < 200 {
        w.step(&[UP, 0, 0, 0]);
        tics += 1;
    }
    assert_eq!(w.phase(), Phase::Won);
}

#[test]
fn hunter_closes_in_on_the_living() {
    let mut w = sobrado_world(1, 60);
    let mut s = Script::new(21);
    to_night(&mut w, 1, false);
    while w.phase() == (Phase::Night { night: 1 }) {
        let inputs = s.next();
        w.step(&inputs);
    }
    to_night(&mut w, 4, true);
    clear_echoes(&mut w);
    put(&mut w, 0, Fx::int(20) + Fx::HALF, Fx::int(6) + Fx::HALF, Ang::ZERO, false);
    w.place(1, (w.map.spawns[0].pos, w.map.spawns[0].rot), Agent::Echo);
    w.echo[1] = Echo::new(tape_id(1, 0), 4, false);
    w.chase_field();
    let start = w.distance_at(w.bodies[1].pos);
    let mut best = start;
    let mut caught = false;
    while w.phase() == (Phase::Night { night: 4 }) && !caught {
        caught = captured(w.step(&[0; MAX_PLAYERS])).is_some();
        best = best.min(w.distance_at(w.bodies[1].pos));
    }
    assert!(start >= 10 && start != u16::MAX, "start {start}");
    assert!(caught || best <= 2, "hunter got from {start} only to {best}");
}

#[test]
fn night_three_glitches_every_kind() {
    let mut w = sobrado_world(2, 60);
    to_night(&mut w, 3, false);
    let mut seen = [false; 3];
    let mut glitches = 0;
    while w.phase() == (Phase::Night { night: 3 }) {
        glitches += w.step(&[0; MAX_PLAYERS]).iter().filter(|e| matches!(e, Event::Glitch { .. })).count();
        w.echo.iter().for_each(|e| match e.mode {
            echo::Mode::Splice(_) => seen[0] = true,
            echo::Mode::Reverse(_) => seen[1] = true,
            echo::Mode::Stare => seen[2] = true,
            _ => {}
        });
    }
    assert!(glitches >= 5 && seen == [true; 3], "{glitches} {seen:?}");
}

#[test]
fn night_five_without_escape_is_lost() {
    let mut w = tiny_world(1, 30);
    to_night(&mut w, 5, true);
    clear_echoes(&mut w);
    (0..w.night_tics).for_each(|_| {
        w.step(&[0; MAX_PLAYERS]);
    });
    assert_eq!(w.phase(), Phase::Lost);
}

#[test]
fn tasks_power_and_copies_per_night() {
    let mut w = sobrado_world(3, 30);
    let active = w.tasks().iter().filter(|t| t.active).count();
    assert!((3..=4).contains(&active));
    assert_eq!(w.tasks().iter().filter(|t| t.active && t.kind == TaskKind::Breaker).count(), 1);
    to_night(&mut w, 1, false);
    let mut out = false;
    while w.phase() == (Phase::Night { night: 1 }) {
        out |= w.step(&[0; MAX_PLAYERS]).iter().any(|e| *e == Event::PowerOut);
    }
    assert!(out && w.extra as usize == active);
    to_night(&mut w, 3, false);
    assert_eq!(w.bodies().iter().filter(|b| b.visible).count(), 3 + 2 * 3);
    assert!(w.power_on());
}

#[test]
fn steady_state_never_grows_buffers() {
    let mut w = sobrado_world(4, 30);
    let caps = |w: &World| {
        [
            w.doors.capacity(),
            w.door_dir.capacity(),
            w.door_at.capacity(),
            w.tasks.capacity(),
            w.room_surface.capacity(),
            w.final_tiles.capacity(),
            w.tape.capacity(),
            w.segs.capacity(),
            w.field.capacity(),
            w.queue.capacity(),
        ]
    };
    let before = caps(&w);
    let mut s = Script::new(11);
    let mut deepest = 0;
    let allocs = crate::audio::alloc::count(|| (0..10_000).for_each(|_| deep_step(&mut w, &mut s, &mut deepest)));
    assert_eq!(allocs, 0);
    assert_eq!(before, caps(&w));
}

#[test]
#[ignore]
fn bench_four_players_twenty_copies() {
    let mut w = sobrado_world(4, 360);
    to_night(&mut w, 4, true);
    w.extra = 4;
    w.begin_dusk(5);
    to_night(&mut w, 5, false);
    let bodies = w.bodies().iter().filter(|b| b.visible).count();
    let mut s = Script::new(3);
    let start = std::time::Instant::now();
    let tics = 3000;
    run(&mut w, &mut s, tics, |_| {});
    let per = start.elapsed().as_nanos() / tics as u128;
    let heap = w.doors.len() * size_of::<Door>()
        + w.door_dir.len()
        + w.door_at.len()
        + w.tasks.len() * size_of::<Task>()
        + w.room_surface.len()
        + w.final_tiles.len() * 2
        + w.tape.len() * 2
        + w.segs.len() * size_of::<tape::Seg>()
        + w.field.len() * 2
        + w.queue.len() * 2;
    println!("bodies {bodies} step {per} ns world {} B heap {heap} B phase {:?}", size_of::<World>(), w.phase());
}
