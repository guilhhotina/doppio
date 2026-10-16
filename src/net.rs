mod glue;
mod json;
mod lobby;
mod proto;
mod session;

use crate::core::Frontend;
use crate::sim::Setup;
use glue::{Event, Packets};
use lobby::Lobby;
use proto::{Msg, Reason};
use session::{BROADCAST, HOST, Role, Session};

pub use glue::register;

const DEFAULT_NIGHT_SECS: u32 = 360;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Link {
    Offline,
    Hosting,
    Joined,
    Lost,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Room {
    pub nick: [u8; 32],
    pub game: [u8; 32],
    pub players: u8,
    pub host: [u8; 64],
    pub port: u16,
    pub mitm: [u8; 48],
    pub country: [u8; 4],
}

impl Room {
    pub const EMPTY: Room = Room { nick: [0; 32], game: [0; 32], players: 0, host: [0; 64], port: 0, mitm: [0; 48], country: [0; 4] };
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Seat {
    pub nick: [u8; 16],
    pub local_port: Option<u8>,
    pub ready: bool,
}

#[derive(Clone, Copy)]
enum Request {
    Refresh,
    Host { public: bool, relay: [u8; 32] },
    Join(Room),
    Disconnect,
}

pub struct Net {
    lobby_ok: bool,
    lobby: Lobby,
    session: Session,
    request: Option<Request>,
    start_pending: bool,
}

pub fn name(bytes: &[u8]) -> &str {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let s = &bytes[..end];
    std::str::from_utf8(s).unwrap_or_else(|e| std::str::from_utf8(&s[..e.valid_up_to()]).unwrap_or(""))
}

fn fit<const N: usize>(s: &str) -> [u8; N] {
    let end = (0..=s.len().min(N - 1)).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0);
    std::array::from_fn(|i| if i < end { s.as_bytes()[i] } else { 0 })
}

fn night_secs(fe: &Frontend) -> u32 {
    fe.variable(c"doppio_night_seconds")
        .and_then(|v| v.trim().parse().ok())
        .filter(|s| (60..=3600).contains(s))
        .unwrap_or(DEFAULT_NIGHT_SECS)
}

fn fresh_seed() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0x5EED, |d| d.subsec_nanos() ^ (d.as_secs() as u32).rotate_left(13))
}

impl Net {
    pub fn new(fe: &Frontend) -> Net {
        let nick = fe.username().filter(|n| !n.trim().is_empty()).unwrap_or_else(|| "jogador".into());
        Net {
            lobby_ok: lobby::supported(fe),
            lobby: Lobby::new(),
            session: Session::new(fit(nick.trim())),
            request: None,
            start_pending: false,
        }
    }

    pub fn lobby_supported(&self) -> bool {
        self.lobby_ok
    }

    pub fn refresh_rooms(&mut self) {
        if self.lobby_ok {
            self.request = Some(Request::Refresh);
        }
    }

    pub fn rooms(&self) -> &[Room] {
        self.lobby.rooms()
    }

    pub fn host(&mut self, public: bool, relay: Option<&str>) {
        self.drop_session();
        self.request = self.lobby_ok.then(|| Request::Host { public, relay: relay.map_or([0; 32], fit) });
    }

    pub fn join(&mut self, room: usize) {
        if let Some(&r) = self.lobby.rooms().get(room) {
            self.drop_session();
            self.request = Some(Request::Join(r));
        }
    }

    pub fn leave(&mut self) {
        let online = self.session.role() != Role::Offline;
        self.drop_session();
        self.request = (self.lobby_ok && online).then_some(Request::Disconnect);
    }

    fn drop_session(&mut self) {
        let bye = Msg::Leave { reason: Reason::Bye };
        match self.session.role() {
            Role::Host => session::Wire::send(&mut Packets, BROADCAST, &bye),
            Role::Client => session::Wire::send(&mut Packets, HOST, &bye),
            Role::Offline => {}
        }
        self.session.reset(Role::Offline, HOST, &mut ());
        self.start_pending = false;
    }

    pub fn link(&self) -> Link {
        self.session.link()
    }

    pub fn seats(&self) -> &[Seat] {
        self.session.seats()
    }

    pub fn set_local_players(&mut self, ports: &[u8]) {
        self.session.set_local_players(ports, &mut Packets);
    }

    pub fn set_ready(&mut self, ready: bool) {
        self.session.set_ready(ready, &mut Packets);
    }

    pub fn start(&mut self) -> bool {
        let ok = self.session.can_start();
        self.start_pending |= ok;
        ok
    }

    pub fn started(&self) -> Option<Setup> {
        self.session.game()
    }

    pub fn push_local(&mut self, local_inputs: &[u16; 4]) {
        self.session.push_local(local_inputs, &mut Packets);
    }

    pub fn pop_tic(&mut self) -> Option<[u16; 4]> {
        self.session.pop_tic()
    }

    pub fn report_hash(&mut self, tic: u32, hash: u32) {
        self.session.report_hash(tic, hash, &mut Packets);
    }

    pub fn desynced(&self) -> bool {
        self.session.desynced()
    }

    pub fn poll(&mut self, env: &Frontend) {
        self.request.take().iter().for_each(|r| match r {
            Request::Refresh => self.lobby.refresh(env),
            Request::Host { public, relay } => {
                lobby::host(env, *public, relay);
            }
            Request::Join(room) => {
                lobby::join(env, room);
            }
            Request::Disconnect => {
                lobby::disconnect(env);
            }
        });
        if self.lobby_ok {
            self.lobby.poll(env);
        }
        glue::pump();
        std::iter::from_fn(glue::next).for_each(|e| self.event(e));
        if glue::take_overflow() {
            self.session.lose();
        }
        if std::mem::take(&mut self.start_pending) {
            self.session.start(fresh_seed(), night_secs(env), &mut Packets);
        }
        glue::set_open(self.session.accepting());
    }

    fn event(&mut self, e: Event) {
        match e {
            Event::Start { me } => {
                self.start_pending = false;
                self.session.reset(if me == HOST { Role::Host } else { Role::Client }, me, &mut Packets);
            }
            Event::Packet { from, len, data } => {
                Msg::decode(&data[..len as usize]).into_iter().for_each(|m| self.session.receive(from, m, &mut Packets));
            }
            Event::Gone { client } => self.session.gone(client, &mut Packets),
            Event::Stop => self.session.stop(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_fit_and_read_back() {
        let a: [u8; 6] = fit("João!");
        assert_eq!(name(&a), "João");
        let b: [u8; 5] = fit("João");
        assert_eq!(name(&b), "Joã");
        let c: [u8; 4] = fit("João");
        assert_eq!(name(&c), "Jo");
        assert_eq!(name(&[b'a', 0xC3, 0, b'z']), "a");
        assert_eq!(name(b"full"), "full");
    }

    #[test]
    fn offline_session_through_net() {
        let fe = crate::core::frontend();
        let mut net = Net::new(&fe);
        assert!(!net.lobby_supported());
        net.set_local_players(&[0, 1]);
        assert_eq!(net.seats().len(), 2);
        assert!(net.start());
        assert_eq!(net.started(), None);
        net.poll(&fe);
        assert_eq!(net.started().map(|s| (s.players, s.night_secs)), Some((2, DEFAULT_NIGHT_SECS)));
        net.push_local(&[4, 5, 6, 7]);
        assert_eq!(net.pop_tic(), Some([4, 5, 0, 0]));
        assert_eq!(net.link(), Link::Offline);
        net.leave();
        assert_eq!(net.started(), None);
        assert_eq!(net.seats().len(), 2);
    }

    #[test]
    fn steady_state_does_not_allocate() {
        let fe = crate::core::frontend();
        let mut net = Net::new(&fe);
        net.set_local_players(&[0]);
        assert!(net.start());
        net.poll(&fe);
        let allocs = crate::audio::alloc::count(|| {
            (0..300u32).for_each(|t| {
                net.poll(&fe);
                net.push_local(&[t as u16, 0, 0, 0]);
                assert_eq!(net.pop_tic().map(|r| r[0]), Some(t as u16));
                net.report_hash(t, t);
            })
        });
        assert_eq!(allocs, 0);
    }
}
