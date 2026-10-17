use super::*;

const SAMPLE: &str = "; two floors, a yard and a gate
[map] name=\"Casa teste — dois andares\" w=14 h=5 sky=moon fog=10,12,20 fog_near=3 fog_far=14.5 legend=corposeco power=off
[solid]
#######|# ####
#..D.P.H# #P.#
#.B#..~.# #..#
#..#....# #..#
######### ####
[rooms]
--------- ----
-aaabbbb- -cc-
-aa-bbbb- -cc-
-aa-bbbb- -cc-
--------- ----
[area]
0 térreo
1 andar de cima
[room]
a sala de estar area=0 floor=taco wall=reboco ceil=forro light=40
b quintal area=0 floor=grama wall=tijolo ceil=- ext=muro h=3 light=12 out=1
c quarto area=1 floor=assoalho wall=papel ceil=forro light=20
[item]
chavinha chave 1 1
fio alicate 6.5 3.5
fus fusivel 1 3
cr cruz 12 3
[lock]
bau_l key chavinha hint=\"Ache a chavinha do baú\"
cofre_l code 317
corrente tool alicate
luz place fus 2 1
saida all corrente luz hint=\"Corte a corrente e ligue a luz\"
[door]
3 1 v locked:bau_l
[portal]
5 1 -> 11 2 0 escada
11 1 -> 6 1 90 escada lock=cofre_l
[prop]
bau 1.5 2.5 0 item=fus
cofre 12.5 3.5 270 lock=cofre_l item=cr
sofa 1.5 3.5 90
[note]
1 2 calendario \"Dia 17: 3-1-7 \\o/\"
[hide]
7 1 armario 180
2 2 cama 0
[light]
6.5 2 2.4 255,200,120 4 flicker
[cam]
0 0 4 4 1.5 2.4 3.5 2 0.5 1 60
5 0 8 4 4.5 2.4 3.5 4 0 1 55.5
10 0 13 4 11 2 3 12 0 1 70
[wake]
0 11 2 0
1 12 2 90
2 12.5 3.5 180
3 11 3 270
[legend]
spawn 6.5 3.5 180
patrol 5 2
patrol 5 3
patrol 7 3
patrol 11 2
patrol 12 3
patrol 1 1
lair 7 2
[exit]
6 2 7 3 lock=saida
";

fn with_line(n: usize, replacement: &str) -> String {
    SAMPLE.lines().enumerate().map(|(i, l)| if i + 1 == n { replacement } else { l }).collect::<Vec<_>>().join("\n")
}

fn line_of(text: &str) -> usize {
    SAMPLE.lines().position(|l| l == text).map_or_else(|| panic!("no line {text:?}"), |i| i + 1)
}

fn fails_at(n: usize, replacement: &str) -> usize {
    parse(&with_line(n, replacement)).map_or_else(|e| e.line, |_| panic!("line {n} -> {replacement:?} parsed"))
}

#[test]
fn parses_header_and_grid() {
    let m = parse(SAMPLE).unwrap();
    assert_eq!((m.name, m.w, m.h, m.sky, m.fog, m.legend, m.power), ("Casa teste — dois andares", 14, 5, Sky::Moon, [10, 12, 20], LegendKind::CorpoSeco, false));
    assert_eq!((m.fog_near, m.fog_far), (Fx::int(3), Fx::int(14) + Fx::HALF));
    assert_eq!([m.tile(3, 1), m.tile(5, 1), m.tile(7, 1), m.tile(6, 2), m.tile(7, 0), m.tile(9, 1), m.tile(-1, 0)], [
        Tile::Door,
        Tile::Portal,
        Tile::Hide,
        Tile::Water,
        Tile::Fence,
        Tile::Void,
        Tile::Void
    ]);
    assert!(Tile::Portal.walkable() && !Tile::Hide.walkable() && !Tile::Fence.walkable());
}

#[test]
fn parses_rooms_and_areas() {
    let m = parse(SAMPLE).unwrap();
    assert_eq!(m.areas.iter().map(|a| a.name).collect::<Vec<_>>(), ["térreo", "andar de cima"]);
    let sala = m.room_at(1, 1).unwrap();
    assert_eq!((sala.name, sala.area, sala.ceil, sala.ext, sala.h, sala.out), ("sala de estar", 0, Some("forro"), None, Fx::int(2) + Fx::ratio(3, 5), false));
    let quintal = m.room_at(6, 2).unwrap();
    assert_eq!((quintal.ceil, quintal.ext, quintal.h, quintal.out, quintal.light), (None, Some("muro"), Fx::int(3), true, 12));
    assert_eq!((m.area_at(12, 2), m.area_at(1, 1), m.area_at(0, 0)), (Some(1), Some(0), None));
    assert_eq!(m.room_index(12, 2), Some(2));
}

#[test]
fn resolves_locks_items_and_containers() {
    let m = parse(SAMPLE).unwrap();
    let (chavinha, fio, fus, cr) = (m.item_index("chavinha").unwrap(), m.item_index("fio").unwrap(), m.item_index("fus").unwrap(), m.item_index("cr").unwrap());
    assert_eq!((chavinha, fio, fus, cr), (0, 1, 2, 3));
    assert_eq!(m.items[1].pos, Vec2::new(Fx::int(6) + Fx::HALF, Fx::int(3) + Fx::HALF));
    assert_eq!(m.items[0].pos, Vec2::new(Fx::int(1) + Fx::HALF, Fx::int(1) + Fx::HALF));
    assert_eq!(m.props[0].pos, Vec2::new(Fx::int(1) + Fx::HALF, Fx::int(2) + Fx::HALF));
    assert_eq!((m.items[fus as usize].inside, m.items[cr as usize].inside, m.items[0].inside), (Some(0), Some(1), None));
    let l = |id: &str| m.locks[m.lock_index(id).unwrap() as usize].kind.clone();
    assert_eq!(l("bau_l"), LockKind::Key(chavinha));
    assert_eq!(l("cofre_l"), LockKind::Code { digits: [3, 1, 7, 0], len: 3 });
    assert_eq!(l("corrente"), LockKind::Tool(ItemKind::Alicate));
    assert_eq!(l("luz"), LockKind::Place { item: fus, x: 2, y: 1 });
    assert_eq!(l("saida"), LockKind::All(vec![2, 3]));
    assert_eq!(m.doors, vec![Door { x: 3, y: 1, axis: Axis::V, state: DoorState::Locked(0) }]);
    assert_eq!(m.props[1].lock, Some(1));
    assert_eq!(m.exit, Exit { rect: Rect { x0: 6, y0: 2, x1: 7, y1: 3 }, lock: 4 });
    assert_eq!(m.lock_index("nada"), None);
    let hints = m.locks.iter().map(|l| l.hint).collect::<Vec<_>>();
    assert_eq!(hints, [Some("Ache a chavinha do baú"), None, None, None, Some("Corte a corrente e ligue a luz")]);
    let bau = line_of("bau_l key chavinha hint=\"Ache a chavinha do baú\"");
    assert_eq!(fails_at(bau, "bau_l key chavinha hint=sem_aspas"), bau);
    assert_eq!(fails_at(bau, "bau_l key chavinha hint=\"\""), bau);
    assert_eq!(fails_at(bau, &format!("bau_l key chavinha hint=\"{}\"", "x".repeat(parse::HINT_MAX_CHARS + 1))), bau);
    assert_eq!(fails_at(bau, "bau_l key hint=\"sem chave\""), bau);
}

#[test]
fn parses_portals_notes_hides_lights_wakes_legend() {
    let m = parse(SAMPLE).unwrap();
    assert_eq!(m.portals[0], Portal { x: 5, y: 1, to_x: 11, to_y: 2, rot: Ang(0), kind: PortalKind::Escada, lock: None });
    assert_eq!((m.portals[1].rot, m.portals[1].lock), (Ang::QUARTER, Some(1)));
    assert_eq!((m.portal_at(11, 1), m.portal_at(1, 1), m.door_at(3, 1), m.hide_at(2, 2)), (Some(1), None, Some(0), Some(1)));
    assert_eq!((m.notes[0].target, m.notes[0].text), ("calendario", "Dia 17: 3-1-7 \\o/"));
    assert_eq!((m.hides[0].kind, m.hides[1].kind), (HideKind::Armario, HideKind::Cama));
    assert_eq!((m.lights[0].rgb, m.lights[0].mode, m.lights[0].radius), ([255, 200, 120], LightMode::Flicker, Fx::int(4)));
    assert_eq!(m.wakes[2].pos, Vec2::new(Fx::int(12) + Fx::HALF, Fx::int(3) + Fx::HALF));
    assert_eq!(m.wakes[0].pos, Vec2::new(Fx::int(11) + Fx::HALF, Fx::int(2) + Fx::HALF));
    assert_eq!(m.lights[0].pos, Vec2::new(Fx::int(6) + Fx::HALF, Fx::int(2)));
    assert_eq!((m.legend_spawns.len(), m.patrol.len(), m.patrol[3], m.lair), (1, 6, (11, 2), (7, 2)));
    assert_eq!(m.cam_at(12, 2).map(|c| c.fov), Some(Fx::int(70)));
}

#[test]
fn doorways_without_room_inherit_a_neighbour() {
    let src = SAMPLE.replace("-aaabbbb- -cc-", "-aa-bbbb- -cc-");
    let m = parse(&src).unwrap();
    assert_eq!((m.room_index(3, 1), m.room_index(4, 1), m.room_index(3, 2)), (Some(0), Some(1), None));
}

#[test]
fn defaults() {
    let src = SAMPLE.replace(" power=off", "").replace(" fog_near=3 fog_far=14.5", "");
    let m = parse(&src).unwrap();
    assert_eq!((m.power, m.fog_near, m.fog_far), (true, Fx::int(4), Fx::int(16)));
}

#[test]
fn errors_carry_line_numbers() {
    let solid = line_of("#..D.P.H# #P.#");
    assert_eq!(fails_at(solid, "#..D.P.X# #P.#"), solid);
    assert_eq!(fails_at(solid, "#..D.P.H# #P.##"), solid);
    let door = line_of("3 1 v locked:bau_l");
    assert_eq!(fails_at(door, "3 1 v locked:nope"), door);
    assert_eq!(fails_at(door, "2 1 v closed"), door);
    assert_eq!(fails_at(door, "3 1 x closed"), door);
    let portal = line_of("5 1 -> 11 2 0 escada");
    assert_eq!(fails_at(portal, "5 1 -> 13 1 0 escada"), portal);
    assert_eq!(fails_at(portal, "4 1 -> 11 2 0 escada"), portal);
    assert_eq!(fails_at(portal, "5 1 -> 11 2 0 elevador"), portal);
    let lock = line_of("cofre_l code 317");
    assert_eq!(fails_at(lock, "cofre_l code 31"), lock);
    assert_eq!(fails_at(lock, "cofre_l code 31a"), lock);
    assert_eq!(fails_at(lock, "cofre_l tool serrote"), lock);
    assert_eq!(fails_at(lock, "corrente tool alicate"), line_of("corrente tool alicate"));
    let saida = line_of("saida all corrente luz hint=\"Corte a corrente e ligue a luz\"");
    assert_eq!(fails_at(saida, "saida all saida"), saida);
    let note = line_of("1 2 calendario \"Dia 17: 3-1-7 \\o/\"");
    assert_eq!(fails_at(note, &format!("1 2 parede \"{}\"", "a".repeat(121))), note);
    assert_eq!(fails_at(note, "1 2 parede \"sem fim"), note);
    let hide = line_of("7 1 armario 180");
    assert_eq!(fails_at(hide, "6 1 armario 180"), hide);
    let prop = line_of("sofa 1.5 3.5 90");
    assert_eq!(fails_at(prop, "sofa 1.5 3.5 90 item=fus"), prop);
    assert_eq!(fails_at(prop, "sofa 1.5 abc 90"), prop);
    let room = line_of("c quarto area=1 floor=assoalho wall=papel ceil=forro light=20");
    assert_eq!(fails_at(room, "c quarto area=2 floor=assoalho wall=papel ceil=forro light=20"), room);
    assert_eq!(fails_at(room, "c quarto area=1 floor=assoalho wall=papel ceil=forro light=101"), room);
    assert_eq!(fails_at(room, "c quarto area=1 floor=assoalho ceil=forro light=20"), room);
    assert_eq!(fails_at(line_of("[area]"), "[areas]"), line_of("[area]"));
    assert_eq!(fails_at(line_of("3 11 3 270"), "3 11 3"), line_of("3 11 3 270"));
    assert_eq!(fails_at(line_of("2 12.5 3.5 180"), "1 12.5 3.5 180"), line_of("2 12.5 3.5 180"));
    assert_eq!(fails_at(line_of("lair 7 2"), "patrol 7 2"), SAMPLE.lines().count());
    let rooms_row = SAMPLE.lines().position(|l| l == "-aa-bbbb- -cc-").unwrap() + 1;
    assert_eq!(fails_at(rooms_row, "-aa-bbbz- -cc-"), rooms_row);
    assert_eq!(fails_at(rooms_row, "-aa--bbb- -cc-"), line_of("#.B#..~.# #..#"));
}

#[test]
fn campaign_maps_parse() {
    let maps = campaign();
    assert!(!maps.is_empty());
    assert!(maps.iter().all(|m| m.areas.len() >= 1 && m.rooms.iter().all(|r| (r.area as usize) < m.areas.len())));
    let listed = include_str!("../../assets/map/campaign.txt").lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with(';')).count();
    assert_eq!(maps.len(), listed);
}
