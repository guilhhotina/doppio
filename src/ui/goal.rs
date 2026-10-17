use super::lore::item;
use super::{Line, line};
use crate::map::{DoorState, LockKind, Map, Tile};
use crate::sim::{ItemKind, ItemPlace, PlayerState, World};

const JAIRO: u8 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Goal {
    Lock(u8),
    Escape,
}

fn tile_of(v: crate::fx::Vec2) -> (i32, i32) {
    (v.x.floor(), v.y.floor())
}

pub fn reach(world: &World) -> Vec<bool> {
    let map = world.map();
    let (w, h) = (map.w, map.h);
    let jairo = world.players().iter().any(|p| p.character == JAIRO);
    let index = |x: i32, y: i32| ((0..w).contains(&x) && (0..h).contains(&y)).then(|| (y * w + x) as usize);
    let open = |x: i32, y: i32| match map.tile(x, y) {
        Tile::Door => map.door_at(x, y).and_then(|d| world.doors().get(d)).map_or(Tile::Door.walkable(), |d| !d.locked && (!d.stuck || jairo || d.open > 0)),
        t => t.walkable(),
    };
    let mut seen = vec![false; (w * h) as usize];
    let mut todo = Vec::with_capacity(seen.len());
    let visit = |x: i32, y: i32, seen: &mut [bool], todo: &mut Vec<(i32, i32)>| {
        if let Some(i) = index(x, y).filter(|&i| !seen[i]) {
            seen[i] = true;
            todo.push((x, y));
        }
    };
    world
        .players()
        .iter()
        .filter(|p| matches!(p.state, PlayerState::Free | PlayerState::Hidden { .. } | PlayerState::Grabbed | PlayerState::Fainted { .. }))
        .for_each(|p| {
            let (x, y) = tile_of(p.pos);
            visit(x, y, &mut seen, &mut todo);
        });
    while let Some((x, y)) = todo.pop() {
        map.portals
            .iter()
            .filter(|p| (p.x, p.y) == (x, y) && p.lock.is_none_or(|l| world.locks().get(l as usize).is_some_and(|s| s.open)))
            .for_each(|p| visit(p.to_x, p.to_y, &mut seen, &mut todo));
        [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)].into_iter().filter(|&(a, b)| open(a, b)).for_each(|(a, b)| visit(a, b, &mut seen, &mut todo));
    }
    seen
}

fn near(map: &Map, reached: &[bool], x: i32, y: i32) -> bool {
    (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (x + dx, y + dy))).any(|(a, b)| {
        (0..map.w).contains(&a) && (0..map.h).contains(&b) && reached[(b * map.w + a) as usize]
    })
}

fn site(world: &World, reached: &[bool], l: u8) -> bool {
    let map = world.map();
    let near = |x: i32, y: i32| near(map, reached, x, y);
    let exit = map.exit.rect;
    let doors = map.doors.iter().any(|d| d.state == DoorState::Locked(l) && near(d.x, d.y));
    let portals = map.portals.iter().any(|p| p.lock == Some(l) && near(p.x, p.y));
    let props = map.props.iter().any(|p| p.lock == Some(l) && {
        let (x, y) = tile_of(p.pos);
        near(x, y)
    });
    let placed = matches!(map.locks[l as usize].kind, LockKind::Place { x, y, .. } if near(x, y));
    let out = map.exit.lock == l && (exit.y0..=exit.y1).any(|y| (exit.x0..=exit.x1).any(|x| near(x, y)));
    doors || portals || props || placed || out
}

fn obtainable(world: &World, reached: &[bool], i: usize) -> bool {
    let map = world.map();
    world.items().get(i).is_some_and(|s| match s.place {
        ItemPlace::Held { .. } => true,
        ItemPlace::Floor { pos, .. } => {
            let (x, y) = tile_of(pos);
            near(map, reached, x, y)
        }
        ItemPlace::Inside { prop } => map.props.get(prop as usize).is_some_and(|p| {
            let (x, y) = tile_of(p.pos);
            p.lock.is_none_or(|l| world.locks().get(l as usize).is_some_and(|s| s.open)) && near(map, reached, x, y)
        }),
        ItemPlace::Used => false,
    })
}

fn held(world: &World, f: impl Fn(usize, ItemKind) -> bool) -> bool {
    world.players().iter().flat_map(|p| p.hands.iter().flatten()).any(|&i| world.items().get(i as usize).is_some_and(|s| f(i as usize, s.kind)))
}

pub fn rank(world: &World, reached: &[bool], l: u8) -> Option<u8> {
    let at = site(world, reached, l);
    let kinds = |k: ItemKind| (0..world.items().len()).filter(move |&i| world.items()[i].kind == k);
    let (has, can) = match world.map().locks[l as usize].kind {
        LockKind::All(_) => return None,
        LockKind::Code { .. } => (false, true),
        LockKind::Key(i) => (held(world, |j, _| j == i as usize), obtainable(world, reached, i as usize)),
        LockKind::Place { item: i, .. } => {
            let k = world.items()[i as usize].kind;
            (held(world, |_, j| j == k), kinds(k).any(|i| obtainable(world, reached, i)))
        }
        LockKind::Tool(k) => (held(world, |_, j| j == k), kinds(k).any(|i| obtainable(world, reached, i))),
    };
    Some(match (at, has, can) {
        (true, true, _) => 0,
        (true, false, true) => 1,
        (false, _, true) => 2,
        _ => 3,
    })
}

pub fn goal(world: &World) -> Goal {
    let exit = world.map().exit.lock;
    if world.locks().get(exit as usize).is_some_and(|s| s.open) {
        return Goal::Escape;
    }
    let reached = reach(world);
    (0..world.locks().len() as u8)
        .filter(|&l| !world.locks()[l as usize].open)
        .filter_map(|l| rank(world, &reached, l).map(|r| (r, l)))
        .min()
        .map_or(Goal::Escape, |(_, l)| Goal::Lock(l))
}

pub fn say(map: &Map, g: Goal) -> Line {
    let lock = |l: u8| map.locks.get(l as usize);
    match g {
        Goal::Escape => match lock(map.exit.lock).and_then(|l| l.hint) {
            Some(h) => line(format_args!("{h}")),
            None => line(format_args!("Fujam pela saída!")),
        },
        Goal::Lock(l) => match lock(l) {
            Some(crate::map::Lock { hint: Some(h), .. }) => line(format_args!("{h}")),
            Some(crate::map::Lock { kind: LockKind::Code { len, .. }, .. }) => line(format_args!("Descubra o segredo de {len} números")),
            Some(crate::map::Lock { kind: LockKind::Tool(k), .. }) => line(format_args!("Ache: {}", item(*k))),
            Some(crate::map::Lock { kind: LockKind::Key(_), .. }) => line(format_args!("Ache a chave certa")),
            Some(crate::map::Lock { kind: LockKind::Place { item: i, .. }, .. }) => {
                line(format_args!("Leve {} ao lugar certo", map.items.get(*i as usize).map_or("o item", |it| item(it.kind))))
            }
            _ => line(format_args!("Explore e procure uma saída")),
        },
    }
}

pub fn opened(world: &World) -> (usize, usize) {
    let leaf = |l: usize| !matches!(world.map().locks[l].kind, LockKind::All(_));
    let all = (0..world.locks().len()).filter(|&l| leaf(l));
    let open = all.clone().filter(|&l| world.locks()[l].open).count();
    (open, all.count())
}

pub fn fotos(world: &World) -> (usize, usize) {
    let fotos = world.items().iter().filter(|s| s.kind == ItemKind::Foto);
    (fotos.clone().filter(|s| s.place == ItemPlace::Used).count(), fotos.count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::campaign;
    use crate::sim::Setup;

    fn world(phase: usize) -> World {
        World::new(campaign()[phase], Setup { seed: 1, phase: phase as u8, players: 1, characters: [1, 0, 2, 3] })
    }

    #[test]
    fn every_map_starts_with_a_readable_objective() {
        campaign().iter().enumerate().for_each(|(i, m)| {
            let w = world(i);
            let g = goal(&w);
            assert!(matches!(g, Goal::Lock(_)), "{}: {g:?}", m.name);
            let text = say(m, g);
            assert!(!text.as_str().is_empty() && text.as_str().chars().count() <= 44, "{}: {}", m.name, text.as_str());
            assert!(m.locks.iter().all(|l| l.hint.is_some()), "{} has a lock without hint", m.name);
            assert_eq!(opened(&w).0, 0);
        });
    }

    #[test]
    fn sobrado_objective_starts_where_the_player_can_act() {
        let w = world(0);
        let reached = reach(&w);
        let Goal::Lock(l) = goal(&w) else { panic!("no lock goal") };
        assert!(site(&w, &reached, l), "first objective {} is out of reach", w.map().locks[l as usize].id);
        assert!(rank(&w, &reached, l).is_some_and(|r| r <= 1));
        assert_eq!(fotos(&w), (0, 1));
    }

    #[test]
    fn reach_is_closed_and_only_crosses_open_tiles() {
        campaign().iter().enumerate().for_each(|(i, m)| {
            let w = world(i);
            let reached = reach(&w);
            let at = |x: i32, y: i32| (0..m.w).contains(&x) && (0..m.h).contains(&y) && reached[(y * m.w + x) as usize];
            let open = |x: i32, y: i32| match m.tile(x, y) {
                Tile::Door => m.door_at(x, y).and_then(|d| w.doors().get(d)).map_or(Tile::Door.walkable(), |d| !d.locked && (!d.stuck || d.open > 0)),
                t => t.walkable(),
            };
            let starts: Vec<(i32, i32)> = w.players().iter().map(|p| tile_of(p.pos)).chain(m.portals.iter().map(|p| (p.to_x, p.to_y))).collect();
            (0..m.h).flat_map(|y| (0..m.w).map(move |x| (x, y))).filter(|&(x, y)| at(x, y)).for_each(|(x, y)| {
                assert!(open(x, y) || starts.contains(&(x, y)), "{}: reached closed tile {x},{y}", m.name);
                [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)].iter().for_each(|&(a, b)| assert!(at(a, b) || !open(a, b), "{}: open tile {a},{b} next to {x},{y} not reached", m.name));
            });
            assert!(starts.iter().take(w.players().len()).all(|&(x, y)| at(x, y)));
        });
    }
}
