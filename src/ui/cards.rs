use super::goal::{self, Goal};
use super::icons::icon;
use super::lore::{character, item, legend};
use super::{BLOOD, DIM, FAINT, GOLD, INK, Item, SCREEN, centered, line, menu, shadowed, width};
use crate::render::Ui2d;
use crate::sim::{DESMAIOS, ItemPlace, World};

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

fn meta(world: &World) -> Option<&str> {
    let map = world.map();
    map.locks.get(map.exit.lock as usize).and_then(|l| l.hint)
}

pub fn card(ui: &mut Ui2d, c: Card, world: &World, phase: u8, phases: u8) {
    let map = world.map();
    let l = legend(map.legend);
    match c {
        Card::Intro => {
            centered(ui, SCREEN, 16, line(format_args!("Fase {} de {}", phase + 1, phases)).as_str(), DIM, 1);
            title(ui, 34, map.name, INK);
            centered(ui, SCREEN, 62, line(format_args!("A lenda: {}", l.name)).as_str(), GOLD, 1);
            centered(ui, SCREEN, 74, l.line, DIM, 1);
            meta(world).iter().for_each(|m| centered(ui, SCREEN, 85, line(format_args!("Meta: {m}")).as_str(), INK, 1));
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
            let (got, of) = goal::fotos(world);
            if of > 0 {
                centered(ui, SCREEN, 142, line(format_args!("fotos encontradas: {got} de {of}")).as_str(), if got == of { GOLD } else { DIM }, 1);
            }
            let fell = DESMAIOS - world.desmaios_left();
            centered(ui, SCREEN, 154, line(format_args!("desmaios: {fell}")).as_str(), DIM, 1);
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

pub fn pause(ui: &mut Ui2d, items: &[Item], sel: usize, online: bool, tick: u32, world: &World, now: Goal) {
    centered(ui, SCREEN, 10, "Pausa", INK, 2);
    centered(ui, SCREEN, 34, "Objetivo", GOLD, 1);
    centered(ui, SCREEN, 46, goal::say(world.map(), now).as_str(), INK, 1);
    if let Some(m) = meta(world).filter(|_| now != Goal::Escape) {
        centered(ui, SCREEN, 58, line(format_args!("Meta: {m}")).as_str(), DIM, 1);
    }
    let ((open, locks), (got, fotos)) = (goal::opened(world), goal::fotos(world));
    let tally = match fotos {
        0 => line(format_args!("trancas abertas {open}/{locks}")),
        _ => line(format_args!("trancas abertas {open}/{locks}   fotos {got}/{fotos}")),
    };
    centered(ui, SCREEN, 70, tally.as_str(), DIM, 1);
    let carried = world.items().iter().filter_map(|s| match s.place {
        ItemPlace::Held { player } => world.players().get(player as usize).map(|p| (s.kind, p.character)),
        _ => None,
    });
    carried.take(4).enumerate().for_each(|(i, (k, c))| {
        let (x, y) = (if i % 2 == 0 { 40 } else { 200 }, 84 + (i / 2) as i16 * 12);
        icon(ui, x, y, k, 1);
        shadowed(ui, x + 12, y, line(format_args!("{} ({})", item(k), character(c).short)).as_str(), INK, 1);
    });
    menu(ui, 140, 112, items, sel, tick);
    if online {
        centered(ui, SCREEN, 168, "a noite não espera: o jogo continua", DIM, 1);
    }
    centered(ui, SCREEN, 190, "START ou segure A para pausar", FAINT, 1);
}

pub fn notice(ui: &mut Ui2d, head: &str, detail: &str) {
    centered(ui, SCREEN, 80, head, INK, 2);
    centered(ui, SCREEN, 112, detail, DIM, 1);
    centered(ui, SCREEN, 170, "A continuar", FAINT, 1);
}
