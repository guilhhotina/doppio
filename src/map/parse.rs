use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Section {
    Map,
    Solid,
    Rooms,
    Room,
    Area,
    Door,
    Portal,
    Lock,
    Item,
    Note,
    Hide,
    Prop,
    Light,
    Cam,
    Wake,
    Legend,
    Exit,
}

const SECTIONS: &[(&str, Section)] = &[
    ("map", Section::Map),
    ("solid", Section::Solid),
    ("rooms", Section::Rooms),
    ("room", Section::Room),
    ("area", Section::Area),
    ("door", Section::Door),
    ("portal", Section::Portal),
    ("lock", Section::Lock),
    ("item", Section::Item),
    ("note", Section::Note),
    ("hide", Section::Hide),
    ("prop", Section::Prop),
    ("light", Section::Light),
    ("cam", Section::Cam),
    ("wake", Section::Wake),
    ("legend", Section::Legend),
    ("exit", Section::Exit),
];

const SKIES: &[(&str, Sky)] = &[("night", Sky::Night), ("moon", Sky::Moon), ("overcast", Sky::Overcast), ("none", Sky::None)];

const LEGENDS: &[(&str, LegendKind)] = &[
    ("corposeco", LegendKind::CorpoSeco),
    ("cabra", LegendKind::Cabra),
    ("mula", LegendKind::Mula),
    ("matinta", LegendKind::Matinta),
];

const ITEMS: &[(&str, ItemKind)] = &[
    ("chave", ItemKind::Chave),
    ("alicate", ItemKind::Alicate),
    ("martelo", ItemKind::Martelo),
    ("pe_de_cabra", ItemKind::PeDeCabra),
    ("chave_de_fenda", ItemKind::ChaveDeFenda),
    ("fusivel", ItemKind::Fusivel),
    ("sal", ItemKind::Sal),
    ("vela", ItemKind::Vela),
    ("fosforo", ItemKind::Fosforo),
    ("tabaco", ItemKind::Tabaco),
    ("balde", ItemKind::Balde),
    ("freio", ItemKind::Freio),
    ("ossos", ItemKind::Ossos),
    ("cruz", ItemKind::Cruz),
    ("bilhete_onibus", ItemKind::BilheteOnibus),
    ("manivela", ItemKind::Manivela),
    ("remo", ItemKind::Remo),
    ("lanterna_pilha", ItemKind::LanternaPilha),
    ("foto", ItemKind::Foto),
];

const TOOLS: &[(&str, ItemKind)] = &[
    ("alicate", ItemKind::Alicate),
    ("martelo", ItemKind::Martelo),
    ("pe_de_cabra", ItemKind::PeDeCabra),
    ("chave_de_fenda", ItemKind::ChaveDeFenda),
];

const PORTALS: &[(&str, PortalKind)] = &[
    ("escada", PortalKind::Escada),
    ("escada_mao", PortalKind::EscadaMao),
    ("bueiro", PortalKind::Bueiro),
    ("portao", PortalKind::Portao),
    ("janela_alta", PortalKind::JanelaAlta),
];

const HIDES: &[(&str, HideKind)] = &[
    ("cama", HideKind::Cama),
    ("guarda_roupa", HideKind::GuardaRoupa),
    ("armario", HideKind::Armario),
    ("tumulo", HideKind::Tumulo),
    ("caixa_dagua", HideKind::CaixaDagua),
    ("confessionario", HideKind::Confessionario),
    ("carro", HideKind::Carro),
    ("cesto", HideKind::Cesto),
    ("mesa", HideKind::Mesa),
];

const LIGHTS: &[(&str, LightMode)] = &[("steady", LightMode::Steady), ("flicker", LightMode::Flicker), ("dead", LightMode::Dead)];

const NOTE_MAX_CHARS: usize = 120;
pub const HINT_MAX_CHARS: usize = 44;

fn err<T>(line: usize, msg: impl Into<String>) -> Result<T, ParseError> {
    Err(ParseError { line, msg: msg.into() })
}

fn tile_of(b: u8) -> Option<Tile> {
    match b {
        b' ' => Some(Tile::Void),
        b'#' => Some(Tile::Wall),
        b'.' => Some(Tile::Floor),
        b'D' => Some(Tile::Door),
        b'W' => Some(Tile::Window),
        b'B' => Some(Tile::Blocker),
        b'H' => Some(Tile::Hide),
        b'P' => Some(Tile::Portal),
        b'~' => Some(Tile::Water),
        b'|' => Some(Tile::Fence),
        _ => None,
    }
}

fn unquote(s: &str) -> Option<&str> {
    s.strip_prefix('"').and_then(|r| r.strip_suffix('"'))
}

struct Fields<'a> {
    n: usize,
    toks: Vec<&'a str>,
    at: usize,
}

impl<'a> Fields<'a> {
    fn new(n: usize, line: &'a str) -> Result<Fields<'a>, ParseError> {
        if line.bytes().filter(|b| *b == b'"').count() % 2 == 1 {
            return err(n, "unterminated \"string\"");
        }
        let mut quoted = false;
        let toks = line
            .split(|c: char| {
                quoted ^= c == '"';
                c.is_whitespace() && !quoted
            })
            .filter(|t| !t.is_empty())
            .collect();
        Ok(Fields { n, toks, at: 0 })
    }

    fn peek(&self) -> Option<&'a str> {
        self.toks.get(self.at).copied()
    }

    fn word(&mut self, what: &str) -> Result<&'a str, ParseError> {
        let w = self.peek().map_or_else(|| err(self.n, format!("missing {what}")), Ok)?;
        self.at += 1;
        Ok(w)
    }

    fn fx(&mut self, what: &str) -> Result<Fx, ParseError> {
        let w = self.word(what)?;
        Fx::parse(w).map_or_else(|| err(self.n, format!("bad number for {what}: '{w}'")), Ok)
    }

    fn int(&mut self, what: &str) -> Result<i32, ParseError> {
        let w = self.word(what)?;
        w.parse().or_else(|_| err(self.n, format!("bad integer for {what}: '{w}'")))
    }

    fn pos(&mut self) -> Result<Vec2, ParseError> {
        Ok(Vec2::new(self.fx("x")?, self.fx("y")?))
    }

    fn centred(&mut self, what: &str) -> Result<Fx, ParseError> {
        let whole = self.peek().is_some_and(|w| !w.contains('.'));
        self.fx(what).map(|v| if whole { v + Fx::HALF } else { v })
    }

    fn spot(&mut self) -> Result<Vec2, ParseError> {
        Ok(Vec2::new(self.centred("x")?, self.centred("y")?))
    }

    fn cell(&mut self) -> Result<(i32, i32), ParseError> {
        Ok((self.int("x")?, self.int("y")?))
    }

    fn rot(&mut self) -> Result<Ang, ParseError> {
        self.fx("rot_deg").map(Ang::deg)
    }

    fn point3(&mut self, names: [&str; 3]) -> Result<[Fx; 3], ParseError> {
        Ok([self.fx(names[0])?, self.fx(names[1])?, self.fx(names[2])?])
    }

    fn rect(&mut self) -> Result<Rect, ParseError> {
        let r = Rect { x0: self.int("x0")?, y0: self.int("y0")?, x1: self.int("x1")?, y1: self.int("y1")? };
        match r.x0 <= r.x1 && r.y0 <= r.y1 {
            true => Ok(r),
            false => err(self.n, "rect corners out of order"),
        }
    }

    fn pick<T: Copy>(&mut self, what: &str, options: &[(&str, T)]) -> Result<T, ParseError> {
        let w = self.word(what)?;
        lookup(self.n, what, w, options)
    }

    fn opt_key(&mut self, key: &str) -> Option<&'a str> {
        let v = self.peek().and_then(|w| w.strip_prefix(key)).and_then(|r| r.strip_prefix('='))?;
        self.at += 1;
        Some(v)
    }

    fn text(&mut self, what: &str) -> Result<&'a str, ParseError> {
        let w = self.word(what)?;
        unquote(w).map_or_else(|| err(self.n, format!("{what} must be a \"quoted string\", got '{w}'")), Ok)
    }

    fn end<T>(self, value: T) -> Result<T, ParseError> {
        match self.peek() {
            None => Ok(value),
            Some(extra) => err(self.n, format!("unexpected '{extra}'")),
        }
    }
}

fn lookup<T: Copy>(n: usize, what: &str, w: &str, options: &[(&str, T)]) -> Result<T, ParseError> {
    options
        .iter()
        .find(|(name, _)| *name == w)
        .map_or_else(|| err(n, format!("unknown {what} '{w}'")), |(_, v)| Ok(*v))
}

struct Keys<'a> {
    n: usize,
    pairs: Vec<(&'a str, &'a str)>,
}

impl<'a> Keys<'a> {
    fn new(n: usize, toks: &[&'a str], allowed: &[&str]) -> Result<Keys<'a>, ParseError> {
        let pairs = toks
            .iter()
            .map(|t| {
                t.split_once('=')
                    .filter(|(k, v)| allowed.contains(k) && !v.is_empty())
                    .map(|(k, v)| (k, unquote(v).unwrap_or(v)))
                    .map_or_else(|| err(n, format!("unexpected '{t}' (expected one of {})", allowed.join("= ") + "=")), Ok)
            })
            .collect::<Result<Vec<_>, _>>()?;
        match pairs.iter().enumerate().find(|(i, (k, _))| pairs[..*i].iter().any(|(o, _)| o == k)) {
            Some((_, (k, _))) => err(n, format!("duplicate {k}=")),
            None => Ok(Keys { n, pairs }),
        }
    }

    fn get(&self, key: &str) -> Option<&'a str> {
        self.pairs.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }

    fn need(&self, key: &str) -> Result<&'a str, ParseError> {
        self.get(key).map_or_else(|| err(self.n, format!("missing {key}=")), Ok)
    }

    fn fx_or(&self, key: &str, default: Fx) -> Result<Fx, ParseError> {
        self.get(key).map_or(Ok(default), |v| Fx::parse(v).map_or_else(|| err(self.n, format!("bad number {key}={v}")), Ok))
    }

    fn int(&self, key: &str, max: u32) -> Result<u8, ParseError> {
        let v = self.need(key)?;
        v.parse::<u8>()
            .ok()
            .filter(|x| u32::from(*x) <= max)
            .map_or_else(|| err(self.n, format!("{key}= must be an integer 0..{max}, got '{v}'")), Ok)
    }
}

fn rgb(n: usize, s: &str) -> Result<[u8; 3], ParseError> {
    let parts: Vec<Option<u8>> = s.split(',').map(|p| p.parse().ok()).collect();
    match parts.as_slice() {
        [Some(r), Some(g), Some(b)] => Ok([*r, *g, *b]),
        _ => err(n, format!("expected r,g,b (0..255), got '{s}'")),
    }
}

fn span<'a>(line: &'a str, first: &str, last: &str) -> &'a str {
    let base = line.as_ptr() as usize;
    &line[first.as_ptr() as usize - base..last.as_ptr() as usize - base + last.len()]
}

struct Src<'a> {
    lines: Vec<(Section, usize, &'a str)>,
}

impl<'a> Src<'a> {
    fn of(&self, s: Section) -> impl Iterator<Item = (usize, &'a str)> + '_ {
        self.lines.iter().filter(move |(sec, _, _)| *sec == s).map(|(_, n, l)| (*n, *l))
    }

    fn fields(&self, s: Section) -> impl Iterator<Item = Result<Fields<'a>, ParseError>> + '_ {
        self.of(s).map(|(n, l)| Fields::new(n, l.trim()))
    }

    fn all<T>(&self, s: Section, f: impl Fn(Fields<'a>) -> Result<T, ParseError>) -> Result<Vec<T>, ParseError> {
        self.fields(s).map(|r| r.and_then(&f)).collect()
    }

    fn last_line(&self) -> usize {
        self.lines.last().map_or(0, |(_, n, _)| *n)
    }
}

fn is_grid(s: Option<Section>) -> bool {
    matches!(s, Some(Section::Solid | Section::Rooms))
}

fn split(src: &str) -> Result<Src<'_>, ParseError> {
    let (_, lines) = src.lines().enumerate().try_fold((None, Vec::new()), |(sec, mut out), (i, raw)| {
        let n = i + 1;
        let line = raw.trim_end_matches('\r');
        let t = line.trim();
        if t.starts_with(';') || line.is_empty() || (t.is_empty() && !is_grid(sec)) {
            return Ok((sec, out));
        }
        if let Some((name, rest)) = t.strip_prefix('[').and_then(|r| r.split_once(']')) {
            let s = lookup(n, "section", name, SECTIONS)?;
            if !rest.trim().is_empty() {
                out.push((s, n, rest.trim()));
            }
            return Ok((Some(s), out));
        }
        let s = sec.map_or_else(|| err(n, "data before any [section]"), Ok)?;
        out.push((s, n, line));
        Ok((sec, out))
    })?;
    Ok(Src { lines })
}

struct Header<'a> {
    name: &'a str,
    w: i32,
    h: i32,
    sky: Sky,
    fog: [u8; 3],
    fog_near: Fx,
    fog_far: Fx,
    legend: LegendKind,
    power: bool,
}

fn header<'a>(src: &Src<'a>) -> Result<Header<'a>, ParseError> {
    let lines: Vec<Fields<'a>> = src.fields(Section::Map).collect::<Result<_, _>>()?;
    let n = lines.first().map_or_else(|| err(0, "missing [map] header"), |f| Ok(f.n))?;
    let toks: Vec<&str> = lines.iter().flat_map(|f| f.toks.iter().copied()).collect();
    let k = Keys::new(n, &toks, &["name", "w", "h", "sky", "fog", "fog_near", "fog_far", "legend", "power"])?;
    let size = |key: &str| {
        k.need(key).and_then(|v| v.parse::<i32>().ok().filter(|x| (1..=255).contains(x)).map_or_else(|| err(n, format!("{key}= must be 1..255")), Ok))
    };
    Ok(Header {
        name: k.need("name")?,
        w: size("w")?,
        h: size("h")?,
        sky: lookup(n, "sky", k.need("sky")?, SKIES)?,
        fog: rgb(n, k.need("fog")?)?,
        fog_near: k.fx_or("fog_near", Fx::int(4))?,
        fog_far: k.fx_or("fog_far", Fx::int(16))?,
        legend: lookup(n, "legend", k.need("legend")?, LEGENDS)?,
        power: k.get("power").map_or(Ok(true), |v| lookup(n, "power", v, &[("on", true), ("off", false)]))?,
    })
}

fn grid<'a, T>(rows: &[(usize, &'a str)], w: i32, h: i32, what: &str, last: usize, cell: impl Fn(u8) -> Option<T>) -> Result<Vec<T>, ParseError> {
    if rows.len() != h as usize {
        return err(rows.last().map_or(last, |(n, _)| *n), format!("[{what}] has {} rows, [map] h={h}", rows.len()));
    }
    rows.iter()
        .map(|&(n, row)| match row.len() <= w as usize {
            false => err(n, format!("[{what}] row is {} wide, [map] w={w}", row.len())),
            true => row
                .bytes()
                .chain(std::iter::repeat_n(b' ', w as usize - row.len()))
                .enumerate()
                .map(|(x, b)| cell(b).map_or_else(|| err(n, format!("bad [{what}] cell '{}' at column {x}", b as char)), Ok))
                .collect::<Result<Vec<T>, _>>(),
        })
        .collect::<Result<Vec<Vec<T>>, _>>()
        .map(|v| v.into_iter().flatten().collect())
}

fn door_rooms(tiles: &[Tile], given: Vec<Option<u8>>, w: i32) -> Vec<Option<u8>> {
    let h = tiles.len() as i32 / w;
    let at = |x: i32, y: i32| ((0..w).contains(&x) && (0..h).contains(&y)).then(|| (y * w + x) as usize);
    let near = |i: usize| {
        let (x, y) = (i as i32 % w, i as i32 / w);
        [(0, -1), (-1, 0), (1, 0), (0, 1)]
            .iter()
            .filter_map(|(dx, dy)| at(x + dx, y + dy))
            .find_map(|j| given[j].filter(|_| tiles[j] != Tile::Door))
    };
    (0..given.len()).map(|i| given[i].or_else(|| (tiles[i] == Tile::Door).then(|| near(i)).flatten())).collect()
}

fn area<'a>(line: &'a str, mut f: Fields<'a>) -> Result<Area<'a>, ParseError> {
    let id = f.int("area number")?;
    let first = f.word("area name")?;
    let name = span(line, first, f.toks.last().copied().unwrap_or(first));
    Ok(Area { id: id.clamp(0, 255) as u8, name })
}

fn room<'a>(line: &'a str, mut f: Fields<'a>, areas: usize) -> Result<Room<'a>, ParseError> {
    let n = f.n;
    let id = f.word("room id")?;
    let id = match id.as_bytes() {
        [b] if *b != b'-' && b.is_ascii_graphic() => *b,
        _ => return err(n, format!("room id must be one character, got '{id}'")),
    };
    let words = f.toks[f.at..].iter().take_while(|t| !t.contains('=')).count();
    let name = match words {
        0 => return err(n, "missing room name"),
        k => span(line, f.toks[f.at], f.toks[f.at + k - 1]),
    };
    let k = Keys::new(n, &f.toks[f.at + words..], &["area", "floor", "wall", "ceil", "ext", "h", "light", "out"])?;
    let opt = |v: &'a str| Some(v).filter(|v| *v != "-");
    let room = Room {
        id,
        name,
        area: k.get("area").map_or(Ok(0), |_| k.int("area", 255))?,
        floor: k.need("floor")?,
        wall: k.need("wall")?,
        ceil: opt(k.need("ceil")?),
        ext: k.get("ext").and_then(opt),
        h: k.fx_or("h", Fx::int(2) + Fx::ratio(3, 5))?,
        light: k.int("light", 100)?,
        out: k.get("out").map_or(Ok(false), |v| lookup(n, "out", v, &[("0", false), ("1", true)]))?,
    };
    match usize::from(room.area) < areas {
        true => Ok(room),
        false => err(n, format!("room area={} but only {areas} [area] lines", room.area)),
    }
}

struct Ids<'a> {
    items: Vec<&'a str>,
    locks: Vec<&'a str>,
}

impl<'a> Ids<'a> {
    fn item(&self, n: usize, id: &str) -> Result<u8, ParseError> {
        self.items.iter().position(|i| *i == id).map_or_else(|| err(n, format!("unknown item '{id}'")), |i| Ok(i as u8))
    }

    fn lock(&self, n: usize, id: &str) -> Result<u8, ParseError> {
        self.locks.iter().position(|l| *l == id).map_or_else(|| err(n, format!("unknown lock '{id}'")), |i| Ok(i as u8))
    }
}

fn unique<'a>(src: &Src<'a>, s: Section, what: &str) -> Result<Vec<&'a str>, ParseError> {
    let ids = src
        .fields(s)
        .map(|f| f.and_then(|mut f| f.word(what).map(|id| (f.n, id))))
        .collect::<Result<Vec<_>, _>>()?;
    if ids.len() > 255 {
        return err(ids[255].0, format!("more than 255 {what}s"));
    }
    match ids.iter().enumerate().find(|(i, (_, id))| ids[..*i].iter().any(|(_, o)| o == id)) {
        Some((_, (n, id))) => err(*n, format!("duplicate {what} '{id}'")),
        None => Ok(ids.into_iter().map(|(_, id)| id).collect()),
    }
}

fn item<'a>(mut f: Fields<'a>) -> Result<Item<'a>, ParseError> {
    let id = f.word("item id")?;
    let kind = f.pick("item kind", ITEMS)?;
    let pos = f.spot()?;
    f.end(Item { id, kind, pos, inside: None })
}

fn hint<'a>(f: &mut Fields<'a>) -> Result<Option<&'a str>, ParseError> {
    let Some(raw) = f.toks.last().and_then(|t| t.strip_prefix("hint=")) else { return Ok(None) };
    f.toks.pop();
    match unquote(raw).map(|t| (t, t.chars().count())) {
        None => err(f.n, format!("hint must be a \"quoted string\", got '{raw}'")),
        Some((_, 0)) => err(f.n, "empty hint"),
        Some((_, c)) if c > HINT_MAX_CHARS => err(f.n, format!("hint is {c} chars, max {HINT_MAX_CHARS}")),
        Some((t, _)) => Ok(Some(t)),
    }
}

fn lock<'a>(mut f: Fields<'a>, ids: &Ids<'a>) -> Result<Lock<'a>, ParseError> {
    let n = f.n;
    let hint = hint(&mut f)?;
    let id = f.word("lock id")?;
    let kind = match f.word("lock kind")? {
        "key" => LockKind::Key(ids.item(n, f.word("key item")?)?),
        "code" => {
            let c = f.word("code digits")?;
            match (3..=4).contains(&c.len()) && c.bytes().all(|b| b.is_ascii_digit()) {
                true => LockKind::Code {
                    digits: std::array::from_fn(|i| c.as_bytes().get(i).map_or(0, |b| b - b'0')),
                    len: c.len() as u8,
                },
                false => return err(n, format!("code must be 3 or 4 digits, got '{c}'")),
            }
        }
        "tool" => LockKind::Tool(f.pick("tool", TOOLS)?),
        "place" => {
            let item = ids.item(n, f.word("place item")?)?;
            let (x, y) = f.cell()?;
            LockKind::Place { item, x, y }
        }
        "all" => {
            let parts = f.toks[f.at..].iter().map(|l| ids.lock(n, l)).collect::<Result<Vec<_>, _>>()?;
            f.at = f.toks.len();
            match parts.is_empty() || parts.iter().any(|p| ids.locks[*p as usize] == id) {
                true => return err(n, "lock all needs other lock ids (not itself)"),
                false => LockKind::All(parts),
            }
        }
        other => return err(n, format!("unknown lock kind '{other}' (key|code|tool|place|all)")),
    };
    f.end(Lock { id, kind, hint })
}

fn door(mut f: Fields<'_>, ids: &Ids<'_>) -> Result<Door, ParseError> {
    let n = f.n;
    let (x, y) = f.cell()?;
    let axis = f.pick("axis", &[("h", Axis::H), ("v", Axis::V)])?;
    let state = match f.word("door state")? {
        "open" => DoorState::Open,
        "closed" => DoorState::Closed,
        "stuck" => DoorState::Stuck,
        s => match s.strip_prefix("locked:") {
            Some(l) => DoorState::Locked(ids.lock(n, l)?),
            None => return err(n, format!("unknown door state '{s}' (open|closed|stuck|locked:<lock>)")),
        },
    };
    f.end(Door { x, y, axis, state })
}

fn portal(mut f: Fields<'_>, ids: &Ids<'_>) -> Result<Portal, ParseError> {
    let n = f.n;
    let (x, y) = f.cell()?;
    if f.word("->")? != "->" {
        return err(n, "expected '->' between portal tiles");
    }
    let (to_x, to_y) = f.cell()?;
    let rot = f.rot()?;
    let kind = f.pick("portal kind", PORTALS)?;
    let lock = f.opt_key("lock").map(|l| ids.lock(n, l)).transpose()?;
    f.end(Portal { x, y, to_x, to_y, rot, kind, lock })
}

fn prop<'a>(mut f: Fields<'a>, ids: &Ids<'a>) -> Result<Prop<'a>, ParseError> {
    let n = f.n;
    let kind = f.word("prop kind")?;
    let pos = f.pos()?;
    let rot = f.rot()?;
    let k = Keys::new(n, &f.toks[f.at..], &["lock", "item"])?;
    f.at = f.toks.len();
    let lock = k.get("lock").map(|l| ids.lock(n, l)).transpose()?;
    let item = k.get("item").map(|i| ids.item(n, i)).transpose()?;
    f.end(Prop { kind, pos, rot, lock, item })
}

fn note<'a>(mut f: Fields<'a>) -> Result<Note<'a>, ParseError> {
    let (x, y) = f.cell()?;
    let target = f.word("note target")?;
    let text = f.text("note text")?;
    match text.chars().count() {
        0 => err(f.n, "empty note"),
        c if c > NOTE_MAX_CHARS => err(f.n, format!("note is {c} chars, max {NOTE_MAX_CHARS}")),
        _ => f.end(Note { x, y, target, text }),
    }
}

fn hide(mut f: Fields<'_>) -> Result<Hide, ParseError> {
    let (x, y) = f.cell()?;
    let kind = f.pick("hide kind", HIDES)?;
    let rot = f.rot()?;
    f.end(Hide { x, y, kind, rot })
}

fn light(mut f: Fields<'_>) -> Result<Light, ParseError> {
    let pos = f.pos()?;
    let h = f.fx("h")?;
    let rgb = rgb(f.n, f.word("r,g,b")?)?;
    let radius = f.fx("radius")?;
    let mode = f.pick("light mode", LIGHTS)?;
    f.end(Light { pos, h, rgb, radius, mode })
}

fn cam(mut f: Fields<'_>) -> Result<Cam, ParseError> {
    let area = f.rect()?;
    let eye = f.point3(["px", "ph", "pz"])?;
    let target = f.point3(["tx", "th", "tz"])?;
    let fov = f.fx("fov_deg")?;
    f.end(Cam { area, eye, target, fov })
}

fn wake(mut f: Fields<'_>) -> Result<(usize, u8, Spot), ParseError> {
    let n = f.n;
    let ch = f.int("character id")?;
    let spot = Spot { pos: f.spot()?, rot: f.rot()? };
    match (0..4).contains(&ch) {
        true => f.end((n, ch as u8, spot)),
        false => err(n, format!("character id must be 0..3, got {ch}")),
    }
}

enum LegendLine {
    Spawn(Spot),
    Patrol((i32, i32)),
    Lair((i32, i32)),
}

fn legend_line(mut f: Fields<'_>) -> Result<(usize, LegendLine), ParseError> {
    let n = f.n;
    let l = match f.word("legend line kind")? {
        "spawn" => LegendLine::Spawn(Spot { pos: f.spot()?, rot: f.rot()? }),
        "patrol" => LegendLine::Patrol(f.cell()?),
        "lair" => LegendLine::Lair(f.cell()?),
        other => return err(n, format!("unknown legend line '{other}' (spawn|patrol|lair)")),
    };
    f.end((n, l))
}

fn exit(mut f: Fields<'_>, ids: &Ids<'_>) -> Result<Exit, ParseError> {
    let n = f.n;
    let rect = f.rect()?;
    let lock = f.opt_key("lock").map_or_else(|| err(n, "exit needs lock=<lock_id>"), |l| ids.lock(n, l))?;
    f.end(Exit { rect, lock })
}

fn first_bad<T>(v: &[(usize, T)], bad: impl Fn(&T) -> Option<String>) -> Result<(), ParseError> {
    v.iter().find_map(|(n, t)| bad(t).map(|m| (*n, m))).map_or(Ok(()), |(n, m)| err(n, m))
}

fn numbered<'a, T>(src: &Src<'a>, s: Section, f: impl Fn(Fields<'a>) -> Result<T, ParseError>) -> Result<Vec<(usize, T)>, ParseError> {
    src.all(s, |fl| {
        let n = fl.n;
        f(fl).map(|t| (n, t))
    })
}

fn strip<T>(v: Vec<(usize, T)>) -> Vec<T> {
    v.into_iter().map(|(_, t)| t).collect()
}

pub fn parse(text: &str) -> Result<Map<'_>, ParseError> {
    let src = split(text)?;
    let last = src.last_line();
    let hd = header(&src)?;
    let areas: Vec<Area> = src.of(Section::Area).map(|(n, l)| Fields::new(n, l.trim()).and_then(|f| area(l.trim(), f))).collect::<Result<_, _>>()?;
    if areas.is_empty() {
        return err(last, "missing [area] lines (at least `0 <name>`)");
    }
    if let Some(((n, _), a)) = src.of(Section::Area).zip(&areas).enumerate().find(|(i, (_, a))| a.id as usize != *i).map(|(_, p)| p) {
        return err(n, format!("area numbers must be 0, 1, 2... in order; got {}", a.id));
    }
    let rooms: Vec<Room> = src.of(Section::Room).map(|(n, l)| Fields::new(n, l.trim()).and_then(|f| room(l.trim(), f, areas.len()))).collect::<Result<_, _>>()?;
    let room_lines: Vec<usize> = src.of(Section::Room).map(|(n, _)| n).collect();
    if let Some((i, r)) = rooms.iter().enumerate().find(|(i, r)| rooms[..*i].iter().any(|o| o.id == r.id)) {
        return err(room_lines[i], format!("duplicate room id '{}'", r.id as char));
    }
    if rooms.len() > 255 {
        return err(room_lines[255], "more than 255 rooms");
    }
    let solid: Vec<(usize, &str)> = src.of(Section::Solid).collect();
    let tiles = grid(&solid, hd.w, hd.h, "solid", last, tile_of)?;
    let room_rows: Vec<(usize, &str)> = src.of(Section::Rooms).collect();
    let given = grid(&room_rows, hd.w, hd.h, "rooms", last, |b| match b {
        b'-' | b' ' => Some(None),
        id => rooms.iter().position(|r| r.id == id).map(|i| Some(i as u8)),
    })?;
    let room_of = door_rooms(&tiles, given, hd.w);
    let ids = Ids { items: unique(&src, Section::Item, "item id")?, locks: unique(&src, Section::Lock, "lock id")? };
    let mut items = src.all(Section::Item, item)?;
    let locks = numbered(&src, Section::Lock, |f| lock(f, &ids))?;
    let doors = numbered(&src, Section::Door, |f| door(f, &ids))?;
    let portals = numbered(&src, Section::Portal, |f| portal(f, &ids))?;
    let props = numbered(&src, Section::Prop, |f| prop(f, &ids))?;
    let notes = numbered(&src, Section::Note, note)?;
    let hides = numbered(&src, Section::Hide, hide)?;
    let lights = src.all(Section::Light, light)?;
    let cams = src.all(Section::Cam, cam)?;
    let wakes = src.all(Section::Wake, wake)?;
    let legend = src.all(Section::Legend, legend_line)?;
    let exits = numbered(&src, Section::Exit, |f| exit(f, &ids))?;
    if props.len() > u16::MAX as usize {
        return err(props[u16::MAX as usize].0, "too many props");
    }
    if let Some((i, (n, p))) = props.iter().enumerate().find(|(i, (_, p))| p.item.is_some() && props[..*i].iter().any(|(_, o)| o.item == p.item)) {
        return err(*n, format!("item '{}' is inside two props (this is prop #{i})", ids.items[p.item.unwrap_or(0) as usize]));
    }
    props.iter().enumerate().for_each(|(i, (_, p))| {
        if let Some(it) = p.item {
            items[it as usize].inside = Some(i as u16);
        }
    });
    let wake_spots: [Option<Spot>; 4] = std::array::from_fn(|c| wakes.iter().find(|(_, ch, _)| *ch as usize == c).map(|(_, _, s)| *s));
    if let Some((n, ch, _)) = wakes.iter().enumerate().find(|(i, (_, ch, _))| wakes[..*i].iter().any(|(_, o, _)| o == ch)).map(|(_, w)| w) {
        return err(*n, format!("duplicate wake for character {ch}"));
    }
    if let Some(c) = wake_spots.iter().position(Option::is_none) {
        return err(last, format!("missing [wake] for character {c}"));
    }
    let spawns: Vec<Spot> = legend.iter().filter_map(|(_, l)| match l { LegendLine::Spawn(s) => Some(*s), _ => None }).collect();
    let patrol: Vec<(i32, i32)> = legend.iter().filter_map(|(_, l)| match l { LegendLine::Patrol(p) => Some(*p), _ => None }).collect();
    let lairs: Vec<(usize, (i32, i32))> = legend.iter().filter_map(|(n, l)| match l { LegendLine::Lair(p) => Some((*n, *p)), _ => None }).collect();
    if spawns.is_empty() {
        return err(last, "[legend] needs at least one spawn");
    }
    if patrol.len() < 6 {
        return err(last, format!("[legend] needs at least 6 patrol points, has {}", patrol.len()));
    }
    let lair = match lairs.as_slice() {
        [(_, p)] => *p,
        [] => return err(last, "[legend] needs a lair"),
        [_, (n, _), ..] => return err(*n, "[legend] has more than one lair"),
    };
    let exit = match exits.as_slice() {
        [(_, e)] => *e,
        [] => return err(last, "missing [exit]"),
        [_, (n, _), ..] => return err(*n, "more than one [exit]"),
    };
    let map = Map {
        name: hd.name,
        w: hd.w,
        h: hd.h,
        sky: hd.sky,
        fog: hd.fog,
        fog_near: hd.fog_near,
        fog_far: hd.fog_far,
        legend: hd.legend,
        power: hd.power,
        tiles,
        room_of,
        rooms,
        areas,
        doors: Vec::new(),
        portals: Vec::new(),
        locks: Vec::new(),
        items,
        notes: Vec::new(),
        hides: Vec::new(),
        props: Vec::new(),
        lights,
        cams,
        wakes: wake_spots.map(|s| s.unwrap_or(Spot { pos: Vec2::new(Fx::ZERO, Fx::ZERO), rot: Ang(0) })),
        legend_spawns: spawns,
        patrol,
        lair,
        exit,
    };
    let at = |x: i32, y: i32| map.tile(x, y);
    let dup = |v: &[(i32, i32)], i: usize| v[..i].contains(&v[i]);
    let door_cells: Vec<(i32, i32)> = doors.iter().map(|(_, d)| (d.x, d.y)).collect();
    first_bad(&doors.iter().enumerate().map(|(i, (n, d))| (*n, (i, *d))).collect::<Vec<_>>(), |(i, d)| match (at(d.x, d.y), dup(&door_cells, *i)) {
        (Tile::Door, false) => None,
        (Tile::Door, true) => Some(format!("two doors at ({}, {})", d.x, d.y)),
        _ => Some(format!("door at ({}, {}) is not on a 'D' tile", d.x, d.y)),
    })?;
    let portal_cells: Vec<(i32, i32)> = portals.iter().map(|(_, p)| (p.x, p.y)).collect();
    first_bad(&portals.iter().enumerate().map(|(i, (n, p))| (*n, (i, *p))).collect::<Vec<_>>(), |(i, p)| {
        match (at(p.x, p.y), at(p.to_x, p.to_y).walkable(), dup(&portal_cells, *i)) {
            (Tile::Portal, true, false) => None,
            (Tile::Portal, false, _) => Some(format!("portal target ({}, {}) is not walkable", p.to_x, p.to_y)),
            (Tile::Portal, true, true) => Some(format!("two portals at ({}, {})", p.x, p.y)),
            _ => Some(format!("portal at ({}, {}) is not on a 'P' tile", p.x, p.y)),
        }
    })?;
    let hide_cells: Vec<(i32, i32)> = hides.iter().map(|(_, h)| (h.x, h.y)).collect();
    first_bad(&hides.iter().enumerate().map(|(i, (n, h))| (*n, (i, *h))).collect::<Vec<_>>(), |(i, h)| match (at(h.x, h.y), dup(&hide_cells, *i)) {
        (Tile::Hide | Tile::Blocker, false) => None,
        (Tile::Hide | Tile::Blocker, true) => Some(format!("two hide spots at ({}, {})", h.x, h.y)),
        _ => Some(format!("hide at ({}, {}) is not on an 'H' (or 'B' under a prop) tile", h.x, h.y)),
    })?;
    first_bad(&notes, |n| map.index(n.x, n.y).is_none().then(|| format!("note at ({}, {}) is outside the map", n.x, n.y)))?;
    let row_line = |y: i32| solid[y as usize].0;
    let cells = (0..map.h).flat_map(|y| (0..map.w).map(move |x| (x, y)));
    let needs_room = |t: Tile| t.walkable() || t == Tile::Hide;
    if let Some((x, y)) = cells.clone().find(|&(x, y)| needs_room(at(x, y)) && map.room_at(x, y).is_none()) {
        return err(row_line(y), format!("{:?} tile ({x}, {y}) has no room", at(x, y)));
    }
    if let Some((x, y)) = cells.clone().find(|&(x, y)| at(x, y).walkable() && map.cam_at(x, y).is_none()) {
        return err(row_line(y), format!("walkable tile ({x}, {y}) not covered by any [cam]"));
    }
    Ok(Map {
        doors: strip(doors),
        portals: strip(portals),
        locks: strip(locks),
        notes: strip(notes),
        hides: strip(hides),
        props: strip(props),
        ..map
    })
}
