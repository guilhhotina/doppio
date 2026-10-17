use super::super::World;
use crate::fx::{Fx, Vec2};
use crate::map::Tile;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub const FAR: u32 = u32::MAX;
pub const STEP: u32 = 10;
const DIAGONAL: u32 = 14;
const SHUT: u32 = 20;
const PORTAL: u32 = 30;
const BODY: Fx = Fx::ratio(34, 100);
const SAMPLE: Fx = Fx::ratio(1, 8);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Walker {
    Player { jairo: bool },
    Legend,
}

pub fn enter(w: &World, c: usize, who: Walker) -> Option<u32> {
    match who {
        Walker::Legend => w.legend_cost(c).map(|k| STEP + k as u32),
        Walker::Player { jairo } => match w.map.tiles[c] {
            Tile::Floor | Tile::Portal => Some(STEP),
            Tile::Door => match w.door_of(c) {
                None => Some(STEP),
                Some(d) if w.doors[d].locked || (w.doors[d].stuck && !jairo) => None,
                Some(d) if w.door_passable(d) => Some(STEP),
                Some(_) => Some(STEP + SHUT),
            },
            _ => None,
        },
    }
}

pub fn portals(w: &World) -> impl Iterator<Item = (u8, usize, usize)> + '_ {
    w.map.portals.iter().enumerate().filter(|&(k, _)| !w.portal_locked(k)).filter_map(|(k, p)| {
        let (from, to) = (w.grid.cell(p.x, p.y)?, w.grid.cell(p.to_x, p.to_y)?);
        Some((k as u8, from, to))
    })
}

fn neighbours(w: &World, c: usize, who: Walker) -> impl Iterator<Item = (usize, u32)> + '_ {
    let (x, y) = w.grid.xy(c);
    let open = move |dx: i32, dy: i32| w.grid.cell(x + dx, y + dy).filter(|&n| enter(w, n, who).is_some());
    let sides = [(0, -1), (1, 0), (0, 1), (-1, 0)].map(|(dx, dy)| open(dx, dy).map(|n| (n, STEP)));
    let corners = [(1, -1), (1, 1), (-1, 1), (-1, -1)].map(|(dx, dy)| {
        let flanks = open(dx, 0).is_some() && open(0, dy).is_some();
        flanks.then(|| open(dx, dy)).flatten().map(|n| (n, DIAGONAL))
    });
    sides.into_iter().chain(corners).flatten()
}

pub struct Field {
    pub dist: Vec<u32>,
    heap: BinaryHeap<Reverse<(u32, u32)>>,
}

impl Field {
    pub fn new(cells: usize) -> Field {
        Field { dist: vec![FAR; cells], heap: BinaryHeap::new() }
    }

    fn settle(&mut self, seeds: impl Iterator<Item = usize>) {
        self.dist.fill(FAR);
        self.heap.clear();
        let cells = self.dist.len();
        seeds.filter(|&c| c < cells).for_each(|c| {
            self.dist[c] = 0;
            self.heap.push(Reverse((0, c as u32)));
        });
    }

    fn relax(&mut self, n: usize, d: u32) {
        if d < self.dist[n] {
            self.dist[n] = d;
            self.heap.push(Reverse((d, n as u32)));
        }
    }

    pub fn toward(&mut self, w: &World, goals: impl Iterator<Item = usize>, who: Walker, avoid: &dyn Fn(usize) -> u32) {
        self.settle(goals);
        let links = portals(w).collect::<Vec<_>>();
        while let Some(Reverse((d, c))) = self.heap.pop() {
            let c = c as usize;
            if d != self.dist[c] {
                continue;
            }
            let into = enter(w, c, who).unwrap_or(STEP) + avoid(c);
            neighbours(w, c, who).for_each(|(n, k)| self.relax(n, d + k + into - STEP));
            links.iter().filter(|l| l.2 == c).for_each(|l| self.relax(l.1, d + PORTAL));
        }
    }

    pub fn from(&mut self, w: &World, start: usize, who: Walker) {
        self.settle(std::iter::once(start));
        let links = portals(w).collect::<Vec<_>>();
        while let Some(Reverse((d, c))) = self.heap.pop() {
            let c = c as usize;
            if d != self.dist[c] {
                continue;
            }
            neighbours(w, c, who).for_each(|(n, k)| self.relax(n, d + k + enter(w, n, who).unwrap_or(STEP) - STEP));
            links.iter().filter(|l| l.1 == c).for_each(|l| self.relax(l.2, d + PORTAL));
        }
    }

    pub fn next(&self, w: &World, c: usize, who: Walker) -> Option<Hop> {
        let into = |n: usize| enter(w, n, who).unwrap_or(STEP) - STEP;
        let here = self.dist[c];
        let walk = neighbours(w, c, who).filter(|&(n, _)| self.dist[n] < here).map(|(n, k)| (self.dist[n] + k + into(n), n, Hop::Walk(n)));
        let warp = portals(w).filter(|l| l.1 == c && self.dist[l.2] < here).map(|l| (self.dist[l.2] + PORTAL, usize::MAX, Hop::Warp(l.0)));
        walk.chain(warp).min_by_key(|&(d, n, _)| (d, n)).map(|(_, _, h)| h)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hop {
    Walk(usize),
    Warp(u8),
}

pub fn clear(w: &World, a: Vec2, b: Vec2) -> bool {
    let d = b - a;
    let steps = (d.len().0 / SAMPLE.0).max(1);
    let body = [(-BODY, -BODY), (BODY, -BODY), (-BODY, BODY), (BODY, BODY)];
    (0..=steps).all(|i| {
        let p = a + Vec2::new(Fx(d.x.0 / steps * i), Fx(d.y.0 / steps * i));
        body.iter().all(|&(ox, oy)| !w.player_solid((p.x + ox).floor(), (p.y + oy).floor()))
    })
}
