use super::player::{slots, JAIRO};
use super::tank::{pressed, A, B, DOWN, LEFT, RIGHT, UP, Y};
use super::{collide, Dial, Event, Focus, ItemKind, ItemPlace, PlayerState, World, BATTERY_FULL, TICS_PER_SEC};
use crate::map::LockKind;

const SWING: u16 = 4369;
pub const DOOR_NOISE: u8 = 60;
pub const RATTLE_NOISE: u8 = 80;
pub const WRONG_NOISE: u8 = 50;
pub const PORTAL_NOISE: u8 = 40;
pub const SALT_TICS: u16 = 45 * TICS_PER_SEC as u16;
pub const BALDE_STUN: u16 = 10 * TICS_PER_SEC as u16;
pub const TABACO_LEAVE: u16 = 60 * TICS_PER_SEC as u16;

impl World {
    pub(super) fn swing(&mut self) {
        self.doors.iter_mut().zip(&self.door_dir).for_each(|(door, &opening)| {
            door.open = match opening {
                true => door.open.saturating_add(SWING),
                false => door.open.saturating_sub(SWING),
            };
        });
        self.props.iter_mut().filter(|s| s.open > 0).for_each(|s| s.open = s.open.saturating_add(SWING));
        let expired = self.salt.iter().any(|&s| s == 1);
        self.salt.iter_mut().for_each(|s| *s = s.saturating_sub(1));
        self.brain.dirty |= expired;
    }

    pub(super) fn press_a(&mut self, p: usize) -> bool {
        self.focus_of(p).map(|f| self.act(p, f)).is_some()
    }

    fn act(&mut self, p: usize, f: Focus) {
        match f {
            Focus::Item { item } => self.pickup(p, item),
            Focus::Door { door, .. } => self.use_door(p, door as usize),
            Focus::Container { prop, locked: false } => self.open_container(p, prop),
            Focus::Container { .. } => {}
            Focus::Lock { lock, code: true, .. } => self.ctl[p].dial = Some(self.fresh_dial(lock)),
            Focus::Lock { lock, .. } => self.try_lock(p, lock),
            Focus::Place { lock, needs } => self.place_item(p, lock, needs),
            Focus::Hide { spot } => self.start_hide(p, spot),
            Focus::Portal { portal, locked: false } => self.travel(p, portal as usize),
            Focus::Portal { .. } => {}
            Focus::Note { note } => self.ctl[p].reading = Some(note),
            Focus::Friend { player } => self.wake(player as usize),
            Focus::Use { item } => self.use_item(p, item),
        }
    }

    pub(super) fn move_door(&mut self, d: usize, opening: bool) {
        if self.door_dir[d] != opening {
            self.door_dir[d] = opening;
            self.events.push(Event::DoorMove { door: d as u16, opening });
        }
    }

    fn door_noise(&mut self, d: usize, loud: u8) {
        let m = self.map.doors[d];
        let at = super::centre(m.x, m.y);
        self.noise(at, self.grid.area_of(at), loud);
    }

    fn rattle(&mut self, d: usize) {
        self.events.push(Event::DoorRattle { door: d as u16 });
        self.door_noise(d, RATTLE_NOISE);
    }

    fn door_occupied(&self, d: usize) -> bool {
        let m = self.map.doors[d];
        let tile = move |x: i32, y: i32| (x, y) == (m.x, m.y);
        let players = self.players[..self.count]
            .iter()
            .filter(|pl| matches!(pl.state, PlayerState::Free | PlayerState::Grabbed | PlayerState::Fainted { .. }))
            .any(|pl| collide::overlaps(pl.pos, &tile));
        players || collide::overlaps(self.legend.pos, &tile)
    }

    fn use_door(&mut self, p: usize, d: usize) {
        let door = self.doors[d];
        match (door.locked, door.stuck, self.door_dir[d]) {
            (true, _, _) => self.rattle(d),
            (false, true, _) if self.players[p].character == JAIRO => {
                self.doors[d].stuck = false;
                self.brain.dirty = true;
                self.move_door(d, true);
                self.door_noise(d, DOOR_NOISE);
            }
            (false, true, _) => self.rattle(d),
            (false, false, true) if self.door_occupied(d) => {}
            (false, false, open) => {
                self.move_door(d, !open);
                self.door_noise(d, DOOR_NOISE);
            }
        }
    }

    fn pickup(&mut self, p: usize, item: u8) {
        let i = item as usize;
        let lies = match self.items[i].place {
            ItemPlace::Floor { pos, area } => (pos, area),
            _ => return,
        };
        match self.items[i].kind {
            ItemKind::LanternaPilha => {
                self.players[p].battery = BATTERY_FULL;
                self.items[i].place = ItemPlace::Used;
            }
            ItemKind::Foto => self.items[i].place = ItemPlace::Used,
            _ => {
                let n = slots(self.players[p].character);
                let hands = self.players[p].hands;
                self.players[p].hands = match (0..n).find(|&s| hands[s].is_none()) {
                    Some(0) => [Some(item), hands[1]],
                    Some(_) => [hands[0], Some(item)],
                    None => {
                        let old = hands[0].unwrap_or(item);
                        self.items[old as usize].place = ItemPlace::Floor { pos: lies.0, area: lies.1 };
                        self.events.push(Event::Drop { player: p as u8, item: old });
                        match n {
                            2 => [hands[1], Some(item)],
                            _ => [Some(item), None],
                        }
                    }
                };
                self.items[i].place = ItemPlace::Held { player: p as u8 };
            }
        }
        self.events.push(Event::Pickup { player: p as u8, item });
    }

    fn open_container(&mut self, p: usize, prop: u16) {
        let at = self.map.props[prop as usize].pos;
        let area = self.players[p].area;
        self.props[prop as usize].open = 1;
        self.items
            .iter_mut()
            .filter(|s| s.place == ItemPlace::Inside { prop })
            .for_each(|s| s.place = ItemPlace::Floor { pos: at, area });
        self.events.push(Event::Container { player: p as u8, prop });
    }

    fn consume(&mut self, p: usize, item: u8) {
        let hands = self.players[p].hands;
        let mut kept = hands.into_iter().flatten().filter(|&i| i != item);
        self.players[p].hands = [kept.next(), kept.next()];
        self.items[item as usize].place = ItemPlace::Used;
        self.events.push(Event::Used { player: p as u8, item });
    }

    pub(super) fn fits(&self, lock: u8, item: u8) -> bool {
        match self.map.locks[lock as usize].kind {
            LockKind::Key(k) => k == item,
            LockKind::Tool(t) => self.items[item as usize].kind == t,
            _ => false,
        }
    }

    fn try_lock(&mut self, p: usize, lock: u8) {
        let fit = self.players[p].hands.into_iter().flatten().find(|&i| self.fits(lock, i));
        match fit {
            Some(i) => {
                if matches!(self.map.locks[lock as usize].kind, LockKind::Key(_)) {
                    self.consume(p, i);
                }
                self.open_lock(p, lock);
            }
            None => {
                if let Some(d) = self.door_ahead(p) {
                    self.rattle(d);
                }
            }
        }
    }

    fn place_item(&mut self, p: usize, lock: u8, needs: ItemKind) {
        if let Some(i) = self.holds(p, needs) {
            self.consume(p, i);
            self.open_lock(p, lock);
        }
    }

    fn travel(&mut self, p: usize, k: usize) {
        let pl = self.players[p];
        let portal = self.map.portals[k];
        self.noise(pl.pos, pl.area, PORTAL_NOISE);
        self.place_player(p, super::centre(portal.to_x, portal.to_y), portal.rot);
        self.events.push(Event::Portal { player: p as u8 });
        self.events.push(Event::PortalUse { player: p as u8, portal: k as u8 });
    }

    fn use_item(&mut self, p: usize, item: u8) {
        match self.items[item as usize].kind {
            ItemKind::Sal => {
                if let Some(d) = self.door_ahead(p) {
                    self.salt[d] = SALT_TICS;
                    self.brain.dirty = true;
                    self.consume(p, item);
                }
            }
            ItemKind::Balde => {
                self.consume(p, item);
                self.stun_legend(BALDE_STUN);
            }
            ItemKind::Tabaco => {
                self.consume(p, item);
                self.send_legend_home(TABACO_LEAVE);
            }
            _ => {}
        }
    }

    fn fresh_dial(&self, lock: u8) -> Dial {
        let len = match self.map.locks[lock as usize].kind {
            LockKind::Code { len, .. } => len.clamp(1, 4),
            _ => 4,
        };
        Dial { lock, digits: [0; 4], len, cursor: 0, wrong: 0 }
    }

    pub(super) fn dial_tic(&mut self, p: usize, prev: u16, input: u16) {
        let Some(mut d) = self.ctl[p].dial else { return };
        let hit = |b: u16| pressed(prev, input, b);
        if hit(Y) || hit(B) {
            self.ctl[p].dial = None;
            return;
        }
        let (len, c) = (d.len, d.cursor as usize);
        let moves = [(LEFT, true), (RIGHT, true), (UP, false), (DOWN, false)];
        let ticked = moves.iter().filter(|&&(b, _)| hit(b)).count() > 0;
        if hit(LEFT) {
            d.cursor = (d.cursor + len - 1) % len;
        }
        if hit(RIGHT) {
            d.cursor = (d.cursor + 1) % len;
        }
        if hit(UP) {
            d.digits[c] = (d.digits[c] + 1) % 10;
        }
        if hit(DOWN) {
            d.digits[c] = (d.digits[c] + 9) % 10;
        }
        if ticked {
            self.events.push(Event::DialTick { player: p as u8 });
        }
        self.ctl[p].dial = Some(d);
        if hit(A) {
            let right = match self.map.locks[d.lock as usize].kind {
                LockKind::Code { digits, len } => digits[..len as usize] == d.digits[..len as usize],
                _ => false,
            };
            match right {
                true => {
                    self.ctl[p].dial = None;
                    self.open_lock(p, d.lock);
                }
                false => {
                    self.ctl[p].dial = Some(Dial { wrong: d.wrong.saturating_add(1), ..d });
                    self.events.push(Event::DialWrong { player: p as u8 });
                    let pl = self.players[p];
                    self.noise(pl.pos, pl.area, WRONG_NOISE);
                }
            }
        }
    }
}
