use crate::fx::{Ang, Fx, Rng, Vec2};
use crate::map::{DoorState as MapDoor, Map};

mod act;
#[cfg(test)]
mod bot;
mod collide;
mod field;
#[cfg(test)]
mod feel;
mod focus;
mod grid;
mod hash;
mod legend;
mod lock;
mod los;
mod pass;
mod player;
mod tank;
#[cfg(test)]
mod tests;

pub use crate::map::{ItemKind, LegendKind};
pub use tank::DROP;

use field::Heap;
use grid::Grid;
use legend::Brain;
use player::Ctl;

pub const MAX_PLAYERS: usize = 4;
pub const MAX_ITEMS: usize = 64;
pub const MAX_DOORS: usize = 96;
pub const TICS_PER_SEC: u32 = 30;
pub const DESMAIOS: u8 = 5;
pub const BATTERY_FULL: u16 = 54000;
const MAX_EVENTS: usize = 128;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Setup {
    pub seed: u32,
    pub phase: u8,
    pub players: u8,
    pub characters: [u8; MAX_PLAYERS],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Playing,
    Escaped,
    Restart,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Player {
    pub character: u8,
    pub pos: Vec2,
    pub heading: Ang,
    pub area: u8,
    pub anim: Anim,
    pub anim_tic: u16,
    pub lantern: bool,
    pub battery: u16,
    pub hands: [Option<u8>; 2],
    pub state: PlayerState,
    pub fade: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerState {
    Free,
    Hidden { spot: u8 },
    Grabbed,
    Fainted { tics_left: u16 },
    Escaped,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Anim {
    Idle,
    Walk,
    Run,
    Turn,
    QuickTurn,
    Reach,
    Crouch,
    Grabbed,
    Faint,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Legend {
    pub kind: LegendKind,
    pub pos: Vec2,
    pub heading: Ang,
    pub area: u8,
    pub anim: LegendAnim,
    pub anim_tic: u16,
    pub mode: LegendMode,
    pub target: Option<u8>,
    pub stunned: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LegendMode {
    Patrol,
    Investigate,
    Chase,
    Search,
    Grab,
    Stunned,
    Leaving,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LegendAnim {
    Idle,
    Walk,
    Run,
    Sniff,
    Grab,
    Stagger,
    Special,
    Alert,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DoorState {
    pub open: u16,
    pub locked: bool,
    pub stuck: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PropState {
    pub open: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ItemState {
    pub kind: ItemKind,
    pub place: ItemPlace,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemPlace {
    Floor { pos: Vec2, area: u8 },
    Held { player: u8 },
    Inside { prop: u16 },
    Used,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LockState {
    pub open: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focus {
    Door { door: u16, locked: bool, stuck: bool, open: bool },
    Item { item: u8 },
    Container { prop: u16, locked: bool },
    Lock { lock: u8, needs: Option<ItemKind>, code: bool },
    Place { lock: u8, needs: ItemKind },
    Hide { spot: u8 },
    Portal { portal: u8, locked: bool },
    Note { note: u16 },
    Friend { player: u8 },
    Use { item: u8 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Dial {
    pub lock: u8,
    pub digits: [u8; 4],
    pub len: u8,
    pub cursor: u8,
    pub wrong: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    Step { player: u8, surface: u8, run: bool },
    LegendStep { surface: u8 },
    LegendCry,
    LegendSpecial,
    LegendSaw { player: u8 },
    LegendLost,
    Noise { pos: Vec2, area: u8, loud: u8 },
    DoorMove { door: u16, opening: bool },
    DoorRattle { door: u16 },
    Pickup { player: u8, item: u8 },
    Drop { player: u8, item: u8 },
    Unlock { lock: u8 },
    DialWrong { player: u8 },
    DialTick { player: u8 },
    Hide { player: u8 },
    Unhide { player: u8 },
    Grabbed { player: u8 },
    Fainted { player: u8 },
    Woke { player: u8 },
    Stunned,
    Portal { player: u8 },
    Lantern { player: u8, on: bool },
    Escaped { player: u8 },
    PhaseWon,
    PhaseRestart,
    PowerOut,
    PowerOn,
    Container { player: u8, prop: u16 },
    Used { player: u8, item: u8 },
    PortalUse { player: u8, portal: u8 },
    Lunge { player: u8 },
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

pub struct World {
    map: &'static Map<'static>,
    setup: Setup,
    count: usize,
    grid: Grid,
    tic: u32,
    rng: Rng,
    outcome: Outcome,
    desmaios: u8,
    power: bool,
    players: [Player; MAX_PLAYERS],
    ctl: [Ctl; MAX_PLAYERS],
    legend: Legend,
    brain: Brain,
    doors: Vec<DoorState>,
    door_dir: Vec<bool>,
    salt: Vec<u16>,
    props: Vec<PropState>,
    items: Vec<ItemState>,
    locks: Vec<LockState>,
    path: Vec<u16>,
    costs: Vec<u8>,
    hear: Vec<u16>,
    heap: Heap,
    events: Events,
}

impl World {
    pub fn new(map: &'static Map<'static>, setup: Setup) -> World {
        let count = setup.players.clamp(1, MAX_PLAYERS as u8) as usize;
        let grid = Grid::new(map);
        let cells = grid.cells();
        let mut rng = Rng::new(setup.seed ^ (setup.phase as u32).wrapping_mul(0x9e37_79b9));
        let players = std::array::from_fn(|p| {
            let character = setup.characters[p].min(3);
            let wake = map.wakes[character as usize];
            Player {
                character,
                pos: wake.pos,
                heading: wake.rot,
                area: grid.area_of(wake.pos),
                anim: Anim::Idle,
                anim_tic: 0,
                lantern: false,
                battery: BATTERY_FULL,
                hands: [None; 2],
                state: if p < count { PlayerState::Free } else { PlayerState::Escaped },
                fade: 0,
            }
        });
        let doors = map
            .doors
            .iter()
            .take(MAX_DOORS)
            .map(|d| DoorState {
                open: if d.state == MapDoor::Open { u16::MAX } else { 0 },
                locked: matches!(d.state, MapDoor::Locked(_)),
                stuck: d.state == MapDoor::Stuck,
            })
            .collect::<Vec<_>>();
        let items = map
            .items
            .iter()
            .take(MAX_ITEMS)
            .map(|i| ItemState {
                kind: i.kind,
                place: i.inside.map_or(ItemPlace::Floor { pos: i.pos, area: grid.area_of(i.pos) }, |prop| ItemPlace::Inside { prop }),
            })
            .collect();
        let legend = legend::spawn(map, &grid, &mut rng);
        let mut world = World {
            map,
            setup,
            count,
            tic: 0,
            outcome: Outcome::Playing,
            desmaios: DESMAIOS,
            power: map.power,
            players,
            ctl: [Ctl::REST; MAX_PLAYERS],
            brain: Brain::new(map, legend.pos),
            legend,
            door_dir: doors.iter().map(|d| d.open > 0).collect(),
            salt: vec![0; doors.len()],
            doors,
            props: vec![PropState { open: 0 }; map.props.len()],
            items,
            locks: vec![LockState { open: false }; map.locks.len()],
            path: vec![field::FAR; cells],
            costs: vec![0; cells],
            hear: vec![field::FAR; cells],
            heap: field::heap(cells),
            events: Events::new(),
            grid,
            rng,
        };
        world.cascade();
        world.events.clear();
        world
    }

    pub fn step(&mut self, inputs: &[u16; MAX_PLAYERS]) {
        self.events.clear();
        if self.outcome == Outcome::Playing {
            self.swing();
            (0..self.count).for_each(|p| self.player_tic(p, inputs[p]));
            self.legend_tic();
            self.settle_outcome();
        }
        self.tic = self.tic.wrapping_add(1);
    }

    pub fn events(&self) -> &Events {
        &self.events
    }

    pub fn hash(&self) -> u32 {
        hash::world(self)
    }

    pub fn map(&self) -> &'static Map<'static> {
        self.map
    }

    pub fn setup(&self) -> Setup {
        self.setup
    }

    pub fn outcome(&self) -> Outcome {
        self.outcome
    }

    pub fn desmaios_left(&self) -> u8 {
        self.desmaios
    }

    pub fn players(&self) -> &[Player] {
        &self.players[..self.count]
    }

    pub fn legend(&self) -> &Legend {
        &self.legend
    }

    pub fn doors(&self) -> &[DoorState] {
        &self.doors
    }

    pub fn props(&self) -> &[PropState] {
        &self.props
    }

    pub fn items(&self) -> &[ItemState] {
        &self.items
    }

    pub fn locks(&self) -> &[LockState] {
        &self.locks
    }

    pub fn focus(&self, player: usize) -> Option<Focus> {
        (player < self.count).then(|| self.focus_of(player)).flatten()
    }

    pub fn dial(&self, player: usize) -> Option<Dial> {
        self.ctl.get(player).filter(|_| player < self.count).and_then(|c| c.dial)
    }

    pub fn reading(&self, player: usize) -> Option<u16> {
        self.ctl.get(player).filter(|_| player < self.count).and_then(|c| c.reading)
    }

    pub fn power(&self) -> bool {
        self.power
    }

    pub fn tic(&self) -> u32 {
        self.tic
    }

    fn settle_outcome(&mut self) {
        let players = &self.players[..self.count];
        let escaped = players.iter().any(|p| p.state == PlayerState::Escaped);
        let done = players.iter().all(|p| matches!(p.state, PlayerState::Escaped | PlayerState::Fainted { .. }));
        if self.outcome == Outcome::Playing && escaped && done {
            self.outcome = Outcome::Escaped;
            self.events.push(Event::PhaseWon);
        }
    }

    fn holds(&self, p: usize, kind: ItemKind) -> Option<u8> {
        self.players[p].hands.iter().flatten().copied().find(|&i| self.items[i as usize].kind == kind)
    }
}

fn centre(x: i32, y: i32) -> Vec2 {
    Vec2::new(Fx::int(x) + Fx::HALF, Fx::int(y) + Fx::HALF)
}
