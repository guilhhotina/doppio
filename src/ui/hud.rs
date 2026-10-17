use super::cues::{self, Cues, Tip};
use super::art::{self, BATTERY, Button, COMPASS, LATAO};
use super::icons::{candles, icon};
use super::lore::{character, item};
use super::overlay::{dial, note};
use super::{Body, DIM, Dir, FAINT, GOLD, INK, Line, arrow, centered, label, line, width};
use crate::map::{LockKind, Map, PortalKind};
use crate::render::{Ui2d, rgb565};
use crate::sim::{BATTERY_FULL, DESMAIOS, Focus, ItemKind, ItemState, Player, PlayerState, TICS_PER_SEC, World};

const LOW: u16 = rgb565(176, 48, 40);
const CHARGE: u16 = rgb565(232, 196, 96);
const IDLE: u16 = rgb565(120, 112, 90);
const SENSE_M: f32 = 12.0;
const JAIRO: u8 = 2;
const BENEDITA: u8 = 0;

fn holds(items: &[ItemState], me: &Player, f: impl Fn(usize, ItemKind) -> bool) -> bool {
    me.hands.iter().flatten().any(|&i| items.get(i as usize).is_some_and(|s| f(i as usize, s.kind)))
}

fn hands_full(me: &Player) -> bool {
    let cap = if me.character == JAIRO { 2 } else { 1 };
    me.hands.iter().take(cap).all(Option::is_some)
}

pub fn say(f: Focus, map: &Map, items: &[ItemState], players: &[Player], me: &Player) -> Line {
    let kind = |i: u8| items.get(i as usize).map(|s| s.kind);
    match f {
        Focus::Door { locked: true, .. } => line(format_args!("trancada")),
        Focus::Door { stuck: true, open: false, .. } if me.character != JAIRO => line(format_args!("emperrada")),
        Focus::Door { stuck: true, open: false, .. } => line(format_args!("A: forçar a porta")),
        Focus::Door { open: true, .. } => line(format_args!("A: fechar porta")),
        Focus::Door { .. } => line(format_args!("A: abrir porta")),
        Focus::Item { item: i } => match (kind(i), hands_full(me)) {
            (Some(k), true) => line(format_args!("A: trocar por {}", item(k))),
            (Some(k), false) => line(format_args!("A: pegar {}", item(k))),
            (None, _) => line(format_args!("A: pegar")),
        },
        Focus::Container { locked: true, .. } => line(format_args!("trancado")),
        Focus::Container { .. } => line(format_args!("A: abrir")),
        Focus::Lock { code: true, .. } => line(format_args!("A: mexer no cadeado")),
        Focus::Lock { lock, needs, .. } => {
            let has = match map.locks.get(lock as usize).map(|l| &l.kind) {
                Some(LockKind::Key(k)) => holds(items, me, |i, _| i == *k as usize),
                _ => needs.is_some_and(|n| holds(items, me, |_, k| k == n)),
            };
            match (has, needs) {
                (true, Some(n)) => line(format_args!("A: usar {}", item(n))),
                (false, Some(ItemKind::Chave) | None) => line(format_args!("trancada")),
                (false, Some(n)) => line(format_args!("precisa de {}", item(n))),
                (true, None) => line(format_args!("A: abrir")),
            }
        }
        Focus::Place { needs, .. } if holds(items, me, |_, k| k == needs) => line(format_args!("A: colocar {}", item(needs))),
        Focus::Place { needs, .. } => line(format_args!("falta {}", item(needs))),
        Focus::Hide { .. } if matches!(me.state, PlayerState::Hidden { .. }) => line(format_args!("A: sair")),
        Focus::Hide { .. } => line(format_args!("A: esconder")),
        Focus::Portal { locked: true, .. } => line(format_args!("trancado")),
        Focus::Portal { portal, .. } => match map.portals.get(portal as usize) {
            Some(p) => match p.kind {
                PortalKind::Escada | PortalKind::EscadaMao => {
                    let up = map.area_at(p.to_x, p.to_y) > map.area_at(p.x, p.y);
                    line(format_args!("A: {} escada", if up { "subir" } else { "descer" }))
                }
                PortalKind::Bueiro => line(format_args!("A: entrar no bueiro")),
                PortalKind::Portao => line(format_args!("A: passar o portão")),
                PortalKind::JanelaAlta => line(format_args!("A: pular a janela")),
            },
            None => line(format_args!("A: passar")),
        },
        Focus::Note { .. } => line(format_args!("A: ler")),
        Focus::Friend { player } => {
            let c = players.get(player as usize).map_or(0, |p| p.character);
            line(format_args!("A: acordar {}", character(c).short))
        }
        Focus::Use { item: i } => line(format_args!("A: usar {}", kind(i).map_or("", item))),
    }
}

fn prompt(world: &World, player: usize) -> Option<Line> {
    let p = world.players().get(player)?;
    world.focus(player).map(|f| say(f, world.map(), world.items(), world.players(), p))
}

fn bearing(dx: f32, dy: f32, heading: f32, r: f32) -> (i16, i16) {
    let rel = dx.atan2(dy) - heading;
    ((-rel.sin() * r).round() as i16, (-rel.cos() * r).round() as i16)
}

fn dial_ring(ui: &mut Ui2d, x: i16, y: i16, to: (i16, i16), color: u16, size: i16) {
    let c = COMPASS[2] as i16;
    art::sprite(ui, LATAO, COMPASS, x - c / 2, y - c / 2);
    (1..4).for_each(|s| ui.rect(x + to.0 * s / 4, y + to.1 * s / 4, 1, 1, color));
    ui.rect(x + to.0 - size / 2, y + to.1 - size / 2, size, size, color);
}

fn sense(ui: &mut Ui2d, world: &World, p: &Player, x: i16, y: i16, tick: u32) {
    let l = world.legend();
    let (dx, dy) = ((l.pos.x - p.pos.x).to_f32(), (l.pos.y - p.pos.y).to_f32());
    let d = (dx * dx + dy * dy).sqrt();
    if l.area != p.area || d > SENSE_M {
        return;
    }
    let period = (6.0 + d * 2.0) as u32;
    let size = if tick % period < period / 2 { 4 } else { 3 };
    dial_ring(ui, x, y, bearing(dx, dy, p.heading.to_rad(), 9.0), super::BLOOD, size);
}

fn rescue(ui: &mut Ui2d, world: &World, me: usize, r: [u16; 4], tick: u32) {
    let p = world.players()[me];
    let down = world.players().iter().enumerate().filter(|&(q, f)| q != me && matches!(f.state, PlayerState::Fainted { .. } | PlayerState::Grabbed));
    let Some((_, f)) = down.min_by_key(|(_, f)| (f.pos - p.pos).len_sq()) else { return };
    let (dx, dy) = ((f.pos.x - p.pos.x).to_f32(), (f.pos.y - p.pos.y).to_f32());
    let (x, y) = (r[0] as i16 + r[2] as i16 - 20, r[1] as i16 + 60);
    let same = f.area == p.area;
    let to = if same { bearing(dx, dy, p.heading.to_rad(), 9.0) } else { (0, 0) };
    dial_ring(ui, x, y, to, GOLD, if tick % 30 < 15 { 4 } else { 3 });
    let msg = match same {
        true => line(format_args!("acorde {} {} m", character(f.character).short, (dx * dx + dy * dy).sqrt().round() as i32)),
        false => line(format_args!("acorde {}: outro andar", character(f.character).short)),
    };
    label(ui, r[0] as i16 + r[2] as i16 - 4 - width(msg.as_str(), Body), y + 18, msg.as_str(), GOLD, Body);
}

fn held(ui: &mut Ui2d, world: &World, p: &Player, x: i16, bottom: i16, size: i16) {
    p.hands.iter().flatten().filter_map(|&i| world.items().get(i as usize)).enumerate().for_each(|(row, s)| {
        let y = bottom - size - row as i16 * (size + 2);
        icon(ui, x, y, s.kind, size);
        label(ui, x + size + 2, y + (size - 8) / 2, item(s.kind), INK, Body);
    });
}

fn battery(ui: &mut Ui2d, p: &Player, x: i16, y: i16) {
    art::sprite(ui, LATAO, BATTERY, x, y);
    let cells = (p.battery.min(BATTERY_FULL) as u32 * 5).div_ceil(BATTERY_FULL as u32) as i16;
    let color = match (cells, p.lantern) {
        (0..=1, _) => LOW,
        (_, true) => CHARGE,
        (_, false) => IDLE,
    };
    (0..cells).for_each(|i| ui.rect(x + 3 + i * 4, y + 3, 3, 6, color));
}

pub fn prompt_line(ui: &mut Ui2d, r: [u16; 4], y: i16, l: &str) {
    match l.strip_prefix("A: ") {
        Some(rest) => {
            let w = 16 + width(rest, Body);
            art::key_hint(ui, r[0] as i16 + (r[2] as i16 - w) / 2, r[1] as i16 + y, Button::A, rest, INK);
        }
        None => centered(ui, r, y, l, DIM, Body),
    }
}

fn hint(ui: &mut Ui2d, rect: [u16; 4]) {
    let w = 28 + width("pegar/usar/lanterna", Body);
    let x = rect[0] as i16 + (rect[2] as i16 - w) / 2;
    let y = rect[1] as i16 + rect[3] as i16 / 6;
    let tx = x + 28;
    arrow(ui, x, y, Dir::Up);
    arrow(ui, x + 12, y, Dir::Down);
    label(ui, tx, y, "andar", INK, Body);
    arrow(ui, x, y + 14, Dir::Left);
    arrow(ui, x + 12, y + 14, Dir::Right);
    label(ui, tx, y + 14, "virar", INK, Body);
    art::button(ui, x + 5, y + 26, Button::A);
    label(ui, tx, y + 28, "pegar/usar/lanterna", INK, Body);
    art::button(ui, x + 5, y + 40, Button::B);
    label(ui, tx, y + 42, "correr (ou cima 2x)", INK, Body);
    label(ui, tx, y + 56, "segure A: pausa", DIM, Body);
}

fn state(ui: &mut Ui2d, rect: [u16; 4], p: &Player) -> bool {
    let mid = rect[3] as i16 / 2;
    match p.state {
        PlayerState::Fainted { tics_left } => {
            centered(ui, rect, mid - 10, "desmaiado...", DIM, Body);
            let secs = (tics_left as u32).div_ceil(TICS_PER_SEC);
            centered(ui, rect, mid + 4, line(format_args!("acordando em {secs} s")).as_str(), FAINT, Body);
            true
        }
        PlayerState::Escaped => {
            centered(ui, rect, mid - 10, "Você saiu.", INK, Body);
            centered(ui, rect, mid + 4, "esperando os outros", DIM, Body);
            true
        }
        _ => false,
    }
}

pub fn hud(ui: &mut Ui2d, world: &World, cues: &Cues, rect: [u16; 4], player: usize, tick: u32) {
    let Some(p) = world.players().get(player) else { return };
    let (x, y) = (rect[0] as i16 + 6, rect[1] as i16 + 5);
    let bottom = rect[1] as i16 + rect[3] as i16;
    candles(ui, x, y, world.desmaios_left(), DESMAIOS, tick);
    battery(ui, p, x, y + 21);
    if p.character == BENEDITA {
        sense(ui, world, p, rect[0] as i16 + rect[2] as i16 - 20, y + 16, tick);
    }
    if state(ui, rect, p) {
        return;
    }
    rescue(ui, world, player, rect, tick);
    let size = if rect[3] >= 200 { 32 } else { 16 };
    held(ui, world, p, x, bottom - 4, size);
    let reading = world.reading(player).and_then(|n| world.map().notes.get(n as usize));
    let dialing = world.dial(player);
    let overlay = dialing.is_some() || reading.is_some();
    if !overlay {
        cues::draw(ui, world, cues, rect, player, tick);
    }
    if cues.tip(player) == Some(Tip::Move) && !overlay && cues.toast(player).is_none() {
        hint(ui, rect);
    }
    match (dialing, reading) {
        (Some(d), _) => dial(ui, rect, d),
        (None, Some(n)) => note(ui, rect, n.text),
        (None, None) => {
            let hands = p.hands.iter().flatten().count() as i16;
            let lift = if size == 32 && hands > 0 { 6 } else { hands * (size + 2) };
            prompt(world, player).iter().for_each(|l| prompt_line(ui, rect, rect[3] as i16 - 16 - lift, l.as_str()));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sim::ItemPlace;

    pub const SAMPLE: &str = include_str!("sample.map");
    use super::*;
    use crate::fx::{Ang, Vec2};
    use crate::sim::Anim;

    fn player(character: u8, hands: [Option<u8>; 2], state: PlayerState) -> Player {
        Player {
            character,
            pos: Vec2::ZERO,
            heading: Ang::ZERO,
            area: 0,
            anim: Anim::Idle,
            anim_tic: 0,
            lantern: false,
            battery: BATTERY_FULL,
            hands,
            state,
            fade: 0,
        }
    }

    #[test]
    fn bearing_points_relative_to_facing() {
        assert_eq!(bearing(0.0, 5.0, 0.0, 8.0), (0, -8));
        assert_eq!(bearing(5.0, 0.0, 0.0, 8.0), (-8, 0));
        assert_eq!(bearing(0.0, -5.0, 0.0, 8.0), (0, 8));
        assert_eq!(bearing(5.0, 0.0, std::f32::consts::FRAC_PI_2, 11.0), (0, -11));
    }

    #[test]
    fn prompts_follow_focus_and_hands() {
        let map = crate::map::parse(SAMPLE).unwrap_or_else(|e| panic!("{e}"));
        let held = |p: u8| ItemPlace::Held { player: p };
        let items = [
            ItemState { kind: ItemKind::Alicate, place: ItemPlace::Floor { pos: Vec2::ZERO, area: 0 } },
            ItemState { kind: ItemKind::Chave, place: held(0) },
            ItemState { kind: ItemKind::PeDeCabra, place: held(1) },
        ];
        let free = player(1, [None, None], PlayerState::Free);
        let keyed = player(1, [Some(1), None], PlayerState::Free);
        let jairo = player(JAIRO, [Some(2), None], PlayerState::Free);
        let hidden = player(0, [None, None], PlayerState::Hidden { spot: 0 });
        let players = [keyed, jairo, free, hidden];
        let s = |f, p: &Player| say(f, &map, &items, &players, p).as_str().to_string();
        let door = |locked, stuck, open| Focus::Door { door: 0, locked, stuck, open };
        assert_eq!(s(door(true, false, false), &free), "trancada");
        assert_eq!(s(door(false, true, false), &free), "emperrada");
        assert_eq!(s(door(false, true, false), &jairo), "A: forçar a porta");
        assert_eq!(s(door(false, false, false), &free), "A: abrir porta");
        assert_eq!(s(door(false, false, true), &free), "A: fechar porta");
        assert_eq!(s(Focus::Item { item: 0 }, &free), "A: pegar alicate");
        assert_eq!(s(Focus::Item { item: 0 }, &keyed), "A: trocar por alicate");
        assert_eq!(s(Focus::Item { item: 0 }, &jairo), "A: pegar alicate");
        assert_eq!(s(Focus::Hide { spot: 0 }, &free), "A: esconder");
        assert_eq!(s(Focus::Hide { spot: 0 }, &hidden), "A: sair");
        assert_eq!(s(Focus::Note { note: 0 }, &free), "A: ler");
        assert_eq!(s(Focus::Friend { player: 1 }, &free), "A: acordar Jairo");
        assert_eq!(s(Focus::Lock { lock: 99, needs: Some(ItemKind::Alicate), code: false }, &free), "precisa de alicate");
        assert_eq!(s(Focus::Lock { lock: 99, needs: Some(ItemKind::PeDeCabra), code: false }, &jairo), "A: usar pé de cabra");
        assert_eq!(s(Focus::Lock { lock: 99, needs: Some(ItemKind::Chave), code: false }, &free), "trancada");
        assert_eq!(s(Focus::Lock { lock: 0, needs: None, code: true }, &free), "A: mexer no cadeado");
        assert_eq!(s(Focus::Place { lock: 0, needs: ItemKind::Fusivel }, &free), "falta fusível");
        assert_eq!(s(Focus::Use { item: 2 }, &jairo), "A: usar pé de cabra");
        assert_eq!(s(Focus::Container { prop: 0, locked: false }, &free), "A: abrir");
        assert_eq!(s(Focus::Portal { portal: 0, locked: true }, &free), "trancado");
        let LockKind::Key(key) = map.locks[0].kind else { panic!("bau_l is a key lock") };
        let mine = [ItemState { kind: ItemKind::Chave, place: held(0) }; 4];
        let with_key = player(1, [Some(key), None], PlayerState::Free);
        let wrong_key = player(1, [Some(key + 1), None], PlayerState::Free);
        let lock = |p: &Player| say(Focus::Lock { lock: 0, needs: Some(ItemKind::Chave), code: false }, &map, &mine, &players, p);
        assert_eq!(lock(&with_key).as_str(), "A: usar chave");
        assert_eq!(lock(&wrong_key).as_str(), "trancada");
        assert_eq!(s(Focus::Portal { portal: 0, locked: false }, &free), "A: subir escada");
        assert_eq!(s(Focus::Portal { portal: 1, locked: false }, &free), "A: descer escada");
    }
}
