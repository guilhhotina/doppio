use super::font;

pub const MAX_QUADS: usize = 1024;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct UiVert {
    pub pos: [i16; 2],
    pub uv: [u8; 2],
    pub pad: [u8; 2],
    pub col: [u8; 4],
}

pub struct Ui2d {
    verts: Vec<UiVert>,
}

pub fn rgb(c: u16) -> [u8; 4] {
    let e = |v: u16, bits: u32| (v as u32 * 255 / ((1 << bits) - 1)) as u8;
    [e(c >> 11, 5), e(c >> 5 & 63, 6), e(c & 31, 5), 255]
}

pub const fn rgb565(r: u8, g: u8, b: u8) -> u16 {
    (r as u16 >> 3) << 11 | (g as u16 >> 2) << 5 | b as u16 >> 3
}

impl Default for Ui2d {
    fn default() -> Ui2d {
        Ui2d::new()
    }
}

impl Ui2d {
    pub fn new() -> Ui2d {
        Ui2d { verts: Vec::with_capacity(MAX_QUADS * 4) }
    }

    pub fn clear(&mut self) {
        self.verts.clear();
    }

    pub fn verts(&self) -> &[UiVert] {
        &self.verts
    }

    pub fn quads(&self) -> usize {
        self.verts.len() / 4
    }

    fn quad(&mut self, x: i16, y: i16, w: i16, h: i16, glyph: usize, col: [u8; 4]) {
        if self.verts.len() + 4 > MAX_QUADS * 4 {
            return;
        }
        let [u, v] = font::cell(glyph).map(|c| c as u8);
        let s = font::CELL as u8;
        let (x1, y1) = (x.saturating_add(w), y.saturating_add(h));
        self.verts.extend([
            UiVert { pos: [x, y], uv: [u, v], pad: [0; 2], col },
            UiVert { pos: [x1, y], uv: [u + s, v], pad: [0; 2], col },
            UiVert { pos: [x1, y1], uv: [u + s, v + s], pad: [0; 2], col },
            UiVert { pos: [x, y1], uv: [u, v + s], pad: [0; 2], col },
        ]);
    }

    pub fn text(&mut self, x: i16, y: i16, s: &str, color: u16, scale: u8) {
        let (col, k) = (rgb(color), (font::CELL as i16) * scale.max(1) as i16);
        s.split('\n').enumerate().for_each(|(line, l)| {
            l.chars().enumerate().filter(|(_, c)| *c != ' ').for_each(|(i, c)| {
                self.quad(x + i as i16 * k, y + line as i16 * k, k, k, font::index(c), col);
            });
        });
    }

    pub fn rect(&mut self, x: i16, y: i16, w: i16, h: i16, color: u16) {
        self.quad(x, y, w, h, font::SOLID, rgb(color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_batches_quads_without_spaces() {
        let mut u = Ui2d::new();
        u.text(10, 20, "a b\nçã", 0xffff, 2);
        assert_eq!(u.quads(), 4);
        assert_eq!(u.verts()[8].pos, [10, 36]);
        assert_eq!(u.verts()[0].col, [255; 4]);
        u.clear();
        assert_eq!(u.quads(), 0);
    }

    #[test]
    fn capacity_is_bounded() {
        let mut u = Ui2d::new();
        (0..MAX_QUADS + 10).for_each(|_| u.rect(0, 0, 1, 1, 0));
        assert_eq!(u.quads(), MAX_QUADS);
        assert_eq!(u.verts.capacity(), MAX_QUADS * 4);
    }

    #[test]
    fn color_roundtrip() {
        assert_eq!(rgb(rgb565(255, 0, 255)), [255, 0, 255, 255]);
    }
}
