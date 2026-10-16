use crate::render::{Ui2d, rgb565};
use crate::sim::{BATTERY_FULL, Phase, TaskKind, World};

const TEXT: u16 = rgb565(206, 200, 182);
const DIM: u16 = rgb565(96, 92, 86);
const SHADOW: u16 = 0;
const BAR: u16 = rgb565(214, 184, 96);
const LOW: u16 = rgb565(176, 48, 40);
const FRAME: u16 = rgb565(40, 38, 36);

fn shadowed(ui: &mut Ui2d, x: i16, y: i16, s: &str, color: u16, scale: u8) {
    ui.text(x + 1, y + 1, s, SHADOW, scale);
    ui.text(x, y, s, color, scale);
}

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

pub fn clock(minutes: u16) -> [u8; 5] {
    let (h, m) = ((minutes / 60 % 24) as u8, (minutes % 60) as u8);
    [b'0' + h / 10, b'0' + h % 10, b':', b'0' + m / 10, b'0' + m % 10]
}

fn night_of(p: Phase) -> Option<u8> {
    match p {
        Phase::Dusk { night } | Phase::Night { night } | Phase::Dawn { night } => Some(night),
        Phase::Won | Phase::Lost => None,
    }
}

fn centered(ui: &mut Ui2d, r: [u16; 4], y: i16, s: &str, color: u16, scale: u8) {
    let w = s.chars().count() as i16 * 8 * scale as i16;
    shadowed(ui, r[0] as i16 + (r[2] as i16 - w) / 2, r[1] as i16 + y, s, color, scale);
}

pub fn draw(ui: &mut Ui2d, world: &World, rect: [u16; 4], player: usize, full: bool) {
    let (x, y) = (rect[0] as i16 + 6, rect[1] as i16 + 5);
    let phase = world.phase();
    let night = night_of(phase).unwrap_or(5);
    let mut label = *b"NOITE 0";
    label[6] = b'0' + night;
    shadowed(ui, x, y, std::str::from_utf8(&label).unwrap_or(""), DIM, 1);
    let time = clock(world.clock_minutes());
    shadowed(ui, x, y + 10, std::str::from_utf8(&time).unwrap_or(""), TEXT, 1);
    let p = world.player(player);
    let level = (p.battery.min(BATTERY_FULL) as u32 * 38 / BATTERY_FULL as u32) as i16;
    ui.rect(x, y + 21, 42, 6, FRAME);
    ui.rect(x + 2, y + 23, level, 2, if level < 8 { LOW } else { BAR });
    if full {
        world
            .tasks()
            .iter()
            .filter(|t| t.active)
            .take(6)
            .enumerate()
            .for_each(|(i, t)| {
                let name = task_name(t.kind);
                let w = name.chars().count() as i16 * 8;
                let tx = (rect[0] + rect[2]) as i16 - 6 - w;
                shadowed(ui, tx, y + i as i16 * 10, name, if t.done { DIM } else { TEXT }, 1);
            });
    }
    let banner = match phase {
        Phase::Dusk { .. } => Some(std::str::from_utf8(&label).unwrap_or("")),
        Phase::Dawn { .. } => Some("AMANHECEU"),
        Phase::Won => Some("VOCÊS VIRAM O SOL"),
        Phase::Lost => Some("A CASA FICOU COM VOCÊS"),
        Phase::Night { .. } => None,
    };
    let scale = if full { 2 } else { 1 };
    banner.into_iter().for_each(|b| centered(ui, rect, rect[3] as i16 / 2 - 8, b, TEXT, scale));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_formats() {
        assert_eq!(&clock(22 * 60 + 5), b"22:05");
        assert_eq!(&clock(3 * 60 + 41), b"03:41");
    }
}
