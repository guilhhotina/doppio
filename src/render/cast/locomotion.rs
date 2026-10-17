use super::super::mat::{self, M4, V3};
use super::skeleton::{Frame, at, lift};
use super::{ARM_L, ARM_R, FOOT_L, FOOT_R, FORE_L, FORE_R, LAMP, PELVIS, SEGS, SHIN_L, SHIN_R, SKIRT, THIGH_L, THIGH_R};
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, Default)]
pub struct Travel {
    previous: Option<(u32, u8, V3)>,
    velocity: [f32; 2],
}

impl Travel {
    pub fn sample(&mut self, tic: u32, area: u8, position: V3) -> [f32; 2] {
        if let Some((before, zone, p)) = self.previous {
            if tic == before {
                return self.velocity;
            }
            let dt = tic.wrapping_sub(before) as f32 / 30.0;
            let distance = [position[0] - p[0], position[1] - p[1]];
            let angle = (position[2] - p[2] + PI).rem_euclid(TAU) - PI;
            self.velocity = if zone == area && dt <= 0.25 && distance[0].hypot(distance[1]) <= 4.0 * dt {
                [(distance[0] * position[2].sin() + distance[1] * position[2].cos()) / dt, angle / dt]
            } else {
                [0.0; 2]
            };
        }
        self.previous = Some((tic, area, position));
        self.velocity
    }
}

pub fn footfall(t: f32, period: f32, duty: f32, speed: f32, clearance: f32) -> [f32; 2] {
    let phase = (t / period).rem_euclid(1.0);
    let stride = speed * period / 30.0 * duty;
    if phase < duty {
        [0.0, stride * (0.5 - phase / duty)]
    } else {
        let u = (phase - duty) / (1.0 - duty);
        let smooth = u * u * (3.0 - 2.0 * u);
        [clearance * (PI * u).sin().powi(2), stride * (smooth - 0.5)]
    }
}

pub fn unpoint(m: &M4, p: V3) -> V3 {
    let d = mat::sub(p, [m[12], m[13], m[14]]);
    [mat::dot(d, [m[0], m[1], m[2]]), mat::dot(d, [m[4], m[5], m[6]]), mat::dot(d, [m[8], m[9], m[10]])]
}

fn bone(a: V3, b: V3, forward: V3) -> M4 {
    let y = mat::norm(mat::sub(a, b));
    let x = mat::norm(mat::cross(y, forward));
    let z = mat::cross(x, y);
    [x[0], x[1], x[2], 0.0, y[0], y[1], y[2], 0.0, z[0], z[1], z[2], 0.0, a[0], a[1], a[2], 1.0]
}

pub fn chain(origin: V3, target: V3, upper: f32, lower: f32, forward: V3, bend: f32) -> [M4; 2] {
    let delta = mat::sub(target, origin);
    let distance = mat::len(delta).clamp((upper - lower).abs() + 0.001, upper + lower - 0.001);
    let direction = mat::norm(delta);
    let across = mat::norm(mat::sub(forward, mat::scale(direction, mat::dot(direction, forward))));
    let along = (upper * upper - lower * lower + distance * distance) / (2.0 * distance);
    let height = (upper * upper - along * along).max(0.0).sqrt() * bend;
    let knee = mat::add(origin, mat::add(mat::scale(direction, along), mat::scale(across, height)));
    [bone(origin, knee, forward), bone(knee, target, forward)]
}

#[derive(Clone, Copy)]
pub struct Gait {
    pub period: f32,
    pub duty: f32,
    pub clearance: f32,
    pub sink: f32,
    pub limp: f32,
}

pub fn biped(m: &mut [M4; SEGS], f: &Frame, place: V3, t: f32, gait: Gait, travel: [f32; 2]) {
    let Gait { period, duty, clearance, sink, limp } = gait;
    let [x, z, heading] = place;
    let root = mat::mul(&mat::translate([x, 0.0, z]), &mat::rot_y(heading));
    let reach = (f.thigh + f.shin - 0.015) * sink;
    let forward = mat::dir(&root, [0.0, 0.0, 1.0]);
    let legs = [(THIGH_L, SHIN_L, FOOT_L, 1.0, 0.0), (THIGH_R, SHIN_R, FOOT_R, -1.0, 0.5)].map(|(thigh, shin, foot, side, offset)| {
        let time = t + period * offset;
        let lame = (side < 0.0) as u8 as f32 * limp;
        let step = footfall(time, period, duty, travel[0], clearance * (1.0 - lame));
        let yaw = footfall(time, period, duty, travel[1], 0.0)[1];
        let lateral = side * f.hip_x * yaw.cos();
        let longitudinal = step[1] - side * f.hip_x * yaw.sin();
        let position = mat::point(&root, [lateral, f.sole + f.meta * 0.94 + step[0], longitudinal - f.meta * 0.34]);
        (thigh, shin, foot, yaw - lame * 0.5, [lateral, step[0], longitudinal], position)
    });
    let dy = legs.iter().map(|&(thigh, _, _, _, _, target)| {
        let h = (target[0] - m[thigh][12]).hypot(target[2] - m[thigh][14]);
        target[1] + (reach * reach - h * h).max(reach * reach * 0.5).sqrt() - m[thigh][13]
    }).fold(f32::MAX, f32::min);
    lift(m, dy);
    legs.into_iter().for_each(|(thigh, shin, foot, yaw, [lateral, raise, longitudinal], position)| {
        let origin = [m[thigh][12], m[thigh][13], m[thigh][14]];
        let [upper, lower] = chain(origin, position, f.thigh, f.shin, forward, 1.0);
        m[thigh] = upper;
        m[shin] = lower;
        m[foot] = if f.meta > 0.0 {
            bone(position, mat::point(&root, [lateral, f.sole + raise, longitudinal]), forward)
        } else {
            at(&root, [lateral, f.sole + raise, longitudinal], &mat::rot_y(yaw))
        };
    });
}

pub struct Quadruped {
    pub hind: [f32; 3],
    pub front: [f32; 2],
    pub sole: f32,
    pub height: f32,
    pub spread: [f32; 2],
    pub z: [f32; 2],
}

pub fn quadruped(m: &mut [M4; SEGS], f: &Quadruped, place: V3, t: f32, period: f32, speed: f32, run: bool) {
    let [x, z, heading] = place;
    let root = mat::mul(&mat::translate([x, 0.0, z]), &mat::rot_y(heading));
    let forward = mat::dir(&root, [0.0, 0.0, 1.0]);
    let duty = if run { 0.36 } else { 0.7 };
    let phases = if run { [0.0, 1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0] } else { [0.0, 0.5, 0.25, 0.75] };
    let dy = f.height - m[PELVIS][13];
    lift(m, dy);
    [(THIGH_L, SHIN_L, FOOT_L, 1.0), (THIGH_R, SHIN_R, FOOT_R, -1.0), (ARM_L, FORE_L, LAMP, 1.0), (ARM_R, FORE_R, SKIRT, -1.0)].into_iter().enumerate().for_each(|(i, (upper, lower, foot, side))| {
        let hind = i < 2;
        let step = footfall(t - period * phases[i], period, duty, speed, if run { 0.19 } else { 0.09 });
        let origin = [m[upper][12], m[upper][13], m[upper][14]];
        let offset = f.z[hind as usize];
        let ankle = mat::point(&root, [side * f.spread[hind as usize], f.sole + step[0], offset + step[1]]);
        let end = if hind { mat::add(ankle, mat::dir(&root, [0.0, f.hind[2] * 0.94, -f.hind[2] * 0.34])) } else { ankle };
        let lengths = if hind { [f.hind[0], f.hind[1]] } else { f.front };
        let [a, b] = chain(origin, end, lengths[0], lengths[1], forward, if hind { 1.0 } else { -1.0 });
        m[upper] = a;
        m[lower] = b;
        m[foot] = if hind { bone(end, ankle, forward) } else { at(&root, [side * f.spread[0], f.sole + step[0], offset + step[1]], &mat::IDENTITY) };
    });
}
