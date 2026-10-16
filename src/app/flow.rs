use super::pad::{Key, list};
use crate::net::{self, Link, Listing, Net, Reason};
use crate::render::Ui2d;
use crate::ui::{self, DIM, FAINT, GOLD, INK, Item, Line, SCREEN, centered, shadowed};
use std::fmt::Write;

pub const RELAYS: [(&str, Option<&str>); 5] =
    [("nenhum", None), ("São Paulo", Some("saopaulo")), ("Nova York", Some("nyc")), ("Madri", Some("madrid")), ("Singapura", Some("singapore"))];
pub const NIGHTS: [u32; 7] = [60, 120, 180, 240, 360, 480, 600];
const DEFAULT_NIGHT: u32 = 360;
const MAIN_ITEMS: usize = 5;
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
    Couch,
    HowTo,
    Online { sel: usize },
    Create { sel: usize, public: bool, relay: usize },
    Rooms { sel: usize },
    Lobby { since: u32, hosting: bool },
    Notice(Notice),
    Game,
}

impl Screen {
    pub const fn main() -> Screen {
        Screen::Main { sel: 0 }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    None,
    Solo,
    Couch,
    CouchReset,
    Night(i8),
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
}

pub fn night_choice(var: Option<u32>) -> u32 {
    var.filter(|s| (60..=3600).contains(s)).unwrap_or(DEFAULT_NIGHT)
}

pub fn next_night(secs: u32, dir: i8) -> u32 {
    match dir < 0 {
        true => NIGHTS.iter().rev().find(|&&n| n < secs).copied().unwrap_or(NIGHTS[NIGHTS.len() - 1]),
        false => NIGHTS.iter().find(|&&n| n > secs).copied().unwrap_or(NIGHTS[0]),
    }
}

pub fn lost_notice(reason: Option<Reason>) -> Notice {
    match reason {
        Some(Reason::Version) => Notice::Version,
        Some(Reason::Full) => Notice::Full,
        Some(Reason::Started) => Notice::Started,
        Some(Reason::Bye) | None => Notice::Lost,
    }
}

fn side(key: Option<Key>) -> Option<i8> {
    match key {
        Some(Key::Left) => Some(-1),
        Some(Key::Right | Key::Confirm) => Some(1),
        _ => None,
    }
}

pub fn step(screen: Screen, key: Option<Key>, c: &Ctx) -> (Screen, Action) {
    use Screen as S;
    match (screen, key) {
        (S::Title, Some(Key::Confirm)) => (S::main(), Action::None),
        (S::Main { .. }, Some(Key::Back)) => (S::Title, Action::None),
        (S::Main { sel: 3 }, k @ Some(Key::Left | Key::Right | Key::Confirm)) => (screen, side(k).map_or(Action::None, Action::Night)),
        (S::Main { sel }, Some(Key::Confirm)) => match sel {
            0 => (screen, Action::Solo),
            1 => (S::Couch, Action::CouchReset),
            2 if c.lobby => (S::Online { sel: 0 }, Action::None),
            4 => (S::HowTo, Action::None),
            _ => (screen, Action::None),
        },
        (S::Main { sel }, k) => (S::Main { sel: list(sel, MAIN_ITEMS, k) }, Action::None),
        (S::Couch, Some(Key::Confirm)) if c.players >= 2 => (screen, Action::Couch),
        (S::Couch, Some(Key::Back)) => (S::Main { sel: 1 }, Action::CouchReset),
        (S::HowTo, Some(Key::Back | Key::Confirm)) => (S::Main { sel: 4 }, Action::None),
        (S::Online { .. }, Some(Key::Back)) | (S::Online { sel: 2 }, Some(Key::Confirm)) => (S::Main { sel: 2 }, Action::None),
        (S::Online { sel: 0 }, Some(Key::Confirm)) => (S::Create { sel: 0, public: false, relay: 0 }, Action::None),
        (S::Online { sel: 1 }, Some(Key::Confirm)) => (S::Rooms { sel: 0 }, Action::Refresh),
        (S::Online { sel }, k) => (S::Online { sel: list(sel, 3, k) }, Action::None),
        (S::Create { .. }, Some(Key::Back)) | (S::Create { sel: 3, .. }, Some(Key::Confirm)) => (S::Online { sel: 0 }, Action::None),
        (S::Create { sel: 2, public, relay }, Some(Key::Confirm)) => {
            (S::Lobby { since: c.frame, hosting: true }, Action::Host { public, relay })
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
            s if s < c.rooms => (S::Lobby { since: c.frame, hosting: false }, Action::Join(s)),
            s if s == c.rooms => (screen, Action::Refresh),
            _ => (S::Online { sel: 1 }, Action::None),
        },
        (S::Rooms { sel }, k) => (S::Rooms { sel: list(sel.min(c.rooms + 1), c.rooms + 2, k) }, Action::None),
        (S::Lobby { hosting, .. }, Some(Key::Back)) => (S::Online { sel: if hosting { 0 } else { 1 } }, Action::Leave),
        (S::Lobby { .. }, Some(Key::Confirm)) => match c.link {
            Link::Hosting if c.remote => (screen, Action::Start),
            Link::Joined => (screen, Action::Ready),
            _ => (screen, Action::None),
        },
        (S::Notice(_), Some(Key::Confirm | Key::Back)) => (S::main(), Action::None),
        _ => (screen, Action::None),
    }
}

fn minutes(secs: u32) -> Line {
    let mut l = Line::new();
    let _ = match secs % 60 {
        0 => write!(l, "{} min", secs / 60),
        s => write!(l, "{}:{s:02} min", secs / 60),
    };
    l
}

fn heading(ui: &mut Ui2d, s: &str) {
    centered(ui, SCREEN, 14, s, INK, 2);
}

fn footer(ui: &mut Ui2d, s: &str) {
    centered(ui, SCREEN, 200, s, FAINT, 1);
}

fn main_menu(ui: &mut Ui2d, sel: usize, lobby: bool, night_secs: u32, t: u32) {
    centered(ui, SCREEN, 18, "DOPPIO", INK, 3);
    let night = minutes(night_secs);
    let items = [
        Item::new("Jogar sozinho"),
        Item { note: Some("dois a quatro controles nesta TV"), ..Item::new("Jogar no sofá") },
        Item { enabled: lobby, note: (!lobby).then_some("não disponível nesta caixinha"), ..Item::new("Jogar online") },
        Item { value: Some(night.as_str()), note: Some("quanto dura cada noite"), ..Item::new("Noite") },
        Item::new("Como jogar"),
    ];
    ui::menu(ui, 120, 70, &items, sel, t);
    footer(ui, "A escolher   B voltar");
}

fn couch(ui: &mut Ui2d, ports: &[Option<u8>; 4], t: u32) {
    heading(ui, "Jogar no sofá");
    (0..4).for_each(|i| {
        let y = 50 + i as i16 * 16;
        let mut l = Line::new();
        let _ = match ports[i] {
            Some(p) => write!(l, "jogador {}: controle {}", i + 1, p + 1),
            None => write!(l, "jogador {}: aperte START", i + 1),
        };
        let color = if ports[i].is_some() { GOLD } else if t / 30 % 2 == 0 { DIM } else { FAINT };
        shadowed(ui, 104, y, l.as_str(), color, 1);
    });
    centered(ui, SCREEN, 128, "Outro controle: START entra, B sai.", DIM, 1);
    centered(ui, SCREEN, 142, "A tela se divide entre os jogadores.", DIM, 1);
    centered(ui, SCREEN, 162, "Na caixinha da TV só existe um controle:", FAINT, 1);
    centered(ui, SCREEN, 172, "chame os amigos pelo Jogar online.", FAINT, 1);
    footer(ui, "A começar (2 ou mais)   B voltar");
}

fn online(ui: &mut Ui2d, sel: usize, t: u32) {
    heading(ui, "Jogar online");
    let items = [
        Item { note: Some("seus amigos entram na sua casa"), ..Item::new("Criar sala") },
        Item { note: Some("entrar na casa de alguém"), ..Item::new("Procurar salas") },
        Item::new("Voltar"),
    ];
    ui::menu(ui, 128, 70, &items, sel, t);
    footer(ui, "A escolher   B voltar");
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
    footer(ui, "A escolher   B voltar");
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

fn lobby(ui: &mut Ui2d, net: &Net, since: u32, hosting: bool, ready: bool, t: u32) {
    let rows: [ui::SeatRow; 4] = std::array::from_fn(|i| {
        let s = net.seats().get(i);
        ui::SeatRow { nick: s.map_or("", |s| net::name(&s.nick)), here: s.is_some_and(|s| s.local_port.is_some()), ready: s.is_some_and(|s| s.ready) }
    });
    let n = net.seats().len().min(4);
    let remote = net.seats().iter().any(|s| s.local_port.is_none());
    let waiting = t.wrapping_sub(since) > NO_ANSWER_FRAMES;
    let (head, status, help) = match net.link() {
        Link::Hosting if remote && net.seats().iter().all(|s| s.ready || s.local_port.is_some()) => {
            ("Sua sala", "todos prontos: START para começar", "START começar   B fechar a sala")
        }
        Link::Hosting if remote => ("Sua sala", "esperando todos ficarem prontos", "B fechar a sala"),
        Link::Hosting => ("Sua sala", "esperando outra TV entrar", "B fechar a sala"),
        Link::Joined if ready => ("Na sala", "pronto. esperando o anfitrião começar", "A não estou pronto   B sair"),
        Link::Joined => ("Na sala", "aperte A quando estiver pronto", "A pronto   B sair"),
        _ if waiting => ("Sala", "sem resposta. B para voltar", "B voltar"),
        _ if hosting => ("Sala", "abrindo a sala...", "B voltar"),
        _ => ("Sala", "entrando...", "B voltar"),
    };
    ui::lobby(ui, head, &rows[..n], status, help);
    if n > 0 {
        centered(ui, SCREEN, 146, "outro controle nesta TV: START entra", FAINT, 1);
    }
}

fn notice(ui: &mut Ui2d, n: Notice) {
    let (head, detail) = match n {
        Notice::Lost => ("Conexão perdida", "a outra TV saiu ou a rede caiu"),
        Notice::Version => ("Versões diferentes", "as TVs têm versões diferentes do doppio"),
        Notice::Full => ("Sala cheia", "não há lugar para todos desta TV"),
        Notice::Started => ("Já começou", "a noite desta sala já começou"),
        Notice::Desync => ("A casa se dividiu", "as TVs viram noites diferentes"),
    };
    ui::notice(ui, head, detail);
}

pub fn draw(ui: &mut Ui2d, s: Screen, net: &Net, ports: &[Option<u8>; 4], night_secs: u32, ready: bool, t: u32) {
    match s {
        Screen::Title | Screen::Game => {}
        Screen::Main { sel } => main_menu(ui, sel, net.lobby_supported(), night_secs, t),
        Screen::Couch => couch(ui, ports, t),
        Screen::HowTo => ui::howto(ui),
        Screen::Online { sel } => online(ui, sel, t),
        Screen::Create { sel, public, relay } => create(ui, sel, public, relay, t),
        Screen::Rooms { sel } => rooms(ui, sel, net, t),
        Screen::Lobby { since, hosting } => lobby(ui, net, since, hosting, ready, t),
        Screen::Notice(n) => notice(ui, n),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Ctx {
        Ctx { lobby: true, rooms: 2, link: Link::Offline, players: 1, remote: false, frame: 7 }
    }

    #[test]
    fn night_choices_cycle() {
        assert_eq!(night_choice(None), 360);
        assert_eq!(night_choice(Some(60)), 60);
        assert_eq!(night_choice(Some(5)), 360);
        assert_eq!(next_night(360, 1), 480);
        assert_eq!(next_night(600, 1), 60);
        assert_eq!(next_night(60, -1), 600);
        assert_eq!(next_night(100, -1), 60);
        assert_eq!(minutes(90).as_str(), "1:30 min");
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
        assert_eq!(s, Screen::Lobby { since: 7, hosting: true });
        assert_eq!(step(s, Some(Key::Confirm), &c).1, Action::None);
        let hosting = Ctx { link: Link::Hosting, remote: true, ..ctx() };
        assert_eq!(step(s, Some(Key::Confirm), &hosting).1, Action::Start);
        assert_eq!(step(s, Some(Key::Back), &c), (Screen::Online { sel: 0 }, Action::Leave));
    }

    #[test]
    fn online_needs_lobby_and_couch_needs_two() {
        let off = Ctx { lobby: false, ..ctx() };
        assert_eq!(step(Screen::Main { sel: 2 }, Some(Key::Confirm), &off).0, Screen::Main { sel: 2 });
        assert_eq!(step(Screen::Couch, Some(Key::Confirm), &ctx()).1, Action::None);
        let two = Ctx { players: 2, ..ctx() };
        assert_eq!(step(Screen::Couch, Some(Key::Confirm), &two).1, Action::Couch);
        assert_eq!(step(Screen::Main { sel: 0 }, Some(Key::Confirm), &ctx()).1, Action::Solo);
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
