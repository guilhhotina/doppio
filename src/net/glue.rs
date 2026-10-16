use super::proto::{MAX_PACKET, Msg};
use super::session::Wire;
use crate::core::Frontend;
use crate::ffi::*;
use std::ffi::c_void;
use std::sync::{Mutex, MutexGuard};

pub const QUEUE: usize = 256;
pub const MAX_CLIENTS: u32 = 3;

#[derive(Clone, Copy)]
pub enum Event {
    Start { me: u16 },
    Packet { from: u16, len: u8, data: [u8; MAX_PACKET] },
    Gone { client: u16 },
    Stop,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Admission {
    pub open: bool,
    pub accepted: u64,
}

impl Admission {
    pub const CLOSED: Admission = Admission { open: false, accepted: 0 };

    pub fn admit(self, client: u16) -> Option<Admission> {
        (self.open && client > 0 && client < 64 && self.accepted.count_ones() < MAX_CLIENTS)
            .then(|| Admission { accepted: self.accepted | 1 << client, ..self })
    }

    pub fn release(self, client: u16) -> Admission {
        Admission { accepted: self.accepted & !1u64.checked_shl(client as u32).unwrap_or(0), ..self }
    }
}

struct Queue {
    events: [Event; QUEUE],
    head: usize,
    len: usize,
    overflow: bool,
}

impl Queue {
    fn push(&mut self, e: Event) {
        match self.len < QUEUE {
            true => {
                self.events[(self.head + self.len) % QUEUE] = e;
                self.len += 1;
            }
            false => self.overflow = true,
        }
    }

    fn pop(&mut self) -> Option<Event> {
        (self.len > 0).then(|| {
            let e = self.events[self.head];
            self.head = (self.head + 1) % QUEUE;
            self.len -= 1;
            e
        })
    }
}

struct State {
    send: Option<NetpacketSendFn>,
    poll_receive: Option<NetpacketPollReceiveFn>,
    admission: Admission,
    queue: Queue,
}

static STATE: Mutex<State> = Mutex::new(State {
    send: None,
    poll_receive: None,
    admission: Admission::CLOSED,
    queue: Queue { events: [Event::Stop; QUEUE], head: 0, len: 0, overflow: false },
});

fn state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

unsafe extern "C" fn start(client_id: u16, send: Option<NetpacketSendFn>, poll_receive: Option<NetpacketPollReceiveFn>) {
    let mut s = state();
    s.send = send;
    s.poll_receive = poll_receive;
    s.admission.accepted = 0;
    s.queue.push(Event::Start { me: client_id });
}

unsafe extern "C" fn receive(buf: *const c_void, len: usize, client_id: u16) {
    if buf.is_null() || len == 0 || len > MAX_PACKET {
        return;
    }
    let bytes = unsafe { std::slice::from_raw_parts(buf.cast::<u8>(), len) };
    let mut data = [0; MAX_PACKET];
    data[..len].copy_from_slice(bytes);
    state().queue.push(Event::Packet { from: client_id, len: len as u8, data });
}

unsafe extern "C" fn stop() {
    let mut s = state();
    s.send = None;
    s.poll_receive = None;
    s.admission.accepted = 0;
    s.queue.push(Event::Stop);
}

unsafe extern "C" fn connected(client_id: u16) -> bool {
    let mut s = state();
    s.admission.admit(client_id).map(|a| s.admission = a).is_some()
}

unsafe extern "C" fn disconnected(client_id: u16) {
    let mut s = state();
    s.admission = s.admission.release(client_id);
    s.queue.push(Event::Gone { client: client_id });
}

pub fn register(fe: &Frontend) {
    let mut cb = NetpacketCallback {
        start: Some(start),
        receive: Some(receive),
        stop: Some(stop),
        poll: None,
        connected: Some(connected),
        disconnected: Some(disconnected),
        protocol_version: c"doppio 1".as_ptr(),
    };
    if !fe.env(ENV_SET_NETPACKET_INTERFACE, &mut cb) {
        fe.log(LOG_INFO, "frontend has no netpacket interface: offline only");
    }
}

pub fn pump() {
    let poll_receive = state().poll_receive;
    poll_receive.iter().for_each(|f| unsafe { f() });
}

pub fn next() -> Option<Event> {
    state().queue.pop()
}

pub fn take_overflow() -> bool {
    std::mem::take(&mut state().queue.overflow)
}

pub fn set_open(open: bool) {
    state().admission.open = open;
}

pub struct Packets;

impl Wire for Packets {
    fn send(&mut self, to: u16, msg: &Msg) {
        let mut buf = [0; MAX_PACKET];
        let n = msg.encode(&mut buf);
        let send = state().send;
        send.iter().for_each(|f| unsafe {
            f(NETPACKET_RELIABLE | NETPACKET_FLUSH_HINT, buf.as_ptr().cast(), n, to);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admits_three_clients_while_open() {
        let open = Admission { open: true, accepted: 0 };
        let a = [1, 2, 5].iter().try_fold(open, |a, &c| a.admit(c)).unwrap();
        assert_eq!(a.accepted.count_ones(), 3);
        assert_eq!(a.admit(6), None);
        assert_eq!(a.release(2).admit(6).map(|b| b.accepted), Some(1 << 1 | 1 << 5 | 1 << 6));
        assert_eq!(Admission { open: false, ..open }.admit(1), None);
        assert_eq!(open.admit(0), None);
        assert_eq!(open.admit(64), None);
        assert_eq!(open.release(400), open);
    }

    #[test]
    fn queue_keeps_order_and_flags_overflow() {
        let mut q = Queue { events: [Event::Stop; QUEUE], head: 0, len: 0, overflow: false };
        (0..QUEUE as u16 + 1).for_each(|c| q.push(Event::Gone { client: c }));
        assert!(q.overflow);
        let order: Vec<u16> = std::iter::from_fn(|| q.pop())
            .map(|e| match e {
                Event::Gone { client } => client,
                _ => u16::MAX,
            })
            .collect();
        assert_eq!(order, (0..QUEUE as u16).collect::<Vec<_>>());
    }
}
