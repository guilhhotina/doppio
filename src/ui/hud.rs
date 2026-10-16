use super::{DIM, Dir, FAINT, GLYPH, GOLD, INK, Line, arrow, centered, shadowed, width};
use crate::render::{Ui2d, rgb565};
use crate::sim::{BATTERY_FULL, Focus, Phase, TaskKind, World};
use std::fmt::Write;

const LOW: u16 = rgb565(176, 48, 40);
const FRAME: u16 = rgb565(40, 38, 36);
const HINT_TICS: u32 = 450;
const BAR: i16 = 64;
const ROW: i16 = 10;

pub fn task_name(k: TaskKind) -> &'static str {
    match k {
        TaskKind::Window => "Fechar a janela",
        TaskKind::Breaker => "Religar o disjuntor",
        TaskKind::Key => "Achar a chave do porão",
        TaskKind::Tv => "Desligar a televisão",
        TaskKind::Gas => "Fechar o gás",
        TaskKind::Faucet => "Fechar a torneira",
    }
}

fn verb(k: TaskKind) -> &'static str {
    match k {
        TaskKind::Window => "A: fechar janela",
        TaskKind::Breaker => "A: religar disjuntor",
        TaskKind::Key => "A: pegar chave",
        TaskKind::Tv => "A: desligar TV",
        TaskKind::Gas => "A: fechar gás",
        TaskKind::Faucet => "A: fechar torneira",
    }
}

pub fn prompt_text(f: Focus) -> &'static str {
    match f {
        Focus::Door { locked: true, .. } => "trancada",
        Focus::Door { open: false, .. } => "A: abrir porta",
        Focus::Door { open: true, .. } => "A: fechar porta",
        Focus::Task { kind, .. } => verb(kind),
    }
}

pub fn clock(minutes: u16) -> [u8; 5] {
    let (h, m) = ((minutes / 60 % 24) as u8, (minutes % 60) as u8);
    [b'0' + h / 10, b'0' + h % 10, b':', b'0' + m / 10, b'0' + m % 10]
}

fn open_task(world: &World, f: Focus) -> bool {
    match f {
        Focus::Task { task, .. } => world.tasks().get(task as usize).is_some_and(|t| t.active && !t.done),
        Focus::Door { .. } => true,
    }
}

fn prompt(ui: &mut Ui2d, world: &World, rect: [u16; 4], player: usize) {
    let Some(f) = world.focus(player).filter(|&f| open_task(world, f)) else { return };
    let base = rect[3] as i16 - 20;
    centered(ui, rect, base, prompt_text(f), if matches!(f, Focus::Door { locked: true, .. }) { DIM } else { INK }, 1);
    if let Focus::Task { progress: held @ 1.., .. } = f {
        let x = rect[0] as i16 + (rect[2] as i16 - BAR) / 2;
        let y = rect[1] as i16 + base + 11;
        ui.rect(x - 1, y - 1, BAR + 2, 5, FRAME);
        ui.rect(x, y, (held as i32 * BAR as i32 / u16::MAX as i32) as i16, 3, GOLD);
    }
}

fn tasks(ui: &mut Ui2d, world: &World, rect: [u16; 4], x: i16, top: i16) {
    let room_w = rect[2] as i16 - 12;
    world.tasks().iter().filter(|t| t.active).take(6).fold(top, |y, t| {
        let (name, room) = (task_name(t.kind), world.room_title(t.room));
        let color = if t.done { FAINT } else { DIM };
        shadowed(ui, x, y, name, color, 1);
        let dash = width(name, 1) + GLYPH / 2;
        let inline = room.is_empty() || dash + GLYPH * 2 + width(room, 1) <= room_w;
        let (rx, ry) = match inline {
            true => (x + dash, y),
            false => (x + GLYPH, y + ROW),
        };
        if !room.is_empty() {
            ui.rect(rx + 1, ry + 4, 6, 1, color);
            shadowed(ui, rx + GLYPH + GLYPH / 2, ry, room, color, 1);
        }
        ry + ROW + 1
    });
}

fn keys(ui: &mut Ui2d, x: i16, y: i16, dirs: [Dir; 2]) {
    arrow(ui, x, y, dirs[0], INK);
    arrow(ui, x + GLYPH + 1, y, dirs[1], INK);
}

fn hint(ui: &mut Ui2d, rect: [u16; 4]) {
    let text = 2 * GLYPH + 6;
    let w = text + width("interagir/lanterna", 1);
    let x = rect[0] as i16 + (rect[2] as i16 - w) / 2;
    let y = rect[1] as i16 + rect[3] as i16 - 76;
    keys(ui, x, y, [Dir::Up, Dir::Down]);
    shadowed(ui, x + text, y, "andar", INK, 1);
    keys(ui, x, y + 11, [Dir::Left, Dir::Right]);
    shadowed(ui, x + text, y + 11, "virar", INK, 1);
    shadowed(ui, x + 4, y + 22, "A", GOLD, 1);
    shadowed(ui, x + text, y + 22, "interagir/lanterna", INK, 1);
    shadowed(ui, x, y + 33, "(B correr)", DIM, 1);
}

pub fn hud(ui: &mut Ui2d, world: &World, rect: [u16; 4], player: usize) {
    let Phase::Night { night } = world.phase() else { return };
    let (x, y) = (rect[0] as i16 + 6, rect[1] as i16 + 5);
    let time = clock(world.clock_minutes());
    shadowed(ui, x, y, std::str::from_utf8(&time).unwrap_or(""), INK, 1);
    let mut label = Line::new();
    let _ = write!(label, "noite {night}");
    shadowed(ui, x + 48, y, label.as_str(), FAINT, 1);
    let p = world.player(player);
    let level = (p.battery.min(BATTERY_FULL) as u32 * 38 / BATTERY_FULL as u32) as i16;
    ui.rect(x, y + 11, 42, 5, FRAME);
    ui.rect(x + 2, y + 12, level, 3, if level < 8 { LOW } else { GOLD });
    if rect[2] >= 300 || rect[3] >= 200 {
        tasks(ui, world, rect, x, y + 22);
    }
    if night == 1 && world.phase_tic() < HINT_TICS {
        hint(ui, rect);
    }
    prompt(ui, world, rect, player);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_formats() {
        assert_eq!(&clock(22 * 60 + 5), b"22:05");
        assert_eq!(&clock(3 * 60 + 41), b"03:41");
    }

    #[test]
    fn prompts_follow_focus() {
        let door = |locked, open| Focus::Door { door: 0, locked, open };
        assert_eq!(prompt_text(door(true, false)), "trancada");
        assert_eq!(prompt_text(door(false, false)), "A: abrir porta");
        assert_eq!(prompt_text(door(false, true)), "A: fechar porta");
        assert_eq!(prompt_text(Focus::Task { task: 0, kind: TaskKind::Window, progress: 9 }), "A: fechar janela");
    }

    #[test]
    fn hint_fits_a_quarter_view() {
        let w = 2 * GLYPH + 6 + width("interagir/lanterna", 1);
        assert!(w <= 192 - 12);
    }
}
