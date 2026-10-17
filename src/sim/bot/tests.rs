use super::{Bot, LIMIT};
use crate::map::{self, Map};
use crate::sim::{Event, Outcome, Setup, TICS_PER_SEC, World};

#[derive(Clone, Debug)]
pub struct Run {
    pub won: bool,
    pub tics: u32,
    pub desmaios: u32,
    pub chases: u32,
    pub restarts: u32,
    pub hash: u32,
    pub log: Vec<String>,
    pub trace: Vec<String>,
}

pub fn play(map: &'static Map<'static>, setup: Setup, limit: u32) -> Run {
    let fresh = || {
        let w = World::new(map, setup);
        let bot = Bot::new(&w, setup.seed);
        (w, bot)
    };
    let (mut w, mut bot) = fresh();
    let (mut tics, mut desmaios, mut chases, mut restarts, mut log, mut trace) = (0, 0, 0, 0, Vec::new(), Vec::new());
    while tics < limit && w.outcome() != Outcome::Escaped {
        let inputs = bot.inputs(&w);
        w.step(&inputs);
        tics += 1;
        chases += w.events().iter().filter(|e| matches!(e, Event::LegendSaw { .. })).count() as u32;
        w.events().iter().for_each(|e| {
            if let Event::Fainted { player } = *e {
                desmaios += 1;
                let (pl, lg) = (w.players[player as usize], w.legend);
                let line = format!("t{} p{player} fainted at {},{} legend at {},{}", w.tic, pl.pos.x.floor(), pl.pos.y.floor(), lg.pos.x.floor(), lg.pos.y.floor());
                bot.shared.trace.push(line);
            }
        });
        if w.outcome() == Outcome::Restart {
            restarts += 1;
            log.append(&mut bot.shared.log);
            trace.append(&mut bot.shared.trace);
            trace.push("restart".to_string());
            (w, bot) = fresh();
        }
    }
    log.append(&mut bot.shared.log);
    trace.append(&mut bot.shared.trace);
    Run { won: w.outcome() == Outcome::Escaped, tics, desmaios, chases, restarts, hash: w.hash(), log, trace }
}

fn phase(name: &str) -> (u8, &'static Map<'static>) {
    let all = map::campaign();
    let i = all.iter().position(|m| m.name.to_lowercase().contains(name)).unwrap_or_else(|| panic!("no map {name}"));
    (i as u8, all[i])
}

fn setup(phase: u8, seed: u32, characters: &[u8]) -> Setup {
    let mut c = [0, 1, 2, 3];
    c[..characters.len()].copy_from_slice(characters);
    Setup { seed, phase, players: characters.len() as u8, characters: c }
}

fn solo(name: &str, seed: u32, character: u8) -> Run {
    let (i, m) = phase(name);
    play(m, setup(i, seed, &[character]), LIMIT)
}

fn check(name: &str, run: &Run) {
    println!("{name}: won {} in {} s, desmaios {}, restarts {}, hash {:#010x}", run.won, run.tics / TICS_PER_SEC, run.desmaios, run.restarts, run.hash);
    run.log.iter().for_each(|l| println!("  {l}"));
    if !run.won {
        run.trace.iter().rev().take(60).rev().for_each(|l| println!("  | {l}"));
    }
    assert!(run.won, "{name} not escaped: {:?}", run.log);
}

#[test]
fn bot_escapes_the_sobrado() {
    check("sobrado", &solo("sobrado", 1, 0));
}

#[test]
fn bot_escapes_the_cidade() {
    check("cidade", &solo("cidade", 2, 1));
}

#[test]
fn bot_escapes_the_igreja() {
    check("igreja", &solo("igreja", 3, 2));
}

#[test]
fn bot_escapes_the_mata() {
    check("mata", &solo("mata", 4, 3));
}

#[test]
fn bot_escapes_the_cemiterio() {
    check("cemiterio", &solo("cemit", 5, 0));
}

const MAPS: [&str; 5] = ["sobrado", "cidade", "igreja", "mata", "cemit"];
const TEAMS: [&[u8]; 5] = [&[0], &[1], &[2], &[3], &[0, 1, 2, 3]];

fn median(mut v: Vec<u32>) -> u32 {
    v.sort_unstable();
    v.get(v.len() / 2).copied().unwrap_or(0)
}

fn sweep(name: &str, seeds: u32) -> String {
    let (i, m) = phase(name);
    let runs = TEAMS
        .iter()
        .flat_map(|team| (1..=seeds).map(move |seed| (team, seed)))
        .map(|(team, seed)| (team, seed, play(m, setup(i, seed, team), LIMIT)))
        .collect::<Vec<_>>();
    let wins = runs.iter().filter(|r| r.2.won).collect::<Vec<_>>();
    let failures = runs
        .iter()
        .filter(|r| !r.2.won || !r.2.log.is_empty() || r.2.restarts > 0)
        .map(|(team, seed, r)| format!("  {team:?} seed {seed}: won {} restarts {} {:?} | {:?}", r.won, r.restarts, r.log, r.trace.iter().rev().take(8).rev().collect::<Vec<_>>()))
        .collect::<Vec<_>>();
    let per_team = TEAMS
        .iter()
        .map(|team| {
            let mine = runs.iter().filter(|r| r.0 == team).collect::<Vec<_>>();
            let won = mine.iter().filter(|r| r.2.won).count();
            let t = median(mine.iter().filter(|r| r.2.won).map(|r| r.2.tics / TICS_PER_SEC).collect());
            format!("{team:?} {won}/{} med {t}s", mine.len())
        })
        .collect::<Vec<_>>();
    format!(
        "{name}: {}/{} won, median {} s, max {} s, desmaios total {} (median {}, max {}), sightings median {}, restarts {} | {}\n{}",
        wins.len(),
        runs.len(),
        median(wins.iter().map(|r| r.2.tics / TICS_PER_SEC).collect()),
        wins.iter().map(|r| r.2.tics / TICS_PER_SEC).max().unwrap_or(0),
        runs.iter().map(|r| r.2.desmaios).sum::<u32>(),
        median(runs.iter().map(|r| r.2.desmaios).collect()),
        runs.iter().map(|r| r.2.desmaios).max().unwrap_or(0),
        median(runs.iter().map(|r| r.2.chases).collect()),
        runs.iter().map(|r| r.2.restarts).sum::<u32>(),
        per_team.join(", "),
        failures.join("\n")
    )
}

#[test]
#[ignore]
fn bot_sweep_all_maps() {
    let seeds = std::env::var("BOT_SEEDS").ok().and_then(|s| s.parse().ok()).unwrap_or(6);
    let only = std::env::var("BOT_MAP").ok();
    let names = MAPS.iter().filter(|n| only.as_deref().is_none_or(|o| o == **n)).collect::<Vec<_>>();
    let reports = std::thread::scope(|s| names.iter().map(|&&n| s.spawn(move || sweep(n, seeds))).collect::<Vec<_>>().into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>());
    reports.iter().for_each(|r| println!("{r}"));
    assert!(reports.iter().all(|r| r.lines().count() == 1 || !r.contains("won false")));
}

#[test]
fn bot_run_is_deterministic() {
    let (i, m) = phase("sobrado");
    let s = setup(i, 9, &[0, 1, 2, 3]);
    let (a, b) = (play(m, s, 3000), play(m, s, 3000));
    assert_eq!((a.hash, a.tics, a.desmaios), (b.hash, b.tics, b.desmaios));
    println!("bot sobrado 4p 3000 tics hash {:#010x}", a.hash);
}

#[test]
#[ignore]
fn bot_hashes_per_map() {
    MAPS.iter().enumerate().for_each(|(k, n)| {
        let (i, m) = phase(n);
        let r = play(m, setup(i, 40 + k as u32, &[0, 1, 2, 3]), LIMIT);
        println!("bot hash {n}: {:#010x} won {} tics {}", r.hash, r.won, r.tics);
    });
}

#[test]
#[ignore]
fn bot_debug_one_run() {
    let env = |k: &str| std::env::var(k).ok();
    let name = env("BOT_MAP").unwrap_or("sobrado".to_string());
    let seed = env("BOT_SEED").and_then(|s| s.parse().ok()).unwrap_or(1);
    let team = env("BOT_TEAM").unwrap_or("0".to_string()).bytes().map(|b| b - b'0').collect::<Vec<_>>();
    let (i, m) = phase(&name);
    let r = play(m, setup(i, seed, &team), LIMIT);
    r.trace.iter().for_each(|l| println!("| {l}"));
    r.log.iter().for_each(|l| println!("! {l}"));
    println!("won {} tics {} desmaios {} restarts {}", r.won, r.tics, r.desmaios, r.restarts);
}
