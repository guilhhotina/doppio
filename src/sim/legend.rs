use super::field::{self, FAR};
use super::grid::{Grid, NONE};
use super::pass::BLOCKED;
use super::player::BENEDITA;
use super::{collide, Event, ItemKind, Legend, LegendAnim, LegendKind, LegendMode, PlayerState, World, TICS_PER_SEC};
use crate::fx::{Ang, Fx, Rng, Vec2};
use crate::map::Map;

const SEC: u16 = TICS_PER_SEC as u16;
const START_GRACE: u16 = 5 * SEC;
const SEARCH_TICS: u16 = 10 * SEC;
pub const LEAVE_TICS: u16 = 15 * SEC;
const INVESTIGATE_TICS: u16 = 20 * SEC;
const GRAB_TICS: u16 = super::player::GRAB_TICS as u16;
const SNIFF_TICS: u16 = 45;
const MULA_STUN: u16 = 4 * SEC;
const CRUZ_STUN: u16 = 5 * SEC;
const CABRA_CRY: u16 = 20 * SEC;
const MATINTA_CRY: u16 = 12 * SEC;
const BREATH_COOL: u16 = 6 * SEC;
const BREATH_TICS: u8 = 20;
const CHARGE_COOL: u16 = 4 * SEC;
const CHARGE_MAX: u16 = 4 * SEC;
const PORTAL_WAIT: u8 = 10;
const STALL_TICS: u16 = 3 * SEC;
const HEAR_TICS: u16 = 15;
const HEAR_LIMIT: u16 = 100;
const SEARCH_RADIUS: i32 = 3;
const TURN_RATE: i32 = 2731;
const WALK_STEP: u8 = 18;
const RUN_STEP: u8 = 10;
const GALLOP_STEP: u8 = 7;
pub const ALERT_TICS: u8 = 15;
pub const LUNGE_TICS: u8 = 8;
pub const CALM_TICS: u32 = 45 * TICS_PER_SEC;
const HEAT_PACE: [i32; 4] = [-1, 0, 1, 2];
const HEAT_HEAR: [i32; 4] = [2, 4, 5, 6];
const HEAT_SNIFF: [u16; 4] = [SNIFF_TICS, SNIFF_TICS, 30, 20];
const HEAT_SEARCH: [u16; 4] = [0, 0, 3 * SEC, 6 * SEC];
const HEAT_UNLOCK: [u8; 4] = [0, 40, 60, 90];

const fn sq(f: Fx) -> i64 {
    f.0 as i64 * f.0 as i64
}

const fn mps(tenths: i32) -> Fx {
    Fx::ratio(tenths, 10 * TICS_PER_SEC as i32)
}

const GRAB_REACH: i64 = sq(Fx::ratio(7, 10));
const LUNGE_REACH: i64 = sq(Fx::ratio(8, 10));
const SNATCH_REACH: i64 = sq(Fx::ONE);
const PULL_REACH: i64 = sq(Fx::ONE);
const ARRIVE: i64 = sq(Fx::ratio(3, 10));
const ON_PORTAL: i64 = sq(Fx::ratio(3, 10));
const BREATH_RANGE: i64 = sq(Fx::int(5));
const SIGHT: i64 = sq(Fx::int(9));
const SIGHT_DARK: i64 = sq(Fx::int(6));
const SIGHT_LIT: i64 = sq(Fx::int(12));
const CONE_COS: Fx = Fx::ratio(6428, 10000);
const DARK_LIGHT: u8 = 20;
const CHARGE_SPEED: Fx = mps(35);
const CHARGE_OVERSHOOT: Fx = Fx::ratio(3, 2);

pub struct Speeds {
    pub patrol: Fx,
    pub investigate: Fx,
    pub chase: Fx,
}

pub fn speeds(kind: LegendKind) -> Speeds {
    let (patrol, investigate, chase) = match kind {
        LegendKind::CorpoSeco => (9, 12, 20),
        LegendKind::Cabra => (13, 16, 22),
        LegendKind::Mula => (11, 15, 22),
        LegendKind::Matinta => (10, 14, 22),
    };
    Speeds { patrol: mps(patrol), investigate: mps(investigate), chase: mps(chase) }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Brain {
    pub dirty: bool,
    pub goal: Vec2,
    pub field_goal: u16,
    pub field_complete: bool,
    pub hear_cell: u16,
    pub hear_age: u16,
    pub waypoint: u8,
    pub timer: u16,
    pub pause: u16,
    pub origin: Vec2,
    pub seen_hide: bool,
    pub cry: u16,
    pub breath: u16,
    pub special: u8,
    pub charge: u16,
    pub charge_dir: Vec2,
    pub charge_cool: u16,
    pub step: u8,
    pub stall: u16,
    pub best: u16,
    pub portal_wait: u8,
    pub alert: u8,
    pub lunge: u8,
}

fn nearest_waypoint(map: &Map, at: Vec2) -> u8 {
    map.patrol
        .iter()
        .enumerate()
        .min_by_key(|&(i, &(x, y))| ((super::centre(x, y) - at).len_sq(), i))
        .map_or(0, |(i, _)| i as u8)
}

fn first_cry(kind: LegendKind) -> u16 {
    match kind {
        LegendKind::Cabra => CABRA_CRY / 2,
        LegendKind::Matinta => MATINTA_CRY / 2,
        _ => 0,
    }
}

impl Brain {
    pub fn new(map: &Map, at: Vec2) -> Brain {
        Brain {
            dirty: true,
            goal: at,
            field_goal: u16::MAX,
            field_complete: false,
            hear_cell: u16::MAX,
            hear_age: 0,
            waypoint: nearest_waypoint(map, at),
            timer: 0,
            pause: START_GRACE,
            origin: at,
            seen_hide: false,
            cry: first_cry(map.legend),
            breath: 0,
            special: 0,
            charge: 0,
            charge_dir: Vec2::ZERO,
            charge_cool: 0,
            step: WALK_STEP,
            stall: 0,
            best: FAR,
            portal_wait: 0,
            alert: 0,
            lunge: 0,
        }
    }
}

pub fn spawn(map: &Map, grid: &Grid, rng: &mut Rng) -> Legend {
    let lair = map.lair;
    let spot = match map.legend_spawns.len() {
        0 => crate::map::Spot { pos: super::centre(lair.0, lair.1), rot: Ang::ZERO },
        n => map.legend_spawns[rng.below(n as u32) as usize],
    };
    Legend {
        kind: map.legend,
        pos: spot.pos,
        heading: spot.rot,
        area: grid.area_of(spot.pos),
        anim: LegendAnim::Idle,
        anim_tic: 0,
        mode: LegendMode::Patrol,
        target: None,
        stunned: 0,
    }
}

enum Steer {
    Toward(Vec2),
    Portal(usize),
}

impl World {
    pub fn heat(&self) -> u8 {
        let opened = self.locks.iter().filter(|l| l.open).count();
        match opened {
            0 if self.tic < CALM_TICS => 0,
            _ => 1 + (opened * 3 / self.locks.len().max(1)).min(2) as u8,
        }
    }

    pub(super) fn unlock_noise(&self) -> u8 {
        HEAT_UNLOCK[self.heat() as usize]
    }

    fn pace(&self, base: Fx) -> Fx {
        base + mps(HEAT_PACE[self.heat() as usize])
    }

    pub(super) fn legend_tic(&mut self) {
        let before = self.legend.anim;
        self.cries();
        let b = &mut self.brain;
        b.hear_age = b.hear_age.saturating_add(1);
        b.breath = b.breath.saturating_sub(1);
        b.charge_cool = b.charge_cool.saturating_sub(1);
        let anim = match self.legend.mode {
            LegendMode::Grab => self.grab_tic(),
            LegendMode::Stunned => self.stunned_tic(),
            LegendMode::Leaving => self.leaving_tic(),
            _ => self.hunt_tic(),
        };
        let area = self.grid.area_of(self.legend.pos);
        let lg = &mut self.legend;
        if area != NONE {
            lg.area = area;
        }
        lg.anim_tic = if anim == before { lg.anim_tic.saturating_add(1) } else { 0 };
        lg.anim = anim;
    }

    fn cries(&mut self) {
        let period = match self.legend.kind {
            LegendKind::Cabra => CABRA_CRY,
            LegendKind::Matinta => MATINTA_CRY,
            _ => return,
        };
        self.brain.cry = self.brain.cry.saturating_sub(1);
        if self.brain.cry == 0 {
            self.events.push(Event::LegendCry);
            self.brain.cry = period - period / 10 + self.rng.below(period as u32 / 5) as u16;
        }
    }

    fn grab_tic(&mut self) -> LegendAnim {
        self.brain.timer = self.brain.timer.saturating_sub(1);
        if self.brain.timer == 0 {
            self.send_legend_home(LEAVE_TICS);
        }
        LegendAnim::Grab
    }

    fn stunned_tic(&mut self) -> LegendAnim {
        self.legend.stunned = self.legend.stunned.saturating_sub(1);
        if self.legend.stunned == 0 {
            self.search_from(self.legend.pos);
        }
        LegendAnim::Stagger
    }

    fn leaving_tic(&mut self) -> LegendAnim {
        self.brain.timer = self.brain.timer.saturating_sub(1);
        if self.brain.timer == 0 {
            self.patrol_again();
            return LegendAnim::Idle;
        }
        let lair = super::centre(self.map.lair.0, self.map.lair.1);
        match self.go(lair, speeds(self.legend.kind).patrol) {
            true => LegendAnim::Idle,
            false => LegendAnim::Walk,
        }
    }

    pub(super) fn stun_legend(&mut self, tics: u16) {
        self.legend.mode = LegendMode::Stunned;
        self.legend.stunned = tics;
        self.legend.target = None;
        self.brain.charge = 0;
        self.brain.special = 0;
        self.brain.alert = 0;
        self.brain.lunge = 0;
        self.brain.seen_hide = false;
        self.events.push(Event::Stunned);
    }

    pub(super) fn send_legend_home(&mut self, tics: u16) {
        self.legend.mode = LegendMode::Leaving;
        self.legend.target = None;
        self.brain.timer = tics;
        self.brain.charge = 0;
        self.brain.special = 0;
        self.brain.alert = 0;
        self.brain.lunge = 0;
        self.brain.pause = 0;
        self.brain.seen_hide = false;
        self.reset_progress();
    }

    fn search_from(&mut self, at: Vec2) {
        self.legend.mode = LegendMode::Search;
        self.legend.target = None;
        self.brain.origin = at;
        self.brain.goal = at;
        self.brain.timer = SEARCH_TICS + HEAT_SEARCH[self.heat() as usize];
        self.brain.seen_hide = false;
        self.reset_progress();
    }

    fn patrol_again(&mut self) {
        self.legend.mode = LegendMode::Patrol;
        self.legend.target = None;
        self.brain.waypoint = nearest_waypoint(self.map, self.legend.pos);
        self.reset_progress();
    }

    fn reset_progress(&mut self) {
        self.brain.stall = 0;
        self.brain.best = FAR;
    }

    fn dark_at(&self, at: Vec2) -> bool {
        let room = self.grid.cell_of(at).map_or(NONE, |c| self.grid.room[c]);
        let lit = self.map.rooms.get(room as usize).map_or(0, |r| match self.power || r.out {
            true => r.light,
            false => 0,
        });
        lit < DARK_LIGHT
    }

    pub(super) fn sees(&self, p: usize) -> bool {
        let (pl, lg) = (self.players[p], self.legend);
        let d = pl.pos - lg.pos;
        let range = match (pl.lantern, self.dark_at(pl.pos)) {
            (true, _) => SIGHT_LIT,
            (false, true) => SIGHT_DARK,
            (false, false) => SIGHT,
        };
        pl.area == lg.area
            && d.len_sq() <= range
            && Vec2::from_ang(lg.heading).dot(d) >= d.len().mul(CONE_COS)
            && self.sight_clear(lg.pos, pl.pos)
    }

    fn spot_player(&self) -> Option<usize> {
        (0..self.count)
            .filter(|&p| self.players[p].state == PlayerState::Free && self.sees(p))
            .min_by_key(|&p| ((self.players[p].pos - self.legend.pos).len_sq(), p))
    }

    fn hunting(&self) -> bool {
        matches!(self.legend.mode, LegendMode::Patrol | LegendMode::Investigate | LegendMode::Search | LegendMode::Chase)
    }

    pub(super) fn legend_watch_hide(&mut self, p: usize) {
        if self.hunting() && self.sees(p) {
            self.chase(p);
            self.brain.seen_hide = true;
        }
    }

    fn chase(&mut self, p: usize) {
        if self.legend.mode != LegendMode::Chase {
            self.brain.alert = ALERT_TICS;
        }
        if self.legend.mode != LegendMode::Chase || self.legend.target != Some(p as u8) {
            self.events.push(Event::LegendSaw { player: p as u8 });
            self.reset_progress();
        }
        self.legend.mode = LegendMode::Chase;
        self.legend.target = Some(p as u8);
        self.brain.origin = self.players[p].pos;
        self.brain.pause = 0;
        self.brain.seen_hide = false;
    }

    fn hunt_tic(&mut self) -> LegendAnim {
        let seen = self.spot_player();
        if let Some(p) = seen {
            self.chase(p);
            if self.brain.alert == 0 {
                self.breathe(p);
                self.start_charge(p);
            }
        }
        if self.brain.alert > 0 {
            return self.alert_tic();
        }
        if self.brain.charge > 0 {
            return self.charge_tic();
        }
        if self.legend.mode != LegendMode::Chase {
            self.listen();
        }
        if self.brain.special > 0 {
            self.brain.special -= 1;
            self.grab_near();
            return LegendAnim::Special;
        }
        let anim = match self.legend.mode {
            LegendMode::Chase => self.chase_tic(seen.is_some()),
            LegendMode::Investigate => self.investigate_tic(),
            LegendMode::Search => self.search_tic(),
            _ => self.patrol_tic(),
        };
        self.grab_near();
        match self.brain.lunge {
            0 => anim,
            _ => LegendAnim::Grab,
        }
    }

    fn alert_tic(&mut self) -> LegendAnim {
        self.brain.alert -= 1;
        let toward = self.legend.target.map(|t| self.players[t as usize].pos - self.legend.pos);
        if let Some(d) = toward.filter(|d| d.len_sq() > 0) {
            let turn = ((d.ang() - self.legend.heading).0 as i16 as i32).clamp(-2 * TURN_RATE, 2 * TURN_RATE);
            self.legend.heading = self.legend.heading + Ang(turn as u16);
        }
        LegendAnim::Alert
    }

    fn paused(&mut self) -> bool {
        self.brain.pause = self.brain.pause.saturating_sub(1);
        self.brain.pause > 0
    }

    fn patrol_tic(&mut self) -> LegendAnim {
        if self.paused() || self.map.patrol.is_empty() {
            return LegendAnim::Sniff;
        }
        let (x, y) = self.map.patrol[self.brain.waypoint as usize % self.map.patrol.len()];
        match self.go(super::centre(x, y), self.pace(speeds(self.legend.kind).patrol)) {
            true => {
                self.brain.waypoint = ((self.brain.waypoint as usize + 1) % self.map.patrol.len()) as u8;
                self.brain.pause = HEAT_SNIFF[self.heat() as usize];
                self.reset_progress();
                LegendAnim::Sniff
            }
            false => LegendAnim::Walk,
        }
    }

    fn investigate_tic(&mut self) -> LegendAnim {
        self.brain.timer = self.brain.timer.saturating_sub(1);
        let arrived = self.go(self.brain.origin, self.pace(speeds(self.legend.kind).investigate));
        if arrived || self.brain.timer == 0 {
            self.search_from(self.legend.pos);
            self.brain.pause = SNIFF_TICS;
            return LegendAnim::Sniff;
        }
        LegendAnim::Walk
    }

    fn search_tic(&mut self) -> LegendAnim {
        self.brain.timer = self.brain.timer.saturating_sub(1);
        if self.brain.timer == 0 {
            self.patrol_again();
            return LegendAnim::Idle;
        }
        if self.paused() {
            return LegendAnim::Sniff;
        }
        match self.go(self.brain.goal, self.pace(speeds(self.legend.kind).patrol)) {
            true => {
                self.pick_search_point();
                LegendAnim::Sniff
            }
            false => LegendAnim::Walk,
        }
    }

    fn pick_search_point(&mut self) {
        let (ox, oy) = (self.brain.origin.x.floor(), self.brain.origin.y.floor());
        let span = (2 * SEARCH_RADIUS + 1) as u32;
        let found = (0..8).find_map(|_| {
            let (x, y) = (ox + self.rng.below(span) as i32 - SEARCH_RADIUS, oy + self.rng.below(span) as i32 - SEARCH_RADIUS);
            self.grid
                .cell(x, y)
                .filter(|&c| self.map.tiles[c] == crate::map::Tile::Floor && self.legend_cost(c).is_some())
                .map(|c| self.grid.centre(c))
        });
        self.brain.goal = found.unwrap_or(self.brain.origin);
        self.brain.pause = SNIFF_TICS / 2;
        self.reset_progress();
    }

    fn chase_tic(&mut self, seen: bool) -> LegendAnim {
        let Some(t) = self.legend.target.map(|t| t as usize) else {
            self.search_from(self.legend.pos);
            return LegendAnim::Idle;
        };
        let held = self.brain.seen_hide && matches!(self.players[t].state, PlayerState::Hidden { .. });
        let goal = match held {
            true => self.players[t].pos,
            false => self.brain.origin,
        };
        match self.go(goal, speeds(self.legend.kind).chase) {
            true if !held && !seen => {
                self.events.push(Event::LegendLost);
                self.search_from(self.legend.pos);
                LegendAnim::Sniff
            }
            _ => LegendAnim::Run,
        }
    }

    pub(super) fn listen(&mut self) {
        if !self.events.iter().any(|e| matches!(e, Event::Noise { .. })) {
            return;
        }
        let Some(here) = self.grid.cell_of(self.legend.pos) else { return };
        if self.brain.hear_cell != here as u16 || self.brain.hear_age >= HEAR_TICS {
            let (mut dist, mut heap) = (std::mem::take(&mut self.hear), std::mem::take(&mut self.heap));
            field::flood(&mut dist, &mut heap, here, HEAR_LIMIT, None, |c| self.sound_edges(c));
            (self.hear, self.heap) = (dist, heap);
            self.brain.hear_cell = here as u16;
            self.brain.hear_age = 0;
        }
        let hear = HEAT_HEAR[self.heat() as usize];
        let heard = self
            .events
            .iter()
            .filter_map(|e| match *e {
                Event::Noise { pos, loud, .. } => self.grid.cell_of(pos).map(|c| (pos, loud as i32 * hear / 4 - self.hear[c] as i32)),
                _ => None,
            })
            .filter(|&(_, margin)| margin >= 0)
            .max_by_key(|&(_, margin)| margin);
        if let Some((pos, _)) = heard {
            if self.legend.mode != LegendMode::Investigate || (pos - self.brain.origin).len_sq() > sq(Fx::ONE) {
                self.reset_progress();
            }
            self.legend.mode = LegendMode::Investigate;
            self.brain.origin = pos;
            self.brain.timer = INVESTIGATE_TICS;
            self.brain.pause = 0;
        }
    }

    fn breathe(&mut self, p: usize) {
        let close = (self.players[p].pos - self.legend.pos).len_sq() <= BREATH_RANGE;
        if self.legend.kind == LegendKind::Cabra && close && self.brain.breath == 0 {
            self.brain.breath = BREATH_COOL;
            self.brain.special = BREATH_TICS;
            self.events.push(Event::LegendSpecial);
        }
    }

    fn start_charge(&mut self, p: usize) {
        if self.legend.kind != LegendKind::Mula || self.brain.charge > 0 || self.brain.charge_cool > 0 {
            return;
        }
        let d = self.players[p].pos - self.legend.pos;
        let run = d.len() + CHARGE_OVERSHOOT;
        self.brain.charge = ((run.0 / CHARGE_SPEED.0.max(1)) as u16).clamp(1, CHARGE_MAX);
        self.brain.charge_dir = d.norm();
        self.legend.heading = d.ang();
        self.events.push(Event::LegendSpecial);
    }

    fn charge_tic(&mut self) -> LegendAnim {
        let lg = self.legend;
        let step = self.brain.charge_dir.scale(CHARGE_SPEED);
        let next = collide::slide(lg.pos, step, |x, y| self.legend_solid(x, y));
        if (next - lg.pos).len_sq() * 4 < step.len_sq() {
            self.stun_legend(MULA_STUN);
            return LegendAnim::Stagger;
        }
        self.legend.pos = next;
        self.brain.charge -= 1;
        if self.brain.charge == 0 {
            self.brain.charge_cool = CHARGE_COOL;
            self.reset_progress();
        }
        self.footfall(GALLOP_STEP);
        self.grab_near();
        LegendAnim::Run
    }

    fn grab_near(&mut self) {
        if !self.hunting() {
            return;
        }
        let at = self.legend.pos;
        let pull = self.legend.target.map(|t| t as usize).filter(|&t| {
            self.brain.seen_hide && matches!(self.players[t].state, PlayerState::Hidden { .. }) && (self.players[t].pos - at).len_sq() <= PULL_REACH
        });
        let near = |reach: i64| {
            (0..self.count)
                .filter(|&p| self.players[p].state == PlayerState::Free)
                .map(|p| (p, (self.players[p].pos - at).len_sq()))
                .filter(|&(_, d)| d <= reach)
                .min_by_key(|&(p, d)| (d, p))
                .map(|(p, _)| p)
        };
        let caught = match (pull, self.brain.charge > 0, self.brain.lunge) {
            (Some(p), _, _) => Some(p),
            (None, true, _) => near(GRAB_REACH),
            (None, false, 0) => {
                if let Some(p) = near(LUNGE_REACH) {
                    self.brain.lunge = 1;
                    self.events.push(Event::Lunge { player: p as u8 });
                }
                None
            }
            (None, false, n) if n >= LUNGE_TICS => {
                self.brain.lunge = 0;
                near(SNATCH_REACH)
            }
            (None, false, n) => {
                self.brain.lunge = n + 1;
                None
            }
        };
        if let Some(p) = caught {
            self.brain.lunge = 0;
            self.seize(p);
        }
    }

    fn seize(&mut self, p: usize) {
        let warded = self.legend.kind == LegendKind::CorpoSeco
            && self.players[p].character == BENEDITA
            && self.holds(p, ItemKind::Cruz).is_some();
        if warded {
            self.events.push(Event::LegendSpecial);
            self.stun_legend(CRUZ_STUN);
            return;
        }
        self.grab_player(p);
        self.legend.mode = LegendMode::Grab;
        self.legend.target = Some(p as u8);
        self.legend.heading = (self.players[p].pos - self.legend.pos).ang();
        self.brain.timer = GRAB_TICS;
        self.brain.charge = 0;
        self.brain.special = 0;
        self.brain.seen_hide = false;
    }

    fn footfall(&mut self, interval: u8) {
        self.brain.step = match self.brain.step {
            0 | 1 => {
                self.events.push(Event::LegendStep { surface: self.grid.surface_at(self.legend.pos) });
                interval
            }
            n => n.min(interval) - 1,
        };
    }

    fn refresh_path(&mut self, goal: usize, here: usize) {
        let b = self.brain;
        if b.field_goal != goal as u16 || b.dirty || (self.path[here] == FAR && !b.field_complete) {
            let mut costs = std::mem::take(&mut self.costs);
            costs.iter_mut().enumerate().for_each(|(c, k)| *k = self.legend_cost(c).map_or(BLOCKED, |v| v as u8));
            let (mut dist, mut heap) = (std::mem::take(&mut self.path), std::mem::take(&mut self.heap));
            let complete = field::flood(&mut dist, &mut heap, goal, FAR, Some(here), |c| self.legend_prev(&costs, c));
            (self.path, self.heap, self.costs) = (dist, heap, costs);
            self.brain.field_goal = goal as u16;
            self.brain.field_complete = complete;
            self.brain.dirty = false;
        }
    }

    fn best_next(&self, c: usize) -> Option<usize> {
        let here = self.path[c];
        self.legend_next(c)
            .iter()
            .filter(|&(n, _)| self.path[n] < here)
            .min_by_key(|&(n, w)| (self.path[n].saturating_add(w), n))
            .map(|(n, _)| n)
    }

    fn adjacent(&self, a: usize, b: usize) -> bool {
        let ((ax, ay), (bx, by)) = (self.grid.xy(a), self.grid.xy(b));
        (ax - bx).abs() <= 1 && (ay - by).abs() <= 1
    }

    fn open_way(&mut self, c: usize) {
        if let Some(d) = self.door_of(c).filter(|&d| !self.door_passable(d) && !self.doors[d].locked && !self.doors[d].stuck) {
            self.move_door(d, true);
        }
    }

    fn steer(&mut self, here: usize, goal_cell: usize, goal: Vec2) -> Steer {
        if here == goal_cell || self.path[here] == FAR {
            return Steer::Toward(goal);
        }
        let Some(n1) = self.best_next(here) else { return Steer::Toward(goal) };
        if !self.adjacent(here, n1) {
            return Steer::Portal(n1);
        }
        self.open_way(n1);
        let aim = |c: usize| if c == goal_cell { goal } else { self.grid.centre(c) };
        let far = self
            .best_next(n1)
            .filter(|&n2| self.adjacent(n1, n2))
            .map(aim)
            .filter(|&p| super::los::clear(self.legend.pos, p, |x, y| self.legend_solid(x, y)));
        Steer::Toward(far.unwrap_or(aim(n1)))
    }

    pub(super) fn go(&mut self, goal: Vec2, speed: Fx) -> bool {
        let at = self.legend.pos;
        let (Some(here), Some(goal_cell)) = (self.grid.cell_of(at), self.grid.cell_of(goal)) else { return true };
        if (goal - at).len_sq() <= ARRIVE {
            return true;
        }
        self.refresh_path(goal_cell, here);
        let progress = self.path[here];
        match progress < self.brain.best {
            true => {
                self.brain.best = progress;
                self.brain.stall = 0;
            }
            false => self.brain.stall = self.brain.stall.saturating_add(1),
        }
        if self.brain.stall >= STALL_TICS {
            self.reset_progress();
            return true;
        }
        match self.steer(here, goal_cell, goal) {
            Steer::Toward(p) => self.advance(p, speed),
            Steer::Portal(to) => self.through_portal(here, to),
        }
        false
    }

    fn through_portal(&mut self, here: usize, to: usize) {
        let mouth = self.grid.centre(here);
        if (mouth - self.legend.pos).len_sq() > ON_PORTAL {
            self.advance(mouth, speeds(self.legend.kind).patrol);
            return;
        }
        self.brain.portal_wait += 1;
        if self.brain.portal_wait >= PORTAL_WAIT {
            self.brain.portal_wait = 0;
            let k = self.grid.portal[here] as usize;
            self.legend.pos = self.grid.centre(to);
            self.legend.heading = self.map.portals.get(k).map_or(self.legend.heading, |p| p.rot);
            self.reset_progress();
        }
    }

    fn advance(&mut self, target: Vec2, speed: Fx) {
        let lg = self.legend;
        let d = target - lg.pos;
        let len = d.len();
        if len.0 == 0 {
            return;
        }
        let speed = match self.brain.lunge {
            0 => speed,
            _ => Fx(speed.0 / 2),
        };
        let step = match len <= speed {
            true => d,
            false => d.norm().scale(speed),
        };
        let next = collide::slide(lg.pos, step, |x, y| self.legend_solid(x, y));
        let turn = ((d.ang() - lg.heading).0 as i16 as i32).clamp(-TURN_RATE, TURN_RATE);
        self.legend.heading = lg.heading + Ang(turn as u16);
        self.legend.pos = next;
        if next != lg.pos {
            let chase = lg.mode == LegendMode::Chase;
            let interval = match (chase, lg.kind) {
                (true, LegendKind::Mula) => GALLOP_STEP,
                (true, _) => RUN_STEP,
                _ => WALK_STEP,
            };
            self.footfall(interval);
        }
    }
}
