#!/usr/bin/env python3
import math
import os
import sys
from collections import deque

GRIDS = ("solid", "rooms")
TASKS = {"window", "breaker", "key", "tv", "gas", "faucet"}
PROPS = {
    "sofa", "poltrona", "tv_tubo", "rack", "cristaleira", "mesa", "cadeira", "cama", "criado_mudo",
    "guarda_roupa", "geladeira", "fogao", "pia", "filtro_barro", "quadro_santo", "relogio_parede",
    "disjuntor", "estante", "banheira", "vaso", "espelho", "cortina", "abajur", "rede",
}
WALKABLE = ".D"


def parse(text):
    sections = {}
    current = None
    for raw in text.splitlines():
        line = raw.rstrip("\n")
        if line.startswith(";"):
            continue
        if line.startswith("[") and line.rstrip().endswith("]"):
            current = line.strip()[1:-1]
            sections.setdefault(current, [])
            continue
        if current is None:
            if line.strip():
                raise ValueError("content before first section: %r" % line)
            continue
        if current in GRIDS:
            if line.strip():
                sections[current].append(line.rstrip())
        elif line.strip():
            sections[current].append(line.split())
    return sections


def num(tok, errors, what):
    try:
        v = float(tok)
    except ValueError:
        errors.append("%s: %r is not a number" % (what, tok))
        return 0.0
    if "." in tok and len(tok.split(".")[1]) > 1:
        errors.append("%s: %r has too many decimals" % (what, tok))
    return v


def integer(tok, errors, what):
    if not tok.lstrip("-").isdigit():
        errors.append("%s: %r is not an integer" % (what, tok))
        return 0
    return int(tok)


def check(path, texdir):
    errors = []
    with open(path) as f:
        sec = parse(f.read())
    for name in ("solid", "rooms", "room", "door", "cam", "prop", "spot"):
        if name not in sec:
            errors.append("missing section [%s]" % name)
    solid = sec.get("solid", [])
    rooms = sec.get("rooms", [])
    width = max((len(r) for r in solid + rooms), default=0)
    height = len(solid)
    if any(len(r) != width for r in solid):
        errors.append("[solid] rows have different widths: %s" % sorted({len(r) for r in solid}))
    if any(len(r) != width for r in rooms):
        errors.append("[rooms] rows have different widths: %s" % sorted({len(r) for r in rooms}))
    if len(rooms) != height:
        errors.append("[rooms] has %d rows, [solid] has %d" % (len(rooms), height))
    solid = [r.ljust(width) for r in solid]
    rooms = [r.ljust(width) for r in rooms] + [" " * width] * max(0, height - len(rooms))
    bad = {c for r in solid for c in r} - set("#.DWB ")
    if bad:
        errors.append("[solid] unknown chars %s" % sorted(bad))

    def tile(x, y):
        return solid[y][x] if 0 <= y < height and 0 <= x < width else " "

    walk = {(x, y) for y in range(height) for x in range(width) if solid[y][x] in WALKABLE}

    room_defs = {}
    for t in sec.get("room", []):
        if len(t) != 6 or len(t[0]) != 1:
            errors.append("[room] bad line %s" % " ".join(t))
            continue
        kv = dict(p.split("=", 1) for p in t[2:] if "=" in p)
        if set(kv) != {"floor", "wall", "ceil", "light"}:
            errors.append("[room] %s needs floor= wall= ceil= light=" % t[0])
            continue
        light = integer(kv["light"], errors, "room %s light" % t[0])
        if not 0 <= light <= 100:
            errors.append("room %s light %d out of 0..100" % (t[0], light))
        for key in ("floor", "wall", "ceil"):
            tex = kv[key]
            if key == "ceil" and tex == "-":
                continue
            if not os.path.isfile(os.path.join(texdir, tex + ".tex")):
                errors.append("room %s %s=%s: missing %s/%s.tex" % (t[0], key, tex, texdir, tex))
        room_defs[t[0]] = t[1]
    used = {c for r in rooms for c in r} - {"-", " "}
    for c in sorted(used - set(room_defs)):
        errors.append("[rooms] uses id %r with no [room] line" % c)
    for c in sorted(set(room_defs) - used):
        errors.append("[room] %s defined but not used in [rooms]" % c)
    for (x, y) in sorted(walk):
        if solid[y][x] == "." and rooms[y][x] in "- ":
            errors.append("walkable tile %d,%d has no room id" % (x, y))
    for y in range(height):
        for x in range(width):
            if solid[y][x] in "#W " and rooms[y][x] not in "- ":
                errors.append("wall/void tile %d,%d has room id %r" % (x, y, rooms[y][x]))

    door_tiles = {(x, y) for (x, y) in walk if solid[y][x] == "D"}
    doors = {}
    door_axes = {}
    for t in sec.get("door", []):
        if len(t) != 4:
            errors.append("[door] bad line %s" % " ".join(t))
            continue
        x, y = integer(t[0], errors, "door x"), integer(t[1], errors, "door y")
        if tile(x, y) != "D":
            errors.append("door %d,%d not on a D tile" % (x, y))
        if t[2] not in ("h", "v"):
            errors.append("door %d,%d axis %r" % (x, y, t[2]))
        if t[3] not in ("open", "closed", "locked"):
            errors.append("door %d,%d state %r" % (x, y, t[3]))
        if t[2] == "h" and not (tile(x - 1, y) in "#W" and tile(x + 1, y) in "#W"):
            errors.append("door %d,%d axis h but wall does not run along x" % (x, y))
        if t[2] == "v" and not (tile(x, y - 1) in "#W" and tile(x, y + 1) in "#W"):
            errors.append("door %d,%d axis v but wall does not run along y" % (x, y))
        if (x, y) in doors:
            errors.append("door %d,%d listed twice" % (x, y))
        doors[(x, y)] = t[3]
        door_axes[(x, y)] = t[2]
    for d in sorted(door_tiles - set(doors)):
        errors.append("D tile %d,%d has no [door] line" % d)

    cams = []
    for i, t in enumerate(sec.get("cam", [])):
        if len(t) != 11:
            errors.append("[cam] line %d needs 11 fields" % (i + 1))
            continue
        r = [integer(v, errors, "cam %d rect" % (i + 1)) for v in t[:4]]
        v = [num(s, errors, "cam %d" % (i + 1)) for s in t[4:]]
        if r[0] > r[2] or r[1] > r[3]:
            errors.append("cam %d rect inverted" % (i + 1))
        if not 10 <= v[6] <= 120:
            errors.append("cam %d fov %g" % (i + 1, v[6]))
        if (v[0], v[1], v[2]) == (v[3], v[4], v[5]):
            errors.append("cam %d eye equals target" % (i + 1))
        ex, ez = int(v[0]), int(v[2])
        if tile(ex, ez) not in WALKABLE + "B" and v[1] <= 2.6:
            errors.append("cam %d eye %.1f,%.1f inside wall/void tile %r" % (i + 1, v[0], v[2], tile(ex, ez)))
        cams.append((r, v))
    owner = {}
    for (x, y) in walk:
        hit = next((i for i, (r, _) in enumerate(cams) if r[0] <= x <= r[2] and r[1] <= y <= r[3]), None)
        if hit is None:
            errors.append("walkable tile %d,%d not covered by any cam" % (x, y))
        else:
            owner[(x, y)] = hit
    for i in range(len(cams)):
        if i not in owner.values():
            errors.append("cam %d never selected (shadowed by earlier rects)" % (i + 1))

    def sees(ax, az, bx, bz):
        steps = int(math.hypot(bx - ax, bz - az) * 10) + 1
        return all(tile(int(ax + (bx - ax) * s / steps), int(az + (bz - az) * s / steps)) not in "#W "
                   for s in range(1, steps))
    for (x, y), i in sorted(owner.items()):
        eye = cams[i][1]
        if not sees(eye[0], eye[2], x + 0.5, y + 0.5):
            errors.append("cam %d cannot see its tile %d,%d (wall in the way)" % (i + 1, x, y))
        dx, dz = eye[3] - eye[0], eye[5] - eye[2]
        vx, vz = x + 0.5 - eye[0], y + 0.5 - eye[2]
        half = math.degrees(math.atan(math.tan(math.radians(eye[6] / 2)) * 16 / 9))
        if abs(math.degrees(math.atan2(vx * dz - vz * dx, vx * dx + vz * dz))) > half:
            errors.append("cam %d tile %d,%d outside the 16:9 view" % (i + 1, x, y))

    for t in sec.get("prop", []):
        if len(t) != 4 or t[0] not in PROPS:
            errors.append("[prop] bad line %s" % " ".join(t))
            continue
        x, y = num(t[1], errors, "prop x"), num(t[2], errors, "prop y")
        num(t[3], errors, "prop rot")
        if tile(int(x), int(y)) in "# ":
            errors.append("prop %s at %s,%s stands in wall/void" % tuple(t[:3]))

    spawns, tasks, finals, exits = [], [], [], []
    for t in sec.get("spot", []):
        kind = t[0] if t else ""
        if kind == "spawn" and len(t) == 4:
            x, y = num(t[1], errors, "spawn x"), num(t[2], errors, "spawn y")
            num(t[3], errors, "spawn rot")
            if (int(x), int(y)) not in walk or solid[int(y)][int(x)] != ".":
                errors.append("spawn %s,%s not on floor" % (t[1], t[2]))
            spawns.append((int(x), int(y)))
        elif kind == "task" and len(t) == 4:
            if t[1] not in TASKS:
                errors.append("task kind %r" % t[1])
            x, y = num(t[2], errors, "task x"), num(t[3], errors, "task y")
            if (int(x), int(y)) not in walk or solid[int(y)][int(x)] != ".":
                errors.append("task %s %s,%s not on floor" % tuple(t[1:4]))
            tasks.append((t[1], int(x), int(y)))
        elif kind == "final" and len(t) == 5:
            finals.append([integer(v, errors, "final") for v in t[1:]])
        elif kind == "exit" and len(t) == 3:
            x, y = integer(t[1], errors, "exit x"), integer(t[2], errors, "exit y")
            if (x, y) not in walk:
                errors.append("exit %d,%d not walkable" % (x, y))
            exits.append((x, y))
        else:
            errors.append("[spot] bad line %s" % " ".join(t))
    if len(spawns) < 4:
        errors.append("need >= 4 spawns, have %d" % len(spawns))
    if len(set(spawns)) != len(spawns):
        errors.append("duplicate spawn tiles")
    for k in sorted(TASKS - {t[0] for t in tasks}):
        errors.append("no task spot of kind %s" % k)
    if len(finals) != 1:
        errors.append("need exactly one final rect, have %d" % len(finals))
    if len(exits) != 1:
        errors.append("need exactly one exit, have %d" % len(exits))
    if finals and exits:
        f, (ex, ey) = finals[0], exits[0]
        if not (f[0] - 1 <= ex <= f[2] + 1 and f[1] - 1 <= ey <= f[3] + 1):
            errors.append("exit %d,%d not adjacent to final rect" % (ex, ey))
        inside = [(x, y) for (x, y) in walk if f[0] <= x <= f[2] and f[1] <= y <= f[3] and solid[y][x] == "."]
        if len({rooms[y][x] for x, y in inside}) != 1:
            errors.append("final rect spans several rooms")

    if walk:
        start = min(walk)
        seen = {start}
        queue = deque([start])
        while queue:
            x, y = queue.popleft()
            for n in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
                if n in walk and n not in seen:
                    seen.add(n)
                    queue.append(n)
        for t in sorted(walk - seen):
            errors.append("walkable tile %d,%d unreachable with doors open" % t)

    edges = []
    for (x, y), axis in sorted(door_axes.items()):
        a, b = ((x, y - 1), (x, y + 1)) if axis == "h" else ((x - 1, y), (x + 1, y))
        ends = [rooms[q][p] for p, q in (a, b) if 0 <= q < height and 0 <= p < width and rooms[q][p] not in "- "]
        if len(ends) != 2 or ends[0] == ends[1]:
            errors.append("door %d,%d does not join two different rooms" % (x, y))
        else:
            edges.append(tuple(ends))
    parent = {r: r for r in room_defs}

    def root(r):
        while parent[r] != r:
            r = parent[r]
        return r
    for a, b in edges:
        if a in parent and b in parent:
            parent[root(a)] = root(b)
    loops = len(edges) - len(parent) + len({root(r) for r in parent})
    if loops < 2:
        errors.append("only %d independent loops through the house, need >= 2" % loops)

    return errors, solid, rooms, room_defs, cams, owner, tasks, spawns, doors, loops


def render(solid, rooms, room_defs, tasks, spawns, doors):
    marks = {(x, y): "S" for x, y in spawns}
    marks.update({(x, y): k[0].upper() if k != "faucet" else "F" for k, x, y in tasks})
    glyph = {"#": "#", "W": "=", "B": "o", " ": " "}
    width = len(solid[0]) if solid else 0
    lines = ["    " + "".join(str(x // 10) if x % 10 == 0 else " " for x in range(width)),
             "    " + "".join(str(x % 10) for x in range(width))]
    for y, row in enumerate(solid):
        cells = []
        for x, c in enumerate(row):
            if (x, y) in marks:
                cells.append(marks[(x, y)])
            elif c == "D":
                cells.append({"open": "/", "closed": "+", "locked": "X"}.get(doors.get((x, y)), "D"))
            elif c == ".":
                cells.append(rooms[y][x])
            else:
                cells.append(glyph[c])
        lines.append("%3d " % y + "".join(cells))
    legend = ["", "legend: # wall, = window, o furniture, / open door, + closed door, X locked door,",
              "        lowercase = room floor, S spawn, W window/B breaker/K key/T tv/G gas/F faucet task"]
    legend += ["        %s = %s" % (k, v) for k, v in sorted(room_defs.items())]
    return "\n".join(lines + legend)


def main(argv):
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    path = argv[1] if len(argv) > 1 else os.path.join(root, "assets", "map", "sobrado.map")
    texdir = os.path.join(root, "assets", "tex")
    errors, solid, rooms, room_defs, cams, owner, tasks, spawns, doors, loops = check(path, texdir)
    print(render(solid, rooms, room_defs, tasks, spawns, doors))
    print("\n%d rooms, %d cams, %d doors, %d loops, %d walkable tiles, %d task spots, %d spawns" % (
        len(room_defs), len(cams), len(doors), loops, len(owner), len(tasks), len(spawns)))
    for e in errors:
        print("ERROR " + e)
    print("OK" if not errors else "%d error(s)" % len(errors))
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
