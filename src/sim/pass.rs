use super::field::{grid_edges, Edges};
use super::grid::NONE;
use super::{los, LegendKind, World};
use crate::fx::Vec2;
use crate::map::Tile;

const HALF_OPEN: u16 = 1 << 15;
pub const DOOR_OPENING: u16 = 10;
pub const PORTAL_COST: u16 = 20;
pub const WINDOW_MUFFLE: u16 = 30;
pub const DOOR_MUFFLE: u16 = 40;
pub const BLOCKED: u8 = u8::MAX;

impl World {
    pub(super) fn door_passable(&self, d: usize) -> bool {
        self.door_dir[d] && self.doors[d].open >= HALF_OPEN
    }

    pub(super) fn door_of(&self, c: usize) -> Option<usize> {
        Some(self.grid.door[c] as usize).filter(|&d| d != NONE as usize && d < self.doors.len())
    }

    pub(super) fn lock_open(&self, lock: Option<u8>) -> bool {
        lock.is_none_or(|l| self.locks.get(l as usize).is_none_or(|s| s.open))
    }

    pub(super) fn portal_locked(&self, k: usize) -> bool {
        !self.lock_open(self.map.portals[k].lock)
    }

    fn walk_cell(&self, c: usize) -> bool {
        match self.map.tiles[c] {
            Tile::Floor | Tile::Portal => true,
            Tile::Door => self.door_of(c).is_none_or(|d| self.door_passable(d)),
            _ => false,
        }
    }

    pub(super) fn salted(&self, c: usize) -> bool {
        self.legend.kind == LegendKind::CorpoSeco && self.door_of(c).is_some_and(|d| self.salt[d] > 0)
    }

    pub(super) fn player_solid(&self, x: i32, y: i32) -> bool {
        self.grid.cell(x, y).is_none_or(|c| !self.walk_cell(c))
    }

    pub(super) fn legend_solid(&self, x: i32, y: i32) -> bool {
        self.grid.cell(x, y).is_none_or(|c| !self.walk_cell(c) || self.salted(c))
    }

    pub(super) fn opaque(&self, x: i32, y: i32) -> bool {
        self.grid.cell(x, y).is_none_or(|c| match self.map.tiles[c] {
            Tile::Floor | Tile::Blocker | Tile::Hide | Tile::Portal | Tile::Water | Tile::Fence => false,
            Tile::Door => self.door_of(c).is_some_and(|d| !self.door_passable(d)),
            _ => true,
        })
    }

    pub(super) fn sight_clear(&self, a: Vec2, b: Vec2) -> bool {
        los::clear(a, b, |x, y| self.opaque(x, y))
    }

    pub(super) fn legend_cost(&self, c: usize) -> Option<u16> {
        match self.map.tiles[c] {
            Tile::Floor | Tile::Portal => Some(0),
            Tile::Door => match self.door_of(c) {
                None => Some(0),
                Some(d) if self.doors[d].locked || self.doors[d].stuck || self.salted(c) => None,
                Some(d) if self.door_passable(d) => Some(0),
                Some(_) => Some(DOOR_OPENING),
            },
            _ => None,
        }
    }

    fn sound_cost(&self, c: usize) -> Option<u16> {
        match self.map.tiles[c] {
            Tile::Wall | Tile::Void => None,
            Tile::Window => Some(WINDOW_MUFFLE),
            Tile::Door => Some(self.door_of(c).filter(|&d| !self.door_passable(d)).map_or(0, |_| DOOR_MUFFLE)),
            _ => Some(0),
        }
    }

    fn portal_out(&self, c: usize) -> Option<usize> {
        let k = self.grid.portal[c];
        (k != NONE && !self.portal_locked(k as usize)).then(|| self.map.portals[k as usize]).and_then(|p| self.grid.cell(p.to_x, p.to_y))
    }

    fn portal_back(&self, c: usize) -> Option<usize> {
        let k = self.grid.portal_in[c];
        (k != NONE && !self.portal_locked(k as usize)).then(|| self.map.portals[k as usize]).and_then(|p| self.grid.cell(p.x, p.y))
    }

    pub(super) fn legend_next(&self, c: usize) -> Edges {
        let e = grid_edges(self.grid.w, self.grid.h, c, |n| self.legend_cost(n));
        match self.portal_out(c).filter(|&t| self.legend_cost(t).is_some()) {
            Some(t) => e.with(t, PORTAL_COST),
            None => e,
        }
    }

    pub(super) fn legend_prev(&self, costs: &[u8], c: usize) -> Edges {
        let open = |n: usize| Some(costs[n]).filter(|&k| k != BLOCKED).map(u16::from);
        let e = grid_edges(self.grid.w, self.grid.h, c, open);
        match self.portal_back(c).filter(|&s| open(s).is_some()) {
            Some(s) => e.with(s, PORTAL_COST),
            None => e,
        }
    }

    pub(super) fn sound_edges(&self, c: usize) -> Edges {
        let e = grid_edges(self.grid.w, self.grid.h, c, |n| self.sound_cost(n));
        [self.portal_out(c), self.portal_back(c)].into_iter().flatten().fold(e, |e, t| e.with(t, PORTAL_COST))
    }
}
