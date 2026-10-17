use super::art::{self, Button, PEN, PEN_FADED, PEN_RED};
use super::goal::{self, Goal};
use super::icons::icon;
use super::lore::{character, item, legend};
use super::{Body, DIM, FAINT, INK, Ink, InkSerif, Item, SCREEN, Serif, centered, label, line, menu, paragraph, width};
use crate::render::Ui2d;
use crate::sim::{DESMAIOS, ItemPlace, World};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Card {
    Intro,
    Outro,
    Restart,
    Ending,
}

pub const INTRO_PORTRAIT: [u16; 4] = [44, 62, 80, 96];
const NOTE: [i16; 4] = [150, 66, 200, 98];

fn title(ui: &mut Ui2d, r: [u16; 4], y: i16, s: &str, color: u16) {
    let face = if width(s, Serif) <= r[2] as i16 - 12 { Serif } else { Body };
    centered(ui, r, y, s, color, face);
}

fn meta(world: &World) -> Option<&str> {
    let map = world.map();
    map.locks.get(map.exit.lock as usize).and_then(|l| l.hint)
}

fn inner(r: [i16; 4]) -> [u16; 4] {
    [r[0] as u16, r[1] as u16, r[2] as u16, r[3] as u16]
}

fn sheet(ui: &mut Ui2d, r: [i16; 4], tape: bool) -> [u16; 4] {
    art::paper(ui, r, art::PAPER);
    match tape {
        true => {
            art::tape(ui, r[0] - 8, r[1] - 5);
            art::tape(ui, r[0] + r[2] - 20, r[1] - 5);
        }
        false => art::pins(ui, r, true),
    }
    inner(r)
}

fn intro(ui: &mut Ui2d, world: &World, phase: u8, phases: u8) {
    let map = world.map();
    let l = legend(map.legend);
    centered(ui, SCREEN, 10, line(format_args!("Fase {} de {}", phase + 1, phases)).as_str(), DIM, Body);
    title(ui, SCREEN, 24, map.name, INK);
    art::photo(ui, INTRO_PORTRAIT, [5, 14]);
    let p = INTRO_PORTRAIT;
    centered(ui, [p[0], p[1], p[2], 0], p[3] as i16 + 3, l.name, PEN, Ink);
    let r = sheet(ui, NOTE, true);
    let x = r[0] as i16 + 10;
    label(ui, x, r[1] as i16 + 8, "A lenda:", PEN_FADED, Ink);
    label(ui, x + width("A lenda: ", Body), r[1] as i16 + 4, l.name, PEN_RED, InkSerif);
    let y = paragraph(ui, x, r[1] as i16 + 26, l.line, r[2] as i16 - 20, PEN);
    meta(world).iter().for_each(|m| {
        paragraph(ui, x, y + 6, line(format_args!("Meta: {m}")).as_str(), r[2] as i16 - 20, PEN_RED);
    });
    centered(ui, SCREEN, 198, "Sexta-feira da Paixão, perto da meia-noite", DIM, Body);
}

fn outro(ui: &mut Ui2d, world: &World, phase: u8, phases: u8) {
    let map = world.map();
    let r = sheet(ui, [72, 46, 240, 124], false);
    title(ui, r, 12, "Vocês escaparam", PEN);
    centered(ui, r, 34, map.name, PEN_FADED, Ink);
    let next = match phase + 1 < phases {
        true => "a próxima fase foi liberada",
        false => "a última lenda ficou para trás",
    };
    centered(ui, r, 56, next, PEN_RED, Ink);
    let (got, of) = goal::fotos(world);
    if of > 0 {
        centered(ui, r, 76, line(format_args!("fotos encontradas: {got} de {of}")).as_str(), if got == of { PEN_RED } else { PEN }, Ink);
    }
    let fell = DESMAIOS - world.desmaios_left();
    centered(ui, r, 90, line(format_args!("desmaios: {fell}")).as_str(), PEN, Ink);
}

fn restart(ui: &mut Ui2d) {
    let r = sheet(ui, [88, 64, 208, 76], false);
    art::sprite(ui, art::MOLDURA, art::BLOOD, r[0] as i16 + r[2] as i16 - 22, r[1] as i16 - 24);
    art::sprite(ui, art::MOLDURA, art::flip(art::BLOOD, true, false), r[0] as i16 - 14, r[1] as i16 + 44);
    centered(ui, r, 18, "Vocês desmaiaram demais.", PEN_RED, InkSerif);
    centered(ui, r, 44, "A noite recomeça.", PEN, Ink);
}

fn ending(ui: &mut Ui2d) {
    let r = sheet(ui, [64, 40, 256, 130], true);
    title(ui, r, 12, "Amanheceu em Xique-Xique", PEN);
    centered(ui, r, 44, "A Sexta-feira da Paixão acabou.", PEN, Ink);
    centered(ui, r, 58, "As lendas voltaram a dormir.", PEN, Ink);
    centered(ui, r, 86, "Obrigado por jogar.", PEN_RED, InkSerif);
    hint(ui, 186, Button::A, "voltar ao menu", FAINT);
}

pub fn hint(ui: &mut Ui2d, y: i16, b: Button, s: &str, color: u16) {
    let w = 16 + width(s, Body);
    art::key_hint(ui, (SCREEN[2] as i16 - w) / 2, y, b, s, color);
}

pub fn card(ui: &mut Ui2d, c: Card, world: &World, phase: u8, phases: u8) {
    match c {
        Card::Intro => intro(ui, world, phase, phases),
        Card::Outro => outro(ui, world, phase, phases),
        Card::Restart => restart(ui),
        Card::Ending => ending(ui),
    }
}

pub fn pause(ui: &mut Ui2d, items: &[Item], sel: usize, online: bool, tick: u32, world: &World, now: Goal) {
    centered(ui, SCREEN, 4, "Pausa", INK, Serif);
    let wide = 232 - 24;
    let goal_line = goal::say(world.map(), now);
    let meta_line = meta(world).filter(|_| now != Goal::Escape).map(|m| line(format_args!("Meta: {m}")));
    let rows = |s: &str| super::wrap(s, wide, Ink).count() as i16 * (Ink.line() + 1);
    let held = world.items().iter().filter(|s| matches!(s.place, ItemPlace::Held { .. })).count().min(4) as i16;
    let text = 26 + rows(goal_line.as_str()) + meta_line.as_ref().map_or(0, |m| 2 + rows(m.as_str())) + 16;
    let h = text + if held > 0 { 6 + (held + 1) / 2 * 34 } else { 0 } + 6;
    let r = sheet(ui, [14, 28, 232, h.max(70)], false);
    let x = r[0] as i16 + 12;
    label(ui, x, r[1] as i16 + 8, "Objetivo", PEN_RED, InkSerif);
    let mut y = paragraph(ui, x, r[1] as i16 + 26, goal_line.as_str(), wide, PEN);
    if let Some(m) = &meta_line {
        y = paragraph(ui, x, y + 2, m.as_str(), wide, PEN_FADED);
    }
    let ((open, locks), (got, fotos)) = (goal::opened(world), goal::fotos(world));
    let tally = match fotos {
        0 => line(format_args!("trancas abertas {open}/{locks}")),
        _ => line(format_args!("trancas abertas {open}/{locks}   fotos {got}/{fotos}")),
    };
    label(ui, x, y + 3, tally.as_str(), PEN_FADED, Ink);
    let carried = world.items().iter().filter_map(|s| match s.place {
        ItemPlace::Held { player } => world.players().get(player as usize).map(|p| (s.kind, p.character)),
        _ => None,
    });
    let base = r[1] as i16 + text + 6;
    carried.take(4).enumerate().for_each(|(i, (k, c))| {
        let (ix, iy) = (x + (i % 2) as i16 * 104, base + (i / 2) as i16 * 34);
        icon(ui, ix, iy, k, 32);
        label(ui, ix + 34, iy + 8, item(k), PEN, Ink);
        label(ui, ix + 34, iy + 18, character(c).short, PEN_FADED, Ink);
    });
    menu(ui, 284, 48, items, sel, tick);
    if online {
        let top = 48 + items.len() as i16 * super::menu::ROW + 8;
        super::wrap("a noite não espera: o jogo continua", 112, Body).enumerate().for_each(|(i, l)| label(ui, 262, top + i as i16 * 11, l, DIM, Body));
    }
    hint(ui, 204, Button::A, "segure para pausar (ou START)", FAINT);
}

pub fn notice(ui: &mut Ui2d, head: &str, detail: &str) {
    let r = sheet(ui, [64, 64, 256, 80], false);
    title(ui, r, 12, head, PEN_RED);
    centered(ui, r, 40, detail, PEN, Ink);
    hint(ui, 168, Button::A, "continuar", DIM);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{CANVAS_H, CANVAS_W};

    #[test]
    fn intro_layout_fits_the_canvas() {
        let p = INTRO_PORTRAIT;
        assert!(p[0] + p[2] + 10 < NOTE[0] as u16 && NOTE[0] + NOTE[2] < CANVAS_W as i16 && p[1] + p[3] + 14 < CANVAS_H - 20);
        assert!(width("Sexta-feira da Paixão, perto da meia-noite", Body) < CANVAS_W as i16);
    }
}
