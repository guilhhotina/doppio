use super::grid::NONE;
use super::{Focus, ItemKind, ItemPlace, LegendKind, LegendMode, PlayerState, World};
use crate::fx::{Fx, Vec2};
use crate::map::LockKind;

pub const REACH: Fx = Fx::ratio(7, 10);
const ITEM_REACH: i64 = sq(Fx::ratio(6, 10));
const FRIEND_REACH: i64 = sq(Fx::ONE);
const PROP_REACH: i64 = sq(Fx::ratio(7, 10));
const BALDE_REACH: i64 = sq(Fx::int(2));
const LAIR_REACH: i64 = sq(Fx::ratio(15, 10));

const fn sq(f: Fx) -> i64 {
    f.0 as i64 * f.0 as i64
}

pub fn facing(heading: Vec2, d: Vec2) -> bool {
    heading.dot(d).0 as i64 * 2 >= d.len().0 as i64
}

impl World {
    fn probe(&self, p: usize) -> Vec2 {
        let pl = self.players[p];
        pl.pos + Vec2::from_ang(pl.heading).scale(REACH)
    }

    fn cells_near(&self, p: usize) -> [Option<usize>; 2] {
        [self.grid.cell_of(self.probe(p)), self.grid.cell_of(self.players[p].pos)]
    }

    pub(super) fn door_ahead(&self, p: usize) -> Option<usize> {
        self.cells_near(p).into_iter().flatten().find_map(|c| self.door_of(c))
    }

    pub(super) fn focus_of(&self, p: usize) -> Option<Focus> {
        match self.players[p].state {
            PlayerState::Hidden { spot } => return Some(Focus::Hide { spot }),
            PlayerState::Free if self.ctl[p].hide_t == 0 => {}
            _ => return None,
        }
        self.friend_near(p)
            .or_else(|| self.balde_use(p))
            .or_else(|| self.item_near(p))
            .or_else(|| self.portal_near(p))
            .or_else(|| self.salt_use(p))
            .or_else(|| self.door_ahead(p).map(|d| self.door_focus(d)))
            .or_else(|| self.place_near(p))
            .or_else(|| self.hide_near(p))
            .or_else(|| self.container_near(p))
            .or_else(|| self.note_near(p))
            .or_else(|| self.exit_lock_near(p))
            .or_else(|| self.tabaco_use(p))
    }

    fn lock_focus(&self, lock: Option<u8>) -> Option<Focus> {
        let l = lock.filter(|_| !self.lock_open(lock))?;
        let needs = match self.map.locks[l as usize].kind {
            LockKind::Key(i) => Some(self.items.get(i as usize).map_or(ItemKind::Chave, |s| s.kind)),
            LockKind::Tool(k) => Some(k),
            LockKind::Code { .. } => None,
            _ => return None,
        };
        Some(Focus::Lock { lock: l, needs, code: needs.is_none() })
    }

    fn door_focus(&self, d: usize) -> Focus {
        let door = self.doors[d];
        let lock = match self.map.doors[d].state {
            crate::map::DoorState::Locked(l) if door.locked => Some(l),
            _ => None,
        };
        self.lock_focus(lock).unwrap_or(Focus::Door { door: d as u16, locked: door.locked, stuck: door.stuck, open: self.door_dir[d] })
    }

    fn friend_near(&self, p: usize) -> Option<Focus> {
        let at = self.players[p].pos;
        (0..self.count)
            .filter(|&q| q != p && matches!(self.players[q].state, PlayerState::Fainted { .. }))
            .map(|q| (q, (self.players[q].pos - at).len_sq()))
            .filter(|&(_, d)| d <= FRIEND_REACH)
            .min_by_key(|&(q, d)| (d, q))
            .map(|(q, _)| Focus::Friend { player: q as u8 })
    }

    fn balde_use(&self, p: usize) -> Option<Focus> {
        let pl = self.players[p];
        let lg = self.legend;
        let d = lg.pos - pl.pos;
        let able = lg.kind == LegendKind::Cabra
            && !matches!(lg.mode, LegendMode::Stunned | LegendMode::Leaving)
            && lg.area == pl.area
            && d.len_sq() <= BALDE_REACH
            && facing(Vec2::from_ang(pl.heading), d);
        able.then(|| self.holds(p, ItemKind::Balde)).flatten().map(|item| Focus::Use { item })
    }

    fn item_near(&self, p: usize) -> Option<Focus> {
        let (probe, pl) = (self.probe(p), self.players[p]);
        self.items
            .iter()
            .enumerate()
            .filter_map(|(i, s)| match s.place {
                ItemPlace::Floor { pos, area } if area == pl.area => Some((i, (pos - probe).len_sq().min((pos - pl.pos).len_sq()))),
                _ => None,
            })
            .filter(|&(_, d)| d <= ITEM_REACH)
            .min_by_key(|&(i, d)| (d, i))
            .map(|(i, _)| Focus::Item { item: i as u8 })
    }

    fn portal_near(&self, p: usize) -> Option<Focus> {
        let [ahead, here] = self.cells_near(p);
        [here, ahead]
            .into_iter()
            .flatten()
            .map(|c| self.grid.portal[c])
            .find(|&k| k != NONE)
            .map(|k| {
                let locked = self.portal_locked(k as usize);
                let lock = self.map.portals[k as usize].lock;
                match locked {
                    true => self.lock_focus(lock).unwrap_or(Focus::Portal { portal: k, locked }),
                    false => Focus::Portal { portal: k, locked },
                }
            })
    }

    fn salt_use(&self, p: usize) -> Option<Focus> {
        let item = self.holds(p, ItemKind::Sal).filter(|_| self.legend.kind == LegendKind::CorpoSeco)?;
        self.door_ahead(p)
            .filter(|&d| self.door_dir[d] && !self.doors[d].locked && self.salt[d] == 0)
            .map(|_| Focus::Use { item })
    }

    fn place_near(&self, p: usize) -> Option<Focus> {
        let cells = self.cells_near(p);
        self.map.locks.iter().enumerate().find_map(|(l, lock)| match lock.kind {
            LockKind::Place { item, x, y } if !self.locks[l].open && cells.contains(&self.grid.cell(x, y)) => {
                let needs = self.items.get(item as usize).map_or(ItemKind::Fusivel, |s| s.kind);
                Some(Focus::Place { lock: l as u8, needs })
            }
            _ => None,
        })
    }

    fn hide_near(&self, p: usize) -> Option<Focus> {
        let ahead = self.grid.cell_of(self.probe(p))?;
        let spot = Some(self.grid.hide[ahead]).filter(|&s| s != NONE)?;
        let taken = (0..self.count).any(|q| match self.players[q].state {
            PlayerState::Hidden { spot: s } => s == spot,
            PlayerState::Free => self.ctl[q].hide_t > 0 && self.ctl[q].spot == spot,
            _ => false,
        });
        (!taken).then_some(Focus::Hide { spot })
    }

    fn container_near(&self, p: usize) -> Option<Focus> {
        let (probe, ahead) = (self.probe(p), self.grid.cell_of(self.probe(p)));
        self.map
            .props
            .iter()
            .enumerate()
            .filter(|&(k, prop)| prop.item.is_some() && self.props[k].open == 0)
            .map(|(k, prop)| (k, prop, (prop.pos - probe).len_sq()))
            .filter(|&(_, prop, d)| d <= PROP_REACH || self.grid.cell_of(prop.pos) == ahead)
            .min_by_key(|&(k, _, d)| (d, k))
            .map(|(k, prop, _)| {
                let locked = !self.lock_open(prop.lock);
                match locked {
                    true => self.lock_focus(prop.lock).unwrap_or(Focus::Container { prop: k as u16, locked }),
                    false => Focus::Container { prop: k as u16, locked },
                }
            })
    }

    fn note_near(&self, p: usize) -> Option<Focus> {
        let cells = self.cells_near(p);
        self.map
            .notes
            .iter()
            .position(|n| cells.contains(&self.grid.cell(n.x, n.y)))
            .map(|n| Focus::Note { note: n as u16 })
    }

    fn attached(&self, lock: u8) -> bool {
        self.map.doors.iter().any(|d| d.state == crate::map::DoorState::Locked(lock))
            || self.map.portals.iter().any(|k| k.lock == Some(lock))
            || self.map.props.iter().any(|k| k.lock == Some(lock))
    }

    fn loose_part(&self, lock: u8, depth: u8, pick: &impl Fn(u8) -> bool) -> Option<u8> {
        match &self.map.locks.get(lock as usize)?.kind {
            LockKind::All(parts) if depth < 8 => parts.iter().find_map(|&q| self.loose_part(q, depth + 1, pick)),
            LockKind::Key(_) | LockKind::Tool(_) | LockKind::Code { .. } => {
                (!self.locks[lock as usize].open && !self.attached(lock) && pick(lock)).then_some(lock)
            }
            _ => None,
        }
    }

    fn exit_lock_near(&self, p: usize) -> Option<Focus> {
        let exit = self.map.exit;
        let (x, y) = (self.players[p].pos.x.floor(), self.players[p].pos.y.floor());
        let r = exit.rect;
        let near = (r.x0 - 1..=r.x1 + 1).contains(&x) && (r.y0 - 1..=r.y1 + 1).contains(&y);
        let hands = self.players[p].hands;
        let fitting = |l: u8| hands.into_iter().flatten().any(|i| self.fits(l, i));
        let part = near.then(|| self.loose_part(exit.lock, 0, &fitting).or_else(|| self.loose_part(exit.lock, 0, &|_| true)));
        part.flatten().and_then(|l| self.lock_focus(Some(l)))
    }

    fn tabaco_use(&self, p: usize) -> Option<Focus> {
        let (lx, ly) = self.map.lair;
        let close = (super::centre(lx, ly) - self.players[p].pos).len_sq() <= LAIR_REACH;
        let able = close && self.legend.kind == LegendKind::Matinta;
        able.then(|| self.holds(p, ItemKind::Tabaco)).flatten().map(|item| Focus::Use { item })
    }
}
