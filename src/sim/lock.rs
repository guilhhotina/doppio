use super::{Event, ItemKind, World};
use crate::map::{DoorState, LockKind};

impl World {
    pub(super) fn open_lock(&mut self, p: usize, l: u8) {
        self.unlock(l);
        self.cascade();
        let pl = self.players[p];
        self.noise(pl.pos, pl.area, self.unlock_noise());
    }

    pub(super) fn cascade(&mut self) {
        while let Some(l) = self.ready_all() {
            self.unlock(l);
        }
    }

    fn ready_all(&self) -> Option<u8> {
        self.map.locks.iter().enumerate().find_map(|(l, lock)| match &lock.kind {
            LockKind::All(parts) if !self.locks[l].open && parts.iter().all(|&q| self.lock_open(Some(q))) => Some(l as u8),
            _ => None,
        })
    }

    fn unlock(&mut self, l: u8) {
        if self.locks[l as usize].open {
            return;
        }
        self.locks[l as usize].open = true;
        self.brain.dirty = true;
        self.events.push(Event::Unlock { lock: l });
        let doors = self.map.doors.iter().take(self.doors.len()).enumerate();
        doors.filter(|(_, d)| d.state == DoorState::Locked(l)).for_each(|(d, _)| {
            self.doors[d].locked = false;
            self.move_door(d, true);
        });
        let fuse = match self.map.locks[l as usize].kind {
            LockKind::Place { item, .. } => self.items.get(item as usize).is_some_and(|s| s.kind == ItemKind::Fusivel),
            _ => false,
        };
        if fuse && !self.power {
            self.power = true;
            self.events.push(Event::PowerOn);
        }
    }
}
