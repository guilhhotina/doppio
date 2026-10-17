use super::{MarkKind, Sink, legs, wall_panel};
use crate::render::mesh::Faces;

const SIDES: Faces = Faces::NO_BOTTOM;

pub fn build(kind: &str, s: &mut Sink) -> bool {
    match kind {
        "banco_igreja" => banco(s),
        "altar" => altar(s),
        "pulpito" => pulpito(s),
        "confessionario" => confessionario(s),
        "sino" => sino(s),
        "santo_estatua" => santo(s),
        "vela_castical" => castical(s),
        "pia_batismal" => pia(s),
        "carteira_escolar" => carteira(s),
        "quadro_negro" => wall_panel(s, "madeira_escura", "quadro_negro", [0.8, 0.4], 1.4, 0.03),
        "vitral" => wall_panel(s, "pedra", "vitral", [0.5, 1.0], 2.6, 0.04),
        _ => return false,
    }
    true
}

fn banco(s: &mut Sink) {
    let mut w = s.pen("madeira_banco");
    w.cuboid([-0.95, 0.42, -0.2], [0.95, 0.46, 0.2], Faces(0x3c));
    w.cuboid([-0.95, 0.46, -0.26], [0.95, 0.95, -0.2], Faces(0x3c));
    w.cuboid([-0.95, 0.0, -0.36], [0.95, 0.75, -0.3], Faces(0x3c));
    [-0.97f32, 0.91].iter().for_each(|&x| w.cuboid([x, 0.0, -0.36], [x + 0.06, 1.0, 0.22], Faces(0x37)));
}

fn altar(s: &mut Sink) {
    s.pen("marmore_sujo").cuboid([-1.0, 0.0, -0.4], [1.0, 0.96, 0.4], SIDES);
    s.pen("colcha").cuboid([-1.04, 0.96, -0.42], [1.04, 1.0, 0.42], Faces::NO_BOTTOM);
    s.pen("croche").panel([-0.6, 0.3, 0.0], [0.6, 0.96, 0.41]);
    s.pen("dourado").cuboid([-0.8, 1.0, -0.4], [0.8, 2.6, -0.3], Faces(0x37));
    s.pen("dourado").cuboid([-0.9, 2.6, -0.42], [0.9, 2.8, -0.25], Faces::ALL);
    s.pen("bronze").cylinder([0.0, 1.0, -0.15], 0.02, 0.02, 0.5, 4, false);
    s.pen("bronze").cuboid([-0.15, 1.38, -0.17], [0.15, 1.42, -0.13], Faces(0x3c));
    [-0.7f32, 0.7].iter().for_each(|&x| {
        s.pen("bronze").cylinder([x, 1.0, 0.0], 0.06, 0.02, 0.25, 5, false);
        s.pen("cal").cylinder([x, 1.25, 0.0], 0.025, 0.025, 0.18, 4, true);
        s.mark(MarkKind::Flame, [x, 1.45, 0.0]);
    });
}

fn pulpito(s: &mut Sink) {
    let mut w = s.pen("madeira_banco");
    w.cylinder([0.0, 0.0, 0.0], 0.42, 0.36, 0.12, 6, true);
    w.cylinder([0.0, 0.12, 0.0], 0.13, 0.13, 0.78, 6, false);
    w.cylinder([0.0, 0.9, 0.0], 0.24, 0.52, 0.22, 6, false);
    w.cylinder([0.0, 1.12, 0.0], 0.52, 0.52, 0.66, 6, false);
    let mut g = s.pen("dourado");
    g.cylinder([0.0, 1.12, 0.0], 0.535, 0.535, 0.05, 6, false);
    g.cylinder([0.0, 1.73, 0.0], 0.56, 0.56, 0.07, 6, true);
    s.pen("madeira_banco").quad([[-0.28, 1.8, 0.18], [0.28, 1.8, 0.18], [0.28, 1.94, 0.46], [-0.28, 1.94, 0.46]], [[0.0, 0.5], [1.0, 0.5], [1.0, 0.0], [0.0, 0.0]]);
}

fn confessionario(s: &mut Sink) {
    let mut w = s.pen("madeira_banco");
    w.cuboid([-0.5, 0.0, -0.45], [0.5, 2.2, -0.4], Faces(0x3c));
    [-0.5f32, 0.45].iter().for_each(|&x| w.cuboid([x, 0.0, -0.45], [x + 0.05, 2.2, 0.45], Faces(0x37)));
    w.cuboid([-0.55, 2.2, -0.5], [0.55, 2.35, 0.5], Faces::ALL);
    w.cuboid([-0.45, 1.6, 0.4], [0.45, 2.2, 0.45], Faces(0x30));
    s.pen("tecido_sofa").panel([-0.43, 0.15, 0.0], [0.43, 1.6, 0.42]);
}

fn sino(s: &mut Sink) {
    s.pen("madeira_escura").cuboid([-0.6, 3.0, -0.06], [0.6, 3.12, 0.06], Faces::ALL);
    let mut b = s.pen("bronze");
    b.cylinder([0.0, 2.38, 0.0], 0.38, 0.3, 0.12, 8, false);
    b.cylinder([0.0, 2.5, 0.0], 0.3, 0.2, 0.4, 8, false);
    b.cylinder([0.0, 2.9, 0.0], 0.2, 0.05, 0.1, 8, true);
    b.cylinder([0.0, 2.25, 0.0], 0.05, 0.05, 0.25, 4, true);
}

pub fn figure(s: &mut Sink, tex: &str, scale: f32) {
    let k = scale;
    let mut p = s.pen(tex);
    p.cuboid([-0.3 * k, 0.0, -0.3 * k], [0.3 * k, 0.4 * k, 0.3 * k], SIDES);
    p.cylinder([0.0, 0.4 * k, 0.0], 0.26 * k, 0.17 * k, 0.95 * k, 6, false);
    p.cylinder([0.0, 1.35 * k, 0.0], 0.17 * k, 0.1 * k, 0.2 * k, 6, false);
    p.cylinder([0.0, 1.55 * k, 0.0], 0.06 * k, 0.11 * k, 0.12 * k, 6, false);
    p.cylinder([0.0, 1.67 * k, 0.0], 0.11 * k, 0.03 * k, 0.12 * k, 6, true);
    p.cuboid([-0.08 * k, 1.2 * k, 0.05 * k], [0.08 * k, 1.3 * k, 0.24 * k], Faces::NO_BACK);
}

fn santo(s: &mut Sink) {
    figure(s, "marmore_sujo", 1.0);
    s.pen("dourado").cylinder([0.0, 1.85, -0.06], 0.16, 0.16, 0.0, 6, true);
}

fn castical(s: &mut Sink) {
    let mut b = s.pen("bronze");
    b.cylinder([0.0, 0.0, 0.0], 0.09, 0.05, 0.04, 6, true);
    b.cylinder([0.0, 0.04, 0.0], 0.015, 0.015, 0.22, 4, false);
    b.cylinder([0.0, 0.26, 0.0], 0.05, 0.03, 0.03, 6, true);
    s.pen("cal").cylinder([0.0, 0.29, 0.0], 0.022, 0.02, 0.14, 4, true);
    s.mark(MarkKind::Flame, [0.0, 0.45, 0.0]);
}

fn pia(s: &mut Sink) {
    let mut m = s.pen("marmore_sujo");
    m.cylinder([0.0, 0.0, 0.0], 0.25, 0.15, 0.12, 8, false);
    m.cylinder([0.0, 0.12, 0.0], 0.1, 0.1, 0.65, 6, false);
    m.cylinder([0.0, 0.77, 0.0], 0.15, 0.45, 0.2, 8, false);
    m.cylinder([0.0, 0.97, 0.0], 0.45, 0.42, 0.04, 8, false);
    s.pen("agua").cylinder([0.0, 0.94, 0.0], 0.4, 0.4, 0.0, 8, true);
}

fn carteira(s: &mut Sink) {
    let mut w = s.pen("madeira_crua");
    w.cuboid([-0.3, 0.68, -0.05], [0.3, 0.72, 0.4], Faces::ALL);
    w.cuboid([-0.22, 0.42, -0.45], [0.22, 0.45, -0.1], Faces::ALL);
    w.cuboid([-0.22, 0.6, -0.48], [0.22, 0.85, -0.45], Faces(0x3c));
    legs(&mut s.pen("ferrugem"), 0.28, -0.03, 0.38, 0.68, 0.03);
    legs(&mut s.pen("ferrugem"), 0.2, -0.46, -0.1, 0.42, 0.03);
}
