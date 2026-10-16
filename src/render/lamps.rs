use super::mat::{self, V3};
use crate::map::{Map, Tile};

pub const LAMP_HEIGHT: f32 = 2.35;
const SPAN: f32 = 9.0;
const MOON: V3 = [0.42, 0.78, 0.46];

#[derive(Clone, Copy, Debug)]
pub struct RoomBox {
    pub lo: [i32; 2],
    pub hi: [i32; 2],
    pub outdoor: bool,
    pub level: f32,
}

pub struct Lamps {
    pub rooms: Vec<RoomBox>,
    lamps: Vec<(u8, V3)>,
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

impl Lamps {
    pub fn new(map: &Map) -> Lamps {
        let rooms = room_boxes(map);
        let lamps = rooms
            .iter()
            .enumerate()
            .filter(|(_, b)| !b.outdoor)
            .flat_map(|(i, b)| spots(b).into_iter().map(move |p| (i as u8, p)))
            .collect();
        Lamps { rooms, lamps }
    }

    pub fn lamps(&self, room: u8) -> impl Iterator<Item = V3> + '_ {
        self.lamps.iter().filter(move |(r, _)| *r == room).map(|(_, p)| *p)
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
                    let d = mat::sub(l, p);
                    let d2 = mat::dot(d, d);
                    b.level * 4.0 * shade * wrap(mat::scale(d, 1.0 / d2.sqrt().max(1e-3))) / (1.0 + 0.2 * d2)
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
    fn doors_find_a_room() {
        let m = map::sobrado();
        m.doors.iter().for_each(|d| assert!(room_near(m, d.x, d.y).is_some()));
    }
}
