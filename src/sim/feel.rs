use super::bot::{Bot, LIMIT};
use super::{Event, Outcome, Setup, World, TICS_PER_SEC};
use crate::map;
use crate::ui::goal::{self, Goal};

#[derive(Clone, Copy, Debug, Default)]
struct Feel {
    won: bool,
    secs: u32,
    first_saw: Option<u32>,
    chases: u32,
    lost: u32,
    desmaios: u32,
    lost_goal: u32,
}

fn stuck(w: &World) -> bool {
    match goal::goal(w) {
        Goal::Escape => false,
        Goal::Lock(l) => goal::rank(w, &goal::reach(w), l).is_none_or(|r| r > 2),
    }
}

fn measure(phase: u8, seed: u32, team: &[u8]) -> Feel {
    let mut characters = [0, 1, 2, 3];
    characters[..team.len()].copy_from_slice(team);
    let setup = Setup { seed, phase, players: team.len() as u8, characters };
    let mut w = World::new(map::campaign()[phase as usize], setup);
    let mut bot = Bot::new(&w, seed);
    let mut f = Feel::default();
    while w.tic() < LIMIT && w.outcome() == Outcome::Playing {
        let inputs = bot.inputs(&w);
        w.step(&inputs);
        w.events().iter().for_each(|e| match e {
            Event::LegendSaw { .. } => {
                f.chases += 1;
                f.first_saw = f.first_saw.or(Some(w.tic() / TICS_PER_SEC));
            }
            Event::LegendLost => f.lost += 1,
            Event::Fainted { .. } => f.desmaios += 1,
            _ => {}
        });
        if w.tic() % (5 * TICS_PER_SEC) == 0 && stuck(&w) {
            f.lost_goal += 1;
        }
    }
    Feel { won: w.outcome() == Outcome::Escaped, secs: w.tic() / TICS_PER_SEC, ..f }
}

fn median(mut v: Vec<u32>) -> u32 {
    v.sort_unstable();
    v.get(v.len() / 2).copied().unwrap_or(0)
}

fn report(phase: u8, seeds: u32) -> String {
    let teams: [&[u8]; 3] = [&[0], &[1], &[0, 1, 2, 3]];
    let runs = teams.iter().flat_map(|t| (1..=seeds).map(move |s| measure(phase, s, t))).collect::<Vec<_>>();
    let n = runs.len() as u32;
    format!(
        "{:<10} won {}/{n}  med {:>4}s  first sighting med {:>4}s (never {})  chases/run {:.1}  lost/run {:.1}  desmaios/run {:.1}  objective dead-ends {}",
        map::campaign()[phase as usize].name.split('—').next_back().unwrap_or("").trim(),
        runs.iter().filter(|r| r.won).count(),
        median(runs.iter().map(|r| r.secs).collect()),
        median(runs.iter().filter_map(|r| r.first_saw).collect()),
        runs.iter().filter(|r| r.first_saw.is_none()).count(),
        runs.iter().map(|r| r.chases).sum::<u32>() as f32 / n as f32,
        runs.iter().map(|r| r.lost).sum::<u32>() as f32 / n as f32,
        runs.iter().map(|r| r.desmaios).sum::<u32>() as f32 / n as f32,
        runs.iter().map(|r| r.lost_goal).sum::<u32>(),
    )
}

#[test]
fn objective_always_points_somewhere_actionable() {
    [measure(0, 3, &[1]), measure(4, 1, &[0])].iter().for_each(|f| assert!(f.won && f.lost_goal == 0, "{f:?}"));
}

#[test]
#[ignore]
fn feel_metrics() {
    let seeds = std::env::var("FEEL_SEEDS").ok().and_then(|s| s.parse().ok()).unwrap_or(4);
    let phases = map::campaign().len() as u8;
    let lines = std::thread::scope(|s| (0..phases).map(|p| s.spawn(move || report(p, seeds))).collect::<Vec<_>>().into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>());
    lines.iter().for_each(|l| println!("{l}"));
}