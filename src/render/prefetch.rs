use crate::map::Map;
use std::thread::{Builder, JoinHandle};

pub struct Prefetch<T> {
    pending: Option<(&'static Map<'static>, JoinHandle<T>)>,
}

impl<T: Send + 'static> Prefetch<T> {
    pub fn new() -> Prefetch<T> {
        Prefetch { pending: None }
    }

    pub fn idle_for(&self, map: &'static Map<'static>) -> bool {
        self.pending.as_ref().is_none_or(|(m, h)| !std::ptr::eq(*m, map) && h.is_finished())
    }

    pub fn want(&mut self, map: &'static Map<'static>, build: impl FnOnce() -> T + Send + 'static) {
        if !self.idle_for(map) {
            return;
        }
        self.pending.take().into_iter().for_each(|(_, h)| drop(h.join()));
        self.pending = Builder::new().name("doppio-stage".into()).spawn(build).ok().map(|h| (map, h));
    }

    pub fn take(&mut self, map: &'static Map<'static>) -> Option<T> {
        match self.pending.take() {
            Some((m, h)) if std::ptr::eq(m, map) => h.join().ok(),
            other => {
                self.pending = other;
                None
            }
        }
    }
}

impl<T> Drop for Prefetch<T> {
    fn drop(&mut self) {
        self.pending.take().into_iter().for_each(|(_, h)| drop(h.join()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn builds_once_per_wanted_map_and_hands_it_over() {
        let maps = map::campaign();
        let built = Arc::new(AtomicUsize::new(0));
        let job = |v: u32| {
            let built = built.clone();
            move || {
                built.fetch_add(1, Ordering::SeqCst);
                v
            }
        };
        let mut p = Prefetch::new();
        assert_eq!(p.take(maps[0]), None);
        assert!(p.idle_for(maps[1]));
        p.want(maps[1], job(1));
        p.want(maps[1], job(9));
        assert!(!p.idle_for(maps[1]));
        assert_eq!(p.take(maps[0]), None);
        assert_eq!(p.take(maps[1]), Some(1));
        assert_eq!(p.take(maps[1]), None);
        p.want(maps[2], job(2));
        while !p.idle_for(maps[3]) {
            std::thread::yield_now();
        }
        p.want(maps[3], job(3));
        assert_eq!(p.take(maps[2]), None);
        assert_eq!(p.take(maps[3]), Some(3));
        assert_eq!(built.load(Ordering::SeqCst), 3);
    }
}
