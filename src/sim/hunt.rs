use super::tape::{Seg, SEG};
use super::{Body, TaskKind, World, MAX_BODIES};
use crate::fx::Vec2;
use crate::map::Tile;

pub const FIELD_EVERY: u32 = 15;
const CANDIDATES: u32 = 24;
const CHASE_CANDIDATES: u32 = 48;
const SPRINT: u8 = (SEG / 3) as u8;
const BLOCKED_PENALTY: u32 = 4;
const UNREACHED: u16 = u16::MAX;

impl World {
    fn path_open(&self, i: usize) -> bool {
        let (x, y) = (i as i32 % self.map.w, i as i32 / self.map.w);
        match self.map.tile(x, y) {
            Tile::Floor => true,
            Tile::Door => self.doors.get(self.door_at[i] as usize).is_none_or(|d| !d.locked),
            _ => false,
        }
    }

    fn flood(&mut self, sources: &[u16]) {
        let mut field = std::mem::take(&mut self.field);
        let mut queue = std::mem::take(&mut self.queue);
        field.fill(UNREACHED);
        let w = self.map.w as usize;
        let cells = field.len();
        let mut tail = sources.iter().map(|&s| s as usize).fold(0, |tail, s| match field[s] == UNREACHED && self.path_open(s) {
            true => {
                field[s] = 0;
                queue[tail] = s as u16;
                tail + 1
            }
            false => tail,
        });
        let mut head = 0;
        while head < tail {
            let c = queue[head] as usize;
            head += 1;
            let next = field[c] + 1;
            let x = c % w;
            [(x > 0).then(|| c - 1), (x + 1 < w).then(|| c + 1), c.checked_sub(w), Some(c + w).filter(|&n| n < cells)]
                .into_iter()
                .flatten()
                .for_each(|n| {
                    if field[n] == UNREACHED && self.path_open(n) {
                        field[n] = next;
                        queue[tail] = n as u16;
                        tail += 1;
                    }
                });
        }
        self.field = field;
        self.queue = queue;
    }

    pub(super) fn reached(&self, p: Vec2) -> bool {
        self.tile_index(p).is_some_and(|t| self.field[t] != UNREACHED)
    }

    pub(super) fn connected(&mut self, night: u8) -> bool {
        let Some(origin) = self.map.spawns.first().and_then(|s| self.tile_index(s.pos)) else { return true };
        self.flood(&[origin as u16]);
        let spawns = self.map.spawns.iter().all(|s| self.reached(s.pos));
        let breakers = self.tasks.iter().filter(|t| t.kind == TaskKind::Breaker).map(|t| self.field[t.y as usize * self.map.w as usize + t.x as usize]);
        let breaker = breakers.clone().next().is_none() || breakers.into_iter().any(|d| d != UNREACHED);
        let finale = night < 5 || self.final_tiles.is_empty() || self.final_tiles.iter().any(|&t| self.field[t as usize] != UNREACHED);
        spawns && breaker && finale
    }

    pub(super) fn chase_field(&mut self) {
        let mut sources = [0u16; MAX_BODIES];
        let n = self
            .living_bodies()
            .filter_map(|b| self.tile_index(b.pos))
            .fold(0, |n, t| {
                sources[n] = t as u16;
                n + 1
            });
        self.flood(&sources[..n]);
    }

    pub(super) fn distance_at(&self, p: Vec2) -> u16 {
        self.tile_index(p).map_or(UNREACHED, |t| self.field[t])
    }

    fn score(&self, b: &Body, here: u16, seg: Seg, chasing: bool) -> u32 {
        let end = b.pos + seg.d.rotate(b.heading);
        let steps = match self.distance_at(end) {
            UNREACHED => here as u32 + BLOCKED_PENALTY,
            d => d as u32,
        };
        let near = self.living_bodies().map(|l| (l.pos - end).len().0 >> 12).min().unwrap_or(0).clamp(0, 1023);
        let sprint = match chasing {
            true => ((steps >= here as u32) as u32) << 31 | ((seg.run < SPRINT) as u32) << 30,
            false => 0,
        };
        sprint | (steps << 10) | near as u32
    }

    pub(super) fn pick_segment(&mut self, i: usize) -> u32 {
        let b = self.bodies[i];
        let here = self.distance_at(b.pos);
        let base = self.echo[i].tape as usize * self.nseg as usize;
        let nseg = self.nseg;
        let chasing = self.echo[i].chase > 0;
        let tries = if chasing { CHASE_CANDIDATES } else { CANDIDATES };
        (0..tries)
            .fold((u32::MAX, 0), |best, _| {
                let s = self.rng.below(nseg);
                let score = self.score(&b, here, self.segs[base + s as usize], chasing);
                match score < best.0 {
                    true => (score, s),
                    false => best,
                }
            })
            .1
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::tiny_world;
    use super::*;
    use crate::fx::Fx;

    #[test]
    fn flood_counts_grid_steps_and_respects_locks() {
        let mut w = tiny_world(1, 30);
        let start = w.tile_index(w.map.spawns[0].pos).unwrap() as u16;
        w.flood(&[start]);
        let at = |w: &World, x: i32, y: i32| w.field[(y * w.map.w + x) as usize];
        assert_eq!(at(&w, 1, 1), 0);
        assert_eq!(at(&w, 2, 1), 1);
        assert_eq!(at(&w, 0, 0), UNREACHED);
        let far = 7;
        assert!(at(&w, far, 1) != UNREACHED);
        w.doors[0].locked = true;
        w.flood(&[start]);
        assert_eq!(at(&w, far, 1), UNREACHED);
        assert!(w.distance_at(Vec2::new(Fx::int(-3), Fx::int(1))) == UNREACHED);
    }
}
