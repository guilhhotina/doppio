mod cemetery;
mod church;
mod house;
mod mata;
mod town;

use super::cloth;
use super::mat::{self, M4, V3};
use super::mesh::{Faces, Mesh, Pen};
use super::texture;
use std::collections::BTreeMap;

pub type Parts = BTreeMap<(u8, u8), Mesh>;
#[cfg(test)]
pub const KINDS: &[&str] = &[
    "sofa", "poltrona", "tv_tubo", "rack", "cristaleira", "mesa", "cadeira", "cama", "criado_mudo", "guarda_roupa",
    "geladeira", "fogao", "pia", "filtro_barro", "quadro_santo", "relogio_parede", "disjuntor", "estante", "banheira",
    "vaso", "espelho", "cortina", "abajur", "rede", "escada", "fogao_lenha", "pilao", "moringa", "oratorio",
    "maquina_costura", "radio_antigo", "calendario", "quadro_fusivel", "cofre", "bau", "caixa_correio", "armario", "cesto",
    "poste", "orelhao", "banco_praca", "coreto", "chafariz", "fusca", "kombi", "onibus", "bicicleta", "carroca", "placa",
    "banca_jornal", "balcao", "prateleira_padaria", "mesa_bar", "engradado", "geladeira_bar", "caixa_dagua", "muro_baixo",
    "portao_ferro", "lixeira", "letreiro_padaria", "letreiro_bar", "letreiro_farmacia", "letreiro_rodoviaria", "cartaz",
    "caixa_registradora", "carro", "banco_igreja", "altar", "pulpito", "confessionario", "sino", "santo_estatua",
    "vela_castical", "pia_batismal", "carteira_escolar", "quadro_negro", "vitral", "arvore", "bananeira", "bambu",
    "cerca_madeira", "tronco", "pedra", "canoa", "forno_farinha", "poco", "lapide", "tumulo", "cruz_ferro", "anjo_estatua",
    "cova_aberta", "ossario", "capela_altar", "vela_chao",
];

const SIDES: Faces = Faces::NO_BOTTOM;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarkKind {
    Flame,
    Abajur,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    pub kind: MarkKind,
    pub pos: V3,
    pub room: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hinge {
    pub pivot: V3,
    pub axis: usize,
    pub angle: f32,
    pub slide: V3,
}

impl Hinge {
    pub fn at(&self, f: f32) -> M4 {
        let r = match self.axis {
            0 => mat::rot_x(self.angle * f),
            1 => mat::rot_y(self.angle * f),
            _ => mat::rot_z(self.angle * f),
        };
        let back = mat::mul(&r, &mat::translate(mat::scale(self.pivot, -1.0)));
        mat::mul(&mat::translate(mat::add(self.pivot, mat::scale(self.slide, f))), &back)
    }
}

pub struct Sink<'a> {
    pub parts: &'a mut Parts,
    pub screens: &'a mut Mesh,
    pub marks: &'a mut Vec<Mark>,
    pub room: u8,
    pub xf: M4,
}

impl Sink<'_> {
    pub fn pen(&mut self, tex: &str) -> Pen<'_> {
        let mesh = self.parts.entry((self.room, texture::id(tex))).or_default();
        Pen { mesh, xf: self.xf, density: 2.0 }
    }

    pub fn pen_at(&mut self, tex: &str, local: &M4) -> Pen<'_> {
        let xf = mat::mul(&self.xf, local);
        let mesh = self.parts.entry((self.room, texture::id(tex))).or_default();
        Pen { mesh, xf, density: 2.0 }
    }

    pub fn screen(&mut self, lo: V3, hi: V3) {
        Pen { mesh: self.screens, xf: self.xf, density: 1.0 }.panel(lo, hi);
    }

    pub fn mark(&mut self, kind: MarkKind, local: V3) {
        let pos = mat::point(&self.xf, local);
        self.marks.push(Mark { kind, pos, room: self.room });
    }

    pub fn cross(&mut self, tex: &str, c: V3, w: f32, h: f32, n: usize, tilt: f32) {
        let mut p = self.pen(tex);
        (0..n).for_each(|i| {
            let a = i as f32 * std::f32::consts::PI / n as f32 + tilt;
            let (s, k) = (a.sin() * w * 0.5, a.cos() * w * 0.5);
            let q = [[c[0] - k, c[1], c[2] - s], [c[0] + k, c[1], c[2] + s], [c[0] + k, c[1] + h, c[2] + s], [c[0] - k, c[1] + h, c[2] - s]];
            let uv = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
            p.quad(q, uv);
        });
    }

    pub fn flat(&mut self, tex: &str, lo: V3, hi: V3) {
        let q = [[lo[0], lo[1], lo[2]], [hi[0], lo[1], hi[2]], [hi[0], hi[1], hi[2]], [lo[0], hi[1], lo[2]]];
        self.pen(tex).quad(q, [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
    }
}

pub fn surface(kind: &str) -> Option<([f32; 2], f32)> {
    match kind {
        "rack" => Some(([0.6, 0.2], 0.5)),
        "criado_mudo" => Some(([0.22, 0.2], 0.55)),
        "mesa" => Some(([0.8, 0.45], 0.78)),
        "mesa_bar" => Some(([0.4, 0.4], 0.74)),
        "balcao" => Some(([1.0, 0.3], 1.0)),
        "altar" => Some(([1.1, 0.4], 1.0)),
        "capela_altar" => Some(([0.8, 0.35], 0.95)),
        "carteira_escolar" => Some(([0.3, 0.22], 0.72)),
        "engradado" => Some(([0.25, 0.2], 0.3)),
        "oratorio" => Some(([0.25, 0.12], 0.06)),
        "tumulo" => Some(([0.5, 1.05], 0.7)),
        "maquina_costura" => Some(([0.45, 0.22], 0.78)),
        "banca_jornal" => Some(([0.7, 0.4], 0.95)),
        _ => None,
    }
}

pub fn content(kind: &str) -> f32 {
    match kind {
        "bau" | "cesto" | "engradado" => 0.25,
        "cofre" => 0.32,
        "geladeira" | "geladeira_bar" | "guarda_roupa" | "armario" | "estante" | "prateleira_padaria" => 0.95,
        "cristaleira" => 0.45,
        "criado_mudo" => 0.4,
        "caixa_correio" | "quadro_fusivel" => 1.2,
        "oratorio" => 1.3,
        "balcao" => 0.6,
        _ => 0.0,
    }
}

pub fn hinge(kind: &str) -> Option<Hinge> {
    let h = |pivot: V3, axis: usize, angle: f32| Some(Hinge { pivot, axis, angle, slide: [0.0; 3] });
    let slide = |d: V3| Some(Hinge { pivot: [0.0; 3], axis: 1, angle: 0.0, slide: d });
    match kind {
        "bau" => h([0.0, 0.42, -0.25], 0, -1.7),
        "cofre" => h([-0.25, 0.0, 0.25], 1, -1.8),
        "guarda_roupa" => h([0.76, 0.0, 0.306], 1, 1.7),
        "armario" => h([0.45, 0.0, 0.26], 1, 1.7),
        "cristaleira" => h([0.44, 0.0, 0.24], 1, 1.7),
        "geladeira" | "geladeira_bar" => h([-0.35, 0.0, 0.34], 1, -1.8),
        "criado_mudo" | "balcao" => slide([0.0, 0.0, 0.28]),
        "caixa_correio" => h([0.0, 1.05, 0.12], 0, 1.4),
        "caixa_dagua" => h([0.0, 1.4, -0.6], 0, -1.2),
        "portao_ferro" => h([-0.5, 0.0, 0.0], 1, -1.5),
        "tumulo" | "ossario" => slide([0.7, 0.0, 0.0]),
        _ => None,
    }
}

pub fn wheel(s: &mut Sink, tex: &str, c: V3, r: f32, w: f32, outward: f32, sides: usize) {
    let local = mat::mul(&mat::translate(c), &mat::rot_z(outward * std::f32::consts::FRAC_PI_2));
    s.pen_at(tex, &local).cylinder([0.0, -w * 0.5, 0.0], r, r, w, sides, true);
}

pub fn build(kind: &str, s: &mut Sink, supported: bool) -> bool {
    let known = v1(kind, s, supported) || house::build(kind, s) || town::build(kind, s) || church::build(kind, s) || mata::build(kind, s) || cemetery::build(kind, s);
    if !known {
        s.pen("concreto").cuboid([-0.3, 0.0, -0.3], [0.3, 0.6, 0.3], SIDES);
    }
    known
}

pub fn part(kind: &str, s: &mut Sink) {
    let _ = v1_part(kind, s) || house::part(kind, s) || town::part(kind, s) || cemetery::part(kind, s);
}

fn v1(kind: &str, s: &mut Sink, supported: bool) -> bool {
    match kind {
        "sofa" => sofa(s, 0.95, 3),
        "poltrona" => sofa(s, 0.45, 1),
        "tv_tubo" => tv(s),
        "rack" => rack(s),
        "cristaleira" => cristaleira(s),
        "mesa" => mesa(s),
        "cadeira" => cadeira(s),
        "cama" => cama(s),
        "criado_mudo" => criado_mudo(s),
        "guarda_roupa" => guarda_roupa(s),
        "geladeira" => geladeira(s),
        "fogao" => fogao(s),
        "pia" => pia(s),
        "filtro_barro" => filtro(s),
        "quadro_santo" => wall_panel(s, "madeira_escura", "quadro_santo", [0.21, 0.28], 1.6, 0.03),
        "relogio_parede" => relogio(s),
        "disjuntor" => disjuntor(s),
        "estante" => estante(s),
        "banheira" => banheira(s),
        "vaso" => vaso(s),
        "espelho" => wall_panel(s, "metal", "vidro_noite", [0.24, 0.34], 1.5, 0.03),
        "cortina" => cortina(s),
        "abajur" => abajur(s, supported),
        "rede" => rede(s),
        _ => return false,
    }
    true
}

fn v1_part(kind: &str, s: &mut Sink) -> bool {
    match kind {
        "criado_mudo" => s.pen("madeira_porta").cuboid([-0.19, 0.1, 0.0], [0.19, 0.5, 0.206], Faces(0x1f)),
        "cristaleira" => s.pen("madeira_porta").cuboid([0.0, 0.08, 0.226], [0.44, 0.85, 0.24], Faces::ALL),
        "guarda_roupa" => s.pen("madeira_porta").cuboid([0.01, 0.06, 0.29], [0.76, 2.0, 0.306], Faces(0x3f)),
        "geladeira" => {
            s.pen("reboco").cuboid([-0.35, 0.06, 0.33], [0.35, 1.58, 0.36], Faces::ALL);
            s.pen("metal").cuboid([0.24, 0.9, 0.36], [0.28, 1.35, 0.4], Faces::NO_BACK);
        }
        _ => return false,
    }
    true
}

pub fn legs(p: &mut Pen, x: f32, z0: f32, z1: f32, h: f32, t: f32) {
    [(-x, z0), (x - t, z0), (-x, z1 - t), (x - t, z1 - t)]
        .iter()
        .for_each(|&(lx, lz)| p.cuboid([lx, 0.0, lz], [lx + t, h, lz + t], Faces(0x33)));
}

fn doily(s: &mut Sink, y: f32, hx: f32, hz: f32) {
    s.pen("croche").quad(
        [[-hx, y, hz], [hx, y, hz], [hx, y, -hz], [-hx, y, -hz]],
        [[0.08, 0.92], [0.92, 0.92], [0.92, 0.08], [0.08, 0.08]],
    );
}

fn sofa(s: &mut Sink, hw: f32, seats: usize) {
    let mut p = s.pen("tecido_sofa");
    p.cuboid([-hw, 0.1, -0.35], [hw, 0.4, 0.42], SIDES);
    p.cuboid([-hw, 0.1, -0.43], [hw, 0.95, -0.2], SIDES);
    p.cuboid([-hw, 0.4, -0.2], [-hw + 0.17, 0.64, 0.42], Faces::NO_BOTTOM_BACK);
    p.cuboid([hw - 0.17, 0.4, -0.2], [hw, 0.64, 0.42], Faces::NO_BOTTOM_BACK);
    let w = (2.0 * hw - 0.34) / seats as f32;
    (0..seats).for_each(|i| {
        let x = -hw + 0.17 + w * i as f32;
        p.cuboid([x + 0.01, 0.4, -0.2], [x + w - 0.01, 0.5, 0.41], Faces::NO_BOTTOM_BACK);
    });
    legs(&mut s.pen("madeira_escura"), hw - 0.02, -0.4, 0.4, 0.1, 0.05);
    doily(s, 0.956, 0.16, 0.1);
}

fn tv(s: &mut Sink) {
    let mut wood = s.pen("madeira_escura");
    wood.cuboid([-0.33, 0.0, -0.15], [0.33, 0.5, 0.25], Faces::NO_BOTTOM);
    let mut m = s.pen("metal");
    m.cuboid([-0.24, 0.06, -0.38], [0.24, 0.44, -0.15], Faces::NO_BOTTOM);
    m.panel([0.16, 0.08, 0.0], [0.3, 0.42, 0.257]);
    m.cuboid([0.2, 0.3, 0.25], [0.26, 0.36, 0.28], Faces::NO_BACK);
    m.cuboid([0.2, 0.18, 0.25], [0.26, 0.24, 0.28], Faces::NO_BACK);
    [-0.5f32, 0.5].iter().for_each(|&a| {
        let local = mat::mul(&mat::translate([0.0, 0.5, -0.05]), &mat::rot_z(a));
        s.pen_at("metal", &local).cuboid([-0.008, 0.0, -0.008], [0.008, 0.45, 0.008], Faces(0x33));
    });
    s.pen("metal").cuboid([-0.06, 0.5, -0.1], [0.06, 0.54, 0.0], Faces::NO_BOTTOM);
    s.screen([-0.29, 0.07, 0.0], [0.13, 0.43, 0.258]);
}

fn rack(s: &mut Sink) {
    let mut p = s.pen("madeira_escura");
    p.cuboid([-0.6, 0.06, -0.2], [0.6, 0.5, 0.2], SIDES);
    legs(&mut p, 0.58, -0.18, 0.18, 0.06, 0.04);
    s.pen("madeira_porta").panel([-0.56, 0.1, 0.0], [-0.04, 0.46, 0.206]);
    s.pen("madeira_porta").panel([0.04, 0.1, 0.0], [0.56, 0.46, 0.206]);
    doily(s, 0.506, 0.2, 0.15);
}

fn cristaleira(s: &mut Sink) {
    let mut p = s.pen("madeira_escura");
    p.cuboid([-0.5, 0.0, -0.22], [0.5, 1.85, 0.22], SIDES);
    p.cuboid([-0.54, 1.85, -0.24], [0.54, 1.95, 0.26], Faces::ALL);
    s.pen("vidro_noite").panel([-0.42, 0.95, 0.0], [0.42, 1.78, 0.226]);
    s.pen("madeira_porta").panel([-0.44, 0.08, 0.0], [0.0, 0.85, 0.226]);
    let mut c = s.pen("ceramica_barro");
    [1.05f32, 1.45].iter().for_each(|&y| c.cuboid([-0.36, y, 0.05], [0.36, y + 0.18, 0.06], Faces(0x10)));
}

fn mesa(s: &mut Sink) {
    let mut p = s.pen("madeira_escura");
    p.cuboid([-0.8, 0.72, -0.45], [0.8, 0.78, 0.45], Faces::ALL);
    legs(&mut p, 0.74, -0.4, 0.4, 0.72, 0.06);
    doily(s, 0.786, 0.32, 0.32);
}

fn cadeira(s: &mut Sink) {
    let mut p = s.pen("madeira_escura");
    p.cuboid([-0.22, 0.42, -0.2], [0.22, 0.46, 0.22], Faces::ALL);
    legs(&mut p, 0.21, -0.19, 0.21, 0.42, 0.04);
    p.cuboid([-0.21, 0.46, -0.2], [-0.17, 0.95, -0.16], Faces(0x37));
    p.cuboid([0.17, 0.46, -0.2], [0.21, 0.95, -0.16], Faces(0x37));
    p.cuboid([-0.17, 0.68, -0.2], [0.17, 0.92, -0.17], Faces(0x3c));
}

fn cama(s: &mut Sink) {
    let mut w = s.pen("madeira_escura");
    w.cuboid([-0.7, 0.08, -0.97], [0.7, 0.35, 1.0], SIDES);
    w.cuboid([-0.74, 0.0, -1.05], [0.74, 1.1, -0.97], Faces::NO_BOTTOM);
    w.cuboid([-0.74, 0.0, 1.0], [0.74, 0.62, 1.06], Faces::NO_BOTTOM);
    s.pen("colcha").cuboid([-0.69, 0.35, -0.97], [0.69, 0.55, 0.99], Faces::NO_BOTTOM_BACK);
    s.pen("reboco").cuboid([-0.55, 0.55, -0.95], [0.55, 0.66, -0.62], Faces::NO_BOTTOM_BACK);
}

fn criado_mudo(s: &mut Sink) {
    let mut p = s.pen("madeira_escura");
    p.cuboid([-0.22, 0.06, -0.2], [0.22, 0.55, 0.2], SIDES);
    legs(&mut p, 0.21, -0.19, 0.19, 0.06, 0.035);
    s.pen("metal").cuboid([-0.025, 0.4, 0.2], [0.025, 0.43, 0.23], Faces::NO_BACK);
}

fn guarda_roupa(s: &mut Sink) {
    let mut p = s.pen("madeira_escura");
    p.cuboid([-0.8, 0.0, -0.3], [0.8, 2.05, 0.3], SIDES);
    p.cuboid([-0.84, 2.05, -0.32], [0.84, 2.15, 0.33], Faces::NO_BOTTOM);
    s.pen("madeira_porta").panel([-0.76, 0.06, 0.0], [-0.01, 2.0, 0.306]);
    let mut m = s.pen("metal");
    [-0.08f32].iter().for_each(|&x| m.cuboid([x, 0.95, 0.3], [x + 0.03, 1.15, 0.33], Faces::NO_BACK));
}

fn geladeira(s: &mut Sink) {
    let mut p = s.pen("reboco");
    p.cuboid([-0.35, 0.06, -0.33], [0.35, 1.58, 0.33], SIDES);
    p.cuboid([-0.31, 1.58, -0.3], [0.31, 1.66, 0.3], Faces::NO_BOTTOM);
    p.quad(
        [[-0.35, 1.58, 0.33], [0.35, 1.58, 0.33], [0.31, 1.66, 0.3], [-0.31, 1.66, 0.3]],
        [[0.0, 0.2], [1.4, 0.2], [1.3, 0.0], [0.1, 0.0]],
    );
    legs(&mut s.pen("metal"), 0.32, -0.3, 0.3, 0.06, 0.04);
}

fn fogao(s: &mut Sink) {
    let mut p = s.pen("reboco");
    p.cuboid([-0.38, 0.05, -0.3], [0.38, 0.85, 0.3], SIDES);
    p.cuboid([-0.38, 0.85, -0.3], [0.38, 1.02, -0.26], Faces::NO_BOTTOM);
    let mut m = s.pen("metal");
    m.panel([-0.33, 0.12, 0.0], [0.33, 0.58, 0.306]);
    [(-0.17f32, -0.12f32), (0.17, -0.12), (-0.17, 0.14), (0.17, 0.14)]
        .iter()
        .for_each(|&(x, z)| m.cylinder([x, 0.85, z], 0.075, 0.06, 0.025, 6, true));
    (0..4).for_each(|i| {
        let x = -0.27 + i as f32 * 0.18;
        m.cuboid([x, 0.66, 0.3], [x + 0.05, 0.71, 0.33], Faces::NO_BACK);
    });
}

fn pia(s: &mut Sink) {
    let mut m = s.pen("metal");
    m.cuboid([-0.6, 0.0, -0.27], [0.6, 0.82, 0.27], SIDES);
    m.cylinder([0.0, 0.88, -0.22], 0.018, 0.018, 0.26, 4, false);
    m.cuboid([-0.02, 1.08, -0.22], [0.02, 1.12, -0.05], Faces::NO_BACK);
    let mut c = s.pen("ceramica");
    c.cuboid([-0.62, 0.82, -0.29], [0.62, 0.88, 0.29], Faces::NO_BOTTOM);
    s.pen("concreto").quad(
        [[-0.28, 0.886, 0.16], [0.28, 0.886, 0.16], [0.28, 0.886, -0.16], [-0.28, 0.886, -0.16]],
        [[0.0, 0.6], [1.1, 0.6], [1.1, 0.0], [0.0, 0.0]],
    );
    s.pen("madeira_porta").panel([-0.55, 0.06, 0.0], [-0.02, 0.76, 0.276]);
    s.pen("madeira_porta").panel([0.02, 0.06, 0.0], [0.55, 0.76, 0.276]);
}

fn filtro(s: &mut Sink) {
    let mut w = s.pen("madeira_escura");
    w.cuboid([-0.26, 0.74, -0.22], [0.26, 0.8, 0.22], Faces::ALL);
    legs(&mut w, 0.24, -0.2, 0.2, 0.74, 0.04);
    let mut c = s.pen("ceramica_barro");
    c.cylinder([0.0, 0.8, 0.0], 0.15, 0.18, 0.3, 8, false);
    c.cylinder([0.0, 1.1, 0.0], 0.18, 0.15, 0.28, 8, false);
    c.cylinder([0.0, 1.38, 0.0], 0.15, 0.04, 0.07, 8, true);
    s.pen("metal").cuboid([-0.015, 0.88, 0.15], [0.015, 0.91, 0.22], Faces::NO_BACK);
}

pub fn wall_panel(s: &mut Sink, frame: &str, face: &str, half: [f32; 2], y: f32, d: f32) {
    let (lo, hi) = ([-half[0], y - half[1], 0.0], [half[0], y + half[1], d]);
    s.pen(frame).cuboid(lo, hi, Faces::NO_BACK);
    s.pen(face).panel([lo[0] + 0.03, lo[1] + 0.03, 0.0], [hi[0] - 0.03, hi[1] - 0.03, d + 0.006]);
}

fn relogio(s: &mut Sink) {
    s.pen("madeira_escura").cuboid([-0.18, 1.15, 0.0], [0.18, 1.95, 0.13], Faces::NO_BACK);
    s.pen("madeira_escura").cuboid([-0.2, 1.95, 0.0], [0.2, 2.02, 0.15], Faces::NO_BACK);
    s.pen("relogio").panel([-0.15, 1.6, 0.0], [0.15, 1.9, 0.136]);
    s.pen("vidro_noite").panel([-0.12, 1.2, 0.0], [0.12, 1.55, 0.136]);
    s.pen("metal").cuboid([-0.03, 1.24, 0.14], [0.03, 1.3, 0.15], Faces(0x10));
}

fn disjuntor(s: &mut Sink) {
    s.pen("metal").cuboid([-0.18, 1.3, 0.0], [0.18, 1.75, 0.1], Faces::NO_BACK);
    s.pen("concreto").panel([-0.15, 1.34, 0.0], [0.15, 1.7, 0.106]);
    let mut m = s.pen("madeira_escura");
    (0..3).for_each(|i| {
        let x = -0.11 + i as f32 * 0.08;
        m.cuboid([x, 1.48, 0.1], [x + 0.05, 1.56, 0.14], Faces::NO_BACK);
    });
}

fn estante(s: &mut Sink) {
    let mut p = s.pen("madeira_escura");
    p.cuboid([-0.5, 0.0, -0.17], [-0.46, 1.8, 0.17], Faces::NO_BOTTOM_BACK);
    p.cuboid([0.46, 0.0, -0.17], [0.5, 1.8, 0.17], Faces::NO_BOTTOM_BACK);
    p.panel([-0.46, 0.0, 0.0], [0.46, 1.8, -0.16]);
    let shelves = [0.02f32, 0.45, 0.88, 1.31, 1.76];
    shelves.iter().for_each(|&y| p.cuboid([-0.46, y, -0.16], [0.46, y + 0.04, 0.17], Faces(0x1c)));
    let mut b = s.pen("papel_jornal");
    shelves[..4].iter().enumerate().for_each(|(i, &y)| {
        let cut = [0.1f32, -0.2, 0.25, -0.05][i];
        b.cuboid([-0.44, y + 0.04, -0.13], [cut, y + 0.34, 0.12], Faces(0x17));
        b.cuboid([cut + 0.02, y + 0.04, -0.13], [0.42, y + 0.27 + 0.05 * i as f32, 0.1], Faces(0x17));
    });
}

fn banheira(s: &mut Sink) {
    let mut p = s.pen("reboco");
    let (x, z, h, t) = (0.8f32, 0.37f32, 0.55f32, 0.07f32);
    p.cuboid([-x, 0.0, -z], [x, h, z], Faces(0x33));
    p.cuboid([-x, h - 0.02, -z], [x, h, -z + t], Faces(0x04));
    p.cuboid([-x, h - 0.02, z - t], [x, h, z], Faces(0x04));
    p.cuboid([-x, h - 0.02, -z + t], [-x + t, h, z - t], Faces(0x04));
    p.cuboid([x - t, h - 0.02, -z + t], [x, h, z - t], Faces(0x04));
    let (ix, iz, b) = (x - t, z - t, 0.12f32);
    p.quad([[-ix, b, iz], [ix, b, iz], [ix, b, -iz], [-ix, b, -iz]], [[0.0, 1.0], [2.0, 1.0], [2.0, 0.0], [0.0, 0.0]]);
    let wall = [[0.0, 0.8], [2.0, 0.8], [2.0, 0.0], [0.0, 0.0]];
    p.quad([[-ix, b, -iz], [ix, b, -iz], [ix, h, -iz], [-ix, h, -iz]], wall);
    p.quad([[ix, b, iz], [-ix, b, iz], [-ix, h, iz], [ix, h, iz]], wall);
    p.quad([[-ix, b, iz], [-ix, b, -iz], [-ix, h, -iz], [-ix, h, iz]], wall);
    p.quad([[ix, b, -iz], [ix, b, iz], [ix, h, iz], [ix, h, -iz]], wall);
    s.pen("metal").cylinder([-0.7, 0.55, 0.0], 0.02, 0.02, 0.2, 4, false);
}

fn vaso(s: &mut Sink) {
    let mut p = s.pen("reboco");
    p.cylinder([0.0, 0.0, 0.05], 0.13, 0.19, 0.38, 8, true);
    p.cuboid([-0.19, 0.38, -0.32], [0.19, 0.78, -0.14], Faces::NO_BOTTOM);
    s.pen("concreto").cylinder([0.0, 0.386, 0.05], 0.13, 0.13, 0.001, 8, true);
}

fn cortina(s: &mut Sink) {
    s.pen("metal").cuboid([-0.75, 2.3, 0.06], [0.75, 2.33, 0.09], Faces(0x17));
    let mut p = s.pen("colcha");
    cloth::quads(1, |q, uv| p.quad(q, uv));
}

fn abajur_dims(supported: bool) -> (f32, f32, f32) {
    if supported { (0.28, 0.18, 0.15) } else { (1.35, 0.28, 0.24) }
}

fn abajur(s: &mut Sink, supported: bool) {
    let (stem, shade, r) = abajur_dims(supported);
    let mut m = s.pen("metal");
    m.cylinder([0.0, 0.0, 0.0], 0.09, 0.07, 0.03, 6, true);
    m.cylinder([0.0, 0.03, 0.0], 0.012, 0.012, stem, 4, false);
    s.pen("colcha").cylinder([0.0, stem - 0.02, 0.0], r, r * 0.6, shade, 6, false);
    s.mark(MarkKind::Abajur, [0.0, stem - 0.02 + shade * 0.5, 0.0]);
}

fn rede(s: &mut Sink) {
    let mut p = s.pen("colcha");
    let n = 8;
    let y = |t: f32, side: f32| 1.35 - (1.0 - (2.0 * t - 1.0).powi(2)) * (0.7 - side * 0.15);
    let pt = |i: usize, x: f32, side: f32| {
        let t = i as f32 / n as f32;
        [x * (0.15 + 0.85 * (1.0 - (2.0 * t - 1.0).powi(2))), y(t, side), -1.3 + 2.6 * t]
    };
    (0..n).for_each(|i| {
        [(-0.4f32, -0.13f32, 1.0f32, 0.0f32), (-0.13, 0.13, 0.0, 0.0), (0.13, 0.4, 0.0, 1.0)]
            .iter()
            .for_each(|&(xa, xb, sa, sb)| {
                let q = [pt(i, xa, sa), pt(i, xb, sb), pt(i + 1, xb, sb), pt(i + 1, xa, sa)];
                let uv = [[0.0, 0.0], [0.5, 0.0], [0.5, 0.6], [0.0, 0.6]];
                p.quad(q, uv);
                p.quad([q[1], q[0], q[3], q[2]], [uv[1], uv[0], uv[3], uv[2]]);
            });
    });
    let mut m = s.pen("madeira_escura");
    [-1.3f32, 1.3].iter().for_each(|&z| m.cylinder([0.0, 1.35, z], 0.01, 0.01, 0.6, 3, false));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    pub fn build_one(kind: &str) -> (usize, usize, usize, bool) {
        let (mut parts, mut screens, mut marks) = (Parts::new(), Mesh::default(), Vec::new());
        let mut sink = Sink { parts: &mut parts, screens: &mut screens, marks: &mut marks, room: 0, xf: mat::IDENTITY };
        let known = build(kind, &mut sink, false);
        let mut moving = Parts::new();
        part(kind, &mut Sink { parts: &mut moving, screens: &mut screens, marks: &mut marks, room: 0, xf: mat::IDENTITY });
        (parts.values().map(Mesh::tris).sum::<usize>(), screens.tris(), moving.values().map(Mesh::tris).sum::<usize>(), known)
    }

    #[test]
    fn catalog_is_low_poly_and_known() {
        let big = ["coreto", "onibus", "kombi", "fusca", "carro", "carroca", "chafariz", "altar", "arvore", "ponte", "bicicleta", "capela_altar", "maquina_costura"];
        KINDS.iter().for_each(|k| {
            let (t, _, _, known) = build_one(k);
            let cap = if big.contains(k) { 420 } else { 200 };
            assert!((2..=cap).contains(&t), "{k}: {t} tris");
            assert!(known, "{k} fell back to the placeholder");
        });
    }

    #[test]
    fn movers_have_a_moving_part() {
        KINDS.iter().filter(|k| hinge(k).is_some_and(|h| h.angle != 0.0 || h.slide != [0.0; 3])).for_each(|k| {
            assert!(build_one(k).2 > 0, "{k} has a hinge but no part");
        });
    }

    #[test]
    fn hinge_rotates_about_its_pivot() {
        let h = hinge("bau").expect("bau");
        assert!(mat::len(mat::sub(mat::point(&h.at(1.0), h.pivot), h.pivot)) < 1e-5);
        let lid = mat::point(&h.at(1.0), [0.0, 0.42, 0.2]);
        assert!(lid[1] > 0.7, "{lid:?}");
    }

    #[test]
    fn tv_has_screen() {
        assert_eq!(build_one("tv_tubo").1, 2);
    }

    #[test]
    fn sample_map_props_are_cataloged() {
        let m = map::parse(include_str!("testdata/rua.map")).expect("rua");
        m.props.iter().for_each(|p| assert!(KINDS.contains(&p.kind), "unknown prop {}", p.kind));
    }
}
