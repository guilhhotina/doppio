use super::lore::character;
use super::online::clip;
use super::{DIM, FAINT, GOLD, INK, Item, Line, SCREEN, centered, menu_at, shadowed};
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
        0 | 1 => [[224, 20, 144, 130], [0; 4], [0; 4], [0; 4]],
        2 => [[208, 24, 84, 124], [296, 24, 84, 124], [0; 4], [0; 4]],
        _ => std::array::from_fn(|i| [208 + (i % 2) as u16 * 88, 18 + (i / 2) as u16 * 76, 84, 60]),
    }
}

fn label(r: [u16; 4], n: usize) -> i16 {
    (r[1] + r[3]) as i16 + if n <= 2 { 4 } else { 2 }
}

pub fn party(ui: &mut Ui2d, p: &Party, tick: u32) {
    let (seats, items) = (p.seats, p.items);
    shadowed(ui, 16, 8, p.head, INK, 1);
    shadowed(ui, 16, 20, p.sub, DIM, 1);
    menu_at(ui, 32, 40, 96, items, p.sel, tick);
    let rects = party_rects(seats.len());
    seats.iter().zip(rects.iter()).for_each(|(s, &r)| {
        let name = character(s.character).short;
        centered(ui, [r[0], 0, r[2], 0], label(r, seats.len()), name, if s.here { GOLD } else { INK }, 1);
    });
    let top = 40 + items.len() as i16 * super::menu::ROW;
    if seats.len() > 1 || seats.iter().any(|s| !s.here) {
        seats.iter().enumerate().for_each(|(i, s)| {
            let mut l = Line::new();
            let mark = match (s.here, s.ready) {
                (true, _) => "aqui",
                (false, true) => "pronto",
                (false, false) => "esperando",
            };
            let _ = write!(l, "{} {:<10} {}", i + 1, clip(s.nick, 10), mark);
            shadowed(ui, 16, top + i as i16 * 9, l.as_str(), if s.here { GOLD } else { DIM }, 1);
        });
    }
    let c = character(p.focus);
    shadowed(ui, 16, 167, c.name, GOLD, 1);
    shadowed(ui, 16, 179, c.lore, DIM, 1);
    shadowed(ui, 16, 191, c.perk, INK, 1);
    centered(ui, SCREEN, 205, p.footer, FAINT, 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{CANVAS_H, CANVAS_W};

    #[test]
    fn portraits_stay_right_of_the_menu_and_above_the_details() {
        (1..=4).for_each(|n| {
            party_rects(n).iter().take(n).for_each(|&r| {
                assert!(r[0] >= 200 && r[0] + r[2] <= CANVAS_W && r[2] > 0);
                assert!(label(r, n) + 8 < 167 && r[1] + r[3] < CANVAS_H);
            });
        });
        let g = party_rects(4);
        assert!(g[0][1] + g[0][3] + 12 <= g[2][1]);
    }
}
