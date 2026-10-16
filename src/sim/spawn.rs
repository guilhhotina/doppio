use super::echo::Echo;
use super::scare::stage_tics;
use super::tape::tape_id;
use super::{shuffle, Agent, Body, Ctl, Role, World, HIDDEN, MAX_BODIES, MAX_SPAWNS};
use crate::fx::{Ang, Fx, Vec2};

type Spot = (Vec2, Ang);

impl World {
    pub(super) fn place(&mut self, i: usize, (pos, heading): Spot, agent: Agent) {
        self.bodies[i] = Body { pos, heading, visible: true, ..HIDDEN };
        self.ctl[i] = Ctl::fresh(agent);
        self.echo[i] = Echo::NONE;
    }

    fn spawn_spot(&self, night: u8, k: usize) -> Spot {
        let row = &self.spawn_of[night as usize - 1];
        let n = self.map.spawns.len().min(MAX_SPAWNS);
        let s = &self.map.spawns[row[k % n] as usize];
        (s.pos, s.rot)
    }

    fn final_spot(&mut self) -> Spot {
        match self.final_tiles.len() as u32 {
            0 => self.spawn_spot(5, 0),
            n => {
                let t = self.final_tiles[self.rng.below(n) as usize] as i32;
                let w = self.map.w;
                let pos = Vec2::new(Fx::int(t % w) + Fx::HALF, Fx::int(t / w) + Fx::HALF);
                (pos, Ang(self.rng.next_u32() as u16))
            }
        }
    }

    pub(super) fn assign_spawns(&mut self, night: u8) {
        let n = self.map.spawns.len().min(MAX_SPAWNS);
        let row = &mut self.spawn_of[night as usize - 1];
        row.iter_mut().enumerate().for_each(|(i, s)| *s = i as u8);
        shuffle(&mut self.rng, &mut row[..n]);
    }

    fn echo_spot(&mut self, night: u8, source: u8, k: usize) -> Spot {
        match night {
            5 => self.final_spot(),
            _ => self.spawn_spot(source, k),
        }
    }

    pub(super) fn populate(&mut self, night: u8) {
        self.bodies = [HIDDEN; MAX_BODIES];
        self.ctl = [Ctl::FREE; MAX_BODIES];
        self.echo = [Echo::NONE; MAX_BODIES];
        let n = self.players_n as usize;
        (0..n).for_each(|p| {
            let spot = match (self.players[p].role, night) {
                (Role::Duplo, 5) => self.final_spot(),
                _ => self.spawn_spot(night, p),
            };
            self.place(p, spot, Agent::Player(p as u8));
            self.players[p].body = p as u8;
        });
        let echoes = (1..night).flat_map(|k| (0..n).map(move |p| (k, p, false)));
        let spare = (self.map.spawns.len().min(MAX_SPAWNS) - n).max(1);
        let extras = (0..self.extra as usize).map(|e| (0, n + e % spare, true));
        echoes.chain(extras).zip(n..MAX_BODIES).for_each(|((k, p, extra), slot)| {
            let (k, p, spot_k) = match extra {
                false => (k, p, p),
                true => (1 + self.rng.below(night as u32 - 1) as u8, self.rng.below(n as u32) as usize, p),
            };
            let spot = match extra {
                false => self.echo_spot(night, k, spot_k),
                true => self.echo_spot(night, night, spot_k),
            };
            self.place(slot, spot, Agent::Echo);
            let wait = stage_tics(night);
            self.echo[slot] = Echo { wait, ..Echo::new(tape_id(k, p), night, extra) };
            self.bodies[slot].visible = wait == 0;
        });
    }
}
