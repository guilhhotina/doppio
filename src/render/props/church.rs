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
        "quadro_negro" => quadro_negro(s),
        "vitral" => wall_panel(s, "pedra", "vitral", [0.5, 1.0], 2.6, 0.04),
        _ => return false,
    }
    true
}

fn banco(s: &mut Sink) {
    let mut w = s.pen("madeira_banco");
    w.cuboid([-0.94, 0.42, -0.18], [0.94, 0.46, 0.22], Faces(0x14));
    w.cuboid([-0.94, 0.56, -0.25], [0.94, 0.65, -0.2], Faces(0x10));
    w.cuboid([-0.94, 0.72, -0.26], [0.94, 0.94, -0.2], Faces(0x30));
    [-0.98f32, 0.91].iter().for_each(|&x| w.cuboid([x, 0.0, -0.32], [x + 0.07, 1.04, 0.24], Faces(0x33)));
    let mut t = s.pen("madeira_crua");
    t.cuboid([-0.98, 0.94, -0.28], [0.98, 0.99, -0.18], Faces(0x14));
    [-0.99f32, 0.9].iter().for_each(|&x| t.cuboid([x, 0.65, -0.16], [x + 0.09, 0.69, 0.26], Faces(0x04)));
    t.cuboid([-0.91, 0.06, 0.1], [0.91, 0.11, 0.23], Faces(0x04));
}

fn altar(s: &mut Sink) {
    s.pen("pedra").cuboid([-1.24, 0.0, -0.62], [1.24, 0.1, 0.62], Faces(0x17));
    s.pen("marmore_sujo").cuboid([-1.12, 0.1, -0.5], [1.12, 0.2, 0.5], Faces(0x17));
    s.pen("marmore_sujo").cuboid([-1.0, 0.2, -0.4], [1.0, 0.96, 0.4], Faces(0x17));
    s.pen("colcha").cuboid([-1.05, 0.94, -0.43], [1.05, 1.01, 0.43], Faces(0x17));
    s.pen("colcha").cuboid([-0.96, 0.45, 0.39], [0.96, 0.94, 0.435], Faces(0x13));
    s.pen("croche").panel([-0.75, 0.32, 0.0], [0.75, 0.96, 0.44]);
    let mut g = s.pen("dourado");
    g.cuboid([-0.88, 1.0, -0.4], [0.88, 2.6, -0.28], Faces(0x17));
    g.cuboid([-0.96, 2.6, -0.44], [0.96, 2.82, -0.22], Faces(0x17));
    g.cuboid([-0.24, 1.01, -0.28], [0.24, 1.44, -0.04], Faces(0x17));
    [-0.58f32, 0.58].iter().for_each(|&x| {
        s.pen("pedra").cuboid([x - 0.16, 1.25, -0.32], [x + 0.16, 2.05, -0.22], Faces(0x17));
        s.pen("cal").cylinder([x, 1.3, -0.24], 0.09, 0.06, 0.58, 4, false);
    });
    s.pen("bronze").cylinder([0.0, 1.44, -0.15], 0.025, 0.025, 0.58, 4, false);
    s.pen("bronze").cuboid([-0.18, 1.8, -0.17], [0.18, 1.85, -0.13], Faces(0x14));
    [(-0.82f32, 0.22f32), (-0.52, 0.3), (0.52, 0.3), (0.82, 0.22)].iter().for_each(|&(x, h)| {
        s.pen("bronze").cylinder([x, 1.01, 0.08], 0.06, 0.022, h, 4, false);
        s.pen("cal").cylinder([x, 1.01 + h, 0.08], 0.026, 0.024, 0.18, 4, false);
        s.mark(MarkKind::Flame, [x, 1.21 + h, 0.08]);
    });
}

fn pulpito(s: &mut Sink) {
    let mut w = s.pen("madeira_banco");
    w.cylinder([0.0, 0.0, 0.0], 0.42, 0.36, 0.12, 5, false);
    w.cylinder([0.0, 0.12, 0.0], 0.13, 0.13, 0.78, 4, false);
    w.cylinder([0.0, 0.9, 0.0], 0.24, 0.52, 0.22, 5, false);
    w.cylinder([0.0, 1.12, 0.0], 0.52, 0.52, 0.66, 5, false);
    let mut g = s.pen("dourado");
    g.cylinder([0.0, 1.12, 0.0], 0.535, 0.535, 0.05, 5, false);
    g.cylinder([0.0, 1.73, 0.0], 0.56, 0.56, 0.07, 5, true);
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
    p.cuboid([-0.32 * k, 0.0, -0.32 * k], [0.32 * k, 0.38 * k, 0.32 * k], SIDES);
    p.cuboid([-0.26 * k, 0.38 * k, -0.26 * k], [0.26 * k, 0.45 * k, 0.26 * k], SIDES);
    p.cylinder([0.0, 0.45 * k, 0.0], 0.25 * k, 0.18 * k, 0.88 * k, 5, false);
    p.cylinder([0.0, 1.33 * k, 0.0], 0.21 * k, 0.09 * k, 0.22 * k, 5, false);
    p.cylinder([0.0, 1.55 * k, 0.0], 0.08 * k, 0.115 * k, 0.14 * k, 5, false);
    p.cylinder([0.0, 1.69 * k, 0.0], 0.115 * k, 0.04 * k, 0.11 * k, 5, true);
    p.cuboid([-0.14 * k, 1.12 * k, 0.04 * k], [0.14 * k, 1.32 * k, 0.24 * k], Faces::NO_BACK);
}

fn santo(s: &mut Sink) {
    figure(s, "marmore_sujo", 1.0);
    s.pen("dourado").cylinder([0.0, 1.85, -0.06], 0.16, 0.16, 0.0, 5, true);
}

fn castical(s: &mut Sink) {
    let mut b = s.pen("bronze");
    b.cylinder([0.0, 0.0, 0.0], 0.09, 0.05, 0.04, 4, false);
    b.cylinder([0.0, 0.04, 0.0], 0.015, 0.015, 0.22, 4, false);
    b.cylinder([0.0, 0.26, 0.0], 0.05, 0.03, 0.03, 4, false);
    s.pen("cal").cylinder([0.0, 0.29, 0.0], 0.022, 0.02, 0.14, 4, false);
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
    w.cuboid([-0.32, 0.68, -0.05], [0.32, 0.72, 0.4], Faces::ALL);
    w.cuboid([-0.28, 0.58, -0.02], [0.28, 0.68, 0.36], Faces(0x33));
    w.cuboid([-0.24, 0.42, -0.45], [0.24, 0.45, -0.1], Faces::ALL);
    w.cuboid([-0.24, 0.6, -0.48], [0.24, 0.85, -0.45], Faces(0x3c));
    legs(&mut s.pen("ferrugem"), 0.29, -0.03, 0.38, 0.68, 0.03);
    legs(&mut s.pen("ferrugem"), 0.21, -0.46, -0.1, 0.42, 0.03);
    s.pen("colcha").cuboid([-0.2, 0.72, 0.05], [0.08, 0.735, 0.29], Faces(0x14));
    s.pen("papel_jornal").cuboid([-0.18, 0.735, 0.06], [0.06, 0.75, 0.28], Faces(0x14));
    s.pen("ceramica_barro").cuboid([0.12, 0.72, 0.18], [0.21, 0.78, 0.26], Faces::NO_BOTTOM);
    s.pen("tecido_sofa").cuboid([0.31, 0.34, 0.04], [0.37, 0.58, 0.3], Faces::NO_BOTTOM);
    s.pen("papel_jornal").cuboid([-0.48, 0.004, -0.14], [-0.22, 0.012, 0.14], Faces(0x04));
}

fn quadro_negro(s: &mut Sink) {
    wall_panel(s, "madeira_escura", "quadro_negro", [0.85, 0.44], 1.42, 0.035);
    s.pen("madeira_crua").cuboid([-0.86, 0.94, 0.0], [0.86, 0.98, 0.09], Faces::NO_BACK);
    s.pen("cal").cuboid([-0.35, 0.98, 0.03], [-0.22, 1.0, 0.06], Faces::NO_BOTTOM);
    s.pen("madeira_escura").cuboid([-0.82, 0.0, 0.0], [0.82, 0.11, 0.56], Faces::NO_BOTTOM_BACK);
    s.pen("papel_jornal").cuboid([-0.32, 0.112, 0.18], [-0.06, 0.12, 0.44], Faces(0x04));
    s.pen("papel_jornal").cuboid([0.12, 0.004, 0.62], [0.42, 0.012, 0.88], Faces(0x04));
    s.pen("madeira_escura").cuboid([-0.02, 1.92, 0.0], [0.02, 2.18, 0.03], Faces::NO_BACK);
    s.pen("madeira_escura").cuboid([-0.09, 2.06, 0.0], [0.09, 2.1, 0.03], Faces::NO_BACK);
}
