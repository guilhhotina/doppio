use super::lore::{LEGENDS, legend};
use super::{DIM, FAINT, GOLD, INK, SCREEN, centered, line, paragraph, shadowed};
use crate::render::Ui2d;

pub const PAGES: usize = 1 + LEGENDS.len();
pub const LEGEND_RECT: [u16; 4] = [16, 20, 112, 160];
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
    centered(ui, SCREEN, 12, "Créditos", INK, 2);
    let t = tally(SOURCES);
    let rows = [
        (line(format_args!("Texturas: {} ao todo", t.total)), INK),
        (line(format_args!("  Poly Haven, CC0: {}", t.polyhaven)), DIM),
        (line(format_args!("  Wikimedia Commons, domínio público: {}", t.commons)), DIM),
        (line(format_args!("  feitas por código para o jogo, CC0: {}", t.own)), DIM),
        (line(format_args!("Modelos, mapas e sons: feitos para o jogo")), INK),
        (line(format_args!("Lendas: folclore brasileiro, contado de novo")), INK),
        (line(format_args!("Feito para as TVs ZaperOS (libretro)")), INK),
    ];
    rows.iter().enumerate().for_each(|(i, (l, c))| shadowed(ui, 24, 44 + i as i16 * 16, l.as_str(), *c, 1));
}

fn lore(ui: &mut Ui2d, i: usize) {
    let l = legend(LEGENDS[i]);
    let x = 140;
    shadowed(ui, x, 20, "Lendas de Pedra Seca", DIM, 1);
    shadowed(ui, x, 36, l.name, GOLD, 2);
    paragraph(ui, x, 64, l.lore, 30, INK);
}

pub fn credits(ui: &mut Ui2d, page: usize) {
    match page {
        0 => about(ui),
        p => lore(ui, (p - 1).min(LEGENDS.len() - 1)),
    }
    let mut dots = [b'.'; PAGES];
    dots[page.min(PAGES - 1)] = b'o';
    centered(ui, SCREEN, 192, std::str::from_utf8(&dots).unwrap_or(""), DIM, 1);
    centered(ui, SCREEN, 204, "A/direita: próxima   B: voltar", FAINT, 1);
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
