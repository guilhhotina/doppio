use super::{DIM, Dir, FAINT, INK, Item, Line, SCREEN, arrow, centered, menu, shadowed};
use crate::render::Ui2d;
use std::fmt::Write;

pub struct RoomRow<'a> {
    pub nick: &'a str,
    pub players: u8,
    pub country: &'a str,
}

const VISIBLE: usize = 7;

pub fn rooms(ui: &mut Ui2d, rows: &[RoomRow], sel: usize, status: Option<&str>, tick: u32) {
    centered(ui, SCREEN, 14, "Salas abertas", INK, 2);
    let x = 56i16;
    shadowed(ui, x, 42, "quem abriu", FAINT, 1);
    shadowed(ui, x + 184, 42, "pessoas", FAINT, 1);
    shadowed(ui, x + 264, 42, "país", FAINT, 1);
    let first = sel.saturating_sub(VISIBLE - 1).min(rows.len().saturating_sub(VISIBLE));
    rows.iter().enumerate().skip(first).take(VISIBLE).for_each(|(i, r)| {
        let y = 56 + (i - first) as i16 * 14;
        let color = if i == sel { INK } else { DIM };
        if i == sel && tick / 20 % 4 != 3 {
            arrow(ui, x - 12, y, Dir::Right, color);
        }
        shadowed(ui, x, y, clip(r.nick, 22), color, 1);
        let mut n = Line::new();
        let _ = write!(n, "{}/4", r.players.min(4));
        shadowed(ui, x + 184, y, n.as_str(), color, 1);
        shadowed(ui, x + 264, y, r.country, color, 1);
    });
    status.iter().for_each(|s| centered(ui, SCREEN, 100, s, DIM, 1));
    let extra = [Item::new("Atualizar a lista"), Item::new("Voltar")];
    menu(ui, x, 160, &extra, sel.wrapping_sub(rows.len()), tick);
    centered(ui, SCREEN, 200, "A entrar   Y atualizar   B voltar", FAINT, 1);
}

pub fn clip(s: &str, n: usize) -> &str {
    &s[..s.char_indices().nth(n).map_or(s.len(), |(i, _)| i)]
}
