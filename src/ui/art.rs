use crate::render::{Face, Ui2d, WHITE, rgb, rgb565};

pub const LATAO: &str = "ui_latao_a";
pub const MOLDURA: &str = "ui_moldura_a";
pub const LOGO: &str = "ui_logo_a";
pub const PAPEL: &str = "ui_papel";

pub const PADLOCK: [u16; 4] = [0, 0, 80, 96];
pub const WHEEL: [u16; 4] = [80, 0, 14, 22];
pub const WHEEL_ON: [u16; 4] = [94, 0, 14, 22];
pub const COMPASS: [u16; 4] = [80, 22, 32, 32];
pub const BATTERY: [u16; 4] = [80, 54, 30, 12];
pub const CANDLES: [[u16; 4]; 3] = [[80, 66, 8, 18], [88, 66, 8, 18], [96, 66, 8, 18]];
pub const CANDLE_OUT: [u16; 4] = [104, 66, 8, 18];
pub const KEY_UP: [u16; 4] = [52, 96, 11, 11];
pub const KEY_LEFT: [u16; 4] = [63, 96, 11, 11];
pub const START: [u16; 4] = [74, 96, 26, 11];
pub const POINTER: [u16; 4] = [100, 96, 12, 11];
const BUTTONS: [[u16; 4]; 4] = [[0, 96, 13, 13], [13, 96, 13, 13], [26, 96, 13, 13], [39, 96, 13, 13]];

pub const TORN_H: [u16; 4] = [0, 0, 128, 6];
pub const TORN_V: [u16; 4] = [0, 8, 6, 56];
pub const TAPE: [u16; 4] = [8, 8, 28, 11];
pub const PIN: [u16; 4] = [36, 8, 8, 8];
pub const PIN_RED: [u16; 4] = [44, 8, 8, 8];
pub const CORNER: [u16; 4] = [52, 8, 10, 10];
pub const STAIN: [u16; 4] = [8, 20, 24, 24];
pub const VIGNETTE: [u16; 4] = [64, 8, 48, 48];
pub const BLOOD: [u16; 4] = [32, 20, 32, 44];
pub const LOGO_SIZE: [i16; 2] = [256, 64];

pub const PEN: u16 = rgb565(52, 36, 26);
pub const PEN_FADED: u16 = rgb565(120, 100, 80);
pub const PEN_RED: u16 = rgb565(150, 26, 20);
pub const PAPER: [u8; 4] = [255, 250, 236, 255];
pub const PHOTO: [u8; 4] = [236, 228, 206, 255];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Button {
    A,
    B,
    X,
    Y,
}

pub fn flip(src: [u16; 4], h: bool, v: bool) -> [u16; 4] {
    let [x, y, w, hh] = src;
    let (x, w) = if h { (x + w, w.wrapping_neg()) } else { (x, w) };
    let (y, hh) = if v { (y + hh, hh.wrapping_neg()) } else { (y, hh) };
    [x, y, w, hh]
}

pub fn sprite(ui: &mut Ui2d, tex: &str, src: [u16; 4], x: i16, y: i16) {
    let size = [src[2] as i16, src[3] as i16].map(|v| if v < 0 { -v } else { v });
    ui.sprite(tex, src, [x, y, size[0], size[1]], WHITE);
}

pub fn button(ui: &mut Ui2d, x: i16, y: i16, b: Button) {
    sprite(ui, LATAO, BUTTONS[b as usize], x, y);
}

pub fn pointer(ui: &mut Ui2d, x: i16, y: i16, tick: u32) {
    let nudge = [0, 1, 2, 1][(tick / 8 % 4) as usize];
    sprite(ui, LATAO, POINTER, x + nudge, y);
}

fn strip(ui: &mut Ui2d, base: [u16; 4], at: [i16; 4], col: [u8; 4], fh: bool, fv: bool) {
    let horizontal = base[2] > base[3];
    let (step, len) = if horizontal { (base[2] as i16, at[2]) } else { (base[3] as i16, at[3]) };
    (0..(len + step - 1) / step).for_each(|i| {
        let part = (len - i * step).min(step);
        let (src, dst) = match horizontal {
            true => ([base[0], base[1], part as u16, base[3]], [at[0] + i * step, at[1], part, at[3]]),
            false => ([base[0], base[1], base[2], part as u16], [at[0], at[1] + i * step, at[2], part]),
        };
        ui.sprite(MOLDURA, flip(src, fh, fv), dst, col);
    });
}

pub fn shadow(ui: &mut Ui2d, r: [i16; 4], d: i16) {
    let [x, y, w, h] = r;
    ui.solid([x + w, y + d, d, h], [0, 0, 0, 150]);
    ui.solid([x + d, y + h, w - d, d], [0, 0, 0, 150]);
}

pub fn paper(ui: &mut Ui2d, r: [i16; 4], col: [u8; 4]) {
    let [x, y, w, h] = r;
    shadow(ui, [x, y, w, h], 3);
    ui.fill(PAPEL, r, col);
    let (eh, ev) = (TORN_H[3] as i16, TORN_V[2] as i16);
    strip(ui, TORN_H, [x, y - eh + 2, w, eh], col, false, false);
    strip(ui, TORN_H, [x, y + h - 2, w, eh], col, true, true);
    strip(ui, TORN_V, [x - ev + 2, y, ev, h], col, false, false);
    strip(ui, TORN_V, [x + w - 2, y, ev, h], col, true, false);
}

pub fn pins(ui: &mut Ui2d, r: [i16; 4], red: bool) {
    let p = if red { PIN_RED } else { PIN };
    sprite(ui, MOLDURA, p, r[0] + 4, r[1] + 2);
    sprite(ui, MOLDURA, p, r[0] + r[2] - 12, r[1] + 2);
}

pub fn tape(ui: &mut Ui2d, x: i16, y: i16) {
    sprite(ui, MOLDURA, TAPE, x, y);
}

pub fn photo(ui: &mut Ui2d, r: [u16; 4], margin: [i16; 2]) {
    let [x, y, w, h] = r.map(|v| v as i16);
    let [m, b] = margin;
    let outer = [x - m, y - m, w + 2 * m, h + m + b];
    shadow(ui, outer, 3);
    ui.fill(PAPEL, [outer[0], outer[1], outer[2], m], PHOTO);
    ui.fill(PAPEL, [outer[0], y + h, outer[2], b], PHOTO);
    ui.fill(PAPEL, [outer[0], y, m, h], PHOTO);
    ui.fill(PAPEL, [x + w, y, m, h], PHOTO);
    ui.solid([x - 1, y - 1, w + 2, 1], rgb(rgb565(70, 60, 48)));
    ui.solid([x - 1, y + h, w + 2, 1], rgb(rgb565(200, 190, 168)));
    [(false, false), (true, false), (false, true), (true, true)].iter().for_each(|&(fh, fv)| {
        let cx = if fh { outer[0] + outer[2] - 8 } else { outer[0] - 2 };
        let cy = if fv { outer[1] + outer[3] - 8 } else { outer[1] - 2 };
        ui.sprite(MOLDURA, flip(CORNER, fh, fv), [cx, cy, 10, 10], WHITE);
    });
}

pub fn logo(ui: &mut Ui2d, x: i16, y: i16, col: [u8; 4]) {
    ui.sprite(LOGO, [0, 0, LOGO_SIZE[0] as u16, LOGO_SIZE[1] as u16], [x, y, LOGO_SIZE[0], LOGO_SIZE[1]], col);
}

pub fn key_hint(ui: &mut Ui2d, x: i16, y: i16, b: Button, label: &str, color: u16) -> i16 {
    button(ui, x, y - 2, b);
    ui.text(x + 16, y, label, color, Face::Body) + 10
}

pub fn vignette(ui: &mut Ui2d, r: [i16; 4], inset: i16, col: [u8; 4]) {
    let [x, y, w, h] = r;
    let s = VIGNETTE[2] as i16;
    [(false, false), (true, false), (false, true), (true, true)].iter().for_each(|&(fh, fv)| {
        let cx = if fh { x + w - s + inset } else { x - inset };
        let cy = if fv { y + h - s + inset } else { y - inset };
        ui.sprite(MOLDURA, flip(VIGNETTE, fh, fv), [cx, cy, s, s], col);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flips_reverse_uvs_and_panels_stay_cheap() {
        assert_eq!(flip([10, 20, 6, 8], true, false), [16, 20, 6u16.wrapping_neg(), 8]);
        let mut ui = Ui2d::new();
        ui.sprite(MOLDURA, flip(CORNER, true, true), [0, 0, 10, 10], WHITE);
        let v = ui.verts();
        assert!(v[0].uv[0] > v[1].uv[0] && v[0].uv[1] > v[3].uv[1]);
        let mut ui = Ui2d::new();
        paper(&mut ui, [40, 30, 300, 150], PAPER);
        assert!(ui.quads() < 48, "{}", ui.quads());
        assert!(ui.verts().iter().all(|v| (30..=350).contains(&v.pos[0]) && (20..=190).contains(&v.pos[1])));
    }
}
