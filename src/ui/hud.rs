use super::icons::{candles, icon};
use super::lore::{character, item};
use super::overlay::{dial, note};
use super::{DIM, Dir, FAINT, GLYPH, GOLD, INK, Line, arrow, centered, line, shadowed, width};
use crate::map::{LockKind, Map, PortalKind};
use crate::render::{Ui2d, rgb565};
use crate::sim::{BATTERY_FULL, DESMAIOS, Focus, ItemKind, ItemState, Player, PlayerState, TICS_PER_SEC, World};

const LOW: u16 = rgb565(176, 48, 40);
const FRAME: u16 = rgb565(40, 38, 36);
const SENSE_M: f32 = 12.0;
const HINT_TICS: u32 = 15 * TICS_PER_SEC;
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

pub fn compass(dx: f32, dy: f32, heading: f32) -> Option<(i16, i16)> {
    let rel = dx.atan2(dy) - heading;
    ((dx * dx + dy * dy).sqrt() <= SENSE_M).then(|| ((-rel.sin() * 8.0).round() as i16, (-rel.cos() * 8.0).round() as i16))
}

fn sense(ui: &mut Ui2d, world: &World, p: &Player, x: i16, y: i16) {
    let l = world.legend();
    let (dx, dy) = ((l.pos.x - p.pos.x).to_f32(), (l.pos.y - p.pos.y).to_f32());
    let Some((mx, my)) = compass(dx, dy, p.heading.to_rad()).filter(|_| l.area == p.area) else { return };
    (0..8).for_each(|k| {
        let a = k as f32 * std::f32::consts::FRAC_PI_4;
        ui.rect(x + (a.sin() * 8.0).round() as i16, y + (a.cos() * 8.0).round() as i16, 1, 1, FAINT);
    });
    ui.rect(x - 1, y - 1, 2, 2, DIM);
    ui.rect(x + mx - 1, y + my - 1, 3, 3, super::BLOOD);
    arrow(ui, x - 3, y - 15, Dir::Up, FAINT);
}

fn held(ui: &mut Ui2d, world: &World, p: &Player, x: i16, bottom: i16) {
    p.hands.iter().flatten().filter_map(|&i| world.items().get(i as usize)).enumerate().for_each(|(row, s)| {
        let y = bottom - 12 - row as i16 * 12;
        icon(ui, x, y, s.kind, 1);
        shadowed(ui, x + 12, y, item(s.kind), INK, 1);
    });
}

fn battery(ui: &mut Ui2d, p: &Player, x: i16, y: i16) {
    let level = (p.battery.min(BATTERY_FULL) as u32 * 28 / BATTERY_FULL as u32) as i16;
    ui.rect(x, y, 32, 5, FRAME);
    ui.rect(x + 2, y + 1, level, 3, if level < 6 { LOW } else if p.lantern { GOLD } else { DIM });
}

fn hint(ui: &mut Ui2d, rect: [u16; 4]) {
    let text = 2 * GLYPH + 6;
    let w = text + width("pegar/usar/lanterna", 1);
    let x = rect[0] as i16 + (rect[2] as i16 - w) / 2;
    let y = rect[1] as i16 + rect[3] as i16 / 2 - 8;
    arrow(ui, x, y, Dir::Up, INK);
    arrow(ui, x + GLYPH + 1, y, Dir::Down, INK);
    shadowed(ui, x + text, y, "andar", INK, 1);
    arrow(ui, x, y + 11, Dir::Left, INK);
    arrow(ui, x + GLYPH + 1, y + 11, Dir::Right, INK);
    shadowed(ui, x + text, y + 11, "virar", INK, 1);
    shadowed(ui, x + 4, y + 22, "A", GOLD, 1);
    shadowed(ui, x + text, y + 22, "pegar/usar/lanterna", INK, 1);
    shadowed(ui, x, y + 33, "(B correr, START pausa)", DIM, 1);
}

fn state(ui: &mut Ui2d, rect: [u16; 4], p: &Player) -> bool {
    let mid = rect[3] as i16 / 2;
    match p.state {
        PlayerState::Fainted { tics_left } => {
            centered(ui, rect, mid - 10, "desmaiado...", DIM, 1);
            let secs = (tics_left as u32).div_ceil(TICS_PER_SEC);
            centered(ui, rect, mid + 4, line(format_args!("acordando em {secs} s")).as_str(), FAINT, 1);
            true
        }
        PlayerState::Escaped => {
            centered(ui, rect, mid - 10, "Você saiu.", INK, 1);
            centered(ui, rect, mid + 4, "esperando os outros", DIM, 1);
            true
        }
        _ => false,
    }
}

pub fn hud(ui: &mut Ui2d, world: &World, rect: [u16; 4], player: usize, tick: u32) {
    let Some(p) = world.players().get(player) else { return };
    let (x, y) = (rect[0] as i16 + 6, rect[1] as i16 + 5);
    let bottom = rect[1] as i16 + rect[3] as i16;
    candles(ui, x, y, world.desmaios_left(), DESMAIOS, tick);
    battery(ui, p, x, y + 13);
    if p.character == BENEDITA {
        sense(ui, world, p, rect[0] as i16 + rect[2] as i16 - 16, y + 14);
    }
    if state(ui, rect, p) {
        return;
    }
    held(ui, world, p, x, bottom - 2);
    if world.setup().phase == 0 && world.tic() < HINT_TICS {
        hint(ui, rect);
    }
    let reading = world.reading(player).and_then(|n| world.map().notes.get(n as usize));
    match (world.dial(player), reading) {
        (Some(d), _) => dial(ui, rect, d),
        (None, Some(n)) => note(ui, rect, n.text),
        (None, None) => {
            let hands = p.hands.iter().flatten().count().max(1) as i16;
            prompt(world, player).iter().for_each(|l| {
                let dim = !l.as_str().starts_with("A:");
                centered(ui, rect, rect[3] as i16 - 16 - hands * 12, l.as_str(), if dim { DIM } else { INK }, 1);
            });
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
    fn compass_points_relative_to_facing() {
        assert_eq!(compass(0.0, 5.0, 0.0), Some((0, -8)));
        assert_eq!(compass(5.0, 0.0, 0.0), Some((-8, 0)));
        assert_eq!(compass(0.0, -5.0, 0.0), Some((0, 8)));
        assert_eq!(compass(5.0, 0.0, std::f32::consts::FRAC_PI_2), Some((0, -8)));
        assert_eq!(compass(20.0, 0.0, 0.0), None);
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
