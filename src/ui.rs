pub mod art;
mod cards;
mod credits;
pub mod cues;
pub mod goal;
mod howto;
mod hud;
mod icons;
pub mod lore;
mod menu;
mod online;
mod overlay;
mod party;
#[cfg(test)]
mod preview;
mod title;

use crate::render::{Ui2d, rgb565, text_width};
use std::fmt;

pub use crate::render::Face::{self, Body, Ink, InkSerif, Serif};
pub use cards::{Card, INTRO_PORTRAIT, card, notice, pause};
pub use cues::Cues;
pub use credits::{PAGES, credits, legend_portrait};
pub use howto::howto;
pub use hud::hud;
pub use menu::{Item, menu, menu_at};
pub use online::{RoomRow, rooms};
pub use party::{Party, PartySeat, party, party_rects};
pub use title::title;

pub const INK: u16 = rgb565(222, 212, 188);
pub const DIM: u16 = rgb565(150, 140, 122);
pub const FAINT: u16 = rgb565(96, 90, 82);
pub const BLOOD: u16 = rgb565(176, 30, 22);
pub const GOLD: u16 = rgb565(226, 190, 100);
pub const SCREEN: [u16; 4] = [0, 0, crate::render::CANVAS_W, crate::render::CANVAS_H];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

pub fn width(s: &str, face: Face) -> i16 {
    text_width(s, face)
}

pub fn label(ui: &mut Ui2d, x: i16, y: i16, s: &str, color: u16, face: Face) {
    ui.text(x, y, s, color, face);
}

pub fn centered(ui: &mut Ui2d, r: [u16; 4], y: i16, s: &str, color: u16, face: Face) {
    label(ui, r[0] as i16 + (r[2] as i16 - width(s, face)) / 2, r[1] as i16 + y, s, color, face);
}

pub fn arrow(ui: &mut Ui2d, x: i16, y: i16, dir: Dir) {
    let (src, fh, fv) = match dir {
        Dir::Up => (art::KEY_UP, false, false),
        Dir::Down => (art::KEY_UP, false, true),
        Dir::Left => (art::KEY_LEFT, false, false),
        Dir::Right => (art::KEY_LEFT, true, false),
    };
    art::sprite(ui, art::LATAO, art::flip(src, fh, fv), x, y - 2);
}

pub fn wrap(s: &str, max: i16, face: Face) -> impl Iterator<Item = &str> {
    let mut rest = s.trim();
    std::iter::from_fn(move || {
        (!rest.is_empty()).then(|| {
            let first = rest.chars().next().map_or(0, char::len_utf8);
            let fits = rest
                .char_indices()
                .scan(0i16, |w, (i, c)| {
                    *w += width(c.encode_utf8(&mut [0; 4]), face);
                    Some((i + c.len_utf8(), *w))
                })
                .take_while(|&(_, w)| w <= max)
                .last()
                .map_or(first, |(i, _)| i.max(first));
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

pub fn paragraph(ui: &mut Ui2d, x: i16, y: i16, s: &str, max: i16, color: u16) -> i16 {
    wrap(s, max, Ink).fold(y, |y, l| {
        label(ui, x, y, l, color, Ink);
        y + Ink.line() + 1
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
    fn wraps_on_pixel_width() {
        let max = width("a senha do cadeado", Body);
        let w: Vec<&str> = wrap("a senha do cadeado é o ano da igreja", max, Body).collect();
        assert_eq!(w[0], "a senha do cadeado");
        assert!(w.iter().all(|l| width(l, Body) <= max));
        assert_eq!(w.join(" "), "a senha do cadeado é o ano da igreja");
        let hard: Vec<&str> = wrap("palavragrandedemais x", width("palavra", Body), Body).collect();
        assert!(hard.len() >= 3 && hard.iter().all(|l| width(l, Body) <= width("palavra", Body)));
        assert_eq!(wrap("  ", 40, Body).count(), 0);
        assert_eq!(wrap("cabe certinho", width("cabe certinho", Body), Body).collect::<Vec<_>>(), ["cabe certinho"]);
        assert!(wrap("ação é ótima também", 40, Serif).all(|l| width(l, Serif) <= 40 || !l.contains(' ')));
    }

    #[test]
    fn arrows_are_single_key_sprites() {
        let mut u = Ui2d::new();
        arrow(&mut u, 0, 0, Dir::Left);
        assert_eq!(u.quads(), 1);
        assert!(width("Noite", Serif) > width("Noite", Body));
    }
}
