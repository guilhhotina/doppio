use super::proto::*;
use super::{Link, Seat};
use crate::sim::Setup;

pub const DELAY: u32 = 3;
pub const HASH_EVERY: u32 = 30;
pub const HOST: u16 = 0;
pub const BROADCAST: u16 = crate::ffi::NETPACKET_BROADCAST;
const RING: usize = 32;
const HASHES: usize = 8;
const PENDING: usize = 16;
const NO_HASH: (u32, u32) = (u32::MAX, 0);
const ALL: u8 = (1 << SLOTS) - 1;

pub trait Wire {
    fn send(&mut self, to: u16, msg: &Msg);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Offline,
    Host,
    Client,
}

#[derive(Clone, Copy)]
struct Machine {
    client: u16,
    nick: [u8; NICK],
    count: u8,
    ready: bool,
}

const NO_MACHINE: Machine = Machine { client: u16::MAX, nick: [0; NICK], count: 0, ready: false };
const NO_SEAT: Seat = Seat { nick: [0; NICK], local_port: None, ready: false };

pub struct Session {
    role: Role,
    me: u16,
    lost: bool,
    nick: [u8; NICK],
    ports: [u8; SLOTS],
    port_count: u8,
    ready: bool,
    machines: [Machine; SLOTS],
    machine_count: usize,
    seats: [Seat; SLOTS],
    owners: [Owner; SLOTS],
    seat_count: usize,
    game: Option<Setup>,
    delay: u32,
    gone: u8,
    local_tic: u32,
    consumed: u32,
    complete: u32,
    rows: [[u16; SLOTS]; RING],
    have: [u8; RING],
    own_hashes: [(u32, u32); HASHES],
    remote_hashes: [(u32, u32); PENDING],
    desynced: bool,
    reason: Option<Reason>,
}

fn ring(tic: u32) -> usize {
    tic as usize % RING
}

impl Session {
    pub fn new(nick: [u8; NICK]) -> Session {
        let mut s = Session {
            role: Role::Offline,
            me: HOST,
            lost: false,
            nick,
            ports: [0, 1, 2, 3],
            port_count: 1,
            ready: false,
            machines: [NO_MACHINE; SLOTS],
            machine_count: 1,
            seats: [NO_SEAT; SLOTS],
            owners: [NO_OWNER; SLOTS],
            seat_count: 0,
            game: None,
            delay: 0,
            gone: ALL,
            local_tic: 0,
            consumed: 0,
            complete: 0,
            rows: [[0; SLOTS]; RING],
            have: [0; RING],
            own_hashes: [NO_HASH; HASHES],
            remote_hashes: [NO_HASH; PENDING],
            desynced: false,
            reason: None,
        };
        s.publish(&mut ());
        s
    }

    pub fn reset(&mut self, role: Role, me: u16, w: &mut impl Wire) {
        let (nick, ports, port_count) = (self.nick, self.ports, self.port_count);
        *self = Session { ports, port_count, ..Session::new(nick) };
        self.role = role;
        self.me = me;
        match role {
            Role::Client => {
                self.machine_count = 0;
                self.seat_count = 0;
                self.owners = [NO_OWNER; SLOTS];
                self.hello(w);
            }
            _ => self.publish(w),
        }
    }

    pub fn link(&self) -> Link {
        match (self.lost, self.role) {
            (true, _) => Link::Lost,
            (false, Role::Offline) => Link::Offline,
            (false, Role::Host) => Link::Hosting,
            (false, Role::Client) => Link::Joined,
        }
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn seats(&self) -> &[Seat] {
        &self.seats[..self.seat_count]
    }

    pub fn game(&self) -> Option<Setup> {
        self.game
    }

    pub fn desynced(&self) -> bool {
        self.desynced
    }

    pub fn reason(&self) -> Option<Reason> {
        self.reason
    }

    pub fn accepting(&self) -> bool {
        self.role == Role::Host && self.game.is_none() && !self.lost
    }

    pub fn lose(&mut self) {
        self.lost |= self.role != Role::Offline;
    }

    pub fn keepalive(&mut self, w: &mut impl Wire) {
        if self.role == Role::Host && self.game.is_none() && !self.lost {
            self.publish(w);
        }
    }

    pub fn set_nick(&mut self, nick: [u8; NICK]) {
        self.nick = nick;
    }

    fn hello(&self, w: &mut impl Wire) {
        w.send(HOST, &Msg::Hello { version: VERSION, nick: self.nick, local_count: self.port_count });
    }

    fn roster(&self) -> (u8, [RosterSeat; SLOTS]) {
        let mut it = self.machines[..self.machine_count].iter().flat_map(|m| {
            (0..m.count).map(move |index| RosterSeat { owner: Owner { client: m.client, index }, ready: m.ready, nick: m.nick })
        });
        let seats: [RosterSeat; SLOTS] = std::array::from_fn(|_| it.next().unwrap_or(EMPTY_SEAT));
        (seats.iter().filter(|s| s.owner != NO_OWNER).count() as u8, seats)
    }

    fn local_port(&self, owner: Owner) -> Option<u8> {
        (owner.client == self.me && owner.index < self.port_count).then(|| self.ports[owner.index as usize])
    }

    fn apply_roster(&mut self, count: u8, seats: &[RosterSeat; SLOTS]) {
        self.seat_count = count as usize;
        self.owners = seats.map(|s| s.owner);
        self.seats = seats.map(|s| Seat { nick: s.nick, local_port: self.local_port(s.owner), ready: s.ready });
    }

    fn others(&self, except: u16) -> u8 {
        self.machines[1..self.machine_count].iter().filter(|m| m.client != except).map(|m| m.count).sum()
    }

    fn publish(&mut self, w: &mut impl Wire) {
        let count = self.port_count.min(SLOTS as u8 - self.others(HOST));
        self.machines[0] = Machine { client: self.me, nick: self.nick, count, ready: self.ready };
        let (count, seats) = self.roster();
        self.apply_roster(count, &seats);
        if self.role == Role::Host {
            w.send(BROADCAST, &Msg::Roster { count, seats });
        }
    }

    pub fn set_local_players(&mut self, ports: &[u8], w: &mut impl Wire) {
        if self.game.is_some() {
            return;
        }
        let mut valid = ports.iter().copied().filter(|&p| (p as usize) < SLOTS);
        self.ports = std::array::from_fn(|_| valid.next().unwrap_or(0));
        self.port_count = ports.iter().filter(|&&p| (p as usize) < SLOTS).count().min(SLOTS) as u8;
        match self.role {
            Role::Client => {
                self.seats = std::array::from_fn(|i| Seat { local_port: self.local_port(self.owners[i]), ..self.seats[i] });
                self.hello(w);
            }
            _ => self.publish(w),
        }
    }

    pub fn set_ready(&mut self, ready: bool, w: &mut impl Wire) {
        if self.game.is_some() {
            return;
        }
        self.ready = ready;
        match self.role {
            Role::Client => w.send(HOST, &Msg::Ready { ready }),
            _ => self.publish(w),
        }
    }

    pub fn can_start(&self) -> bool {
        self.role != Role::Client
            && self.game.is_none()
            && !self.lost
            && self.seat_count > 0
            && self.machines[1..self.machine_count].iter().all(|m| m.count == 0 || m.ready)
    }

    pub fn start(&mut self, seed: u32, night_secs: u32, w: &mut impl Wire) -> bool {
        let ok = self.can_start();
        if ok {
            let game = Setup { seed, players: self.seat_count as u8, night_secs };
            if self.role == Role::Host {
                w.send(BROADCAST, &Msg::Start { seed, players: game.players, night_secs, slots: self.owners });
            }
            self.begin(game, if self.role == Role::Offline { 0 } else { DELAY });
        }
        ok
    }

    fn begin(&mut self, game: Setup, delay: u32) {
        self.game = Some(game);
        self.delay = delay;
        self.gone = ALL & !((1u8 << game.players) - 1);
        self.local_tic = delay;
        self.consumed = 0;
        self.complete = delay;
        self.rows = [[0; SLOTS]; RING];
        self.have = [0; RING];
    }

    pub fn push_local(&mut self, inputs: &[u16; SLOTS], w: &mut impl Wire) {
        if self.game.is_none() || self.lost || self.local_tic > self.consumed + self.delay {
            return;
        }
        let tic = self.local_tic;
        let mine: [Option<u16>; SLOTS] = std::array::from_fn(|s| {
            (self.owners[s].client == self.me && self.gone & 1 << s == 0)
                .then(|| self.local_port(self.owners[s]).map_or(0, |p| inputs[p as usize]))
        });
        mine.iter().enumerate().filter_map(|(s, b)| b.map(|b| (s, b))).for_each(|(slot, bits)| match self.role {
            Role::Client => w.send(HOST, &Msg::Input { tic, slot: slot as u8, bits }),
            _ => self.store(tic, slot, bits),
        });
        self.local_tic += 1;
        self.emit(w);
    }

    fn store(&mut self, tic: u32, slot: usize, bits: u16) {
        self.rows[ring(tic)][slot] = bits;
        self.have[ring(tic)] |= 1 << slot;
    }

    fn in_window(&self, tic: u32) -> bool {
        tic >= self.complete && tic < self.consumed + RING as u32
    }

    fn emit(&mut self, w: &mut impl Wire) {
        if self.role == Role::Client {
            return;
        }
        let need = ALL & !self.gone;
        while self.in_window(self.complete) && self.have[ring(self.complete)] & need == need {
            let i = ring(self.complete);
            self.rows[i] = std::array::from_fn(|s| if need & 1 << s != 0 { self.rows[i][s] } else { 0 });
            if self.role == Role::Host {
                w.send(BROADCAST, &Msg::Tic { tic: self.complete, inputs: self.rows[i] });
            }
            self.complete += 1;
        }
    }

    pub fn pop_tic(&mut self) -> Option<[u16; SLOTS]> {
        (self.game.is_some() && self.consumed < self.complete).then(|| {
            let i = ring(self.consumed);
            let row = self.rows[i];
            self.rows[i] = [0; SLOTS];
            self.have[i] = 0;
            self.consumed += 1;
            row
        })
    }

    pub fn report_hash(&mut self, tic: u32, hash: u32, w: &mut impl Wire) {
        if self.role == Role::Offline || self.game.is_none() || tic % HASH_EVERY != 0 {
            return;
        }
        self.own_hashes[(tic / HASH_EVERY) as usize % HASHES] = (tic, hash);
        w.send(if self.role == Role::Host { BROADCAST } else { HOST }, &Msg::Hash { tic, hash });
        let mismatch = self.remote_hashes.iter().any(|&(t, h)| t == tic && h != hash);
        self.remote_hashes.iter_mut().filter(|(t, _)| *t == tic).for_each(|e| *e = NO_HASH);
        self.desynced |= mismatch;
    }

    fn remote_hash(&mut self, tic: u32, hash: u32) {
        match self.own_hashes[(tic / HASH_EVERY) as usize % HASHES] {
            (t, h) if t == tic => self.desynced |= h != hash,
            _ => {
                let free = self.remote_hashes.iter_mut().find(|(t, _)| *t == NO_HASH.0);
                free.into_iter().for_each(|e| *e = (tic, hash));
            }
        }
    }

    pub fn gone(&mut self, client: u16, w: &mut impl Wire) {
        match (self.role, self.game) {
            (Role::Client, _) if client == HOST => self.lose(),
            (Role::Host, None) if client != HOST => {
                let mut kept = self.machines[..self.machine_count].iter().copied().filter(|m| m.client != client);
                self.machines = std::array::from_fn(|_| kept.next().unwrap_or(NO_MACHINE));
                self.machine_count = self.machines.iter().filter(|m| m.client != NO_MACHINE.client).count();
                self.publish(w);
            }
            (Role::Host, Some(_)) => {
                self.gone |= (0..SLOTS).filter(|&s| self.owners[s].client == client).fold(0, |m, s| m | 1 << s);
                self.emit(w);
            }
            _ => {}
        }
    }

    pub fn stop(&mut self) {
        self.lose();
    }

    fn host_hello(&mut self, from: u16, version: [u8; 16], nick: [u8; NICK], count: u8, w: &mut impl Wire) {
        let known = self.machines[1..self.machine_count].iter().position(|m| m.client == from).map(|i| i + 1);
        let count = count.min(SLOTS as u8);
        let fits = self.others(from) + count + self.port_count <= SLOTS as u8
            && (known.is_some() || self.machine_count < SLOTS);
        let refusal = match (version == VERSION, self.game.is_none(), fits) {
            (false, _, _) => Some(Reason::Version),
            (true, false, _) => Some(Reason::Started),
            (true, true, false) => Some(Reason::Full),
            (true, true, true) => None,
        };
        match (refusal, known) {
            (Some(reason), _) => {
                w.send(from, &Msg::Leave { reason });
                if known.is_some() && self.game.is_none() {
                    self.gone(from, w);
                }
            }
            (None, Some(i)) => {
                self.machines[i] = Machine { nick, count, ..self.machines[i] };
                self.publish(w);
            }
            (None, None) => {
                self.machines[self.machine_count] = Machine { client: from, nick, count, ready: false };
                self.machine_count += 1;
                self.publish(w);
            }
        }
    }

    fn host_ready(&mut self, from: u16, ready: bool, w: &mut impl Wire) {
        let i = self.machines[1..self.machine_count].iter().position(|m| m.client == from).map(|i| i + 1);
        if let (Some(i), None) = (i, self.game) {
            self.machines[i].ready = ready;
            self.publish(w);
        }
    }

    fn host_input(&mut self, from: u16, tic: u32, slot: u8, bits: u16, w: &mut impl Wire) {
        let slot = slot as usize;
        if self.game.is_none() || self.owners[slot].client != from || self.gone & 1 << slot != 0 {
            return;
        }
        match self.in_window(tic) {
            true => {
                self.store(tic, slot, bits);
                self.emit(w);
            }
            false => self.lose(),
        }
    }

    fn client_start(&mut self, seed: u32, players: u8, night_secs: u32, slots: [Owner; SLOTS]) {
        if self.game.is_some() {
            return self.lose();
        }
        let seats = std::array::from_fn(|i| {
            let nick = (0..self.seat_count).find(|&k| self.owners[k] == slots[i]).map_or([0; NICK], |k| self.seats[k].nick);
            RosterSeat { owner: slots[i], ready: true, nick }
        });
        self.apply_roster(players, &seats);
        self.begin(Setup { seed, players, night_secs }, DELAY);
    }

    fn client_tic(&mut self, tic: u32, inputs: [u16; SLOTS]) {
        match self.game.is_some() && tic == self.complete && self.in_window(tic) {
            true => {
                self.rows[ring(tic)] = inputs;
                self.complete += 1;
            }
            false => self.lose(),
        }
    }

    pub fn receive(&mut self, from: u16, msg: Msg, w: &mut impl Wire) {
        match (self.role, msg) {
            (Role::Host, Msg::Hello { version, nick, local_count }) => self.host_hello(from, version, nick, local_count, w),
            (Role::Host, Msg::Ready { ready }) => self.host_ready(from, ready, w),
            (Role::Host, Msg::Input { tic, slot, bits }) => self.host_input(from, tic, slot, bits, w),
            (Role::Host, Msg::Leave { .. }) => self.gone(from, w),
            (Role::Host, Msg::Hash { tic, hash }) => self.remote_hash(tic, hash),
            (Role::Client, _) if from != HOST => {}
            (Role::Client, Msg::Roster { count, seats }) if self.game.is_none() => self.apply_roster(count, &seats),
            (Role::Client, Msg::Start { seed, players, night_secs, slots }) => self.client_start(seed, players, night_secs, slots),
            (Role::Client, Msg::Tic { tic, inputs }) => self.client_tic(tic, inputs),
            (Role::Client, Msg::Hash { tic, hash }) => self.remote_hash(tic, hash),
            (Role::Client, Msg::Leave { reason }) => {
                self.reason = Some(reason);
                self.lose();
            }
            _ => {}
        }
    }
}

impl Wire for () {
    fn send(&mut self, _to: u16, _msg: &Msg) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::Rng;

    fn nick(s: &str) -> [u8; NICK] {
        std::array::from_fn(|i| s.as_bytes().get(i).copied().unwrap_or(0))
    }

    #[derive(Default)]
    struct Outbox(Vec<(u16, Vec<u8>)>);

    impl Wire for Outbox {
        fn send(&mut self, to: u16, msg: &Msg) {
            let mut buf = [0; MAX_PACKET];
            let n = msg.encode(&mut buf);
            self.0.push((to, buf[..n].to_vec()));
        }
    }

    struct Peer {
        id: u16,
        s: Session,
        out: Outbox,
        rows: Vec<[u16; SLOTS]>,
        pushed: Vec<[u16; SLOTS]>,
        world: u32,
        alive: bool,
    }

    impl Peer {
        fn new(id: u16, name: &str, ports: &[u8]) -> Peer {
            let mut p = Peer {
                id,
                s: Session::new(nick(name)),
                out: Outbox::default(),
                rows: vec![],
                pushed: vec![],
                world: 0,
                alive: true,
            };
            p.s.set_local_players(ports, &mut ());
            p.s.reset(if id == HOST { Role::Host } else { Role::Client }, id, &mut p.out);
            p
        }

        fn tics(&mut self, rng: &mut Rng) {
            (0..rng.below(3)).try_for_each(|_| {
                let inputs = std::array::from_fn(|_| rng.below(1 << 10) as u16);
                let before = self.s.local_tic;
                self.s.push_local(&inputs, &mut self.out);
                if self.s.local_tic > before {
                    self.pushed.push(inputs);
                }
                let row = self.s.pop_tic()?;
                let tic = self.rows.len() as u32;
                self.world = row.iter().fold(self.world.wrapping_mul(31).wrapping_add(tic), |h, &b| h.rotate_left(5) ^ b as u32);
                self.rows.push(row);
                self.s.report_hash(tic, self.world, &mut self.out);
                Some(())
            });
        }
    }

    struct Fake {
        now: u64,
        seq: u64,
        latency: u32,
        last: Vec<((u16, u16), u64)>,
        flight: Vec<(u64, u64, u16, u16, Vec<u8>)>,
    }

    impl Fake {
        fn new(latency: u32) -> Fake {
            Fake { now: 0, seq: 0, latency, last: vec![], flight: vec![] }
        }

        fn post(&mut self, rng: &mut Rng, from: u16, to: u16, bytes: Vec<u8>) {
            let floor = self.last.iter().find(|(k, _)| *k == (from, to)).map_or(0, |e| e.1);
            let at = (self.now + 1 + rng.below(self.latency) as u64).max(floor);
            self.last.retain(|(k, _)| *k != (from, to));
            self.last.push(((from, to), at));
            self.seq += 1;
            self.flight.push((at, self.seq, from, to, bytes));
        }

        fn route(&mut self, rng: &mut Rng, peers: &mut [Peer]) {
            let ids: Vec<u16> = peers.iter().filter(|p| p.alive).map(|p| p.id).collect();
            peers.iter_mut().for_each(|p| {
                let sent = std::mem::take(&mut p.out.0);
                let from = p.id;
                sent.into_iter()
                    .filter(|_| p.alive)
                    .flat_map(|(to, b)| {
                        let targets: Vec<u16> = match to {
                            BROADCAST => ids.iter().copied().filter(|&i| i != from).collect(),
                            t => vec![t],
                        };
                        targets.into_iter().map(move |t| (t, b.clone()))
                    })
                    .for_each(|(to, b)| self.post(rng, from, to, b));
            });
        }

        fn deliver(&mut self, peers: &mut [Peer]) {
            self.flight.sort_by_key(|f| (f.0, f.1));
            let now = self.now;
            let due: Vec<_> = self.flight.iter().filter(|f| f.0 <= now).cloned().collect();
            self.flight.retain(|f| f.0 > now);
            due.into_iter().for_each(|(_, _, from, to, bytes)| {
                let alive_from = peers.iter().any(|p| p.id == from && p.alive);
                peers.iter_mut().filter(|p| p.id == to && p.alive && alive_from).for_each(|p| {
                    p.s.receive(from, Msg::decode(&bytes).expect("decodes"), &mut p.out);
                });
            });
        }

        fn frame(&mut self, rng: &mut Rng, peers: &mut [Peer]) {
            self.now += 1;
            self.deliver(peers);
            peers.iter_mut().filter(|p| p.alive).for_each(|p| p.tics(rng));
            self.route(rng, peers);
        }
    }

    fn lobby(rng: &mut Rng, net: &mut Fake) -> Vec<Peer> {
        let mut peers = vec![Peer::new(0, "host", &[0]), Peer::new(1, "duo", &[0, 1])];
        (0..30).for_each(|_| net.frame(rng, &mut peers));
        assert_eq!(peers[1].s.seats().len(), 3);
        peers.push(Peer::new(2, "solo", &[2]));
        (0..30).for_each(|_| net.frame(rng, &mut peers));
        assert!(!peers[0].s.can_start());
        assert_eq!(peers[1].s.seats().len(), 4);
        [1, 2].iter().for_each(|&i| {
            let p = &mut peers[i];
            p.s.set_ready(true, &mut p.out);
        });
        (0..30).for_each(|_| net.frame(rng, &mut peers));
        let seats = peers[1].s.seats();
        assert_eq!(seats.iter().map(|s| s.local_port).collect::<Vec<_>>(), [None, Some(0), Some(1), None]);
        assert!(seats.iter().all(|s| s.ready || s.local_port.is_none()));
        assert!(peers[0].s.accepting());
        let host = &mut peers[0];
        assert!(host.s.start(77, 360, &mut host.out));
        assert!(!peers[0].s.accepting());
        peers
    }

    fn agree(a: &Peer, b: &Peer) -> usize {
        let n = a.rows.len().min(b.rows.len());
        assert_eq!(a.rows[..n], b.rows[..n]);
        n
    }

    fn slot_inputs(p: &Peer, slot: usize) -> Vec<u16> {
        p.rows.iter().skip(DELAY as usize).map(|r| r[slot]).collect()
    }

    #[test]
    fn host_and_two_clients_agree() {
        [(1234, 1), (7, 6), (42, 12)].iter().for_each(|&(seed, latency)| {
            let mut rng = Rng::new(seed);
            let mut net = Fake::new(latency);
            let mut peers = lobby(&mut rng, &mut net);
            (0..2000).for_each(|_| net.frame(&mut rng, &mut peers));
            let game = peers[0].s.game();
            assert!(peers.iter().all(|p| p.s.game() == game && p.s.link() != Link::Lost && !p.s.desynced()));
            assert_eq!(game.map(|g| g.players), Some(4));
            let n = agree(&peers[0], &peers[1]).min(agree(&peers[0], &peers[2]));
            assert!(n > 300, "only {n} tics at latency {latency}");
            assert!(peers[0].rows[..DELAY as usize].iter().all(|r| *r == [0; SLOTS]));
            [(0, 0, 0), (1, 1, 0), (2, 1, 1), (3, 2, 2)].iter().for_each(|&(slot, peer, port)| {
                let got = slot_inputs(&peers[0], slot);
                let sent: Vec<u16> = peers[peer].pushed.iter().map(|i| i[port]).take(got.len()).collect();
                assert_eq!(got[..sent.len()], sent[..]);
                assert!(sent.len() + 4 > got.len());
            });
            let ahead = peers.iter().map(|p| p.pushed.len() as i64 + DELAY as i64 - p.rows.len() as i64).max();
            assert!(ahead.is_some_and(|a| a <= DELAY as i64 + 1));
        });
    }

    #[test]
    fn disconnect_freezes_slot_and_game_goes_on() {
        let mut rng = Rng::new(99);
        let mut net = Fake::new(4);
        let mut peers = lobby(&mut rng, &mut net);
        (0..400).for_each(|_| net.frame(&mut rng, &mut peers));
        peers[2].alive = false;
        let at = peers[0].rows.len();
        let host = &mut peers[0];
        host.s.gone(2, &mut host.out);
        (0..800).for_each(|_| net.frame(&mut rng, &mut peers));
        let n = agree(&peers[0], &peers[1]);
        assert!(n > at + 300, "stalled at {n} from {at}");
        let tail = &peers[0].rows[at + DELAY as usize + 2..n];
        assert!(tail.iter().all(|r| r[3] == 0));
        assert!(tail.iter().any(|r| r[1] != 0));
        assert!(!peers[0].s.desynced() && !peers[1].s.desynced());
        assert_eq!(peers[0].s.link(), Link::Hosting);
    }

    #[test]
    fn hash_mismatch_flags_desync() {
        let mut rng = Rng::new(5);
        let mut net = Fake::new(3);
        let mut peers = lobby(&mut rng, &mut net);
        (0..100).for_each(|_| net.frame(&mut rng, &mut peers));
        let tic = (peers[2].rows.len() as u32 / HASH_EVERY + 1) * HASH_EVERY;
        peers[2].s.remote_hash(tic, 0xBAD);
        (0..100).for_each(|_| net.frame(&mut rng, &mut peers));
        assert!(peers[2].s.desynced());
        assert!(!peers[1].s.desynced());
    }

    #[test]
    fn refuses_bad_version_full_and_late_joins() {
        let mut host = Session::new(nick("h"));
        host.set_local_players(&[0, 1, 2], &mut ());
        host.reset(Role::Host, HOST, &mut ());
        let mut out = Outbox::default();
        let hello = |v, c| Msg::Hello { version: v, nick: nick("x"), local_count: c };
        host.receive(1, hello(*b"doppio 0\0\0\0\0\0\0\0\0", 1), &mut out);
        host.receive(2, hello(VERSION, 2), &mut out);
        host.receive(3, hello(VERSION, 1), &mut out);
        let leaves: Vec<(u16, Msg)> = out.0.iter().map(|(t, b)| (*t, Msg::decode(b).unwrap())).filter(|(_, m)| matches!(m, Msg::Leave { .. })).collect();
        assert_eq!(leaves, [(1, Msg::Leave { reason: Reason::Version }), (2, Msg::Leave { reason: Reason::Full })]);
        assert_eq!(host.seats().len(), 4);
        assert!(!host.can_start());
        host.receive(3, Msg::Ready { ready: true }, &mut out);
        assert!(host.start(1, 120, &mut out));
        out.0.clear();
        host.receive(4, hello(VERSION, 1), &mut out);
        assert_eq!(Msg::decode(&out.0[0].1), Some(Msg::Leave { reason: Reason::Started }));
        let mut client = Session::new(nick("c"));
        client.reset(Role::Client, 1, &mut ());
        client.receive(HOST, Msg::Leave { reason: Reason::Version }, &mut ());
        assert_eq!(client.link(), Link::Lost);
    }

    #[test]
    fn offline_is_immediate() {
        let mut s = Session::new(nick("me"));
        s.set_local_players(&[0, 2], &mut ());
        assert_eq!(s.seats().iter().map(|s| s.local_port).collect::<Vec<_>>(), [Some(0), Some(2)]);
        assert_eq!(s.pop_tic(), None);
        assert!(s.start(3, 360, &mut ()));
        assert_eq!(s.link(), Link::Offline);
        assert_eq!(s.game(), Some(Setup { seed: 3, players: 2, night_secs: 360 }));
        s.push_local(&[1, 2, 3, 4], &mut ());
        s.push_local(&[9, 9, 9, 9], &mut ());
        assert_eq!(s.pop_tic(), Some([1, 3, 0, 0]));
        assert_eq!(s.pop_tic(), None);
        s.push_local(&[5, 6, 7, 8], &mut ());
        assert_eq!(s.pop_tic(), Some([5, 7, 0, 0]));
        s.set_local_players(&[1], &mut ());
        assert_eq!(s.seats().len(), 2);
    }

    #[test]
    fn stop_loses_only_online() {
        let mut s = Session::new(nick("me"));
        s.stop();
        assert_eq!(s.link(), Link::Offline);
        s.reset(Role::Client, 3, &mut ());
        s.stop();
        assert_eq!(s.link(), Link::Lost);
    }

    #[test]
    fn hosting_lockstep_does_not_allocate() {
        let mut s = Session::new(nick("me"));
        s.reset(Role::Host, HOST, &mut ());
        assert!(s.start(9, 360, &mut ()));
        let allocs = crate::audio::alloc::count(|| {
            (0..300u32).for_each(|t| {
                s.push_local(&[1, 0, 0, 0], &mut ());
                assert!(s.pop_tic().is_some());
                s.report_hash(t, t, &mut ());
            })
        });
        assert_eq!(allocs, 0);
        assert!(!s.desynced());
    }
}
