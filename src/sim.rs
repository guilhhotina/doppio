use crate::fx::{Ang, Rng, Vec2};
use crate::map::{self, Map, Tile};

mod body;
mod capture;
mod collide;
mod doors;
mod echo;
mod finale;
mod focus;
mod hash;
mod hunt;
mod label;
mod mirror;
mod night;
mod scare;
mod sight;
mod spawn;
mod tank;
mod tape;
mod tasks;
#[cfg(test)]
mod tests;

use echo::Echo;
use mirror::Pose;
use sight::View;
use tank::Tank;
use tape::Seg;

pub const MAX_PLAYERS: usize = 4;
pub const MAX_BODIES: usize = 24;
pub const TICS_PER_SEC: u32 = 30;

const NIGHTS: u8 = 5;
const MAX_EVENTS: usize = 96;
const MAX_TASKS: usize = 64;
const MAX_SPAWNS: usize = 16;
const NO_BODY: u8 = u8::MAX;
const NO_ROOM: u8 = u8::MAX;
const NO_DOOR: u8 = u8::MAX;
const NO_CAM: u8 = u8::MAX;
pub const BATTERY_FULL: u16 = 54000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Setup {
    pub seed: u32,
    pub players: u8,
    pub night_secs: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Dusk { night: u8 },
    Night { night: u8 },
    Dawn { night: u8 },
    Won,
    Lost,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Anim {
    Idle,
    Walk,
    Run,
    Turn,
    QuickTurn,
    Reach,
    Stagger,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Body {
    pub pos: Vec2,
    pub heading: Ang,
    pub anim: Anim,
    pub anim_tic: u16,
    pub lantern: bool,
    pub visible: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Living,
    Duplo,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Player {
    pub body: u8,
    pub role: Role,
    pub battery: u16,
    pub room: u8,
    pub fade: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Door {
    pub x: u8,
    pub y: u8,
    pub vertical: bool,
    pub open: u16,
    pub locked: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Task {
    pub kind: TaskKind,
    pub x: u8,
    pub y: u8,
    pub active: bool,
    pub done: bool,
    pub room: u8,
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
pub enum Focus {
    Door { door: u8, locked: bool, open: bool },
    Task { task: u8, kind: TaskKind, progress: u16 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    Step { body: u8, surface: u8 },
    DoorMove { door: u8, opening: bool },
    DoorRattle { door: u8 },
    Lantern { body: u8, on: bool },
    TaskDone { task: u8 },
    PowerOut,
    PowerOn,
    Captured { player: u8 },
    NightBegins { night: u8 },
    DawnBreaks { night: u8 },
    Glitch { body: u8 },
    CaughtBy { player: u8, body: u8 },
    EchoStep { body: u8, surface: u8, of: u8 },
}

#[derive(Clone)]
pub struct Events {
    buf: [Event; MAX_EVENTS],
    len: usize,
}

impl Events {
    const fn new() -> Events {
        Events { buf: [Event::PowerOn; MAX_EVENTS], len: 0 }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Event> {
        self.buf[..self.len].iter()
    }

    fn push(&mut self, e: Event) {
        if let Some(slot) = self.buf.get_mut(self.len) {
            *slot = e;
            self.len += 1;
        }
    }

    fn clear(&mut self) {
        self.len = 0;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Agent {
    Free,
    Player(u8),
    Echo,
}

#[derive(Clone, Copy, Debug)]
struct Ctl {
    agent: Agent,
    tank: Tank,
    battery: u16,
    step: u8,
    hold: u8,
    hold_task: u8,
}

impl Ctl {
    const FREE: Ctl = Ctl { agent: Agent::Free, tank: Tank::REST, battery: BATTERY_FULL, step: 0, hold: 0, hold_task: 0 };

    const fn fresh(agent: Agent) -> Ctl {
        Ctl { agent, ..Ctl::FREE }
    }
}

const HIDDEN: Body = Body {
    pos: Vec2::ZERO,
    heading: Ang::ZERO,
    anim: Anim::Idle,
    anim_tic: 0,
    lantern: false,
    visible: false,
};

const ABSENT: Player = Player { body: NO_BODY, role: Role::Living, battery: 0, room: NO_ROOM, fade: 0 };

fn shuffle(rng: &mut Rng, xs: &mut [u8]) {
    (1..xs.len()).rev().for_each(|i| xs.swap(i, rng.below(i as u32 + 1) as usize));
}

pub struct World {
    map: &'static Map<'static>,
    players_n: u8,
    night_tics: u32,
    nseg: u32,
    tic: u32,
    phase: Phase,
    phase_tic: u32,
    rng: Rng,
    power: bool,
    extra: u8,
    still: u16,
    exit_open: bool,
    bodies: [Body; MAX_BODIES],
    ctl: [Ctl; MAX_BODIES],
    echo: [Echo; MAX_BODIES],
    players: [Player; MAX_PLAYERS],
    capture: [u8; MAX_PLAYERS],
    captor: [u8; MAX_PLAYERS],
    eyes: [u8; MAX_PLAYERS],
    mirror: [Pose; MAX_PLAYERS],
    spawn_of: [[u8; MAX_SPAWNS]; NIGHTS as usize],
    doors: Vec<Door>,
    door_dir: Vec<bool>,
    door_at: Vec<u8>,
    exit_door: u8,
    tasks: Vec<Task>,
    anchors: Vec<Vec2>,
    views: Vec<View>,
    cam_of: Vec<u8>,
    poses: Vec<Pose>,
    room_surface: Vec<u8>,
    final_tiles: Vec<u16>,
    tape: Vec<u16>,
    segs: Vec<Seg>,
    field: Vec<u16>,
    queue: Vec<u16>,
    events: Events,
}

fn task_kind(k: map::TaskKind) -> TaskKind {
    match k {
        map::TaskKind::Window => TaskKind::Window,
        map::TaskKind::Breaker => TaskKind::Breaker,
        map::TaskKind::Key => TaskKind::Key,
        map::TaskKind::Tv => TaskKind::Tv,
        map::TaskKind::Gas => TaskKind::Gas,
        map::TaskKind::Faucet => TaskKind::Faucet,
    }
}

fn surface_of(floor: &str) -> u8 {
    crate::assets::TEXTURES.binary_search_by(|(name, _)| (*name).cmp(floor)).map_or(0, |i| i as u8)
}

impl World {
    pub fn new(map: &'static Map<'static>, setup: Setup) -> World {
        let players_n = setup.players.clamp(1, MAX_PLAYERS as u8);
        let night_tics = setup.night_secs.clamp(night::MIN_SECS, night::MAX_SECS) * TICS_PER_SEC;
        let nseg = night_tics / tape::SEG;
        let cells = (map.w * map.h) as usize;
        let doors: Vec<Door> = map
            .doors
            .iter()
            .map(|d| Door {
                x: d.x as u8,
                y: d.y as u8,
                vertical: d.axis == map::Axis::V,
                open: 0,
                locked: false,
            })
            .collect();
        let door_at = (0..cells as i32)
            .map(|i| {
                map.doors
                    .iter()
                    .position(|d| d.x == i % map.w && d.y == i / map.w)
                    .map_or(NO_DOOR, |d| d as u8)
            })
            .collect::<Vec<u8>>();
        let exit_door = door_at[(map.exit.y.floor() * map.w + map.exit.x.floor()).clamp(0, cells as i32 - 1) as usize];
        let room_at = |x: i32, y: i32| map.room_of.get((y * map.w + x) as usize).copied().flatten().unwrap_or(NO_ROOM);
        let tasks: Vec<Task> = map
            .tasks
            .iter()
            .take(MAX_TASKS)
            .map(|t| Task {
                kind: task_kind(t.kind),
                x: t.pos.x.floor() as u8,
                y: t.pos.y.floor() as u8,
                active: false,
                done: false,
                room: room_at(t.pos.x.floor(), t.pos.y.floor()),
            })
            .collect();
        let anchors = tasks.iter().map(|t| focus::anchor(map, t.kind, t.x as i32, t.y as i32)).collect();
        let cam_of = (0..cells as i32)
            .map(|i| map.cams.iter().position(|c| c.area.contains(i % map.w, i / map.w)).map_or(NO_CAM, |c| c as u8))
            .collect();
        let a = map.final_area;
        let final_tiles = (a.y0..=a.y1)
            .flat_map(|y| (a.x0..=a.x1).map(move |x| (x, y)))
            .filter(|&(x, y)| map.tile(x, y) == Tile::Floor)
            .map(|(x, y)| (y * map.w + x) as u16)
            .collect();
        let mut world = World {
            map,
            players_n,
            night_tics,
            nseg,
            tic: 0,
            phase: Phase::Dusk { night: 1 },
            phase_tic: 0,
            rng: Rng::new(setup.seed),
            power: true,
            extra: 0,
            still: 0,
            exit_open: false,
            bodies: [HIDDEN; MAX_BODIES],
            ctl: [Ctl::FREE; MAX_BODIES],
            echo: [Echo::NONE; MAX_BODIES],
            players: std::array::from_fn(|p| match p < players_n as usize {
                true => Player { body: p as u8, battery: BATTERY_FULL, ..ABSENT },
                false => ABSENT,
            }),
            capture: [0; MAX_PLAYERS],
            captor: [NO_BODY; MAX_PLAYERS],
            eyes: [NO_CAM; MAX_PLAYERS],
            mirror: [Pose::NONE; MAX_PLAYERS],
            spawn_of: [[0; MAX_SPAWNS]; NIGHTS as usize],
            door_dir: vec![false; doors.len()],
            doors,
            door_at,
            exit_door,
            tasks,
            anchors,
            views: map.cams.iter().map(sight::view).collect(),
            cam_of,
            poses: vec![Pose::NONE; MAX_PLAYERS * night_tics as usize],
            room_surface: map.rooms.iter().map(|r| surface_of(r.floor)).collect(),
            final_tiles,
            tape: vec![0; NIGHTS as usize * MAX_PLAYERS * night_tics as usize],
            segs: vec![Seg::ZERO; NIGHTS as usize * MAX_PLAYERS * nseg as usize],
            field: vec![u16::MAX; cells],
            queue: vec![0; cells],
            events: Events::new(),
        };
        world.begin_dusk(1);
        world.events.clear();
        world
    }

    pub fn step(&mut self, inputs: &[u16; MAX_PLAYERS]) -> &Events {
        self.events.clear();
        match self.phase {
            Phase::Dusk { night } => self.dusk_tic(night),
            Phase::Night { night } => self.night_tic(night, inputs),
            Phase::Dawn { night } => self.dawn_tic(night),
            Phase::Won | Phase::Lost => {}
        }
        self.tic = self.tic.wrapping_add(1);
        &self.events
    }

    pub fn events(&self) -> &Events {
        &self.events
    }

    pub fn hash(&self) -> u32 {
        hash::world(self)
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn clock_minutes(&self) -> u16 {
        night::clock(self.phase, self.phase_tic, self.night_tics)
    }

    pub fn bodies(&self) -> &[Body] {
        &self.bodies
    }

    pub fn player(&self, i: usize) -> &Player {
        &self.players[i]
    }

    pub fn doors(&self) -> &[Door] {
        &self.doors
    }

    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    pub fn power_on(&self) -> bool {
        self.power
    }

    pub fn map(&self) -> &'static Map<'static> {
        self.map
    }

    pub fn phase_tic(&self) -> u32 {
        self.phase_tic
    }

    pub fn focus(&self, player: usize) -> Option<Focus> {
        self.players.get(player).filter(|p| p.body != NO_BODY).and_then(|p| self.focus_of(p.body as usize))
    }

    pub fn task_spot(&self, task: usize) -> Vec2 {
        self.anchors[task]
    }

    pub fn room_title(&self, room: u8) -> &'static str {
        self.map.rooms.get(room as usize).map_or("", |r| label::room_title(r.name))
    }

    pub fn gaze(&self, body: usize) -> Option<Ang> {
        self.head_of(body)
    }

    pub fn captor(&self, player: usize) -> Option<(u8, u8)> {
        (player < self.players_n as usize && self.capture[player] > 0 && self.captor[player] != NO_BODY)
            .then(|| (self.captor[player], self.capture[player].saturating_sub(2)))
    }

    pub fn echo_pose(&self, player: usize) -> Option<(Vec2, Ang, Anim, u16)> {
        let recorded = matches!(self.phase, Phase::Night { night } if night >= 2);
        (recorded && player < self.players_n as usize).then(|| self.mirror[player].decode())
    }

    fn tile_index(&self, p: Vec2) -> Option<usize> {
        let (x, y) = (p.x.floor(), p.y.floor());
        ((0..self.map.w).contains(&x) && (0..self.map.h).contains(&y)).then(|| (y * self.map.w + x) as usize)
    }

    fn living(&self, p: usize) -> bool {
        p < self.players_n as usize && self.players[p].role == Role::Living && self.capture[p] == 0
    }

    fn living_bodies(&self) -> impl Iterator<Item = &Body> + '_ {
        (0..self.players_n as usize).filter(|&p| self.living(p)).map(|p| &self.bodies[self.players[p].body as usize])
    }
}
