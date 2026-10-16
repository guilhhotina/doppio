use super::mat::{self, V3};
use crate::map::{Map, Tile};

pub const LAMP_HEIGHT: f32 = 2.35;
const SPAN: f32 = 9.0;
const MOON: V3 = [0.42, 0.78, 0.46];
const ABAJUR_HEIGHT: f32 = 1.2;
const ABAJUR_GAIN: f32 = 2.4;
const ABAJUR_FALLOFF: f32 = 0.9;
const CEILING_GAIN: f32 = 4.0;
const CEILING_FALLOFF: f32 = 0.2;

#[derive(Clone, Copy, Debug)]
pub struct RoomBox {
    pub lo: [i32; 2],
    pub hi: [i32; 2],
    pub outdoor: bool,
    pub level: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Lamp {
    pub room: u8,
    pub pos: V3,
    pub gain: f32,
    pub falloff: f32,
}

pub struct Lamps {
    pub rooms: Vec<RoomBox>,
    lamps: Vec<Lamp>,
}

pub fn room_near(map: &Map, x: i32, y: i32) -> Option<u8> {
    let at = |x: i32, y: i32| (map.tile(x, y) != Tile::Void).then(|| map.room_at(x, y)).flatten();
    at(x, y)
        .or_else(|| [(0, 1), (0, -1), (1, 0), (-1, 0)].iter().find_map(|(dx, dy)| at(x + dx, y + dy)))
        .and_then(|r| map.rooms.iter().position(|q| q.id == r.id))
        .map(|i| i as u8)
}

fn room_boxes(map: &Map) -> Vec<RoomBox> {
    map.rooms
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let cells = (0..map.h).flat_map(|y| (0..map.w).map(move |x| (x, y)));
            let (lo, hi) = cells
                .filter(|&(x, y)| map.room_of[(y * map.w + x) as usize] == Some(i as u8))
                .fold(([i32::MAX; 2], [i32::MIN; 2]), |(lo, hi), (x, y)| {
                    ([lo[0].min(x), lo[1].min(y)], [hi[0].max(x), hi[1].max(y)])
                });
            RoomBox { lo, hi, outdoor: r.ceil.is_none(), level: r.light as f32 / 100.0 }
        })
        .collect()
}

fn spots(b: &RoomBox) -> Vec<V3> {
    let (w, h) = ((b.hi[0] - b.lo[0] + 1) as f32, (b.hi[1] - b.lo[1] + 1) as f32);
    let (nx, ny) = ((w / SPAN).ceil().max(1.0) as usize, (h / SPAN).ceil().max(1.0) as usize);
    (0..nx * ny)
        .map(|i| {
            let (ix, iy) = ((i % nx) as f32, (i / nx) as f32);
            [b.lo[0] as f32 + w * (ix + 0.5) / nx as f32, LAMP_HEIGHT, b.lo[1] as f32 + h * (iy + 0.5) / ny as f32]
        })
        .collect()
}

fn abajurs<'a>(map: &'a Map, rooms: &'a [RoomBox]) -> impl Iterator<Item = Lamp> + 'a {
    map.props.iter().filter(|p| p.kind == "abajur").filter_map(move |p| {
        let (x, z) = (p.pos.x.to_f32(), p.pos.y.to_f32());
        let room = room_near(map, x.floor() as i32, z.floor() as i32).filter(|&r| !rooms[r as usize].outdoor)?;
        Some(Lamp { room, pos: [x, ABAJUR_HEIGHT, z], gain: ABAJUR_GAIN, falloff: ABAJUR_FALLOFF })
    })
}

impl Lamps {
    pub fn new(map: &Map) -> Lamps {
        let rooms = room_boxes(map);
        let ceiling = rooms.iter().enumerate().filter(|(_, b)| !b.outdoor).flat_map(|(i, b)| {
            spots(b).into_iter().map(move |pos| Lamp { room: i as u8, pos, gain: CEILING_GAIN * b.level, falloff: CEILING_FALLOFF })
        });
        let lamps = ceiling.chain(abajurs(map, &rooms)).collect();
        Lamps { rooms, lamps }
    }

    pub fn lamps(&self, room: u8) -> impl Iterator<Item = &Lamp> + '_ {
        self.lamps.iter().filter(move |l| l.room == room)
    }

    pub fn light(&self, room: u8, p: V3, n: Option<V3>) -> f32 {
        let b = self.rooms[room as usize];
        let shade = n.map_or(1.0, |n| if n[1] < -0.5 { 0.35 } else { 1.0 });
        let wrap = |d: V3| n.map_or(0.75, |n| mat::dot(n, d).max(0.0) * 0.85 + 0.15);
        match b.outdoor {
            true => b.level * 4.5 * wrap(MOON),
            false => self
                .lamps(room)
                .map(|l| {
                    let d = mat::sub(l.pos, p);
                    let d2 = mat::dot(d, d);
                    l.gain * shade * wrap(mat::scale(d, 1.0 / d2.sqrt().max(1e-3))) / (1.0 + l.falloff * d2)
                })
                .sum::<f32>(),
        }
        .min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    #[test]
    fn every_room_has_light_and_bounds() {
        let m = map::sobrado();
        let l = Lamps::new(m);
        assert_eq!(l.rooms.len(), m.rooms.len());
        l.rooms.iter().enumerate().for_each(|(i, b)| {
            assert!(b.lo[0] <= b.hi[0] && b.lo[1] <= b.hi[1]);
            assert!(b.outdoor || l.lamps(i as u8).count() >= 1);
            let c = [(b.lo[0] + b.hi[0]) as f32 * 0.5 + 0.5, 0.0, (b.lo[1] + b.hi[1]) as f32 * 0.5 + 0.5];
            let v = l.light(i as u8, c, Some([0.0, 1.0, 0.0]));
            assert!(v > 0.0 && v <= 1.0);
        });
    }

    #[test]
    fn abajur_brightens_its_corner() {
        let m = map::sobrado();
        let lit = Lamps::new(m);
        let unlit = Lamps { lamps: lit.lamps.iter().copied().filter(|l| l.gain != ABAJUR_GAIN).collect(), rooms: lit.rooms.clone() };
        let abajur = lit.lamps.iter().find(|l| l.gain == ABAJUR_GAIN).expect("map has an abajur");
        let floor = [abajur.pos[0] + 0.6, 0.0, abajur.pos[2]];
        let up = Some([0.0, 1.0, 0.0]);
        assert!(lit.light(abajur.room, floor, up) > unlit.light(abajur.room, floor, up) + 0.1);
    }

    #[test]
    fn doors_find_a_room() {
        let m = map::sobrado();
        m.doors.iter().for_each(|d| assert!(room_near(m, d.x, d.y).is_some()));
    }
}
