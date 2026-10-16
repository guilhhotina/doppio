use super::voice::Place;
use crate::fx::Vec2;
use crate::map::{Map, Tile};
use crate::sim::{PlayerState, World};

const MAX_CELLS: usize = 96;
const FLOORS_APART: f32 = 2.2;

#[derive(Clone, Copy, Debug)]
pub struct Ear {
    pub x: f32,
    pub y: f32,
    pub rx: f32,
    pub ry: f32,
    pub area: u8,
    pub room: Option<u8>,
    pub out: bool,
    pub hidden: bool,
}

pub fn point(v: Vec2) -> (f32, f32) {
    (v.x.to_f32(), v.y.to_f32())
}

pub fn room_near(map: &Map<'_>, x: f32, y: f32) -> Option<u8> {
    [(0.0, 0.0), (0.6, 0.0), (-0.6, 0.0), (0.0, 0.6), (0.0, -0.6)]
        .iter()
        .find_map(|(dx, dy)| map.room_index((x + dx).floor() as i32, (y + dy).floor() as i32))
}

pub fn area_near(map: &Map<'_>, x: f32, y: f32) -> Option<u8> {
    room_near(map, x, y).and_then(|r| map.rooms.get(r as usize)).map(|r| r.area)
}

fn cam_right(map: &Map<'_>, x: f32, y: f32) -> (f32, f32) {
    map.cam_at(x.floor() as i32, y.floor() as i32)
        .map(|c| ((c.target[0] - c.eye[0]).to_f32(), (c.target[2] - c.eye[2]).to_f32()))
        .map(|(fx, fz)| (fx, fz, (fx * fx + fz * fz).sqrt()))
        .filter(|(_, _, l)| *l > 1e-3)
        .map_or((1.0, 0.0), |(fx, fz, l)| (-fz / l, fx / l))
}

fn stereo(pan: f32, gain: f32, muffle: f32) -> Place {
    Place { left: ((1.0 - pan) * 0.5).sqrt() * gain, right: ((1.0 + pan) * 0.5).sqrt() * gain, muffle }
}

impl Ear {
    pub fn new(world: &World, listener: u8) -> Option<Ear> {
        let map = world.map();
        world.players().get(listener as usize).map(|p| {
            let (x, y) = point(p.pos);
            let (rx, ry) = cam_right(map, x, y);
            let room = room_near(map, x, y);
            let out = room.and_then(|r| map.rooms.get(r as usize)).is_some_and(|r| r.out);
            Ear { x, y, rx, ry, area: p.area, room, out, hidden: matches!(p.state, PlayerState::Hidden { .. }) }
        })
    }

    pub fn distance(&self, x: f32, y: f32) -> f32 {
        ((x - self.x) * (x - self.x) + (y - self.y) * (y - self.y)).sqrt()
    }

    fn cover(&self, p: Place) -> Place {
        match self.hidden {
            true => Place { left: p.left * 0.6, right: p.right * 0.6, muffle: p.muffle.max(0.8) },
            false => p,
        }
    }

    fn pan(&self, x: f32, y: f32) -> f32 {
        let d = self.distance(x, y);
        match d > 1e-3 {
            true => ((x - self.x) * self.rx + (y - self.y) * self.ry) / d * d.min(1.0) * 0.8,
            false => 0.0,
        }
    }

    fn occluded(&self, world: &World, x: f32, y: f32, area: u8) -> f32 {
        let map = world.map();
        let walls = occlusion(world, (self.x, self.y), (x, y));
        let apart = matches!((self.room, room_near(map, x, y)), (Some(a), Some(b)) if a != b);
        let floors = if area != self.area { FLOORS_APART } else { 0.0 };
        (if apart { walls.max(0.5) } else { walls }) + floors
    }

    pub fn place(&self, world: &World, x: f32, y: f32, area: u8) -> Place {
        self.reach(world, x, y, area, 0.0)
    }

    pub fn reach(&self, world: &World, x: f32, y: f32, area: u8, floor: f32) -> Place {
        let d = self.distance(x, y);
        let occ = self.occluded(world, x, y, area);
        let gain = (1.0 / (1.0 + 0.3 * d) * 0.5f32.powf(occ)).max(floor);
        let muffle = (occ * 0.9 + (d - 4.0).max(0.0) * 0.03).min(if floor > 0.0 { 0.7 } else { 1.0 });
        self.cover(stereo(self.pan(x, y), gain, muffle))
    }

    pub fn far(&self, pan: f32, level: f32) -> Place {
        let (gain, muffle) = if self.out { (level, 0.15) } else { (level * 0.45, 0.85) };
        self.cover(stereo(pan.clamp(-1.0, 1.0), gain, muffle))
    }

    pub fn own(&self, level: f32) -> Place {
        stereo(0.0, level, 0.0)
    }

    pub fn around(&self, level: f32) -> Place {
        self.cover(stereo(0.0, level, 0.0))
    }
}

#[derive(Clone, Copy)]
struct Ray {
    cx: i32,
    cy: i32,
    sx: i32,
    sy: i32,
    tx: f32,
    ty: f32,
    dx: f32,
    dy: f32,
    t: f32,
}

impl Ray {
    fn new((x0, y0): (f32, f32), (x1, y1): (f32, f32)) -> Ray {
        let axis = |p: f32, d: f32| match d {
            d if d > 0.0 => (1, (p.floor() + 1.0 - p) / d, 1.0 / d),
            d if d < 0.0 => (-1, (p - p.floor()) / -d, -1.0 / d),
            _ => (0, f32::INFINITY, f32::INFINITY),
        };
        let (sx, tx, dx) = axis(x0, x1 - x0);
        let (sy, ty, dy) = axis(y0, y1 - y0);
        Ray { cx: x0.floor() as i32, cy: y0.floor() as i32, sx, sy, tx, ty, dx, dy, t: 0.0 }
    }

    fn step(self) -> Ray {
        match self.tx < self.ty {
            true => Ray { cx: self.cx + self.sx, tx: self.tx + self.dx, t: self.tx, ..self },
            false => Ray { cy: self.cy + self.sy, ty: self.ty + self.dy, t: self.ty, ..self },
        }
    }
}

fn weight(world: &World, x: i32, y: i32) -> f32 {
    let map = world.map();
    match map.tile(x, y) {
        Tile::Wall => 1.0,
        Tile::Window => 0.5,
        Tile::Hide => 0.3,
        Tile::Fence => 0.1,
        Tile::Door => map
            .door_at(x, y)
            .and_then(|i| world.doors().get(i))
            .map_or(0.0, |d| 0.8 * (1.0 - d.open as f32 / 65535.0)),
        Tile::Void | Tile::Floor | Tile::Blocker | Tile::Portal | Tile::Water => 0.0,
    }
}

pub fn occlusion(world: &World, from: (f32, f32), to: (f32, f32)) -> f32 {
    let end = (to.0.floor() as i32, to.1.floor() as i32);
    std::iter::successors(Some(Ray::new(from, to)), |r| Some(r.step()))
        .skip(1)
        .take(MAX_CELLS)
        .take_while(|r| r.t < 1.0 && (r.cx, r.cy) != end)
        .map(|r| weight(world, r.cx, r.cy))
        .sum::<f32>()
        .min(3.0)
}
