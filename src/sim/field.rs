use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub const FAR: u16 = u16::MAX;
pub const STRAIGHT: u16 = 10;
pub const DIAGONAL: u16 = 14;
const FANOUT: usize = 9;
const SLACK: u16 = 40;

pub type Heap = BinaryHeap<Reverse<u32>>;

#[derive(Clone, Copy)]
pub struct Edges {
    to: [(u16, u16); FANOUT],
    len: u8,
}

impl Edges {
    pub const NONE: Edges = Edges { to: [(0, 0); FANOUT], len: 0 };

    pub fn with(mut self, cell: usize, cost: u16) -> Edges {
        self.push(cell, cost);
        self
    }

    fn push(&mut self, cell: usize, cost: u16) {
        if (self.len as usize) < FANOUT {
            self.to[self.len as usize] = (cell as u16, cost);
            self.len += 1;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, u16)> + '_ {
        self.to[..self.len as usize].iter().map(|&(c, w)| (c as usize, w))
    }
}

pub fn heap(cells: usize) -> Heap {
    BinaryHeap::with_capacity(cells * FANOUT + 1)
}

pub fn flood(dist: &mut [u16], heap: &mut Heap, seed: usize, limit: u16, stop: Option<usize>, edges: impl Fn(usize) -> Edges) -> bool {
    dist.fill(FAR);
    heap.clear();
    if seed >= dist.len() {
        return true;
    }
    dist[seed] = 0;
    heap.push(Reverse(seed as u32));
    let mut until = FAR;
    while let Some(Reverse(key)) = heap.pop() {
        let (d, c) = ((key >> 16) as u16, (key & 0xFFFF) as usize);
        if d > until {
            return false;
        }
        if d == dist[c] {
            if stop == Some(c) {
                until = d.saturating_add(SLACK);
            }
            edges(c).iter().for_each(|(n, w)| {
                let nd = d.saturating_add(w);
                if nd < dist[n] && nd <= limit {
                    dist[n] = nd;
                    heap.push(Reverse((nd as u32) << 16 | n as u32));
                }
            });
        }
    }
    true
}

pub fn grid_edges(w: i32, h: i32, c: usize, pass: impl Fn(usize) -> Option<u16>) -> Edges {
    let (x, y) = (c as i32 % w, c as i32 / w);
    let at = |dx: i32, dy: i32| {
        let (nx, ny) = (x + dx, y + dy);
        let n = ((0..w).contains(&nx) && (0..h).contains(&ny)).then_some((ny * w + nx) as usize)?;
        pass(n).map(|k| (n, k))
    };
    let sides = [at(0, -1), at(1, 0), at(0, 1), at(-1, 0)];
    let corners = [(0, 1, 1, -1), (1, 2, 1, 1), (2, 3, -1, 1), (3, 0, -1, -1)];
    let mut e = Edges::NONE;
    sides.iter().flatten().for_each(|&(n, k)| e.push(n, STRAIGHT + k));
    corners
        .iter()
        .filter(|&&(a, b, _, _)| sides[a].is_some() && sides[b].is_some())
        .filter_map(|&(_, _, dx, dy)| at(dx, dy))
        .for_each(|(n, k)| e.push(n, DIAGONAL + k));
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flood_takes_cheapest_route_and_respects_limit() {
        let (w, h) = (5, 3);
        let wall = |c: usize| c == 7;
        let door = |c: usize| c == 2;
        let pass = |c: usize| (!wall(c)).then(|| if door(c) { 40 } else { 0 });
        let mut dist = vec![0; 15];
        let mut q = heap(15);
        flood(&mut dist, &mut q, 0, FAR, None, |c| grid_edges(w, h, c, pass));
        assert_eq!(dist[1], 10);
        assert_eq!(dist[2], 10 + 10 + 40);
        assert_eq!(dist[7], FAR);
        assert_eq!(dist[14], 14 + 10 + 10 + 10 + 10);
        flood(&mut dist, &mut q, 0, 20, None, |c| grid_edges(w, h, c, pass));
        assert_eq!((dist[1], dist[6], dist[3]), (10, 14, FAR));
        assert!(q.capacity() >= 15 * 9);
        assert!(!flood(&mut dist, &mut q, 0, FAR, Some(1), |c| grid_edges(w, h, c, pass)));
        assert_eq!((dist[1], dist[6], dist[5]), (10, 14, 10));
        assert!(dist[14] == FAR || dist[14] > 50);
    }
}
