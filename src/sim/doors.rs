use super::{collide, shuffle, Event, World, NO_DOOR};
use crate::fx::Vec2;
use crate::map::DoorState;

const SWING: u16 = 4369;
const HALF_OPEN: u16 = 1 << 15;
const MAX_DOORS: usize = 64;
const ATTEMPTS: u32 = 8;

impl World {
    pub(super) fn door_passable(&self, d: usize) -> bool {
        self.doors.get(d).is_none_or(|door| self.door_dir[d] && door.open >= HALF_OPEN)
    }

    pub(super) fn door_on(&self, p: Vec2) -> Option<usize> {
        self.tile_index(p).map(|t| self.door_at[t]).filter(|&d| d != NO_DOOR).map(|d| d as usize)
    }

    fn occupied(&self, d: usize) -> bool {
        let (x, y) = (self.doors[d].x as i32, self.doors[d].y as i32);
        let tile = move |tx: i32, ty: i32| (tx, ty) == (x, y);
        self.bodies.iter().any(|b| b.visible && collide::overlaps(b.pos, &tile))
    }

    pub(super) fn use_door(&mut self, d: usize) {
        let door = d as u8;
        match (self.doors[d].locked, self.door_dir[d]) {
            (true, _) => self.events.push(Event::DoorRattle { door }),
            (false, true) if !self.occupied(d) => {
                self.door_dir[d] = false;
                self.events.push(Event::DoorMove { door, opening: false });
            }
            (false, true) => {}
            (false, false) => {
                self.door_dir[d] = true;
                self.events.push(Event::DoorMove { door, opening: true });
            }
        }
    }

    pub(super) fn swing_doors(&mut self) {
        self.doors.iter_mut().zip(&self.door_dir).for_each(|(door, &opening)| {
            door.open = match opening {
                true => door.open.saturating_add(SWING),
                false => door.open.saturating_sub(SWING),
            };
        });
    }

    pub(super) fn open_door(&mut self, d: usize) {
        if let Some(door) = self.doors.get_mut(d) {
            door.locked = false;
            self.door_dir[d] = true;
            self.events.push(Event::DoorMove { door: d as u8, opening: true });
        }
    }

    fn restore_doors(&mut self) {
        let map = self.map;
        self.doors.iter_mut().zip(self.door_dir.iter_mut()).zip(&map.doors).for_each(|((door, dir), m)| {
            *dir = m.state == DoorState::Open;
            door.open = if *dir { u16::MAX } else { 0 };
            door.locked = m.state == DoorState::Locked;
        });
    }

    pub(super) fn relock(&mut self, night: u8) {
        self.restore_doors();
        let mut pool = [0u8; MAX_DOORS];
        let exit = self.exit_door as usize;
        let n = (0..self.doors.len().min(MAX_DOORS))
            .filter(|&d| d != exit && self.map.doors[d].state == DoorState::Open)
            .fold(0, |n, d| {
                pool[n] = d as u8;
                n + 1
            });
        let count = (night as usize).saturating_sub(1).min(n);
        let settled = (0..ATTEMPTS).any(|_| {
            self.restore_doors();
            shuffle(&mut self.rng, &mut pool[..n]);
            pool[..count].iter().for_each(|&d| {
                let d = d as usize;
                self.doors[d].locked = true;
                self.doors[d].open = 0;
                self.door_dir[d] = false;
            });
            self.connected(night)
        });
        if !settled {
            self.restore_doors();
            self.connected(night);
        }
    }
}
