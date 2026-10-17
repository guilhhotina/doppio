use super::super::tank::{A, B, DOWN, LEFT, RIGHT, UP, Y};
use super::super::{Dial, Focus, LegendMode, PlayerState, World, MAX_PLAYERS};
use super::nav::{self, Field, Hop, Walker, FAR};
use super::plan::{self, Ask, Job, Site};
use crate::fx::{Ang, Fx, Rng, Vec2};

const REPLAN: u32 = 90;
const RENAV: u32 = 60;
const STALL: u32 = 150;
const WIGGLE: u16 = 24;
const UNSTICKS: u8 = 4;
const CREEP: u16 = 60;
const PRESSES: u16 = 40;
const BAN: u32 = 450;
const LOOKAHEAD: usize = 6;
const ALIGN: i32 = 4551;
const FACE: i32 = 1820;
const QUICK: i32 = 23666;
const ARRIVE: i64 = sq(Fx::ratio(15, 100));
const FOLLOW: i64 = sq(Fx::int(2));
const DANGER: i64 = sq(Fx::int(4));
const RUN_FREE: i64 = sq(Fx::int(14));
const CALM_RANGE: i64 = sq(Fx::int(12));
const WATCH: i64 = sq(Fx::int(8));
const PATIENCE: u32 = 240;
const ALERT: i64 = sq(Fx::int(7));
const NEAR_LEGEND: i64 = sq(Fx::int(12));
const SHADOW: i64 = sq(Fx::int(3));
const SHADOW_FAR: i64 = sq(Fx::int(5));
const SHADOW_COST: u32 = 300;
const SHADOW_FAR_COST: u32 = 100;
const RENAV_NEAR: u32 = 15;
const CALM_TICS: u16 = 60;
const HIDE_MAX: u32 = 1500;
const FLEE_HOLD: u32 = 60;
const THREAT_EVERY: u32 = 10;
const HIDE_REACH: u32 = 250;
const HIDE_LEAD: u32 = 50;
const FLEE_REACH: u32 = 200;
const FLEE_MIN: u32 = 30;
const FLEE_LEAD: i64 = 20;
const SEEN_BAN: u32 = 120;
const RUN_COMMIT: u32 = 45;

const fn sq(f: Fx) -> i64 {
    f.0 as i64 * f.0 as i64
}

pub struct Shared {
    pub known: Vec<bool>,
    pub jobs: [Option<Job>; MAX_PLAYERS],
    pub rng: Rng,
    pub log: Vec<String>,
    pub trace: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Work,
    Flee,
    Hidden { since: u32, calm: u16, patience: u16 },
}

pub struct Pilot {
    p: usize,
    who: Walker,
    job: Option<Job>,
    sites: Vec<Site>,
    bad: Vec<usize>,
    nav: Field,
    reach: Field,
    threat: Field,
    shape: u64,
    nav_tic: u32,
    plan_tic: u32,
    threat_tic: u32,
    best: u32,
    since: u32,
    unsticks: u8,
    wiggle: u16,
    turn: u16,
    creep: u16,
    presses: u16,
    last: u16,
    mode: Mode,
    flee_tic: u32,
    run_tic: u32,
    banned: Vec<(Job, u32)>,
    delay: u32,
}

fn shape(w: &World) -> u64 {
    let doors = w.doors.iter().map(|d| d.locked as u64 | (d.stuck as u64) << 1);
    let locks = w.locks.iter().map(|l| l.open as u64);
    doors.chain(locks).fold(0xcbf2_9ce4_8422_2325, |h, v| (h ^ v).wrapping_mul(0x100_0000_01b3))
}

fn shadow(w: &World, c: usize) -> u32 {
    let lg = w.legend;
    let hunting = matches!(lg.mode, LegendMode::Patrol | LegendMode::Investigate | LegendMode::Search | LegendMode::Chase);
    let d2 = (w.grid.centre(c) - lg.pos).len_sq();
    match (hunting && w.grid.area[c] == lg.area, d2) {
        (true, d) if d < SHADOW => SHADOW_COST,
        (true, d) if d < SHADOW_FAR => SHADOW_FAR_COST,
        _ => 0,
    }
}

fn head(heading: Ang, aim: Ang, go: u16) -> u16 {
    let (turn, diff) = turn_to(heading, aim);
    match diff {
        d if d > QUICK => Y,
        d if d < ALIGN => turn | go,
        _ => turn,
    }
}

fn turn_to(heading: Ang, aim: Ang) -> (u16, i32) {
    let diff = (aim - heading).0 as i16 as i32;
    let bits = match diff {
        d if d > FACE / 2 => LEFT,
        d if d < -FACE / 2 => RIGHT,
        _ => 0,
    };
    (bits, diff.abs())
}

impl Pilot {
    pub fn new(w: &World, p: usize, delay: u32) -> Pilot {
        let cells = w.grid.cells();
        Pilot {
            p,
            who: Walker::Player { jairo: w.players[p].character == super::super::player::JAIRO },
            job: None,
            sites: Vec::new(),
            bad: Vec::new(),
            nav: Field::new(cells),
            reach: Field::new(cells),
            threat: Field::new(cells),
            shape: 0,
            nav_tic: 0,
            plan_tic: 0,
            threat_tic: 0,
            best: FAR,
            since: 0,
            unsticks: 0,
            wiggle: 0,
            turn: LEFT,
            creep: 0,
            presses: 0,
            last: 0,
            mode: Mode::Work,
            flee_tic: 0,
            run_tic: 0,
            banned: Vec::new(),
            delay,
        }
    }

    pub fn job(&self) -> Option<Job> {
        self.job
    }

    pub fn tick(&mut self, w: &World, sh: &mut Shared) -> u16 {
        if w.tic < self.delay {
            return 0;
        }
        let bits = self.decide(w, sh) & !(self.last & (A | Y));
        let walking = w.players[self.p].state == PlayerState::Free && w.dial(self.p).is_none();
        let released = self.last & UP != 0 && bits & (UP | LEFT | RIGHT) == 0;
        let bits = match walking && released {
            true => bits | DOWN,
            false => bits,
        };
        self.last = bits;
        bits
    }

    fn decide(&mut self, w: &World, sh: &mut Shared) -> u16 {
        let p = self.p;
        match w.players[p].state {
            PlayerState::Free => {}
            PlayerState::Hidden { .. } => return self.hidden(w, sh),
            _ => {
                self.set_job(w, None);
                self.mode = Mode::Work;
                return 0;
            }
        }
        if matches!(self.mode, Mode::Hidden { .. }) {
            self.mode = Mode::Work;
            self.set_job(w, None);
        }
        if w.ctl[p].hide_t > 0 {
            return 0;
        }
        if let Some(d) = w.dial(p) {
            return self.dial(w, sh, d);
        }
        if let Some(n) = w.reading(p) {
            (0..w.locks.len() as u8).filter(|&l| plan::reveals(w, n, l)).for_each(|l| sh.known[l as usize] = true);
            return A;
        }
        if self.threatened(w) {
            self.flee_tic = w.tic;
            if self.mode != Mode::Flee {
                self.mode = Mode::Flee;
                self.set_job(w, None);
            }
        }
        if self.mode == Mode::Flee && self.calm(w) && w.tic.saturating_sub(self.flee_tic) > FLEE_HOLD {
            self.mode = Mode::Work;
            self.set_job(w, None);
        }
        match self.mode {
            Mode::Flee => self.flee(w, sh),
            _ => self.work(w, sh),
        }
    }

    fn threatened(&self, w: &World) -> bool {
        let (me, lg) = (w.players[self.p], w.legend);
        let d2 = (lg.pos - me.pos).len_sq();
        let here = lg.area == me.area;
        let chased = lg.mode == LegendMode::Chase && lg.target == Some(self.p as u8);
        let close = matches!(lg.mode, LegendMode::Patrol) && here && d2 < WATCH && w.sight_clear(lg.pos, me.pos);
        let alert = matches!(lg.mode, LegendMode::Investigate | LegendMode::Search | LegendMode::Chase) && here && d2 < ALERT;
        chased || close || alert
    }

    fn calm(&self, w: &World) -> bool {
        let (me, lg) = (w.players[self.p], w.legend);
        match lg.mode {
            LegendMode::Leaving | LegendMode::Stunned | LegendMode::Grab => true,
            LegendMode::Chase if lg.target == Some(self.p as u8) => false,
            _ => lg.area != me.area || (lg.pos - me.pos).len_sq() > CALM_RANGE,
        }
    }

    fn hidden(&mut self, w: &World, sh: &mut Shared) -> u16 {
        if w.ctl[self.p].hide_t > 0 {
            return 0;
        }
        let (since, calm, patience) = match self.mode {
            Mode::Hidden { since, calm, patience } => (since, calm, patience),
            _ => (w.tic, 0, CALM_TICS + sh.rng.below(PATIENCE) as u16),
        };
        let calm = match self.calm(w) {
            true => calm.saturating_add(1),
            false => 0,
        };
        self.mode = Mode::Hidden { since, calm, patience };
        let lg = w.legend;
        let away = lg.area != w.players[self.p].area || (lg.pos - w.players[self.p].pos).len_sq() > DANGER;
        let bored = w.tic.saturating_sub(since) > HIDE_MAX && lg.mode != LegendMode::Chase && away && !w.sees(self.p);
        match calm >= patience || bored {
            true => A,
            false => 0,
        }
    }

    fn set_job(&mut self, w: &World, job: Option<Job>) {
        if self.job != job {
            self.job = job;
            self.bad.clear();
            self.presses = 0;
            self.unsticks = 0;
            self.creep = 0;
            self.best = FAR;
            self.since = w.tic;
            self.sites = job.map_or(Vec::new(), |j| plan::sites(w, j));
            self.nav_tic = 0;
            self.shape = 0;
        }
    }

    fn fail(&mut self, w: &World, sh: &mut Shared, why: &str) {
        if let Some(j) = self.job {
            let pl = w.players[self.p];
            let what = match j {
                Job::Take(i) => format!("{:?}", w.items[i as usize].place),
                _ => String::new(),
            };
            let sites = self.sites.iter().map(|s| w.grid.xy(s.cell)).collect::<Vec<_>>();
            sh.log.push(format!("t{} p{} {:?} failed ({why}) at {},{} area {} focus {:?} {what} sites {sites:?}", w.tic, self.p, j, pl.pos.x.floor(), pl.pos.y.floor(), pl.area, w.focus(self.p)));
            self.banned.push((j, w.tic + BAN));
        }
        self.set_job(w, None);
    }

    fn here(&self, w: &World) -> usize {
        w.grid.cell_of(w.players[self.p].pos).unwrap_or(0)
    }

    fn reach_from_here(&mut self, w: &World) {
        let here = self.here(w);
        self.reach.from(w, here, self.who);
    }

    fn work(&mut self, w: &World, sh: &mut Shared) -> u16 {
        if self.job.is_some_and(|j| !plan::valid(w, self.p, j, &sh.known) || matches!(j, Job::Hide(_))) {
            self.set_job(w, None);
        }
        if self.job.is_none() || w.tic >= self.plan_tic {
            self.plan_tic = w.tic + REPLAN;
            self.reach_from_here(w);
            self.banned.retain(|&(_, until)| until > w.tic);
            let taken = (0..MAX_PLAYERS).filter(|&q| q != self.p).filter_map(|q| sh.jobs[q]).collect::<Vec<_>>();
            let ask = Ask { reach: &self.reach, known: &sh.known, taken: &taken, banned: &self.banned };
            let job = plan::choose(w, self.p, &ask);
            self.set_job(w, job);
        }
        match self.job {
            Some(_) => self.drive(w, sh, self.run_free(w)),
            None => self.follow(w, sh),
        }
    }

    fn run_free(&self, w: &World) -> bool {
        let (me, lg) = (w.players[self.p], w.legend);
        lg.area != me.area || (lg.pos - me.pos).len_sq() > RUN_FREE
    }

    fn follow(&mut self, w: &World, sh: &mut Shared) -> u16 {
        let me = w.players[self.p];
        let leader = (0..w.count).find(|&q| q != self.p && sh.jobs[q].is_some() && w.players[q].state == PlayerState::Free);
        let Some(q) = leader.filter(|&q| (w.players[q].pos - me.pos).len_sq() > FOLLOW) else { return 0 };
        if w.tic >= self.nav_tic {
            self.nav_tic = w.tic + RENAV / 2;
            let at = w.grid.cell_of(w.players[q].pos).unwrap_or(0);
            self.nav.toward(w, std::iter::once(at), self.who, &|_| 0);
        }
        let here = self.here(w);
        match self.nav.dist[here] {
            0 | FAR => 0,
            _ => self.steer(w, here, false),
        }
    }

    fn renav(&mut self, w: &World) {
        let now = shape(w);
        if now != self.shape || w.tic >= self.nav_tic {
            self.shape = now;
            let lg = w.legend;
            let near = lg.area == w.players[self.p].area && (lg.pos - w.players[self.p].pos).len_sq() < NEAR_LEGEND;
            self.nav_tic = w.tic + if near { RENAV_NEAR } else { RENAV };
            let bad = &self.bad;
            let goals = self.sites.iter().map(|s| s.cell).filter(|c| !bad.contains(c)).collect::<Vec<_>>();
            self.nav.toward(w, goals.into_iter(), self.who, &|c| shadow(w, c));
        }
    }

    fn drive(&mut self, w: &World, sh: &mut Shared, run: bool) -> u16 {
        if self.wiggle > 0 {
            return self.wiggling();
        }
        self.renav(w);
        let here = self.here(w);
        let d = self.nav.dist[here];
        if d == FAR {
            self.fail(w, sh, "unreachable");
            return 0;
        }
        if d < self.best {
            self.best = d;
            self.since = w.tic;
        }
        if d > 0 && w.tic.saturating_sub(self.since) > STALL {
            return self.unstick(w, sh);
        }
        match d {
            0 => self.engage(w, sh, here),
            _ => self.steer(w, here, run),
        }
    }

    fn unstick(&mut self, w: &World, sh: &mut Shared) -> u16 {
        self.unsticks += 1;
        self.since = w.tic;
        self.best = FAR;
        if self.unsticks > UNSTICKS {
            self.fail(w, sh, "stuck");
            return 0;
        }
        self.wiggle = WIGGLE;
        self.turn = match sh.rng.below(2) {
            0 => LEFT,
            _ => RIGHT,
        };
        self.wiggling()
    }

    fn wiggling(&mut self) -> u16 {
        self.wiggle -= 1;
        match self.wiggle > WIGGLE / 2 {
            true => DOWN | self.turn,
            false => UP,
        }
    }

    fn engage(&mut self, w: &World, sh: &mut Shared, here: usize) -> u16 {
        let Some(job) = self.job else { return 0 };
        let focus = w.focus(self.p);
        if plan::wants(job, focus) {
            if let Job::Hide(s) = job {
                if w.sees(self.p) {
                    self.banned.push((Job::Hide(s), w.tic + SEEN_BAN));
                    self.set_job(w, None);
                    self.threat_tic = 0;
                    return UP | B;
                }
            }
            self.presses += 1;
            if self.presses > PRESSES {
                self.fail(w, sh, "no effect");
                return 0;
            }
            return A;
        }
        if plan::harmless(w, focus) || plan::stacked(w, job, focus) {
            return A;
        }
        self.creep += 1;
        if self.creep > CREEP {
            self.creep = 0;
            self.bad.push(here);
            self.nav_tic = 0;
            if self.sites.iter().all(|s| self.bad.contains(&s.cell)) {
                self.fail(w, sh, "no focus");
            }
            return 0;
        }
        let aim = self.sites.iter().find(|s| s.cell == here).and_then(|s| s.aim).unwrap_or(w.grid.centre(here));
        self.approach(w, aim)
    }

    fn approach(&self, w: &World, aim: Vec2) -> u16 {
        let me = w.players[self.p];
        let off = aim - me.pos;
        match off.len_sq() < ARRIVE {
            true => 0,
            false => head(me.heading, off.ang(), UP),
        }
    }

    fn steer(&self, w: &World, here: usize, run: bool) -> u16 {
        let me = w.players[self.p];
        let gait = if run { B } else { 0 };
        match self.nav.next(w, here, self.who) {
            None => 0,
            Some(Hop::Warp(k)) => match w.focus(self.p) {
                Some(Focus::Portal { portal, locked: false }) if portal == k => A,
                f if plan::harmless(w, f) => A,
                _ => self.approach(w, w.grid.centre(here)),
            },
            Some(Hop::Walk(n)) => match w.door_of(n).filter(|&d| !w.door_passable(d)) {
                Some(d) => {
                    let (turn, diff) = turn_to(me.heading, (w.grid.centre(n) - me.pos).ang());
                    match (w.door_dir[d], diff < FACE * 2, w.focus(self.p)) {
                        (true, _, _) => turn,
                        (false, true, Some(Focus::Door { door, .. })) if door as usize == d => A,
                        (false, true, f) if plan::harmless(w, f) => A,
                        (false, true, _) => UP,
                        (false, false, _) => turn,
                    }
                }
                None => {
                    let aim = self.lookahead(w, n);
                    head(me.heading, (aim - me.pos).ang(), UP | gait)
                }
            },
        }
    }

    fn lookahead(&self, w: &World, first: usize) -> Vec2 {
        let me = w.players[self.p].pos;
        let chain = std::iter::successors(Some(first), |&c| match self.nav.next(w, c, self.who) {
            Some(Hop::Walk(n)) if self.nav.dist[c] > 0 && w.door_of(n).is_none_or(|d| w.door_passable(d)) => Some(n),
            _ => None,
        });
        chain
            .take(LOOKAHEAD)
            .map(|c| w.grid.centre(c))
            .take_while(|&at| nav::clear(w, me, at))
            .last()
            .unwrap_or(w.grid.centre(first))
    }

    fn dial(&mut self, w: &World, sh: &mut Shared, d: Dial) -> u16 {
        if self.last != 0 {
            return 0;
        }
        let code = plan::code_of(w, d.lock).unwrap_or_default().bytes().map(|b| b - b'0').collect::<Vec<_>>();
        if d.wrong > 0 || code.len() != d.len as usize {
            sh.log.push(format!("t{} p{} dial for lock {} went wrong", w.tic, self.p, d.lock));
            self.fail(w, sh, "dial");
            return Y;
        }
        let c = d.cursor as usize;
        match (d.digits[c] != code[c], (0..code.len()).any(|i| d.digits[i] != code[i])) {
            (true, _) => match (code[c] + 10 - d.digits[c]) % 10 <= 5 {
                true => UP,
                false => DOWN,
            },
            (false, true) => RIGHT,
            (false, false) => A,
        }
    }

    fn flee(&mut self, w: &World, sh: &mut Shared) -> u16 {
        let here = self.here(w);
        let idle = self.job.is_none() && matches!(self.nav.dist[here], 0 | FAR);
        if w.tic >= self.threat_tic || idle {
            self.threat_tic = w.tic + THREAT_EVERY;
            let at = w.grid.cell_of(w.legend.pos).unwrap_or(0);
            self.threat.from(w, at, Walker::Legend);
            self.reach_from_here(w);
            self.banned.retain(|&(_, until)| until > w.tic);
            let keep = self.job.filter(|&j| matches!(j, Job::Hide(_)) && plan::valid(w, self.p, j, &sh.known) && self.safe_hide(w, j));
            match keep.or_else(|| self.pick_hide(w, sh)) {
                Some(j) => self.set_job(w, Some(j)),
                None if idle || w.tic >= self.run_tic || self.job.is_some() => self.run_away(w),
                None => {}
            }
        }
        match self.job {
            Some(_) => self.drive(w, sh, true),
            None => match self.nav.dist[here] {
                0 | FAR => 0,
                _ => self.steer(w, here, true),
            },
        }
    }

    fn safe_hide(&self, w: &World, j: Job) -> bool {
        !self.banned.iter().any(|&(b, _)| b == j)
            && plan::sites(w, j).iter().any(|s| {
                let (mine, theirs) = (self.reach.dist[s.cell], self.threat.dist[s.cell]);
                mine <= HIDE_REACH && (theirs == FAR || theirs > mine + HIDE_LEAD)
            })
    }

    fn pick_hide(&self, w: &World, sh: &Shared) -> Option<Job> {
        (0..w.map.hides.len() as u8)
            .filter(|&s| plan::valid(w, self.p, Job::Hide(s), &sh.known) && self.safe_hide(w, Job::Hide(s)))
            .filter(|&s| !(0..MAX_PLAYERS).any(|q| q != self.p && sh.jobs[q] == Some(Job::Hide(s))))
            .min_by_key(|&s| (plan::cost(w, Job::Hide(s), &self.reach), s))
            .map(Job::Hide)
    }

    fn run_away(&mut self, w: &World) {
        self.set_job(w, None);
        self.run_tic = w.tic + RUN_COMMIT;
        let (reach, threat) = (&self.reach.dist, &self.threat.dist);
        let lead = |c: usize| threat[c].min(FAR / 2) as i64 - reach[c] as i64;
        let near = (0..w.grid.cells()).filter(|&c| reach[c] <= FLEE_REACH && reach[c] >= FLEE_MIN).collect::<Vec<_>>();
        let ahead = near.iter().copied().filter(|&c| lead(c) > FLEE_LEAD).max_by_key(|&c| (threat[c].min(FAR / 2), std::cmp::Reverse(reach[c]), std::cmp::Reverse(c)));
        let best = ahead.or_else(|| near.iter().copied().max_by_key(|&c| (lead(c), threat[c].min(FAR / 2), std::cmp::Reverse(c))));
        if let Some(c) = best {
            self.nav.toward(w, std::iter::once(c), self.who, &|c| shadow(w, c));
        }
    }
}
