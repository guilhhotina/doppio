use super::{BLOOD, DIM, FAINT, INK, Item, Line, SCREEN, centered, menu, ordinal};
use crate::render::Ui2d;
use std::fmt::Write;

fn night_title(night: u8) -> Line {
    let word = ordinal(night);
    let mut l = Line::new();
    let _ = write!(l, "{}{} noite", word.as_bytes()[0].to_ascii_uppercase() as char, &word[1..]);
    l
}

pub fn dusk(ui: &mut Ui2d, rect: [u16; 4], night: u8) {
    let scale = if rect[2] >= 300 { 2 } else { 1 };
    let mid = rect[3] as i16 / 2;
    centered(ui, rect, mid - 10, night_title(night).as_str(), INK, scale);
    centered(ui, rect, mid + 12, "22:00", DIM, 1);
}

pub fn dawn(ui: &mut Ui2d, rect: [u16; 4], night: u8) {
    let scale = if rect[2] >= 300 { 2 } else { 1 };
    let mid = rect[3] as i16 / 2;
    centered(ui, rect, mid - 10, "Amanheceu", INK, scale);
    let mut l = Line::new();
    let _ = match 5u8.saturating_sub(night) {
        1 => write!(l, "falta uma noite"),
        n => write!(l, "faltam {n} noites"),
    };
    centered(ui, rect, mid + 12, l.as_str(), DIM, 1);
}

pub fn whisper(ui: &mut Ui2d, rect: [u16; 4], tick: u32) {
    let y = rect[3] as i16 - 30;
    let color = if tick % 47 < 2 { FAINT } else { BLOOD };
    centered(ui, rect, y, "você agora é cópia. ninguém sabe.", color, 1);
}

pub fn pause(ui: &mut Ui2d, sel: usize, online: bool, tick: u32) {
    centered(ui, SCREEN, 60, "Pausa", INK, 2);
    menu(ui, 148, 100, &[Item::new("Continuar"), Item::new("Sair para o menu")], sel, tick);
    if online {
        centered(ui, SCREEN, 160, "a casa não espera: o jogo continua", DIM, 1);
    }
}

pub fn ending(ui: &mut Ui2d, won: bool, duplo: bool, night: u8) {
    let (head, sub) = match (won, duplo) {
        (true, _) => ("Vocês viram o sol", "sobreviveram às cinco noites"),
        (false, true) => ("Ninguém percebeu", "agora todos são cópias"),
        (false, false) => ("A casa ficou com vocês", ""),
    };
    centered(ui, SCREEN, 70, head, if won { INK } else { BLOOD }, 2);
    let mut l = Line::new();
    let _ = match sub.is_empty() {
        true => write!(l, "chegaram até a {} noite", ordinal(night)),
        false => write!(l, "{sub}"),
    };
    centered(ui, SCREEN, 104, l.as_str(), DIM, 1);
    centered(ui, SCREEN, 170, "A voltar ao menu", FAINT, 1);
}

pub fn notice(ui: &mut Ui2d, head: &str, detail: &str) {
    centered(ui, SCREEN, 80, head, INK, 2);
    centered(ui, SCREEN, 112, detail, DIM, 1);
    centered(ui, SCREEN, 170, "A continuar", FAINT, 1);
}
