pub const B: u16 = 1 << 0;
pub const Y: u16 = 1 << 1;
pub const SELECT: u16 = 1 << 2;
pub const START: u16 = 1 << 3;
pub const UP: u16 = 1 << 4;
pub const DOWN: u16 = 1 << 5;
pub const LEFT: u16 = 1 << 6;
pub const RIGHT: u16 = 1 << 7;
pub const A: u16 = 1 << 8;
pub const PLAY: u16 = !(START | SELECT);
pub const PAUSE: u16 = START | SELECT;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Back,
    Alt,
}

#[derive(Default)]
pub struct Pad {
    held: [u16; 4],
}

impl Pad {
    pub fn edges(&mut self, input: &[u16; 4]) -> [u16; 4] {
        let e = std::array::from_fn(|p| input[p] & !self.held[p]);
        self.held = *input;
        e
    }
}

pub fn merged(edges: &[u16; 4]) -> u16 {
    edges.iter().fold(0, |m, e| m | e)
}

pub fn key(bits: u16) -> Option<Key> {
    [(A | START, Key::Confirm), (B | SELECT, Key::Back), (UP, Key::Up), (DOWN, Key::Down), (LEFT, Key::Left), (RIGHT, Key::Right), (Y, Key::Alt)]
        .into_iter()
        .find(|(m, _)| bits & m != 0)
        .map(|(_, k)| k)
}

pub fn list(sel: usize, n: usize, key: Option<Key>) -> usize {
    match key {
        Some(Key::Up) => (sel + n - 1) % n.max(1),
        Some(Key::Down) => (sel + 1) % n.max(1),
        _ => sel,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_fire_once_per_press() {
        let mut p = Pad::default();
        assert_eq!(p.edges(&[A, 0, START, 0]), [A, 0, START, 0]);
        assert_eq!(p.edges(&[A, 0, START, 0]), [0; 4]);
        assert_eq!(p.edges(&[A | UP, 0, 0, 0]), [UP, 0, 0, 0]);
    }

    #[test]
    fn keys_and_lists() {
        assert_eq!(key(START), Some(Key::Confirm));
        assert_eq!(key(B | UP), Some(Key::Back));
        assert_eq!(key(1 << 9), None);
        assert_eq!(list(0, 3, Some(Key::Up)), 2);
        assert_eq!(list(2, 3, Some(Key::Down)), 0);
        assert_eq!(list(1, 3, Some(Key::Confirm)), 1);
        assert_eq!(merged(&[A, 0, B, 0]), A | B);
        assert_eq!(PLAY & START, 0);
    }
}
