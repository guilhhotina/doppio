use crate::fx::{Ang, Fx, Vec2};
use crate::map::{Axis, DoorState, Map};
use crate::render::Scene;
use crate::sim::{Anim, Body, Door, MAX_BODIES};

pub struct Bench {
    bodies: [Body; MAX_BODIES],
    doors: Vec<Door>,
    power: bool,
    origin: [f32; 2],
    tic: u32,
}

fn fx(v: f32) -> Fx {
    Fx((v * 65536.0) as i32)
}

fn ang(rad: f32) -> Ang {
    Ang((rad.rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU * 65536.0) as u16)
}

pub struct Spec {
    pub bodies: usize,
    pub at: [f32; 3],
    pub power: bool,
    pub lantern: bool,
    pub anim: Anim,
}

impl Bench {
    pub fn new(map: &Map, s: &Spec) -> Bench {
        let mut bodies = [Body { pos: Vec2::ZERO, heading: Ang::ZERO, anim: Anim::Idle, anim_tic: 0, lantern: false, visible: false }; MAX_BODIES];
        bodies.iter_mut().take(s.bodies.max(1)).enumerate().for_each(|(i, b)| {
            b.visible = true;
            b.lantern = s.lantern && i % 3 == 0;
            b.anim = if i == 0 { s.anim } else { [Anim::Walk, Anim::Idle, Anim::Run, Anim::Turn, Anim::Reach, Anim::Stagger][i % 6] };
            b.anim_tic = (i * 7) as u16;
        });
        bodies[0].pos = Vec2::new(fx(s.at[0]), fx(s.at[1]));
        bodies[0].heading = ang(s.at[2].to_radians());
        let doors = map
            .doors
            .iter()
            .map(|d| Door {
                x: d.x as u8,
                y: d.y as u8,
                vertical: d.axis == Axis::V,
                open: if d.state == DoorState::Open { 52000 } else { 0 },
                locked: d.state == DoorState::Locked,
            })
            .collect();
        let mut b = Bench { bodies, doors, power: s.power, origin: [s.at[0], s.at[1]], tic: 0 };
        b.place();
        b
    }

    fn place(&mut self) {
        let t = self.tic as f32 / 30.0;
        let n = self.bodies.iter().filter(|b| b.visible).count().max(2) - 1;
        let [ox, oy] = self.origin;
        self.bodies.iter_mut().skip(1).filter(|b| b.visible).enumerate().for_each(|(i, b)| {
            let r = 1.2 + (i % 3) as f32 * 0.7;
            let a = i as f32 / n as f32 * std::f32::consts::TAU + t * 0.25 / r;
            b.pos = Vec2::new(fx(ox + a.sin() * r), fx(oy + a.cos() * r));
            b.heading = ang(a + std::f32::consts::FRAC_PI_2);
        });
    }

    pub fn step(&mut self) {
        self.tic += 1;
        self.bodies.iter_mut().filter(|b| b.visible).for_each(|b| b.anim_tic = b.anim_tic.wrapping_add(1));
        self.place();
    }

    pub fn scene(&self) -> Scene<'_> {
        Scene { bodies: &self.bodies, doors: &self.doors, power: self.power }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    #[test]
    fn bench_places_requested_bodies() {
        let spec = Spec { bodies: 20, at: [4.5, 6.5, 90.0], power: false, lantern: true, anim: Anim::Idle };
        let mut b = Bench::new(map::sobrado(), &spec);
        (0..40).for_each(|_| b.step());
        let s = b.scene();
        assert_eq!(s.bodies.iter().filter(|b| b.visible).count(), 20);
        assert_eq!(s.doors.len(), map::sobrado().doors.len());
        assert!(!s.power);
        assert_eq!(s.bodies[0].pos, Vec2::new(fx(4.5), fx(6.5)));
    }
}
