use super::mat::V3;
use super::ui2d::{Ui2d, rgb565};
use crate::map::{Hide, HideKind};
use crate::sim::{Anim, HIDE_TICS, PlayerState};
use crate::ui::art;

pub const EMERGE: u32 = 12;
pub const DIVE: u32 = 6;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    Out,
    Entering { spot: u8, k: f32 },
    Hidden { spot: u8, age: u32, dove: bool },
    Leaving { spot: u8, k: f32 },
    Emerging { spot: u8, k: f32 },
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Track {
    spot: u8,
    hidden: bool,
    entering: bool,
    dove: bool,
    since: u32,
    left: Option<u32>,
}

pub fn step(t: Track, state: PlayerState, hiding: Option<(u8, u8)>, tic: u32) -> (Track, Phase) {
    let k = |left: u8| (HIDE_TICS - left.min(HIDE_TICS)) as f32 / HIDE_TICS as f32;
    match (state, hiding) {
        (PlayerState::Hidden { spot }, h) => {
            let t = match t.hidden {
                true => t,
                false => Track { spot, hidden: true, entering: false, dove: !t.entering, since: tic, left: None },
            };
            let phase = match h {
                Some((_, left)) => Phase::Leaving { spot, k: k(left) },
                None => Phase::Hidden { spot, age: tic.wrapping_sub(t.since), dove: t.dove },
            };
            (t, phase)
        }
        (PlayerState::Free, Some((spot, left))) => (Track { spot, entering: true, hidden: false, left: None, ..t }, Phase::Entering { spot, k: k(left) }),
        (PlayerState::Free, None) => {
            let left = match t.hidden {
                true => Some(tic),
                false => t.left,
            };
            let t = Track { hidden: false, entering: false, left, ..t };
            let phase = match left.map(|l| tic.wrapping_sub(l)) {
                Some(age) if age < EMERGE => Phase::Emerging { spot: t.spot, k: age as f32 / EMERGE as f32 },
                _ => Phase::Out,
            };
            (t, phase)
        }
        _ => (Track::default(), Phase::Out),
    }
}

fn ramp(v: f32, a: f32, b: f32) -> f32 {
    let t = ((v - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn door(phase: Phase) -> Option<(u8, f32)> {
    match phase {
        Phase::Entering { spot, k } => Some((spot, ramp(k, 0.0, 0.25) * (1.0 - ramp(k, 0.62, 0.93)))),
        Phase::Leaving { spot, k } => Some((spot, ramp(k, 0.45, 1.0))),
        Phase::Emerging { spot, k } => Some((spot, 1.0 - ramp(k, 0.55, 1.0))),
        Phase::Hidden { spot, age, dove: true } if age < DIVE => Some((spot, 1.0 - age as f32 / DIVE as f32)),
        _ => None,
    }
}

pub fn stands(kind: HideKind) -> bool {
    matches!(kind, HideKind::GuardaRoupa | HideKind::Armario | HideKind::Confessionario)
}

pub fn centre(h: &Hide) -> [f32; 2] {
    [h.x as f32 + 0.5, h.y as f32 + 0.5]
}

pub fn outward(h: &Hide, from: [f32; 2]) -> [f32; 2] {
    let c = centre(h);
    let d = [from[0] - c[0], from[1] - c[1]];
    match d[0].abs() > d[1].abs() {
        true => [d[0].signum(), 0.0],
        false => [0.0, if d[1] == 0.0 { 1.0 } else { d[1].signum() }],
    }
}

pub struct Pose {
    pub at: [f32; 2],
    pub heading: f32,
    pub anim: Anim,
    pub tic: u16,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn turn(a: f32, b: f32, t: f32) -> f32 {
    let d = (b - a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
    a + d * t
}

pub fn pose(h: &Hide, phase: Phase, pos: [f32; 2], heading: f32) -> Option<Pose> {
    let c = centre(h);
    let out = outward(h, pos);
    let inward = (-out[0]).atan2(-out[1]);
    let facing_out = out[0].atan2(out[1]);
    let inside = [c[0] + out[0] * 0.05, c[1] + out[1] * 0.05];
    let (walk, duck) = match stands(h.kind) {
        true => (Anim::Walk, Anim::Walk),
        false => (Anim::Walk, Anim::Crouch),
    };
    match phase {
        Phase::Entering { k, .. } => {
            let go = ramp(k, 0.35, 0.96);
            let spin = ramp(k, 0.28, 0.72);
            let at = [lerp(pos[0], inside[0], go), lerp(pos[1], inside[1], go)];
            let heading = turn(turn(heading, inward, ramp(k, 0.0, 0.28)), facing_out, spin);
            let anim = if k < 0.6 { walk } else { duck };
            let tic = if anim == Anim::Crouch { ((k - 0.6) / 0.4 * 10.0) as u16 } else { (k * 36.0) as u16 };
            Some(Pose { at, heading, anim, tic })
        }
        Phase::Emerging { k, .. } => {
            let go = ramp(k, 0.0, 0.52);
            let at = [lerp(inside[0], pos[0], go), lerp(inside[1], pos[1], go)];
            let anim = if stands(h.kind) || k > 0.35 { Anim::Walk } else { Anim::Crouch };
            let tic = if anim == Anim::Crouch { ((1.0 - k / 0.35) * 10.0) as u16 } else { (k * 28.0) as u16 };
            Some(Pose { at, heading: facing_out, anim, tic })
        }
        _ => None,
    }
}

pub fn eye_height(kind: HideKind) -> f32 {
    match kind {
        HideKind::GuardaRoupa | HideKind::Armario => 1.45,
        HideKind::Confessionario => 1.25,
        HideKind::Cama => 0.24,
        HideKind::Mesa => 0.4,
        HideKind::Tumulo => 0.45,
        HideKind::CaixaDagua => 1.5,
        HideKind::Carro => 1.0,
        HideKind::Cesto => 0.55,
    }
}

pub fn peek(h: &Hide, pos: [f32; 2], breath: f32) -> (V3, V3) {
    let c = centre(h);
    let out = outward(h, pos);
    let y = eye_height(h.kind) + breath * 0.012;
    let eye = [c[0] + out[0] * 0.42, y, c[1] + out[1] * 0.42];
    let drop = if y > 1.0 { 0.35 } else { -0.05 };
    (eye, [eye[0] + out[0] * 3.0, y - drop, eye[2] + out[1] * 3.0])
}

const SHADE: [u8; 4] = [86, 70, 58, 255];
const DARK: [u8; 4] = [46, 38, 34, 255];
const LEAK: u16 = rgb565(236, 184, 110);
const RIM: u16 = rgb565(14, 10, 8);
const MOTE: u16 = rgb565(255, 236, 200);

struct View<'a> {
    ui: &'a mut Ui2d,
    rect: [i32; 4],
    dx: i32,
    dy: i32,
}

impl View<'_> {
    fn clip(&self, x: i32, y: i32, w: i32, h: i32) -> Option<[i16; 4]> {
        let [rx, ry, rw, rh] = self.rect;
        let (x0, y0) = ((rx + x + self.dx).max(rx), (ry + y + self.dy).max(ry));
        let (x1, y1) = ((rx + x + self.dx + w).min(rx + rw), (ry + y + self.dy + h).min(ry + rh));
        (x1 > x0 && y1 > y0).then(|| [x0 as i16, y0 as i16, (x1 - x0) as i16, (y1 - y0) as i16])
    }

    fn board(&mut self, tex: &str, x: i32, y: i32, w: i32, h: i32, col: [u8; 4]) {
        if let Some(r) = self.clip(x, y, w, h) {
            self.ui.fill(tex, r, col);
        }
    }

    fn line(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16, alpha: u8) {
        if let Some(r) = self.clip(x, y, w, h) {
            self.ui.shade(r[0], r[1], r[2], r[3], color, alpha);
        }
    }
}

fn louvres(v: &mut View, w: i32, h: i32, t: f32, kind: HideKind) {
    let pitch = (h / 15).max(7);
    let slat = pitch * 3 / 5;
    let tex = if kind == HideKind::Armario { "madeira_porta" } else { "madeira_escura" };
    (-1..h / pitch + 2).for_each(|i| {
        let y = i * pitch;
        v.board(tex, 0, y, w, slat, SHADE);
        v.line(0, y + slat, w, 1, RIM, 255);
        v.line(0, y - 1, w, 1, LEAK, 150);
    });
    let stile = w / 8;
    v.board(tex, 0, -pitch, stile, h + 2 * pitch, DARK);
    v.board(tex, w - stile, -pitch, stile, h + 2 * pitch, DARK);
    v.board(tex, w / 2 - stile / 4, -pitch, stile / 2, h + 2 * pitch, DARK);
    v.line(stile, 0, 1, h, LEAK, 90);
    v.line(w - stile - 1, 0, 1, h, RIM, 255);
    motes(v, w, h, pitch, slat, t);
}

fn motes(v: &mut View, w: i32, h: i32, pitch: i32, slat: i32, t: f32) {
    (0..9).for_each(|i| {
        let f = i as f32 * 1.618;
        let x = ((f * 97.0 + t * (4.0 + i as f32)) % w as f32) as i32;
        let row = ((f * 13.0) as i32) % (h / pitch).max(1);
        let y = row * pitch + slat + 1 + ((t * 0.7 + f).sin() * 1.2 + 1.0) as i32;
        v.line(x, y, 1, 1, MOTE, 120 + (i * 13 % 90) as u8);
    });
}

fn lattice(v: &mut View, w: i32, h: i32) {
    let pitch = (w / 22).max(8);
    (0..w / pitch + 1).for_each(|i| v.board("madeira_escura", i * pitch, 0, 3, h, SHADE));
    (0..h / pitch + 1).for_each(|i| {
        v.board("madeira_escura", 0, i * pitch, w, 3, SHADE);
        v.line(0, i * pitch + 3, w, 1, RIM, 255);
    });
    v.board("colcha", 0, 0, w, h / 5, DARK);
    v.board("madeira_escura", 0, h - h / 6, w, h / 6, DARK);
}

fn skirt(v: &mut View, w: i32, h: i32, tex: &str, fringe: bool, t: f32) {
    let hem = h * 46 / 100;
    let rail = h / 9;
    v.board("madeira_escura", 0, 0, w, rail, DARK);
    v.line(0, rail, w, 1, RIM, 255);
    v.board(tex, 0, rail, w, hem - rail, DARK);
    let fold = (w / 18).max(5);
    (0..w / fold + 1).for_each(|i| {
        let drop = h / 36 + (i * 7 % 5) * h / 95;
        v.board(tex, i * fold, hem, fold, drop, SHADE);
        v.line(i * fold, hem + drop, fold, 1, RIM, 255);
        if fringe && i % 2 == 0 {
            v.line(i * fold + fold / 2, hem + drop, 1, h / 26, LEAK, 130);
        }
    });
    v.line(0, hem - 1, w, 1, LEAK, 85);
    v.board("madeira_escura", 0, 0, w / 14, h, DARK);
    v.board("madeira_escura", w - w / 14, 0, w / 14, h, DARK);
    motes(v, w, h, (h / 12).max(6), hem, t);
}

fn slit(v: &mut View, w: i32, h: i32) {
    let gap = h * 18 / 100;
    let mid = h / 2;
    v.board("lapide_granito", 0, 0, w, mid - gap / 2, SHADE);
    v.board("lapide_granito", 0, mid + gap / 2, w, h, DARK);
    v.line(0, mid - gap / 2, w, 2, RIM, 255);
    v.line(0, mid + gap / 2 - 1, w, 1, LEAK, 120);
    v.board("lapide_granito", 0, 0, w / 6, h, DARK);
    v.board("lapide_granito", w - w / 6, 0, w / 6, h, DARK);
}

fn porthole(v: &mut View, w: i32, h: i32) {
    let (cx, cy, rad) = (w / 2, h / 2, h * 30 / 100);
    let step = 3;
    (0..h / step + 1).for_each(|i| {
        let y = i * step;
        let dy = (y - cy).abs();
        let half = if dy < rad { ((rad * rad - dy * dy) as f32).sqrt() as i32 } else { 0 };
        match half {
            0 => v.board("ferrugem", 0, y, w, step, SHADE),
            _ => {
                v.board("ferrugem", 0, y, cx - half, step, SHADE);
                v.board("ferrugem", cx + half, y, w, step, SHADE);
                v.line(cx - half, y, 1, step, LEAK, 100);
            }
        }
    });
}

fn cabin(v: &mut View, w: i32, h: i32) {
    v.board("fusca_lataria", 0, 0, w, h / 7, DARK);
    v.board("fusca_lataria", 0, h - h * 3 / 10, w, h, SHADE);
    v.line(0, h - h * 3 / 10, w, 1, LEAK, 90);
    (0..6).for_each(|i| {
        let k = i * h / 10;
        v.board("fusca_lataria", 0, h / 7 + k, w / 7 - i * w / 60, h / 10, DARK);
        v.board("fusca_lataria", w - w / 7 + i * w / 60, h / 7 + k, w, h / 10, DARK);
    });
    v.board("metal", w / 2 - w / 12, h - h * 3 / 10 - h / 14, w / 6, h / 14, DARK);
}

fn weave(v: &mut View, w: i32, h: i32) {
    let pitch = (w / 20).max(6);
    (0..w / pitch + 1).for_each(|i| v.board("sape", i * pitch, 0, pitch * 2 / 3, h, SHADE));
    (0..h / pitch + 1).filter(|i| i % 2 == 0).for_each(|i| {
        v.board("sape", 0, i * pitch, w, pitch * 2 / 3, DARK);
        v.line(0, i * pitch + pitch * 2 / 3, w, 1, RIM, 255);
    });
}

pub fn overlay(kind: HideKind, phase: Phase, rect: [i32; 4], t: f32, ui: &mut Ui2d) {
    let [_, _, w, h] = rect;
    let breathe = ((t * 1.3).sin() * 1.5).round() as i32;
    let slide = match phase {
        Phase::Leaving { k, .. } => ramp(k, 0.45, 1.0),
        Phase::Hidden { age, dove: true, .. } if age < DIVE => 1.0 - age as f32 / DIVE as f32,
        _ => 0.0,
    };
    let dx = match stands(kind) {
        true => (slide * w as f32 * 1.05) as i32,
        false => 0,
    };
    let dy = breathe + if stands(kind) { 0 } else { (-slide * h as f32 * 0.6) as i32 };
    let mut v = View { ui, rect, dx, dy };
    match kind {
        HideKind::GuardaRoupa | HideKind::Armario => louvres(&mut v, w, h, t, kind),
        HideKind::Confessionario => lattice(&mut v, w, h),
        HideKind::Cama => skirt(&mut v, w, h, "colcha", false, t),
        HideKind::Mesa => skirt(&mut v, w, h, "croche", true, t),
        HideKind::Tumulo => slit(&mut v, w, h),
        HideKind::CaixaDagua => porthole(&mut v, w, h),
        HideKind::Carro => cabin(&mut v, w, h),
        HideKind::Cesto => weave(&mut v, w, h),
    }
    let r = rect.map(|c| c as i16);
    art::vignette(ui, [r[0], r[1] + breathe as i16, r[2], r[3]], 4, [0, 0, 0, 255]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::Ang;

    const KINDS: [HideKind; 9] = [
        HideKind::Cama,
        HideKind::GuardaRoupa,
        HideKind::Armario,
        HideKind::Tumulo,
        HideKind::CaixaDagua,
        HideKind::Confessionario,
        HideKind::Carro,
        HideKind::Cesto,
        HideKind::Mesa,
    ];

    #[test]
    fn overlays_stay_inside_their_view_and_fit_the_quad_budget() {
        KINDS.iter().for_each(|&k| {
            [Phase::Hidden { spot: 0, age: 40, dove: false }, Phase::Leaving { spot: 0, k: 0.8 }].iter().for_each(|&p| {
                let mut ui = Ui2d::new();
                overlay(k, p, [192, 108, 192, 108], 0.7, &mut ui);
                assert!(ui.quads() > 3 && ui.quads() < 400, "{k:?} {}", ui.quads());
                assert!(ui.verts().iter().all(|v| (186..=390).contains(&v.pos[0]) && (100..=222).contains(&v.pos[1])), "{k:?}");
            });
        });
    }

    #[test]
    fn hiding_walks_through_enter_hide_leave_and_emerge() {
        let free = PlayerState::Free;
        let hid = PlayerState::Hidden { spot: 2 };
        let (t, p) = step(Track::default(), free, Some((2, HIDE_TICS)), 10);
        assert_eq!(p, Phase::Entering { spot: 2, k: 0.0 });
        let (t, p) = step(t, hid, None, 25);
        assert_eq!(p, Phase::Hidden { spot: 2, age: 0, dove: false });
        assert_eq!(door(p), None);
        let (t, p) = step(t, hid, Some((2, 5)), 90);
        assert!(matches!(p, Phase::Leaving { spot: 2, k } if (k - 10.0 / 15.0).abs() < 1e-6));
        let (t, p) = step(t, free, None, 100);
        assert_eq!(p, Phase::Emerging { spot: 2, k: 0.0 });
        let (_, p) = step(t, free, None, 100 + EMERGE);
        assert_eq!(p, Phase::Out);
        let (t, p) = step(Track::default(), hid, None, 7);
        assert_eq!(p, Phase::Hidden { spot: 2, age: 0, dove: true });
        assert_eq!(door(p), Some((2, 1.0)));
        assert_eq!(step(t, hid, None, 7 + DIVE).1, Phase::Hidden { spot: 2, age: DIVE, dove: true });
        assert_eq!(step(t, PlayerState::Grabbed, None, 9).1, Phase::Out);
    }

    #[test]
    fn door_opens_then_closes_around_the_body() {
        let at = |k| door(Phase::Entering { spot: 0, k }).map_or(-1.0, |d| d.1);
        assert_eq!(at(0.0), 0.0);
        assert!(at(0.5) > 0.99);
        assert!(at(14.0 / 15.0) < 0.1);
        let open = |k| door(Phase::Leaving { spot: 0, k }).map_or(-1.0, |d| d.1);
        assert!(open(0.2) < 0.01 && open(1.0) > 0.99);
        assert!(door(Phase::Emerging { spot: 0, k: 1.0 }).is_some_and(|d| d.1 < 0.01));
    }

    #[test]
    fn body_walks_in_and_faces_out() {
        let h = Hide { x: 3, y: 5, kind: HideKind::GuardaRoupa, rot: Ang::ZERO };
        let pos = [3.5, 6.4];
        let start = pose(&h, Phase::Entering { spot: 0, k: 0.0 }, pos, 1.0).expect("pose");
        assert_eq!(start.at, pos);
        let end = pose(&h, Phase::Entering { spot: 0, k: 1.0 }, pos, 1.0).expect("pose");
        assert!((end.at[1] - 5.55).abs() < 1e-4 && (end.at[0] - 3.5).abs() < 1e-4);
        assert!(end.heading.abs() < 1e-4);
        let out = pose(&h, Phase::Emerging { spot: 0, k: 1.0 }, pos, 0.0).expect("pose");
        assert!((out.at[1] - pos[1]).abs() < 1e-4);
        let (eye, target) = peek(&h, pos, 0.0);
        assert!(eye[2] > 5.5 && target[2] > eye[2] && eye[1] > 1.0);
        let bed = Hide { kind: HideKind::Cama, ..h };
        assert!(peek(&bed, pos, 0.0).0[1] < 0.3);
        assert_eq!(pose(&bed, Phase::Entering { spot: 0, k: 0.6 }, pos, 0.0).map(|p| p.anim), Some(Anim::Crouch));
    }
}
