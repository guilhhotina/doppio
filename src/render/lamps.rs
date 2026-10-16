use super::mat::{self, V3};
use super::zones::{self, Zones};
use crate::map::{LightMode, Map, Sky, Tile};

pub const WARM: V3 = [1.45, 1.12, 0.78];
pub const FLAME: V3 = [1.0, 0.62, 0.28];
pub const MOON_DIR: V3 = [0.42, 0.78, 0.46];
const SPAN: f32 = 9.0;
const SPOT_HEIGHT: f32 = 0.25;
const ABAJUR_HEIGHT: f32 = 1.2;
const ABAJUR_GAIN: f32 = 2.4;
const ABAJUR_FALLOFF: f32 = 0.9;
const CEILING_GAIN: f32 = 4.0;
const CEILING_FALLOFF: f32 = 0.2;
const POINT_GAIN: f32 = 3.2;
const POINT_FALLOFF: f32 = 0.1;
const CANDLE_REACH: f32 = 0.8;
const WINDOW_Y: f32 = 1.5;
const WINDOW_GAIN: f32 = 0.55;
const WINDOW_FALLOFF: f32 = 0.3;
const CANDLES: [&str; 7] = ["vela_castical", "vela_chao", "fogao_lenha", "forno_farinha", "oratorio", "altar", "capela_altar"];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Source {
    Electric,
    Flame,
}

#[derive(Clone, Copy, Debug)]
pub struct Lamp {
    pub room: u8,
    pub pos: V3,
    pub rgb: V3,
    pub falloff: f32,
    pub radius: f32,
    pub source: Source,
    pub mode: LightMode,
    pub open: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Sample {
    pub electric: V3,
    pub flame: f32,
    pub moon: f32,
}

impl Sample {
    pub fn lit(&self) -> [u8; 4] {
        let q = |v: f32| (v * 127.5).round().clamp(0.0, 255.0) as u8;
        [q(self.electric[0]), q(self.electric[1]), q(self.electric[2]), q(self.flame)]
    }

    pub fn colour(&self, power: f32, moon: V3) -> V3 {
        let e = mat::scale(self.electric, power);
        mat::add(mat::add(e, mat::scale(FLAME, self.flame)), mat::scale(moon, self.moon))
    }
}

pub struct Lamps {
    pub lamps: Vec<Lamp>,
    pub moon_level: Vec<f32>,
    windows: Vec<(u8, V3, V3)>,
    solid: Vec<bool>,
    w: i32,
}

pub fn moon_strength(sky: Sky) -> f32 {
    match sky {
        Sky::Moon => 1.0,
        Sky::Night => 0.7,
        Sky::Overcast => 0.8,
        Sky::None => 0.45,
    }
}

fn spots(map: &Map, room: usize) -> Vec<V3> {
    let cells = (0..map.h).flat_map(|y| (0..map.w).map(move |x| (x, y))).filter(|&(x, y)| map.room_of[(y * map.w + x) as usize] == Some(room as u8));
    let (lo, hi) = cells.fold(([i32::MAX; 2], [i32::MIN; 2]), |(lo, hi), (x, y)| ([lo[0].min(x), lo[1].min(y)], [hi[0].max(x), hi[1].max(y)]));
    if lo[0] > hi[0] {
        return Vec::new();
    }
    let (w, h) = ((hi[0] - lo[0] + 1) as f32, (hi[1] - lo[1] + 1) as f32);
    let (nx, ny) = ((w / SPAN).ceil().max(1.0) as usize, (h / SPAN).ceil().max(1.0) as usize);
    let y = map.rooms[room].h.to_f32() - SPOT_HEIGHT;
    (0..nx * ny)
        .map(|i| {
            let (ix, iy) = ((i % nx) as f32, (i / nx) as f32);
            [lo[0] as f32 + w * (ix + 0.5) / nx as f32, y, lo[1] as f32 + h * (iy + 0.5) / ny as f32]
        })
        .collect()
}

impl Lamps {
    pub fn new(map: &Map, zones: &Zones) -> Lamps {
        let strength = moon_strength(map.sky);
        let ceiling = map.rooms.iter().enumerate().filter(|(_, r)| !r.out && r.light > 0).flat_map(|(i, r)| {
            let gain = CEILING_GAIN * r.light as f32 / 100.0;
            spots(map, i).into_iter().map(move |pos| Lamp {
                room: i as u8,
                pos,
                rgb: mat::scale(WARM, gain),
                falloff: CEILING_FALLOFF,
                radius: 0.0,
                source: Source::Electric,
                mode: LightMode::Steady,
                open: false,
            })
        });
        let abajurs = map.props.iter().filter(|p| p.kind == "abajur").filter_map(|p| {
            let pos = [p.pos.x.to_f32(), ABAJUR_HEIGHT, p.pos.y.to_f32()];
            let room = zones.at(pos).filter(|&r| !zones.rooms[r as usize].out)?;
            Some(Lamp { room, pos, rgb: mat::scale(WARM, ABAJUR_GAIN), falloff: ABAJUR_FALLOFF, radius: 0.0, source: Source::Electric, mode: LightMode::Steady, open: false })
        });
        let points = map.lights.iter().filter_map(|l| {
            let pos = [l.pos.x.to_f32(), l.h.to_f32(), l.pos.y.to_f32()];
            let room = zones.at(pos)?;
            let candle = map.props.iter().filter(|p| CANDLES.contains(&p.kind)).any(|p| {
                let (dx, dz) = (p.pos.x.to_f32() - pos[0], p.pos.y.to_f32() - pos[2]);
                dx * dx + dz * dz <= CANDLE_REACH * CANDLE_REACH
            });
            let rgb = l.rgb.map(|c| c as f32 / 255.0 * POINT_GAIN);
            let source = if candle { Source::Flame } else { Source::Electric };
            Some(Lamp { room, pos, rgb, falloff: POINT_FALLOFF, radius: l.radius.to_f32().max(0.5), source, mode: l.mode, open: true })
        });
        let lamps = ceiling.chain(abajurs).chain(points).collect();
        let moon_level = map.rooms.iter().map(|r| if r.out { r.light as f32 / 100.0 * strength } else { 0.0 }).collect();
        let windows = (0..map.h)
            .flat_map(|y| (0..map.w).map(move |x| (x, y)))
            .filter(|&(x, y)| map.tile(x, y) == Tile::Window)
            .flat_map(|(x, y)| zones::DIRS.iter().filter_map(move |d| zones.room(x + d[0], y + d[1]).filter(|&r| !map.rooms[r as usize].out).map(|r| (r, x, y, *d))))
            .map(|(r, x, y, d)| (r, [x as f32 + 0.5 + d[0] as f32 * 0.5, WINDOW_Y, y as f32 + 0.5 + d[1] as f32 * 0.5], [d[0] as f32, -0.35, d[1] as f32]))
            .collect();
        let solid = map.tiles.iter().map(|&t| zones::solid(t) || t == Tile::Door).collect();
        Lamps { lamps, moon_level, windows: if strength > 0.5 { windows } else { Vec::new() }, solid, w: map.w }
    }

    fn window_moon(&self, room: u8, p: V3, n: Option<V3>) -> f32 {
        self.windows
            .iter()
            .filter(|w| w.0 == room)
            .map(|&(_, at, out)| {
                let d = mat::sub(p, at);
                let d2 = mat::dot(d, d);
                let ahead = (mat::dot(d, out) / d2.sqrt().max(1e-3)).max(0.0);
                let face = n.map_or(0.75, |n| (-mat::dot(n, mat::norm(d))).max(0.0) * 0.8 + 0.2);
                WINDOW_GAIN * ahead * face / (1.0 + WINDOW_FALLOFF * d2)
            })
            .sum()
    }

    fn blocked(&self, x: i32, y: i32) -> bool {
        x < 0 || y < 0 || x >= self.w || (y * self.w + x) as usize >= self.solid.len() || self.solid[(y * self.w + x) as usize]
    }

    pub fn clear(&self, a: V3, b: V3) -> bool {
        let (d, n) = ([b[0] - a[0], b[2] - a[2]], ((b[0] - a[0]).abs().max((b[2] - a[2]).abs()) * 4.0).ceil().max(1.0) as i32);
        let (ta, tb) = ((a[0].floor() as i32, a[2].floor() as i32), (b[0].floor() as i32, b[2].floor() as i32));
        (1..n).map(|i| i as f32 / n as f32).map(|t| ((a[0] + d[0] * t).floor() as i32, (a[2] + d[1] * t).floor() as i32)).filter(|&c| c != ta && c != tb).all(|(x, y)| !self.blocked(x, y))
    }

    fn reach(&self, l: &Lamp, room: u8, p: V3, n: Option<V3>) -> f32 {
        let d = mat::sub(l.pos, p);
        let d2 = mat::dot(d, d);
        let wrap = n.map_or(0.75, |n| mat::dot(n, mat::scale(d, 1.0 / d2.sqrt().max(1e-3))).max(0.0) * 0.85 + 0.15);
        let under = n.map_or(1.0, |n| if n[1] < -0.5 && l.radius == 0.0 { 0.35 } else { 1.0 });
        match l.open {
            false => (l.room == room) as u8 as f32 * under * wrap / (1.0 + l.falloff * d2),
            true => {
                let r2 = l.radius * l.radius + l.pos[1] * l.pos[1];
                if d2 >= r2 {
                    return 0.0;
                }
                let window = (1.0 - d2 / r2).powi(2);
                let probe = n.map_or(p, |n| mat::add(p, mat::scale(n, 0.08)));
                let seen = l.room == room || self.clear(l.pos, probe);
                seen as u8 as f32 * wrap * window / (1.0 + l.falloff * d2)
            }
        }
    }

    pub fn sample(&self, room: u8, p: V3, n: Option<V3>) -> Sample {
        let sky = self.moon_level.get(room as usize).copied().unwrap_or(0.0) * 4.5 * n.map_or(0.75, |n| mat::dot(n, MOON_DIR).max(0.0) * 0.85 + 0.15);
        let moon = sky + self.window_moon(room, p, n);
        self.lamps.iter().filter(|l| l.mode == LightMode::Steady).fold(Sample { moon: moon.min(1.0), ..Sample::default() }, |mut s, l| {
            let k = self.reach(l, room, p, n);
            if k > 0.0 {
                match l.source {
                    Source::Electric => s.electric = mat::add(s.electric, mat::scale(l.rgb, k)),
                    Source::Flame => s.flame += k * mat::len(l.rgb) * 0.58,
                }
            }
            s
        })
    }

    pub fn bake(&self, room: u8, p: V3, n: V3) -> (f32, [u8; 4]) {
        let s = self.sample(room, p, Some(n));
        (s.moon, Sample { electric: s.electric.map(|c| c.min(2.0)), flame: s.flame.min(2.0), moon: s.moon }.lit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    fn street() -> map::Map<'static> {
        map::parse(include_str!("testdata/rua.map")).expect("rua.map")
    }

    #[test]
    fn street_lamp_lights_the_street_but_not_inside_the_house() {
        let m = street();
        let z = Zones::new(&m);
        let l = Lamps::new(&m, &z);
        let up = [0.0, 1.0, 0.0];
        let street = z.room(3, 11).expect("street");
        let near = l.sample(street, [3.5, 0.0, 11.5], Some(up));
        let far = l.sample(street, [30.5, 0.0, 14.5], Some(up));
        assert!(near.electric[0] > far.electric[0] + 0.3, "{near:?} {far:?}");
        assert!(near.moon > 0.0);
        let house = z.room(3, 7).expect("house");
        let inside = Lamps { lamps: l.lamps.iter().copied().filter(|x| x.open && x.pos[1] > 4.0).collect(), ..Lamps::new(&m, &z) };
        assert_eq!(inside.sample(house, [3.5, 0.0, 7.5], Some(up)).electric, [0.0; 3]);
    }

    #[test]
    fn candle_is_a_flame_and_dead_lamps_bake_nothing() {
        let m = street();
        let z = Zones::new(&m);
        let l = Lamps::new(&m, &z);
        assert!(l.lamps.iter().any(|x| x.source == Source::Flame));
        let dead = l.lamps.iter().find(|x| x.mode == LightMode::Dead).expect("dead lamp");
        let room = dead.room;
        let only_dead = Lamps { lamps: vec![*dead], ..Lamps::new(&m, &z) };
        assert_eq!(only_dead.sample(room, [dead.pos[0], 0.0, dead.pos[2] + 1.0], None).electric, [0.0; 3]);
    }

    #[test]
    fn line_of_sight_stops_at_walls() {
        let m = street();
        let z = Zones::new(&m);
        let l = Lamps::new(&m, &z);
        assert!(l.clear([3.5, 2.0, 10.5], [10.5, 0.0, 12.5]));
        assert!(!l.clear([3.5, 2.0, 10.5], [3.5, 0.0, 5.5]));
    }

    #[test]
    fn sobrado_rooms_are_lit() {
        let m = map::campaign()[0];
        let z = Zones::new(m);
        let l = Lamps::new(m, &z);
        m.rooms.iter().enumerate().filter(|(_, r)| !r.out && r.light > 0).for_each(|(i, _)| assert!(l.lamps.iter().any(|x| x.room == i as u8)));
    }
}
