use super::lore::legend;
use super::{BLOOD, DIM, FAINT, GOLD, INK, Item, SCREEN, centered, line, menu, width};
use crate::map::Map;
use crate::render::Ui2d;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Card {
    Intro,
    Outro,
    Restart,
    Ending,
}

pub const INTRO_PORTRAIT: [u16; 4] = [152, 96, 80, 96];

fn title(ui: &mut Ui2d, y: i16, s: &str, color: u16) {
    let scale = if width(s, 2) <= SCREEN[2] as i16 - 16 { 2 } else { 1 };
    centered(ui, SCREEN, y, s, color, scale);
}

pub fn card(ui: &mut Ui2d, c: Card, map: &Map, phase: u8, phases: u8) {
    let l = legend(map.legend);
    match c {
        Card::Intro => {
            centered(ui, SCREEN, 16, line(format_args!("Fase {} de {}", phase + 1, phases)).as_str(), DIM, 1);
            title(ui, 34, map.name, INK);
            centered(ui, SCREEN, 62, line(format_args!("A lenda: {}", l.name)).as_str(), GOLD, 1);
            centered(ui, SCREEN, 76, l.line, DIM, 1);
            centered(ui, SCREEN, 200, "Sexta-feira da Paixão, perto da meia-noite", FAINT, 1);
        }
        Card::Outro => {
            title(ui, 70, "Vocês escaparam", INK);
            centered(ui, SCREEN, 100, map.name, DIM, 1);
            let next = match phase + 1 < phases {
                true => "a próxima fase foi liberada",
                false => "a última lenda ficou para trás",
            };
            centered(ui, SCREEN, 120, next, GOLD, 1);
        }
        Card::Restart => {
            centered(ui, SCREEN, 88, "Vocês desmaiaram demais.", BLOOD, 1);
            centered(ui, SCREEN, 104, "A noite recomeça.", INK, 1);
        }
        Card::Ending => {
            title(ui, 56, "Amanheceu em Pedra Seca", INK);
            centered(ui, SCREEN, 92, "A Sexta-feira da Paixão acabou.", DIM, 1);
            centered(ui, SCREEN, 106, "As lendas voltaram a dormir.", DIM, 1);
            centered(ui, SCREEN, 132, "Obrigado por jogar.", GOLD, 1);
            centered(ui, SCREEN, 184, "A voltar ao menu", FAINT, 1);
        }
    }
}

pub fn pause(ui: &mut Ui2d, items: &[Item], sel: usize, online: bool, tick: u32) {
    centered(ui, SCREEN, 56, "Pausa", INK, 2);
    menu(ui, 140, 92, items, sel, tick);
    if online {
        centered(ui, SCREEN, 164, "a noite não espera: o jogo continua", DIM, 1);
    }
    centered(ui, SCREEN, 184, "START ou segure A para pausar", FAINT, 1);
}

pub fn notice(ui: &mut Ui2d, head: &str, detail: &str) {
    centered(ui, SCREEN, 80, head, INK, 2);
    centered(ui, SCREEN, 112, detail, DIM, 1);
    centered(ui, SCREEN, 170, "A continuar", FAINT, 1);
}
