use super::art::{self, Button, PEN, PEN_FADED, PEN_RED};
use super::{Body, DIM, INK, Ink, Item, Line, SCREEN, Serif, centered, label, menu, width};
use crate::render::Ui2d;
use std::fmt::Write;

pub struct RoomRow<'a> {
    pub nick: &'a str,
    pub players: u8,
    pub country: &'a str,
}

const VISIBLE: usize = 7;
const SHEET: [i16; 4] = [36, 34, 312, 110];

pub fn rooms(ui: &mut Ui2d, rows: &[RoomRow], sel: usize, status: Option<&str>, tick: u32) {
    centered(ui, SCREEN, 8, "Salas abertas", INK, Serif);
    art::paper(ui, SHEET, art::PAPER);
    art::pins(ui, SHEET, false);
    let x = SHEET[0] + 20;
    label(ui, x, SHEET[1] + 6, "quem abriu", PEN_FADED, Ink);
    label(ui, x + 184, SHEET[1] + 6, "pessoas", PEN_FADED, Ink);
    label(ui, x + 248, SHEET[1] + 6, "país", PEN_FADED, Ink);
    let first = sel.saturating_sub(VISIBLE - 1).min(rows.len().saturating_sub(VISIBLE));
    rows.iter().enumerate().skip(first).take(VISIBLE).for_each(|(i, r)| {
        let y = SHEET[1] + 20 + (i - first) as i16 * 12;
        let color = if i == sel { PEN_RED } else { PEN };
        if i == sel {
            art::pointer(ui, x - 16, y - 1, tick);
        }
        label(ui, x, y, clip(r.nick, 22), color, Ink);
        let mut n = Line::new();
        let _ = write!(n, "{}/4", r.players.min(4));
        label(ui, x + 184, y, n.as_str(), color, Ink);
        label(ui, x + 248, y, r.country, color, Ink);
    });
    status.iter().for_each(|s| label(ui, SHEET[0] + (SHEET[2] - width(s, Body)) / 2, SHEET[1] + 52, s, PEN_FADED, Ink));
    let extra = [Item::new("Atualizar a lista"), Item::new("Voltar")];
    menu(ui, 140, 160, &extra, sel.wrapping_sub(rows.len()), tick);
    let w = 16 + width("entrar", Body) + 10 + 16 + width("atualizar", Body) + 10 + 16 + width("voltar", Body);
    let x = art::key_hint(ui, (SCREEN[2] as i16 - w) / 2, 204, Button::A, "entrar", DIM);
    let x = art::key_hint(ui, x, 204, Button::Y, "atualizar", DIM);
    art::key_hint(ui, x, 204, Button::B, "voltar", DIM);
}

pub fn clip(s: &str, n: usize) -> &str {
    &s[..s.char_indices().nth(n).map_or(s.len(), |(i, _)| i)]
}
