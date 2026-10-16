use crate::fx::{Ang, Fx, Vec2};
use std::fmt;
use std::str::SplitWhitespace;
use std::sync::LazyLock;

const SOBRADO: &str = include_str!("../assets/map/sobrado.map");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Void,
    Wall,
    Floor,
    Door,
    Window,
    Blocker,
}

impl Tile {
    fn from_byte(b: u8) -> Option<Tile> {
        match b {
            b' ' => Some(Tile::Void),
            b'#' => Some(Tile::Wall),
            b'.' => Some(Tile::Floor),
            b'D' => Some(Tile::Door),
            b'W' => Some(Tile::Window),
            b'B' => Some(Tile::Blocker),
            _ => None,
        }
    }

    pub fn walkable(self) -> bool {
        matches!(self, Tile::Floor | Tile::Door)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis {
    H,
    V,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DoorState {
    Open,
    Closed,
    Locked,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaskKind {
    Window,
    Breaker,
    Key,
    Tv,
    Gas,
    Faucet,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl Rect {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        (self.x0..=self.x1).contains(&x) && (self.y0..=self.y1).contains(&y)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Room<'a> {
    pub id: u8,
    pub name: &'a str,
    pub floor: &'a str,
    pub wall: &'a str,
    pub ceil: Option<&'a str>,
    pub light: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Door {
    pub x: i32,
    pub y: i32,
    pub axis: Axis,
    pub state: DoorState,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cam {
    pub area: Rect,
    pub eye: [Fx; 3],
    pub target: [Fx; 3],
    pub fov: Fx,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Prop<'a> {
    pub kind: &'a str,
    pub pos: Vec2,
    pub rot: Ang,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Spawn {
    pub pos: Vec2,
    pub rot: Ang,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Task {
    pub kind: TaskKind,
    pub pos: Vec2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Map<'a> {
    pub w: i32,
    pub h: i32,
    pub tiles: Vec<Tile>,
    pub room_of: Vec<Option<u8>>,
    pub rooms: Vec<Room<'a>>,
    pub doors: Vec<Door>,
    pub cams: Vec<Cam>,
    pub props: Vec<Prop<'a>>,
    pub spawns: Vec<Spawn>,
    pub tasks: Vec<Task>,
    pub final_area: Rect,
    pub exit: Vec2,
}

impl<'a> Map<'a> {
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        ((0..self.w).contains(&x) && (0..self.h).contains(&y)).then(|| (y * self.w + x) as usize)
    }

    pub fn tile(&self, x: i32, y: i32) -> Tile {
        self.index(x, y).map_or(Tile::Void, |i| self.tiles[i])
    }

    pub fn room_at(&self, x: i32, y: i32) -> Option<&Room<'a>> {
        self.index(x, y).and_then(|i| self.room_of[i]).map(|r| &self.rooms[r as usize])
    }

    pub fn cam_at(&self, x: i32, y: i32) -> Option<&Cam> {
        self.cams.iter().find(|c| c.area.contains(x, y))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParseError {
    pub line: usize,
    pub msg: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            0 => write!(f, "map: {}", self.msg),
            n => write!(f, "map line {n}: {}", self.msg),
        }
    }
}

fn err<T>(line: usize, msg: impl Into<String>) -> Result<T, ParseError> {
    Err(ParseError { line, msg: msg.into() })
}

static SOBRADO_MAP: LazyLock<Map<'static>> =
    LazyLock::new(|| parse(SOBRADO).unwrap_or_else(|e| panic!("sobrado.map: {e}")));

pub fn sobrado() -> &'static Map<'static> {
    &SOBRADO_MAP
}

pub fn parse(src: &str) -> Result<Map<'_>, ParseError> {
    src.lines().enumerate().try_fold(Builder::default(), |b, (i, line)| b.feed(i + 1, line))?.finish()
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Section {
    #[default]
    None,
    Solid,
    Rooms,
    Room,
    Door,
    Cam,
    Prop,
    Spot,
}

impl Section {
    fn named(name: &str) -> Option<Section> {
        match name {
            "solid" => Some(Section::Solid),
            "rooms" => Some(Section::Rooms),
            "room" => Some(Section::Room),
            "door" => Some(Section::Door),
            "cam" => Some(Section::Cam),
            "prop" => Some(Section::Prop),
            "spot" => Some(Section::Spot),
            _ => None,
        }
    }
}

struct Fields<'a> {
    line: usize,
    it: SplitWhitespace<'a>,
}

impl<'a> Fields<'a> {
    fn word(&mut self, what: &str) -> Result<&'a str, ParseError> {
        self.it.next().map_or_else(|| err(self.line, format!("missing {what}")), Ok)
    }

    fn fx(&mut self, what: &str) -> Result<Fx, ParseError> {
        let w = self.word(what)?;
        Fx::parse(w).map_or_else(|| err(self.line, format!("bad number for {what}: '{w}'")), Ok)
    }

    fn int(&mut self, what: &str) -> Result<i32, ParseError> {
        let w = self.word(what)?;
        w.parse().or_else(|_| err(self.line, format!("bad integer for {what}: '{w}'")))
    }

    fn key(&mut self, key: &str) -> Result<&'a str, ParseError> {
        let w = self.word(key)?;
        w.strip_prefix(key)
            .and_then(|r| r.strip_prefix('='))
            .filter(|v| !v.is_empty())
            .map_or_else(|| err(self.line, format!("expected {key}=<value>, got '{w}'")), Ok)
    }

    fn pos(&mut self) -> Result<Vec2, ParseError> {
        Ok(Vec2::new(self.fx("x")?, self.fx("y")?))
    }

    fn rot(&mut self) -> Result<Ang, ParseError> {
        self.fx("rot_deg").map(Ang::deg)
    }

    fn point3(&mut self, names: [&str; 3]) -> Result<[Fx; 3], ParseError> {
        Ok([self.fx(names[0])?, self.fx(names[1])?, self.fx(names[2])?])
    }

    fn rect(&mut self) -> Result<Rect, ParseError> {
        let r = Rect { x0: self.int("x0")?, y0: self.int("y0")?, x1: self.int("x1")?, y1: self.int("y1")? };
        match r.x0 <= r.x1 && r.y0 <= r.y1 {
            true => Ok(r),
            false => err(self.line, "rect corners out of order"),
        }
    }

    fn pick<T: Copy>(&mut self, what: &str, options: &[(&str, T)]) -> Result<T, ParseError> {
        let w = self.word(what)?;
        options
            .iter()
            .find(|(name, _)| *name == w)
            .map_or_else(|| err(self.line, format!("unknown {what} '{w}'")), |(_, v)| Ok(*v))
    }

    fn end<T>(mut self, value: T) -> Result<T, ParseError> {
        match self.it.next() {
            None => Ok(value),
            Some(extra) => err(self.line, format!("unexpected '{extra}'")),
        }
    }
}

#[derive(Default)]
struct Builder<'a> {
    section: Section,
    solid: Vec<(usize, &'a str)>,
    room_rows: Vec<(usize, &'a str)>,
    rooms: Vec<Room<'a>>,
    doors: Vec<(usize, Door)>,
    cams: Vec<Cam>,
    props: Vec<Prop<'a>>,
    spawns: Vec<Spawn>,
    tasks: Vec<Task>,
    final_area: Option<Rect>,
    exit: Option<Vec2>,
}

fn room<'a>(mut f: Fields<'a>) -> Result<Room<'a>, ParseError> {
    let id = f.word("room id")?;
    let id = match id.as_bytes() {
        [b] if *b != b'-' && b.is_ascii_graphic() => *b,
        _ => return err(f.line, format!("room id must be one character, got '{id}'")),
    };
    let name = f.word("room name")?;
    let floor = f.key("floor")?;
    let wall = f.key("wall")?;
    let ceil = Some(f.key("ceil")?).filter(|c| *c != "-");
    let light = f.key("light")?;
    let light = light.parse::<u8>().ok().filter(|l| *l <= 100);
    let light = light.map_or_else(|| err(f.line, "light must be an integer 0..100"), Ok)?;
    f.end(Room { id, name, floor, wall, ceil, light })
}

fn door(mut f: Fields<'_>) -> Result<Door, ParseError> {
    let (x, y) = (f.int("x")?, f.int("y")?);
    let axis = f.pick("axis", &[("h", Axis::H), ("v", Axis::V)])?;
    let state = f.pick("state", &[("open", DoorState::Open), ("closed", DoorState::Closed), ("locked", DoorState::Locked)])?;
    f.end(Door { x, y, axis, state })
}

fn cam(mut f: Fields<'_>) -> Result<Cam, ParseError> {
    let area = f.rect()?;
    let eye = f.point3(["px", "ph", "pz"])?;
    let target = f.point3(["tx", "th", "tz"])?;
    let fov = f.fx("fov_deg")?;
    f.end(Cam { area, eye, target, fov })
}

fn prop<'a>(mut f: Fields<'a>) -> Result<Prop<'a>, ParseError> {
    let kind = f.word("prop kind")?;
    let pos = f.pos()?;
    let rot = f.rot()?;
    f.end(Prop { kind, pos, rot })
}

const TASKS: &[(&str, TaskKind)] = &[
    ("window", TaskKind::Window),
    ("breaker", TaskKind::Breaker),
    ("key", TaskKind::Key),
    ("tv", TaskKind::Tv),
    ("gas", TaskKind::Gas),
    ("faucet", TaskKind::Faucet),
];

impl<'a> Builder<'a> {
    fn feed(mut self, n: usize, raw: &'a str) -> Result<Self, ParseError> {
        let line = raw.trim_end_matches('\r');
        let t = line.trim();
        if t.is_empty() || t.starts_with(';') {
            return Ok(self);
        }
        if let Some(name) = t.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            self.section = Section::named(name).map_or_else(|| err(n, format!("unknown section [{name}]")), Ok)?;
            return Ok(self);
        }
        let f = Fields { line: n, it: t.split_whitespace() };
        match self.section {
            Section::None => return err(n, "data before any [section]"),
            Section::Solid => self.solid.push((n, line)),
            Section::Rooms => self.room_rows.push((n, line)),
            Section::Room => {
                let r = room(f)?;
                if self.rooms.iter().any(|o| o.id == r.id) {
                    return err(n, format!("duplicate room id '{}'", r.id as char));
                }
                self.rooms.push(r);
            }
            Section::Door => self.doors.push((n, door(f)?)),
            Section::Cam => self.cams.push(cam(f)?),
            Section::Prop => self.props.push(prop(f)?),
            Section::Spot => self.spot(f)?,
        }
        Ok(self)
    }

    fn spot(&mut self, mut f: Fields<'a>) -> Result<(), ParseError> {
        let n = f.line;
        match f.word("spot kind")? {
            "spawn" => {
                let s = Spawn { pos: f.pos()?, rot: f.rot()? };
                self.spawns.push(f.end(s)?);
            }
            "task" => {
                let t = Task { kind: f.pick("task", TASKS)?, pos: f.pos()? };
                self.tasks.push(f.end(t)?);
            }
            "final" if self.final_area.is_some() => return err(n, "duplicate final"),
            "final" => {
                let r = f.rect()?;
                self.final_area = Some(f.end(r)?);
            }
            "exit" if self.exit.is_some() => return err(n, "duplicate exit"),
            "exit" => {
                let p = f.pos()?;
                self.exit = Some(f.end(p)?);
            }
            other => return err(n, format!("unknown spot kind '{other}'")),
        }
        Ok(())
    }

    fn finish(self) -> Result<Map<'a>, ParseError> {
        if self.solid.is_empty() {
            return err(0, "missing [solid] grid");
        }
        let w = self.solid.iter().map(|(_, r)| r.len()).max().unwrap_or(0);
        let h = self.solid.len();
        if self.room_rows.len() != h {
            let line = self.room_rows.last().map_or(0, |(n, _)| *n);
            return err(line, format!("[rooms] has {} rows, [solid] has {h}", self.room_rows.len()));
        }
        let tiles = self
            .solid
            .iter()
            .map(|&(n, row)| grid_row(n, row, w, b' ', |b| Tile::from_byte(b).ok_or("tile")))
            .collect::<Result<Vec<_>, _>>()?
            .concat();
        let rooms = &self.rooms;
        let room_of = self
            .room_rows
            .iter()
            .map(|&(n, row)| {
                grid_row(n, row, w, b'-', |b| match b {
                    b'-' | b' ' => Ok(None),
                    id => rooms.iter().position(|r| r.id == id).map(|i| Some(i as u8)).ok_or("room id without [room] line"),
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .concat();
        let map = Map {
            w: w as i32,
            h: h as i32,
            tiles,
            room_of,
            rooms: self.rooms,
            doors: self.doors.iter().map(|(_, d)| *d).collect(),
            cams: self.cams,
            props: self.props,
            spawns: self.spawns,
            tasks: self.tasks,
            final_area: self.final_area.map_or_else(|| err(0, "missing spot final"), Ok)?,
            exit: self.exit.map_or_else(|| err(0, "missing spot exit"), Ok)?,
        };
        if map.spawns.len() < 4 {
            return err(0, format!("need at least 4 spawn spots, found {}", map.spawns.len()));
        }
        if map.rooms.len() > u8::MAX as usize {
            return err(0, "too many rooms");
        }
        if let Some((n, d)) = self.doors.iter().find(|(_, d)| map.tile(d.x, d.y) != Tile::Door) {
            return err(*n, format!("door at ({}, {}) is not on a 'D' tile", d.x, d.y));
        }
        let walkable = (0..map.h).flat_map(|y| (0..map.w).map(move |x| (x, y))).filter(|&(x, y)| map.tile(x, y).walkable());
        let row_line = |y: i32| self.solid[y as usize].0;
        let floor_without_room = |&(x, y): &(i32, i32)| map.tile(x, y) == Tile::Floor && map.room_at(x, y).is_none();
        if let Some((x, y)) = walkable.clone().find(floor_without_room) {
            return err(row_line(y), format!("floor tile ({x}, {y}) has no room"));
        }
        if let Some((x, y)) = walkable.clone().find(|&(x, y)| map.cam_at(x, y).is_none()) {
            return err(row_line(y), format!("walkable tile ({x}, {y}) not covered by any [cam]"));
        }
        Ok(map)
    }
}

fn grid_row<T>(n: usize, row: &str, w: usize, pad: u8, cell: impl Fn(u8) -> Result<T, &'static str>) -> Result<Vec<T>, ParseError> {
    if row.len() > w {
        return err(n, format!("row is {} wide, expected {w}", row.len()));
    }
    row.bytes()
        .chain(std::iter::repeat_n(pad, w - row.len()))
        .enumerate()
        .map(|(x, b)| cell(b).or_else(|what| err(n, format!("bad {what} '{}' at column {x}", b as char))))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "; tiny house
[solid]
######
#..D.#
#.B..#
######

[rooms]
------
-aaab-
-aabb-
------
[room]
a sala floor=taco wall=reboco ceil=forro light=60
b quintal floor=terra wall=muro ceil=- light=100
[door]
3 1 v closed
[cam]
0 0 2 3 1.5 2.4 3.5 2 0.5 1 60
3 0 5 3 4.5 2.4 3.5 4 0 1 55.5
[prop]
sofa 1.5 2.5 90
[spot]
spawn 1.5 1.5 0
spawn 2.5 1.5 90
spawn 4.5 1.5 180
spawn 4.5 2.5 270
task window 4 1
task breaker 1 2.5
final 1 1 4 2
exit 4.5 2.5
";

    fn with_line(n: usize, replacement: &str) -> String {
        SAMPLE
            .lines()
            .enumerate()
            .map(|(i, l)| if i + 1 == n { replacement } else { l })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn parses_sample() {
        let m = parse(SAMPLE).unwrap();
        assert_eq!((m.w, m.h), (6, 4));
        assert_eq!(m.tile(3, 1), Tile::Door);
        assert_eq!(m.tile(2, 2), Tile::Blocker);
        assert_eq!(m.tile(-1, 0), Tile::Void);
        assert_eq!(m.room_at(4, 1).map(|r| r.name), Some("quintal"));
        assert_eq!(m.room_at(4, 1).and_then(|r| r.ceil), None);
        assert_eq!(m.room_at(1, 1).and_then(|r| r.ceil), Some("forro"));
        assert_eq!(m.doors, vec![Door { x: 3, y: 1, axis: Axis::V, state: DoorState::Closed }]);
        assert_eq!(m.cam_at(4, 2).map(|c| c.fov), Some(Fx::int(55) + Fx::HALF));
        assert_eq!(m.props[0].rot, Ang::QUARTER);
        assert_eq!(m.spawns.len(), 4);
        assert_eq!(m.tasks[1], Task { kind: TaskKind::Breaker, pos: Vec2::new(Fx::int(1), Fx::int(2) + Fx::HALF) });
        assert_eq!(m.final_area, Rect { x0: 1, y0: 1, x1: 4, y1: 2 });
    }

    #[test]
    fn errors_carry_line_numbers() {
        let line_of = |src: &str| parse(src).unwrap_err().line;
        assert_eq!(line_of(&with_line(4, "#..X.#")), 4);
        assert_eq!(line_of(&with_line(17, "2 1 v closed")), 17);
        assert_eq!(line_of(&with_line(17, "3 1 diagonal closed")), 17);
        assert_eq!(line_of(&with_line(20, "")), 4);
        assert_eq!(line_of(&with_line(22, "sofa 1.5 abc 90")), 22);
        assert_eq!(line_of(&with_line(15, "b quintal floor=terra wall=muro ceil=- light=101")), 15);
        assert_eq!(line_of(&with_line(9, "-aaac-")), 9);
        assert_eq!(parse(&with_line(24, "")).unwrap_err().line, 0);
    }

    #[test]
    fn sobrado_is_valid() {
        let m = sobrado();
        assert!(m.spawns.len() >= 4 && !m.cams.is_empty() && !m.rooms.is_empty());
    }
}
