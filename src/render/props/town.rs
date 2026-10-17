use super::{Sink, legs, wheel};
use crate::render::mat;
use crate::render::mesh::Faces;
use std::f32::consts::{PI, TAU};

const SIDES: Faces = Faces::NO_BOTTOM;

pub fn build(kind: &str, s: &mut Sink) -> bool {
    match kind {
        "poste" => poste(s),
        "orelhao" => orelhao(s),
        "banco_praca" => banco(s),
        "coreto" => coreto(s),
        "chafariz" => chafariz(s),
        "fusca" | "carro" => fusca(s),
        "kombi" => kombi(s),
        "onibus" => onibus(s),
        "bicicleta" => bicicleta(s),
        "carroca" => carroca(s),
        "placa" => placa(s),
        "banca_jornal" => banca(s),
        "balcao" => balcao(s),
        "prateleira_padaria" => prateleira(s),
        "mesa_bar" => mesa_bar(s),
        "engradado" => engradado(s),
        "geladeira_bar" => geladeira_bar(s),
        "caixa_dagua" => caixa_dagua(s),
        "muro_baixo" => muro(s),
        "portao_ferro" => portao(s),
        "lixeira" => lixeira(s),
        "letreiro_padaria" | "letreiro_bar" | "letreiro_farmacia" | "letreiro_rodoviaria" => letreiro(s, kind),
        "cartaz" => s.pen("cartaz_festa").panel([-0.3, 1.2, -0.1], [0.3, 1.8, -0.09]),
        "caixa_registradora" => caixa(s),
        _ => return false,
    }
    true
}

pub fn part(kind: &str, s: &mut Sink) -> bool {
    match kind {
        "geladeira_bar" => {
            s.pen("metal").cuboid([-0.35, 0.1, 0.3], [0.35, 1.75, 0.34], Faces(0x33));
            s.pen("vitrine").panel([-0.3, 0.2, 0.0], [0.3, 1.65, 0.342]);
        }
        "caixa_dagua" => s.pen("concreto").cylinder([0.0, 1.4, 0.0], 0.62, 0.15, 0.2, 8, true),
        "portao_ferro" => {
            let mut p = s.pen("portao_a");
            let q = [[-0.48, 0.0, 0.0], [0.48, 0.0, 0.0], [0.48, 1.9, 0.0], [-0.48, 1.9, 0.0]];
            let uv = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
            p.quad(q, uv);
        }
        "balcao" => s.pen("madeira_porta").cuboid([-0.4, 0.6, 0.24], [0.4, 0.8, 0.3], Faces::ALL),
        _ => return false,
    }
    true
}

fn poste(s: &mut Sink) {
    let mut f = s.pen("ferrugem");
    f.cylinder([0.0, 0.0, 0.0], 0.11, 0.09, 0.5, 6, false);
    f.cylinder([0.0, 0.5, 0.0], 0.07, 0.05, 3.75, 5, true);
    f.cuboid([-0.03, 4.12, -0.03], [0.03, 4.18, 0.8], Faces(0x37));
    let mut m = s.pen("metal");
    m.cylinder([0.0, 3.95, 0.8], 0.06, 0.24, 0.17, 6, false);
    m.cylinder([0.0, 4.12, 0.8], 0.24, 0.05, 0.08, 6, true);
    s.pen("cal").cylinder([0.0, 3.84, 0.8], 0.05, 0.08, 0.12, 5, true);
    s.mark(super::MarkKind::Abajur, [0.0, 3.86, 0.8]);
}

fn orelhao(s: &mut Sink) {
    s.pen("ferrugem").cylinder([0.0, 0.0, -0.2], 0.04, 0.04, 1.4, 4, false);
    let mut h = s.pen("ceramica_barro");
    (0..6).for_each(|i| {
        let (a0, a1) = (PI * (0.15 + 0.7 * i as f32 / 6.0), PI * (0.15 + 0.7 * (i + 1) as f32 / 6.0));
        let p = |a: f32, y: f32, r: f32| [a.cos() * r * 0.38, y, -0.2 + a.sin() * r * 0.38];
        h.quad([p(a1, 1.2, 1.0), p(a0, 1.2, 1.0), p(a0, 2.15, 0.85), p(a1, 2.15, 0.85)], [[0.0, 1.0], [0.4, 1.0], [0.4, 0.0], [0.0, 0.0]]);
        h.quad([p(a0, 1.2, 0.97), p(a1, 1.2, 0.97), p(a1, 2.15, 0.82), p(a0, 2.15, 0.82)], [[0.0, 1.0], [0.4, 1.0], [0.4, 0.0], [0.0, 0.0]]);
    });
    s.pen("metal").cuboid([-0.1, 1.35, -0.14], [0.1, 1.7, -0.06], Faces::NO_BACK);
}

fn banco(s: &mut Sink) {
    let mut w = s.pen("madeira_crua");
    (0..3).for_each(|i| {
        let z = -0.2 + i as f32 * 0.13;
        w.cuboid([-0.85, 0.42, z], [0.85, 0.45, z + 0.1], Faces(0x3c));
    });
    (0..2).for_each(|i| {
        let y = 0.6 + i as f32 * 0.17;
        w.cuboid([-0.85, y, -0.24], [0.85, y + 0.11, -0.21], Faces(0x3c));
    });
    let mut f = s.pen("ferrugem");
    [-0.7f32, 0.66].iter().for_each(|&x| {
        f.cuboid([x, 0.0, -0.22], [x + 0.04, 0.42, 0.2], Faces(0x33));
        f.cuboid([x, 0.42, -0.25], [x + 0.04, 0.9, -0.2], Faces(0x33));
    });
}

fn coreto(s: &mut Sink) {
    let ring = |i: usize, r: f32, y: f32| {
        let a = (i as f32 + 0.5) / 8.0 * TAU;
        [a.sin() * r, y, a.cos() * r]
    };
    let mut b = s.pen("cal");
    (0..8).for_each(|i| b.quad([ring(i, 1.45, 0.0), ring(i + 1, 1.45, 0.0), ring(i + 1, 1.45, 0.6), ring(i, 1.45, 0.6)], [[0.0, 0.6], [1.1, 0.6], [1.1, 0.0], [0.0, 0.0]]));
    let mut fl = s.pen("ladrilho_hidraulico");
    (1..7).for_each(|i| fl.tri([ring(0, 1.45, 0.6), ring(i + 1, 1.45, 0.6), ring(i, 1.45, 0.6)], [[0.0, 0.0], [1.0, 1.0], [0.0, 1.0]]));
    let mut post = s.pen("madeira_escura");
    (0..8).map(|i| ring(i, 1.33, 0.6)).for_each(|p| post.cuboid([p[0] - 0.05, 0.6, p[2] - 0.05], [p[0] + 0.05, 3.0, p[2] + 0.05], Faces(0x33)));
    let mut roof = s.pen("telha");
    let apex = [0.0, 3.9, 0.0];
    (0..8).for_each(|i| {
        roof.tri([ring(i + 1, 1.7, 3.0), apex, ring(i, 1.7, 3.0)], [[1.2, 1.0], [0.6, 0.0], [0.0, 1.0]]);
        roof.tri([ring(i, 1.7, 3.0), apex, ring(i + 1, 1.7, 3.0)], [[0.0, 1.0], [0.6, 0.0], [1.2, 1.0]]);
    });
    let mut rail = s.pen("grade_a");
    (0..8).filter(|&i| i != 0 && i != 7).for_each(|i| {
        let (a, b) = (ring(i, 1.33, 0.6), ring(i + 1, 1.33, 0.6));
        let q = [a, b, [b[0], 1.5, b[2]], [a[0], 1.5, a[2]]];
        let uv = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.1], [0.0, 0.1]];
        rail.quad(q, uv);
    });
    let mut st = s.pen("cal");
    (0..3).for_each(|i| {
        let (z, y) = (1.35 + i as f32 * 0.25, 0.6 - (i + 1) as f32 * 0.15);
        st.cuboid([-0.5, 0.0, z], [0.5, y + 0.15, z + 0.25], Faces(0x17));
    });
}

fn chafariz(s: &mut Sink) {
    let mut st = s.pen("pedra");
    st.cylinder([0.0, 0.0, 0.0], 0.95, 0.92, 0.5, 10, false);
    st.cylinder([0.0, 0.5, 0.0], 0.92, 0.82, 0.06, 10, false);
    let mut inner = s.pen("pedra");
    (0..10).for_each(|i| {
        let p = |k: usize, y: f32| {
            let a = k as f32 / 10.0 * TAU;
            [a.sin() * 0.82, y, a.cos() * 0.82]
        };
        inner.quad([p(i + 1, 0.2), p(i, 0.2), p(i, 0.56), p(i + 1, 0.56)], [[0.5, 0.4], [0.0, 0.4], [0.0, 0.0], [0.5, 0.0]]);
    });
    s.pen("agua").cylinder([0.0, 0.0, 0.0], 0.82, 0.82, 0.38, 10, true);
    let mut c = s.pen("pedra");
    c.cylinder([0.0, 0.38, 0.0], 0.16, 0.12, 0.8, 6, false);
    c.cylinder([0.0, 1.18, 0.0], 0.12, 0.42, 0.14, 8, false);
    c.cylinder([0.0, 1.32, 0.0], 0.42, 0.06, 0.06, 8, true);
    s.pen("bronze").cylinder([0.0, 1.36, 0.0], 0.05, 0.02, 0.22, 4, true);
}

fn body(s: &mut Sink, tex: &str, w: f32, l: f32, low: f32, belt: f32) {
    s.pen(tex).cuboid([-w, low, -l], [w, belt, l], SIDES);
}

fn cabin(s: &mut Sink, paint: &str, glass: &str, w: f32, z0: f32, z1: f32, y0: f32, y1: f32, slope: f32) {
    let (t, f, b) = (w - 0.12, z1 - slope, z0 + slope * 0.7);
    let mut g = s.pen(glass);
    g.quad([[-w, y0, z1], [w, y0, z1], [t, y1, f], [-t, y1, f]], [[0.0, 1.0], [2.0, 1.0], [1.8, 0.0], [0.2, 0.0]]);
    g.quad([[w, y0, z0], [-w, y0, z0], [-t, y1, b], [t, y1, b]], [[0.0, 1.0], [2.0, 1.0], [1.8, 0.0], [0.2, 0.0]]);
    g.quad([[w, y0, z1], [w, y0, z0], [t, y1, b], [t, y1, f]], [[0.0, 1.0], [2.0, 1.0], [2.0, 0.0], [0.0, 0.0]]);
    g.quad([[-w, y0, z0], [-w, y0, z1], [-t, y1, f], [-t, y1, b]], [[0.0, 1.0], [2.0, 1.0], [2.0, 0.0], [0.0, 0.0]]);
    s.pen(paint).quad([[-t, y1, f], [t, y1, f], [t, y1, b], [-t, y1, b]], [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
}

fn wheels(s: &mut Sink, w: f32, zs: &[f32], r: f32) {
    zs.iter().for_each(|&z| [-1.0f32, 1.0].iter().for_each(|&side| wheel(s, "asfalto", [side * (w - 0.08), r, z], r, 0.18, side, 7)));
}

fn fusca(s: &mut Sink) {
    body(s, "fusca_lataria", 0.78, 1.75, 0.22, 0.85);
    let mut p = s.pen("fusca_lataria");
    p.quad([[-0.78, 0.85, 1.75], [0.78, 0.85, 1.75], [0.7, 0.85, 1.2], [-0.7, 0.85, 1.2]], [[0.0, 0.5], [1.0, 0.5], [1.0, 0.0], [0.0, 0.0]]);
    p.quad([[-0.7, 0.85, -1.35], [0.7, 0.85, -1.35], [0.78, 0.85, -1.75], [-0.78, 0.85, -1.75]], [[0.0, 0.5], [1.0, 0.5], [1.0, 0.0], [0.0, 0.0]]);
    cabin(s, "fusca_lataria", "vidro_noite", 0.74, -1.35, 1.2, 0.85, 1.42, 0.5);
    wheels(s, 0.86, &[-1.15, 1.15], 0.3);
    s.pen("metal").cuboid([-0.8, 0.3, 1.75], [0.8, 0.4, 1.82], Faces::NO_BACK);
    s.pen("metal").cuboid([-0.8, 0.3, -1.82], [0.8, 0.4, -1.75], Faces(0x2c));
    [-0.54f32, 0.42].iter().for_each(|&x| {
        s.pen("cal").cuboid([x, 0.52, 1.74], [x + 0.12, 0.64, 1.78], Faces::NO_BACK);
        s.pen("ceramica_barro").cuboid([x, 0.5, -1.78], [x + 0.1, 0.6, -1.74], Faces(0x2c));
    });
}

fn kombi(s: &mut Sink) {
    body(s, "kombi_lataria", 0.85, 2.05, 0.25, 1.95);
    s.pen("kombi_lataria").quad([[-0.85, 1.95, 2.05], [0.85, 1.95, 2.05], [0.85, 1.95, -2.05], [-0.85, 1.95, -2.05]], [[0.0, 2.0], [1.0, 2.0], [1.0, 0.0], [0.0, 0.0]]);
    let mut g = s.pen("onibus_janela");
    g.panel([-0.75, 1.15, 0.0], [0.75, 1.8, 2.06]);
    [-0.851f32, 0.851].iter().for_each(|&x| {
        let (z0, z1) = if x < 0.0 { (1.9, -1.9) } else { (-1.9, 1.9) };
        g.quad([[x, 1.2, z0], [x, 1.2, z1], [x, 1.75, z1], [x, 1.75, z0]], [[0.0, 1.0], [4.0, 1.0], [4.0, 0.0], [0.0, 0.0]]);
    });
    s.pen("metal").cuboid([-0.86, 0.3, 2.04], [0.86, 0.42, 2.12], Faces::NO_BACK);
    [-0.62f32, 0.48].iter().for_each(|&x| s.pen("cal").cuboid([x, 0.62, 2.04], [x + 0.14, 0.76, 2.08], Faces::NO_BACK));
    wheels(s, 0.93, &[-1.35, 1.4], 0.32);
}

fn onibus(s: &mut Sink) {
    body(s, "onibus_lataria", 1.0, 3.0, 0.35, 2.6);
    s.pen("ferrugem").quad([[-1.0, 2.6, 3.0], [1.0, 2.6, 3.0], [1.0, 2.6, -3.0], [-1.0, 2.6, -3.0]], [[0.0, 3.0], [1.0, 3.0], [1.0, 0.0], [0.0, 0.0]]);
    let mut g = s.pen("onibus_janela");
    g.panel([-0.9, 1.3, 0.0], [0.9, 2.35, 3.01]);
    [-1.001f32, 1.001].iter().for_each(|&x| {
        let (z0, z1) = if x < 0.0 { (2.6, -2.8) } else { (-2.8, 2.6) };
        g.quad([[x, 1.45, z0], [x, 1.45, z1], [x, 2.3, z1], [x, 2.3, z0]], [[0.0, 1.0], [5.0, 1.0], [5.0, 0.0], [0.0, 0.0]]);
    });
    s.pen("letreiro_rodoviaria").panel([-0.7, 2.38, 0.0], [0.7, 2.55, 3.012]);
    s.pen("metal").cuboid([-1.02, 0.35, 2.95], [1.02, 0.55, 3.08], Faces::NO_BOTTOM_BACK);
    s.pen("metal").cuboid([-1.02, 0.35, -3.08], [1.02, 0.55, -2.95], Faces(0x2c));
    [-0.78f32, 0.58].iter().for_each(|&x| s.pen("cal").cuboid([x, 0.68, 2.99], [x + 0.2, 0.84, 3.04], Faces::NO_BACK));
    wheels(s, 1.08, &[-2.0, 2.0], 0.45);
}

fn bicicleta(s: &mut Sink) {
    const SEG: usize = 12;
    let wheel = |s: &mut Sink, z: f32| {
        let at = |i: usize, r: f32, x: f32| {
            let a = i as f32 / SEG as f32 * TAU;
            [x, 0.34 + a.cos() * r, z + a.sin() * r]
        };
        let mut t = s.pen("asfalto");
        (0..SEG).for_each(|i| {
            let (j, uv) = (i + 1, [[0.0, 0.0], [1.0, 0.0], [1.0, 0.2], [0.0, 0.2]]);
            t.quad([at(i, 0.33, 0.025), at(j, 0.33, 0.025), at(j, 0.33, -0.025), at(i, 0.33, -0.025)], uv);
            t.quad([at(i, 0.29, 0.02), at(j, 0.29, 0.02), at(j, 0.33, 0.02), at(i, 0.33, 0.02)], uv);
            t.quad([at(j, 0.29, -0.02), at(i, 0.29, -0.02), at(i, 0.33, -0.02), at(j, 0.33, -0.02)], uv);
        });
        let mut m = s.pen("metal");
        (0..SEG / 2).for_each(|i| {
            let (a, b) = (at(i, 0.29, 0.0), at(i + SEG / 2, 0.29, 0.0));
            let a_ = i as f32 / SEG as f32 * TAU;
            let w = [0.0, -a_.sin() * 0.008, a_.cos() * 0.008];
            let q = [mat::sub(a, w), mat::sub(b, w), mat::add(b, w), mat::add(a, w)];
            m.quad(q, [[0.0, 0.0], [1.0, 0.0], [1.0, 0.05], [0.0, 0.05]]);
            m.quad([q[3], q[2], q[1], q[0]], [[0.0, 0.0], [1.0, 0.0], [1.0, 0.05], [0.0, 0.05]]);
        });
        m.cuboid([-0.05, 0.31, z - 0.03], [0.05, 0.37, z + 0.03], Faces::ALL);
    };
    wheel(s, -0.52);
    wheel(s, 0.52);
    let mut f = s.pen("ferrugem");
    f.cuboid([-0.02, 0.33, -0.52], [0.02, 0.37, -0.02], Faces(0x33));
    f.cuboid([-0.02, 0.34, -0.03], [0.02, 0.82, 0.01], Faces(0x33));
    f.cuboid([-0.02, 0.74, -0.08], [0.02, 0.78, 0.46], Faces(0x33));
    f.cuboid([-0.02, 0.36, -0.03], [0.02, 0.4, 0.44], Faces(0x33));
    f.cuboid([-0.02, 0.34, 0.44], [0.02, 0.98, 0.48], Faces(0x33));
    f.cuboid([-0.25, 0.96, 0.43], [0.25, 0.99, 0.47], Faces(0x3c));
    s.pen("madeira_escura").cuboid([-0.07, 0.82, -0.14], [0.07, 0.86, 0.06], Faces::NO_BOTTOM);
}

fn carroca(s: &mut Sink) {
    let mut w = s.pen("madeira_crua");
    w.cuboid([-0.6, 0.55, -1.0], [0.6, 0.6, 1.0], Faces::ALL);
    [-0.6f32, 0.56].iter().for_each(|&x| w.cuboid([x, 0.6, -1.0], [x + 0.04, 0.95, 1.0], Faces(0x37)));
    w.cuboid([-0.56, 0.6, -1.0], [0.56, 0.95, -0.96], Faces(0x34));
    [-0.35f32, 0.31].iter().for_each(|&x| w.cuboid([x, 0.5, 1.0], [x + 0.04, 0.55, 2.4], Faces(0x37)));
    [-1.0f32, 1.0].iter().for_each(|&side| wheel(s, "madeira_escura", [side * 0.66, 0.5, 0.0], 0.5, 0.08, side, 8));
    s.pen("sape").cuboid([-0.5, 0.6, -0.9], [0.4, 0.8, -0.2], Faces::NO_BOTTOM);
}

fn placa(s: &mut Sink) {
    s.pen("ferrugem").cylinder([0.0, 0.0, 0.0], 0.03, 0.03, 2.5, 4, true);
    let mut p = s.pen("placa_rua");
    p.quad([[-0.4, 2.1, 0.035], [0.4, 2.1, 0.035], [0.4, 2.5, 0.035], [-0.4, 2.5, 0.035]], [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
    p.quad([[0.4, 2.1, 0.03], [-0.4, 2.1, 0.03], [-0.4, 2.5, 0.03], [0.4, 2.5, 0.03]], [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
}

fn banca(s: &mut Sink) {
    let mut b = s.pen("fachada_azul");
    b.cuboid([-0.95, 0.0, -0.45], [0.95, 2.2, -0.35], Faces(0x37));
    [-0.95f32, 0.85].iter().for_each(|&x| b.cuboid([x, 0.0, -0.35], [x + 0.1, 2.2, 0.4], Faces(0x37)));
    b.cuboid([-0.85, 0.0, 0.2], [0.85, 0.95, 0.4], Faces(0x17));
    s.pen("ferrugem").cuboid([-1.05, 2.2, -0.5], [1.05, 2.28, 0.7], Faces::ALL);
    let mut j = s.pen("papel_jornal");
    j.quad([[-0.85, 0.95, 0.4], [0.85, 0.95, 0.4], [0.85, 0.98, 0.2], [-0.85, 0.98, 0.2]], [[0.0, 0.5], [2.0, 0.5], [2.0, 0.0], [0.0, 0.0]]);
    j.panel([-0.85, 1.1, 0.0], [0.85, 2.0, -0.34]);
}

fn balcao(s: &mut Sink) {
    let mut w = s.pen("madeira_banco");
    w.cuboid([-1.0, 0.0, -0.3], [1.0, 0.96, 0.24], SIDES);
    w.cuboid([-1.04, 0.96, -0.32], [1.04, 1.0, 0.3], Faces::ALL);
    s.pen("vitrine").panel([-0.9, 0.1, 0.0], [-0.45, 0.85, 0.245]);
    s.pen("vitrine").panel([0.45, 0.1, 0.0], [0.9, 0.85, 0.245]);
}

fn prateleira(s: &mut Sink) {
    let mut w = s.pen("madeira_crua");
    w.cuboid([-0.9, 0.0, -0.22], [-0.86, 2.0, 0.22], Faces(0x33));
    w.cuboid([0.86, 0.0, -0.22], [0.9, 2.0, 0.22], Faces(0x33));
    w.panel([-0.86, 0.0, 0.0], [0.86, 2.0, -0.21]);
    let shelves = [0.05f32, 0.55, 1.05, 1.55, 1.95];
    shelves.iter().for_each(|&y| w.cuboid([-0.86, y, -0.21], [0.86, y + 0.04, 0.22], Faces(0x14)));
    let mut b = s.pen("ceramica_barro");
    shelves[..4].iter().enumerate().for_each(|(i, &y)| {
        (0..4).filter(|k| (k + i) % 3 != 0).for_each(|k| {
            let x = -0.78 + k as f32 * 0.4;
            b.cuboid([x, y + 0.04, -0.12], [x + 0.3, y + 0.16, 0.1], Faces(0x17));
        });
    });
}

fn mesa_bar(s: &mut Sink) {
    let mut m = s.pen("ferrugem");
    m.cylinder([0.0, 0.0, 0.0], 0.25, 0.2, 0.03, 6, true);
    m.cylinder([0.0, 0.03, 0.0], 0.03, 0.03, 0.68, 4, false);
    s.pen("metal").cylinder([0.0, 0.71, 0.0], 0.4, 0.4, 0.03, 8, true);
}

fn engradado(s: &mut Sink) {
    let mut w = s.pen("madeira_crua");
    w.cuboid([-0.25, 0.0, -0.2], [0.25, 0.3, 0.2], SIDES);
    w.cuboid([-0.23, 0.3, -0.18], [0.23, 0.58, 0.18], SIDES);
}

fn geladeira_bar(s: &mut Sink) {
    let mut m = s.pen("metal");
    m.cuboid([-0.35, 0.1, -0.3], [0.35, 1.85, 0.3], SIDES);
    legs(&mut m, 0.33, -0.28, 0.28, 0.1, 0.04);
    s.pen("letreiro_bar").panel([-0.32, 1.86, -0.25], [0.32, 1.96, 0.31]);
    s.pen("vitrine").panel([-0.3, 0.2, 0.0], [0.3, 1.65, 0.298]);
}

fn caixa_dagua(s: &mut Sink) {
    let mut c = s.pen("concreto");
    c.cylinder([0.0, 0.0, 0.0], 0.45, 0.45, 0.4, 6, false);
    c.cylinder([0.0, 0.4, 0.0], 0.55, 0.62, 1.0, 8, false);
}

fn muro(s: &mut Sink) {
    s.pen("muro_chapiscado").cuboid([-0.5, 0.0, -0.1], [0.5, 0.85, 0.1], SIDES);
    s.pen("concreto").cuboid([-0.52, 0.85, -0.13], [0.52, 0.92, 0.13], Faces::ALL);
}

fn portao(s: &mut Sink) {
    let mut f = s.pen("ferrugem");
    [-0.56f32, 0.48].iter().for_each(|&x| f.cuboid([x, 0.0, -0.04], [x + 0.08, 2.1, 0.04], Faces(0x37)));
}

fn lixeira(s: &mut Sink) {
    s.pen("ferrugem").cylinder([0.0, 0.0, -0.18], 0.03, 0.03, 1.0, 4, true);
    s.pen("ferrugem").cylinder([0.0, 0.55, 0.0], 0.18, 0.2, 0.45, 7, false);
}

fn letreiro(s: &mut Sink, kind: &str) {
    s.pen("ferrugem").cuboid([-1.05, 2.55, -0.1], [1.05, 3.15, -0.05], Faces(0x17));
    s.pen(kind).panel([-1.0, 2.6, -0.1], [1.0, 3.1, -0.045]);
}

fn caixa(s: &mut Sink) {
    let mut m = s.pen("metal");
    m.cuboid([-0.2, 0.0, -0.18], [0.2, 0.18, 0.18], Faces::NO_BOTTOM);
    let local = mat::mul(&mat::translate([0.0, 0.18, -0.05]), &mat::rot_x(-0.5));
    s.pen_at("metal", &local).cuboid([-0.18, 0.0, -0.1], [0.18, 0.14, 0.08], Faces::NO_BOTTOM);
}
