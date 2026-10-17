use super::font::{self, Face};
use super::texture;

pub const MAX_QUADS: usize = 1024;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct UiVert {
    pub pos: [i16; 2],
    pub uv: [u16; 2],
    pub col: [u8; 4],
}

pub struct Ui2d {
    verts: Vec<UiVert>,
    tiles: Vec<[u8; 4]>,
}

pub fn rgb(c: u16) -> [u8; 4] {
    let e = |v: u16, bits: u32| (v as u32 * 255 / ((1 << bits) - 1)) as u8;
    [e(c >> 11, 5), e(c >> 5 & 63, 6), e(c & 31, 5), 255]
}

pub const fn rgb565(r: u8, g: u8, b: u8) -> u16 {
    (r as u16 >> 3) << 11 | (g as u16 >> 2) << 5 | b as u16 >> 3
}

pub const WHITE: [u8; 4] = [255; 4];

impl Default for Ui2d {
    fn default() -> Ui2d {
        Ui2d::new()
    }
}

impl Ui2d {
    pub fn new() -> Ui2d {
        Ui2d { verts: Vec::with_capacity(MAX_QUADS * 4), tiles: texture::layout() }
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

    fn tile(&self, tex: &str) -> [u16; 4] {
        let t = self.tiles[texture::id(tex) as usize];
        let c = texture::CELL as u16;
        [t[0] as u16 * c, t[1] as u16 * c, t[2] as u16 * c, t[3] as u16 * c]
    }

    pub fn size(&self, tex: &str) -> [i16; 2] {
        let t = self.tile(tex);
        [t[2] as i16, t[3] as i16]
    }

    pub fn quad(&mut self, dst: [i16; 4], src: [u16; 4], col: [u8; 4]) {
        if self.verts.len() + 4 > MAX_QUADS * 4 || dst[2] <= 0 || dst[3] <= 0 {
            return;
        }
        let [x, y, w, h] = dst;
        let (x1, y1) = (x.saturating_add(w), y.saturating_add(h));
        let (u0, v0, u1, v1) = (src[0], src[1], src[0].wrapping_add(src[2]), src[1].wrapping_add(src[3]));
        self.verts.extend([
            UiVert { pos: [x, y], uv: [u0, v0], col },
            UiVert { pos: [x1, y], uv: [u1, v0], col },
            UiVert { pos: [x1, y1], uv: [u1, v1], col },
            UiVert { pos: [x, y1], uv: [u0, v1], col },
        ]);
    }

    pub fn sprite(&mut self, tex: &str, src: [u16; 4], dst: [i16; 4], col: [u8; 4]) {
        let o = self.tile(tex);
        self.quad(dst, [o[0].wrapping_add(src[0]), o[1].wrapping_add(src[1]), src[2], src[3]], col);
    }

    pub fn blit(&mut self, tex: &str, src: [u16; 4], x: i16, y: i16) {
        self.sprite(tex, src, [x, y, src[2] as i16, src[3] as i16], WHITE);
    }

    pub fn fill(&mut self, tex: &str, dst: [i16; 4], col: [u8; 4]) {
        let t = self.tile(tex);
        let (tw, th) = (t[2] as i16, t[3] as i16);
        let [x, y, w, h] = dst;
        (0..(h + th - 1) / th).flat_map(|j| (0..(w + tw - 1) / tw).map(move |i| (i, j))).for_each(|(i, j)| {
            let (cw, ch) = ((w - i * tw).min(tw), (h - j * th).min(th));
            self.quad([x + i * tw, y + j * th, cw, ch], [t[0], t[1], cw as u16, ch as u16], col);
        });
    }

    pub fn text(&mut self, x: i16, y: i16, s: &str, color: u16, face: Face) -> i16 {
        self.ink(x, y, s, rgb(color), face)
    }

    pub fn ink(&mut self, x: i16, y: i16, s: &str, col: [u8; 4], face: Face) -> i16 {
        let o = self.tile(font::SHEET);
        s.split('\n').enumerate().fold(x, |_, (line, l)| {
            let ly = y + line as i16 * face.line();
            l.chars().fold(x, |pen, c| {
                let g = font::glyph(face, c);
                let [sx, sy, w, h] = g.src;
                self.quad([pen + g.dx, ly + g.dy, w as i16, h as i16], [o[0] + sx, o[1] + sy, w, h], col);
                pen + g.adv
            })
        })
    }

    pub fn rect(&mut self, x: i16, y: i16, w: i16, h: i16, color: u16) {
        self.solid([x, y, w, h], rgb(color));
    }

    pub fn shade(&mut self, x: i16, y: i16, w: i16, h: i16, color: u16, alpha: u8) {
        let [r, g, b, _] = rgb(color);
        self.solid([x, y, w, h], [r, g, b, alpha]);
    }

    pub fn solid(&mut self, dst: [i16; 4], col: [u8; 4]) {
        let o = self.tile(font::SHEET);
        let s = font::solid();
        self.quad(dst, [o[0] + s[0] + 2, o[1] + s[1] + 2, 1, 1], col);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_batches_quads_without_spaces() {
        let mut u = Ui2d::new();
        let end = u.text(10, 20, "a b\nçã", 0xffff, Face::Body);
        assert_eq!(u.quads(), 4);
        assert_eq!(u.verts()[8].pos[1], 20 + Face::Body.line() - 1);
        assert_eq!(u.verts()[0].col, [255; 4]);
        assert_eq!(end, 10 + font::width("çã", Face::Body));
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
    fn fill_tiles_partial_edges() {
        let mut u = Ui2d::new();
        let [tw, th] = u.size("bilhete");
        u.fill("bilhete", [5, 7, tw * 2 + 3, th], WHITE);
        assert_eq!(u.quads(), 3);
        let last = &u.verts()[8..12];
        assert_eq!(last[1].pos[0] - last[0].pos[0], 3);
        assert_eq!(last[1].uv[0] - last[0].uv[0], 3);
    }

    #[test]
    fn color_roundtrip() {
        assert_eq!(rgb(rgb565(255, 0, 255)), [255, 0, 255, 255]);
    }
}
