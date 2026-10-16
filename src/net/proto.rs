pub const VERSION: [u8; 16] = *b"doppio 2\0\0\0\0\0\0\0\0";
pub const MAX_PACKET: usize = 96;
pub const NICK: usize = 16;
pub const SLOTS: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Owner {
    pub client: u16,
    pub index: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RosterSeat {
    pub owner: Owner,
    pub ready: bool,
    pub nick: [u8; NICK],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reason {
    Bye,
    Version,
    Full,
    Started,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Msg {
    Hello { version: [u8; 16], nick: [u8; NICK], local_count: u8, ready: bool },
    Roster { count: u8, seats: [RosterSeat; SLOTS] },
    Ready { ready: bool },
    Start { seed: u32, players: u8, night_secs: u32, slots: [Owner; SLOTS] },
    Input { tic: u32, slot: u8, bits: u16 },
    Tic { tic: u32, inputs: [u16; SLOTS] },
    Hash { tic: u32, hash: u32 },
    Leave { reason: Reason },
}

pub const NO_OWNER: Owner = Owner { client: u16::MAX, index: 0 };
pub const EMPTY_SEAT: RosterSeat = RosterSeat { owner: NO_OWNER, ready: false, nick: [0; NICK] };

const HELLO: u8 = 1;
const ROSTER: u8 = 2;
const READY: u8 = 3;
const START: u8 = 4;
const INPUT: u8 = 5;
const TIC: u8 = 6;
const HASH: u8 = 7;
const LEAVE: u8 = 8;

struct W<'a> {
    buf: &'a mut [u8; MAX_PACKET],
    n: usize,
}

impl W<'_> {
    fn bytes(mut self, b: &[u8]) -> Self {
        self.buf[self.n..self.n + b.len()].copy_from_slice(b);
        self.n += b.len();
        self
    }
    fn u8(self, v: u8) -> Self {
        self.bytes(&[v])
    }
    fn u16(self, v: u16) -> Self {
        self.bytes(&v.to_le_bytes())
    }
    fn u32(self, v: u32) -> Self {
        self.bytes(&v.to_le_bytes())
    }
    fn owner(self, o: &Owner) -> Self {
        self.u16(o.client).u8(o.index)
    }
}

struct R<'a>(&'a [u8]);

impl R<'_> {
    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let (head, rest) = self.0.split_first_chunk::<N>()?;
        self.0 = rest;
        Some(*head)
    }
    fn u8(&mut self) -> Option<u8> {
        self.take::<1>().map(|[b]| b)
    }
    fn u16(&mut self) -> Option<u16> {
        self.take().map(u16::from_le_bytes)
    }
    fn u32(&mut self) -> Option<u32> {
        self.take().map(u32::from_le_bytes)
    }
    fn bool(&mut self) -> Option<bool> {
        self.u8().filter(|&b| b < 2).map(|b| b == 1)
    }
    fn owner(&mut self) -> Option<Owner> {
        Some(Owner { client: self.u16()?, index: self.u8()? })
    }
    fn seat(&mut self) -> Option<RosterSeat> {
        Some(RosterSeat { owner: self.owner()?, ready: self.bool()?, nick: self.take()? })
    }
    fn array<T: Copy, const N: usize>(&mut self, mut f: impl FnMut(&mut Self) -> Option<T>) -> Option<[T; N]> {
        let items: [Option<T>; N] = std::array::from_fn(|_| f(self));
        items.iter().all(Option::is_some).then(|| items.map(Option::unwrap))
    }
    fn end<T>(&self, v: T) -> Option<T> {
        self.0.is_empty().then_some(v)
    }
}

fn reason_code(r: Reason) -> u8 {
    match r {
        Reason::Bye => 0,
        Reason::Version => 1,
        Reason::Full => 2,
        Reason::Started => 3,
    }
}

fn reason(code: u8) -> Option<Reason> {
    [Reason::Bye, Reason::Version, Reason::Full, Reason::Started].get(code as usize).copied()
}

impl Msg {
    pub fn encode(&self, buf: &mut [u8; MAX_PACKET]) -> usize {
        let w = W { buf, n: 0 };
        match self {
            Msg::Hello { version, nick, local_count, ready } => w.u8(HELLO).bytes(version).bytes(nick).u8(*local_count).u8(*ready as u8),
            Msg::Roster { count, seats } => seats
                .iter()
                .fold(w.u8(ROSTER).u8(*count), |w, s| w.owner(&s.owner).u8(s.ready as u8).bytes(&s.nick)),
            Msg::Ready { ready } => w.u8(READY).u8(*ready as u8),
            Msg::Start { seed, players, night_secs, slots } => {
                slots.iter().fold(w.u8(START).u32(*seed).u8(*players).u32(*night_secs), |w, o| w.owner(o))
            }
            Msg::Input { tic, slot, bits } => w.u8(INPUT).u32(*tic).u8(*slot).u16(*bits),
            Msg::Tic { tic, inputs } => inputs.iter().fold(w.u8(TIC).u32(*tic), |w, &b| w.u16(b)),
            Msg::Hash { tic, hash } => w.u8(HASH).u32(*tic).u32(*hash),
            Msg::Leave { reason } => w.u8(LEAVE).u8(reason_code(*reason)),
        }
        .n
    }

    pub fn decode(b: &[u8]) -> Option<Msg> {
        let mut r = R(b);
        let msg = match r.u8()? {
            HELLO => Msg::Hello { version: r.take()?, nick: r.take()?, local_count: r.u8()?, ready: r.bool()? },
            ROSTER => Msg::Roster { count: r.u8().filter(|&c| c as usize <= SLOTS)?, seats: r.array(R::seat)? },
            READY => Msg::Ready { ready: r.bool()? },
            START => Msg::Start {
                seed: r.u32()?,
                players: r.u8().filter(|&p| (1..=SLOTS as u8).contains(&p))?,
                night_secs: r.u32()?,
                slots: r.array(R::owner)?,
            },
            INPUT => Msg::Input { tic: r.u32()?, slot: r.u8().filter(|&s| (s as usize) < SLOTS)?, bits: r.u16()? },
            TIC => Msg::Tic { tic: r.u32()?, inputs: r.array(R::u16)? },
            HASH => Msg::Hash { tic: r.u32()?, hash: r.u32()? },
            LEAVE => Msg::Leave { reason: r.u8().and_then(reason)? },
            _ => return None,
        };
        r.end(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nick(s: &str) -> [u8; NICK] {
        std::array::from_fn(|i| s.as_bytes().get(i).copied().unwrap_or(0))
    }

    #[test]
    fn roundtrip() {
        let seat = |c, i, r, n| RosterSeat { owner: Owner { client: c, index: i }, ready: r, nick: nick(n) };
        let msgs = [
            Msg::Hello { version: VERSION, nick: nick("João"), local_count: 2, ready: true },
            Msg::Hello { version: VERSION, nick: nick("x"), local_count: 1, ready: false },
            Msg::Roster { count: 3, seats: [seat(0, 0, true, "a"), seat(0, 1, true, "a"), seat(7, 0, false, "b"), EMPTY_SEAT] },
            Msg::Ready { ready: true },
            Msg::Ready { ready: false },
            Msg::Start {
                seed: 0xDEAD_BEEF,
                players: 3,
                night_secs: 360,
                slots: [Owner { client: 0, index: 0 }, Owner { client: 0, index: 1 }, Owner { client: 7, index: 0 }, NO_OWNER],
            },
            Msg::Input { tic: 123_456, slot: 3, bits: 0x3FF },
            Msg::Tic { tic: u32::MAX, inputs: [1, 2, 0x200, 0xFFFF] },
            Msg::Hash { tic: 30, hash: 0x1234_5678 },
            Msg::Leave { reason: Reason::Version },
            Msg::Leave { reason: Reason::Started },
        ];
        let mut buf = [0; MAX_PACKET];
        msgs.iter().for_each(|m| {
            let n = m.encode(&mut buf);
            assert_eq!(Msg::decode(&buf[..n]), Some(*m));
            assert_eq!(Msg::decode(&buf[..n - 1]), None);
            assert!((0..n).all(|k| Msg::decode(&buf[..k]).is_none()));
        });
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(Msg::decode(&[]), None);
        assert_eq!(Msg::decode(&[99, 0, 0]), None);
        assert_eq!(Msg::decode(&[READY, 2]), None);
        assert_eq!(Msg::decode(&[LEAVE, 9]), None);
        assert_eq!(Msg::decode(&[INPUT, 0, 0, 0, 0, 4, 0, 0]), None);
        assert_eq!(Msg::decode(&[READY, 1, 0]), None);
    }
}
