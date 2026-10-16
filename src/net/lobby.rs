use super::Room;
use super::json;
use crate::core::Frontend;
use crate::ffi::*;
use std::ffi::c_char;
use std::ptr::null;

pub const MAX_ROOMS: usize = 32;
const CORE_NAME: &str = "doppio";
const RELAYED: i64 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Listing {
    Idle,
    Loading,
    Ready,
    Failed,
}

pub struct Lobby {
    rooms: [Room; MAX_ROOMS],
    count: usize,
    listing: Listing,
}

fn field<const N: usize>(fields: &[u8], key: &str) -> [u8; N] {
    let mut out = [0; N];
    json::get(fields, key).iter().for_each(|v| {
        json::text(v, &mut out);
    });
    out
}

fn int(fields: &[u8], key: &str) -> i64 {
    json::get(fields, key).and_then(json::number).unwrap_or(0)
}

fn room(fields: &[u8]) -> Option<Room> {
    let core: [u8; 32] = field(fields, "core_name");
    let session: [u8; 48] = field(fields, "mitm_session");
    let relayed = session[0] != 0 && int(fields, "host_method") == RELAYED;
    let (host_key, port_key) = if relayed { ("mitm_ip", "mitm_port") } else { ("ip", "port") };
    let host: [u8; 64] = field(fields, host_key);
    let port = u16::try_from(int(fields, port_key)).unwrap_or(0);
    let locked = json::get(fields, "has_password").is_some_and(json::truthy);
    let ours = super::name(&core).eq_ignore_ascii_case(CORE_NAME);
    (ours && !locked && host[0] != 0 && port != 0).then(|| Room {
        nick: field(fields, "username"),
        game: field(fields, "game_name"),
        players: int(fields, "player_count").clamp(0, 255) as u8,
        host,
        port,
        mitm: if relayed { session } else { [0; 48] },
        country: field(fields, "country"),
    })
}

pub fn parse(js: &[u8], out: &mut [Room; MAX_ROOMS]) -> usize {
    json::elements(js)
        .filter_map(|e| json::get(e, "fields"))
        .filter_map(room)
        .zip(out.iter_mut())
        .fold(0, |n, (r, slot)| {
            *slot = r;
            n + 1
        })
}

impl Lobby {
    pub const fn new() -> Lobby {
        Lobby { rooms: [Room::EMPTY; MAX_ROOMS], count: 0, listing: Listing::Idle }
    }

    pub fn rooms(&self) -> &[Room] {
        &self.rooms[..self.count]
    }

    pub fn listing(&self) -> Listing {
        self.listing
    }

    pub fn refresh(&mut self, fe: &Frontend) {
        self.ask(fe, true);
    }

    pub fn poll(&mut self, fe: &Frontend) {
        if self.listing == Listing::Loading {
            self.ask(fe, false);
        }
    }

    fn ask(&mut self, fe: &Frontend, refresh: bool) {
        let mut lobby = GecndLobby { refresh, state: GECND_LOBBY_NONE, json: null(), size: 0 };
        let answered = fe.env(GECND_NETPLAY_GET_LOBBY, &mut lobby);
        self.listing = match (answered, lobby.state) {
            (false, _) | (true, GECND_LOBBY_FAILED) => Listing::Failed,
            (true, GECND_LOBBY_LOADING) => Listing::Loading,
            (true, GECND_LOBBY_READY) => {
                let js = match lobby.json.is_null() {
                    true => &[][..],
                    false => unsafe { std::slice::from_raw_parts(lobby.json.cast::<u8>(), lobby.size) },
                };
                self.count = parse(js, &mut self.rooms);
                Listing::Ready
            }
            _ => Listing::Idle,
        };
    }
}

pub fn supported(fe: &Frontend) -> bool {
    fe.probe(GECND_NETPLAY_GET_LOBBY)
}

pub fn host(fe: &Frontend, public: bool, relay: &[u8]) -> bool {
    let mut room = GecndRoom {
        game_name: c"doppio".as_ptr(),
        port: 0,
        listed: public,
        relay: if relay.first().is_some_and(|&b| b != 0) { relay.as_ptr().cast::<c_char>() } else { null() },
    };
    fe.env(GECND_NETPLAY_POST_LOBBY, &mut room)
}

pub fn join(fe: &Frontend, room: &Room) -> bool {
    let mut join =
        GecndJoin { host: room.host.as_ptr().cast(), port: room.port, mitm_session: room.mitm.as_ptr().cast() };
    fe.env(GECND_NETPLAY_CONNECT, &mut join)
}

pub fn disconnect(fe: &Frontend) -> bool {
    fe.probe(GECND_NETPLAY_DISCONNECT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::name;

    #[test]
    fn parses_captured_lobby() {
        let mut rooms = [Room::EMPTY; MAX_ROOMS];
        let n = parse(include_bytes!("lobby_sample.json"), &mut rooms);
        assert_eq!(n, 2);
        let relayed = &rooms[0];
        assert_eq!(name(&relayed.nick), "João & Zé");
        assert_eq!(name(&relayed.game), "doppio");
        assert_eq!(relayed.players, 2);
        assert_eq!(name(&relayed.host), "southamerica-east1.relay.retroarch.com");
        assert_eq!(relayed.port, 55435);
        assert_eq!(name(&relayed.mitm), "wqTCdWSIghj/IgbV");
        assert_eq!(name(&relayed.country), "br");
        let direct = &rooms[1];
        assert_eq!(name(&direct.host), "192.0.2.50");
        assert_eq!(direct.port, 55435);
        assert_eq!(direct.players, 0);
        assert_eq!(direct.mitm, [0; 48]);
        assert!(rooms[2..].iter().all(|r| r.port == 0));
    }

    #[test]
    fn caps_at_table_size() {
        let one = r#"{"fields":{"core_name":"Doppio","ip":"1.2.3.4","port":5,"username":"x"}}"#;
        let js = format!("[{}]", vec![one; 40].join(","));
        let mut rooms = [Room::EMPTY; MAX_ROOMS];
        assert_eq!(parse(js.as_bytes(), &mut rooms), MAX_ROOMS);
        assert_eq!(name(&rooms[31].host), "1.2.3.4");
    }
}
