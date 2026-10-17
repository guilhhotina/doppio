use super::art::{self, CANDLES, CANDLE_OUT, LATAO};
use crate::map::ItemKind;
use crate::render::{Ui2d, WHITE};

pub const ICON: i16 = 32;

pub fn texture(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Chave => "item_chave_a",
        ItemKind::Alicate => "item_alicate_a",
        ItemKind::Martelo => "item_martelo_a",
        ItemKind::PeDeCabra => "item_pe_de_cabra_a",
        ItemKind::ChaveDeFenda => "item_chave_de_fenda_a",
        ItemKind::Fusivel => "item_fusivel_a",
        ItemKind::Sal => "item_sal_a",
        ItemKind::Vela => "item_vela_a",
        ItemKind::Fosforo => "item_fosforo_a",
        ItemKind::Tabaco => "item_tabaco_a",
        ItemKind::Balde => "item_balde_a",
        ItemKind::Freio => "item_freio_a",
        ItemKind::Ossos => "item_ossos_a",
        ItemKind::Cruz => "item_cruz_a",
        ItemKind::BilheteOnibus => "item_bilhete_onibus_a",
        ItemKind::Manivela => "item_manivela_a",
        ItemKind::Remo => "item_remo_a",
        ItemKind::LanternaPilha => "item_lanterna_pilha_a",
        ItemKind::Foto => "item_foto_a",
    }
}

pub fn icon(ui: &mut Ui2d, x: i16, y: i16, kind: ItemKind, size: i16) {
    ui.sprite(texture(kind), [0, 0, ICON as u16, ICON as u16], [x, y, size, size], WHITE);
}

pub fn candles(ui: &mut Ui2d, x: i16, y: i16, lit: u8, total: u8, tick: u32) {
    (0..total).for_each(|i| {
        let cx = x + i as i16 * 8;
        let src = match i < lit {
            true => CANDLES[((tick / 6 + i as u32 * 5) % 7 % 3) as usize],
            false => CANDLE_OUT,
        };
        art::sprite(ui, LATAO, src, cx, y);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::lore::ITEMS;

    #[test]
    fn every_item_has_a_photo_icon() {
        let mut u = Ui2d::new();
        ITEMS.iter().for_each(|&k| {
            assert_eq!(u.size(texture(k)), [ICON, ICON]);
            icon(&mut u, 0, 0, k, 16);
        });
        assert_eq!(u.quads(), ITEMS.len());
        candles(&mut u, 0, 0, 3, 5, 7);
        assert_eq!(u.quads(), ITEMS.len() + 5);
    }
}
