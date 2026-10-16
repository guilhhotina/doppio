mod cards;
mod credits;
mod howto;
mod hud;
mod icons;
pub mod lore;
mod menu;
mod online;
mod overlay;
mod party;
mod title;

use crate::render::{Ui2d, rgb565};
use std::fmt;

pub use cards::{Card, INTRO_PORTRAIT, card, notice, pause};
pub use credits::{PAGES, credits, legend_portrait};
pub use howto::howto;
pub use hud::hud;
pub use menu::{Item, menu, menu_at};
pub use online::{RoomRow, rooms};
pub use party::{Party, PartySeat, party, party_rects};
pub use title::title;

pub const INK: u16 = rgb565(214, 206, 186);
pub const DIM: u16 = rgb565(112, 106, 98);
pub const FAINT: u16 = rgb565(58, 55, 52);
pub const BLOOD: u16 = rgb565(150, 22, 18);
pub const GOLD: u16 = rgb565(214, 184, 96);
pub const SHADOW: u16 = 0;
pub const GLYPH: i16 = 8;
pub const SCREEN: [u16; 4] = [0, 0, crate::render::CANVAS_W, crate::render::CANVAS_H];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

pub fn width(s: &str, scale: u8) -> i16 {
    s.chars().count() as i16 * GLYPH * scale as i16
}

pub fn shadowed(ui: &mut Ui2d, x: i16, y: i16, s: &str, color: u16, scale: u8) {
    ui.text(x + scale as i16, y + scale as i16, s, SHADOW, scale);
    ui.text(x, y, s, color, scale);
}

pub fn centered(ui: &mut Ui2d, r: [u16; 4], y: i16, s: &str, color: u16, scale: u8) {
    shadowed(ui, r[0] as i16 + (r[2] as i16 - width(s, scale)) / 2, r[1] as i16 + y, s, color, scale);
}

pub fn arrow(ui: &mut Ui2d, x: i16, y: i16, dir: Dir, color: u16) {
    (0..4i16).for_each(|i| {
        let (len, off) = (7 - 2 * i, i);
        match dir {
            Dir::Up => ui.rect(x + off, y + 4 - i, len, 1, color),
            Dir::Down => ui.rect(x + off, y + 1 + i, len, 1, color),
            Dir::Left => ui.rect(x + 4 - i, y + off, 1, len, color),
            Dir::Right => ui.rect(x + 1 + i, y + off, 1, len, color),
        }
    });
}

pub fn wrap(s: &str, cols: usize) -> impl Iterator<Item = &str> {
    let mut rest = s.trim();
    std::iter::from_fn(move || {
        (!rest.is_empty()).then(|| {
            let fits = rest.char_indices().nth(cols).map_or(rest.len(), |(i, _)| i);
            let cut = match fits == rest.len() || rest[fits..].starts_with(' ') {
                true => fits,
                false => rest[..fits].rfind(' ').filter(|&i| i > 0).unwrap_or(fits),
            };
            let line = rest[..cut].trim_end();
            rest = rest[cut..].trim_start();
            line
        })
    })
}

pub fn paragraph(ui: &mut Ui2d, x: i16, y: i16, s: &str, cols: usize, color: u16) -> i16 {
    wrap(s, cols).fold(y, |y, l| {
        shadowed(ui, x, y, l, color, 1);
        y + 10
    })
}

pub struct Line {
    buf: [u8; 96],
    len: usize,
}

impl Line {
    pub const fn new() -> Line {
        Line { buf: [0; 96], len: 0 }
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

impl Default for Line {
    fn default() -> Line {
        Line::new()
    }
}

impl fmt::Write for Line {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let room = self.buf.len() - self.len;
        let end = (0..=s.len().min(room)).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0);
        self.buf[self.len..self.len + end].copy_from_slice(&s.as_bytes()[..end]);
        self.len += end;
        Ok(())
    }
}

pub fn line(args: fmt::Arguments) -> Line {
    let mut l = Line::new();
    let _ = fmt::write(&mut l, args);
    l
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_formats_and_truncates_on_char_boundary() {
        assert_eq!(line(format_args!("{} de {}", 2, 4)).as_str(), "2 de 4");
        let long = "é".repeat(60);
        let l = line(format_args!("{long}"));
        assert_eq!(l.as_str().chars().count(), 48);
    }

    #[test]
    fn wraps_on_words() {
        let w: Vec<&str> = wrap("a senha do cadeado é o ano da igreja", 12).collect();
        assert_eq!(w, ["a senha do", "cadeado é o", "ano da", "igreja"]);
        assert_eq!(wrap("palavragrandedemais x", 8).collect::<Vec<_>>(), ["palavrag", "randedem", "ais x"]);
        assert_eq!(wrap("  ", 8).count(), 0);
        assert_eq!(wrap("cabe certinho", 13).collect::<Vec<_>>(), ["cabe certinho"]);
        assert!(wrap("ação é ótima também", 6).all(|l| l.chars().count() <= 6));
    }

    #[test]
    fn arrows_are_seven_pixels() {
        let mut u = Ui2d::new();
        arrow(&mut u, 0, 0, Dir::Left, INK);
        assert_eq!(u.quads(), 4);
        assert_eq!(width("Noite", 2), 80);
    }
}
