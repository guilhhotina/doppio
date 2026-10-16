#!/usr/bin/env python3
import math
import os
import re
import sys
from collections import deque

SECTIONS = ("map", "solid", "rooms", "room", "area", "door", "portal", "lock", "item", "note", "hide", "prop",
            "light", "cam", "wake", "legend", "exit")
GRIDS = ("solid", "rooms")
TILES = "#.DWBHP~| "
WALK = ".DP"
LOS_BLOCK = "#W "
SKIES = ("night", "moon", "overcast", "none")
LEGENDS = ("corposeco", "cabra", "mula", "matinta")
ITEMS = ("chave", "alicate", "martelo", "pe_de_cabra", "chave_de_fenda", "fusivel", "sal", "vela", "fosforo",
         "tabaco", "balde", "freio", "ossos", "cruz", "bilhete_onibus", "manivela", "remo", "lanterna_pilha", "foto")
TOOLS = ("alicate", "martelo", "pe_de_cabra", "chave_de_fenda")
PORTALS = ("escada", "escada_mao", "bueiro", "portao", "janela_alta")
HIDES = ("cama", "guarda_roupa", "armario", "tumulo", "caixa_dagua", "confessionario", "carro", "cesto", "mesa")
LIGHTS = ("steady", "flicker", "dead")
PROPS = set(re.findall(r'"([a-z_]+)"', re.search(
    r"pub const KINDS: &\[&str\] = &\[(.*?)\];",
    open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "src", "render", "props.rs")).read(),
    re.S).group(1)))
NOTE_MAX = 120
MAX_ITEMS = 64
MAX_DOORS = 96
MAX_ROOMS = 128


class Bad(Exception):
    pass


def tokens(line):
    return re.findall(r'(?:[^\s"]|"[^"]*")+', line)


def unquote(s):
    return s[1:-1] if len(s) >= 2 and s[0] == s[-1] == '"' else s


def read_sections(text):
    out = {s: [] for s in SECTIONS}
    cur = None
    for n, raw in enumerate(text.split("\n"), 1):
        line = raw.rstrip("\r")
        t = line.strip()
        if t.startswith(";") or not line or (not t and cur not in GRIDS):
            continue
        m = re.match(r"^\[(\w+)\](.*)$", t)
        if m:
            if m.group(1) not in SECTIONS:
                raise Bad("line %d: unknown section [%s]" % (n, m.group(1)))
            cur = m.group(1)
            if m.group(2).strip():
                out[cur].append((n, m.group(2).strip()))
            continue
        if cur is None:
            raise Bad("line %d: data before any [section]" % n)
        out[cur].append((n, line if cur in GRIDS else t))
    return out


def fnum(tok):
    if not re.fullmatch(r"-?\d+(\.\d)?", tok):
        raise ValueError(tok)
    return float(tok)


def inum(tok):
    if not re.fullmatch(r"-?\d+", tok):
        raise ValueError(tok)
    return int(tok)


class Map:
    pass


def keyvals(toks, allowed, n, errors):
    kv = {}
    for t in toks:
        k, _, v = t.partition("=")
        if k not in allowed or not v:
            errors.append("line %d: unexpected %r" % (n, t))
        elif k in kv:
            errors.append("line %d: duplicate %s=" % (n, k))
        else:
            kv[k] = unquote(v)
    return kv


def load(path, texdir):
    errors = []
    with open(path, encoding="utf-8") as f:
        sec = read_sections(f.read())
    m = Map()
    m.errors = errors
    m.warnings = []
    head = [t for _, l in sec["map"] for t in tokens(l)]
    hn = sec["map"][0][0] if sec["map"] else 0
    if not sec["map"]:
        raise Bad("missing [map] header")
    kv = keyvals(head, ("name", "w", "h", "sky", "fog", "fog_near", "fog_far", "legend", "power"), hn, errors)
    for k in ("name", "w", "h", "sky", "fog", "legend"):
        if k not in kv:
            errors.append("line %d: [map] missing %s=" % (hn, k))
    m.name = kv.get("name", "?")
    try:
        m.w, m.h = int(kv.get("w", 0)), int(kv.get("h", 0))
    except ValueError:
        raise Bad("line %d: w=/h= must be integers" % hn)
    if kv.get("sky") not in SKIES:
        errors.append("line %d: sky=%s not in %s" % (hn, kv.get("sky"), "|".join(SKIES)))
    if kv.get("legend") not in LEGENDS:
        errors.append("line %d: legend=%s not in %s" % (hn, kv.get("legend"), "|".join(LEGENDS)))
    if kv.get("power", "on") not in ("on", "off"):
        errors.append("line %d: power= must be on|off" % hn)
    m.power = kv.get("power", "on") == "on"
    if not re.fullmatch(r"\d+,\d+,\d+", kv.get("fog", "")) or any(int(c) > 255 for c in kv.get("fog", "0,0,0").split(",")):
        errors.append("line %d: fog= must be r,g,b" % hn)
    fn, ff = (float(kv.get(k, d)) for k, d in (("fog_near", 4), ("fog_far", 16)))
    if not 0 <= fn < ff:
        errors.append("line %d: fog_near must be < fog_far" % hn)

    solid, rooms = sec["solid"], sec["rooms"]
    for name, rows in (("solid", solid), ("rooms", rooms)):
        if len(rows) != m.h:
            errors.append("[%s] has %d rows, [map] h=%d" % (name, len(rows), m.h))
        for n, r in rows:
            if len(r) > m.w:
                errors.append("line %d: [%s] row is %d wide, [map] w=%d" % (n, name, len(r), m.w))
    m.solid = [r.ljust(m.w)[:m.w] for _, r in solid][:m.h] + [" " * m.w] * max(0, m.h - len(solid))
    m.rooms = [r.ljust(m.w)[:m.w] for _, r in rooms][:m.h] + [" " * m.w] * max(0, m.h - len(rooms))
    m.solid_line = [n for n, _ in solid]
    for (n, r) in solid:
        bad = sorted(set(r) - set(TILES))
        if bad:
            errors.append("line %d: [solid] unknown chars %s" % (n, bad))
    m.solid = [re.sub("[^" + re.escape(TILES) + "]", " ", r) for r in m.solid]

    m.areas = {}
    for i, (n, l) in enumerate(sec["area"]):
        t = l.split(None, 1)
        if len(t) != 2 or not t[0].isdigit() or int(t[0]) != i:
            errors.append("line %d: [area] lines must be '<n> <name>' numbered 0,1,2.. in order" % n)
            continue
        m.areas[i] = t[1]
    if not m.areas:
        errors.append("missing [area] lines")

    m.room_defs, m.room_line = {}, {}
    for n, l in sec["room"]:
        t = tokens(l)
        if not t or len(t[0]) != 1 or t[0] == "-":
            errors.append("line %d: room id must be one character" % n)
            continue
        names = [w for w in t[1:] if "=" not in w]
        kv = keyvals([w for w in t[1:] if "=" in w], ("area", "floor", "wall", "ceil", "ext", "h", "light", "out"), n, errors)
        if not names:
            errors.append("line %d: room %s has no name" % (n, t[0]))
        for k in ("floor", "wall", "ceil", "light"):
            if k not in kv:
                errors.append("line %d: room %s missing %s=" % (n, t[0], k))
        r = {"name": " ".join(names), "area": 0, "h": 2.6, "out": False, "ext": None, "ceil": None, "light": 0}
        try:
            r["area"] = int(kv.get("area", 0))
            r["h"] = float(kv.get("h", 2.6))
            r["light"] = int(kv.get("light", 0))
        except ValueError:
            errors.append("line %d: room %s bad number" % (n, t[0]))
        if r["area"] not in m.areas:
            errors.append("line %d: room %s area=%d has no [area] line" % (n, t[0], r["area"]))
        if not 0 <= r["light"] <= 100:
            errors.append("line %d: room %s light out of 0..100" % (n, t[0]))
        if kv.get("out", "0") not in ("0", "1"):
            errors.append("line %d: room %s out= must be 0|1" % (n, t[0]))
        r["out"] = kv.get("out") == "1"
        r["ceil"] = None if kv.get("ceil", "-") == "-" else kv["ceil"]
        r["ext"] = None if kv.get("ext", "-") == "-" else kv["ext"]
        for k in ("floor", "wall", "ceil", "ext"):
            tex = kv.get(k, "-")
            if tex != "-" and not os.path.isfile(os.path.join(texdir, tex + ".tex")):
                errors.append("line %d: room %s %s=%s: missing assets/tex/%s.tex" % (n, t[0], k, tex, tex))
        if t[0] in m.room_defs:
            errors.append("line %d: duplicate room id %r" % (n, t[0]))
        m.room_defs[t[0]] = r
        m.room_line[t[0]] = n
    if len(m.room_defs) > MAX_ROOMS:
        errors.append("%d rooms, renderer max %d" % (len(m.room_defs), MAX_ROOMS))
    used = {c for r in m.rooms for c in r} - {"-", " "}
    for c in sorted(used - set(m.room_defs)):
        errors.append("[rooms] uses id %r with no [room] line" % c)
    for c in sorted(set(m.room_defs) - used):
        errors.append("line %d: room %s defined but not used in [rooms]" % (m.room_line[c], c))

    def room_of(x, y):
        if not (0 <= x < m.w and 0 <= y < m.h):
            return None
        c = m.rooms[y][x]
        if c not in "- ":
            return c
        if m.solid[y][x] == "D":
            for nx, ny in ((x, y - 1), (x - 1, y), (x + 1, y), (x, y + 1)):
                if 0 <= nx < m.w and 0 <= ny < m.h and m.solid[ny][nx] != "D" and m.rooms[ny][nx] not in "- ":
                    return m.rooms[ny][nx]
        return None
    m.room_of = room_of
    m.tile = lambda x, y: m.solid[y][x] if 0 <= x < m.w and 0 <= y < m.h else " "
    m.area_of = lambda x, y: m.room_defs.get(room_of(x, y), {}).get("area") if room_of(x, y) else None
    m.walk = {(x, y) for y in range(m.h) for x in range(m.w) if m.solid[y][x] in WALK}
    for y in range(m.h):
        for x in range(m.w):
            c = m.solid[y][x]
            if c in WALK + "H" and room_of(x, y) is None:
                errors.append("line %d: %r tile %d,%d has no room" % (m.solid_line[y] if y < len(m.solid_line) else 0, c, x, y))
            if c in "#W " and m.rooms[y][x] not in "- ":
                errors.append("wall/void tile %d,%d has room id %r" % (x, y, m.rooms[y][x]))

    m.items, m.item_line = {}, {}
    for n, l in sec["item"]:
        t = tokens(l)
        try:
            if len(t) != 4 or t[1] not in ITEMS:
                raise ValueError(l)
            m.items[t[0]] = {"kind": t[1], "x": fnum(t[2]), "y": fnum(t[3]), "inside": None, "line": n}
        except ValueError:
            errors.append("line %d: [item] needs '<id> <kind> <x> <y>' with kind in %s" % (n, "|".join(ITEMS)))
            continue
        if t[0] in m.item_line:
            errors.append("line %d: duplicate item id %r" % (n, t[0]))
        m.item_line[t[0]] = n
    if len(m.items) > MAX_ITEMS:
        errors.append("%d items, sim max %d" % (len(m.items), MAX_ITEMS))

    m.locks = {}
    lock_ids = [tokens(l)[0] for _, l in sec["lock"] if tokens(l)]
    for n, l in sec["lock"]:
        t = tokens(l)
        if len(t) < 3:
            errors.append("line %d: [lock] needs '<id> <kind> ...'" % n)
            continue
        lid, kind, args = t[0], t[1], t[2:]
        if lid in m.locks:
            errors.append("line %d: duplicate lock id %r" % (n, lid))
        lk = {"kind": kind, "line": n, "users": []}
        if kind == "key" and len(args) == 1 and args[0] in m.items:
            lk["item"] = args[0]
        elif kind == "code" and len(args) == 1 and re.fullmatch(r"\d{3,4}", args[0]):
            lk["code"] = args[0]
        elif kind == "tool" and len(args) == 1 and args[0] in TOOLS:
            lk["tool"] = args[0]
        elif kind == "place" and len(args) == 3 and args[0] in m.items and re.fullmatch(r"\d+", args[1]) and re.fullmatch(r"\d+", args[2]):
            lk["item"], lk["at"] = args[0], (int(args[1]), int(args[2]))
        elif kind == "all" and args and all(a in lock_ids and a != lid for a in args):
            lk["parts"] = args
        else:
            errors.append("line %d: bad lock %r (key <item>|code <3-4 digits>|tool <%s>|place <item> <x> <y>|all <locks>)" % (n, l, "|".join(TOOLS)))
            continue
        m.locks[lid] = lk
    for lid, lk in m.locks.items():
        if lk["kind"] == "place" and m.tile(*lk["at"]) not in WALK + "B#W":
            errors.append("line %d: place spot %d,%d is not a reachable tile" % (lk["line"], *lk["at"]))

    def lockref(v, n, user):
        if v not in m.locks:
            errors.append("line %d: unknown lock %r" % (n, v))
            return None
        m.locks[v]["users"].append(user)
        return v

    m.doors = {}
    for n, l in sec["door"]:
        t = tokens(l)
        try:
            x, y = inum(t[0]), inum(t[1])
            if len(t) != 4 or t[2] not in ("h", "v"):
                raise ValueError
        except (ValueError, IndexError):
            errors.append("line %d: [door] needs '<x> <y> <h|v> <state>'" % n)
            continue
        st = t[3]
        lock = None
        if st.startswith("locked:"):
            lock = lockref(st[7:], n, ("door", (x, y)))
        elif st not in ("open", "closed", "stuck"):
            errors.append("line %d: door state %r (open|closed|stuck|locked:<lock>)" % (n, st))
        if m.tile(x, y) != "D":
            errors.append("line %d: door %d,%d not on a D tile" % (n, x, y))
        if t[2] == "h" and not (m.tile(x - 1, y) in "#W" and m.tile(x + 1, y) in "#W"):
            errors.append("line %d: door %d,%d axis h but wall does not run along x" % (n, x, y))
        if t[2] == "v" and not (m.tile(x, y - 1) in "#W" and m.tile(x, y + 1) in "#W"):
            errors.append("line %d: door %d,%d axis v but wall does not run along y" % (n, x, y))
        if (x, y) in m.doors:
            errors.append("line %d: door %d,%d listed twice" % (n, x, y))
        a, b = ((x, y - 1), (x, y + 1)) if t[2] == "h" else ((x - 1, y), (x + 1, y))
        ra, rb = m.rooms[a[1]][a[0]] if m.tile(*a) != " " else "-", m.rooms[b[1]][b[0]] if m.tile(*b) != " " else "-"
        if "-" in (ra, rb) or " " in (ra, rb) or ra == rb:
            errors.append("line %d: door %d,%d does not join two different rooms" % (n, x, y))
        elif m.room_defs.get(ra, {}).get("area") != m.room_defs.get(rb, {}).get("area"):
            errors.append("line %d: door %d,%d joins two areas (use a portal)" % (n, x, y))
        m.doors[(x, y)] = {"axis": t[2], "state": "locked" if lock else st, "lock": lock, "line": n, "rooms": (ra, rb), "sides": (a, b)}
    for y in range(m.h):
        for x in range(m.w):
            if m.solid[y][x] == "D" and (x, y) not in m.doors:
                errors.append("D tile %d,%d has no [door] line" % (x, y))
    if len(m.doors) > MAX_DOORS:
        errors.append("%d doors, sim max %d" % (len(m.doors), MAX_DOORS))

    m.portals = {}
    for n, l in sec["portal"]:
        t = tokens(l)
        try:
            if len(t) not in (7, 8) or t[2] != "->" or t[6] not in PORTALS:
                raise ValueError
            x, y, tx, ty, rot = inum(t[0]), inum(t[1]), inum(t[3]), inum(t[4]), fnum(t[5])
        except ValueError:
            errors.append("line %d: [portal] needs '<x> <y> -> <x2> <y2> <rot> <%s> [lock=<id>]'" % (n, "|".join(PORTALS)))
            continue
        lock = None
        if len(t) == 8:
            if not t[7].startswith("lock="):
                errors.append("line %d: unexpected %r" % (n, t[7]))
            else:
                lock = lockref(t[7][5:], n, ("portal", (x, y)))
        if m.tile(x, y) != "P":
            errors.append("line %d: portal %d,%d not on a P tile" % (n, x, y))
        if m.tile(tx, ty) not in ".D":
            errors.append("line %d: portal target %d,%d is not a walkable non-portal tile" % (n, tx, ty))
        if (x, y) in m.portals:
            errors.append("line %d: two portals at %d,%d" % (n, x, y))
        m.portals[(x, y)] = {"to": (tx, ty), "rot": rot, "kind": t[6], "lock": lock, "line": n}
    for y in range(m.h):
        for x in range(m.w):
            if m.solid[y][x] == "P" and (x, y) not in m.portals:
                errors.append("P tile %d,%d has no [portal] line" % (x, y))

    m.props = []
    for n, l in sec["prop"]:
        t = tokens(l)
        try:
            if len(t) < 4 or t[0] not in PROPS:
                raise ValueError
            x, y, rot = fnum(t[1]), fnum(t[2]), fnum(t[3])
        except ValueError:
            errors.append("line %d: [prop] needs '<kind> <x> <y> <rot> [lock=] [item=]' with a catalog kind" % n)
            continue
        kv = keyvals(t[4:], ("lock", "item"), n, errors)
        p = {"kind": t[0], "x": x, "y": y, "rot": rot, "lock": None, "item": None, "line": n, "i": len(m.props)}
        if "lock" in kv:
            p["lock"] = lockref(kv["lock"], n, ("prop", len(m.props)))
        if "item" in kv:
            if kv["item"] not in m.items:
                errors.append("line %d: unknown item %r" % (n, kv["item"]))
            elif m.items[kv["item"]]["inside"] is not None:
                errors.append("line %d: item %r is already inside another prop" % (n, kv["item"]))
            else:
                p["item"] = kv["item"]
                m.items[kv["item"]]["inside"] = len(m.props)
        if m.tile(int(x), int(y)) == " ":
            errors.append("line %d: prop %s at %g,%g stands in the void" % (n, t[0], x, y))
        m.props.append(p)

    def neighbours(x, y):
        return ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1))

    def cell_of(p):
        return int(p["x"]), int(p["y"])

    def near_walk(x, y):
        return (x, y) in m.walk or any(q in m.walk for q in neighbours(x, y))

    for p in m.props:
        if (p["item"] or p["lock"]) and not near_walk(*cell_of(p)):
            errors.append("line %d: container %s at %g,%g has no walkable tile at or next to it" % (p["line"], p["kind"], p["x"], p["y"]))
    for iid, it in m.items.items():
        c = (int(it["x"]), int(it["y"]))
        if it["inside"] is not None:
            p = m.props[it["inside"]]
            if abs(p["x"] - it["x"]) > 1.5 or abs(p["y"] - it["y"]) > 1.5:
                errors.append("line %d: item %s is inside %s at %g,%g but listed at %g,%g" % (it["line"], iid, p["kind"], p["x"], p["y"], it["x"], it["y"]))
        elif not (c in m.walk or (m.tile(*c) == "B" and near_walk(*c))):
            errors.append("line %d: item %s at %g,%g is not on a walkable tile (or a B tile next to one)" % (it["line"], iid, it["x"], it["y"]))

    m.notes = []
    for n, l in sec["note"]:
        t = tokens(l)
        try:
            if len(t) != 4 or not (t[3].startswith('"') and t[3].endswith('"') and len(t[3]) >= 2):
                raise ValueError
            x, y = inum(t[0]), inum(t[1])
        except ValueError:
            errors.append("line %d: [note] needs '<x> <y> <target> \"text\"'" % n)
            continue
        text = t[3][1:-1]
        if not text or len(text) > NOTE_MAX:
            errors.append("line %d: note text is %d chars (1..%d)" % (n, len(text), NOTE_MAX))
        if not near_walk(x, y):
            errors.append("line %d: note at %d,%d cannot be reached (no walkable tile at or next to it)" % (n, x, y))
        m.notes.append({"x": x, "y": y, "target": t[2], "text": text, "line": n})

    m.hides = {}
    for n, l in sec["hide"]:
        t = tokens(l)
        try:
            if len(t) != 4 or t[2] not in HIDES:
                raise ValueError
            x, y, rot = inum(t[0]), inum(t[1]), fnum(t[3])
        except ValueError:
            errors.append("line %d: [hide] needs '<x> <y> <%s> <rot>'" % (n, "|".join(HIDES)))
            continue
        c = m.tile(x, y)
        if c == "B" and not any(cell_of(p) == (x, y) or (abs(p["x"] - x - 0.5) <= 1.0 and abs(p["y"] - y - 0.5) <= 1.0) for p in m.props):
            errors.append("line %d: hide on B tile %d,%d needs a prop over it (or use an H tile)" % (n, x, y))
        if c not in "HB":
            errors.append("line %d: hide %d,%d not on an H (or B under a prop) tile" % (n, x, y))
        if not any(q in m.walk for q in neighbours(x, y)):
            errors.append("line %d: hide %d,%d has no walkable neighbour to enter from" % (n, x, y))
        if (x, y) in m.hides:
            errors.append("line %d: two hides at %d,%d" % (n, x, y))
        m.hides[(x, y)] = {"kind": t[2], "rot": rot, "line": n}
    for y in range(m.h):
        for x in range(m.w):
            if m.solid[y][x] == "H" and (x, y) not in m.hides:
                errors.append("H tile %d,%d has no [hide] line" % (x, y))

    m.lights = []
    for n, l in sec["light"]:
        t = tokens(l)
        try:
            if len(t) != 6 or t[5] not in LIGHTS or not re.fullmatch(r"\d+,\d+,\d+", t[3]):
                raise ValueError
            x, y, h, r = fnum(t[0]), fnum(t[1]), fnum(t[2]), fnum(t[4])
        except ValueError:
            errors.append("line %d: [light] needs '<x> <y> <h> <r,g,b> <radius> <%s>'" % (n, "|".join(LIGHTS)))
            continue
        if not (0 <= x < m.w and 0 <= y < m.h) or r <= 0:
            errors.append("line %d: light outside the map or radius <= 0" % n)
        m.lights.append((x, y, h, r, t[5]))

    m.cams = []
    for n, l in sec["cam"]:
        t = tokens(l)
        try:
            if len(t) != 11:
                raise ValueError
            r = [inum(v) for v in t[:4]]
            v = [fnum(s) for s in t[4:]]
        except ValueError:
            errors.append("line %d: [cam] needs '<x0> <y0> <x1> <y1> <px> <ph> <pz> <tx> <th> <tz> <fov>'" % n)
            continue
        if r[0] > r[2] or r[1] > r[3]:
            errors.append("line %d: cam rect inverted" % n)
        if not 10 <= v[6] <= 120:
            errors.append("line %d: cam fov %g" % (n, v[6]))
        if (v[0], v[1], v[2]) == (v[3], v[4], v[5]):
            errors.append("line %d: cam eye equals target" % n)
        ex, ez = int(v[0]), int(v[2])
        if m.tile(ex, ez) not in WALK + "BH|~" and v[1] <= 2.6:
            errors.append("line %d: cam eye %.1f,%.1f inside wall/void tile %r" % (n, v[0], v[2], m.tile(ex, ez)))
        m.cams.append({"rect": r, "v": v, "line": n})

    m.wakes = {}
    for n, l in sec["wake"]:
        t = tokens(l)
        try:
            if len(t) != 4:
                raise ValueError
            ch, x, y, rot = inum(t[0]), fnum(t[1]), fnum(t[2]), fnum(t[3])
        except ValueError:
            errors.append("line %d: [wake] needs '<character 0..3> <x> <y> <rot>'" % n)
            continue
        if ch not in range(4) or ch in m.wakes:
            errors.append("line %d: wake character %d out of 0..3 or duplicated" % (n, ch))
        if m.tile(int(x), int(y)) != ".":
            errors.append("line %d: wake %g,%g not on a '.' floor tile" % (n, x, y))
        m.wakes[ch] = (int(x), int(y))
    if sorted(m.wakes) != [0, 1, 2, 3]:
        errors.append("[wake] needs exactly characters 0,1,2,3")
    if len(set(m.wakes.values())) != len(m.wakes):
        errors.append("[wake] spots share a tile")

    m.spawns, m.patrol, m.lair = [], [], None
    for n, l in sec["legend"]:
        t = tokens(l)
        try:
            if t[0] == "spawn" and len(t) == 4:
                m.spawns.append((int(fnum(t[1])), int(fnum(t[2]))))
                fnum(t[3])
            elif t[0] == "patrol" and len(t) == 3:
                m.patrol.append((inum(t[1]), inum(t[2])))
            elif t[0] == "lair" and len(t) == 3 and m.lair is None:
                m.lair = (inum(t[1]), inum(t[2]))
            else:
                raise ValueError
        except (ValueError, IndexError):
            errors.append("line %d: [legend] needs 'spawn <x> <y> <rot>' | 'patrol <x> <y>' | one 'lair <x> <y>'" % n)
            continue
        p = m.spawns[-1] if t[0] == "spawn" else m.patrol[-1] if t[0] == "patrol" else m.lair
        if m.tile(*p) not in ".D":
            errors.append("line %d: legend %s %d,%d not on a walkable non-portal tile" % (n, t[0], *p))
    if not m.spawns:
        errors.append("[legend] needs a spawn")
    if len(m.patrol) < 6:
        errors.append("[legend] needs >= 6 patrol points, has %d" % len(m.patrol))
    if m.lair is None:
        errors.append("[legend] needs a lair")

    m.exit = None
    for n, l in sec["exit"]:
        t = tokens(l)
        try:
            if len(t) != 5 or not t[4].startswith("lock=") or m.exit is not None:
                raise ValueError
            r = [inum(v) for v in t[:4]]
        except ValueError:
            errors.append("line %d: one [exit] '<x0> <y0> <x1> <y1> lock=<id>'" % n)
            continue
        lock = lockref(t[4][5:], n, ("exit", None))
        m.exit = {"rect": r, "lock": lock, "line": n}
        if not any(r[0] <= x <= r[2] and r[1] <= y <= r[3] for x, y in m.walk):
            errors.append("line %d: exit rect has no walkable tile" % n)
    if m.exit is None:
        errors.append("missing [exit]")
    exit_tree, todo = set(), [m.exit["lock"]] if m.exit and m.exit["lock"] else []
    while todo:
        lid = todo.pop()
        if lid not in exit_tree and lid in m.locks:
            exit_tree.add(lid)
            todo += m.locks[lid].get("parts", [])
    parts_used = {p for lk in m.locks.values() for p in lk.get("parts", ())}
    for lid, lk in m.locks.items():
        if lk["users"] or lk["kind"] == "place" or (lk["kind"] == "all" and lid in parts_used):
            continue
        if lid not in exit_tree:
            errors.append("line %d: lock %s has no door/portal/prop and is not part of the exit lock: nowhere to operate it" % (lk["line"], lid))
    spots = {}
    for k, cells in (("hide", m.hides), ("door", m.doors), ("portal", m.portals)):
        for c in cells:
            spots.setdefault(c, []).append(k)
    for p in m.props:
        if p["item"] or p["lock"]:
            spots.setdefault(cell_of(p), []).append("container")
    for lk in m.locks.values():
        if lk["kind"] == "place":
            spots.setdefault(lk["at"], []).append("place spot")
    for nt in m.notes:
        if (nt["x"], nt["y"]) in spots:
            errors.append("line %d: note at %d,%d shares its tile with a %s (focus ambiguity)" % (nt["line"], nt["x"], nt["y"], spots[(nt["x"], nt["y"])][0]))
    for c, ks in spots.items():
        if len(ks) > 1:
            errors.append("tile %d,%d hosts several interactive things %s (focus ambiguity)" % (c[0], c[1], ks))
    consumed = {}
    for lid, lk in m.locks.items():
        if lk["kind"] in ("key", "place"):
            consumed.setdefault(lk["item"], []).append(lid)
    for iid, lids in consumed.items():
        if len(lids) > 1:
            errors.append("item %s is consumed by several locks %s (key/place items are used up)" % (iid, lids))
    return m


def cam_checks(m):
    errors = m.errors
    owner = {}
    for (x, y) in sorted(m.walk):
        hit = next((i for i, c in enumerate(m.cams) if c["rect"][0] <= x <= c["rect"][2] and c["rect"][1] <= y <= c["rect"][3]), None)
        if hit is None:
            errors.append("walkable tile %d,%d (room %s) not covered by any cam" % (x, y, m.room_of(x, y)))
        else:
            owner[(x, y)] = hit
    for i, c in enumerate(m.cams):
        mine = [t for t, o in owner.items() if o == i]
        if not mine:
            errors.append("line %d: cam never selected (shadowed by earlier rects or no walkable tile)" % c["line"])
        areas = {m.area_of(*t) for t in mine}
        if len(areas) > 1:
            errors.append("line %d: cam covers tiles of several areas %s" % (c["line"], sorted(a for a in areas if a is not None)))

    def cells(ax, az, bx, bz):
        steps = int(math.hypot(bx - ax, bz - az) * 10) + 1
        return [(int(math.floor(ax + (bx - ax) * s / steps)), int(math.floor(az + (bz - az) * s / steps))) for s in range(0, steps)]

    for (x, y), i in sorted(owner.items()):
        v = m.cams[i]["v"]
        line = m.cams[i]["line"]
        path = cells(v[0], v[2], x + 0.5, y + 0.5)
        area = m.area_of(x, y)
        blocked = [c for c in path[1:] if m.tile(*c) in LOS_BLOCK]
        if blocked:
            errors.append("line %d: cam cannot see its tile %d,%d (wall at %d,%d)" % (line, x, y, *blocked[0]))
        if any(m.tile(*c) == "D" and c != (x, y) for c in path):
            errors.append("line %d: cam looks at tile %d,%d through a door that can close" % (line, x, y))
        other = [c for c in path if c in m.walk and m.area_of(*c) not in (None, area)]
        if other:
            errors.append("line %d: cam LOS to %d,%d crosses area %s tiles" % (line, x, y, m.area_of(*other[0])))
        dx, dz = v[3] - v[0], v[5] - v[2]
        vx, vz = x + 0.5 - v[0], y + 0.5 - v[2]
        half = math.degrees(math.atan(math.tan(math.radians(v[6] / 2)) * 16 / 9))
        if abs(math.degrees(math.atan2(vx * dz - vz * dx, vx * dx + vz * dz))) > half:
            errors.append("line %d: cam tile %d,%d outside the 16:9 view" % (line, x, y))
    return owner


def step_graph(m, passable_door, portal_ok):
    def nexts(c):
        x, y = c
        for q in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
            if q in m.walk and (q not in m.doors or passable_door(q)):
                yield q
        if c in m.portals and portal_ok(c):
            yield m.portals[c]["to"]
    return nexts


def flood(starts, nexts):
    seen = set(starts)
    q = deque(starts)
    while q:
        c = q.popleft()
        for n in nexts(c):
            if n not in seen:
                seen.add(n)
                q.append(n)
    return seen


def legend_checks(m):
    errors = m.errors
    nexts = step_graph(m, lambda d: m.doors[d]["state"] in ("open", "closed"), lambda p: m.portals[p]["lock"] is None)
    reach = flood([s for s in m.spawns if s in m.walk], nexts)
    for p in m.patrol:
        if p not in reach:
            errors.append("patrol point %d,%d unreachable by the legend from its spawn (initial doors; it opens closed doors only)" % p)
    if m.lair and m.lair not in reach:
        errors.append("lair %d,%d unreachable by the legend from its spawn" % m.lair)
    return reach


def solve(m, jairo):
    open_locks = set()
    order = []
    used = set()

    def door_ok(d):
        st = m.doors[d]
        return st["state"] in ("open", "closed") or (jairo and st["state"] == "stuck") or st["lock"] in open_locks

    def portal_ok(p):
        return m.portals[p]["lock"] is None or m.portals[p]["lock"] in open_locks

    def near(c, reach):
        x, y = c
        return c in reach or any(q in reach for q in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)))

    def prop_open(p):
        return p["lock"] is None or p["lock"] in open_locks

    def parts(lid, seen=()):
        lk = m.locks.get(lid)
        if lk is None or lid in seen:
            return set()
        return {lid}.union(*[parts(p, seen + (lid,)) for p in lk.get("parts", ())])

    exit_parts = parts(m.exit["lock"]) if m.exit and m.exit["lock"] else set()

    def at_exit(reach):
        if not m.exit:
            return False
        r = m.exit["rect"]
        return any(r[0] - 1 <= x <= r[2] + 1 and r[1] - 1 <= y <= r[3] + 1 for x, y in reach)

    while True:
        reach = flood(list(m.wakes.values()), step_graph(m, door_ok, portal_ok))
        have = {}
        for iid, it in m.items.items():
            if iid in used:
                continue
            if it["inside"] is not None:
                p = m.props[it["inside"]]
                ok = near((int(p["x"]), int(p["y"])), reach) and prop_open(p)
            else:
                ok = near((int(it["x"]), int(it["y"])), reach)
            if ok:
                have[iid] = it["kind"]
        notes = " ".join(n["text"] for n in m.notes if near((n["x"], n["y"]), reach))
        note_digits = re.sub(r"\D", "", notes)
        progress = None
        for lid, lk in m.locks.items():
            if lid in open_locks:
                continue
            users = lk["users"]
            where = any(
                (u[0] == "door" and any(s in reach for s in m.doors[u[1]]["sides"])) or
                (u[0] == "portal" and u[1] in reach) or
                (u[0] == "prop" and near((int(m.props[u[1]]["x"]), int(m.props[u[1]]["y"])), reach)) or
                (u[0] == "exit" and m.exit and any(m.exit["rect"][0] <= x <= m.exit["rect"][2] and m.exit["rect"][1] <= y <= m.exit["rect"][3] for x, y in reach))
                for u in users) or (not users and lid in exit_parts and at_exit(reach))
            k = lk["kind"]
            if k == "key":
                ok = where and lk["item"] in have
                why = "key %s" % lk["item"]
            elif k == "tool":
                ok = where and lk["tool"] in have.values()
                why = "tool %s" % lk["tool"]
            elif k == "code":
                ok = where and lk["code"] in note_digits
                why = "code %s from a note" % lk["code"]
            elif k == "place":
                ok = lk["item"] in have and near(lk["at"], reach)
                why = "place %s at %d,%d" % (lk["item"], *lk["at"])
            else:
                ok = all(p in open_locks for p in lk["parts"])
                why = "all of %s" % ",".join(lk["parts"])
            if ok:
                progress = (lid, why)
                break
        if progress is None:
            break
        lid, why = progress
        open_locks.add(lid)
        if m.locks[lid]["kind"] in ("key", "place"):
            used.add(m.locks[lid]["item"])
        order.append((lid, why))
    reach = flood(list(m.wakes.values()), step_graph(m, door_ok, portal_ok))
    shut = [lid for lid in m.locks if lid not in open_locks]
    r = m.exit["rect"] if m.exit else None
    out = bool(r) and any(r[0] <= x <= r[2] and r[1] <= y <= r[3] for x, y in reach)
    return order, reach, shut, out


def solvability(m):
    order, reach, shut, out = solve(m, True)
    for lid in shut:
        m.errors.append("line %d: unsolvable: lock %s (%s) never opens" % (m.locks[lid]["line"], lid, m.locks[lid]["kind"]))
    if not out:
        m.errors.append("unsolvable: exit rect not reachable")
    for d in m.doors:
        if d not in reach:
            m.warnings.append("door %d,%d never reachable" % d)
    for iid, it in m.items.items():
        c = (int(it["x"]), int(it["y"]))
        if it["inside"] is None and c not in reach and not any(q in reach for q in ((c[0] + 1, c[1]), (c[0] - 1, c[1]), (c[0], c[1] + 1), (c[0], c[1] - 1))):
            m.warnings.append("item %s never reachable" % iid)
    _, _, shut2, out2 = solve(m, False)
    if shut2 or not out2:
        m.warnings.append("without Jairo (stuck doors stay shut) the phase is unsolvable: locks %s never open%s" % (shut2, "" if out2 else ", exit unreachable"))
    return order


def loops(m):
    edges = [st["rooms"] for st in m.doors.values() if "-" not in st["rooms"]]
    edges += sorted({tuple(sorted((m.room_of(*c), m.room_of(*q))))
                     for c in m.walk if c not in m.doors
                     for q in ((c[0] + 1, c[1]), (c[0], c[1] + 1))
                     if q in m.walk and q not in m.doors and m.room_of(*c) != m.room_of(*q) and m.area_of(*c) == m.area_of(*q)})
    edges += [(m.room_of(*p), m.room_of(*v["to"])) for p, v in m.portals.items()]
    parent = {r: r for r in m.room_defs}

    def root(r):
        while parent[r] != r:
            r = parent[r]
        return r
    for a, b in edges:
        if a in parent and b in parent:
            parent[root(a)] = root(b)
    return len(edges) - len(parent) + len({root(r) for r in parent})


def plan(m):
    marks = {}
    for p in m.patrol:
        marks[p] = "p"
    if m.lair:
        marks[m.lair] = "Z"
    for s in m.spawns:
        marks[s] = "L"
    for iid, it in m.items.items():
        if it["inside"] is None:
            marks[(int(it["x"]), int(it["y"]))] = "*"
    for p in m.props:
        if p["item"] or p["lock"]:
            marks[(int(p["x"]), int(p["y"]))] = "$" if p["lock"] else "&"
    for n in m.notes:
        marks.setdefault((n["x"], n["y"]), "?")
    for c in m.hides:
        marks.setdefault(c, "h")
    for ch, c in m.wakes.items():
        marks[c] = str(ch)
    glyph = {"#": "#", "W": "=", "B": "o", " ": " ", "~": "~", "|": "|", "H": "h", "P": "^"}
    door_glyph = {"open": "/", "closed": "+", "stuck": "S", "locked": "X"}
    out = []
    for a, name in sorted(m.areas.items()):
        tiles = [(x, y) for y in range(m.h) for x in range(m.w) if m.area_of(x, y) == a]
        if not tiles:
            continue
        x0, x1 = max(0, min(x for x, _ in tiles) - 1), min(m.w - 1, max(x for x, _ in tiles) + 1)
        y0, y1 = max(0, min(y for _, y in tiles) - 1), min(m.h - 1, max(y for _, y in tiles) + 1)
        out.append("area %d: %s  (x %d..%d, y %d..%d)" % (a, name, x0, x1, y0, y1))
        out.append("    " + "".join(str(x // 10 % 10) if x % 10 == 0 else " " for x in range(x0, x1 + 1)))
        out.append("    " + "".join(str(x % 10) for x in range(x0, x1 + 1)))
        ex = m.exit["rect"] if m.exit else None
        for y in range(y0, y1 + 1):
            row = []
            for x in range(x0, x1 + 1):
                c = m.solid[y][x]
                ra = m.area_of(x, y)
                if (x, y) in marks:
                    row.append(marks[(x, y)])
                elif c == "D":
                    row.append(door_glyph.get(m.doors.get((x, y), {}).get("state"), "D"))
                elif c == ".":
                    row.append("E" if ex and ex[0] <= x <= ex[2] and ex[1] <= y <= ex[3] else m.rooms[y][x] if ra == a else " ")
                elif c in "HP" and ra != a:
                    row.append(" ")
                else:
                    row.append(glyph[c])
            out.append("%3d " % y + "".join(row))
        out.append("")
    out.append("legend: # wall  = window  o furniture  h hide  ^ portal  ~ water  | railing  / open  + closed  S stuck  X locked door")
    out.append("        0-3 wake spots  * item  & container  $ locked container  ? note  L legend spawn  p patrol  Z lair  E exit")
    out += ["        %s = %s (area %d%s)" % (k, v["name"], v["area"], ", outdoor" if v["out"] else "") for k, v in sorted(m.room_defs.items())]
    return "\n".join(out)


def main(argv):
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    path = argv[1] if len(argv) > 1 else os.path.join(root, "assets", "map", "sobrado.map")
    texdir = os.path.join(root, "assets", "tex")
    try:
        m = load(path, texdir)
    except Bad as e:
        print("ERROR " + str(e))
        return 1
    owner = cam_checks(m)
    legend_checks(m)
    order = solvability(m)
    nloops = loops(m)
    if nloops < 2:
        m.errors.append("only %d independent loops between rooms, need >= 2" % nloops)
    print(plan(m))
    print("\n%s: %d areas, %d rooms, %d cams, %d doors, %d portals, %d locks, %d items, %d notes, %d hides, %d loops, %d walkable tiles" % (
        m.name, len(m.areas), len(m.room_defs), len(m.cams), len(m.doors), len(m.portals), len(m.locks), len(m.items),
        len(m.notes), len(m.hides), nloops, len(owner)))
    print("solution order:")
    for i, (lid, why) in enumerate(order, 1):
        print("  %2d. %s  <- %s" % (i, lid, why))
    for w in m.warnings:
        print("WARN " + w)
    for e in m.errors:
        print("ERROR " + e)
    print("OK" if not m.errors else "%d error(s)" % len(m.errors))
    return 1 if m.errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
