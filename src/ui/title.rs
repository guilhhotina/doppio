use super::{BLOOD, DIM, FAINT, INK, SCREEN, centered, shadowed, width};
use crate::render::Ui2d;

const NAME: &str = "DOPPIO";
const SCALE: u8 = 5;

fn noise(t: u32) -> u32 {
    let x = t.wrapping_mul(0x9e37_79b9) ^ t >> 7;
    x ^ x >> 13
}

pub fn title(ui: &mut Ui2d, tick: u32, version: &str) {
    let x = (SCREEN[2] as i16 - width(NAME, SCALE)) / 2;
    let y = 58;
    let n = noise(tick / 3);
    let ghost = tick % 360 > 300 || n % 23 == 0;
    if ghost {
        let dx = (n % 7) as i16 - 3;
        ui.text(x + dx + 2, y + (n >> 3) as i16 % 3 - 1, NAME, BLOOD, SCALE);
    }
    shadowed(ui, x, y, NAME, if n % 41 == 0 { DIM } else { INK }, SCALE);
    centered(ui, SCREEN, y + 50, "cinco noites num sobrado", DIM, 1);
    if tick / 30 % 2 == 0 {
        centered(ui, SCREEN, 160, "aperte A", INK, 1);
    }
    shadowed(ui, SCREEN[2] as i16 - 6 - width(version, 1), SCREEN[3] as i16 - 12, version, FAINT, 1);
}
