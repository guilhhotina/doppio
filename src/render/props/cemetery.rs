use super::church::figure;
use super::{MarkKind, Sink};
use crate::render::mesh::Faces;

const SIDES: Faces = Faces::NO_BOTTOM;

pub fn build(kind: &str, s: &mut Sink) -> bool {
    match kind {
        "lapide" => lapide(s),
        "tumulo" => tumulo(s),
        "cruz_ferro" => s.flat("cruz_ferro_a", [-0.5, 0.0, 0.0], [0.5, 1.6, 0.0]),
        "anjo_estatua" => anjo(s),
        "cova_aberta" => cova(s),
        "ossario" => ossario(s),
        "capela_altar" => capela(s),
        "vela_chao" => vela(s),
        _ => return false,
    }
    true
}

pub fn part(kind: &str, s: &mut Sink) -> bool {
    match kind {
        "tumulo" => s.pen("marmore_sujo").cuboid([-0.47, 0.62, -1.07], [0.47, 0.7, 1.07], Faces::ALL),
        "ossario" => s.pen("lapide_granito").cuboid([-0.3, 0.95, 0.38], [0.3, 1.45, 0.42], Faces::ALL),
        _ => return false,
    }
    true
}

fn lapide(s: &mut Sink) {
    let mut g = s.pen("lapide_granito");
    g.cuboid([-0.32, 0.0, -0.08], [0.32, 0.75, 0.08], SIDES);
    g.cylinder([0.0, 0.75, -0.08], 0.32, 0.0, 0.0, 6, false);
    g.quad([[-0.32, 0.75, 0.08], [0.32, 0.75, 0.08], [0.16, 0.95, 0.08], [-0.16, 0.95, 0.08]], [[0.0, 0.3], [0.6, 0.3], [0.45, 0.0], [0.15, 0.0]]);
    g.quad([[0.32, 0.75, -0.08], [-0.32, 0.75, -0.08], [-0.16, 0.95, -0.08], [0.16, 0.95, -0.08]], [[0.0, 0.3], [0.6, 0.3], [0.45, 0.0], [0.15, 0.0]]);
    g.quad([[-0.16, 0.95, 0.08], [0.16, 0.95, 0.08], [0.16, 0.95, -0.08], [-0.16, 0.95, -0.08]], [[0.0, 0.1], [0.3, 0.1], [0.3, 0.0], [0.0, 0.0]]);
    g.quad([[0.32, 0.75, 0.08], [0.32, 0.75, -0.08], [0.16, 0.95, -0.08], [0.16, 0.95, 0.08]], [[0.0, 0.1], [0.3, 0.1], [0.3, 0.0], [0.0, 0.0]]);
    g.quad([[-0.32, 0.75, -0.08], [-0.32, 0.75, 0.08], [-0.16, 0.95, 0.08], [-0.16, 0.95, -0.08]], [[0.0, 0.1], [0.3, 0.1], [0.3, 0.0], [0.0, 0.0]]);
    s.pen("terra_cova").quad([[-0.4, 0.02, 1.4], [0.4, 0.02, 1.4], [0.4, 0.02, 0.1], [-0.4, 0.02, 0.1]], [[0.0, 1.3], [0.8, 1.3], [0.8, 0.0], [0.0, 0.0]]);
}

fn tumulo(s: &mut Sink) {
    s.pen("marmore_sujo").cuboid([-0.45, 0.0, -1.05], [0.45, 0.62, 1.05], SIDES);
    s.pen("lapide_granito").cuboid([-0.4, 0.7, -1.05], [0.4, 1.3, -0.95], Faces::NO_BOTTOM);
    s.pen("musgo").quad([[-0.46, 0.01, 1.06], [0.46, 0.01, 1.06], [0.46, 0.3, 1.06], [-0.46, 0.3, 1.06]], [[0.0, 0.3], [0.9, 0.3], [0.9, 0.0], [0.0, 0.0]]);
}

fn anjo(s: &mut Sink) {
    figure(s, "marmore_sujo", 0.95);
    let mut w = s.pen("marmore_sujo");
    [-1.0f32, 1.0].iter().for_each(|&side| {
        let q = [[side * 0.08, 0.7, -0.15], [side * 0.08, 1.55, -0.15], [side * 0.55, 1.75, -0.3], [side * 0.45, 1.0, -0.25]];
        let uv = [[0.0, 1.0], [0.0, 0.0], [0.5, 0.0], [0.5, 1.0]];
        w.quad(q, uv);
        w.quad([q[3], q[2], q[1], q[0]], [uv[3], uv[2], uv[1], uv[0]]);
    });
}

fn cova(s: &mut Sink) {
    let mut e = s.pen("terra_cova");
    let (x, z, d) = (0.45f32, 0.95f32, -0.9f32);
    e.quad([[-x, d, z], [x, d, z], [x, d, -z], [-x, d, -z]], [[0.0, 1.9], [0.9, 1.9], [0.9, 0.0], [0.0, 0.0]]);
    let wall = [[0.0, 0.9], [1.9, 0.9], [1.9, 0.0], [0.0, 0.0]];
    e.quad([[-x, d, -z], [x, d, -z], [x, 0.0, -z], [-x, 0.0, -z]], wall);
    e.quad([[x, d, z], [-x, d, z], [-x, 0.0, z], [x, 0.0, z]], wall);
    e.quad([[-x, d, z], [-x, d, -z], [-x, 0.0, -z], [-x, 0.0, z]], wall);
    e.quad([[x, d, -z], [x, d, z], [x, 0.0, z], [x, 0.0, -z]], wall);
    let mut m = s.pen("terra_cova");
    m.quad([[x + 0.05, 0.0, z], [x + 0.75, 0.0, z], [x + 0.35, 0.45, 0.0], [x + 0.1, 0.25, 0.0]], [[0.0, 1.0], [0.7, 1.0], [0.4, 0.0], [0.1, 0.2]]);
    m.quad([[x + 0.75, 0.0, -z], [x + 0.05, 0.0, -z], [x + 0.1, 0.25, 0.0], [x + 0.35, 0.45, 0.0]], [[0.0, 1.0], [0.7, 1.0], [0.6, 0.2], [0.3, 0.0]]);
    m.tri([[x + 0.75, 0.0, z], [x + 0.75, 0.0, -z], [x + 0.35, 0.45, 0.0]], [[0.0, 1.0], [1.0, 1.0], [0.5, 0.0]]);
    let local = crate::render::mat::mul(&crate::render::mat::translate([x + 0.5, 0.0, -0.7]), &crate::render::mat::rot_x(0.3));
    s.pen_at("madeira_crua", &local).cuboid([-0.02, 0.0, -0.02], [0.02, 1.1, 0.02], Faces(0x33));
}

fn ossario(s: &mut Sink) {
    let mut g = s.pen("marmore_sujo");
    g.cuboid([-1.0, 0.0, -0.4], [1.0, 2.2, 0.38], SIDES);
    let mut n = s.pen("lapide_granito");
    (0..3).flat_map(|r| (0..3).map(move |c| (r, c))).filter(|&(r, c)| !(r == 1 && c == 1)).for_each(|(r, c)| {
        let (x, y) = (-0.95 + c as f32 * 0.65, 0.25 + r as f32 * 0.7);
        n.panel([x, y, 0.0], [x + 0.6, y + 0.5, 0.385]);
    });
    s.pen("madeira_escura").panel([-0.3, 0.95, 0.0], [0.3, 1.45, 0.3]);
}

fn capela(s: &mut Sink) {
    s.pen("cal").cuboid([-0.8, 0.0, -0.35], [0.8, 0.9, 0.35], SIDES);
    s.pen("azulejo_capela").panel([-0.75, 0.1, 0.0], [0.75, 0.8, 0.352]);
    s.pen("colcha").cuboid([-0.82, 0.9, -0.37], [0.82, 0.95, 0.37], Faces::NO_BOTTOM);
    s.flat("cruz_ferro_a", [-0.35, 0.95, -0.3], [0.35, 1.85, -0.3]);
    [-0.6f32, 0.6].iter().for_each(|&x| {
        s.pen("cal").cylinder([x, 0.95, 0.0], 0.03, 0.03, 0.22, 4, true);
        s.mark(MarkKind::Flame, [x, 1.22, 0.0]);
    });
}

fn vela(s: &mut Sink) {
    let mut c = s.pen("cal");
    c.cylinder([0.0, 0.0, 0.0], 0.03, 0.028, 0.2, 4, true);
    c.cylinder([0.12, 0.0, 0.06], 0.025, 0.024, 0.13, 4, true);
    s.mark(MarkKind::Flame, [0.0, 0.25, 0.0]);
    s.mark(MarkKind::Flame, [0.12, 0.17, 0.06]);
}
