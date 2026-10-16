use crate::fx::{Fx, Vec2};
use crate::map::Map;

pub const NONE: u8 = u8::MAX;

pub struct Grid {
    pub w: i32,
    pub h: i32,
    pub door: Vec<u8>,
    pub portal: Vec<u8>,
    pub portal_in: Vec<u8>,
    pub hide: Vec<u8>,
    pub area: Vec<u8>,
    pub room: Vec<u8>,
    pub surface: Vec<u8>,
}

fn surface_of(floor: &str) -> u8 {
    crate::assets::TEXTURES.binary_search_by(|(name, _)| (*name).cmp(floor)).map_or(0, |i| i as u8)
}

fn table(cells: usize, entries: impl Iterator<Item = (i32, i32)>, w: i32, h: i32) -> Vec<u8> {
    let mut t = vec![NONE; cells];
    entries
        .enumerate()
        .filter(|&(i, (x, y))| i < NONE as usize && (0..w).contains(&x) && (0..h).contains(&y))
        .for_each(|(i, (x, y))| t[(y * w + x) as usize] = i as u8);
    t
}

impl Grid {
    pub fn new(map: &Map) -> Grid {
        let (w, h) = (map.w, map.h);
        let cells = (w * h) as usize;
        assert!(cells <= 1 << 16 && map.tiles.len() == cells);
        let room = (0..cells).map(|c| map.room_of[c].unwrap_or(NONE)).collect::<Vec<u8>>();
        Grid {
            w,
            h,
            door: table(cells, map.doors.iter().take(super::MAX_DOORS).map(|d| (d.x, d.y)), w, h),
            portal: table(cells, map.portals.iter().map(|p| (p.x, p.y)), w, h),
            portal_in: table(cells, map.portals.iter().map(|p| (p.to_x, p.to_y)), w, h),
            hide: table(cells, map.hides.iter().map(|s| (s.x, s.y)), w, h),
            area: room.iter().map(|&r| map.rooms.get(r as usize).map_or(NONE, |r| r.area)).collect(),
            surface: map.rooms.iter().map(|r| surface_of(r.floor)).collect(),
            room,
        }
    }

    pub fn cells(&self) -> usize {
        (self.w * self.h) as usize
    }

    pub fn cell(&self, x: i32, y: i32) -> Option<usize> {
        ((0..self.w).contains(&x) && (0..self.h).contains(&y)).then(|| (y * self.w + x) as usize)
    }

    pub fn cell_of(&self, p: Vec2) -> Option<usize> {
        self.cell(p.x.floor(), p.y.floor())
    }

    pub fn xy(&self, c: usize) -> (i32, i32) {
        (c as i32 % self.w, c as i32 / self.w)
    }

    pub fn centre(&self, c: usize) -> Vec2 {
        let (x, y) = self.xy(c);
        Vec2::new(Fx::int(x) + Fx::HALF, Fx::int(y) + Fx::HALF)
    }

    pub fn area_of(&self, p: Vec2) -> u8 {
        self.cell_of(p).map_or(NONE, |c| self.area[c])
    }

    pub fn surface_at(&self, p: Vec2) -> u8 {
        self.cell_of(p).and_then(|c| self.surface.get(self.room[c] as usize)).copied().unwrap_or(0)
    }
}
