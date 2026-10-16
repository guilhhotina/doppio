use crate::fx::{Ang, Fx, Vec2};
use crate::map::{Axis, DoorState, Map};
use crate::render::{CANVAS_H, CANVAS_W, Cues, Scene, View};
use crate::sim::{Anim, Body, Door, MAX_BODIES};
use std::f32::consts::TAU;

const SHOTS: [(i32, i32); 4] = [(4, 7), (12, 6), (8, 11), (19, 7)];
const STEPS: u32 = 7;

pub struct Backdrop {
    map: &'static Map<'static>,
    bodies: [Body; MAX_BODIES],
    doors: Vec<Door>,
    shot: usize,
    step: u32,
    dark: bool,
}

fn noise(t: u32) -> u32 {
    let x = t.wrapping_mul(0x2c1b_3c6d) ^ t >> 5;
    x ^ x >> 11
}

pub fn fx(v: f32) -> Fx {
    Fx((v * 65536.0) as i32)
}

pub fn ang(rad: f32) -> Ang {
    Ang((rad.rem_euclid(TAU) / TAU * 65536.0) as u16)
}

pub fn doors(map: &Map) -> Vec<Door> {
    map.doors
        .iter()
        .map(|d| Door {
            x: d.x as u8,
            y: d.y as u8,
            vertical: d.axis == Axis::V,
            open: if d.state == DoorState::Open { 52000 } else { 0 },
            locked: d.state == DoorState::Locked,
        })
        .collect()
}

pub fn figure(map: &Map, shot: (i32, i32), step: u32) -> Option<([f32; 2], f32)> {
    let cam = map.cam_at(shot.0, shot.1)?;
    let (e, t) = ([cam.eye[0].to_f32(), cam.eye[2].to_f32()], [cam.target[0].to_f32(), cam.target[2].to_f32()]);
    let k = 0.1 + 0.55 * step.min(STEPS) as f32 / STEPS as f32;
    let p = [t[0] + (e[0] - t[0]) * k, t[1] + (e[1] - t[1]) * k];
    Some((p, (e[0] - p[0]).atan2(e[1] - p[1])))
}

impl Backdrop {
    pub fn new(map: &'static Map<'static>) -> Backdrop {
        let hidden = Body { pos: Vec2::ZERO, heading: Ang::ZERO, anim: Anim::Idle, anim_tic: 0, lantern: false, visible: false };
        let mut b = Backdrop { map, bodies: [hidden; MAX_BODIES], doors: doors(map), shot: 0, step: 0, dark: false };
        b.place();
        b
    }

    fn place(&mut self) {
        let (x, y) = SHOTS[self.shot];
        self.bodies[0].pos = Vec2::new(fx(x as f32 + 0.5), fx(y as f32 + 0.5));
        let fig = figure(self.map, (x, y), self.step);
        self.bodies[1].visible = fig.is_some();
        fig.iter().for_each(|&([px, py], h)| {
            self.bodies[1].pos = Vec2::new(fx(px), fx(py));
            self.bodies[1].heading = ang(h);
        });
    }

    pub fn tick(&mut self, frame: u32) {
        let n = noise(frame / 4);
        let dark = n % 61 < 2 || frame % 600 < 6;
        if dark && !self.dark {
            self.step += 1;
            if self.step > STEPS {
                self.step = 0;
                self.shot = (self.shot + 1) % SHOTS.len();
            }
            self.place();
        }
        self.dark = dark;
        self.bodies[1].anim_tic = self.bodies[1].anim_tic.wrapping_add(frame as u16 & 1);
    }

    pub fn view(&self, dim: u8) -> View {
        View { rect: [0, 0, CANVAS_W, CANVAS_H], follow: 0, fade: if self.dark { 255 } else { dim }, duplo: false }
    }

    pub fn scene(&self) -> Scene<'_> {
        Scene { bodies: &self.bodies, doors: &self.doors, power: true, cues: &Cues::NONE }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    #[test]
    fn figures_stand_on_floor_and_approach() {
        let m = map::sobrado();
        SHOTS.iter().for_each(|&s| {
            let (far, _) = figure(m, s, 0).expect("shot has a camera");
            let (near, _) = figure(m, s, STEPS).expect("shot has a camera");
            let cam = m.cam_at(s.0, s.1).unwrap();
            let eye = [cam.eye[0].to_f32(), cam.eye[2].to_f32()];
            let d = |p: [f32; 2]| ((p[0] - eye[0]).powi(2) + (p[1] - eye[1]).powi(2)).sqrt();
            assert!(d(near) < d(far));
            assert!(m.tile(near[0] as i32, near[1] as i32).walkable() || m.tile(far[0] as i32, far[1] as i32).walkable());
        });
        let mut b = Backdrop::new(m);
        (0..2000).for_each(|f| b.tick(f));
        assert!(b.scene().bodies.iter().filter(|b| b.visible).count() == 1);
    }
}
