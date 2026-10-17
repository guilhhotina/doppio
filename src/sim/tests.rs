use super::player::{BENEDITA, BIA, GRAB_TICS, JAIRO, TONHO};
use super::tank::{A, B, DOWN, LEFT, RIGHT, UP, X, Y};
use super::*;
use crate::fx::{Fx, Rng};
use crate::map;
use std::sync::LazyLock;

macro_rules! test_map {
    ($header:literal, $solid:literal, $tail:literal) => {
        concat!(
            "[map] name=\"t\" ",
            $header,
            " sky=none fog=0,0,0 fog_near=5 fog_far=30 legend=corposeco\n[solid]\n",
            $solid,
            "[area]\n0 baixo\n1 cima\n[room]\na sala area=0 floor=taco wall=reboco ceil=forro light=40\nf cima area=1 floor=taco wall=reboco ceil=forro light=40\n[cam]\n0 0 199 199 1 2 1 2 0 2 60\n",
            $tail
        )
    };
}

const KINDS: [LegendKind; 4] = [LegendKind::CorpoSeco, LegendKind::Cabra, LegendKind::Mula, LegendKind::Matinta];

fn parsed(text: &'static str) -> [Map<'static>; 4] {
    KINDS.map(|legend| Map { legend, ..map::parse(text).unwrap_or_else(|e| panic!("{e}")) })
}

macro_rules! maps {
    ($($name:ident = $text:expr;)*) => {
        $(static $name: LazyLock<[Map<'static>; 4]> = LazyLock::new(|| parsed($text));)*
    };
}

maps! {
    CORRIDOR = test_map!(
        "w=42 h=4",
        "##########################################\n#........................................#\n#........................................#\n##########################################\n[rooms]\n------------------------------------------\n-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-\n-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-\n------------------------------------------\n",
        "[wake]\n0 2.5 1.5 90\n1 2.5 2.5 90\n2 3.5 1.5 90\n3 3.5 2.5 90\n[legend]\nspawn 3.5 1.5 90\npatrol 5 1\npatrol 10 1\npatrol 15 1\npatrol 20 1\npatrol 25 1\npatrol 30 2\nlair 39 2\n[lock]\nsaida code 999\n[exit]\n40 1 40 2 lock=saida\n"
    );
    LAB = test_map!(
        "w=25 h=9 power=off",
        "#########################\n#....#.....#.......#....#\n#....D.....D.......#....#\n#H...#..B..#.......#..P.#\n##D###.....#.......#....#\n#....#######..B....######\n#..P.#######.......######\n#....#######.......######\n#########################\n[rooms]\n-------------------------\n-aaaa-aaaaa-aaaaaaa-ffff-\n-aaaaaaaaaaaaaaaaaa-ffff-\n-aaaa-aaaaa-aaaaaaa-ffff-\n--a---aaaaa-aaaaaaa-ffff-\n-aaaa-------aaaaaaa------\n-aaaa-------aaaaaaa------\n-aaaa-------aaaaaaa------\n-------------------------\n",
        "[item]\nchave1 chave 20.5 1.5\nalicate alicate 1.5 6.5\nfus fusivel 14.5 5.5\nsal sal 1.5 1.5\npilha lanterna_pilha 4.5 7.5\nfoto foto 4.5 5.5\ncruz cruz 17.5 3.5\n[lock]\nporta key chave1\ncadeado code 317\ncorrente tool alicate\nluz place fus 8 3\nsaida all porta luz corrente cadeado\n[door]\n5 2 v closed\n2 4 h stuck\n11 2 v locked:porta\n[portal]\n3 6 -> 21 2 0 escada lock=corrente\n22 3 -> 2 6 90 escada\n[prop]\nquadro_fusivel 8.5 3.5 0\ncofre 14.5 5.5 270 lock=cadeado item=fus\n[note]\n10 4 calendario \"3-1-7\"\n[hide]\n1 3 cama 0\n[wake]\n0 1.5 1.5 90\n1 2.5 1.5 90\n2 3.5 1.5 90\n3 4.5 1.5 90\n[legend]\nspawn 16.5 3.5 270\npatrol 13 1\npatrol 18 1\npatrol 18 4\npatrol 13 4\npatrol 16 7\npatrol 13 7\nlair 18 2\n[exit]\n17 6 18 7 lock=saida\n"
    );
    HEAR = test_map!(
        "w=13 h=3",
        "#############\n#.....D.....#\n#############\n[rooms]\n-------------\n-aaaaaaaaaaa-\n-------------\n",
        "[door]\n6 1 v open\n[wake]\n0 10.5 1.5 90\n1 10.5 1.5 90\n2 10.5 1.5 90\n3 10.5 1.5 90\n[legend]\nspawn 2.5 1.5 270\npatrol 1 1\npatrol 2 1\npatrol 3 1\npatrol 4 1\npatrol 5 1\npatrol 3 1\nlair 1 1\n[lock]\nsaida code 999\n[exit]\n11 1 11 1 lock=saida\n"
    );
    HIDE = test_map!(
        "w=15 h=5",
        "###############\n#.............#\n#.............#\n#H............#\n###############\n[rooms]\n---------------\n-aaaaaaaaaaaaa-\n-aaaaaaaaaaaaa-\n-aaaaaaaaaaaaa-\n---------------\n",
        "[hide]\n1 3 guarda_roupa 0\n[wake]\n0 1.5 2.5 0\n1 2.5 2.5 0\n2 3.5 2.5 0\n3 4.5 2.5 0\n[legend]\nspawn 13.5 1.5 90\npatrol 13 1\npatrol 2 1\npatrol 1 2\npatrol 2 2\npatrol 7 2\npatrol 13 2\nlair 13 3\n[lock]\nsaida code 999\n[exit]\n13 3 13 3 lock=saida\n"
    );
    BALDE = test_map!(
        "w=9 h=3",
        "#########\n#.......#\n#########\n[rooms]\n---------\n-aaaaaaa-\n---------\n",
        "[item]\nbalde balde 1.5 1.5\n[wake]\n0 1.5 1.5 90\n1 1.5 1.5 90\n2 1.5 1.5 90\n3 1.5 1.5 90\n[legend]\nspawn 3.5 1.5 270\npatrol 1 1\npatrol 2 1\npatrol 3 1\npatrol 4 1\npatrol 5 1\npatrol 6 1\nlair 7 1\n[lock]\nsaida code 999\n[exit]\n7 1 7 1 lock=saida\n"
    );
    ARENA = test_map!(
        "w=9 h=9",
        "#########\n#.......#\n#.......#\n#.......#\n#.......#\n#.......#\n#.......#\n#.......#\n#########\n[rooms]\n---------\n-aaaaaaa-\n-aaaaaaa-\n-aaaaaaa-\n-aaaaaaa-\n-aaaaaaa-\n-aaaaaaa-\n-aaaaaaa-\n---------\n",
        "[wake]\n0 4.5 1.5 90\n1 4.5 1.5 90\n2 4.5 1.5 90\n3 4.5 1.5 90\n[legend]\nspawn 4.5 6.5 180\npatrol 1 1\npatrol 7 1\npatrol 7 7\npatrol 1 7\npatrol 4 4\npatrol 2 2\nlair 1 7\n[lock]\nsaida code 999\n[exit]\n7 7 7 7 lock=saida\n"
    );
}

fn pick(maps: &'static LazyLock<[Map<'static>; 4]>, kind: LegendKind) -> &'static Map<'static> {
    &maps[kind as usize]
}

fn setup(players: u8, characters: [u8; 4]) -> Setup {
    Setup { seed: 7, phase: 0, players, characters }
}

fn deg(d: i32) -> Ang {
    Ang::deg(Fx::int(d))
}

fn put(w: &mut World, p: usize, x: Fx, y: Fx, d: i32) {
    w.place_player(p, Vec2::new(x, y), deg(d));
}

fn mid(t: i32) -> Fx {
    Fx::int(t) + Fx::HALF
}

fn calm(w: &mut World) {
    w.send_legend_home(u16::MAX);
}

fn tic(w: &mut World, inputs: [u16; 4]) -> Vec<Event> {
    w.step(&inputs);
    w.events().iter().copied().collect()
}

fn press(w: &mut World, p: usize, bits: u16) -> Vec<Event> {
    let mut inputs = [0; 4];
    inputs[p] = bits;
    let e = tic(w, inputs);
    tic(w, [0; 4]);
    e
}

fn hand(w: &mut World, p: usize, item: &str) -> u8 {
    let i = w.map.item_index(item).unwrap();
    let slot = w.players[p].hands.iter().position(Option::is_none).unwrap();
    w.players[p].hands[slot] = Some(i);
    w.items[i as usize].place = ItemPlace::Held { player: p as u8 };
    i
}

fn lock(w: &World, id: &str) -> u8 {
    w.map.lock_index(id).unwrap()
}

#[test]
fn chase_never_catches_a_running_player() {
    KINDS.into_iter().for_each(|kind| {
        let mut w = World::new(pick(&CORRIDOR, kind), setup(1, [BIA, 1, 2, 3]));
        put(&mut w, 0, mid(8), mid(1), 90);
        w.legend.pos = Vec2::new(mid(3), mid(1));
        w.legend.heading = deg(90);
        let start = (w.players[0].pos - w.legend.pos).len();
        let mut chased = false;
        (0..300).for_each(|_| {
            w.step(&[UP | B, 0, 0, 0]);
            chased |= w.legend.mode == LegendMode::Chase;
            assert_eq!(w.players[0].state, PlayerState::Free, "{kind:?} caught the runner at tic {}", w.tic);
        });
        let end = (w.players[0].pos - w.legend.pos).len();
        let kept = match kind {
            LegendKind::Mula => end > Fx::int(3),
            _ => end > start,
        };
        assert!(chased && kept, "{kind:?}: chased={chased} start={start:?} end={end:?}");
    });
}

#[test]
fn walking_player_is_caught_by_a_chase() {
    let mut w = World::new(pick(&CORRIDOR, LegendKind::CorpoSeco), setup(1, [BIA, 1, 2, 3]));
    put(&mut w, 0, mid(8), mid(1), 90);
    w.legend.pos = Vec2::new(mid(3), mid(1));
    w.legend.heading = deg(90);
    let caught = (0..900).any(|_| {
        w.step(&[UP, 0, 0, 0]);
        w.players[0].state == PlayerState::Grabbed
    });
    assert!(caught);
    (0..GRAB_TICS).for_each(|_| w.step(&[0; 4]));
    assert!(matches!(w.players[0].state, PlayerState::Fainted { .. }));
    assert_eq!((w.legend.mode, w.desmaios_left()), (LegendMode::Leaving, DESMAIOS - 1));
    let leaving = (0..legend::LEAVE_TICS * 2).take_while(|_| {
        w.step(&[0; 4]);
        w.legend.mode == LegendMode::Leaving
    });
    assert_eq!(leaving.count(), legend::LEAVE_TICS as usize - 1);
}

#[test]
fn legend_freezes_to_roar_before_chasing() {
    let mut w = World::new(pick(&CORRIDOR, LegendKind::CorpoSeco), setup(1, [BIA, 1, 2, 3]));
    put(&mut w, 0, mid(8), mid(1), 270);
    w.legend.pos = Vec2::new(mid(3), mid(1));
    w.legend.heading = deg(90);
    let at = w.legend.pos;
    assert!(tic(&mut w, [0; 4]).contains(&Event::LegendSaw { player: 0 }));
    (1..legend::ALERT_TICS).for_each(|_| {
        w.step(&[0; 4]);
        assert_eq!((w.legend.pos, w.legend.anim, w.legend.mode), (at, LegendAnim::Alert, LegendMode::Chase));
    });
    w.step(&[0; 4]);
    assert!(w.legend.pos.x > at.x && w.legend.anim == LegendAnim::Run);
}

#[test]
fn grab_winds_up_and_a_sprint_slips_away() {
    let fresh = || {
        let mut w = World::new(pick(&CORRIDOR, LegendKind::CorpoSeco), setup(1, [BIA, 1, 2, 3]));
        put(&mut w, 0, mid(8), mid(1), 90);
        w.legend.pos = Vec2::new(mid(8) - Fx::ratio(7, 10), mid(1));
        w.legend.heading = deg(90);
        w.legend.mode = LegendMode::Chase;
        w.legend.target = Some(0);
        w
    };
    let mut w = fresh();
    assert!(tic(&mut w, [0; 4]).contains(&Event::Lunge { player: 0 }));
    (1..legend::LUNGE_TICS).for_each(|_| {
        w.step(&[0; 4]);
        assert_eq!((w.players[0].state, w.legend.anim), (PlayerState::Free, LegendAnim::Grab));
    });
    w.step(&[0; 4]);
    assert_eq!(w.players[0].state, PlayerState::Grabbed);
    let mut w = fresh();
    let lunged = (0..60).fold(false, |seen, _| {
        let e = tic(&mut w, [UP | B, 0, 0, 0]);
        assert_eq!(w.players[0].state, PlayerState::Free);
        seen || e.contains(&Event::Lunge { player: 0 })
    });
    assert!(lunged);
}

#[test]
fn heat_rises_after_the_calm_and_with_each_lock() {
    let mut w = World::new(map::campaign()[0], setup(1, [0, 1, 2, 3]));
    let n = w.locks.len();
    assert_eq!((w.heat(), w.unlock_noise()), (0, 0));
    w.tic = legend::CALM_TICS;
    assert_eq!(w.heat(), 1);
    w.tic = 0;
    w.locks[0].open = true;
    assert_eq!(w.heat(), 1);
    w.locks.iter_mut().take(n.div_ceil(3)).for_each(|l| l.open = true);
    assert_eq!(w.heat(), 2);
    w.locks.iter_mut().for_each(|l| l.open = true);
    assert_eq!(w.heat(), 3);
    assert!(w.unlock_noise() > act::DOOR_NOISE);
    let m = pick(&HEAR, LegendKind::CorpoSeco);
    let heard = |at: u32| {
        let mut w = World::new(m, setup(1, [0, 1, 2, 3]));
        w.tic = at;
        w.door_dir[0] = true;
        w.doors[0].open = u16::MAX;
        w.events.clear();
        w.noise(Vec2::new(mid(8), mid(1)), 0, player::RUN_NOISE);
        w.listen();
        w.legend.mode == LegendMode::Investigate
    };
    assert!(heard(legend::CALM_TICS) && !heard(0));
}

#[test]
fn hearing_is_muffled_by_closed_doors() {
    let m = pick(&HEAR, LegendKind::CorpoSeco);
    let heard = |open: bool, loud: u8, at: i32| {
        let mut w = World::new(m, setup(1, [0, 1, 2, 3]));
        w.tic = legend::CALM_TICS;
        w.door_dir[0] = open;
        w.doors[0].open = if open { u16::MAX } else { 0 };
        w.events.clear();
        w.noise(Vec2::new(mid(at), mid(1)), 0, loud);
        w.listen();
        w.legend.mode == LegendMode::Investigate
    };
    assert!(heard(true, player::RUN_NOISE, 9));
    assert!(!heard(false, player::RUN_NOISE, 9));
    assert!(heard(false, player::RUN_NOISE, 5));
    assert!(heard(false, act::RATTLE_NOISE, 6));
    assert!(!heard(false, act::RATTLE_NOISE, 8));
    assert!(!heard(true, player::WALK_NOISE, 9));
    assert!(heard(true, player::WALK_NOISE, 4));
    let mut w = World::new(m, setup(1, [0, 1, 2, 3]));
    w.tic = legend::CALM_TICS;
    put(&mut w, 0, mid(9), mid(1), 90);
    let investigated = (0..60).any(|_| {
        w.step(&[UP | B, 0, 0, 0]);
        w.legend.mode == LegendMode::Investigate
    });
    assert!(investigated);
}

#[test]
fn tonho_steps_are_silent_and_his_run_is_half() {
    let mut w = World::new(pick(&CORRIDOR, LegendKind::CorpoSeco), setup(2, [TONHO, JAIRO, 0, 0]));
    calm(&mut w);
    let noise = |w: &mut World, bits: [u16; 4]| {
        (0..60)
            .flat_map(|_| tic(w, bits))
            .filter_map(|e| match e {
                Event::Noise { loud, .. } => Some(loud),
                _ => None,
            })
            .max()
    };
    assert_eq!(noise(&mut w, [UP, 0, 0, 0]), None);
    assert_eq!(noise(&mut w, [UP | B, 0, 0, 0]), Some(player::RUN_NOISE / 2));
    assert_eq!(noise(&mut w, [0, UP, 0, 0]), Some(player::WALK_NOISE));
}

fn hide_room() -> &'static Map<'static> {
    pick(&HIDE, LegendKind::CorpoSeco)
}

#[test]
fn unseen_hidden_player_is_never_found() {
    let mut w = World::new(hide_room(), setup(1, [BENEDITA, 1, 2, 3]));
    put(&mut w, 0, mid(1), mid(2), 0);
    assert_eq!(w.focus(0), Some(Focus::Hide { spot: 0 }));
    press(&mut w, 0, A);
    (0..player::HIDE_TICS).for_each(|_| w.step(&[0; 4]));
    assert_eq!(w.players[0].state, PlayerState::Hidden { spot: 0 });
    let mut passed = false;
    (0..90 * TICS_PER_SEC).for_each(|_| {
        w.step(&[0; 4]);
        passed |= (w.legend.pos - w.players[0].pos).len_sq() < Fx::ONE.0 as i64 * Fx::ONE.0 as i64;
        assert_eq!(w.players[0].state, PlayerState::Hidden { spot: 0 });
    });
    assert!(passed);
    press(&mut w, 0, A);
    (0..player::HIDE_TICS).for_each(|_| w.step(&[0; 4]));
    assert_eq!(w.players[0].state, PlayerState::Free);
}

#[test]
fn hiding_in_sight_gets_you_pulled_out() {
    let mut w = World::new(hide_room(), setup(1, [BENEDITA, 1, 2, 3]));
    put(&mut w, 0, mid(1), mid(2), 0);
    w.legend.pos = Vec2::new(mid(8), mid(2));
    w.legend.heading = deg(270);
    press(&mut w, 0, A);
    assert_eq!(w.legend.mode, LegendMode::Chase);
    let pulled = (0..10 * TICS_PER_SEC).any(|_| {
        w.step(&[0; 4]);
        w.players[0].state == PlayerState::Grabbed
    });
    assert!(pulled);
}

#[test]
fn tonho_hides_instantly() {
    let mut w = World::new(hide_room(), setup(1, [TONHO, 1, 2, 3]));
    put(&mut w, 0, mid(1), mid(2), 0);
    let e = press(&mut w, 0, A);
    assert!(e.contains(&Event::Hide { player: 0 }));
    assert_eq!(w.players[0].state, PlayerState::Hidden { spot: 0 });
    let e = press(&mut w, 0, A);
    assert!(e.contains(&Event::Unhide { player: 0 }));
}

#[test]
fn key_tool_code_place_and_all_locks() {
    let mut w = World::new(pick(&LAB, LegendKind::CorpoSeco), setup(1, [JAIRO, 1, 2, 3]));
    calm(&mut w);
    assert!(!w.power());
    let (porta, cadeado, corrente, luz, saida) = (lock(&w, "porta"), lock(&w, "cadeado"), lock(&w, "corrente"), lock(&w, "luz"), lock(&w, "saida"));
    put(&mut w, 0, mid(10), mid(2), 90);
    assert_eq!(w.focus(0), Some(Focus::Lock { lock: porta, needs: Some(ItemKind::Chave), code: false }));
    let e = press(&mut w, 0, A);
    assert!(e.contains(&Event::DoorRattle { door: 2 }) && e.iter().any(|e| matches!(e, Event::Noise { loud: 80, .. })));
    let key = hand(&mut w, 0, "chave1");
    let e = press(&mut w, 0, A);
    assert!(e.contains(&Event::Unlock { lock: porta }) && e.contains(&Event::DoorMove { door: 2, opening: true }));
    assert_eq!((w.items[key as usize].place, w.doors[2].locked, w.players[0].hands), (ItemPlace::Used, false, [None, None]));

    put(&mut w, 0, mid(3), mid(6), 0);
    assert_eq!(w.focus(0), Some(Focus::Lock { lock: corrente, needs: Some(ItemKind::Alicate), code: false }));
    let tool = hand(&mut w, 0, "alicate");
    press(&mut w, 0, A);
    assert!(w.locks[corrente as usize].open);
    assert_eq!(w.players[0].hands[0], Some(tool));
    assert_eq!(w.focus(0), Some(Focus::Portal { portal: 0, locked: false }));
    let e = press(&mut w, 0, A);
    assert!(e.contains(&Event::Portal { player: 0 }) && e.contains(&Event::PortalUse { player: 0, portal: 0 }));
    assert_eq!((w.players[0].pos, w.players[0].area), (Vec2::new(mid(21), mid(2)), 1));

    put(&mut w, 0, mid(13), mid(5), 90);
    assert_eq!(w.focus(0), Some(Focus::Lock { lock: cadeado, needs: None, code: true }));
    press(&mut w, 0, A);
    [UP, UP, UP, RIGHT, UP, RIGHT, DOWN, DOWN, DOWN].into_iter().for_each(|b| {
        press(&mut w, 0, b);
    });
    assert_eq!(w.dial(0).map(|d| (d.digits, d.len, d.cursor)), Some(([3, 1, 7, 0], 3, 2)));
    let e = press(&mut w, 0, A);
    assert!(e.contains(&Event::Unlock { lock: cadeado }) && w.dial(0).is_none());
    assert_eq!(w.focus(0), Some(Focus::Container { prop: 1, locked: false }));
    let e = press(&mut w, 0, A);
    assert!(e.contains(&Event::Container { player: 0, prop: 1 }));
    let fus = w.map.item_index("fus").unwrap();
    assert_eq!(w.focus(0), Some(Focus::Item { item: fus }));
    press(&mut w, 0, A);
    assert_eq!(w.players[0].hands, [Some(tool), Some(fus)]);
    (0..20).for_each(|_| w.step(&[0; 4]));
    assert!(w.props[1].open > u16::MAX / 2);

    put(&mut w, 0, mid(8), mid(2), 0);
    assert_eq!(w.focus(0), Some(Focus::Place { lock: luz, needs: ItemKind::Fusivel }));
    let e = press(&mut w, 0, A);
    assert!(e.contains(&Event::PowerOn) && e.contains(&Event::Unlock { lock: luz }) && e.contains(&Event::Unlock { lock: saida }));
    assert!(w.power() && w.locks.iter().all(|l| l.open));
    assert_eq!(w.outcome(), Outcome::Playing);
    put(&mut w, 0, mid(17), mid(5), 0);
    let won = (0..60).flat_map(|_| tic(&mut w, [UP, 0, 0, 0])).collect::<Vec<_>>();
    assert!(won.contains(&Event::Escaped { player: 0 }) && won.contains(&Event::PhaseWon));
    assert_eq!(w.outcome(), Outcome::Escaped);
}

#[test]
fn dial_input_wraps_and_wrong_code_makes_noise() {
    let mut w = World::new(pick(&LAB, LegendKind::CorpoSeco), setup(1, [0, 1, 2, 3]));
    calm(&mut w);
    put(&mut w, 0, mid(13), mid(5), 90);
    press(&mut w, 0, A);
    assert_eq!(w.dial(0), Some(Dial { lock: lock(&w, "cadeado"), digits: [0; 4], len: 3, cursor: 0, wrong: 0 }));
    let e = press(&mut w, 0, LEFT);
    assert!(e.contains(&Event::DialTick { player: 0 }));
    press(&mut w, 0, DOWN);
    assert_eq!(w.dial(0).map(|d| (d.digits, d.cursor)), Some(([0, 0, 9, 0], 2)));
    press(&mut w, 0, RIGHT);
    press(&mut w, 0, UP | RIGHT);
    assert_eq!(w.dial(0).map(|d| (d.digits, d.cursor)), Some(([1, 0, 9, 0], 1)));
    let e = press(&mut w, 0, A);
    assert!(e.contains(&Event::DialWrong { player: 0 }) && e.iter().any(|e| matches!(e, Event::Noise { loud: act::WRONG_NOISE, .. })));
    assert_eq!(w.dial(0).map(|d| d.wrong), Some(1));
    assert!(!w.locks[lock(&w, "cadeado") as usize].open);
    press(&mut w, 0, UP);
    assert_eq!(w.players[0].pos, Vec2::new(mid(13), mid(5)));
    press(&mut w, 0, Y);
    assert_eq!(w.dial(0), None);
}

#[test]
fn stuck_door_only_for_jairo_and_hands() {
    let mut w = World::new(pick(&LAB, LegendKind::CorpoSeco), setup(2, [BIA, JAIRO, 2, 3]));
    calm(&mut w);
    put(&mut w, 0, mid(2), mid(3), 0);
    put(&mut w, 1, mid(3), mid(1), 270);
    assert_eq!(w.focus(0), Some(Focus::Door { door: 1, locked: false, stuck: true, open: false }));
    let e = press(&mut w, 0, A);
    assert!(e.contains(&Event::DoorRattle { door: 1 }));
    put(&mut w, 1, mid(2), mid(3), 0);
    put(&mut w, 0, mid(1), mid(1), 0);
    let e = press(&mut w, 1, A);
    assert!(e.contains(&Event::DoorMove { door: 1, opening: true }) && !w.doors[1].stuck);
    let pick = |w: &mut World, p: usize, item: &str, x: i32, y: i32| {
        let i = w.map.item_index(item).unwrap();
        w.items[i as usize].place = ItemPlace::Floor { pos: Vec2::new(mid(x), mid(y)), area: 0 };
        put(w, p, mid(x), mid(y), 0);
        assert_eq!(w.focus(p), Some(Focus::Item { item: i }));
        press(w, p, A);
        i
    };
    let s = pick(&mut w, 1, "sal", 7, 1);
    let c = pick(&mut w, 1, "cruz", 7, 3);
    assert_eq!(w.players[1].hands, [Some(s), Some(c)]);
    let a = pick(&mut w, 1, "alicate", 9, 1);
    assert_eq!(w.players[1].hands, [Some(c), Some(a)]);
    assert_eq!(w.items[s as usize].place, ItemPlace::Floor { pos: Vec2::new(mid(9), mid(1)), area: 0 });
    let s2 = pick(&mut w, 0, "sal", 9, 1);
    let c2 = pick(&mut w, 0, "foto", 9, 2);
    assert_eq!((w.players[0].hands, w.items[c2 as usize].place), ([Some(s2), None], ItemPlace::Used));
    w.players[0].battery = 10;
    pick(&mut w, 0, "pilha", 8, 2);
    assert_eq!(w.players[0].battery, BATTERY_FULL);
    let e = press(&mut w, 0, DROP);
    assert!(e.contains(&Event::Drop { player: 0, item: s2 }) && e.iter().any(|e| matches!(e, Event::Noise { loud: player::DROP_NOISE, .. })));
    assert_eq!(w.players[0].hands, [None, None]);
}

#[test]
fn friend_wakes_a_fainted_player_and_desmaios_run_out() {
    let mut w = World::new(pick(&LAB, LegendKind::CorpoSeco), setup(2, [0, BIA, 2, 3]));
    calm(&mut w);
    put(&mut w, 0, mid(7), mid(2), 90);
    put(&mut w, 1, mid(8), mid(2), 270);
    w.grab_player(1);
    (0..GRAB_TICS).for_each(|_| w.step(&[0; 4]));
    assert!(matches!(w.players[1].state, PlayerState::Fainted { .. }));
    assert_eq!(w.players[1].fade, 255);
    assert_eq!(w.focus(0), Some(Focus::Friend { player: 1 }));
    let e = press(&mut w, 0, A);
    assert!(e.contains(&Event::Woke { player: 1 }));
    assert_eq!((w.players[1].state, w.players[1].pos), (PlayerState::Free, Vec2::new(mid(8), mid(2))));
    let wake = |w: &mut World, p: usize| (0..400).take_while(|_| tic(w, [0; 4]).iter().all(|e| *e != Event::Woke { player: p as u8 })).count();
    w.grab_player(0);
    (0..GRAB_TICS).for_each(|_| w.step(&[0; 4]));
    assert_eq!(wake(&mut w, 0), 3 * TICS_PER_SEC as usize - 1);
    assert_eq!(w.players[0].pos, w.map.wakes[0].pos);
    w.grab_player(1);
    (0..GRAB_TICS).for_each(|_| w.step(&[0; 4]));
    assert_eq!(wake(&mut w, 1), 6 * TICS_PER_SEC as usize - 1);
    assert_eq!(w.desmaios_left(), DESMAIOS - 3);
    (0..2).for_each(|_| {
        w.grab_player(0);
        (0..GRAB_TICS).for_each(|_| w.step(&[0; 4]));
        wake(&mut w, 0);
    });
    assert_eq!((w.desmaios_left(), w.outcome()), (0, Outcome::Playing));
    w.grab_player(1);
    let e = (0..GRAB_TICS).flat_map(|_| tic(&mut w, [0; 4])).collect::<Vec<_>>();
    assert!(e.contains(&Event::PhaseRestart));
    assert_eq!(w.outcome(), Outcome::Restart);
}

#[test]
fn faint_drops_held_items_at_the_grab_point() {
    let mut w = World::new(pick(&LAB, LegendKind::CorpoSeco), setup(1, [JAIRO, 1, 2, 3]));
    calm(&mut w);
    put(&mut w, 0, mid(7), mid(3), 90);
    let (a, b) = (hand(&mut w, 0, "sal"), hand(&mut w, 0, "cruz"));
    w.grab_player(0);
    (0..GRAB_TICS).for_each(|_| w.step(&[0; 4]));
    let floor = ItemPlace::Floor { pos: Vec2::new(mid(7), mid(3)), area: 0 };
    assert_eq!((w.items[a as usize].place, w.items[b as usize].place, w.players[0].hands), (floor, floor, [None, None]));
}

#[test]
fn salt_closes_a_doorway_to_corpo_seco() {
    let mut w = World::new(pick(&LAB, LegendKind::CorpoSeco), setup(1, [0, 1, 2, 3]));
    calm(&mut w);
    w.door_dir[0] = true;
    w.doors[0].open = u16::MAX;
    put(&mut w, 0, mid(4), mid(2), 90);
    assert_eq!(w.focus(0), Some(Focus::Door { door: 0, locked: false, stuck: false, open: true }));
    let s = hand(&mut w, 0, "sal");
    assert_eq!(w.focus(0), Some(Focus::Use { item: s }));
    let door_cell = w.grid.cell(5, 2).unwrap();
    assert!(w.legend_cost(door_cell).is_some());
    let e = press(&mut w, 0, A);
    assert!(e.contains(&Event::Used { player: 0, item: s }));
    assert!(w.legend_cost(door_cell).is_none() && w.legend_solid(5, 2) && !w.player_solid(5, 2));
    (0..act::SALT_TICS).for_each(|_| w.step(&[0; 4]));
    assert!(w.legend_cost(door_cell).is_some());
    let mut c = World::new(pick(&LAB, LegendKind::Cabra), setup(1, [0, 1, 2, 3]));
    c.salt[0] = act::SALT_TICS;
    assert!(c.legend_cost(door_cell).is_some());
}

#[test]
fn legend_avoids_salted_doorway() {
    let mut w = World::new(pick(&LAB, LegendKind::CorpoSeco), setup(1, [0, 1, 2, 3]));
    w.door_dir[0] = true;
    w.doors[0].open = u16::MAX;
    w.legend.pos = Vec2::new(mid(8), mid(2));
    w.salt[0] = act::SALT_TICS;
    w.brain.dirty = true;
    let goal = Vec2::new(mid(2), mid(2));
    let crossed = (0..20 * TICS_PER_SEC).any(|_| {
        w.go(goal, legend::speeds(LegendKind::CorpoSeco).chase);
        w.legend.pos.x < Fx::int(6)
    });
    assert!(!crossed);
    w.salt[0] = 0;
    w.brain.dirty = true;
    let crossed = (0..20 * TICS_PER_SEC).any(|_| {
        w.go(goal, legend::speeds(LegendKind::CorpoSeco).chase);
        w.legend.pos.x < Fx::int(5)
    });
    assert!(crossed);
}

#[test]
fn balde_stuns_cabra() {
    let mut w = World::new(pick(&BALDE, LegendKind::Cabra), setup(1, [0, 1, 2, 3]));
    assert_eq!(w.focus(0), Some(Focus::Item { item: 0 }));
    press(&mut w, 0, A);
    assert_eq!(w.focus(0), Some(Focus::Use { item: 0 }));
    let e = press(&mut w, 0, A);
    assert!(e.contains(&Event::Stunned) && e.contains(&Event::Used { player: 0, item: 0 }));
    assert_eq!((w.legend.mode, w.players[0].state), (LegendMode::Stunned, PlayerState::Free));
    let stunned = (0..act::BALDE_STUN - 3).all(|_| {
        w.step(&[0; 4]);
        w.legend.mode == LegendMode::Stunned
    });
    assert!(stunned);
}

#[test]
fn mula_charges_straight_and_stuns_on_the_wall() {
    let mut w = World::new(pick(&ARENA, LegendKind::Mula), setup(1, [BIA, 1, 2, 3]));
    let run = [UP | B, 0, 0, 0];
    let first = tic(&mut w, run);
    assert!(first.contains(&Event::LegendSaw { player: 0 }) && !first.contains(&Event::LegendSpecial) && w.legend.anim == LegendAnim::Alert);
    let charge = (0..legend::ALERT_TICS).position(|_| tic(&mut w, run).contains(&Event::LegendSpecial));
    assert_eq!(charge, Some(legend::ALERT_TICS as usize - 1));
    let heading = w.legend.heading;
    let mut stunned = false;
    (0..60).for_each(|_| {
        let e = tic(&mut w, [UP | B, 0, 0, 0]);
        stunned |= e.contains(&Event::Stunned);
        assert!(stunned || w.legend.heading == heading);
        assert_eq!(w.players[0].state, PlayerState::Free);
    });
    assert!(stunned && w.legend.mode == LegendMode::Stunned && w.legend.pos.y < Fx::int(2));
}

#[test]
fn tabaco_at_the_lair_sends_matinta_away() {
    let mut w = World::new(pick(&LAB, LegendKind::Matinta), setup(1, [0, 1, 2, 3]));
    let t = w.map.items.len();
    assert!(t > 0);
    put(&mut w, 0, mid(17), mid(2), 90);
    assert_eq!(w.focus(0), None);
    let i = hand(&mut w, 0, "sal");
    w.items[i as usize].kind = ItemKind::Tabaco;
    assert_eq!(w.focus(0), Some(Focus::Use { item: i }));
    press(&mut w, 0, A);
    assert_eq!((w.legend.mode, w.brain.timer), (LegendMode::Leaving, act::TABACO_LEAVE - 2));
    let cries = (0..act::TABACO_LEAVE - 3)
        .filter(|_| {
            w.step(&[0; 4]);
            assert_eq!(w.legend.mode, LegendMode::Leaving);
            w.events().iter().any(|e| *e == Event::LegendCry)
        })
        .count();
    assert!((4..=6).contains(&cries));
}

#[test]
fn cruz_in_benedita_hands_wards_off_corpo_seco() {
    let mut w = World::new(pick(&CORRIDOR, LegendKind::CorpoSeco), setup(2, [BENEDITA, BIA, 2, 3]));
    put(&mut w, 0, mid(5), mid(1), 90);
    put(&mut w, 1, mid(20), mid(2), 90);
    let cruz = w.items.len() as u8;
    w.items.push(ItemState { kind: ItemKind::Cruz, place: ItemPlace::Held { player: 0 } });
    w.players[0].hands[0] = Some(cruz);
    let e = (0..90).flat_map(|_| tic(&mut w, [0; 4])).collect::<Vec<_>>();
    assert!(e.contains(&Event::Stunned) && e.contains(&Event::LegendSpecial));
    assert_eq!((w.players[0].state, w.legend.mode), (PlayerState::Free, LegendMode::Stunned));
    w.legend.pos = Vec2::new(mid(18), mid(2));
    w.legend.mode = LegendMode::Patrol;
    w.legend.heading = deg(90);
    let grabbed = (0..90).any(|_| tic(&mut w, [0; 4]).contains(&Event::Grabbed { player: 1 }));
    assert!(grabbed);
}

#[test]
fn cabra_bleats_and_breathes_fire_when_chasing() {
    let mut w = World::new(pick(&CORRIDOR, LegendKind::Cabra), setup(1, [BIA, 1, 2, 3]));
    put(&mut w, 0, mid(7), mid(1), 90);
    w.legend.pos = Vec2::new(mid(3), mid(1));
    w.legend.heading = deg(90);
    let first = tic(&mut w, [0; 4]);
    assert!(first.contains(&Event::LegendSaw { player: 0 }) && w.legend.anim == LegendAnim::Alert);
    let breath = (0..legend::ALERT_TICS).position(|_| tic(&mut w, [0; 4]).contains(&Event::LegendSpecial));
    assert_eq!((breath, w.legend.anim), (Some(legend::ALERT_TICS as usize - 1), LegendAnim::Special));
    let mut w = World::new(pick(&CORRIDOR, LegendKind::Cabra), setup(1, [BIA, 1, 2, 3]));
    calm(&mut w);
    let cries = (0..60 * TICS_PER_SEC).filter(|_| tic(&mut w, [0; 4]).contains(&Event::LegendCry)).count();
    assert!((3..=4).contains(&cries), "{cries}");
}

#[test]
fn lantern_tap_battery_and_bia() {
    let mut w = World::new(pick(&CORRIDOR, LegendKind::CorpoSeco), setup(2, [BIA, JAIRO, 2, 3]));
    calm(&mut w);
    let e = press(&mut w, 0, A);
    assert!(e.is_empty() || !e.iter().any(|e| matches!(e, Event::Lantern { .. })));
    assert!(w.players[0].lantern);
    press(&mut w, 1, X);
    assert!(w.players[1].lantern);
    let before = (w.players[0].battery, w.players[1].battery);
    (0..300).for_each(|_| w.step(&[0; 4]));
    let (bia, jairo) = (before.0 - w.players[0].battery, before.1 - w.players[1].battery);
    assert!(jairo == 3 * bia && bia > 0);
    (0..31).for_each(|_| w.step(&[A, 0, 0, 0]));
    w.step(&[0; 4]);
    assert!(w.players[0].lantern);
}

#[test]
fn notes_open_and_close() {
    let mut w = World::new(pick(&LAB, LegendKind::CorpoSeco), setup(1, [0, 1, 2, 3]));
    calm(&mut w);
    put(&mut w, 0, mid(10), mid(3), 0);
    assert_eq!(w.focus(0), Some(Focus::Note { note: 0 }));
    press(&mut w, 0, A);
    assert_eq!(w.reading(0), Some(0));
    tic(&mut w, [UP, 0, 0, 0]);
    assert_eq!(w.players[0].pos, Vec2::new(mid(10), mid(3)));
    press(&mut w, 0, A);
    assert_eq!(w.reading(0), None);
}

#[test]
fn bodies_never_enter_walls() {
    let mut w = World::new(pick(&LAB, LegendKind::Cabra), setup(4, [0, 1, 2, 3]));
    let mut rng = Rng::new(11);
    let solid = |w: &World, p: Vec2, legend: bool| match legend {
        true => collide::overlaps(p, &|x, y| w.legend_solid(x, y) && w.map.tile(x, y) != map::Tile::Door),
        false => collide::overlaps(p, &|x, y| w.player_solid(x, y) && w.map.tile(x, y) != map::Tile::Door),
    };
    (0..6000).for_each(|_| {
        let inputs = std::array::from_fn(|_| rng.next_u32() as u16 & (UP | DOWN | LEFT | RIGHT | B | A | X | Y));
        w.step(&inputs);
        assert!(w.players().iter().all(|p| !solid(&w, p.pos, false)));
        assert!(!solid(&w, w.legend.pos, true));
    });
}

fn scripted(w: &mut World, rng: &mut Rng, held: &mut [u16; 4]) {
    let inputs = std::array::from_fn(|p| {
        if rng.below(12) == 0 {
            held[p] = rng.next_u32() as u16 & (UP | DOWN | LEFT | RIGHT | B | A | X | Y | DROP);
        }
        held[p]
    });
    w.step(&inputs);
    if w.outcome() == Outcome::Restart {
        *w = World::new(w.map, w.setup);
    }
}

const GOLDEN: u32 = 0x5c56_be0c;

fn replay(map: &'static Map<'static>) -> u32 {
    let mut w = World::new(map, Setup { seed: 0xC0FFEE, phase: 1, players: 4, characters: [3, 1, 0, 2] });
    let (mut rng, mut held) = (Rng::new(99), [0; 4]);
    (0..6000).fold(0u32, |a, _| {
        scripted(&mut w, &mut rng, &mut held);
        a.rotate_left(5) ^ w.hash()
    })
}

#[test]
fn determinism_golden_hash() {
    let all = || KINDS.into_iter().flat_map(|k| [replay(pick(&LAB, k)), replay(pick(&HIDE, k))]).fold(0u32, |a, h| a.rotate_left(7) ^ h);
    let (a, b) = (all(), all());
    assert_eq!(a, b);
    assert_eq!(a, GOLDEN, "golden {a:#010x}");
    let sobrado = map::campaign()[0];
    let s = replay(sobrado);
    assert_eq!(s, replay(sobrado));
    println!("sobrado replay hash {s:#010x}");
}

#[test]
fn step_does_not_allocate() {
    let maps = KINDS.iter().flat_map(|&k| [pick(&LAB, k), pick(&HIDE, k)]).chain([map::campaign()[0]]);
    maps.for_each(|m| {
        let mut w = World::new(m, setup(4, [0, 1, 2, 3]));
        let (mut rng, mut held) = (Rng::new(5), [0; 4]);
        let mut inputs = [[0u16; 4]; 4000];
        inputs.iter_mut().for_each(|i| {
            *i = std::array::from_fn(|p| {
                if rng.below(12) == 0 {
                    held[p] = rng.next_u32() as u16 & (UP | DOWN | LEFT | RIGHT | B | A | X | Y | DROP);
                }
                held[p]
            })
        });
        let allocs = crate::audio::alloc::count(|| inputs.iter().for_each(|i| w.step(i)));
        assert_eq!(allocs, 0, "{} {:?}", m.name, m.legend);
    });
}



fn grid_town() -> String {
    let (w, h) = (72, 56);
    let tile = |x: i32, y: i32| match (x, y) {
        _ if x == 0 || y == 0 || x == w - 1 || y == h - 1 => '#',
        _ if (x % 8 == 0 && y % 8 == 4) || (y % 8 == 0 && x % 8 == 4) => 'D',
        _ if x % 8 == 0 || y % 8 == 0 => '#',
        _ => '.',
    };
    let rows = |f: &dyn Fn(char) -> char| (0..h).map(|y| (0..w).map(|x| f(tile(x, y))).collect::<String>() + "\n").collect::<String>();
    let doors = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| tile(x, y) == 'D')
        .enumerate()
        .map(|(i, (x, y))| format!("{x} {y} {} {}\n", if x % 8 == 0 { "v" } else { "h" }, if i % 2 == 0 { "open" } else { "closed" }))
        .collect::<String>();
    format!(
        "[map] name=\"cidade grande\" w={w} h={h} sky=none fog=0,0,0 fog_near=5 fog_far=30 legend=corposeco\n[solid]\n{}[rooms]\n{}[area]\n0 rua\n[room]\na casa area=0 floor=taco wall=reboco ceil=forro light=40\n[cam]\n0 0 199 199 1 2 1 2 0 2 60\n[door]\n{doors}[wake]\n0 2.5 2.5 90\n1 3.5 2.5 90\n2 2.5 3.5 90\n3 3.5 3.5 90\n[legend]\nspawn 60.5 50.5 0\npatrol 4 4\npatrol 60 4\npatrol 60 50\npatrol 4 50\npatrol 36 28\npatrol 20 44\nlair 68 52\n[lock]\nsaida code 999\n[exit]\n69 53 70 54 lock=saida\n",
        rows(&|c| c),
        rows(&|c| if c == '#' { '-' } else { 'a' })
    )
}

static TOWN_TEXT: LazyLock<String> = LazyLock::new(grid_town);
static TOWN: LazyLock<[Map<'static>; 4]> = LazyLock::new(|| parsed(&TOWN_TEXT));

#[test]
#[ignore]
fn step_cost_four_players_on_the_largest_map() {
    KINDS.into_iter().for_each(|kind| {
        let mut w = World::new(pick(&TOWN, kind), setup(4, [0, 1, 2, 3]));
        let (mut rng, mut held) = (Rng::new(3), [0; 4]);
        let tics = 30_000;
        let mut worst = std::time::Duration::ZERO;
        let mut stepping = std::time::Duration::ZERO;
        (0..tics).for_each(|_| {
            let inputs = std::array::from_fn(|p| {
                if rng.below(12) == 0 {
                    held[p] = rng.next_u32() as u16 & (UP | DOWN | LEFT | RIGHT | B | A | X | Y | DROP);
                }
                held[p]
            });
            let t = std::time::Instant::now();
            w.step(&inputs);
            let spent = t.elapsed();
            stepping += spent;
            worst = worst.max(spent);
            if w.outcome() != Outcome::Playing {
                w = World::new(w.map, w.setup);
            }
        });
        let mean = stepping / tics;

        let mut flood = vec![0u16; w.grid.cells()];
        let mut heap = field::heap(w.grid.cells());
        let t = std::time::Instant::now();
        let costs = (0..w.grid.cells()).map(|c| w.legend_cost(c).map_or(pass::BLOCKED, |v| v as u8)).collect::<Vec<_>>();
        (0..100).for_each(|_| {
            field::flood(&mut flood, &mut heap, 2 * 72 + 2, field::FAR, None, |c| w.legend_prev(&costs, c));
        });
        println!("{kind:?}: mean {mean:?} worst {worst:?} full flood {:?}", t.elapsed() / 100);
    });
}


fn route(w: &World, goal: usize) -> Vec<u16> {
    let mut dist = vec![u16::MAX; w.grid.cells()];
    let open = |c: usize| match w.map.tiles[c] {
        map::Tile::Floor | map::Tile::Portal => true,
        map::Tile::Door => w.door_of(c).is_none_or(|d| !w.doors[d].locked),
        _ => false,
    };
    let mut queue = std::collections::VecDeque::from([goal]);
    dist[goal] = 0;
    while let Some(c) = queue.pop_front() {
        let (x, y) = w.grid.xy(c);
        [(0, -1), (1, 0), (0, 1), (-1, 0)].into_iter().filter_map(|(dx, dy)| w.grid.cell(x + dx, y + dy)).for_each(|n| {
            if dist[n] == u16::MAX && open(n) {
                dist[n] = dist[c] + 1;
                queue.push_back(n);
            }
        });
    }
    dist
}

fn steer(heading: Ang, aim: Ang) -> (u16, i32) {
    let diff = (aim - heading).0 as i16 as i32;
    let turn = match diff {
        d if d > tank::TURN as i32 / 2 => LEFT,
        d if d < -(tank::TURN as i32) / 2 => RIGHT,
        _ => 0,
    };
    (turn, diff.abs())
}

fn walk_to(w: &mut World, p: usize, x: Fx, y: Fx) {
    let target = Vec2::new(x, y);
    let goal = w.grid.cell_of(target).unwrap();
    let dist = route(w, goal);
    let arrived = (0..4000).any(|_| {
        let pl = w.players[p];
        if (target - pl.pos).len_sq() < sq(Fx::ratio(15, 100)) {
            return true;
        }
        let here = w.grid.cell_of(pl.pos).unwrap();
        let (hx, hy) = w.grid.xy(here);
        let next = [(0, -1), (1, 0), (0, 1), (-1, 0)]
            .into_iter()
            .filter_map(|(dx, dy)| w.grid.cell(hx + dx, hy + dy))
            .filter(|&n| dist[n] < dist[here])
            .min_by_key(|&n| dist[n]);
        let aim = match next {
            Some(n) if n != goal => w.grid.centre(n),
            _ => target,
        };
        let (turn, off) = steer(pl.heading, (aim - pl.pos).ang());
        let shut = next.and_then(|n| w.door_of(n)).filter(|&d| !w.door_passable(d) && !w.door_dir[d]);
        let bits = match shut {
            Some(_) if off < 3641 && w.tic % 2 == 0 => A,
            Some(_) => turn,
            None if off < 8192 => turn | UP,
            None => turn,
        };
        let mut inputs = [0; 4];
        inputs[p] = bits;
        w.step(&inputs);
        false
    });
    assert!(arrived, "player {p} stuck at {:?} heading to {target:?}", w.players[p].pos);
}

fn face(w: &mut World, p: usize, d: i32) {
    let turned = (0..200).any(|_| {
        let (turn, _) = steer(w.players[p].heading, deg(d));
        let mut inputs = [0; 4];
        inputs[p] = turn;
        w.step(&inputs);
        turn == 0
    });
    assert!(turned);
}

const fn sq(f: Fx) -> i64 {
    f.0 as i64 * f.0 as i64
}

#[test]
fn sobrado_is_solvable_by_a_scripted_route() {
    let sobrado = map::campaign()[0];
    let mut w = World::new(sobrado, setup(1, [JAIRO, 0, 1, 3]));
    calm(&mut w);
    let item = |w: &World, id: &str| w.map.item_index(id).unwrap();
    let lock = |w: &World, id: &str| w.map.lock_index(id).unwrap();
    let opened = |w: &World, id: &str| w.locks[lock(w, id) as usize].open;
    let held = |w: &World, id: &str| w.players[0].hands.contains(&Some(item(w, id)));
    let take = |w: &mut World, x: Fx, y: Fx, id: &str| {
        walk_to(w, 0, x, y);
        assert_eq!(w.focus(0), Some(Focus::Item { item: item(w, id) }), "taking {id}");
        press(w, 0, A);
        assert!(held(w, id), "{id}");
    };

    take(&mut w, mid(33), mid(6), "chave_de_fenda");
    walk_to(&mut w, 0, mid(53), mid(11));
    assert!(matches!(w.focus(0), Some(Focus::Lock { needs: Some(ItemKind::ChaveDeFenda), .. })));
    press(&mut w, 0, A);
    assert!(opened(&w, "trinco"));
    press(&mut w, 0, A);
    assert_eq!((w.players[0].area, w.grid.cell_of(w.players[0].pos)), (0, w.grid.cell(25, 13)));

    take(&mut w, mid(24), mid(17), "alicate");
    walk_to(&mut w, 0, mid(25), mid(2));
    face(&mut w, 0, 180);
    assert!(matches!(w.focus(0), Some(Focus::Lock { needs: Some(ItemKind::Alicate), .. })));
    press(&mut w, 0, A);
    assert!(opened(&w, "corrente") && !opened(&w, "saida"));

    walk_to(&mut w, 0, mid(22), mid(16));
    face(&mut w, 0, 0);
    assert!(matches!(w.focus(0), Some(Focus::Lock { code: true, .. })));
    press(&mut w, 0, A);
    [UP, UP, UP, RIGHT, UP, RIGHT, DOWN, DOWN, DOWN, A].into_iter().for_each(|b| {
        press(&mut w, 0, b);
    });
    assert!(opened(&w, "caixa"));
    press(&mut w, 0, A);
    assert_eq!(w.focus(0), Some(Focus::Item { item: item(&w, "fusivel") }));
    press(&mut w, 0, A);
    assert!(held(&w, "fusivel") && held(&w, "alicate"));

    walk_to(&mut w, 0, mid(21), mid(12));
    face(&mut w, 0, 0);
    assert!(matches!(w.focus(0), Some(Focus::Place { needs: ItemKind::Fusivel, .. })));
    press(&mut w, 0, A);
    assert!(opened(&w, "luz") && w.power());

    walk_to(&mut w, 0, mid(4), mid(15));
    face(&mut w, 0, 90);
    assert!(matches!(w.focus(0), Some(Focus::Container { locked: false, .. })));
    press(&mut w, 0, A);
    take(&mut w, mid(4), mid(15), "chavinha");

    walk_to(&mut w, 0, mid(15), mid(5));
    face(&mut w, 0, 180);
    assert!(matches!(w.focus(0), Some(Focus::Lock { needs: Some(ItemKind::Chave), .. })));
    press(&mut w, 0, A);
    assert!(opened(&w, "cristal"));
    press(&mut w, 0, A);
    take(&mut w, mid(15), mid(5), "chave_portao");

    walk_to(&mut w, 0, mid(25), mid(2));
    face(&mut w, 0, 180);
    press(&mut w, 0, A);
    assert!(opened(&w, "cadeado") && opened(&w, "saida"));
    let won = (0..60).any(|_| {
        w.step(&[UP, 0, 0, 0]);
        w.outcome() == Outcome::Escaped
    });
    assert!(won, "{:?}", w.players[0]);
    println!("sobrado solved in {} tics ({} s)", w.tic, w.tic / TICS_PER_SEC);
}

