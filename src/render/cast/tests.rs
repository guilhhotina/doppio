use super::*;
use std::cell::RefCell;
use std::f32::consts::FRAC_PI_2;

const ANIMS: [Anim; 9] = [Anim::Idle, Anim::Walk, Anim::Run, Anim::Turn, Anim::QuickTurn, Anim::Reach, Anim::Crouch, Anim::Grabbed, Anim::Faint];
const LEGEND_ANIMS: [LegendAnim; 7] = [LegendAnim::Idle, LegendAnim::Walk, LegendAnim::Run, LegendAnim::Sniff, LegendAnim::Grab, LegendAnim::Stagger, LegendAnim::Special];

fn everyone() -> Vec<Who> {
    (0..HEROES).map(Who::Hero).chain(LEGENDS.iter().map(|&k| Who::Legend(k))).collect()
}

fn cast() -> Cast {
    Cast::new(|n| [n.len() as u8, n.as_bytes()[5], 1, 1])
}

fn posed(c: &Cast, who: Who, rig: &Rig, far: bool) -> Vec<Vert> {
    let m = c.model(who, far);
    let mut out = vec![Vert::default(); m.mesh.verts.len()];
    m.skin(&rig.mats, |_| 0.5, 0.0, &mut out);
    out
}

fn bounds(v: &[Vert]) -> (V3, V3) {
    v.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), v| (std::array::from_fn(|i| lo[i].min(v.pos[i])), std::array::from_fn(|i| hi[i].max(v.pos[i]))))
}

fn rest(who: Who, x: f32, z: f32, heading: f32) -> Rig {
    match who {
        Who::Hero(c) => hero(c, x, z, heading, Anim::Idle, 0, false, 0.0),
        Who::Legend(k) => legend(k, x, z, heading, LegendAnim::Idle, 0),
    }
}

#[test]
fn budgets_and_segments() {
    let c = cast();
    everyone().iter().for_each(|&who| {
        let (near, far) = (c.model(who, false), c.model(who, true));
        assert!((200..=500).contains(&near.mesh.tris()), "{who:?} near {}", near.mesh.tris());
        assert!(far.mesh.tris() <= 135, "{who:?} far {}", far.mesh.tris());
        assert!(far.mesh.tris() * 2 < near.mesh.tris(), "{who:?}");
        assert!([near, far].iter().all(|m| m.mesh.verts.iter().all(|v| (v.nrm[3] as usize) < SEGS)));
        assert!([near, far].iter().all(|m| m.mesh.idx.iter().all(|&i| (i as usize) < m.mesh.verts.len())));
    });
    assert!(c.max_verts() * 8 <= u16::MAX as usize);
}

#[test]
fn only_cast_textures() {
    let names = RefCell::new(Vec::<String>::new());
    let _ = Cast::new(|n| {
        names.borrow_mut().push(n.to_string());
        [0; 4]
    });
    let names = names.into_inner();
    assert!(names.iter().all(|n| n.starts_with("cast_")), "{names:?}");
    ["cast_misc", "cast_benedita", "cast_tonho", "cast_jairo", "cast_bia", "cast_corpo", "cast_cabra", "cast_matinta"].iter().for_each(|n| assert!(names.iter().any(|m| m == n), "{n}"));
}

#[test]
fn standing_on_the_floor_with_distinct_heights() {
    let c = cast();
    let heights = [(0, 1.45, 1.62), (1, 1.28, 1.45), (2, 1.8, 1.98), (3, 1.6, 1.75)];
    heights.iter().for_each(|&(h, lo, hi)| {
        let who = Who::Hero(h);
        [false, true].iter().for_each(|&far| {
            let v = posed(&c, who, &rest(who, 3.0, 4.0, 0.0), far);
            let (a, b) = bounds(&v);
            assert!(a[1].abs() < 0.02, "{who:?} feet at {}", a[1]);
            assert!((lo..hi).contains(&b[1]), "{who:?} top at {}", b[1]);
        });
    });
    everyone().iter().for_each(|&who| {
        let v = posed(&c, who, &rest(who, 3.0, 4.0, 0.7), false);
        let (a, b) = bounds(&v);
        let e = extent(who);
        assert!(a[1].abs() < 0.02, "{who:?} floor {}", a[1]);
        assert!(b[1] <= e[1] * 2.0 + 0.05, "{who:?} top {} > {}", b[1], e[1] * 2.0);
        assert!((a[0] - 3.0).abs() <= e[0] + 0.05 && (b[0] - 3.0).abs() <= e[0] + 0.05, "{who:?} x {a:?} {b:?}");
        assert!((a[2] - 4.0).abs() <= e[2] + 0.05 && (b[2] - 4.0).abs() <= e[2] + 0.05, "{who:?} z {a:?} {b:?}");
    });
}

#[test]
fn every_animation_is_finite_and_grounded() {
    let c = cast();
    let check = |who: Who, r: Rig, tag: &str| {
        assert!(r.mats.iter().flatten().chain(r.hands.iter().flatten()).all(|v| v.is_finite()), "{who:?} {tag}");
        let (a, _) = bounds(&posed(&c, who, &r, false));
        assert!(a[1] > -0.05 && a[1] < 0.06, "{who:?} {tag} floor {}", a[1]);
    };
    (0..HEROES).for_each(|h| {
        ANIMS.iter().for_each(|&a| (0..90).step_by(7).for_each(|t| check(Who::Hero(h), hero(h, 1.0, 1.0, 0.3, a, t, t % 2 == 0, 0.4), &format!("{a:?} {t}"))));
    });
    LEGENDS.iter().for_each(|&k| {
        LEGEND_ANIMS.iter().for_each(|&a| (0..90).step_by(7).for_each(|t| check(Who::Legend(k), legend(k, 1.0, 1.0, 0.3, a, t), &format!("{a:?} {t}"))));
    });
}

#[test]
fn heading_and_lantern() {
    let r = hero(0, 0.0, 0.0, 0.0, Anim::Walk, 7, true, 0.0);
    assert!(r.lantern[2] > 0.15 && (0.7..1.4).contains(&r.lantern[1]), "{:?}", r.lantern);
    assert!(r.beam[2] > 0.8, "{:?}", r.beam);
    let r = hero(2, 0.0, 0.0, FRAC_PI_2, Anim::Idle, 0, true, 0.0);
    assert!(r.lantern[0] > 0.15 && r.head.1[0] > 0.9, "{:?} {:?}", r.lantern, r.head);
    let off = hero(1, 0.0, 0.0, 0.0, Anim::Idle, 0, false, 0.0);
    assert!(off.mats[LAMP][..12].iter().all(|&v| v == 0.0));
    let bia = hero(3, 0.0, 0.0, 0.0, Anim::Idle, 0, false, 0.0);
    assert!(bia.mats[LAMP][15] == 1.0, "camcorder always held");
    let filming = hero(3, 0.0, 0.0, 0.0, Anim::Idle, 0, true, 0.0);
    assert!(filming.lantern[1] > 1.25 && filming.lantern[2] > 0.1, "camcorder raised {:?}", filming.lantern);
}

#[test]
fn hands_follow_the_fists() {
    (0..HEROES).for_each(|h| {
        let r = hero(h, 0.0, 0.0, 0.0, Anim::Reach, 20, false, 0.0);
        let grip = mat::point(&r.hands[0], [0.0; 3]);
        assert!(grip[2] > 0.3 && grip[1] > 0.7, "hero {h} reach grip {grip:?}");
        let r = hero(h, 0.0, 0.0, 0.0, Anim::Walk, 0, false, 0.0);
        let (right, left) = (mat::point(&r.hands[0], [0.0; 3]), mat::point(&r.hands[1], [0.0; 3]));
        assert!(right[0] < left[0], "hero {h} hands sides");
    });
}

#[test]
fn faint_lies_down_and_grab_struggles() {
    let c = cast();
    (0..HEROES).for_each(|h| {
        let (_, top) = bounds(&posed(&c, Who::Hero(h), &hero(h, 0.0, 0.0, 0.0, Anim::Faint, 60, false, 0.0), false));
        assert!(top[1] < 0.55, "hero {h} still up at {}", top[1]);
        let a = hero(h, 0.0, 0.0, 0.0, Anim::Grabbed, 3, false, 0.0);
        let b = hero(h, 0.0, 0.0, 0.0, Anim::Grabbed, 9, false, 0.0);
        assert!(a.mats[FORE_L] != b.mats[FORE_L]);
    });
}

#[test]
fn legend_traits() {
    let c = cast();
    assert!(legend(LegendKind::Mula, 0.0, 0.0, 0.0, LegendAnim::Idle, 0).fire.is_some());
    assert!(legend(LegendKind::Cabra, 0.0, 0.0, 0.0, LegendAnim::Walk, 9).fire.is_none());
    let breath = legend(LegendKind::Cabra, 0.0, 0.0, 0.0, LegendAnim::Special, 12).fire.expect("fire breath");
    assert!(breath.1[2] > 0.5 && breath.2 > 0.5, "{breath:?}");
    [LegendKind::CorpoSeco, LegendKind::Cabra, LegendKind::Matinta].iter().for_each(|&k| {
        let r = legend(k, 0.0, 0.0, 0.0, LegendAnim::Idle, 0);
        let eyes = r.eyes.expect("eyes");
        assert!(eyes.iter().all(|e| e[2] > 0.0 && e[1] > 1.0), "{k:?} {eyes:?}");
    });
    let stand = bounds(&posed(&c, Who::Legend(LegendKind::Cabra), &legend(LegendKind::Cabra, 0.0, 0.0, 0.0, LegendAnim::Idle, 0), false)).1[1];
    let run = bounds(&posed(&c, Who::Legend(LegendKind::Cabra), &legend(LegendKind::Cabra, 0.0, 0.0, 0.0, LegendAnim::Run, 4), false)).1[1];
    assert!(stand > 2.0 && run < stand * 0.8, "cabra stand {stand} run {run}");
    let rear = bounds(&posed(&c, Who::Legend(LegendKind::Mula), &legend(LegendKind::Mula, 0.0, 0.0, 0.0, LegendAnim::Grab, 20), false)).1[1];
    assert!(rear > 2.0, "mula rears to {rear}");
}

#[test]
fn fade_dissolves_faces() {
    let c = cast();
    let who = Who::Legend(LegendKind::Matinta);
    let m = c.model(who, false);
    let r = rest(who, 0.0, 0.0, 0.0);
    let mut out = vec![Vert::default(); m.mesh.verts.len()];
    let gone = |out: &[Vert]| m.mesh.idx.chunks(3).filter(|t| t.iter().all(|&i| out[i as usize].pos == out[t[0] as usize].pos)).count();
    m.skin(&r.mats, |_| 0.5, 0.0, &mut out);
    assert_eq!(gone(&out), 0);
    m.skin(&r.mats, |_| 0.5, 0.5, &mut out);
    let half = gone(&out);
    assert!(half > m.mesh.tris() / 4 && half < m.mesh.tris() * 3 / 4, "{half}");
    m.skin(&r.mats, |_| 0.5, 1.0, &mut out);
    assert_eq!(gone(&out), m.mesh.tris());
}

#[test]
fn portraits_look_at_the_face() {
    everyone().iter().for_each(|&who| {
        let (eye, target, fov) = portrait_cam(who);
        assert!(eye[2] > 0.5 && target[1] > 0.5 && (20.0..70.0).contains(&fov), "{who:?}");
    });
}
