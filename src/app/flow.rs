use super::pad::{Key, list};
use crate::map::Map;
use crate::net::{self, Link, Listing, Net, Reason};
use crate::render::{Ui2d, WHITE};
use crate::ui::lore::{CHARACTERS, character, legend};
use crate::ui::art::{self, Button};
use crate::ui::{self, Body, DIM, INK, Item, Line, Party, PartySeat, SCREEN, Serif, centered, line, width};

pub const RELAYS: [(&str, Option<&str>); 5] =
    [("nenhum", None), ("São Paulo", Some("saopaulo")), ("Nova York", Some("nyc")), ("Madri", Some("madrid")), ("Singapura", Some("singapore"))];
const MAIN_ITEMS: usize = 5;
const PARTY_ITEMS: usize = 3;
const NO_ANSWER_FRAMES: u32 = 20 * 60;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Notice {
    Lost,
    Version,
    Full,
    Started,
    Desync,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Title,
    Main { sel: usize },
    Campaign { sel: usize, couch: bool },
    Party { sel: usize, couch: bool },
    HowTo,
    Credits { page: usize },
    Online { sel: usize },
    Create { sel: usize, public: bool, relay: usize },
    Rooms { sel: usize },
    Lobby { since: u32, hosting: bool, sel: usize },
    Notice(Notice),
    Game,
}

impl Screen {
    pub const fn main() -> Screen {
        Screen::Main { sel: 0 }
    }

    pub fn picking(self) -> bool {
        matches!(self, Screen::Party { .. } | Screen::Lobby { .. })
    }

    pub fn joining(self) -> bool {
        matches!(self, Screen::Party { couch: true, .. } | Screen::Lobby { .. })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    None,
    Solo,
    Couch,
    Phase(u8),
    PhaseStep(i8),
    Character(i8),
    Begin,
    Host { public: bool, relay: usize },
    Refresh,
    Join(usize),
    Start,
    Ready,
    Leave,
}

pub struct Ctx {
    pub lobby: bool,
    pub rooms: usize,
    pub link: Link,
    pub players: usize,
    pub remote: bool,
    pub frame: u32,
    pub phases: usize,
    pub unlocked: usize,
}

pub fn lost_notice(reason: Option<Reason>) -> Notice {
    match reason {
        Some(Reason::Version) => Notice::Version,
        Some(Reason::Full) => Notice::Full,
        Some(Reason::Started) => Notice::Started,
        Some(Reason::Bye) | None => Notice::Lost,
    }
}

pub fn next_free(current: u8, dir: i8, taken: u8) -> u8 {
    let n = CHARACTERS.len() as i8;
    (1..=n).map(|k| (current as i8 + dir.signum() * k).rem_euclid(n) as u8).find(|&c| taken & 1 << c == 0).unwrap_or(current)
}

fn side(key: Option<Key>) -> Option<i8> {
    match key {
        Some(Key::Left) => Some(-1),
        Some(Key::Right | Key::Confirm) => Some(1),
        _ => None,
    }
}

fn lobby_items(hosting: bool) -> usize {
    if hosting { 4 } else { 3 }
}

pub fn step(screen: Screen, key: Option<Key>, c: &Ctx) -> (Screen, Action) {
    use Screen as S;
    let sideways = matches!(key, Some(Key::Left | Key::Right | Key::Confirm));
    match (screen, key) {
        (S::Title, Some(Key::Confirm)) => (S::main(), Action::None),
        (S::Main { .. }, Some(Key::Back)) => (S::Title, Action::None),
        (S::Main { sel }, Some(Key::Confirm)) => match sel {
            0 => (S::Campaign { sel: 0, couch: false }, Action::Solo),
            1 => (S::Campaign { sel: 0, couch: true }, Action::Couch),
            2 if c.lobby => (S::Online { sel: 0 }, Action::None),
            3 => (S::HowTo, Action::None),
            4 => (S::Credits { page: 0 }, Action::None),
            _ => (screen, Action::None),
        },
        (S::Main { sel }, k) => (S::Main { sel: list(sel, MAIN_ITEMS, k) }, Action::None),
        (S::Campaign { couch, .. }, Some(Key::Back)) => (S::Main { sel: couch as usize }, Action::None),
        (S::Campaign { sel, couch }, Some(Key::Confirm)) => match sel {
            0 => (S::Party { sel: 0, couch }, Action::Phase(c.unlocked.saturating_sub(1) as u8)),
            s if s <= c.unlocked.min(c.phases) => (S::Party { sel: 0, couch }, Action::Phase(s as u8 - 1)),
            s if s == c.phases + 1 => (S::Main { sel: couch as usize }, Action::None),
            _ => (screen, Action::None),
        },
        (S::Campaign { sel, couch }, k) => (S::Campaign { sel: list(sel, c.phases + 2, k), couch }, Action::None),
        (S::Party { couch, .. }, Some(Key::Back)) | (S::Party { sel: 2, couch }, Some(Key::Confirm)) => {
            (S::Campaign { sel: 0, couch }, Action::None)
        }
        (S::Party { sel: 0, .. }, k) if sideways => (screen, side(k).map_or(Action::None, Action::Character)),
        (S::Party { sel: 1, couch }, Some(Key::Confirm)) if !couch || c.players >= 2 => (screen, Action::Begin),
        (S::Party { sel, couch }, k) => (S::Party { sel: list(sel, PARTY_ITEMS, k), couch }, Action::None),
        (S::HowTo, Some(Key::Back | Key::Confirm)) => (S::Main { sel: 3 }, Action::None),
        (S::Credits { .. }, Some(Key::Back)) => (S::Main { sel: 4 }, Action::None),
        (S::Credits { page }, Some(Key::Right | Key::Confirm)) if page + 1 >= ui::PAGES => (S::Main { sel: 4 }, Action::None),
        (S::Credits { page }, Some(Key::Right | Key::Confirm)) => (S::Credits { page: page + 1 }, Action::None),
        (S::Credits { page }, Some(Key::Left)) => (S::Credits { page: page.saturating_sub(1) }, Action::None),
        (S::Online { .. }, Some(Key::Back)) | (S::Online { sel: 2 }, Some(Key::Confirm)) => (S::Main { sel: 2 }, Action::None),
        (S::Online { sel: 0 }, Some(Key::Confirm)) => (S::Create { sel: 0, public: false, relay: 0 }, Action::None),
        (S::Online { sel: 1 }, Some(Key::Confirm)) => (S::Rooms { sel: 0 }, Action::Refresh),
        (S::Online { sel }, k) => (S::Online { sel: list(sel, 3, k) }, Action::None),
        (S::Create { .. }, Some(Key::Back)) | (S::Create { sel: 3, .. }, Some(Key::Confirm)) => (S::Online { sel: 0 }, Action::None),
        (S::Create { sel: 2, public, relay }, Some(Key::Confirm)) => {
            (S::Lobby { since: c.frame, hosting: true, sel: 0 }, Action::Host { public, relay })
        }
        (S::Create { sel: 0, public, relay }, Some(Key::Left | Key::Right | Key::Confirm)) => {
            (S::Create { sel: 0, public: !public, relay }, Action::None)
        }
        (S::Create { sel: 1, public, relay }, k @ Some(Key::Left | Key::Right | Key::Confirm)) => {
            let n = RELAYS.len() as i8;
            let r = (relay as i8 + side(k).unwrap_or(1) + n) % n;
            (S::Create { sel: 1, public, relay: r as usize }, Action::None)
        }
        (S::Create { sel, public, relay }, k) => (S::Create { sel: list(sel, 4, k), public, relay }, Action::None),
        (S::Rooms { .. }, Some(Key::Back)) => (S::Online { sel: 1 }, Action::None),
        (S::Rooms { .. }, Some(Key::Alt)) => (screen, Action::Refresh),
        (S::Rooms { sel }, Some(Key::Confirm)) => match sel.min(c.rooms + 1) {
            s if s < c.rooms => (S::Lobby { since: c.frame, hosting: false, sel: 0 }, Action::Join(s)),
            s if s == c.rooms => (screen, Action::Refresh),
            _ => (S::Online { sel: 1 }, Action::None),
        },
        (S::Rooms { sel }, k) => (S::Rooms { sel: list(sel.min(c.rooms + 1), c.rooms + 2, k) }, Action::None),
        (S::Lobby { hosting, .. }, Some(Key::Back)) => (S::Online { sel: !hosting as usize }, Action::Leave),
        (S::Lobby { hosting, sel, .. }, Some(Key::Confirm)) if sel + 1 == lobby_items(hosting) => {
            (S::Online { sel: !hosting as usize }, Action::Leave)
        }
        (S::Lobby { sel: 0, .. }, k) if sideways => (screen, side(k).map_or(Action::None, Action::Character)),
        (S::Lobby { hosting: true, sel: 1, .. }, k) if sideways => (screen, side(k).map_or(Action::None, Action::PhaseStep)),
        (S::Lobby { hosting: true, sel: 2, .. }, Some(Key::Confirm)) => match c.link == Link::Hosting && c.remote {
            true => (screen, Action::Start),
            false => (screen, Action::None),
        },
        (S::Lobby { hosting: false, sel: 1, .. }, _) if sideways && c.link == Link::Joined => (screen, Action::Ready),
        (S::Lobby { since, hosting, sel }, k) => (S::Lobby { since, hosting, sel: list(sel, lobby_items(hosting), k) }, Action::None),
        (S::Notice(_), Some(Key::Confirm | Key::Back)) => (S::main(), Action::None),
        _ => (screen, Action::None),
    }
}

pub struct Look<'a> {
    pub net: &'a Net,
    pub maps: &'a [&'static Map<'static>],
    pub unlocked: usize,
    pub focus: u8,
    pub t: u32,
}

fn heading(ui: &mut Ui2d, s: &str) {
    centered(ui, SCREEN, 14, s, INK, Serif);
}

fn footer(ui: &mut Ui2d) {
    let w = 16 + width("escolher", Body) + 10 + 16 + width("voltar", Body);
    let x = art::key_hint(ui, (SCREEN[2] as i16 - w) / 2, 202, Button::A, "escolher", DIM);
    art::key_hint(ui, x, 202, Button::B, "voltar", DIM);
}

fn main_menu(ui: &mut Ui2d, sel: usize, lobby: bool, t: u32) {
    art::logo(ui, (SCREEN[2] as i16 - art::LOGO_SIZE[0]) / 2, 2, WHITE);
    let items = [
        Item { note: Some("uma pessoa, fase a fase"), ..Item::new("Campanha") },
        Item { note: Some("dois a quatro controles nesta TV"), ..Item::new("Jogar no sofá") },
        Item { enabled: lobby, note: Some(if lobby { "cada um na sua TV" } else if cfg!(target_os = "emscripten") { "indisponível no navegador" } else { "não disponível nesta caixinha" }), ..Item::new("Jogar online") },
        Item::new("Como jogar"),
        Item::new("Créditos"),
    ];
    ui::menu(ui, 128, 76, &items, sel, t);
    footer(ui);
}

fn campaign(ui: &mut Ui2d, sel: usize, couch: bool, l: &Look) {
    heading(ui, if couch { "Sofá: escolha a fase" } else { "Campanha" });
    let names: [Line; 8] = std::array::from_fn(|i| match l.maps.get(i) {
        Some(m) => line(format_args!("{}. {}", i + 1, m.name)),
        None => Line::new(),
    });
    let notes: [Line; 8] = std::array::from_fn(|i| match l.maps.get(i) {
        Some(m) if i < l.unlocked => line(format_args!("lenda: {}", legend(m.legend).name)),
        _ => line(format_args!("trancada")),
    });
    let last = l.unlocked.clamp(1, l.maps.len().max(1)) - 1;
    let resume = line(format_args!("fase {}: {}", last + 1, l.maps.get(last).map_or("", |m| m.name)));
    let n = l.maps.len().min(names.len());
    let items: [Item; 10] = std::array::from_fn(|i| match i {
        0 => Item { note: Some(resume.as_str()), ..Item::new("Continuar") },
        i if i <= n => Item { enabled: i <= l.unlocked, note: Some(notes[i - 1].as_str()), ..Item::new(names[i - 1].as_str()) },
        _ => Item::new("Voltar"),
    });
    ui::menu(ui, 64, 42, &items[..n + 2], sel, l.t);
    footer(ui);
}

fn seat_nicks(net: &Net, local_names: bool) -> [Line; 4] {
    std::array::from_fn(|i| match net.seats().get(i) {
        Some(s) if local_names => line(format_args!("controle {}", s.local_port.map_or(0, |p| p + 1))),
        Some(s) => line(format_args!("{}", net::name(&s.nick))),
        None => Line::new(),
    })
}

fn party_seats<'a>(net: &Net, nicks: &'a [Line; 4]) -> ([PartySeat<'a>; 4], usize) {
    let seats = std::array::from_fn(|i| {
        let s = net.seats().get(i);
        PartySeat {
            character: s.map_or(0, |s| s.character),
            nick: nicks[i].as_str(),
            here: s.is_some_and(|s| s.local_port.is_some()),
            ready: s.is_some_and(|s| s.ready),
        }
    });
    (seats, net.seats().len().min(4))
}

fn party(ui: &mut Ui2d, sel: usize, couch: bool, l: &Look) {
    let nicks = seat_nicks(l.net, true);
    let (seats, n) = party_seats(l.net, &nicks);
    let phase = l.net.phase() as usize;
    let map = l.maps.get(phase);
    let sub = line(format_args!("fase {} - lenda: {}", phase + 1, map.map_or("", |m| legend(m.legend).name)));
    let mine = l.net.seats().iter().find(|s| s.local_port.is_some()).map_or(0, |s| s.character);
    let enough = !couch || n >= 2;
    let items = [
        Item { value: Some(character(mine).short), ..Item::new("Personagem") },
        Item { enabled: enough, ..Item::new(if enough { "Começar" } else { "Começar (2+)" }) },
        Item::new("Voltar"),
    ];
    let footer = if couch { "outro controle: START entra, B sai" } else { "setas trocam   A escolher   B voltar" };
    let p = Party { head: map.map_or("", |m| m.name), sub: sub.as_str(), seats: &seats[..n], items: &items, sel, focus: l.focus, footer };
    ui::party(ui, &p, l.t);
}

fn online(ui: &mut Ui2d, sel: usize, t: u32) {
    heading(ui, "Jogar online");
    let items = [
        Item { note: Some("seus amigos entram na sua sala"), ..Item::new("Criar sala") },
        Item { note: Some("entrar na sala de alguém"), ..Item::new("Procurar salas") },
        Item::new("Voltar"),
    ];
    ui::menu(ui, 128, 70, &items, sel, t);
    footer(ui);
}

fn create(ui: &mut Ui2d, sel: usize, public: bool, relay: usize, t: u32) {
    heading(ui, "Criar sala");
    let items = [
        Item {
            value: Some(if public { "sim" } else { "não" }),
            note: Some(if public { "aparece na lista de salas" } else { "só entra quem tem o endereço" }),
            ..Item::new("Pública")
        },
        Item { value: Some(RELAYS[relay].0), note: Some("se a outra TV não conseguir entrar"), ..Item::new("Relay") },
        Item::new("Criar sala"),
        Item::new("Voltar"),
    ];
    ui::menu(ui, 96, 64, &items, sel, t);
    footer(ui);
}

fn rooms(ui: &mut Ui2d, sel: usize, net: &Net, t: u32) {
    let rows: [ui::RoomRow; 32] = std::array::from_fn(|i| {
        let r = net.rooms().get(i);
        ui::RoomRow { nick: r.map_or("", |r| net::name(&r.nick)), players: r.map_or(0, |r| r.players), country: r.map_or("", |r| net::name(&r.country)) }
    });
    let n = net.rooms().len().min(rows.len());
    let status = match (n, net.listing()) {
        (_, Listing::Failed) => Some("não foi possível buscar as salas"),
        (0, Listing::Loading | Listing::Idle) => Some("procurando..."),
        (0, Listing::Ready) => Some("nenhuma sala aberta"),
        _ => None,
    };
    ui::rooms(ui, &rows[..n], sel.min(n + 1), status, t);
}

fn lobby(ui: &mut Ui2d, since: u32, hosting: bool, sel: usize, l: &Look) {
    let net = l.net;
    let nicks = seat_nicks(net, false);
    let (seats, n) = party_seats(net, &nicks);
    let remote = net.seats().iter().any(|s| s.local_port.is_none());
    let all = remote && net.seats().iter().all(|s| s.ready || s.local_port.is_some());
    let ready = net.ready();
    let waiting = l.t.wrapping_sub(since) > NO_ANSWER_FRAMES;
    let status = match net.link() {
        Link::Hosting if all => "todos prontos: pode começar",
        Link::Hosting if remote => "esperando todos ficarem prontos",
        Link::Hosting => "esperando outra TV entrar",
        Link::Joined if ready => "pronto. esperando o anfitrião",
        Link::Joined => "marque Pronto quando quiser",
        _ if waiting => "sem resposta",
        _ if hosting => "abrindo a sala...",
        _ => "entrando...",
    };
    let phase = net.phase() as usize;
    let place = line(format_args!("{} de {}", phase + 1, l.maps.len()));
    let mine = net.seats().iter().find(|s| s.local_port.is_some()).map_or(0, |s| s.character);
    let pick = Item { value: Some(character(mine).short), ..Item::new("Personagem") };
    let leave = Item::new("Sair da sala");
    let host_items = [
        pick,
        Item { value: Some(place.as_str()), ..Item::new("Fase") },
        Item { enabled: net.link() == Link::Hosting && all, ..Item::new("Começar") },
        leave,
    ];
    let guest_items = [pick, Item { value: Some(if ready { "sim" } else { "não" }), ..Item::new("Pronto") }, leave];
    let items: &[Item] = if hosting { &host_items } else { &guest_items };
    let head = l.maps.get(phase).map_or("Sala", |m| m.name);
    let footer = if n > 0 { "outro controle nesta TV: START entra" } else { "B sair da sala" };
    let p = Party { head, sub: status, seats: &seats[..n], items, sel, focus: l.focus, footer };
    ui::party(ui, &p, l.t);
}

fn notice(ui: &mut Ui2d, n: Notice) {
    let (head, detail) = match n {
        Notice::Lost => ("Conexão perdida", "a outra TV saiu ou a rede caiu"),
        Notice::Version => ("Versões diferentes", "as TVs têm versões diferentes do doppio"),
        Notice::Full => ("Sala cheia", "não há lugar para todos desta TV"),
        Notice::Started => ("Já começou", "a noite desta sala já começou"),
        Notice::Desync => ("A noite se dividiu", "as TVs viram coisas diferentes"),
    };
    ui::notice(ui, head, detail);
}

pub fn draw(ui: &mut Ui2d, s: Screen, l: &Look) {
    match s {
        Screen::Title | Screen::Game => {}
        Screen::Main { sel } => main_menu(ui, sel, l.net.lobby_supported(), l.t),
        Screen::Campaign { sel, couch } => campaign(ui, sel, couch, l),
        Screen::Party { sel, couch } => party(ui, sel, couch, l),
        Screen::HowTo => ui::howto(ui, l.t),
        Screen::Credits { page } => ui::credits(ui, page),
        Screen::Online { sel } => online(ui, sel, l.t),
        Screen::Create { sel, public, relay } => create(ui, sel, public, relay, l.t),
        Screen::Rooms { sel } => rooms(ui, sel, l.net, l.t),
        Screen::Lobby { since, hosting, sel } => lobby(ui, since, hosting, sel, l),
        Screen::Notice(n) => notice(ui, n),
    }
}

pub fn portraits(s: Screen, net: &Net) -> ([([u16; 4], u8); 4], usize) {
    let mut out = [([0; 4], 0); 4];
    let n = match s {
        Screen::Party { .. } | Screen::Lobby { .. } => {
            let rects = ui::party_rects(net.seats().len());
            net.seats().iter().take(4).zip(out.iter_mut()).zip(rects).for_each(|((seat, o), r)| *o = (r, seat.character));
            net.seats().len().min(4)
        }
        Screen::Credits { page } => {
            ui::legend_portrait(page).iter().for_each(|&p| out[0] = p);
            ui::legend_portrait(page).is_some() as usize
        }
        _ => 0,
    };
    (out, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Ctx {
        Ctx { lobby: true, rooms: 2, link: Link::Offline, players: 1, remote: false, frame: 7, phases: 5, unlocked: 2 }
    }

    #[test]
    fn characters_cycle_past_taken_ones() {
        assert_eq!(next_free(0, 1, 0), 1);
        assert_eq!(next_free(0, -1, 0), 3);
        assert_eq!(next_free(0, 1, 0b0110), 3);
        assert_eq!(next_free(3, 1, 0b0001), 1);
        assert_eq!(next_free(2, 1, 0b1011), 2);
        assert_eq!(next_free(2, -1, 0b0010), 0);
    }

    #[test]
    fn campaign_picks_unlocked_phases_only() {
        let c = ctx();
        let (s, a) = step(Screen::main(), Some(Key::Confirm), &c);
        assert_eq!((s, a), (Screen::Campaign { sel: 0, couch: false }, Action::Solo));
        assert_eq!(step(s, Some(Key::Confirm), &c), (Screen::Party { sel: 0, couch: false }, Action::Phase(1)));
        let fase = |sel| Screen::Campaign { sel, couch: false };
        assert_eq!(step(fase(1), Some(Key::Confirm), &c).1, Action::Phase(0));
        assert_eq!(step(fase(2), Some(Key::Confirm), &c).1, Action::Phase(1));
        assert_eq!(step(fase(3), Some(Key::Confirm), &c), (fase(3), Action::None));
        assert_eq!(step(fase(6), Some(Key::Confirm), &c).0, Screen::Main { sel: 0 });
        assert_eq!(step(fase(6), Some(Key::Down), &c).0, fase(0));
        let p = Screen::Party { sel: 0, couch: false };
        assert_eq!(step(p, Some(Key::Right), &c).1, Action::Character(1));
        assert_eq!(step(p, Some(Key::Left), &c).1, Action::Character(-1));
        assert_eq!(step(p, Some(Key::Confirm), &c).1, Action::Character(1));
        assert_eq!(step(Screen::Party { sel: 1, couch: false }, Some(Key::Confirm), &c).1, Action::Begin);
        assert_eq!(step(Screen::Party { sel: 1, couch: true }, Some(Key::Confirm), &c).1, Action::None);
        let two = Ctx { players: 2, ..ctx() };
        assert_eq!(step(Screen::Party { sel: 1, couch: true }, Some(Key::Confirm), &two).1, Action::Begin);
        assert_eq!(step(Screen::Main { sel: 1 }, Some(Key::Confirm), &c), (Screen::Campaign { sel: 0, couch: true }, Action::Couch));
    }

    #[test]
    fn menus_walk_to_online_lobby_and_back() {
        let c = ctx();
        let (s, _) = step(Screen::Title, Some(Key::Confirm), &c);
        let (s, _) = step(s, Some(Key::Down), &c);
        let (s, _) = step(s, Some(Key::Down), &c);
        assert_eq!(s, Screen::Main { sel: 2 });
        let (s, _) = step(s, Some(Key::Confirm), &c);
        assert_eq!(s, Screen::Online { sel: 0 });
        let (s, _) = step(s, Some(Key::Confirm), &c);
        let (s, _) = step(s, Some(Key::Right), &c);
        let (s, _) = step(s, Some(Key::Down), &c);
        let (s, _) = step(s, Some(Key::Left), &c);
        assert_eq!(s, Screen::Create { sel: 1, public: true, relay: 4 });
        let (s, _) = step(s, Some(Key::Down), &c);
        let (s, a) = step(s, Some(Key::Confirm), &c);
        assert_eq!(a, Action::Host { public: true, relay: 4 });
        assert_eq!(s, Screen::Lobby { since: 7, hosting: true, sel: 0 });
        assert_eq!(step(s, Some(Key::Right), &c).1, Action::Character(1));
        let (f, _) = step(s, Some(Key::Down), &c);
        assert_eq!(step(f, Some(Key::Left), &c).1, Action::PhaseStep(-1));
        let (go, _) = step(f, Some(Key::Down), &c);
        assert_eq!(step(go, Some(Key::Confirm), &c).1, Action::None);
        let hosting = Ctx { link: Link::Hosting, remote: true, ..ctx() };
        assert_eq!(step(go, Some(Key::Confirm), &hosting).1, Action::Start);
        assert_eq!(step(s, Some(Key::Back), &c), (Screen::Online { sel: 0 }, Action::Leave));
    }

    fn reaches_main(from: Screen, c: &Ctx) -> bool {
        let mut seen = vec![from];
        let mut next = 0;
        while next < seen.len() && !matches!(seen[next], Screen::Main { .. }) {
            let s = seen[next];
            next += 1;
            [Key::Up, Key::Down, Key::Left, Key::Right, Key::Confirm].iter().map(|&k| step(s, Some(k), c).0).for_each(|n| {
                if !seen.contains(&n) {
                    seen.push(n);
                }
            });
        }
        next < seen.len()
    }

    fn reachable(from: Screen, c: &Ctx) -> Vec<Screen> {
        let mut seen = vec![from];
        let mut next = 0;
        while next < seen.len() {
            let s = seen[next];
            next += 1;
            [Key::Up, Key::Down, Key::Left, Key::Right, Key::Confirm].iter().map(|&k| step(s, Some(k), c).0).for_each(|n| {
                if !seen.contains(&n) {
                    seen.push(n);
                }
            });
        }
        seen
    }

    #[test]
    fn every_screen_gets_back_with_arrows_and_a() {
        let lobby = |hosting| Screen::Lobby { since: 0, hosting, sel: 0 };
        let screens = [
            Screen::Campaign { sel: 3, couch: true },
            Screen::Party { sel: 0, couch: false },
            Screen::Party { sel: 0, couch: true },
            Screen::HowTo,
            Screen::Credits { page: 0 },
            Screen::Online { sel: 0 },
            Screen::Create { sel: 0, public: false, relay: 0 },
            Screen::Rooms { sel: 0 },
            Screen::Notice(Notice::Lost),
            lobby(true),
            lobby(false),
        ];
        [Link::Offline, Link::Hosting, Link::Joined].iter().for_each(|&link| {
            let c = Ctx { link, remote: true, ..ctx() };
            screens.iter().for_each(|&s| assert!(reaches_main(s, &c), "{s:?} stuck with {link:?}"));
        });
        let joined = Ctx { link: Link::Joined, ..ctx() };
        let (s, _) = step(lobby(false), Some(Key::Down), &joined);
        assert_eq!(step(s, Some(Key::Confirm), &joined).1, Action::Ready);
        let (s, _) = step(s, Some(Key::Down), &joined);
        assert_eq!(step(s, Some(Key::Confirm), &joined), (Screen::Online { sel: 1 }, Action::Leave));
    }

    #[test]
    fn every_menu_screen_is_reachable_from_the_title_with_arrows_and_a() {
        let c = Ctx { players: 2, ..ctx() };
        let all = reachable(Screen::Title, &c);
        let kinds = |s: &Screen| std::mem::discriminant(s);
        let want = [
            Screen::main(),
            Screen::Campaign { sel: 0, couch: false },
            Screen::Party { sel: 0, couch: false },
            Screen::HowTo,
            Screen::Credits { page: 0 },
            Screen::Online { sel: 0 },
            Screen::Create { sel: 0, public: false, relay: 0 },
            Screen::Rooms { sel: 0 },
            Screen::Lobby { since: 0, hosting: true, sel: 0 },
        ];
        want.iter().for_each(|w| assert!(all.iter().any(|s| kinds(s) == kinds(w)), "{w:?} unreachable"));
        assert!(all.contains(&Screen::Credits { page: ui::PAGES - 1 }));
        assert!(all.contains(&Screen::Party { sel: 1, couch: true }));
    }

    #[test]
    fn room_list_includes_refresh_row() {
        let c = ctx();
        assert_eq!(step(Screen::Rooms { sel: 0 }, Some(Key::Confirm), &c).1, Action::Join(0));
        assert_eq!(step(Screen::Rooms { sel: 2 }, Some(Key::Confirm), &c).1, Action::Refresh);
        assert_eq!(step(Screen::Rooms { sel: 3 }, Some(Key::Down), &c).0, Screen::Rooms { sel: 0 });
        assert_eq!(step(Screen::Rooms { sel: 3 }, Some(Key::Confirm), &c).0, Screen::Online { sel: 1 });
        assert_eq!(step(Screen::Rooms { sel: 0 }, Some(Key::Alt), &c).1, Action::Refresh);
        assert_eq!(lost_notice(Some(Reason::Full)), Notice::Full);
    }
}
