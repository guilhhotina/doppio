use super::mat::{self, V3};
use super::mesh::{Faces, Mesh, Pen};
use super::props::Parts;
use super::texture;
use crate::map::ItemKind;

pub const KINDS: usize = 19;
pub const GLINT: f32 = 4.0;
pub const BIA_GLINT: f32 = 12.0;
pub const BIA: u8 = 3;

pub const ALL: [ItemKind; KINDS] = [
    ItemKind::Chave,
    ItemKind::Alicate,
    ItemKind::Martelo,
    ItemKind::PeDeCabra,
    ItemKind::ChaveDeFenda,
    ItemKind::Fusivel,
    ItemKind::Sal,
    ItemKind::Vela,
    ItemKind::Fosforo,
    ItemKind::Tabaco,
    ItemKind::Balde,
    ItemKind::Freio,
    ItemKind::Ossos,
    ItemKind::Cruz,
    ItemKind::BilheteOnibus,
    ItemKind::Manivela,
    ItemKind::Remo,
    ItemKind::LanternaPilha,
    ItemKind::Foto,
];

pub fn glint_range(character: u8) -> f32 {
    if character == BIA { BIA_GLINT } else { GLINT }
}

struct Kit {
    parts: Parts,
}

impl Kit {
    fn pen(&mut self, tex: &str) -> Pen<'_> {
        Pen { mesh: self.parts.entry((0, texture::id(tex))).or_default(), xf: mat::IDENTITY, density: 4.0 }
    }

    fn rod(&mut self, tex: &str, a: V3, b: V3, t: f32) {
        let (lo, hi) = (std::array::from_fn(|k| a[k].min(b[k]) - t), std::array::from_fn(|k| a[k].max(b[k]) + t));
        self.pen(tex).cuboid(lo, hi, Faces::ALL);
    }

    fn card(&mut self, tex: &str, hx: f32, hz: f32) {
        self.pen(tex).quad([[-hx, 0.012, hz], [hx, 0.012, hz], [hx, 0.012, -hz], [-hx, 0.012, -hz]], [[0.15, 0.85], [0.85, 0.85], [0.85, 0.15], [0.15, 0.15]]);
    }
}

fn shape(kind: ItemKind, k: &mut Kit) {
    match kind {
        ItemKind::Chave => {
            k.rod("bronze", [0.0, 0.01, -0.02], [0.0, 0.01, 0.07], 0.008);
            k.pen("bronze").cylinder([0.0, 0.0, -0.045], 0.025, 0.025, 0.012, 6, true);
            k.rod("bronze", [0.0, 0.01, 0.05], [0.02, 0.01, 0.07], 0.007);
        }
        ItemKind::Alicate => {
            k.rod("metal", [-0.01, 0.01, 0.0], [0.0, 0.01, 0.07], 0.008);
            k.rod("ceramica_barro", [-0.03, 0.01, -0.12], [-0.01, 0.01, 0.0], 0.009);
            k.rod("ceramica_barro", [0.01, 0.01, -0.12], [0.03, 0.01, 0.0], 0.009);
        }
        ItemKind::Martelo => {
            k.rod("madeira_crua", [0.0, 0.015, -0.14], [0.0, 0.015, 0.1], 0.014);
            k.rod("metal", [-0.06, 0.02, 0.09], [0.05, 0.02, 0.11], 0.02);
        }
        ItemKind::PeDeCabra => {
            k.rod("ferrugem", [0.0, 0.012, -0.3], [0.0, 0.012, 0.28], 0.011);
            k.rod("ferrugem", [0.0, 0.012, 0.28], [0.05, 0.012, 0.32], 0.011);
        }
        ItemKind::ChaveDeFenda => {
            k.rod("ceramica_barro", [0.0, 0.015, -0.1], [0.0, 0.015, 0.0], 0.014);
            k.rod("metal", [0.0, 0.015, 0.0], [0.0, 0.015, 0.12], 0.004);
        }
        ItemKind::Fusivel => {
            k.pen("ceramica").cylinder([0.0, 0.0, 0.0], 0.03, 0.03, 0.05, 6, true);
            k.pen("bronze").cylinder([0.0, 0.05, 0.0], 0.018, 0.018, 0.02, 6, true);
        }
        ItemKind::Sal => {
            k.pen("colcha").cuboid([-0.07, 0.0, -0.05], [0.07, 0.14, 0.05], Faces::NO_BOTTOM);
            k.pen("colcha").cylinder([0.0, 0.14, 0.0], 0.04, 0.015, 0.05, 4, true);
        }
        ItemKind::Vela => k.pen("cal").cylinder([0.0, 0.0, 0.0], 0.018, 0.018, 0.16, 5, true),
        ItemKind::Fosforo => {
            k.pen("papel_jornal").cuboid([-0.025, 0.0, -0.035], [0.025, 0.015, 0.035], Faces::NO_BOTTOM);
            k.pen("ceramica_barro").cuboid([0.025, 0.0, -0.035], [0.027, 0.015, 0.035], Faces(0x01));
        }
        ItemKind::Tabaco => {
            let xf = mat::rot_x(std::f32::consts::FRAC_PI_2);
            Pen { mesh: k.parts.entry((0, texture::id("madeira_escura"))).or_default(), xf, density: 4.0 }.cylinder([0.0, -0.08, -0.03], 0.035, 0.035, 0.16, 6, true);
        }
        ItemKind::Balde => {
            k.pen("metal").cylinder([0.0, 0.0, 0.0], 0.09, 0.12, 0.2, 7, false);
            k.pen("agua").cylinder([0.0, 0.0, 0.0], 0.11, 0.11, 0.17, 7, true);
            k.rod("ferrugem", [-0.12, 0.2, 0.0], [0.12, 0.28, 0.0], 0.005);
        }
        ItemKind::Freio => {
            k.rod("ferrugem", [-0.08, 0.008, -0.08], [0.08, 0.008, -0.07], 0.008);
            k.rod("ferrugem", [-0.08, 0.008, 0.07], [0.08, 0.008, 0.08], 0.008);
            k.rod("ferrugem", [-0.08, 0.008, -0.08], [-0.07, 0.008, 0.08], 0.008);
            k.rod("ferrugem", [0.07, 0.008, -0.08], [0.08, 0.008, 0.08], 0.008);
            k.rod("ferrugem", [-0.01, 0.008, 0.08], [0.01, 0.008, 0.25], 0.006);
        }
        ItemKind::Ossos => {
            k.rod("cal", [-0.1, 0.012, -0.02], [0.1, 0.012, 0.02], 0.012);
            k.rod("cal", [-0.05, 0.03, -0.09], [0.04, 0.03, 0.08], 0.01);
            k.pen("cal").cylinder([0.07, 0.0, 0.06], 0.045, 0.04, 0.07, 5, true);
        }
        ItemKind::Cruz => {
            k.rod("madeira_escura", [0.0, 0.01, -0.12], [0.0, 0.01, 0.12], 0.012);
            k.rod("madeira_escura", [-0.06, 0.01, 0.05], [0.06, 0.01, 0.05], 0.012);
        }
        ItemKind::BilheteOnibus => k.card("item_bilhete_onibus_a", 0.07, 0.04),
        ItemKind::Manivela => {
            k.rod("metal", [0.0, 0.012, -0.15], [0.0, 0.012, 0.12], 0.01);
            k.rod("metal", [0.0, 0.012, 0.12], [0.1, 0.012, 0.12], 0.01);
            k.rod("madeira_escura", [0.1, 0.012, 0.12], [0.1, 0.09, 0.12], 0.016);
        }
        ItemKind::Remo => {
            k.rod("madeira_ponte", [0.0, 0.015, -0.6], [0.0, 0.015, 0.35], 0.016);
            k.pen("madeira_ponte").cuboid([-0.07, 0.005, 0.35], [0.07, 0.025, 0.7], Faces::ALL);
        }
        ItemKind::LanternaPilha => {
            let xf = mat::rot_z(std::f32::consts::FRAC_PI_2);
            Pen { mesh: k.parts.entry((0, texture::id("metal"))).or_default(), xf, density: 4.0 }.cylinder([0.017, -0.03, 0.0], 0.017, 0.017, 0.06, 6, true);
        }
        ItemKind::Foto => k.card("item_foto_a", 0.06, 0.05),
    }
}

pub fn meshes(tiles: &[[u8; 4]]) -> Vec<Mesh> {
    ALL.iter()
        .map(|&kind| {
            let mut k = Kit { parts: Parts::new() };
            shape(kind, &mut k);
            k.parts.iter().fold(Mesh::default(), |mut m, ((_, tex), part)| {
                let base = m.verts.len() as u16;
                m.verts.extend(part.verts.iter().map(|v| v.with_tile(tiles[*tex as usize])));
                m.idx.extend(part.idx.iter().map(|i| i + base));
                m
            })
        })
        .collect()
}

pub fn alpha(kind: ItemKind) -> bool {
    matches!(kind, ItemKind::BilheteOnibus | ItemKind::Foto)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_item_kind_has_a_small_mesh_in_order() {
        let m = meshes(&vec![[0, 0, 1, 1]; crate::assets::TEXTURES.len()]);
        assert_eq!(m.len(), KINDS);
        ALL.iter().enumerate().for_each(|(i, &k)| {
            assert_eq!(k as usize, i);
            let t = m[i].tris();
            assert!((2..=60).contains(&t), "{i}: {t}");
            assert!(m[i].verts.iter().all(|v| v.pos.iter().all(|c| c.abs() < 0.8)));
        });
    }

    #[test]
    fn bia_sees_glints_three_times_farther() {
        assert_eq!(glint_range(BIA), 3.0 * glint_range(0));
    }
}
