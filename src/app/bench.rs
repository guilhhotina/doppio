use super::backdrop::{ang, doors, fx};
use crate::fx::{Ang, Vec2};
use crate::map::Map;
use crate::render::{Cues, Echo, MAX_TELLS, Scene, Tell};
use crate::sim::{Anim, Body, Door, MAX_BODIES};

pub struct Bench {
    bodies: [Body; MAX_BODIES],
    doors: Vec<Door>,
    power: bool,
    origin: [f32; 2],
    tic: u32,
    cues: Cues,
}

pub struct Spec {
    pub bodies: usize,
    pub at: [f32; 3],
    pub power: bool,
    pub lantern: bool,
    pub anim: Anim,
    pub tells: Tell,
    pub gaze: Option<f32>,
    pub caught: bool,
    pub echo: Option<[f32; 3]>,
}

fn cues(map: &Map, s: &Spec) -> Cues {
    let mut c = Cues::NONE;
    c.tells.iter_mut().take(map.tasks.len().min(MAX_TELLS)).for_each(|t| *t = s.tells);
    c.gaze.iter_mut().take(s.bodies).skip(1).for_each(|g| *g = s.gaze.map(|d| ang(d.to_radians())));
    c.caught[0] = (s.caught && s.bodies > 1).then_some((1, 0));
    c.echo[0] = s.echo.map(|e| Echo { pos: Vec2::new(fx(e[0]), fx(e[1])), heading: ang(e[2].to_radians()), anim: Anim::Idle, tic: 0 });
    c
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
        let mut b = Bench { bodies, doors: doors(map), power: s.power, origin: [s.at[0], s.at[1]], tic: 0, cues: cues(map, s) };
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
        self.cues.echo[0].iter_mut().for_each(|e| e.tic = e.tic.wrapping_add(1));
        self.place();
    }

    pub fn scene(&self) -> Scene<'_> {
        Scene { bodies: &self.bodies, doors: &self.doors, power: self.power, cues: &self.cues }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    #[test]
    fn bench_places_requested_bodies() {
        let spec = Spec { bodies: 20, at: [4.5, 6.5, 90.0], power: false, lantern: true, anim: Anim::Idle, tells: Tell::Pending, gaze: Some(90.0), caught: true, echo: None };
        let mut b = Bench::new(map::sobrado(), &spec);
        (0..40).for_each(|_| b.step());
        let s = b.scene();
        assert_eq!(s.bodies.iter().filter(|b| b.visible).count(), 20);
        assert_eq!(s.doors.len(), map::sobrado().doors.len());
        assert!(!s.power);
        assert_eq!(s.bodies[0].pos, Vec2::new(fx(4.5), fx(6.5)));
        assert_eq!(s.cues.tells.iter().filter(|t| **t == Tell::Pending).count(), map::sobrado().tasks.len());
        assert!(s.cues.gaze[0].is_none() && s.cues.gaze[19].is_some() && s.cues.gaze[20].is_none());
        assert_eq!(s.cues.caught[0], Some((1, 0)));
    }
}
