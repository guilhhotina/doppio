use super::{MarkKind, Sink, legs, wall_panel};
use crate::render::mesh::Faces;

const SIDES: Faces = Faces::NO_BOTTOM;
const STEPS: usize = 11;
const RISE: f32 = 2.6;

pub fn build(kind: &str, s: &mut Sink) -> bool {
    match kind {
        "escada" => escada(s),
        "fogao_lenha" => fogao_lenha(s),
        "pilao" => pilao(s),
        "moringa" => moringa(s),
        "oratorio" => oratorio(s),
        "maquina_costura" => maquina(s),
        "radio_antigo" => radio(s),
        "calendario" => wall_panel(s, "madeira_escura", "calendario", [0.16, 0.22], 1.55, 0.015),
        "quadro_fusivel" => quadro_fusivel(s),
        "cofre" => cofre(s),
        "bau" => bau(s),
        "caixa_correio" => caixa_correio(s),
        "armario" => armario(s),
        "cesto" => cesto(s),
        _ => return false,
    }
    true
}

pub fn part(kind: &str, s: &mut Sink) -> bool {
    match kind {
        "bau" => {
            s.pen("madeira_crua").cuboid([-0.45, 0.42, -0.25], [0.45, 0.52, 0.25], Faces::ALL);
            s.pen("ferrugem").cuboid([-0.3, 0.42, -0.26], [-0.25, 0.53, 0.26], Faces(0x34));
            s.pen("ferrugem").cuboid([0.25, 0.42, -0.26], [0.3, 0.53, 0.26], Faces(0x34));
        }
        "cofre" => {
            s.pen("metal").cuboid([-0.25, 0.05, 0.25], [0.25, 0.6, 0.29], Faces::ALL);
            s.pen("madeira_escura").cylinder([0.0, 0.32, 0.29], 0.06, 0.06, 0.0, 6, true);
            s.pen("ferrugem").cuboid([0.15, 0.3, 0.29], [0.2, 0.36, 0.32], Faces::NO_BACK);
        }
        "caixa_correio" => s.pen("ferrugem").cuboid([-0.15, 1.05, 0.11], [0.15, 1.25, 0.13], Faces::ALL),
        "armario" => {
            s.pen("madeira_porta").cuboid([0.01, 0.06, 0.25], [0.44, 1.84, 0.27], Faces::ALL);
            s.pen("metal").cuboid([0.04, 0.9, 0.27], [0.07, 1.05, 0.3], Faces::NO_BACK);
        }
        _ => return false,
    }
    true
}

fn escada(s: &mut Sink) {
    let (run, rise) = (3.0 / STEPS as f32, RISE / STEPS as f32);
    let mut p = s.pen("madeira_escura");
    (0..STEPS).for_each(|i| {
        let z0 = -1.5 + i as f32 * run;
        p.cuboid([-0.45, i as f32 * rise, z0], [0.45, (i + 1) as f32 * rise, 1.5], Faces(0x27));
    });
    let mut r = s.pen("madeira_porta");
    [-0.47f32, 0.43].iter().for_each(|&x| {
        r.quad([[x, 0.9, -1.5], [x, 0.9 + RISE, 1.5], [x + 0.04, 0.9 + RISE, 1.5], [x + 0.04, 0.9, -1.5]], [[0.0, 0.0], [3.0, 0.0], [3.0, 0.05], [0.0, 0.05]]);
        r.cuboid([x, 0.0, -1.5], [x + 0.04, 0.95, -1.44], Faces(0x37));
    });
}

fn fogao_lenha(s: &mut Sink) {
    s.pen("tijolo").cuboid([-0.6, 0.0, -0.3], [0.6, 0.78, 0.3], SIDES);
    s.pen("ferrugem").cuboid([-0.62, 0.78, -0.32], [0.62, 0.82, 0.32], Faces::NO_BOTTOM);
    s.pen("ferrugem").cylinder([0.0, 0.82, 0.05], 0.12, 0.1, 0.02, 6, true);
    s.pen("ceramica_barro").cylinder([0.0, 0.84, 0.05], 0.1, 0.12, 0.16, 6, true);
    s.pen("ferrugem").cylinder([0.45, 0.82, -0.2], 0.07, 0.07, 1.9, 5, false);
    s.pen("metal").panel([-0.4, 0.25, 0.0], [-0.05, 0.6, 0.302]);
    s.pen("concreto").panel([0.05, 0.12, 0.0], [0.45, 0.4, 0.301]);
    s.pen("casca").cuboid([0.1, 0.13, 0.22], [0.38, 0.2, 0.33], Faces::NO_BOTTOM);
    s.mark(MarkKind::Flame, [0.25, 0.22, 0.28]);
}

fn pilao(s: &mut Sink) {
    let mut p = s.pen("madeira_crua");
    p.cylinder([0.0, 0.0, 0.0], 0.2, 0.15, 0.3, 6, false);
    p.cylinder([0.0, 0.3, 0.0], 0.15, 0.22, 0.3, 6, true);
    let local = crate::render::mat::mul(&crate::render::mat::translate([0.0, 0.5, 0.0]), &crate::render::mat::rot_z(0.25));
    s.pen_at("madeira_escura", &local).cylinder([0.0, 0.0, 0.0], 0.04, 0.05, 0.9, 4, true);
}

fn moringa(s: &mut Sink) {
    let mut w = s.pen("madeira_escura");
    w.cuboid([-0.2, 0.6, -0.2], [0.2, 0.64, 0.2], Faces::ALL);
    legs(&mut w, 0.18, -0.18, 0.18, 0.6, 0.035);
    let mut c = s.pen("ceramica_barro");
    c.cylinder([0.0, 0.64, 0.0], 0.1, 0.16, 0.14, 7, false);
    c.cylinder([0.0, 0.78, 0.0], 0.16, 0.05, 0.16, 7, false);
    c.cylinder([0.0, 0.94, 0.0], 0.05, 0.06, 0.06, 7, true);
}

fn oratorio(s: &mut Sink) {
    let mut w = s.pen("madeira_escura");
    w.cuboid([-0.24, 1.08, 0.0], [0.24, 1.12, 0.14], Faces::NO_BACK);
    w.cuboid([-0.24, 1.12, 0.0], [-0.21, 1.6, 0.13], Faces(0x17));
    w.cuboid([0.21, 1.12, 0.0], [0.24, 1.6, 0.13], Faces(0x17));
    w.tri([[-0.26, 1.6, 0.14], [0.26, 1.6, 0.14], [0.0, 1.78, 0.14]], [[0.0, 0.3], [1.0, 0.3], [0.5, 0.0]]);
    w.cuboid([-0.26, 1.6, 0.0], [0.26, 1.63, 0.14], Faces::ALL);
    w.cuboid([-0.24, 0.0, 0.04], [0.24, 0.14, 0.36], Faces::NO_BOTTOM);
    s.pen("colcha").cuboid([-0.22, 0.14, 0.06], [0.22, 0.18, 0.34], Faces::NO_BOTTOM);
    let mut g = s.pen("dourado");
    g.panel([-0.21, 1.12, 0.0], [0.21, 1.6, 0.01]);
    g.cuboid([-0.012, 1.76, 0.11], [0.012, 1.9, 0.135], Faces::NO_BOTTOM);
    g.cuboid([-0.05, 1.82, 0.11], [0.05, 1.845, 0.135], Faces::ALL);
    let mut f = s.pen("cal");
    f.cylinder([0.0, 1.12, 0.06], 0.05, 0.035, 0.26, 5, false);
    f.cylinder([0.0, 1.38, 0.06], 0.03, 0.025, 0.07, 5, true);
    s.mark(MarkKind::Flame, [0.12, 1.15, 0.1]);
}

fn maquina(s: &mut Sink) {
    let mut w = s.pen("madeira_escura");
    w.cuboid([-0.45, 0.74, -0.22], [0.45, 0.78, 0.22], Faces::ALL);
    let mut f = s.pen("ferrugem");
    [-0.42f32, 0.38].iter().for_each(|&x| {
        f.quad([[x, 0.0, -0.2], [x, 0.0, 0.2], [x, 0.74, 0.12], [x, 0.74, -0.12]], [[0.0, 0.7], [0.4, 0.7], [0.3, 0.0], [0.1, 0.0]]);
        f.quad([[x + 0.04, 0.0, 0.2], [x + 0.04, 0.0, -0.2], [x + 0.04, 0.74, -0.12], [x + 0.04, 0.74, 0.12]], [[0.0, 0.7], [0.4, 0.7], [0.3, 0.0], [0.1, 0.0]]);
    });
    f.cuboid([-0.3, 0.08, -0.15], [0.3, 0.12, 0.15], Faces(0x3c));
    let mut m = s.pen("metal");
    m.cuboid([-0.22, 0.78, -0.06], [0.2, 0.88, 0.06], Faces::NO_BOTTOM);
    m.cuboid([0.1, 0.88, -0.05], [0.2, 1.06, 0.05], Faces::NO_BOTTOM);
    m.cuboid([-0.22, 0.98, -0.05], [0.1, 1.06, 0.05], Faces::NO_BOTTOM);
    m.cuboid([-0.22, 0.86, -0.03], [-0.18, 0.98, 0.03], Faces(0x33));
}

fn radio(s: &mut Sink) {
    s.pen("madeira_escura").cuboid([-0.2, 0.0, -0.1], [0.2, 0.28, 0.1], Faces::NO_BOTTOM);
    s.pen("tecido_sofa").panel([-0.16, 0.08, 0.0], [0.04, 0.24, 0.103]);
    s.pen("relogio").panel([0.07, 0.12, 0.0], [0.17, 0.22, 0.103]);
}

fn quadro_fusivel(s: &mut Sink) {
    s.pen("metal").cuboid([-0.17, 1.3, 0.0], [0.17, 1.75, 0.1], Faces::NO_BACK);
    s.pen("quadro_fusivel").panel([-0.15, 1.32, 0.0], [0.15, 1.73, 0.104]);
}

fn cofre(s: &mut Sink) {
    let mut m = s.pen("metal");
    m.cuboid([-0.28, 0.0, -0.28], [0.28, 0.65, 0.25], SIDES);
    m.cuboid([-0.28, 0.0, 0.25], [-0.25, 0.65, 0.29], Faces(0x17));
    m.cuboid([0.25, 0.0, 0.25], [0.28, 0.65, 0.29], Faces(0x17));
    m.cuboid([-0.25, 0.0, 0.25], [0.25, 0.05, 0.29], Faces(0x14));
    m.cuboid([-0.25, 0.6, 0.25], [0.25, 0.65, 0.29], Faces(0x14));
    s.pen("madeira_escura").panel([-0.25, 0.05, 0.0], [0.25, 0.6, 0.2]);
}

fn bau(s: &mut Sink) {
    s.pen("madeira_crua").cuboid([-0.45, 0.0, -0.25], [0.45, 0.42, 0.25], Faces(0x33));
    s.pen("madeira_escura").quad([[-0.43, 0.06, 0.23], [0.43, 0.06, 0.23], [0.43, 0.06, -0.23], [-0.43, 0.06, -0.23]], [[0.0, 0.5], [0.9, 0.5], [0.9, 0.0], [0.0, 0.0]]);
    let mut p = s.pen("madeira_escura");
    [[[-0.43, 0.06, -0.23], [0.43, 0.06, -0.23]], [[0.43, 0.06, 0.23], [-0.43, 0.06, 0.23]]].iter().for_each(|e| {
        p.quad([e[0], e[1], [e[1][0], 0.42, e[1][2]], [e[0][0], 0.42, e[0][2]]], [[0.0, 0.4], [0.9, 0.4], [0.9, 0.0], [0.0, 0.0]]);
    });
    let mut f = s.pen("ferrugem");
    [-0.3f32, 0.25].iter().for_each(|&x| f.cuboid([x, 0.0, -0.26], [x + 0.05, 0.42, 0.26], Faces(0x30)));
}

fn caixa_correio(s: &mut Sink) {
    s.pen("madeira_escura").cuboid([-0.04, 0.0, -0.08], [0.04, 1.0, 0.0], Faces(0x33));
    let mut m = s.pen("ferrugem");
    m.cuboid([-0.16, 1.0, -0.2], [0.16, 1.26, 0.11], Faces(0x27));
    m.cuboid([-0.16, 1.0, -0.2], [-0.15, 1.26, 0.11], Faces(0x02));
    m.cuboid([0.15, 1.0, -0.2], [0.16, 1.26, 0.11], Faces(0x01));
    m.cuboid([-0.16, 1.0, -0.2], [0.16, 1.05, 0.11], Faces(0x10));
}

fn armario(s: &mut Sink) {
    let mut p = s.pen("madeira_escura");
    p.cuboid([-0.45, 0.0, -0.25], [0.45, 1.9, 0.25], SIDES);
    s.pen("madeira_porta").panel([-0.44, 0.06, 0.0], [-0.01, 1.84, 0.256]);
    s.pen("metal").cuboid([-0.07, 0.9, 0.25], [-0.04, 1.05, 0.28], Faces::NO_BACK);
}

fn cesto(s: &mut Sink) {
    let mut p = s.pen("sape");
    p.cylinder([0.0, 0.0, 0.0], 0.3, 0.38, 0.75, 8, false);
    p.cylinder([0.0, 0.75, 0.0], 0.39, 0.3, 0.06, 8, true);
}
