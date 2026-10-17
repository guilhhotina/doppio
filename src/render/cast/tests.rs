use super::*;
use std::cell::RefCell;
use std::f32::consts::FRAC_PI_2;

const ANIMS: [Anim; 9] = [Anim::Idle, Anim::Walk, Anim::Run, Anim::Turn, Anim::QuickTurn, Anim::Reach, Anim::Crouch, Anim::Grabbed, Anim::Faint];
const LEGEND_ANIMS: [LegendAnim; 8] = [LegendAnim::Idle, LegendAnim::Walk, LegendAnim::Run, LegendAnim::Sniff, LegendAnim::Grab, LegendAnim::Stagger, LegendAnim::Special, LegendAnim::Alert];

fn hero_speed(anim: Anim) -> [f32; 2] {
    match anim {
        Anim::Walk => [1.3, 0.0],
        Anim::Run => [2.6, 0.0],
        Anim::Turn => [0.0, 2.618],
        Anim::QuickTurn => [0.0, 7.854],
        _ => [0.0; 2],
    }
}

fn legend_speed(kind: LegendKind, anim: LegendAnim) -> [f32; 2] {
    let speed = match (kind, anim) {
        (LegendKind::CorpoSeco, LegendAnim::Walk) => 0.9,
        (LegendKind::CorpoSeco, LegendAnim::Run) => 2.0,
        (LegendKind::Mula, LegendAnim::Run) => 3.5,
        (_, LegendAnim::Walk) => 1.3,
        (_, LegendAnim::Run) => 2.2,
        _ => 0.0,
    };
    [speed, 0.0]
}

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
        Who::Hero(c) => hero(c, x, z, heading, Anim::Idle, 0, false, 0.0, hero_speed(Anim::Idle)),
        Who::Legend(k) => legend(k, x, z, heading, LegendAnim::Idle, 0, legend_speed(k, LegendAnim::Idle)),
    }
}

#[test]
fn budgets_and_segments() {
    let c = cast();
    everyone().iter().for_each(|&who| {
        let (near, far) = (c.model(who, false), c.model(who, true));
        assert!((200..=820).contains(&near.mesh.tris()), "{who:?} near {}", near.mesh.tris());
        assert!(far.mesh.tris() <= 170, "{who:?} far {}", far.mesh.tris());
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
        let vertices = posed(&c, who, &r, false);
        let (index, lowest) = vertices.iter().enumerate().min_by(|(_, a), (_, b)| a.pos[1].total_cmp(&b.pos[1])).unwrap();
        let segment = c.model(who, false).mesh.verts[index].nrm[3];
        assert!(lowest.pos[1] > -0.05 && lowest.pos[1] < 0.06, "{who:?} {tag} floor {:?} segment {segment}", lowest.pos);
    };
    (0..HEROES).for_each(|h| {
        ANIMS.iter().for_each(|&a| (0..90).step_by(7).for_each(|t| check(Who::Hero(h), hero(h, 1.0, 1.0, 0.3, a, t, t % 2 == 0, 0.4, hero_speed(a)), &format!("{a:?} {t}"))));
    });
    LEGENDS.iter().for_each(|&k| {
        LEGEND_ANIMS.iter().for_each(|&a| (0..90).step_by(7).for_each(|t| check(Who::Legend(k), legend(k, 1.0, 1.0, 0.3, a, t, legend_speed(k, a)), &format!("{a:?} {t}"))));
    });
}

#[test]
fn heading_and_lantern() {
    let r = hero(0, 0.0, 0.0, 0.0, Anim::Walk, 7, true, 0.0, hero_speed(Anim::Walk));
    assert!(r.lantern[2] > 0.15 && (0.7..1.4).contains(&r.lantern[1]), "{:?}", r.lantern);
    assert!(r.beam[2] > 0.8, "{:?}", r.beam);
    let r = hero(2, 0.0, 0.0, FRAC_PI_2, Anim::Idle, 0, true, 0.0, hero_speed(Anim::Idle));
    assert!(r.lantern[0] > 0.15 && r.head.1[0] > 0.9, "{:?} {:?}", r.lantern, r.head);
    let off = hero(1, 0.0, 0.0, 0.0, Anim::Idle, 0, false, 0.0, hero_speed(Anim::Idle));
    assert!(off.mats[LAMP][..12].iter().all(|&v| v == 0.0));
    let bia = hero(3, 0.0, 0.0, 0.0, Anim::Idle, 0, false, 0.0, hero_speed(Anim::Idle));
    assert!(bia.mats[LAMP][15] == 1.0, "camcorder always held");
    let filming = hero(3, 0.0, 0.0, 0.0, Anim::Idle, 0, true, 0.0, hero_speed(Anim::Idle));
    assert!(filming.lantern[1] > 1.25 && filming.lantern[2] > 0.1, "camcorder raised {:?}", filming.lantern);
}

#[test]
fn hands_follow_the_fists() {
    (0..HEROES).for_each(|h| {
        let r = hero(h, 0.0, 0.0, 0.0, Anim::Reach, 20, false, 0.0, hero_speed(Anim::Reach));
        let grip = mat::point(&r.hands[0], [0.0; 3]);
        assert!(grip[2] > 0.3 && grip[1] > 0.7, "hero {h} reach grip {grip:?}");
        let r = hero(h, 0.0, 0.0, 0.0, Anim::Walk, 0, false, 0.0, hero_speed(Anim::Walk));
        let (right, left) = (mat::point(&r.hands[0], [0.0; 3]), mat::point(&r.hands[1], [0.0; 3]));
        assert!(right[0] < left[0], "hero {h} hands sides");
    });
}

#[test]
fn faint_lies_down_and_grab_struggles() {
    let c = cast();
    (0..HEROES).for_each(|h| {
        let (_, top) = bounds(&posed(&c, Who::Hero(h), &hero(h, 0.0, 0.0, 0.0, Anim::Faint, 60, false, 0.0, hero_speed(Anim::Faint)), false));
        assert!(top[1] < 0.55, "hero {h} still up at {}", top[1]);
        let a = hero(h, 0.0, 0.0, 0.0, Anim::Grabbed, 3, false, 0.0, hero_speed(Anim::Grabbed));
        let b = hero(h, 0.0, 0.0, 0.0, Anim::Grabbed, 9, false, 0.0, hero_speed(Anim::Grabbed));
        assert!(a.mats[FORE_L] != b.mats[FORE_L]);
    });
}

#[test]
fn legend_traits() {
    let c = cast();
    assert!(legend(LegendKind::Mula, 0.0, 0.0, 0.0, LegendAnim::Idle, 0, [0.0; 2]).fire.is_some());
    assert!(legend(LegendKind::Cabra, 0.0, 0.0, 0.0, LegendAnim::Walk, 9, [1.3, 0.0]).fire.is_none());
    let breath = legend(LegendKind::Cabra, 0.0, 0.0, 0.0, LegendAnim::Special, 12, legend_speed(LegendKind::Cabra, LegendAnim::Special)).fire.expect("fire breath");
    assert!(breath.1[2] > 0.5 && breath.2 > 0.5, "{breath:?}");
    [LegendKind::CorpoSeco, LegendKind::Cabra, LegendKind::Matinta].iter().for_each(|&k| {
        let r = legend(k, 0.0, 0.0, 0.0, LegendAnim::Idle, 0, legend_speed(k, LegendAnim::Idle));
        let eyes = r.eyes.expect("eyes");
        assert!(eyes.iter().all(|e| e[2] > 0.0 && e[1] > 1.0), "{k:?} {eyes:?}");
    });
    let stand = bounds(&posed(&c, Who::Legend(LegendKind::Cabra), &legend(LegendKind::Cabra, 0.0, 0.0, 0.0, LegendAnim::Idle, 0, legend_speed(LegendKind::Cabra, LegendAnim::Idle)), false)).1[1];
    let run = bounds(&posed(&c, Who::Legend(LegendKind::Cabra), &legend(LegendKind::Cabra, 0.0, 0.0, 0.0, LegendAnim::Run, 4, legend_speed(LegendKind::Cabra, LegendAnim::Run)), false)).1[1];
    assert!(stand > 2.0 && run < stand * 0.8, "cabra stand {stand} run {run}");
    let rear = bounds(&posed(&c, Who::Legend(LegendKind::Mula), &legend(LegendKind::Mula, 0.0, 0.0, 0.0, LegendAnim::Grab, 20, legend_speed(LegendKind::Mula, LegendAnim::Grab)), false)).1[1];
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

fn skirted() -> Vec<(Who, String, Rig)> {
    let heroes = [Who::Hero(0), Who::Hero(1)].into_iter().flat_map(|who| {
        let Who::Hero(c) = who else { unreachable!() };
        let moving = [(Anim::Walk, [-0.8, 0.0]), (Anim::Walk, [1.3, 1.5]), (Anim::Run, [2.6, -1.5])];
        ANIMS.iter().map(|&a| (a, hero_speed(a))).chain(moving).flat_map(move |(a, v)| (0..120).step_by(2).map(move |tic| (who, format!("{a:?} {v:?} tic {tic}"), hero(c, 0.0, 0.0, 0.0, a, tic, true, 0.0, v))))
    });
    let matinta = LEGEND_ANIMS.iter().flat_map(|&a| (0..120).step_by(2).map(move |tic| {
        let k = LegendKind::Matinta;
        (Who::Legend(k), format!("{a:?} tic {tic}"), legend(k, 0.0, 0.0, 0.0, a, tic, legend_speed(k, a)))
    }));
    heroes.chain(matinta).collect()
}

fn pelvis_local(cast: &Cast, who: Who, rig: &Rig, far: bool) -> Vec<V3> {
    posed(cast, who, rig, far).iter().map(|v| locomotion::unpoint(&rig.mats[PELVIS], v.pos)).collect()
}

#[test]
fn cloth_contains_legs_through_every_animation_and_lod() {
    let cast = cast();
    skirted().into_iter().for_each(|(who, tag, rig)| {
        [false, true].into_iter().for_each(|far| {
            let model = cast.model(who, far);
            let local = pelvis_local(&cast, who, &rig, far);
            let skirt: Vec<[V3; 3]> = model.mesh.idx.chunks_exact(3).filter(|tri| model.mesh.verts[tri[0] as usize].nrm[3] as usize == SKIRT).map(|tri| [local[tri[0] as usize], local[tri[1] as usize], local[tri[2] as usize]]).collect();
            let hem = model.cloth.unwrap().hem;
            let bottom = model.mesh.verts.iter().zip(&local).filter(|(v, _)| v.nrm[3] as usize == SKIRT && v.pos[1] < -hem + 0.001).map(|(_, p)| p[1]).fold(f32::MIN, f32::max);
            let top = skirt.iter().flatten().map(|p| p[1]).fold(f32::MIN, f32::max);
            model.mesh.verts.iter().zip(&local).filter(|(v, p)| cloth::leg(v.nrm[3] as usize) && p[1] > bottom + 0.001 && p[1] < top - 0.001).for_each(|(_, p)| {
                let hits: Vec<f32> = skirt.iter().filter_map(|tri| {
                    let [a, b, c] = *tri;
                    let det = (b[1] - a[1]) * (c[2] - a[2]) - (c[1] - a[1]) * (b[2] - a[2]);
                    if det.abs() < 1e-7 { return None; }
                    let u = ((p[1] - a[1]) * (c[2] - a[2]) - (c[1] - a[1]) * (p[2] - a[2])) / det;
                    let v = ((b[1] - a[1]) * (p[2] - a[2]) - (p[1] - a[1]) * (b[2] - a[2])) / det;
                    (u >= -1e-4 && v >= -1e-4 && u + v <= 1.0001).then_some(a[0] + u * (b[0] - a[0]) + v * (c[0] - a[0]))
                }).collect();
                let lo = hits.iter().copied().fold(f32::MAX, f32::min);
                let hi = hits.iter().copied().fold(f32::MIN, f32::max);
                assert!(p[0] >= lo - 0.002 && p[0] <= hi + 0.002, "{who:?} far={far} {tag} leg={p:?} skirt=[{lo},{hi}]");
            });
        });
    });
}

#[test]
fn skirt_stays_sane() {
    let cast = cast();
    let mut worst = [0.0f32; 4];
    skirted().into_iter().for_each(|(who, tag, rig)| {
        let model = cast.model(who, false);
        let cloth = model.cloth.unwrap();
        let local = pelvis_local(&cast, who, &rig, false);
        let skirt: Vec<V3> = model.mesh.verts.iter().zip(&local).filter(|(v, _)| v.nrm[3] as usize == SKIRT).map(|(_, p)| *p).collect();
        let hem: Vec<V3> = model.mesh.verts.iter().zip(&local).filter(|(v, _)| v.nrm[3] as usize == SKIRT && v.pos[1] < -cloth.hem + 0.001).map(|(_, p)| *p).collect();
        let reach = |axis: usize| skirt.iter().map(|p| p[axis].abs()).fold(0.0, f32::max);
        let extent = |axis: usize| hem.iter().fold([f32::MAX, f32::MIN], |[lo, hi], p| [lo.min(p[axis]), hi.max(p[axis])]);
        let [x, z] = [extent(0), extent(2)];
        let (half, centre) = ([(x[1] - x[0]) * 0.5, (z[1] - z[0]) * 0.5], [(x[1] + x[0]) * 0.5, (z[1] + z[0]) * 0.5]);
        worst = [worst[0].max(half[0] / cloth.edge[0]), worst[1].max(half[1] / cloth.edge[1]), worst[2].max(centre[1].abs()), worst[3].max(reach(2) - cloth.edge[1])];
        assert!(half[0] <= cloth.edge[0] * 1.6 && half[1] <= cloth.edge[1] * 1.8, "{who:?} {tag} hem flare {half:?}");
        assert!(centre[0].abs() <= 0.08 && centre[1].abs() <= 0.25, "{who:?} {tag} hem shifted {centre:?}");
        assert!(reach(0) <= cloth.edge[0] + 0.3 && reach(2) <= cloth.edge[1] + 0.3, "{who:?} {tag} train {} {}", reach(0), reach(2));
        assert!(skirt.iter().all(|p| p[1] >= -cloth.hem - 0.002 && p[1] <= cloth.top + 0.002), "{who:?} {tag} hem length");
        assert!(hem.iter().all(|p| p[1] <= -0.1 + 0.002), "{who:?} {tag} hem rides up");
    });
    eprintln!("worst hem flare x {:.2} z {:.2}, shift {:.2}, train {:.2}", worst[0], worst[1], worst[2], worst[3]);
}

#[test]
fn planted_feet_match_forward_running_and_backward_speeds() {
    (0..4).for_each(|c| {
        [(Anim::Walk, 1.3), (Anim::Run, 2.6), (Anim::Walk, -0.8)].into_iter().for_each(|(anim, speed)| {
            let gait = motion::gait_of(c, anim).unwrap();
            (1..(gait.period * gait.duty) as u16 - 1).for_each(|t| {
                let a = hero(c, 0.0, speed * t as f32 / 30.0, 0.0, anim, t, false, 0.0, [speed, 0.0]);
                let b = hero(c, 0.0, speed * (t + 1) as f32 / 30.0, 0.0, anim, t + 1, false, 0.0, [speed, 0.0]);
                let pa = mat::point(&a.mats[FOOT_L], [0.0, -heroes::frame(c).sole, 0.0]);
                let pb = mat::point(&b.mats[FOOT_L], [0.0, -heroes::frame(c).sole, 0.0]);
                assert!(mat::len(mat::sub(pa, pb)) < 1e-5, "hero {c} {anim:?} {speed} {pa:?} {pb:?}");
                assert!(pa[1].abs() < 1e-5);
            });
        });
    });
}

#[test]
fn travel_sampling_handles_repeated_frames_wrap_and_portals() {
    let mut travel = Travel::default();
    assert_eq!(travel.sample(0, 0, [0.0; 3]), [0.0; 2]);
    let forward = travel.sample(1, 0, [0.0, 1.3 / 30.0, 0.0]);
    assert!((forward[0] - 1.3).abs() < 1e-5);
    assert_eq!(travel.sample(1, 0, [0.0, 1.3 / 30.0, 0.0]), forward);
    assert!((travel.sample(2, 0, [0.0, 0.5 / 30.0, 0.0])[0] + 0.8).abs() < 1e-5);
    assert_eq!(travel.sample(3, 1, [30.0, 30.0, 3.13]), [0.0; 2]);
    assert!(travel.sample(4, 1, [30.0, 30.0, -3.13])[1].abs() < 1.0);
}

#[test]
fn turning_steps_hold_the_contact_foot_in_world_space() {
    [(Anim::Turn, 2.618, 18), (Anim::QuickTurn, 7.854, 12)].into_iter().for_each(|(anim, rate, period)| {
        (0..4).for_each(|c| (1..period / 2).for_each(|t| {
            let a = hero(c, 0.0, 0.0, rate * t as f32 / 30.0, anim, t, false, 0.0, [0.0, rate]);
            let b = hero(c, 0.0, 0.0, rate * (t + 1) as f32 / 30.0, anim, t + 1, false, 0.0, [0.0, rate]);
            let foot = |r: Rig| mat::point(&r.mats[FOOT_L], [0.0, -heroes::frame(c).sole, 0.0]);
            assert!(mat::len(mat::sub(foot(a), foot(b))) < 1e-5, "hero {c} {anim:?} tic {t}");
        }));
    });
}

#[test]
fn legend_contacts_match_patrol_chase_and_charge() {
    LEGENDS.into_iter().for_each(|kind| {
        [LegendAnim::Walk, LegendAnim::Run].into_iter().for_each(|anim| {
            let run = matches!(anim, LegendAnim::Run);
            let (period, len, sole) = match kind {
                LegendKind::CorpoSeco => (corpo::PERIOD[run as usize], 0.0, corpo::FRAME.sole),
                LegendKind::Cabra => (cabra::PERIOD[run as usize], cabra::FRAME.meta, cabra::FRAME.sole),
                LegendKind::Mula => (mula::PERIOD[run as usize], 0.34, 0.06),
                LegendKind::Matinta => (matinta::PERIOD[run as usize], 0.0, matinta::FRAME.sole),
            };
            let period = period as u16;
            let speed = legend_speed(kind, anim)[0];
            (1..period / 3 - 1).for_each(|t| {
                let a = legend(kind, 0.0, speed * t as f32 / 30.0, 0.0, anim, t, [speed, 0.0]);
                let b = legend(kind, 0.0, speed * (t + 1) as f32 / 30.0, 0.0, anim, t + 1, [speed, 0.0]);
                let foot = |r: Rig| mat::point(&r.mats[FOOT_L], [0.0, -len - sole, 0.0]);
                assert!(mat::len(mat::sub(foot(a), foot(b))) < 1e-4, "{kind:?} {anim:?} tic {t}");
            });
        });
    });
}

#[test]
fn held_bucket_uses_handle_and_long_tools_point_up() {
    use crate::map::ItemKind;
    (0..HEROES).for_each(|c| {
        let rig = hero(c, 0.0, 0.0, 0.0, Anim::Idle, 0, false, 0.0, [0.0; 2]);
        let grip = &rig.hands[0];
        let bucket = held(ItemKind::Balde, grip);
        assert!(mat::len(mat::sub(mat::point(&bucket, [0.0, 0.28, 0.0]), mat::point(grip, [0.0; 3]))) < 1e-5);
        let paddle = held(ItemKind::Remo, grip);
        assert!(mat::dir(&paddle, [0.0, 0.0, 1.0])[1] > 0.7);
    });
}
