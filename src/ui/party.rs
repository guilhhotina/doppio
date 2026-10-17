use super::lore::character;
use super::online::clip;
use super::art::{self, PEN, PEN_FADED, PEN_RED};
use super::{Body, DIM, GOLD, INK, Ink, Item, Line, SCREEN, Serif, centered, label, menu_at};
use crate::render::Ui2d;
use std::fmt::Write;

pub struct PartySeat<'a> {
    pub character: u8,
    pub nick: &'a str,
    pub here: bool,
    pub ready: bool,
}

pub struct Party<'a> {
    pub head: &'a str,
    pub sub: &'a str,
    pub seats: &'a [PartySeat<'a>],
    pub items: &'a [Item<'a>],
    pub sel: usize,
    pub focus: u8,
    pub footer: &'a str,
}

pub fn party_rects(n: usize) -> [[u16; 4]; 4] {
    match n {
        0 | 1 => [[238, 18, 122, 124], [0; 4], [0; 4], [0; 4]],
        2 => [[228, 22, 70, 112], [306, 22, 70, 112], [0; 4], [0; 4]],
        _ => std::array::from_fn(|i| [228 + (i % 2) as u16 * 78, 14 + (i / 2) as u16 * 72, 72, 56]),
    }
}

fn caption(r: [u16; 4], n: usize) -> i16 {
    (r[1] + r[3]) as i16 + if n <= 2 { 3 } else { 1 }
}

pub fn party(ui: &mut Ui2d, p: &Party, tick: u32) {
    let (seats, items) = (p.seats, p.items);
    label(ui, 12, 4, p.head, INK, Serif);
    label(ui, 12, 21, p.sub, DIM, Body);
    menu_at(ui, 32, 46, 96, items, p.sel, tick);
    let rects = party_rects(seats.len());
    let margin = if seats.len() <= 2 { [4, 12] } else { [3, 10] };
    seats.iter().zip(rects.iter()).for_each(|(s, &r)| {
        art::photo(ui, r, margin);
        let name = character(s.character).short;
        centered(ui, [r[0], 0, r[2], 0], caption(r, seats.len()), name, if s.here { PEN_RED } else { PEN }, Ink);
    });
    let top = 46 + items.len() as i16 * super::menu::ROW + 4;
    if seats.len() > 1 || seats.iter().any(|s| !s.here) {
        seats.iter().enumerate().for_each(|(i, s)| {
            let mut l = Line::new();
            let mark = match (s.here, s.ready) {
                (true, _) => "aqui",
                (false, true) => "pronto",
                (false, false) => "esperando",
            };
            let _ = write!(l, "{} {:<10} {}", i + 1, clip(s.nick, 10), mark);
            label(ui, 16, top + i as i16 * 10, l.as_str(), if s.here { GOLD } else { DIM }, Body);
        });
    }
    let c = character(p.focus);
    let r = [10, 160, 364, 38];
    art::paper(ui, r, art::PAPER);
    label(ui, r[0] + 8, r[1] + 3, c.name, PEN_RED, Ink);
    label(ui, r[0] + 8, r[1] + 14, c.lore, PEN_FADED, Ink);
    label(ui, r[0] + 8, r[1] + 25, c.perk, PEN, Ink);
    centered(ui, SCREEN, 206, p.footer, DIM, Body);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{CANVAS_H, CANVAS_W};

    #[test]
    fn portraits_stay_right_of_the_menu_and_above_the_details() {
        (1..=4).for_each(|n| {
            party_rects(n).iter().take(n).for_each(|&r| {
                assert!(r[0] >= 224 && r[0] + r[2] + 4 <= CANVAS_W && r[2] > 0);
                assert!(caption(r, n) + 8 < 160 && r[1] + r[3] < CANVAS_H);
            });
        });
        let g = party_rects(4);
        assert!(g[0][1] + g[0][3] + 12 <= g[2][1]);
    }
}
