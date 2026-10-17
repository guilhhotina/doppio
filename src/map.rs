use crate::fx::{Ang, Fx, Vec2};
use std::fmt;
use std::sync::LazyLock;

mod parse;
#[cfg(test)]
mod tests;

pub use parse::parse;

include!(concat!(env!("OUT_DIR"), "/maps.rs"));

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Void,
    Wall,
    Floor,
    Door,
    Window,
    Blocker,
    Hide,
    Portal,
    Water,
    Fence,
}

impl Tile {
    pub fn walkable(self) -> bool {
        matches!(self, Tile::Floor | Tile::Door | Tile::Portal)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sky {
    Night,
    Moon,
    Overcast,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LegendKind {
    CorpoSeco,
    Cabra,
    Mula,
    Matinta,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemKind {
    Chave,
    Alicate,
    Martelo,
    PeDeCabra,
    ChaveDeFenda,
    Fusivel,
    Sal,
    Vela,
    Fosforo,
    Tabaco,
    Balde,
    Freio,
    Ossos,
    Cruz,
    BilheteOnibus,
    Manivela,
    Remo,
    LanternaPilha,
    Foto,
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
    Stuck,
    Locked(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PortalKind {
    Escada,
    EscadaMao,
    Bueiro,
    Portao,
    JanelaAlta,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HideKind {
    Cama,
    GuardaRoupa,
    Armario,
    Tumulo,
    CaixaDagua,
    Confessionario,
    Carro,
    Cesto,
    Mesa,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LightMode {
    Steady,
    Flicker,
    Dead,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LockKind {
    Key(u8),
    Code { digits: [u8; 4], len: u8 },
    Tool(ItemKind),
    Place { item: u8, x: i32, y: i32 },
    All(Vec<u8>),
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
pub struct Area<'a> {
    pub id: u8,
    pub name: &'a str,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Room<'a> {
    pub id: u8,
    pub name: &'a str,
    pub area: u8,
    pub floor: &'a str,
    pub wall: &'a str,
    pub ceil: Option<&'a str>,
    pub ext: Option<&'a str>,
    pub h: Fx,
    pub light: u8,
    pub out: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Door {
    pub x: i32,
    pub y: i32,
    pub axis: Axis,
    pub state: DoorState,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Portal {
    pub x: i32,
    pub y: i32,
    pub to_x: i32,
    pub to_y: i32,
    pub rot: Ang,
    pub kind: PortalKind,
    pub lock: Option<u8>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Lock<'a> {
    pub id: &'a str,
    pub kind: LockKind,
    pub hint: Option<&'a str>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Item<'a> {
    pub id: &'a str,
    pub kind: ItemKind,
    pub pos: Vec2,
    pub inside: Option<u16>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Note<'a> {
    pub x: i32,
    pub y: i32,
    pub target: &'a str,
    pub text: &'a str,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hide {
    pub x: i32,
    pub y: i32,
    pub kind: HideKind,
    pub rot: Ang,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Prop<'a> {
    pub kind: &'a str,
    pub pos: Vec2,
    pub rot: Ang,
    pub lock: Option<u8>,
    pub item: Option<u8>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Light {
    pub pos: Vec2,
    pub h: Fx,
    pub rgb: [u8; 3],
    pub radius: Fx,
    pub mode: LightMode,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cam {
    pub area: Rect,
    pub eye: [Fx; 3],
    pub target: [Fx; 3],
    pub fov: Fx,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Spot {
    pub pos: Vec2,
    pub rot: Ang,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Exit {
    pub rect: Rect,
    pub lock: u8,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Map<'a> {
    pub name: &'a str,
    pub w: i32,
    pub h: i32,
    pub sky: Sky,
    pub fog: [u8; 3],
    pub fog_near: Fx,
    pub fog_far: Fx,
    pub legend: LegendKind,
    pub power: bool,
    pub tiles: Vec<Tile>,
    pub room_of: Vec<Option<u8>>,
    pub rooms: Vec<Room<'a>>,
    pub areas: Vec<Area<'a>>,
    pub doors: Vec<Door>,
    pub portals: Vec<Portal>,
    pub locks: Vec<Lock<'a>>,
    pub items: Vec<Item<'a>>,
    pub notes: Vec<Note<'a>>,
    pub hides: Vec<Hide>,
    pub props: Vec<Prop<'a>>,
    pub lights: Vec<Light>,
    pub cams: Vec<Cam>,
    pub wakes: [Spot; 4],
    pub legend_spawns: Vec<Spot>,
    pub patrol: Vec<(i32, i32)>,
    pub lair: (i32, i32),
    pub exit: Exit,
}

impl<'a> Map<'a> {
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        ((0..self.w).contains(&x) && (0..self.h).contains(&y)).then(|| (y * self.w + x) as usize)
    }

    pub fn tile(&self, x: i32, y: i32) -> Tile {
        self.index(x, y).map_or(Tile::Void, |i| self.tiles[i])
    }

    pub fn room_index(&self, x: i32, y: i32) -> Option<u8> {
        self.index(x, y).and_then(|i| self.room_of[i])
    }

    pub fn room_at(&self, x: i32, y: i32) -> Option<&Room<'a>> {
        self.room_index(x, y).map(|r| &self.rooms[r as usize])
    }

    pub fn area_at(&self, x: i32, y: i32) -> Option<u8> {
        self.room_at(x, y).map(|r| r.area)
    }

    pub fn cam_at(&self, x: i32, y: i32) -> Option<&Cam> {
        self.cams.iter().find(|c| c.area.contains(x, y))
    }

    pub fn door_at(&self, x: i32, y: i32) -> Option<usize> {
        self.doors.iter().position(|d| (d.x, d.y) == (x, y))
    }

    pub fn portal_at(&self, x: i32, y: i32) -> Option<usize> {
        self.portals.iter().position(|p| (p.x, p.y) == (x, y))
    }

    pub fn hide_at(&self, x: i32, y: i32) -> Option<usize> {
        self.hides.iter().position(|h| (h.x, h.y) == (x, y))
    }

    pub fn lock_index(&self, id: &str) -> Option<u8> {
        self.locks.iter().position(|l| l.id == id).map(|i| i as u8)
    }

    pub fn item_index(&self, id: &str) -> Option<u8> {
        self.items.iter().position(|i| i.id == id).map(|i| i as u8)
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

static PARSED: LazyLock<Vec<Map<'static>>> = LazyLock::new(|| {
    MAPS.iter().map(|(name, src)| parse(src).unwrap_or_else(|e| panic!("{name}.map: {e}"))).collect()
});

static CAMPAIGN: LazyLock<Vec<&'static Map<'static>>> = LazyLock::new(|| PARSED.iter().collect());

pub fn campaign() -> &'static [&'static Map<'static>] {
    &CAMPAIGN
}
