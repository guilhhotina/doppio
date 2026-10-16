use super::mat::V3;
use crate::map::{Map, Tile};
use std::collections::{BTreeMap, VecDeque};

pub type Mask = u128;
pub const ROOMS: usize = 128;
pub const DIRS: [[i32; 2]; 4] = [[0, -1], [0, 1], [1, 0], [-1, 0]];

#[derive(Clone, Copy, Debug)]
pub struct RoomInfo {
    pub area: u8,
    pub out: bool,
    pub h: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Portal {
    pub rooms: Mask,
    pub door: Option<u16>,
    pub lo: V3,
    pub hi: V3,
}

pub struct Zones {
    pub w: i32,
    pub h: i32,
    pub owner: Vec<Option<u8>>,
    pub rooms: Vec<RoomInfo>,
    pub portals: Vec<Portal>,
    pub links: Vec<Vec<u16>>,
}

pub fn see_through(t: Tile) -> bool {
    matches!(t, Tile::Floor | Tile::Blocker | Tile::Hide | Tile::Portal | Tile::Water | Tile::Fence)
}

pub fn solid(t: Tile) -> bool {
    matches!(t, Tile::Wall | Tile::Window)
}

fn owners(map: &Map) -> Vec<Option<u8>> {
    let mut owner: Vec<Option<u8>> = (0..map.w * map.h).map(|i| map.room_of[i as usize].filter(|_| !solid(map.tiles[i as usize]))).collect();
    let mut queue: VecDeque<(i32, i32)> = (0..map.h).flat_map(|y| (0..map.w).map(move |x| (x, y))).filter(|&(x, y)| owner[(y * map.w + x) as usize].is_some()).collect();
    while let Some((x, y)) = queue.pop_front() {
        let r = owner[(y * map.w + x) as usize];
        DIRS.iter().map(|d| (x + d[0], y + d[1])).filter(|&(nx, ny)| see_through(map.tile(nx, ny)) || map.tile(nx, ny) == Tile::Door).for_each(|(nx, ny)| {
            let at = (ny * map.w + nx) as usize;
            if owner[at].is_none() {
                owner[at] = r;
                queue.push_back((nx, ny));
            }
        });
    }
    owner
}

impl Zones {
    pub fn new(map: &Map) -> Zones {
        assert!(map.rooms.len() <= ROOMS, "{} rooms (max {ROOMS})", map.rooms.len());
        let owner = owners(map);
        let rooms: Vec<RoomInfo> = map.rooms.iter().map(|r| RoomInfo { area: r.area, out: r.out, h: r.h.to_f32() }).collect();
        let at = |x: i32, y: i32| ((0..map.w).contains(&x) && (0..map.h).contains(&y)).then(|| owner[(y * map.w + x) as usize]).flatten();
        let doors = map.doors.iter().enumerate().map(|(i, d)| {
            let rooms = DIRS.iter().filter_map(|o| at(d.x + o[0], d.y + o[1])).chain(at(d.x, d.y)).fold(0 as Mask, |m, r| m | 1 << r);
            let (fx, fy) = (d.x as f32, d.y as f32);
            Portal { rooms, door: Some(i as u16), lo: [fx, 0.0, fy], hi: [fx + 1.0, 2.1, fy + 1.0] }
        });
        let mut open: BTreeMap<(u8, u8), (V3, V3)> = BTreeMap::new();
        (0..map.h).flat_map(|y| (0..map.w).map(move |x| (x, y))).filter(|&(x, y)| see_through(map.tile(x, y))).for_each(|(x, y)| {
            [[1, 0], [0, 1]].iter().filter(|d| see_through(map.tile(x + d[0], y + d[1]))).for_each(|d| {
                let (Some(a), Some(b)) = (at(x, y), at(x + d[0], y + d[1])) else { return };
                if a == b || rooms[a as usize].area != rooms[b as usize].area {
                    return;
                }
                let top = rooms[a as usize].h.max(rooms[b as usize].h);
                let (lo, hi) = match d[0] {
                    1 => ([(x + 1) as f32, 0.0, y as f32], [(x + 1) as f32, top, (y + 1) as f32]),
                    _ => ([x as f32, 0.0, (y + 1) as f32], [(x + 1) as f32, top, (y + 1) as f32]),
                };
                let e = open.entry((a.min(b), a.max(b))).or_insert((lo, hi));
                *e = (std::array::from_fn(|k| e.0[k].min(lo[k])), std::array::from_fn(|k| e.1[k].max(hi[k])));
            });
        });
        let portals: Vec<Portal> = doors.chain(open.into_iter().map(|((a, b), (lo, hi))| Portal { rooms: 1 << a | 1 << b, door: None, lo, hi })).collect();
        let links = (0..rooms.len()).map(|r| (0..portals.len() as u16).filter(|&p| portals[p as usize].rooms >> r & 1 != 0).collect()).collect();
        Zones { w: map.w, h: map.h, owner, rooms, portals, links }
    }

    pub fn room(&self, x: i32, y: i32) -> Option<u8> {
        ((0..self.w).contains(&x) && (0..self.h).contains(&y)).then(|| self.owner[(y * self.w + x) as usize]).flatten()
    }

    pub fn near(&self, x: i32, y: i32) -> Option<u8> {
        self.room(x, y).or_else(|| DIRS.iter().find_map(|d| self.room(x + d[0], y + d[1])))
    }

    pub fn at(&self, p: V3) -> Option<u8> {
        self.near(p[0].floor() as i32, p[2].floor() as i32)
    }

    pub fn area_mask(&self, area: u8) -> Mask {
        self.rooms.iter().enumerate().filter(|(_, r)| r.area == area).fold(0, |m, (i, _)| m | 1 << i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    pub const STREET: &str = include_str!("testdata/rua.map");

    #[test]
    fn every_open_tile_has_an_owner() {
        let m = map::parse(STREET).expect("rua.map");
        let z = Zones::new(&m);
        (0..m.h).flat_map(|y| (0..m.w).map(move |x| (x, y))).filter(|&(x, y)| see_through(m.tile(x, y)) || m.tile(x, y) == Tile::Door).for_each(|(x, y)| {
            assert!(z.room(x, y).is_some(), "{x},{y} {:?}", m.tile(x, y));
        });
    }

    #[test]
    fn adjacent_rooms_get_an_open_portal_and_doors_link_both_sides() {
        let m = map::parse(STREET).expect("rua.map");
        let z = Zones::new(&m);
        assert!(z.portals.iter().any(|p| p.door.is_none()));
        z.portals.iter().filter(|p| p.door.is_some()).for_each(|p| assert!(p.rooms.count_ones() >= 2, "{p:?}"));
        z.links.iter().enumerate().for_each(|(r, l)| l.iter().for_each(|&p| assert!(z.portals[p as usize].rooms >> r & 1 != 0)));
    }

    #[test]
    fn sobrado_zones() {
        let m = map::campaign()[0];
        let z = Zones::new(m);
        assert!(z.portals.len() >= m.doors.len());
        m.doors.iter().for_each(|d| assert!(z.near(d.x, d.y).is_some()));
    }
}
