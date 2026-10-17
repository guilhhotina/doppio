use super::super::{centre, Focus, ItemKind, ItemPlace, PlayerState, World};
use super::nav::{Field, FAR};
use crate::fx::{Fx, Vec2};
use crate::map::{DoorState, LockKind, Tile};

const ITEM_STAND: i64 = sq(Fx::ratio(125, 100));
const PROP_STAND: i64 = sq(Fx::ratio(14, 10));
const STACKED: i64 = sq(Fx::ratio(35, 100));
const PLACE_STAND: i64 = sq(Fx::ratio(15, 10));
const WAKE_RANGE: u32 = 120;

const fn sq(f: Fx) -> i64 {
    f.0 as i64 * f.0 as i64
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Job {
    Take(u8),
    Open(u16),
    Unlock(u8),
    Read(u16),
    Leave,
    Wake(u8),
    Hide(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Site {
    pub cell: usize,
    pub aim: Option<Vec2>,
}

fn stand(w: &World, x: i32, y: i32) -> Option<usize> {
    w.grid.cell(x, y).filter(|&c| matches!(w.map.tiles[c], Tile::Floor | Tile::Portal))
}

fn ring(w: &World, x: i32, y: i32, diagonal: bool) -> impl Iterator<Item = (usize, Vec2)> + '_ {
    (-1..=1)
        .flat_map(move |dy| (-1..=1).map(move |dx| (dx, dy)))
        .filter(move |&(dx, dy)| diagonal || dx == 0 || dy == 0)
        .filter_map(move |(dx, dy)| stand(w, x + dx, y + dy).map(|c| (c, w.grid.centre(c))))
}

fn near(w: &World, at: Vec2, reach: i64) -> Vec<Site> {
    ring(w, at.x.floor(), at.y.floor(), true)
        .filter(|&(_, mid)| (mid - at).len_sq() <= reach)
        .map(|(cell, _)| Site { cell, aim: Some(at) })
        .collect()
}

fn facing_tile(w: &World, x: i32, y: i32) -> Vec<Site> {
    ring(w, x, y, false).map(|(cell, _)| Site { cell, aim: Some(centre(x, y)) }).collect()
}

fn in_exit_tree(w: &World, root: u8, l: u8, depth: u8) -> bool {
    root == l
        || match &w.map.locks[root as usize].kind {
            LockKind::All(parts) if depth < 8 => parts.iter().any(|&q| in_exit_tree(w, q, l, depth + 1)),
            _ => false,
        }
}

fn lock_sites(w: &World, l: u8) -> Vec<Site> {
    let m = w.map;
    if let LockKind::Place { x, y, .. } = m.locks[l as usize].kind {
        return ring(w, x, y, true)
            .filter(|&(_, mid)| (mid - centre(x, y)).len_sq() <= PLACE_STAND)
            .map(|(cell, _)| Site { cell, aim: Some(centre(x, y)) })
            .collect();
    }
    let doors = m.doors.iter().filter(|d| d.state == DoorState::Locked(l)).flat_map(|d| facing_tile(w, d.x, d.y));
    let portals = m.portals.iter().filter(|k| k.lock == Some(l)).flat_map(|k| {
        let on = stand(w, k.x, k.y).map(|cell| Site { cell, aim: None });
        on.into_iter().chain(facing_tile(w, k.x, k.y))
    });
    let props = m.props.iter().filter(|k| k.lock == Some(l)).flat_map(|k| near(w, k.pos, PROP_STAND));
    let attached = doors.chain(portals).chain(props).collect::<Vec<_>>();
    match attached.is_empty() && in_exit_tree(w, m.exit.lock, l, 0) {
        true => exit_cells(w, 1).into_iter().map(|cell| Site { cell, aim: None }).collect(),
        false => attached,
    }
}

fn exit_cells(w: &World, grow: i32) -> Vec<usize> {
    let r = w.map.exit.rect;
    (r.y0 - grow..=r.y1 + grow).flat_map(|y| (r.x0 - grow..=r.x1 + grow).filter_map(move |x| stand(w, x, y))).collect()
}

pub fn sites(w: &World, job: Job) -> Vec<Site> {
    match job {
        Job::Take(i) => match w.items[i as usize].place {
            ItemPlace::Floor { pos, .. } => near(w, pos, ITEM_STAND),
            _ => Vec::new(),
        },
        Job::Open(k) => near(w, w.map.props[k as usize].pos, PROP_STAND),
        Job::Unlock(l) => lock_sites(w, l),
        Job::Read(n) => {
            let note = w.map.notes[n as usize];
            let on = stand(w, note.x, note.y).map(|cell| Site { cell, aim: None });
            on.into_iter().chain(facing_tile(w, note.x, note.y)).collect()
        }
        Job::Leave => exit_cells(w, 0).into_iter().map(|cell| Site { cell, aim: None }).collect(),
        Job::Wake(q) => near(w, w.players[q as usize].pos, ITEM_STAND),
        Job::Hide(s) => {
            let h = w.map.hides[s as usize];
            facing_tile(w, h.x, h.y)
        }
    }
}

pub fn wants(job: Job, focus: Option<Focus>) -> bool {
    match (job, focus) {
        (Job::Take(i), Some(Focus::Item { item })) => i == item,
        (Job::Open(k), Some(Focus::Container { prop, locked: false })) => k == prop,
        (Job::Unlock(l), Some(Focus::Lock { lock, .. } | Focus::Place { lock, .. })) => l == lock,
        (Job::Read(n), Some(Focus::Note { note })) => n == note,
        (Job::Wake(q), Some(Focus::Friend { player })) => q == player,
        (Job::Hide(s), Some(Focus::Hide { spot })) => s == spot,
        _ => false,
    }
}

pub fn fits(w: &World, l: u8, item: u8) -> bool {
    let kind = w.items[item as usize].kind;
    match w.map.locks[l as usize].kind {
        LockKind::Key(k) => k == item,
        LockKind::Tool(t) => kind == t,
        LockKind::Place { item: j, .. } => w.items.get(j as usize).is_some_and(|s| s.kind == kind),
        _ => false,
    }
}

fn consumed(w: &World, l: u8) -> bool {
    !matches!(w.map.locks[l as usize].kind, LockKind::Tool(_))
}

pub fn digits(text: &str) -> String {
    text.chars().filter(char::is_ascii_digit).collect()
}

pub fn code_of(w: &World, l: u8) -> Option<String> {
    match w.map.locks[l as usize].kind {
        LockKind::Code { digits, len } => Some(digits[..len as usize].iter().map(|d| char::from(b'0' + d)).collect()),
        _ => None,
    }
}

pub fn reveals(w: &World, n: u16, l: u8) -> bool {
    code_of(w, l).is_some_and(|code| digits(w.map.notes[n as usize].text).contains(&code))
}

pub fn usable(w: &World, p: usize, l: u8, known: &[bool]) -> bool {
    let lock = &w.map.locks[l as usize];
    !w.locks[l as usize].open
        && match lock.kind {
            LockKind::Code { .. } => known[l as usize],
            LockKind::All(_) => false,
            _ => w.players[p].hands.iter().flatten().any(|&i| fits(w, l, i)),
        }
}

pub fn valid(w: &World, p: usize, job: Job, known: &[bool]) -> bool {
    match job {
        Job::Take(i) => matches!(w.items[i as usize].place, ItemPlace::Floor { .. }),
        Job::Open(k) => w.props[k as usize].open == 0 && w.lock_open(w.map.props[k as usize].lock),
        Job::Unlock(l) => usable(w, p, l, known),
        Job::Read(n) => (0..w.locks.len() as u8).any(|l| !known[l as usize] && !w.locks[l as usize].open && reveals(w, n, l)),
        Job::Leave => w.lock_open(Some(w.map.exit.lock)),
        Job::Wake(q) => matches!(w.players[q as usize].state, PlayerState::Fainted { .. }),
        Job::Hide(s) => !(0..w.count).any(|q| q != p && w.players[q].state == PlayerState::Hidden { spot: s }),
    }
}

pub fn cost(w: &World, job: Job, reach: &Field) -> u32 {
    sites(w, job).iter().map(|s| reach.dist[s.cell]).min().unwrap_or(FAR)
}

fn uncovered(w: &World, reach: &Field) -> Vec<u8> {
    let held = w.players().iter().flat_map(|pl| pl.hands).flatten().collect::<Vec<_>>();
    let mut spent = Vec::new();
    (0..w.locks.len() as u8)
        .filter(|&l| !w.locks[l as usize].open && cost(w, Job::Unlock(l), reach) != FAR)
        .filter(|&l| {
            let cover = held.iter().copied().find(|&i| fits(w, l, i) && !spent.contains(&i));
            if let Some(i) = cover.filter(|_| consumed(w, l)) {
                spent.push(i);
            }
            cover.is_none()
        })
        .collect()
}

pub struct Ask<'a> {
    pub reach: &'a Field,
    pub known: &'a [bool],
    pub taken: &'a [Job],
    pub banned: &'a [(Job, u32)],
}

pub fn choose(w: &World, p: usize, ask: &Ask) -> Option<Job> {
    let free = |j: Job| !ask.taken.contains(&j) && !ask.banned.iter().any(|&(b, until)| b == j && until > w.tic);
    let priced = |j: Job| Some(cost(w, j, ask.reach)).filter(|&c| c != FAR).map(|c| (j, c));
    let leave = valid(w, p, Job::Leave, ask.known).then_some(Job::Leave).into_iter();
    let wake = (0..w.count as u8).map(Job::Wake).filter(|&j| valid(w, p, j, ask.known));
    let urgent = leave.chain(wake).filter(|&j| free(j) || j == Job::Leave).filter_map(priced).filter(|&(j, c)| j == Job::Leave || c <= WAKE_RANGE);
    let unlock = (0..w.locks.len() as u8).filter(|&l| usable(w, p, l, ask.known)).map(Job::Unlock).filter(|&j| free(j)).filter_map(priced);
    let best = |it: &mut dyn Iterator<Item = (Job, u32)>| it.min_by_key(|&(_, c)| c).map(|(j, _)| j);
    best(&mut urgent.collect::<Vec<_>>().into_iter())
        .or_else(|| best(&mut unlock.collect::<Vec<_>>().into_iter()))
        .or_else(|| best(&mut gather(w, ask).into_iter().filter(|&j| free(j)).filter_map(priced)))
}

fn gather(w: &World, ask: &Ask) -> Vec<Job> {
    let need = uncovered(w, ask.reach);
    let wanted = |i: u8| need.iter().any(|&l| fits(w, l, i));
    let take = (0..w.items.len() as u8).filter(|&i| matches!(w.items[i as usize].place, ItemPlace::Floor { .. }) && wanted(i)).map(Job::Take);
    let open = (0..w.map.props.len() as u16)
        .filter(|&k| {
            let inside = (0..w.items.len() as u8).any(|i| w.items[i as usize].place == ItemPlace::Inside { prop: k } && wanted(i));
            inside && w.props[k as usize].open == 0 && w.lock_open(w.map.props[k as usize].lock)
        })
        .map(Job::Open);
    let codes = (0..w.locks.len() as u8)
        .filter(|&l| code_of(w, l).is_some() && !ask.known[l as usize] && !w.locks[l as usize].open && cost(w, Job::Unlock(l), ask.reach) != FAR)
        .collect::<Vec<_>>();
    let read = (0..w.map.notes.len() as u16).filter(|&n| codes.iter().any(|&l| reveals(w, n, l))).map(Job::Read);
    take.chain(open).chain(read).collect()
}

pub fn harmless(w: &World, focus: Option<Focus>) -> bool {
    matches!(focus, Some(Focus::Item { item }) if matches!(w.items[item as usize].kind, ItemKind::Foto | ItemKind::LanternaPilha))
}

pub fn stacked(w: &World, job: Job, focus: Option<Focus>) -> bool {
    let at = |i: u8| match w.items[i as usize].place {
        ItemPlace::Floor { pos, .. } => Some(pos),
        _ => None,
    };
    match (job, focus) {
        (Job::Take(want), Some(Focus::Item { item })) => at(want).zip(at(item)).is_some_and(|(a, b)| (a - b).len_sq() <= STACKED),
        _ => false,
    }
}
