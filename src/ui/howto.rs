use super::{DIM, Dir, FAINT, GOLD, INK, SCREEN, arrow, centered, shadowed};
use crate::render::Ui2d;

enum Key {
    Pad(Dir, Dir),
    Button(&'static str),
}

const ROWS: [(Key, &str); 7] = [
    (Key::Pad(Dir::Up, Dir::Down), "andar (frente e trás)"),
    (Key::Pad(Dir::Left, Dir::Right), "virar"),
    (Key::Button("A"), "interagir / lanterna"),
    (Key::Button("B"), "segurar para correr"),
    (Key::Button("X"), "lanterna"),
    (Key::Button("Y"), "virar rápido"),
    (Key::Button("START"), "pausa (ou SELECT)"),
];

const RULES: [&str; 4] = [
    "Sobreviva a cinco noites, das 22h às 6h.",
    "Segure A nas tarefas antes da luz cair.",
    "Desde a segunda noite, a casa tem cópias.",
    "Não dê as costas no escuro.",
];

pub fn howto(ui: &mut Ui2d) {
    centered(ui, SCREEN, 14, "Como jogar", INK, 2);
    let (kx, tx) = (72i16, 132i16);
    ROWS.iter().enumerate().for_each(|(i, (k, text))| {
        let y = 44 + i as i16 * 13;
        match k {
            Key::Pad(a, b) => {
                arrow(ui, tx - 30, y, *a, GOLD);
                arrow(ui, tx - 20, y, *b, GOLD);
            }
            Key::Button(name) => shadowed(ui, kx + 40 - super::width(name, 1), y, name, GOLD, 1),
        }
        shadowed(ui, tx, y, text, INK, 1);
    });
    RULES.iter().enumerate().for_each(|(i, r)| centered(ui, SCREEN, 144 + i as i16 * 11, r, DIM, 1));
    centered(ui, SCREEN, 200, "B voltar", FAINT, 1);
}
