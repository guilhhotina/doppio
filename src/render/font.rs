pub const SHEET: &str = "ui_font_a";
pub const EXTRA: &str = "ªº°«»ÀÁÂÃÄÇÈÉÊÍÑÓÔÕÖÚÜàáâãäçèéêíñóôõöúü";
const ASCII: usize = 95;
const COUNT: usize = ASCII + 39;
const RECORD: usize = 6;
static METRICS: &[u8] = include_bytes!("../../assets/ui/fonts.bin");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    Body,
    Serif,
    Ink,
    InkSerif,
}

impl Face {
    pub const fn line(self) -> i16 {
        match self {
            Face::Body | Face::Ink => 10,
            Face::Serif | Face::InkSerif => 15,
        }
    }

    const fn ringed(self) -> bool {
        matches!(self, Face::Body | Face::Serif)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Glyph {
    pub src: [u16; 4],
    pub dx: i16,
    pub dy: i16,
    pub adv: i16,
}

pub fn index(c: char) -> usize {
    let c = match c {
        '\u{2013}' | '\u{2014}' | '\u{2212}' => '-',
        '\u{2026}' => '.',
        '\u{2018}' | '\u{2019}' => '\'',
        '\u{201c}' | '\u{201d}' => '"',
        c => c,
    };
    match c as u32 {
        v @ 0x20..=0x7e => (v - 0x20) as usize,
        _ => EXTRA.chars().position(|e| e == c).map_or('?' as usize - 0x20, |i| ASCII + i),
    }
}

pub fn glyph(face: Face, c: char) -> Glyph {
    let at = 2 + (face as usize * COUNT + index(c)) * RECORD;
    let m = &METRICS[at..at + RECORD];
    Glyph { src: [m[0], m[1], m[2], m[3]].map(u16::from), dx: -(face.ringed() as i16), dy: m[4] as i8 as i16, adv: m[5] as i16 }
}

pub fn solid() -> [u16; 2] {
    [METRICS[0] as u16, METRICS[1] as u16]
}

pub fn width(s: &str, face: Face) -> i16 {
    s.chars().map(|c| glyph(face, c).adv).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_cover_both_faces() {
        assert_eq!(EXTRA.chars().count() + ASCII, COUNT);
        assert_eq!(METRICS.len(), 2 + 4 * COUNT * RECORD);
        [Face::Body, Face::Serif, Face::Ink, Face::InkSerif].iter().for_each(|&f| {
            let set = "aãáàâçeéêiíoóôõuúAÃÁÇÉÊÍÓÔÕÚ";
            let g: Vec<Glyph> = set.chars().map(|c| glyph(f, c)).collect();
            assert!(g.iter().all(|v| v.src[2] > 0 && v.src[3] > 0 && v.adv > 0), "{f:?}");
            assert!(g.iter().enumerate().all(|(i, a)| g[i + 1..].iter().all(|b| a.src != b.src)), "{f:?}");
            assert!(g.iter().all(|v| v.src[0] + v.src[2] <= 256 && v.src[1] + v.src[3] <= 256));
            assert_eq!(glyph(f, ' ').src[2], 0);
        });
        assert_eq!(glyph(Face::Body, '\u{2014}'), glyph(Face::Body, '-'));
        assert_eq!(glyph(Face::Body, '€'), glyph(Face::Body, '?'));
        assert_eq!(width("ação", Face::Ink), width("ação", Face::Body));
        assert_eq!(glyph(Face::Ink, 'a').src[2] + 2, glyph(Face::Body, 'a').src[2]);
        assert!(width("Mim", Face::Body) < 24 && width("iii", Face::Body) < width("mmm", Face::Body));
    }
}
