use super::art::{self, Button, PEN, PEN_FADED, PEN_RED, START};
use super::{Body, Dir, INK, Ink, Item, SCREEN, Serif, arrow, centered, label, menu, width};
use crate::render::Ui2d;

enum Key {
    Pad(Dir, Dir),
    Button(Button),
    Pair(Button, Button),
    Start,
}

const ROWS: [(Key, &str); 7] = [
    (Key::Pad(Dir::Up, Dir::Down), "andar (frente e trás)"),
    (Key::Pad(Dir::Left, Dir::Right), "virar"),
    (Key::Button(Button::A), "pegar, usar, abrir, esconder, ler"),
    (Key::Button(Button::A), "sem nada perto: lanterna"),
    (Key::Button(Button::B), "segurar para correr (faz barulho)"),
    (Key::Pair(Button::X, Button::Y), "lanterna / meia-volta"),
    (Key::Start, "pausa (ou segure A)"),
];

const RULES: [&str; 4] = [
    "Achem itens, abram trancas e fujam.",
    "A lenda ouve passos e corridas: escondam-se.",
    "Quem a lenda pega desmaia e acorda longe.",
    "5 desmaios por fase; no sexto, a fase recomeça.",
];

const SHEET: [i16; 4] = [24, 32, 336, 148];

pub fn howto(ui: &mut Ui2d, tick: u32) {
    centered(ui, SCREEN, 8, "Como jogar", INK, Serif);
    art::paper(ui, SHEET, art::PAPER);
    art::pins(ui, SHEET, false);
    let (kx, tx) = (SHEET[0] + 16, SHEET[0] + 52);
    ROWS.iter().enumerate().for_each(|(i, (k, text))| {
        let y = SHEET[1] + 10 + i as i16 * 13;
        match k {
            Key::Pad(a, b) => {
                arrow(ui, kx, y, *a);
                arrow(ui, kx + 13, y, *b);
            }
            Key::Button(b) => art::button(ui, kx + 6, y - 2, *b),
            Key::Pair(a, b) => {
                art::button(ui, kx - 1, y - 2, *a);
                art::button(ui, kx + 13, y - 2, *b);
            }
            Key::Start => art::sprite(ui, art::LATAO, START, kx - 1, y - 1),
        }
        label(ui, tx, y, text, PEN, Ink);
    });
    RULES.iter().enumerate().for_each(|(i, r)| {
        let y = SHEET[1] + 104 + i as i16 * 11;
        label(ui, SHEET[0] + (SHEET[2] - width(r, Body)) / 2, y, r, if i == 3 { PEN_RED } else { PEN_FADED }, Ink);
    });
    menu(ui, 168, 198, &[Item::new("Voltar")], 0, tick);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_fits() {
        assert!(RULES.iter().all(|r| width(r, Body) <= SHEET[2] - 16));
        assert!(ROWS.iter().all(|(_, t)| SHEET[0] + 52 + width(t, Body) <= SHEET[0] + SHEET[2] - 8));
    }
}
