use super::clock::Clock;
use crate::map::Map;
use crate::render::{CANVAS_H, CANVAS_W, View};
use crate::sim::{MAX_PLAYERS, Outcome, Setup, TICS_PER_SEC, World};
use std::time::Instant;

pub const SHOT_TICS: u32 = 40 * TICS_PER_SEC;

pub struct Backdrop {
    maps: &'static [&'static Map<'static>],
    world: Box<World>,
    clock: Clock,
    shot: u32,
}

fn shot_setup(shot: u32, unlocked: usize, maps: usize) -> Setup {
    let ch = (shot % MAX_PLAYERS as u32) as u8;
    Setup {
        seed: 0xD0_BB10 ^ shot,
        phase: (shot as usize % unlocked.clamp(1, maps.max(1))) as u8,
        players: 1,
        characters: std::array::from_fn(|i| (ch + i as u8) % MAX_PLAYERS as u8),
    }
}

impl Backdrop {
    pub fn new(maps: &'static [&'static Map<'static>], now: Instant) -> Backdrop {
        let setup = shot_setup(0, 1, maps.len());
        Backdrop { maps, world: Box::new(World::new(maps[0], setup)), clock: Clock::new(now), shot: 0 }
    }

    pub fn tick(&mut self, now: Instant, unlocked: usize) {
        let world = &mut self.world;
        self.clock.run(now, || {
            world.step(&[0; MAX_PLAYERS]);
            true
        });
        if self.world.tic() >= SHOT_TICS || self.world.outcome() != Outcome::Playing {
            self.shot = self.shot.wrapping_add(1);
            let setup = shot_setup(self.shot, unlocked, self.maps.len());
            self.world = Box::new(World::new(self.maps[setup.phase as usize], setup));
        }
    }

    pub fn view(&self, dim: u8) -> View {
        let fade = self.world.players().first().map_or(0, |p| p.fade);
        View { rect: [0, 0, CANVAS_W, CANVAS_H], follow: 0, fade: fade.max(dim), portrait: None }
    }

    pub fn world(&self) -> &World {
        &self.world
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::sample_maps;
    use crate::app::clock::TIC;

    #[test]
    fn shots_rotate_through_unlocked_phases_and_characters() {
        assert_eq!(shot_setup(0, 1, 5).phase, 0);
        assert_eq!(shot_setup(3, 2, 5).phase, 1);
        assert_eq!(shot_setup(3, 9, 2).phase, 1);
        assert_eq!(shot_setup(5, 0, 5).phase, 0);
        assert_eq!(shot_setup(6, 1, 1).characters, [2, 3, 0, 1]);
        let t0 = Instant::now();
        let mut b = Backdrop::new(sample_maps(), t0);
        (1..=SHOT_TICS / 5 + 2).for_each(|k| b.tick(t0 + TIC * (5 * k), 1));
        assert_eq!(b.shot, 1);
        assert!(b.world().tic() < SHOT_TICS);
        assert_eq!(b.world().players()[0].character, 1);
    }
}
