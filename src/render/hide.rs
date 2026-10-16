use super::ui2d::{Ui2d, rgb565};
use crate::map::HideKind;

const WOOD: u16 = rgb565(34, 22, 14);
const WOOD_LIT: u16 = rgb565(60, 40, 24);
const BLACK: u16 = rgb565(6, 5, 6);
const CLOTH: u16 = rgb565(58, 30, 32);
const STONE: u16 = rgb565(30, 30, 32);
const STRAW: u16 = rgb565(70, 58, 30);
const METAL: u16 = rgb565(20, 22, 26);

pub fn overlay(kind: HideKind, rect: [i32; 4], t: f32, ui: &mut Ui2d) {
    let [x, y, w, h] = rect;
    let breathe = ((t * 1.3).sin() * 1.5).round() as i32;
    let r = |ui: &mut Ui2d, rx: i32, ry: i32, rw: i32, rh: i32, c: u16| {
        let (x0, y0) = ((x + rx).max(x), (y + ry).max(y));
        let (x1, y1) = ((x + rx + rw).min(x + w), (y + ry + rh).min(y + h));
        if x1 > x0 && y1 > y0 {
            ui.rect(x0 as i16, y0 as i16, (x1 - x0) as i16, (y1 - y0) as i16, c);
        }
    };
    match kind {
        HideKind::GuardaRoupa | HideKind::Armario => {
            let pitch = (h / 14).max(6);
            (0..h / pitch + 2).for_each(|i| {
                let yy = i * pitch + breathe;
                r(ui, 0, yy, w, pitch * 2 / 5, WOOD);
                r(ui, 0, yy + pitch * 2 / 5, w, 1, WOOD_LIT);
            });
            r(ui, 0, 0, w / 9, h, WOOD);
            r(ui, w - w / 9, 0, w / 9, h, WOOD);
            r(ui, w / 2 - 2, 0, 4, h, WOOD);
        }
        HideKind::Confessionario => {
            let pitch = (w / 24).max(6);
            (0..w / pitch + 1).for_each(|i| r(ui, i * pitch, 0, pitch / 2, h, WOOD));
            (0..h / pitch + 1).for_each(|i| r(ui, 0, i * pitch + breathe, w, pitch / 2, WOOD));
            r(ui, 0, 0, w, h / 5, CLOTH);
            r(ui, 0, h - h / 6, w, h / 6, WOOD);
        }
        HideKind::Cama | HideKind::Mesa => {
            let skirt = h * 55 / 100 + breathe;
            let c = if kind == HideKind::Cama { BLACK } else { CLOTH };
            r(ui, 0, 0, w, skirt, c);
            let fold = (w / 16).max(4);
            (0..w / fold + 1).for_each(|i| r(ui, i * fold, skirt - 1, fold, h / 40 + (i * 7 % 5) * h / 90, c));
            r(ui, 0, h - h / 10, w, h / 10, BLACK);
            if kind == HideKind::Mesa {
                (0..w / fold).filter(|i| i % 2 == 0).for_each(|i| r(ui, i * fold + fold / 4, skirt - h / 12, 2, 2, WOOD_LIT));
            }
        }
        HideKind::Tumulo => {
            let slit = h * 16 / 100;
            let mid = h / 2 + breathe;
            r(ui, 0, 0, w, mid - slit / 2, STONE);
            r(ui, 0, mid + slit / 2, w, h, STONE);
            r(ui, 0, mid - slit / 2 - 2, w, 2, BLACK);
            r(ui, 0, 0, w / 6, h, STONE);
            r(ui, w - w / 6, 0, w / 6, h, STONE);
        }
        HideKind::CaixaDagua => {
            let (cx, cy, rad) = (w / 2, h / 2 + breathe, h * 28 / 100);
            let step = 3;
            (0..h / step + 1).for_each(|i| {
                let yy = i * step;
                let dy = (yy - cy).abs();
                let half = if dy < rad { ((rad * rad - dy * dy) as f32).sqrt() as i32 } else { 0 };
                match half {
                    0 => r(ui, 0, yy, w, step, METAL),
                    _ => {
                        r(ui, 0, yy, cx - half, step, METAL);
                        r(ui, cx + half, yy, w, step, METAL);
                    }
                }
            });
        }
        HideKind::Carro => {
            r(ui, 0, 0, w, h / 7, METAL);
            r(ui, 0, h - h * 3 / 10 + breathe, w, h, METAL);
            (0..6).for_each(|i| {
                let k = i * h / 10;
                r(ui, 0, h / 7 + k, w / 7 - i * w / 60, h / 10, METAL);
                r(ui, w - w / 7 + i * w / 60, h / 7 + k, w, h / 10, METAL);
            });
            r(ui, w / 2 - w / 12, h - h * 3 / 10 - h / 14 + breathe, w / 6, h / 14, BLACK);
        }
        HideKind::Cesto => {
            let pitch = (w / 20).max(6);
            (0..w / pitch + 1).for_each(|i| r(ui, i * pitch, 0, pitch * 2 / 3, h, STRAW));
            (0..h / pitch + 1).filter(|i| i % 2 == 0).for_each(|i| r(ui, 0, i * pitch + breathe, w, pitch * 2 / 3, STRAW));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlays_stay_inside_their_view_and_fit_the_quad_budget() {
        let kinds = [
            HideKind::Cama,
            HideKind::GuardaRoupa,
            HideKind::Armario,
            HideKind::Tumulo,
            HideKind::CaixaDagua,
            HideKind::Confessionario,
            HideKind::Carro,
            HideKind::Cesto,
            HideKind::Mesa,
        ];
        kinds.iter().for_each(|&k| {
            let mut ui = Ui2d::new();
            overlay(k, [192, 108, 192, 108], 0.7, &mut ui);
            assert!(ui.quads() > 3 && ui.quads() < 300, "{k:?} {}", ui.quads());
            assert!(ui.verts().iter().all(|v| (192..=384).contains(&v.pos[0]) && (108..=216).contains(&v.pos[1])), "{k:?}");
        });
    }
}
