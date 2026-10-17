use super::{BLOOD, DIM, Dir, GOLD, INK, arrow, centered, line, width, wrap};
use crate::render::{Ui2d, rgb565};
use crate::sim::Dial;

const PANEL: u16 = rgb565(22, 20, 18);
const CELL: u16 = rgb565(46, 42, 38);
const PAPER: u16 = rgb565(196, 184, 150);
const PEN: u16 = rgb565(44, 34, 26);

pub fn dial(ui: &mut Ui2d, r: [u16; 4], d: Dial) {
    let len = d.len.clamp(1, 4) as i16;
    let (cw, ch, gap) = (16i16, 20i16, 6i16);
    let w = len * (cw + gap) - gap;
    let (cx, cy) = (r[0] as i16 + r[2] as i16 / 2, r[1] as i16 + r[3] as i16 / 2);
    let (x0, y0) = (cx - w / 2, cy - ch / 2);
    let help = match d.wrong {
        0 => line(format_args!("A: abrir   Y: sair")),
        n => line(format_args!("errou {n}x   Y: sair")),
    };
    let pw = w.max(width(help.as_str(), 1)) + 28;
    ui.rect(cx - pw / 2, y0 - 26, pw, ch + 58, PANEL);
    centered(ui, [r[0], 0, r[2], 0], y0 - 22, "cadeado", DIM, 1);
    (0..len).for_each(|i| {
        let x = x0 + i * (cw + gap);
        let on = i == d.cursor as i16;
        if on {
            ui.rect(x - 1, y0 - 1, cw + 2, ch + 2, GOLD);
            arrow(ui, x + 4, y0 - 9, Dir::Up, GOLD);
            arrow(ui, x + 4, y0 + ch + 2, Dir::Down, GOLD);
        }
        ui.rect(x, y0, cw, ch, CELL);
        let digit = [b'0' + d.digits[i as usize] % 10];
        ui.text(x, y0 + 2, std::str::from_utf8(&digit).unwrap_or("0"), if on { GOLD } else { INK }, 2);
    });
    centered(ui, [r[0], 0, r[2], 0], y0 + ch + 12, help.as_str(), if d.wrong > 0 { BLOOD } else { DIM }, 1);
}

pub fn note(ui: &mut Ui2d, r: [u16; 4], text: &str) {
    let w = (r[2] as i16 - 16).min(320);
    let cols = ((w - 16) / 8).max(8) as usize;
    let lines = wrap(text, cols).count() as i16;
    let h = lines * 10 + 30;
    let (x, y) = (r[0] as i16 + (r[2] as i16 - w) / 2, r[1] as i16 + (r[3] as i16 - h) / 2);
    ui.rect(x + 2, y + 2, w, h, 0);
    ui.rect(x, y, w, h, PAPER);
    wrap(text, cols).enumerate().for_each(|(i, l)| ui.text(x + 8, y + 8 + i as i16 * 10, l, PEN, 1));
    ui.text(x + w - 8 - 72, y + h - 12, "A: fechar", DIM, 1);
}
