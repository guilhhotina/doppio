use super::cloth;
use super::cues::Tell;
use super::lamps;
use super::level::Level;
use super::mat::{self, M4, V3};
use super::mesh::{UV_ONE, Vert};
use crate::map::{Map, Prop, TaskKind};
use std::f32::consts::{PI, TAU};

pub const TV_REACH: f32 = 2.0;
const PROP_REACH: f32 = 2.0;
const MOON: [u8; 3] = [92, 122, 196];
const FLAME: [u8; 3] = [60, 110, 255];
const RED: [u8; 3] = [255, 36, 18];
const GREEN: [u8; 3] = [40, 255, 70];
const WATER: [u8; 3] = [150, 190, 255];
const GOLD: [u8; 3] = [255, 236, 190];
const WARM: [u8; 3] = [255, 178, 96];
const KEYS: [(&str, V3); 6] = [
    ("criado_mudo", [0.13, 0.6, 0.12]),
    ("rack", [0.3, 0.52, 0.05]),
    ("cristaleira", [0.1, 1.0, 0.25]),
    ("estante", [0.1, 0.95, 0.19]),
    ("geladeira", [0.1, 1.0, 0.36]),
    ("guarda_roupa", [0.1, 1.0, 0.33]),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fixture {
    Window { drape: Option<u8>, run: Option<u8>, spill: [V3; 4] },
    Tv(Option<u8>),
    Flame(V3),
    Lamp(V3),
    Drip(V3, f32),
    Glint(V3),
}

#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub fixture: Fixture,
    pub rooms: u32,
    pub lo: V3,
    pub hi: V3,
    pub seed: f32,
}

fn prop_xf(p: &Prop) -> M4 {
    mat::mul(&mat::translate([p.pos.x.to_f32(), 0.0, p.pos.y.to_f32()]), &mat::rot_y(p.rot.to_rad()))
}

fn nearest<'a>(map: &'a Map, kinds: &[&str], c: V3) -> Option<&'a Prop<'a>> {
    let d2 = |p: &Prop| {
        let (dx, dz) = (p.pos.x.to_f32() - c[0], p.pos.y.to_f32() - c[2]);
        dx * dx + dz * dz
    };
    map.props.iter().filter(|p| kinds.contains(&p.kind) && d2(p) < PROP_REACH * PROP_REACH).min_by(|a, b| d2(a).total_cmp(&d2(b)))
}

fn horizontal(a: V3, b: V3) -> f32 {
    let (dx, dz) = (a[0] - b[0], a[2] - b[2]);
    (dx * dx + dz * dz).sqrt()
}

fn spill(xf: &M4) -> [V3; 4] {
    [[-0.55, 0.01, 0.12], [0.55, 0.01, 0.12], [0.8, 0.01, 1.9], [-0.8, 0.01, 1.9]].map(|p| mat::point(xf, p))
}

fn fixture(map: &Map, level: &Level, tvs: &[V3], kind: TaskKind, c: V3) -> Fixture {
    let at = |kinds: &[&str], local: V3, fallback: V3| nearest(map, kinds, c).map_or(fallback, |p| mat::point(&prop_xf(p), local));
    match kind {
        TaskKind::Window => {
            let drape = (0..level.drapes.len())
                .map(|d| (d, horizontal(mat::point(&level.drapes[d].xf, [0.0; 3]), c)))
                .filter(|&(_, d)| d < PROP_REACH)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(d, _)| d as u8);
            let spill = drape.map_or([c; 4], |d| spill(&level.drapes[d as usize].xf));
            Fixture::Window { drape, run: level.window_run(c[0].floor() as i32, c[2].floor() as i32), spill }
        }
        TaskKind::Tv => Fixture::Tv(
            (0..tvs.len())
                .map(|k| (k, horizontal(tvs[k], c)))
                .filter(|&(_, d)| d < TV_REACH)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(k, _)| k as u8),
        ),
        TaskKind::Gas => Fixture::Flame(at(&["fogao"], [0.17, 0.885, 0.14], [c[0], 0.9, c[2]])),
        TaskKind::Breaker => Fixture::Lamp(at(&["disjuntor"], [0.12, 1.66, 0.11], [c[0], 1.5, c[2]])),
        TaskKind::Faucet => nearest(map, &["pia"], c).map_or(Fixture::Drip([c[0], 1.0, c[2]], 0.85), |p| {
            Fixture::Drip(mat::point(&prop_xf(p), [0.0, 1.065, -0.07]), 0.89)
        }),
        TaskKind::Key => {
            let kinds = KEYS.map(|k| k.0);
            Fixture::Glint(nearest(map, &kinds, c).map_or([c[0], 0.04, c[2]], |p| {
                let local = KEYS.iter().find(|k| k.0 == p.kind).map_or([0.0; 3], |k| k.1);
                mat::point(&prop_xf(p), local)
            }))
        }
    }
}

fn bounds(f: &Fixture, c: V3) -> (V3, V3) {
    let points: &[V3] = match f {
        Fixture::Window { spill, .. } => spill,
        Fixture::Flame(p) | Fixture::Lamp(p) | Fixture::Glint(p) | Fixture::Drip(p, _) => std::slice::from_ref(p),
        Fixture::Tv(_) => std::slice::from_ref(&c),
    };
    points.iter().fold((mat::sub(points[0], [0.3; 3]), mat::add(points[0], [0.3; 3])), |(lo, hi), p| {
        (std::array::from_fn(|k| lo[k].min(p[k] - 0.3)), std::array::from_fn(|k| hi[k].max(p[k] + 0.3)))
    })
}

pub fn sites(map: &Map, level: &Level, tvs: &[V3]) -> Vec<Site> {
    map.tasks
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let (x, y) = (t.pos.x.floor(), t.pos.y.floor());
            let c = [x as f32 + 0.5, 1.0, y as f32 + 0.5];
            let fixture = fixture(map, level, tvs, t.kind, c);
            let rooms = lamps::room_near(map, x, y).map_or(0, |r| 1 << r);
            let (lo, hi) = bounds(&fixture, c);
            Site { fixture, rooms, lo, hi, seed: i as f32 * 2.399 }
        })
        .collect()
}

fn vert(p: V3, a: f32, c: [u8; 3]) -> Vert {
    Vert { pos: p, uv: [(a * UV_ONE).clamp(0.0, 32767.0) as i16, UV_ONE as i16], nrm: [0; 4], tile: [c[0], c[1], c[2], 255] }
}

fn quad(out: &mut Vec<Vert>, p: [V3; 4], a: [f32; 4], c: [u8; 3]) {
    out.extend((0..4).map(|i| vert(p[i], a[i], c)));
}

fn tri(out: &mut Vec<Vert>, p: [V3; 3], a: [f32; 3], c: [u8; 3]) {
    quad(out, [p[0], p[1], p[2], p[2]], [a[0], a[1], a[2], a[2]], c);
}

fn blob(out: &mut Vec<Vert>, c: V3, r: V3, u: V3, a: f32, col: [u8; 3], sides: usize) {
    let rim = |i: usize| {
        let (s, k) = (i as f32 / sides as f32 * TAU).sin_cos();
        mat::add(c, mat::add(mat::scale(r, k), mat::scale(u, s)))
    };
    (0..sides).for_each(|i| tri(out, [c, rim(i), rim(i + 1)], [a, 0.0, 0.0], col));
}

pub fn facing(p: V3, eye: V3) -> (V3, V3) {
    let f = mat::norm(mat::sub(p, eye));
    let r = mat::norm(mat::cross(f, [0.0, 1.0, 0.0]));
    (r, mat::cross(r, f))
}

fn spark(out: &mut Vec<Vert>, p: V3, eye: V3, size: f32, a: f32, c: [u8; 3]) {
    let to = mat::sub(eye, p);
    let k = (size / mat::len(to).max(size * 2.0)).min(0.5);
    let (q, s) = (mat::add(p, mat::scale(to, k)), size * (1.0 - k));
    let (r, u) = facing(q, eye);
    blob(out, q, mat::scale(r, s), mat::scale(u, s), a, c, 6);
}

fn flame(out: &mut Vec<Vert>, p: V3, eye: V3, t: f32, seed: f32) {
    let (r, _) = facing(p, eye);
    (0..6).for_each(|i| {
        let a = i as f32 / 6.0 * TAU;
        let base = mat::add(p, [a.sin() * 0.055, 0.0, a.cos() * 0.055]);
        let h = 0.05 + 0.04 * ((t * 23.0 + i as f32 * 1.7 + seed).sin() * 0.5 + 0.5);
        let w = mat::scale(r, 0.024);
        tri(out, [mat::sub(base, w), mat::add(base, w), mat::add(base, [0.0, h, 0.0])], [1.5, 1.5, 0.0], FLAME);
    });
    spark(out, mat::add(p, [0.0, 0.03, 0.0]), eye, 0.2, 0.3, FLAME);
}

fn lamp(out: &mut Vec<Vert>, p: V3, eye: V3, tell: Tell, power: bool, t: f32) {
    let (on, c) = match tell {
        Tell::Done => (true, GREEN),
        _ => ((t * if power { 1.0 } else { 4.0 }).fract() < 0.5, RED),
    };
    let k = if on { 1.0 } else { 0.12 };
    spark(out, p, eye, 0.035, 1.8 * k, c);
    spark(out, p, eye, 0.16, 0.35 * k, c);
}

fn drip(out: &mut Vec<Vert>, top: V3, bottom: f32, eye: V3, t: f32, seed: f32) {
    const PERIOD: f32 = 1.3;
    const HANG: f32 = 0.7;
    let fall = (2.0 * (top[1] - bottom) / 9.81).sqrt() / PERIOD;
    let ph = (t / PERIOD + seed).fract();
    match ph {
        ph if ph < HANG => spark(out, mat::sub(top, [0.0, 0.008 * ph / HANG, 0.0]), eye, 0.014 + 0.008 * ph / HANG, 1.0, WATER),
        ph if ph < HANG + fall => {
            let s = (ph - HANG) * PERIOD;
            spark(out, [top[0], top[1] - 0.5 * 9.81 * s * s, top[2]], eye, 0.022, 1.2, WATER);
        }
        ph => {
            let k = 1.0 - ((ph - HANG - fall) / 0.12).min(1.0);
            if k > 0.0 {
                let r = 0.03 + 0.06 * (1.0 - k);
                blob(out, [top[0], bottom + 0.004, top[2]], [r, 0.0, 0.0], [0.0, 0.0, r], 0.7 * k, WATER, 6);
            }
        }
    }
}

fn glint(out: &mut Vec<Vert>, p: V3, eye: V3, t: f32, seed: f32) {
    const PERIOD: f32 = 2.1;
    const FLASH: f32 = 0.25;
    let ph = (t / PERIOD + seed).fract();
    let k = if ph < FLASH { (PI * ph / FLASH).sin() } else { 0.0 };
    if k <= 0.0 {
        return;
    }
    let q = mat::add(p, mat::scale(mat::norm(mat::sub(eye, p)), 0.06));
    let (r, u) = facing(q, eye);
    blob(out, q, mat::scale(r, 0.22 * k), mat::scale(u, 0.016), 1.8 * k, GOLD, 4);
    blob(out, q, mat::scale(r, 0.016), mat::scale(u, 0.22 * k), 1.8 * k, GOLD, 4);
    spark(out, p, eye, 0.06, 1.4 * k, GOLD);
}

pub fn emit(site: &Site, tell: Tell, power: bool, t: f32, eye: V3, out: &mut Vec<Vert>) {
    let live = tell == Tell::Pending;
    match site.fixture {
        Fixture::Window { drape: Some(_), spill, .. } if live => quad(out, spill, [0.34, 0.34, 0.0, 0.0], MOON),
        Fixture::Flame(p) if live => flame(out, p, eye, t, site.seed),
        Fixture::Lamp(p) if tell != Tell::Idle => lamp(out, p, eye, tell, power, t + site.seed),
        Fixture::Drip(p, bottom) if live => drip(out, p, bottom, eye, t, site.seed),
        Fixture::Glint(p) if live => glint(out, p, eye, t, site.seed),
        _ => {}
    }
}

pub fn abajur(out: &mut Vec<Vert>, p: V3, eye: V3, lamp: f32) {
    spark(out, p, eye, 0.34, 0.32 * lamp, WARM);
    spark(out, p, eye, 0.15, 0.4 * lamp, WARM);
}

pub fn drape(xf: &M4, t: f32, seed: f32, tile: [u8; 4], out: &mut Vec<Vert>) {
    cloth::quads(cloth::ROWS, |p, e| cloth::sway(t, seed, p, e), |q, uv| {
        let p = q.map(|v| mat::point(xf, v));
        let n = mat::norm(mat::cross(mat::sub(p[1], p[0]), mat::sub(p[2], p[0])));
        let light = (0.32 + 2.2 * n[1].max(0.0)).min(1.0);
        out.extend((0..4).map(|i| Vert::new(p[i], uv[i], n).with_light(light).with_tile(tile)));
    });
}

#[cfg(test)]
mod tests {
    use super::super::lamps::Lamps;
    use super::super::texture;
    use super::*;
    use crate::map;

    #[test]
    fn every_task_spot_has_its_fixture() {
        let m = map::sobrado();
        let l = Level::new(m, &Lamps::new(m), &texture::atlas().tiles);
        let tvs: Vec<V3> = l.screens.verts.chunks(4).map(|q| q[0].pos).collect();
        let s = sites(m, &l, &tvs);
        assert_eq!(s.len(), m.tasks.len());
        s.iter().zip(&m.tasks).for_each(|(s, t)| {
            assert!(s.rooms != 0);
            let c = [t.pos.x.to_f32(), 1.0, t.pos.y.to_f32()];
            match s.fixture {
                Fixture::Window { drape, run, .. } => assert!(drape.is_some() && run.is_some(), "window {c:?}"),
                Fixture::Tv(k) => assert!(k.is_some(), "tv {c:?}"),
                Fixture::Flame(p) | Fixture::Lamp(p) | Fixture::Drip(p, _) | Fixture::Glint(p) => {
                    assert!(horizontal(p, c) < 2.5 && p[1] > 0.0 && p[1] < 2.0, "{:?} at {p:?} for {c:?}", t.kind)
                }
            }
        });
    }

    #[test]
    fn tells_only_light_pending_tasks_and_breaker_turns_green() {
        let site = |f| Site { fixture: f, rooms: 1, lo: [0.0; 3], hi: [1.0; 3], seed: 0.0 };
        let eye = [0.0, 1.6, -3.0];
        let count = |s: &Site, tell, t| {
            let mut out = Vec::new();
            emit(s, tell, true, t, eye, &mut out);
            out
        };
        let flame = site(Fixture::Flame([0.0, 0.9, 0.0]));
        assert!(count(&flame, Tell::Idle, 0.0).is_empty() && count(&flame, Tell::Done, 0.0).is_empty());
        assert!(count(&flame, Tell::Pending, 0.0).len() % 4 == 0 && !count(&flame, Tell::Pending, 0.0).is_empty());
        let lamp = site(Fixture::Lamp([0.0, 1.6, 0.0]));
        assert!(count(&lamp, Tell::Pending, 0.1).iter().all(|v| v.tile[..3] == RED));
        assert!(count(&lamp, Tell::Done, 0.1).iter().all(|v| v.tile[..3] == GREEN));
        let key = site(Fixture::Glint([0.0, 0.5, 0.0]));
        assert!(!count(&key, Tell::Pending, 0.1).is_empty() && count(&key, Tell::Pending, 1.0).is_empty());
        assert!(count(&key, Tell::Done, 0.1).is_empty());
    }
}
