use super::art::{self, Button, LOGO_SIZE};
use super::{Body, DIM, FAINT, INK, SCREEN, Serif, centered, label, width};
use crate::render::{Ui2d, WHITE};

fn noise(t: u32) -> u32 {
    let x = t.wrapping_mul(0x9e37_79b9) ^ t >> 7;
    x ^ x >> 13
}

pub fn title(ui: &mut Ui2d, tick: u32, version: &str) {
    let x = (SCREEN[2] as i16 - LOGO_SIZE[0]) / 2;
    let y = 40;
    let n = noise(tick / 3);
    let ghost = tick % 360 > 300 || n % 23 == 0;
    if ghost {
        let dx = (n % 7) as i16 - 3;
        art::logo(ui, x + dx + 2, y + (n >> 3) as i16 % 3 - 1, [150, 24, 18, 120]);
    }
    let flicker = if n % 41 == 0 { [170, 160, 150, 255] } else { WHITE };
    art::logo(ui, x, y, flicker);
    centered(ui, SCREEN, y + LOGO_SIZE[1] + 4, "as lendas de Xique-Xique", DIM, Serif);
    if tick / 30 % 2 == 0 {
        let w = 16 + width("aperte", Body);
        let bx = (SCREEN[2] as i16 - w) / 2;
        art::key_hint(ui, bx, 160, Button::A, "aperte", INK);
    }
    label(ui, SCREEN[2] as i16 - 6 - width(version, Body), SCREEN[3] as i16 - 12, version, FAINT, Body);
}
