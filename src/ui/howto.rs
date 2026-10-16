use super::{DIM, Dir, GOLD, INK, Item, SCREEN, arrow, centered, menu, shadowed};
use crate::render::Ui2d;

enum Key {
    Pad(Dir, Dir),
    Button(&'static str),
}

const ROWS: [(Key, &str); 7] = [
    (Key::Pad(Dir::Up, Dir::Down), "andar (frente e trás)"),
    (Key::Pad(Dir::Left, Dir::Right), "virar"),
    (Key::Button("A"), "pegar, usar, abrir, esconder, ler"),
    (Key::Button("A"), "sem nada perto: lanterna"),
    (Key::Button("B"), "segurar para correr (faz barulho)"),
    (Key::Button("X Y"), "lanterna / meia-volta"),
    (Key::Button("START"), "pausa (ou segure A)"),
];

const RULES: [&str; 4] = [
    "Achem itens, abram trancas e fujam.",
    "A lenda ouve passos e corridas: escondam-se.",
    "Quem a lenda pega desmaia e acorda longe.",
    "5 desmaios por fase; no sexto, a fase recomeça.",
];

pub fn howto(ui: &mut Ui2d, tick: u32) {
    centered(ui, SCREEN, 12, "Como jogar", INK, 2);
    let (kx, tx) = (56i16, 112i16);
    ROWS.iter().enumerate().for_each(|(i, (k, text))| {
        let y = 38 + i as i16 * 12;
        match k {
            Key::Pad(a, b) => {
                arrow(ui, tx - 30, y, *a, GOLD);
                arrow(ui, tx - 20, y, *b, GOLD);
            }
            Key::Button(name) => shadowed(ui, kx + 40 - super::width(name, 1), y, name, GOLD, 1),
        }
        shadowed(ui, tx, y, text, INK, 1);
    });
    RULES.iter().enumerate().for_each(|(i, r)| centered(ui, SCREEN, 130 + i as i16 * 12, r, DIM, 1));
    menu(ui, 168, 192, &[Item::new("Voltar")], 0, tick);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_fits() {
        assert!(RULES.iter().all(|r| r.chars().count() <= 48));
        assert!(ROWS.iter().all(|(_, t)| 112 + super::super::width(t, 1) <= 384));
    }
}
