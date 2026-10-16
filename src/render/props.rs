use super::mat::{self, M4, V3};
use super::mesh::{Faces, Mesh, Pen};
use super::texture;
use std::collections::BTreeMap;

pub type Parts = BTreeMap<(u8, u8), Mesh>;

#[cfg(test)]
pub const KINDS: [&str; 24] = [
    "sofa", "poltrona", "tv_tubo", "rack", "cristaleira", "mesa", "cadeira", "cama", "criado_mudo", "guarda_roupa",
    "geladeira", "fogao", "pia", "filtro_barro", "quadro_santo", "relogio_parede", "disjuntor", "estante", "banheira",
    "vaso", "espelho", "cortina", "abajur", "rede",
];

const SIDES: Faces = Faces::NO_BOTTOM;

pub struct Sink<'a> {
    pub parts: &'a mut Parts,
    pub screens: &'a mut Mesh,
    pub room: u8,
    pub xf: M4,
}

impl Sink<'_> {
    fn pen(&mut self, tex: &str) -> Pen<'_> {
        let mesh = self.parts.entry((self.room, texture::id(tex))).or_default();
        Pen { mesh, xf: self.xf, density: 2.0 }
    }

    fn pen_at(&mut self, tex: &str, local: &M4) -> Pen<'_> {
        let xf = mat::mul(&self.xf, local);
        let mesh = self.parts.entry((self.room, texture::id(tex))).or_default();
        Pen { mesh, xf, density: 2.0 }
    }

    fn screen(&mut self, lo: V3, hi: V3) {
        Pen { mesh: self.screens, xf: self.xf, density: 1.0 }.panel(lo, hi);
    }
}

pub fn surface(kind: &str) -> Option<([f32; 2], f32)> {
    match kind {
        "rack" => Some(([0.6, 0.2], 0.5)),
        "criado_mudo" => Some(([0.22, 0.2], 0.55)),
        "mesa" => Some(([0.8, 0.45], 0.78)),
        _ => None,
    }
}

pub fn build(kind: &str, s: &mut Sink, supported: bool) {
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
        _ => s.pen("concreto").cuboid([-0.3, 0.0, -0.3], [0.3, 0.6, 0.3], SIDES),
    }
}

fn legs(p: &mut Pen, x: f32, z0: f32, z1: f32, h: f32, t: f32) {
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
    s.pen("madeira_porta").panel([0.0, 0.08, 0.0], [0.44, 0.85, 0.226]);
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
    s.pen("madeira_porta").panel([-0.19, 0.1, 0.0], [0.19, 0.5, 0.206]);
    s.pen("metal").cuboid([-0.025, 0.4, 0.2], [0.025, 0.43, 0.23], Faces::NO_BACK);
}

fn guarda_roupa(s: &mut Sink) {
    let mut p = s.pen("madeira_escura");
    p.cuboid([-0.8, 0.0, -0.3], [0.8, 2.05, 0.3], SIDES);
    p.cuboid([-0.84, 2.05, -0.32], [0.84, 2.15, 0.33], Faces::NO_BOTTOM);
    s.pen("madeira_porta").panel([-0.76, 0.06, 0.0], [-0.01, 2.0, 0.306]);
    s.pen("madeira_porta").panel([0.01, 0.06, 0.0], [0.76, 2.0, 0.306]);
    let mut m = s.pen("metal");
    [-0.08f32, 0.05].iter().for_each(|&x| m.cuboid([x, 0.95, 0.3], [x + 0.03, 1.15, 0.33], Faces::NO_BACK));
}

fn geladeira(s: &mut Sink) {
    let mut p = s.pen("reboco");
    p.cuboid([-0.35, 0.06, -0.33], [0.35, 1.58, 0.33], SIDES);
    p.cuboid([-0.31, 1.58, -0.3], [0.31, 1.66, 0.3], Faces::NO_BOTTOM);
    p.quad(
        [[-0.35, 1.58, 0.33], [0.35, 1.58, 0.33], [0.31, 1.66, 0.3], [-0.31, 1.66, 0.3]],
        [[0.0, 0.2], [1.4, 0.2], [1.3, 0.0], [0.1, 0.0]],
    );
    let mut m = s.pen("metal");
    m.cuboid([0.24, 0.9, 0.33], [0.28, 1.35, 0.37], Faces::NO_BACK);
    m.cuboid([-0.35, 1.08, 0.33], [0.35, 1.1, 0.335], Faces(0x10));
    legs(&mut m, 0.32, -0.3, 0.3, 0.06, 0.04);
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

fn wall_panel(s: &mut Sink, frame: &str, face: &str, half: [f32; 2], y: f32, d: f32) {
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
    [-0.72f32, 0.36].iter().for_each(|&x0| {
        (0..4).for_each(|i| {
            let (a, b) = (x0 + i as f32 * 0.09, x0 + (i + 1) as f32 * 0.09);
            let (za, zb) = if i % 2 == 0 { (0.05, 0.12) } else { (0.12, 0.05) };
            p.quad(
                [[a, 0.25, za], [b, 0.25, zb], [b, 2.3, zb], [a, 2.3, za]],
                [[i as f32 * 0.2, 4.0], [(i + 1) as f32 * 0.2, 4.0], [(i + 1) as f32 * 0.2, 0.0], [i as f32 * 0.2, 0.0]],
            );
        });
    });
}

fn abajur(s: &mut Sink, supported: bool) {
    let (stem, shade, r) = if supported { (0.28f32, 0.18f32, 0.15f32) } else { (1.35, 0.28, 0.24) };
    let mut m = s.pen("metal");
    m.cylinder([0.0, 0.0, 0.0], 0.09, 0.07, 0.03, 6, true);
    m.cylinder([0.0, 0.03, 0.0], 0.012, 0.012, stem, 4, false);
    s.pen("colcha").cylinder([0.0, stem - 0.02, 0.0], r, r * 0.6, shade, 6, false);
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

    fn build_one(kind: &str) -> (usize, usize) {
        let (mut parts, mut screens) = (Parts::new(), Mesh::default());
        build(kind, &mut Sink { parts: &mut parts, screens: &mut screens, room: 0, xf: mat::IDENTITY }, false);
        (parts.values().map(Mesh::tris).sum::<usize>(), screens.tris())
    }

    #[test]
    fn catalog_is_low_poly() {
        KINDS.iter().for_each(|k| {
            let (t, _) = build_one(k);
            assert!((8..=160).contains(&t), "{k}: {t} tris");
        });
    }

    #[test]
    fn map_props_are_cataloged() {
        map::sobrado().props.iter().for_each(|p| assert!(KINDS.contains(&p.kind), "unknown prop {}", p.kind));
    }

    #[test]
    fn tv_has_screen() {
        assert_eq!(build_one("tv_tubo").1, 2);
    }
}
