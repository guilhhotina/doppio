use super::art::{self, Button, LATAO, PADLOCK, PEN, WHEEL, WHEEL_ON};
use super::{BLOOD, Body, DIM, Dir, INK, Ink, arrow, label, line, width, wrap};
use crate::render::Ui2d;
use crate::sim::Dial;

const SLOT: [i16; 4] = [9, 53, 62, 22];

pub fn dial(ui: &mut Ui2d, r: [u16; 4], d: Dial) {
    let len = d.len.clamp(1, 4) as i16;
    let (cw, gap) = (WHEEL[2] as i16, 2i16);
    let (lw, lh) = (PADLOCK[2] as i16, PADLOCK[3] as i16);
    let (cx, cy) = (r[0] as i16 + r[2] as i16 / 2, r[1] as i16 + r[3] as i16 / 2);
    let (lx, ly) = (cx - lw / 2, cy - lh / 2 - 6);
    art::sprite(ui, LATAO, PADLOCK, lx, ly);
    let w = len * (cw + gap) - gap;
    let x0 = lx + SLOT[0] + (SLOT[2] - w) / 2;
    let y0 = ly + SLOT[1];
    (0..len).for_each(|i| {
        let x = x0 + i * (cw + gap);
        let on = i == d.cursor as i16;
        art::sprite(ui, LATAO, if on { WHEEL_ON } else { WHEEL }, x, y0);
        let digit = [b'0' + d.digits[i as usize] % 10];
        let s = std::str::from_utf8(&digit).unwrap_or("0");
        label(ui, x + (cw - width(s, Body)) / 2, y0 + 7, s, if on { BLOOD } else { PEN }, Ink);
        if on {
            arrow(ui, x + 1, y0 - 12, Dir::Up);
            arrow(ui, x + 1, y0 + 26, Dir::Down);
        }
    });
    let by = ly + lh + 8;
    match d.wrong {
        0 => {
            let w = 16 + width("abrir", Body) + 10 + 16 + width("sair", Body);
            let x = cx - w / 2;
            let x = art::key_hint(ui, x, by, Button::A, "abrir", INK);
            art::key_hint(ui, x, by, Button::Y, "sair", INK);
        }
        n => {
            let msg = line(format_args!("errou {n}x"));
            let w = width(msg.as_str(), Body) + 10 + 16 + width("sair", Body);
            let x = ui.text(cx - w / 2, by, msg.as_str(), BLOOD, Body) + 10;
            art::key_hint(ui, x, by, Button::Y, "sair", DIM);
        }
    }
}

pub fn note(ui: &mut Ui2d, r: [u16; 4], text: &str) {
    let w = (r[2] as i16 - 24).min(300);
    let cols = w - 24;
    let lines = wrap(text, cols, Body).count() as i16;
    let h = lines * (Body.line() + 1) + 34;
    let (x, y) = (r[0] as i16 + (r[2] as i16 - w) / 2, r[1] as i16 + (r[3] as i16 - h) / 2 - 4);
    art::paper(ui, [x, y, w, h], [255, 244, 214, 255]);
    art::tape(ui, x + w / 2 - 14, y - 6);
    art::sprite(ui, art::MOLDURA, art::STAIN, x + w - 30, y + h - 30);
    wrap(text, cols, Body).enumerate().for_each(|(i, l)| {
        label(ui, x + 12, y + 12 + i as i16 * (Body.line() + 1), l, PEN, Ink);
    });
    let hint = 16 + width("fechar", Body);
    art::key_hint(ui, x + w - 10 - hint, y + h + 6, Button::A, "fechar", INK);
}
