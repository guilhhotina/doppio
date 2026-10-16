use super::{DIM, Dir, FAINT, INK, arrow, shadowed, width};
use crate::render::Ui2d;

#[derive(Clone, Copy)]
pub struct Item<'a> {
    pub label: &'a str,
    pub value: Option<&'a str>,
    pub note: Option<&'a str>,
    pub enabled: bool,
}

impl<'a> Item<'a> {
    pub const fn new(label: &'a str) -> Item<'a> {
        Item { label, value: None, note: None, enabled: true }
    }
}

pub const ROW: i16 = 22;

pub fn menu(ui: &mut Ui2d, x: i16, y: i16, items: &[Item], sel: usize, tick: u32) {
    items.iter().enumerate().for_each(|(i, it)| {
        let ry = y + i as i16 * ROW;
        let on = i == sel;
        let color = match (it.enabled, on) {
            (false, _) => FAINT,
            (true, true) => INK,
            (true, false) => DIM,
        };
        if on && tick / 20 % 4 != 3 {
            arrow(ui, x - 12, ry, Dir::Right, color);
        }
        shadowed(ui, x, ry, it.label, color, 1);
        it.value.iter().for_each(|v| {
            let vx = x + 144;
            if on {
                arrow(ui, vx - 12, ry, Dir::Left, DIM);
                arrow(ui, vx + width(v, 1) + 5, ry, Dir::Right, DIM);
            }
            shadowed(ui, vx, ry, v, color, 1);
        });
        it.note.filter(|_| on || !it.enabled).iter().for_each(|n| shadowed(ui, x + 8, ry + 10, n, FAINT, 1));
    });
}
