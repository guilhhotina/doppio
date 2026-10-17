use super::goal::{self, Goal};
use super::lore::character;
use super::{BLOOD, DIM, GOLD, INK, centered, line, wrap};
use crate::render::Ui2d;
use crate::sim::{Event, Focus, ItemKind, LegendKind, LegendMode, TICS_PER_SEC, World, MAX_PLAYERS};

const SEC: u16 = TICS_PER_SEC as u16;
pub const FLASH: u16 = SEC;
pub const TOAST: u16 = 5 * SEC / 2;
pub const TIP: u16 = 6 * SEC;
pub const MOVE_TIP: u16 = 15 * SEC;
pub const REWARD: u16 = 3 * SEC;
const GOAL_EVERY: u32 = 2 * TICS_PER_SEC;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Toast {
    Saw,
    Lost,
    Opened { open: u8, of: u8 },
    Exit,
    Foto { got: u8, of: u8 },
    Down { player: u8 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tip {
    Move,
    Pause,
    Item,
    Hide,
    Locked,
    Note,
    Dial,
    Run,
    CorpoSeco,
    Cabra,
    Mula,
    Matinta,
    Desmaio,
    Friend,
}

impl Tip {
    pub fn bit(self) -> u32 {
        1 << self as u32
    }

    pub fn text(self) -> &'static str {
        match self {
            Tip::Move => "",
            Tip::Pause => "Segure A longe de tudo: objetivo e itens",
            Tip::Item => "Você leva um item por vez (Jairo leva dois)",
            Tip::Hide => "Escondido, ela só te acha se te viu entrar",
            Tip::Locked => "Trancado? Segure A e veja o objetivo",
            Tip::Note => "Bilhetes guardam pistas e segredos",
            Tip::Dial => "Cima e baixo: número. Lados: outra casa",
            Tip::Run => "Corra (B ou cima duas vezes) e se esconda",
            Tip::CorpoSeco => "O Corpo-seco é lento. Sal na porta o barra",
            Tip::Cabra => "A Cabra cospe fogo de perto: use o balde",
            Tip::Mula => "A Mula não vira enquanto corre: desvie!",
            Tip::Matinta => "O assobio mostra onde a Matinta está",
            Tip::Desmaio => "Cada desmaio apaga uma vela. O item caiu lá",
            Tip::Friend => "Vá até quem desmaiou e aperte A",
        }
    }
}

pub struct Cues {
    flash: [u16; MAX_PLAYERS],
    toast: [Option<(Toast, u16)>; MAX_PLAYERS],
    tip: [Option<(Tip, u16)>; MAX_PLAYERS],
    reward: u16,
    hunted: Option<u8>,
    goal: Goal,
    seen: u32,
    fresh: bool,
}

fn tick<T: Copy>(t: Option<(T, u16)>) -> Option<(T, u16)> {
    t.and_then(|(k, n)| (n > 1).then_some((k, n - 1)))
}

fn focus_tip(f: Focus) -> Option<Tip> {
    match f {
        Focus::Item { .. } => Some(Tip::Item),
        Focus::Hide { .. } => Some(Tip::Hide),
        Focus::Door { locked: true, .. } | Focus::Portal { locked: true, .. } | Focus::Container { locked: true, .. } | Focus::Lock { .. } | Focus::Place { .. } => Some(Tip::Locked),
        Focus::Note { .. } => Some(Tip::Note),
        Focus::Friend { .. } => Some(Tip::Friend),
        _ => None,
    }
}

impl Cues {
    pub fn new(world: &World, seen: u32, local: [bool; MAX_PLAYERS]) -> Cues {
        let mut c = Cues { flash: [0; MAX_PLAYERS], toast: [None; MAX_PLAYERS], tip: [None; MAX_PLAYERS], reward: 0, hunted: None, goal: goal::goal(world), seen, fresh: false };
        (0..MAX_PLAYERS).filter(|&p| local[p]).for_each(|p| c.teach(p, Tip::Move, MOVE_TIP));
        c
    }

    pub fn take_seen(&mut self) -> Option<u32> {
        std::mem::take(&mut self.fresh).then_some(self.seen)
    }

    pub fn seen(&self) -> u32 {
        self.seen
    }

    pub fn goal(&self) -> Goal {
        self.goal
    }

    pub fn tip(&self, p: usize) -> Option<Tip> {
        self.tip.get(p).copied().flatten().map(|(t, _)| t)
    }

    pub fn toast(&self, p: usize) -> Option<Toast> {
        self.toast.get(p).copied().flatten().map(|(t, _)| t)
    }

    pub fn flash(&self, p: usize) -> u16 {
        self.flash.get(p).copied().unwrap_or(0)
    }

    fn teach(&mut self, p: usize, t: Tip, tics: u16) {
        if self.seen & t.bit() == 0 && self.tip[p].is_none() {
            self.tip[p] = Some((t, tics));
            self.seen |= t.bit();
            self.fresh = true;
        }
    }

    fn say(&mut self, p: usize, t: Toast) {
        self.toast[p] = Some((t, TOAST));
    }

    pub fn tic(&mut self, world: &World, local: [bool; MAX_PLAYERS]) {
        let n = world.players().len();
        self.flash = self.flash.map(|f| f.saturating_sub(1));
        self.toast = self.toast.map(tick);
        let ended = self.tip.map(|t| t.is_some_and(|(k, n)| k == Tip::Move && n == 1));
        self.tip = self.tip.map(tick);
        self.reward = self.reward.saturating_sub(1);
        let kind = world.legend().kind;
        let mine = |p: usize| p < n && local[p];
        let mut regoal = world.tic() % GOAL_EVERY == 0;
        world.events().iter().for_each(|&e| match e {
            Event::LegendSaw { player } => {
                let p = player as usize;
                self.flash[p] = FLASH;
                self.hunted = Some(player);
                self.say(p, Toast::Saw);
                if mine(p) {
                    self.teach(p, Tip::Run, TIP);
                    if kind == LegendKind::CorpoSeco {
                        self.teach(p, Tip::CorpoSeco, TIP);
                    }
                }
            }
            Event::Lunge { player } => self.flash[player as usize] = FLASH,
            Event::LegendLost => {
                if let Some(p) = self.hunted.take() {
                    self.say(p as usize, Toast::Lost);
                }
            }
            Event::LegendSpecial | Event::LegendCry => {
                let tip = match (kind, e) {
                    (LegendKind::Cabra, Event::LegendSpecial) => Some(Tip::Cabra),
                    (LegendKind::Mula, Event::LegendSpecial) => Some(Tip::Mula),
                    (LegendKind::Matinta, Event::LegendCry) => Some(Tip::Matinta),
                    _ => None,
                };
                tip.iter().for_each(|&t| (0..n).filter(|&p| mine(p)).for_each(|p| self.teach(p, t, TIP)));
            }
            Event::Unlock { lock } => {
                regoal = true;
                self.reward = REWARD;
                let (open, of) = goal::opened(world);
                let toast = match world.map().exit.lock == lock {
                    true => Toast::Exit,
                    false => Toast::Opened { open: open as u8, of: of as u8 },
                };
                (0..n).for_each(|p| self.say(p, toast));
            }
            Event::Pickup { player, item } => {
                regoal = true;
                if world.items().get(item as usize).is_some_and(|s| s.kind == ItemKind::Foto) {
                    let (got, of) = goal::fotos(world);
                    self.say(player as usize, Toast::Foto { got: got as u8, of: of as u8 });
                }
            }
            Event::Fainted { player } => {
                regoal = true;
                self.hunted = None;
                if mine(player as usize) {
                    self.teach(player as usize, Tip::Desmaio, TIP);
                }
                (0..n).filter(|&q| q != player as usize).for_each(|q| {
                    self.say(q, Toast::Down { player });
                    if mine(q) {
                        self.teach(q, Tip::Friend, TIP);
                    }
                });
            }
            Event::Drop { .. } | Event::Container { .. } | Event::Used { .. } | Event::Portal { .. } | Event::Woke { .. } | Event::DoorMove { .. } => regoal = true,
            _ => {}
        });
        (0..n).filter(|&p| mine(p)).for_each(|p| {
            if ended[p] {
                self.teach(p, Tip::Pause, TIP);
            }
            if world.dial(p).is_some() {
                self.teach(p, Tip::Dial, TIP);
            }
            if let Some(t) = world.focus(p).and_then(focus_tip) {
                self.teach(p, t, TIP);
            }
        });
        if regoal {
            self.goal = goal::goal(world);
        }
    }
}

fn band(ui: &mut Ui2d, r: [u16; 4], inset: i16, thick: i16, color: u16, alpha: u8) {
    let (x, y, w, h) = (r[0] as i16 + inset, r[1] as i16 + inset, r[2] as i16 - 2 * inset, r[3] as i16 - 2 * inset);
    ui.shade(x, y, w, thick, color, alpha);
    ui.shade(x, y + h - thick, w, thick, color, alpha);
    ui.shade(x, y + thick, thick, h - 2 * thick, color, alpha);
    ui.shade(x + w - thick, y + thick, thick, h - 2 * thick, color, alpha);
}

pub fn vignette(ui: &mut Ui2d, r: [u16; 4], color: u16, level: u8) {
    [(0, 210u16), (5, 140), (10, 80), (15, 35)].iter().for_each(|&(inset, a)| band(ui, r, inset, 5, color, (a * level as u16 / 255) as u8));
}

pub fn dread(world: &World, cues: &Cues, p: usize, frame: u32) -> u8 {
    let l = world.legend();
    let chased = l.mode == LegendMode::Chase && l.target == Some(p as u8);
    let pulse = ((frame as f32 * 0.21).sin() * 0.5 + 0.5) * 70.0;
    let base = if chased { 80.0 + pulse } else { 0.0 };
    let hit = cues.flash(p) as f32 / FLASH as f32 * 255.0;
    base.max(hit).min(255.0) as u8
}

pub fn toast_line(world: &World, t: Toast) -> (super::Line, u16, u8) {
    match t {
        Toast::Saw => (line(format_args!("Ela te viu!")), BLOOD, 2),
        Toast::Lost => (line(format_args!("ela te perdeu de vista")), DIM, 1),
        Toast::Opened { open, of } => (line(format_args!("Abriu! ({open}/{of})")), GOLD, 1),
        Toast::Exit => (line(format_args!("A saída está aberta!")), GOLD, 1),
        Toast::Foto { got, of } => (line(format_args!("foto {got} de {of}")), GOLD, 1),
        Toast::Down { player } => {
            let c = world.players().get(player as usize).map_or(0, |p| p.character);
            (line(format_args!("{} desmaiou!", character(c).short)), BLOOD, 1)
        }
    }
}

pub fn draw(ui: &mut Ui2d, world: &World, cues: &Cues, r: [u16; 4], p: usize, frame: u32) {
    let level = dread(world, cues, p, frame);
    if level > 0 {
        vignette(ui, r, BLOOD, level);
    }
    if cues.reward > 0 {
        band(ui, r, 0, 2, GOLD, (cues.reward.min(SEC) as u32 * 200 / SEC as u32) as u8);
    }
    let cols = (r[2] as usize / super::GLYPH as usize).saturating_sub(2).max(8);
    let mid = r[3] as i16 / 3;
    if let Some(t) = cues.toast(p) {
        let (l, color, scale) = toast_line(world, t);
        let scale = if super::width(l.as_str(), scale) > r[2] as i16 - 8 { 1 } else { scale };
        centered(ui, r, mid, l.as_str(), color, scale);
        if matches!(t, Toast::Opened { .. } | Toast::Exit) {
            let g = goal::say(world.map(), cues.goal());
            wrap(g.as_str(), cols).enumerate().for_each(|(i, s)| centered(ui, r, mid + 14 + i as i16 * 10, s, INK, 1));
        }
    }
    if let Some(t) = cues.tip(p).filter(|&t| t != Tip::Move) {
        wrap(t.text(), cols).enumerate().for_each(|(i, s)| centered(ui, r, 30 + i as i16 * 10, s, GOLD, 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::campaign;
    use crate::sim::Setup;

    #[test]
    fn tips_fit_and_are_distinct() {
        let all = [Tip::Pause, Tip::Item, Tip::Hide, Tip::Locked, Tip::Note, Tip::Dial, Tip::Run, Tip::CorpoSeco, Tip::Cabra, Tip::Mula, Tip::Matinta, Tip::Desmaio, Tip::Friend];
        all.iter().for_each(|t| assert!((1..=44).contains(&t.text().chars().count()), "{t:?}"));
        let bits = all.iter().fold(Tip::Move.bit(), |m, t| {
            assert_eq!(m & t.bit(), 0);
            m | t.bit()
        });
        assert_eq!(bits.count_ones(), 14);
    }

    #[test]
    fn first_tip_shows_once_per_save() {
        let w = World::new(campaign()[0], Setup { seed: 1, phase: 0, players: 2, characters: [0, 1, 2, 3] });
        let local = [true, false, false, false];
        let mut c = Cues::new(&w, 0, local);
        assert_eq!((c.tip(0), c.tip(1)), (Some(Tip::Move), None));
        assert_eq!(c.take_seen(), Some(Tip::Move.bit()));
        assert_eq!(c.take_seen(), None);
        (0..MOVE_TIP).for_each(|_| c.tic(&w, local));
        assert_eq!(c.tip(0), Some(Tip::Pause));
        let again = Cues::new(&w, Tip::Move.bit() | Tip::Pause.bit(), local);
        assert_eq!(again.tip(0), None);
    }

    #[test]
    fn dread_follows_the_chase() {
        let w = World::new(campaign()[0], Setup { seed: 1, phase: 0, players: 1, characters: [0, 1, 2, 3] });
        let mut c = Cues::new(&w, u32::MAX, [true, false, false, false]);
        assert_eq!(dread(&w, &c, 0, 0), 0);
        c.flash[0] = FLASH;
        assert_eq!(dread(&w, &c, 0, 0), 255);
        c.flash[0] = FLASH / 2;
        assert!((100..160).contains(&dread(&w, &c, 0, 0)));
    }
}
