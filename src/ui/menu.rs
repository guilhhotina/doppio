use super::art::{self, PAPER, PEN, PEN_FADED, PEN_RED};
use super::{Body, Dir, Ink, arrow, label, width};
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
    menu_at(ui, x, y, 144, items, sel, tick);
}

pub fn panel(x: i16, y: i16, value_x: i16, items: &[Item]) -> [i16; 4] {
    let text = |it: &Item| match it.value {
        Some(v) => value_x + width(v, Body) + 16,
        None => width(it.label, Body).max(it.note.map_or(0, |n| 8 + width(n, Body))),
    };
    let w = items.iter().map(text).max().unwrap_or(0) + 40;
    [x - 24, y - 10, w, items.len() as i16 * ROW + 6]
}

pub fn menu_at(ui: &mut Ui2d, x: i16, y: i16, value_x: i16, items: &[Item], sel: usize, tick: u32) {
    let r = panel(x, y, value_x, items);
    art::paper(ui, r, PAPER);
    art::pins(ui, r, false);
    items.iter().enumerate().for_each(|(i, it)| {
        let ry = y + i as i16 * ROW;
        let on = i == sel;
        let color = match (it.enabled, on) {
            (false, _) => PEN_FADED,
            (true, true) => PEN_RED,
            (true, false) => PEN,
        };
        if on {
            art::pointer(ui, x - 18, ry - 1, tick);
        }
        label(ui, x, ry, it.label, color, Ink);
        it.value.iter().for_each(|v| {
            let vx = x + value_x;
            if on {
                arrow(ui, vx - 14, ry, Dir::Left);
                arrow(ui, vx + width(v, Body) + 4, ry, Dir::Right);
            }
            label(ui, vx, ry, v, color, Ink);
        });
        it.note.filter(|_| on || !it.enabled).iter().for_each(|n| label(ui, x + 8, ry + 10, n, PEN_FADED, Ink));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_wraps_labels_values_and_notes() {
        let items = [Item { note: Some("uma nota bem comprida aqui"), ..Item::new("A") }, Item { value: Some("sim"), ..Item::new("Pública") }];
        let r = panel(100, 50, 96, &items);
        assert_eq!([r[0], r[1], r[3]], [76, 40, 2 * ROW + 6]);
        assert!(r[2] >= 8 + width("uma nota bem comprida aqui", Body) + 40);
        assert!(r[2] >= 96 + width("sim", Body) + 56);
    }
}
