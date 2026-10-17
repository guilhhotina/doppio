use super::lore::{LEGENDS, legend};
use super::art::{self, Button, PEN, PEN_FADED, PEN_RED};
use super::{Body, DIM, INK, Ink, InkSerif, SCREEN, Serif, centered, label, line, paragraph, width};
use crate::render::Ui2d;

pub const PAGES: usize = 1 + LEGENDS.len();
pub const LEGEND_RECT: [u16; 4] = [20, 24, 104, 146];
const SOURCES: &str = include_str!("../../tools/sources.txt");

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Tally {
    pub total: u32,
    pub polyhaven: u32,
    pub commons: u32,
    pub own: u32,
}

pub fn tally(src: &str) -> Tally {
    src.lines().filter_map(|l| l.split_whitespace().nth(1)).fold(Tally::default(), |t, s| Tally {
        total: t.total + 1,
        polyhaven: t.polyhaven + s.contains("polyhaven.com") as u32,
        commons: t.commons + s.contains("wikimedia.org") as u32,
        own: t.own + !s.contains("http") as u32,
    })
}

pub fn legend_portrait(page: usize) -> Option<([u16; 4], u8)> {
    page.checked_sub(1).and_then(|i| LEGENDS.get(i)).map(|&k| (LEGEND_RECT, 4 + k as u8))
}

fn about(ui: &mut Ui2d) {
    centered(ui, SCREEN, 6, "Créditos", INK, Serif);
    let r = [28, 34, 328, 136];
    art::paper(ui, r, art::PAPER);
    art::pins(ui, r, false);
    let t = tally(SOURCES);
    let rows = [
        (line(format_args!("Texturas: {} ao todo", t.total)), PEN),
        (line(format_args!("  Poly Haven, CC0: {}", t.polyhaven)), PEN_FADED),
        (line(format_args!("  Wikimedia Commons, domínio público: {}", t.commons)), PEN_FADED),
        (line(format_args!("  feitas por código para o jogo, CC0: {}", t.own)), PEN_FADED),
        (line(format_args!("Letras: Not Jam Old Style (CC0), IM Fell (OFL)")), PEN),
        (line(format_args!("Modelos, mapas e sons: feitos para o jogo")), PEN),
        (line(format_args!("Lendas: folclore brasileiro, contado de novo")), PEN),
        (line(format_args!("Feito para as TVs ZaperOS (libretro)")), PEN_RED),
    ];
    rows.iter().enumerate().for_each(|(i, (l, c))| {
        label(ui, r[0] + 14, r[1] + 10 + i as i16 * 15, l.as_str(), *c, Ink);
    });
}

fn lore(ui: &mut Ui2d, i: usize) {
    let l = legend(LEGENDS[i]);
    art::photo(ui, LEGEND_RECT, [5, 14]);
    let r = [146, 24, 222, 150];
    art::paper(ui, r, art::PAPER);
    art::tape(ui, r[0] + r[2] / 2 - 14, r[1] - 6);
    label(ui, r[0] + 10, r[1] + 8, "Lendas de Xique-Xique", PEN_FADED, Ink);
    label(ui, r[0] + 10, r[1] + 20, l.name, PEN_RED, InkSerif);
    paragraph(ui, r[0] + 10, r[1] + 42, l.lore, r[2] - 20, PEN);
}

pub fn credits(ui: &mut Ui2d, page: usize) {
    match page {
        0 => about(ui),
        p => lore(ui, (p - 1).min(LEGENDS.len() - 1)),
    }
    let dots = PAGES as i16 * 10;
    let x0 = (SCREEN[2] as i16 - dots) / 2;
    (0..PAGES).for_each(|p| {
        let on = p == page.min(PAGES - 1);
        art::sprite(ui, art::MOLDURA, if on { art::PIN_RED } else { art::PIN }, x0 + p as i16 * 10, 184);
    });
    let hint = "próxima";
    let w = 16 + width(hint, Body) + 10 + 16 + width("voltar", Body);
    let x = art::key_hint(ui, (SCREEN[2] as i16 - w) / 2, 200, Button::A, hint, DIM);
    art::key_hint(ui, x, 200, Button::B, "voltar", DIM);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tallies_sources() {
        let t = tally("a https://polyhaven.com/a/x CC0\nb procedural:tools/synth.py(a) own CC0\nc procedural:x+https://polyhaven.com/a/y CC0\nd https://commons.wikimedia.org/wiki/F pd\n\n");
        assert_eq!(t, Tally { total: 4, polyhaven: 2, commons: 1, own: 1 });
        let real = tally(SOURCES);
        assert!(real.total > 0 && real.polyhaven + real.commons + real.own <= real.total);
        assert_eq!(legend_portrait(0), None);
        assert_eq!(legend_portrait(1).map(|p| p.1), Some(4));
        assert_eq!(legend_portrait(PAGES - 1).map(|p| p.1), Some(7));
        assert_eq!(legend_portrait(PAGES), None);
    }
}
