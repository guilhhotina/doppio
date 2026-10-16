use super::{FAINT, GOLD, INK, SHADOW};
use crate::map::ItemKind;
use crate::render::{Ui2d, rgb565};

const METAL: u16 = rgb565(168, 172, 180);

fn sprite(kind: ItemKind) -> ([u8; 8], u16) {
    match kind {
        ItemKind::Chave => ([0x00, 0x00, 0x70, 0x8F, 0x75, 0x00, 0x00, 0x00], GOLD),
        ItemKind::Alicate => ([0x18, 0x18, 0x3C, 0x24, 0x24, 0x42, 0x42, 0x81], METAL),
        ItemKind::Martelo => ([0x00, 0x7E, 0x7E, 0x18, 0x18, 0x18, 0x18, 0x18], METAL),
        ItemKind::PeDeCabra => ([0x0C, 0x02, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40], rgb565(170, 60, 50)),
        ItemKind::ChaveDeFenda => ([0x10, 0x10, 0x10, 0x10, 0x38, 0x7C, 0x7C, 0x38], rgb565(200, 160, 40)),
        ItemKind::Fusivel => ([0x00, 0x00, 0xE7, 0xFF, 0xE7, 0x00, 0x00, 0x00], METAL),
        ItemKind::Sal => ([0x00, 0x00, 0x00, 0x18, 0x3C, 0x7E, 0xFF, 0x00], rgb565(236, 236, 230)),
        ItemKind::Vela => ([0x10, 0x18, 0x10, 0x18, 0x18, 0x18, 0x18, 0x3C], rgb565(232, 220, 180)),
        ItemKind::Fosforo => ([0x00, 0x00, 0xFE, 0x82, 0xFE, 0xFE, 0x00, 0x00], rgb565(196, 72, 48)),
        ItemKind::Tabaco => ([0x3C, 0x42, 0x99, 0xA5, 0xA5, 0x99, 0x42, 0x3C], rgb565(120, 78, 40)),
        ItemKind::Balde => ([0x3C, 0x42, 0xFF, 0xFF, 0x7E, 0x7E, 0x3C, 0x00], rgb565(110, 150, 190)),
        ItemKind::Freio => ([0x00, 0x66, 0x99, 0x81, 0x81, 0x42, 0x3C, 0x00], rgb565(120, 120, 128)),
        ItemKind::Ossos => ([0xC3, 0xE7, 0x3C, 0x18, 0x18, 0x3C, 0xE7, 0xC3], rgb565(222, 214, 190)),
        ItemKind::Cruz => ([0x18, 0x18, 0x7E, 0x18, 0x18, 0x18, 0x18, 0x18], GOLD),
        ItemKind::BilheteOnibus => ([0x00, 0xFF, 0x81, 0xBD, 0x81, 0xFF, 0x00, 0x00], rgb565(230, 214, 140)),
        ItemKind::Manivela => ([0xF0, 0x10, 0x10, 0x10, 0x1F, 0x01, 0x01, 0x00], METAL),
        ItemKind::Remo => ([0x01, 0x02, 0x04, 0x08, 0x10, 0x60, 0xE0, 0xC0], rgb565(160, 110, 60)),
        ItemKind::LanternaPilha => ([0x18, 0x3C, 0x24, 0x24, 0x3C, 0x3C, 0x3C, 0x3C], rgb565(90, 180, 90)),
        ItemKind::Foto => ([0xFF, 0x81, 0x99, 0x99, 0x81, 0xBD, 0x81, 0xFF], rgb565(190, 160, 120)),
    }
}

fn runs(row: u8) -> impl Iterator<Item = (i16, i16)> {
    (0..8i16).filter(move |&c| row & 0x80 >> c != 0 && (c == 0 || row & 0x80 >> (c - 1) == 0)).map(move |start| {
        let len = (start..8).take_while(|&c| row & 0x80 >> c != 0).count() as i16;
        (start, len)
    })
}

fn blit(ui: &mut Ui2d, x: i16, y: i16, rows: &[u8; 8], color: u16, scale: i16) {
    rows.iter().enumerate().for_each(|(r, &bits)| {
        runs(bits).for_each(|(c, len)| ui.rect(x + c * scale, y + r as i16 * scale, len * scale, scale, color));
    });
}

pub fn icon(ui: &mut Ui2d, x: i16, y: i16, kind: ItemKind, scale: u8) {
    let (rows, color) = sprite(kind);
    let s = scale as i16;
    blit(ui, x + s, y + s, &rows, SHADOW, s);
    blit(ui, x, y, &rows, color, s);
}

pub fn candles(ui: &mut Ui2d, x: i16, y: i16, lit: u8, total: u8, tick: u32) {
    (0..total).for_each(|i| {
        let cx = x + i as i16 * 6;
        let on = i < lit;
        ui.rect(cx + 1, y + 3, 3, 6, SHADOW);
        ui.rect(cx, y + 2, 3, 6, if on { INK } else { FAINT });
        if on {
            let flick = (tick / 7 + i as u32 * 3) % 5 == 0;
            ui.rect(cx + 1, y + if flick { 1 } else { 0 }, 1, 2, GOLD);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::lore::ITEMS;

    #[test]
    fn runs_cover_set_bits() {
        assert_eq!(runs(0x8F).collect::<Vec<_>>(), [(0, 1), (4, 4)]);
        assert_eq!(runs(0xFF).collect::<Vec<_>>(), [(0, 8)]);
        assert_eq!(runs(0).count(), 0);
        ITEMS.iter().for_each(|&k| {
            let (rows, _) = sprite(k);
            let px: u32 = rows.iter().map(|r| runs(*r).map(|(_, l)| l as u32).sum::<u32>()).sum();
            assert_eq!(px, rows.iter().map(|r| r.count_ones()).sum());
        });
        let mut u = Ui2d::new();
        icon(&mut u, 0, 0, ItemKind::Martelo, 1);
        assert_eq!(u.quads(), 2 * 7);
    }
}
