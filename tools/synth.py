#!/usr/bin/env python3
import math
import random
import subprocess
import sys


def load(path, w, h):
    raw = subprocess.run(["magick", path, "-colorspace", "sRGB", "-alpha", "off", "-resize", "%dx%d!" % (w, h), "-depth", "8", "rgb:-"],
                         capture_output=True, check=True).stdout
    return [[tuple(raw[(y * w + x) * 3:(y * w + x) * 3 + 3]) for x in range(w)] for y in range(h)]


def save(path, img):
    h, w = len(img), len(img[0])
    data = b"P6\n%d %d\n255\n" % (w, h) + bytes(max(0, min(255, int(round(c)))) for row in img for p in row for c in p)
    subprocess.run(["magick", "ppm:-", path], input=data, check=True)


def save_rgba(path, img):
    h, w = len(img), len(img[0])
    head = b"P7\nWIDTH %d\nHEIGHT %d\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n" % (w, h)
    data = head + bytes(max(0, min(255, int(round(c)))) for row in img for p in row for c in p)
    subprocess.run(["magick", "pam:-", path], input=data, check=True)


def fbm(seed, n, cells, octaves):
    layers = [periodic_noise(seed + i, n, cells << i) for i in range(octaves)]
    norm = sum(0.5 ** i for i in range(octaves))

    def at(x, y):
        return sum(f(x, y) * 0.5 ** i for i, f in enumerate(layers)) / norm
    return at


def shade(col, k):
    return tuple(c * k for c in col)


def grid(w, h, fn):
    return [[fn(x, y) for x in range(w)] for y in range(h)]


def mix(a, b, t):
    return tuple(x + (y - x) * t for x, y in zip(a, b))


def smooth(e0, e1, x):
    t = max(0.0, min(1.0, (x - e0) / (e1 - e0)))
    return t * t * (3 - 2 * t)


def periodic_noise(seed, n, cells):
    rnd = random.Random(seed)
    g = [[rnd.random() for _ in range(cells)] for _ in range(cells)]

    def at(x, y):
        fx, fy = x * cells / n, y * cells / n
        x0, y0 = int(math.floor(fx)), int(math.floor(fy))
        tx, ty = smooth(0, 1, fx - x0), smooth(0, 1, fy - y0)
        a = g[y0 % cells][x0 % cells] * (1 - tx) + g[y0 % cells][(x0 + 1) % cells] * tx
        b = g[(y0 + 1) % cells][x0 % cells] * (1 - tx) + g[(y0 + 1) % cells][(x0 + 1) % cells] * tx
        return a * (1 - ty) + b * ty
    return at


def taco(grain_path, out):
    n, k, cells = 512, 4, 16
    cell = n // cells
    grain = load(grain_path, 512, 512)
    rnd = random.Random(7)
    tones = {}
    bricks = [(0, (i, 0)) for i in range(k)] + [(1, (0, j)) for j in range(1, k + 1)]

    def brick_at(cx, cy):
        for kind, (bx, by) in bricks:
            dx, dy = cx - bx, cy - by
            if (dx + dy) % 2 == 0 and (dx - dy) % (2 * k) == 0:
                a, m = (dx + dy) // 2, (dx - dy) // (2 * k)
                ox, oy = a + k * m, a - k * m + (0 if kind == 0 else 1)
                return kind, ox, oy
        raise ValueError((cx, cy))

    img = []
    for y in range(n):
        row = []
        for x in range(n):
            cx, cy = x // cell, y // cell
            kind, ox, oy = brick_at(cx, cy)
            key = (kind, ox % cells, oy % cells)
            if key not in tones:
                tones[key] = (rnd.uniform(0.72, 1.12), rnd.randrange(512), rnd.randrange(512), rnd.uniform(-0.06, 0.06))
            tone, gx, gy, warm = tones[key]
            lx = x - ox * cell
            ly = y - oy * cell
            if kind == 0:
                along, across, length = lx % (cells * cell), ly % (cells * cell), k * cell
            else:
                along, across, length = ly % (cells * cell), lx % (cells * cell), k * cell
            along %= length
            across %= cell
            g = grain[(gy + across * 3) % 512][(gx + along) % 512]
            edge = min(along, length - 1 - along, across, cell - 1 - across)
            shade = 0.45 + 0.55 * smooth(0, 3.5, edge)
            lit = 1.06 if (across < 3 and edge >= 1) else 1.0
            base = (g[0] * (1 + warm), g[1], g[2] * (1 - warm))
            row.append(tuple(c * tone * shade * lit for c in base))
        img.append(row)
    save(out, img)


def azulejo(out):
    n = 512
    tile = n // 2
    white = (236, 232, 216)
    blue = (24, 46, 128)
    wash = mix(white, blue, 0.42)
    wob = periodic_noise(3, n, 16)
    ink = periodic_noise(4, n, 40)
    dots = [(0.56 * math.cos(a), 0.56 * math.sin(a)) for a in (math.pi / 8 + i * math.pi / 4 for i in range(8))]
    leaves = [(0.5 * sx, 0.5 * sy, math.atan2(sy, sx)) for sx in (-1, 1) for sy in (-1, 1)]

    def shape(sd, line):
        return 2 if abs(sd) < line else (1 if sd < 0 else 0)

    img = []
    for y in range(n):
        row = []
        for x in range(n):
            u = (x % tile) / tile * 2 - 1 + (wob(x, y) - 0.5) * 0.04
            v = (y % tile) / tile * 2 - 1 + (wob(y + 77, x) - 0.5) * 0.04
            r = math.hypot(u, v)
            th = math.atan2(v, u)
            layers = []
            layers.append(shape(r - 0.4 * (0.45 + 0.55 * abs(math.cos(4 * th)) ** 0.7), 0.025))
            if r < 0.11:
                layers.append(3)
            elif r < 0.15:
                layers.append(0)
            if any(math.hypot(u - px, v - py) < 0.055 for px, py in dots):
                layers.append(3)
            for lx, ly, la in leaves:
                ru = (u - lx) * math.cos(la) + (v - ly) * math.sin(la)
                rv = -(u - lx) * math.sin(la) + (v - ly) * math.cos(la)
                layers.append(shape(math.hypot(ru / 0.2, rv / 0.07) - 1, 0.18))
            cx, cy = (1 if u > 0 else -1), (1 if v > 0 else -1)
            cr = math.hypot(u - cx, v - cy)
            cth = math.atan2(v - cy, u - cx)
            if 0.1 <= cr and cr - 0.55 * (0.55 + 0.45 * abs(math.cos(4 * cth)) ** 0.7) < 0:
                layers.append(3)
            er = min(math.hypot(u - ex, v - ey) for ex, ey in ((0, 1), (0, -1), (1, 0), (-1, 0)))
            layers.append(shape(er - 0.24, 0.025))
            if 0.86 < max(abs(u), abs(v)) < 0.9 and min(abs(u), abs(v)) < 0.6:
                layers.append(3)
            top = max(layers)
            t = 0.8 + 0.45 * (ink(x, y) - 0.5)
            col = {0: white, 1: wash, 2: mix(white, blue, min(1.0, t + 0.1)), 3: mix(white, blue, min(1.0, t))}[top]
            gx, gy = x % tile, y % tile
            if min(gx, gy, tile - 1 - gx, tile - 1 - gy) < 3:
                col = (140, 136, 124)
            row.append(col)
        img.append(row)
    save(out, img)


def floral(paper_path, out):
    n = 256
    paper = load(paper_path, n, n)
    bg = (182, 172, 132)
    stripe = (150, 140, 104)
    rose = (172, 96, 90)
    rose_dark = (118, 54, 58)
    leaf = (92, 108, 66)
    motifs = [(0.25, 0.125, 0), (0.75, 0.375, 1), (0.25, 0.625, 1), (0.75, 0.875, 0)]
    rnd = random.Random(11)
    petals = [(rnd.uniform(-0.03, 0.03), rnd.uniform(-0.03, 0.03), rnd.uniform(0.022, 0.032)) for _ in range(7)]
    leafs = [(-0.06, 0.04, 0.6), (0.06, 0.045, -0.6), (0.0, -0.07, 0.0), (-0.055, -0.045, -0.9), (0.06, -0.04, 0.9)]
    img = []
    for y in range(n):
        row = []
        for x in range(n):
            u, v = x / n, y / n
            col = stripe if min(abs(u - 0.5), u, 1 - u) < 0.006 or abs(min(abs(u - 0.5), u, 1 - u) - 0.02) < 0.004 else bg
            for mx, my, flip in motifs:
                du = ((u - mx + 0.5) % 1 - 0.5) * (-1 if flip else 1) / 1.7
                dv = ((v - my + 0.5) % 1 - 0.5) / 1.7
                if abs(du) > 0.16 or abs(dv) > 0.16:
                    continue
                for lx, ly, la in leafs:
                    ru = (du - lx) * math.cos(la) + (dv - ly) * math.sin(la)
                    rv = -(du - lx) * math.sin(la) + (dv - ly) * math.cos(la)
                    if (ru / 0.042) ** 2 + (rv / 0.017) ** 2 < 1:
                        col = leaf if abs(rv) > 0.003 else mix(leaf, bg, 0.4)
                hit = [(math.hypot(du - px, dv - py), rr) for px, py, rr in petals]
                inside = [rr - d for d, rr in hit if d < rr]
                if inside:
                    col = rose_dark if min(inside) < 0.006 else rose
                if math.hypot(du, dv) < 0.01:
                    col = rose_dark
                for bx, by in ((0.09, 0.08), (-0.1, 0.07)):
                    if math.hypot(du - bx, dv - by) < 0.013:
                        col = rose
            p = paper[y][x]
            row.append(tuple(c * (0.5 + 0.5 * pc / 170) for c, pc in zip(col, p)))
        img.append(row)
    save(out, img)


def croche(out):
    n = 256
    thread = (238, 232, 214)
    img = []
    for y in range(n):
        row = []
        for x in range(n):
            u, v = x / n * 2 - 1, y / n * 2 - 1
            r = math.hypot(u, v)
            th = math.atan2(v, u)
            flower = 0.14 + 0.2 * abs(math.cos(4 * th)) ** 2
            if r < 0.14:
                on = r < 0.05 or abs(r - 0.1) < 0.025
            elif r < 0.38:
                on = abs(r - flower) < 0.035 or abs(r - 0.37) < 0.02
            elif r < 0.62:
                band = (r - 0.38) / 0.24 * 4
                cell = (th / (2 * math.pi) * 32 + (int(band) % 2) * 0.5) % 1
                on = abs(band % 1 - 0.5) > 0.32 or abs(cell - 0.5) > 0.3
            elif r < 0.68:
                on = True
            elif r < 0.84:
                cell = (th / (2 * math.pi) * 16) % 1
                on = abs(cell - 0.5) > 0.38 or abs(r - 0.76) < 0.02 or r > 0.82
            else:
                on = r < 0.84 + 0.1 * abs(math.cos(th * 12)) and (r < 0.88 or abs(math.cos(th * 12)) > 0.5)
            if on:
                shade = 0.86 + 0.14 * math.cos(x * 0.9) * math.cos(y * 0.9)
                row.append(tuple(c * shade for c in thread))
            else:
                row.append((0, 0, 0))
        img.append(row)
    save(out, img)


def vidro(out):
    n = 256
    cloud = periodic_noise(21, n, 6)
    dirt = periodic_noise(22, n, 48)
    img = []
    for y in range(n):
        row = []
        for x in range(n):
            u, v = x / n, y / n
            sky = mix((26, 34, 56), (6, 8, 16), v ** 0.8)
            sky = mix(sky, (40, 48, 70), 0.25 * smooth(0.55, 0.8, cloud(x, y)) * (1 - v))
            streak = math.exp(-((u + v * 0.6 - 0.62) / 0.05) ** 2) * 0.22 + math.exp(-((u + v * 0.6 - 0.78) / 0.018) ** 2) * 0.12
            col = mix(sky, (150, 160, 175), streak)
            ghost = math.exp(-(((u - 0.3) / 0.09) ** 2 + ((v - 0.62) / 0.22) ** 2)) * 0.12
            col = mix(col, (90, 70, 50), ghost)
            spec = dirt(x, y)
            col = tuple(c * (0.75 + 0.35 * spec) for c in col)
            edge = min(u, v, 1 - u, 1 - v)
            col = tuple(c * (0.55 + 0.45 * smooth(0.0, 0.08, edge)) for c in col)
            row.append(col)
        img.append(row)
    save(out, img)


def porta(grain_path, out):
    w, h = 256, 512
    grain = load(grain_path, 512, 512)
    stile, rail = 34, 34
    panels = [(stile, rail, w // 2 - 10, 230), (w // 2 + 10, rail, w - stile, 230),
              (stile, 270, w // 2 - 10, h - rail - 60), (w // 2 + 10, 270, w - stile, h - rail - 60)]
    img = []
    for y in range(h):
        row = []
        for x in range(w):
            g = grain[(y * 2 + x // 64 * 37) % 512][(x * 2 + y // 128 * 91) % 512]
            col = g
            shade = 1.0
            for x0, y0, x1, y1 in panels:
                if x0 <= x < x1 and y0 <= y < y1:
                    d = min(x - x0, y - y0, x1 - 1 - x, y1 - 1 - y)
                    if d < 12:
                        top_left = (x - x0) < (y1 - 1 - y) and (y - y0) < (x1 - 1 - x)
                        shade = 0.62 if top_left else 1.22
                        if d < 2:
                            shade = 0.45
                    else:
                        g2 = grain[(x * 2 + 50) % 512][(y + x0 * 3) % 512]
                        col = g2
                        shade = 0.95
            hx, hy = w - 46, 300
            if math.hypot(x - hx, y - hy) < 13:
                col, shade = (150, 118, 52), 1.0 + 0.4 * (hx - x + hy - y) / 26
            elif hx - 8 <= x <= hx + 8 and hy + 16 <= y <= hy + 52:
                col, shade = (120, 92, 40), 0.9
                if hx - 3 <= x <= hx + 3 and hy + 28 <= y <= hy + 42:
                    col, shade = (10, 8, 6), 1.0
            if x < 3 or y < 3 or x >= w - 3:
                shade *= 0.5
            row.append(tuple(c * shade for c in col))
        img.append(row)
    save(out, img)


def listrado(paper_path, out):
    n = 256
    paper = load(paper_path, n, n)
    cream, sage, pin, vine, bud = (196, 186, 150), (128, 136, 104), (112, 50, 46), (84, 100, 66), (156, 72, 64)
    buds = [(0.4, (k + 0.25) / 8) for k in range(8)] + [(0.1, (k + 0.75) / 8) for k in range(8)]

    def px(x, y):
        u, v = x / n, y / n
        s = (u * 4) % 1
        col = cream if s < 0.5 else sage
        if min(s, 1 - s, abs(s - 0.5)) < 0.014:
            col = pin
        if s < 0.5 and abs(s - 0.25 - 0.1 * math.sin(2 * math.pi * v * 8)) < 0.016:
            col = vine
        if any(math.hypot((s - bs) * n / 4, ((v - bv + 0.5) % 1 - 0.5) * n) < 3.2 for bs, bv in buds):
            col = bud
        return tuple(c * (0.5 + 0.5 * pc / 170) for c, pc in zip(col, paper[y][x]))
    save(out, grid(n, n, px))


def calcada(out):
    n, cells = 512, 13
    size = n / cells
    rnd = random.Random(41)
    pts = [[(rnd.random(), rnd.random(), rnd.uniform(0.84, 1.08)) for _ in range(cells)] for _ in range(cells)]
    grain = fbm(42, n, 32, 3)
    white, black, grout = (214, 208, 192), (46, 44, 42), (70, 64, 54)

    def seed(cx, cy):
        jx, jy, tone = pts[cy % cells][cx % cells]
        return cx + jx, cy + jy, tone

    def px(x, y):
        gx, gy = x / size, y / size
        cx, cy = int(gx), int(gy)
        near = sorted((math.hypot(gx - sx, gy - sy), sx, sy, tone)
                      for sx, sy, tone in (seed(cx + i, cy + j) for i in (-1, 0, 1) for j in (-1, 0, 1)))
        (d1, sx, sy, tone), d2 = near[0], near[1][0]
        dark = (sy / cells + 0.11 * math.sin(2 * math.pi * sx / cells)) % 0.5 < 0.22
        e = (d2 - d1) * size
        k = tone * (1.25 if dark else 1.0) * (0.74 + 0.26 * smooth(0, 5, e) + 0.2 * (grain(x, y) - 0.5))
        return grout if e < 1.8 else shade(black if dark else white, k)
    save(out, grid(n, n, px))


def veins(seed, n, freq):
    turb = fbm(seed, n, 4, 4)
    fine = fbm(seed + 9, n, 16, 3)

    def at(x, y):
        a = 2 * math.pi * (x + 2 * y) / n * freq + 7 * turb(x, y)
        f = fine(x, y)
        main_vein = math.exp(-(math.sin(a) / 0.08) ** 2)
        thin = 0.5 * math.exp(-(math.sin(2 * a + 4 * f) / 0.05) ** 2)
        return max(main_vein, thin) * (0.4 + 0.6 * f)
    return at


def marmore(out):
    n = 256
    vein = veins(51, n, 1)
    cloud = fbm(53, n, 6, 3)
    save(out, grid(n, n, lambda x, y: mix(shade((226, 222, 212), 0.9 + 0.12 * cloud(x, y)), (112, 110, 108), 0.85 * vein(x, y))))


def xadrez(out):
    n = 256
    tile = n // 2
    vein_w, vein_b = veins(61, n, 2), veins(67, n, 1)
    cloud = fbm(63, n, 8, 3)

    def px(x, y):
        lx, ly = x % tile, y % tile
        if min(lx, ly) < 2:
            return (74, 70, 64)
        c = cloud(x, y)
        if (x // tile + y // tile) % 2:
            col = mix(shade((30, 30, 34), 0.85 + 0.3 * c), (150, 148, 144), 0.55 * vein_b(x, y))
        else:
            col = mix(shade((226, 222, 212), 0.9 + 0.12 * c), (118, 116, 112), 0.8 * vein_w(x, y))
        return shade(col, 0.82 + 0.18 * smooth(0, 4, min(lx - 2, ly - 2, tile - 1 - lx, tile - 1 - ly)))
    save(out, grid(n, n, px))


def corners(u, v):
    return min(math.hypot(u - sx, v - sy) for sx in (-1, 1) for sy in (-1, 1))


def ladrilho(out):
    n = 256
    tile = n // 2
    cream, red, black, ochre = (204, 188, 150), (146, 56, 40), (40, 34, 30), (184, 136, 62)
    mottle = fbm(71, n, 16, 3)
    wear = fbm(73, n, 6, 3)

    def px(x, y):
        lx, ly = x % tile, y % tile
        if min(lx, ly) < 2:
            return (90, 84, 72)
        u, v = lx / tile * 2 - 1, ly / tile * 2 - 1
        r, th, rc = math.hypot(u, v), math.atan2(v, u), corners(u, v)
        if rc < 0.38:
            col = red
        elif rc < 0.46:
            col = black
        elif r < 0.13:
            col = red
        elif r < 0.5 * (0.62 + 0.38 * abs(math.cos(4 * th))):
            col = ochre if r > 0.18 else black
        elif abs(abs(u) + abs(v) - 1.0) < 0.05:
            col = black
        else:
            col = cream
        col = shade(col, 0.88 + 0.2 * (mottle(x, y) - 0.5))
        return mix(col, (150, 140, 118), 0.35 * smooth(0.55, 0.8, wear(x, y)))
    save(out, grid(n, n, px))


def capela(out):
    n = 256
    tile = n // 2
    white, blue = (232, 230, 218), (30, 54, 140)
    wash = mix(white, blue, 0.28)
    ink = fbm(81, n, 20, 2)
    wob = periodic_noise(83, n, 16)

    def px(x, y):
        lx, ly = x % tile, y % tile
        if min(lx, ly) < 2:
            return (150, 146, 134)
        u = lx / tile * 2 - 1 + (wob(x, y) - 0.5) * 0.03
        v = ly / tile * 2 - 1 + (wob(y + 31, x) - 0.5) * 0.03
        r, rc = math.hypot(u, v), corners(u, v)
        arm, reach = min(abs(u), abs(v)), max(abs(u), abs(v))
        cross = reach < 0.6 and arm < 0.07 + 0.17 * smooth(0.25, 0.6, reach) and r > 0.09
        edge = min(math.hypot(u - ex, v - ey) for ex, ey in ((0, 1), (0, -1), (1, 0), (-1, 0)))
        line = cross or abs(rc - 0.66) < 0.05 or rc < 0.2 or edge < 0.1 or abs(r - 0.09) < 0.025
        t = 0.78 + 0.5 * (ink(x, y) - 0.5)
        return mix(white, blue, min(1.0, t)) if line else (wash if 0.2 < rc < 0.6 else white)
    save(out, grid(n, n, px))


def vitral(out):
    w, h = 256, 512
    mot = fbm(91, 256, 8, 3)
    rnd = random.Random(93)
    lead, frame = (20, 18, 16), (66, 60, 52)
    red, gold, blue, purple = (156, 34, 36), (208, 156, 52), (36, 66, 156), (96, 46, 128)
    pale = [(120, 160, 140), (130, 150, 180), (190, 170, 110), (110, 150, 120), (150, 140, 170)]
    quarry = {}

    def glass(x, y):
        u, yy = x / w, y / w
        border = min(u, 1 - u, yy, 2 - yy)
        if border < 0.03:
            return frame
        if border < 0.05 or abs(border - 0.13) < 0.016:
            return lead
        if border < 0.13:
            along = yy if min(u, 1 - u) < min(yy, 2 - yy) else u
            f = along / 0.09
            return lead if abs(f % 1 - 0.5) > 0.4 else (red if int(f) % 2 else gold)
        d = math.hypot(u - 0.5, yy - 0.62)
        if abs(d - 0.34) < 0.018 or abs(d - 0.19) < 0.016:
            return lead
        if d < 0.19:
            arm = min(abs(u - 0.5), abs(yy - 0.62))
            return lead if abs(arm - 0.045) < 0.014 else (gold if arm < 0.045 else red)
        if d < 0.34:
            seg = (math.atan2(yy - 0.62, u - 0.5) / (2 * math.pi) * 10) % 1
            return lead if min(seg, 1 - seg) < 0.06 else (blue if int((math.atan2(yy - 0.62, u - 0.5) / (2 * math.pi) * 10) % 10) % 2 else purple)
        a, b = (u + yy) / 0.16, (u - yy) / 0.16
        if min(a % 1, 1 - a % 1, b % 1, 1 - b % 1) < 0.09:
            return lead
        key = (math.floor(a), math.floor(b))
        if key not in quarry:
            quarry[key] = rnd.choice(pale)
        return quarry[key]

    def px(x, y):
        col = glass(x, y)
        if col in (lead, frame):
            return col
        glow = 0.8 + 0.3 * (1 - min(1.0, math.hypot(x / w - 0.5, y / w - 0.8) / 1.0))
        return shade(col, glow * (0.72 + 0.5 * mot(x, y % 256)))
    save(out, grid(w, h, px))


def dourado(out):
    n = 256
    cell = n // 2
    wear = fbm(131, n, 8, 3)

    def height(x, y):
        u, v = (x % cell) / cell * 2 - 1, (y % cell) / cell * 2 - 1
        r, th = math.hypot(u, v), math.atan2(v, u)
        edge = 1 - max(abs(u), abs(v))
        bead = 0.7 * math.cos(edge / 0.09 * math.pi / 2) if edge < 0.09 else 0.0
        reach = 0.42 * (0.6 + 0.4 * abs(math.cos(4 * th)))
        petal = smooth(reach, reach - 0.12, r) * (0.55 + 0.45 * abs(math.cos(4 * th)))
        boss = math.sqrt(max(0.0, 1 - (r / 0.11) ** 2))
        rs = corners(u, v)
        best = min(((math.hypot(u - sx, v - sy), math.atan2(v - sy, u - sx)) for sx in (-1, 1) for sy in (-1, 1)), key=lambda p: p[0])
        s = (best[0] / 0.12 + best[1] / (2 * math.pi)) % 1
        scroll = max(0.0, 1 - abs(s - 0.5) * 4) * max(0.0, 1 - rs / 0.7) * 0.9 if rs < 0.7 else 0.0
        leaf = (0.35 + 0.3 * abs(math.cos(6 * th + 5 * r))) * smooth(0.85, 0.6, r) * smooth(0.3, 0.45, r)
        return max(bead, petal, boss, scroll, leaf, 0.2 * wear(x, y))

    hh = grid(n, n, height)
    light = (-0.45, -0.6, 0.66)

    def px(x, y):
        gx = (hh[y][(x + 1) % n] - hh[y][(x - 1) % n]) * 6
        gy = (hh[(y + 1) % n][x] - hh[(y - 1) % n][x]) * 6
        norm = math.sqrt(gx * gx + gy * gy + 1)
        lam = max(0.0, (-gx * light[0] - gy * light[1] + light[2]) / norm)
        t = min(1.0, lam * 1.15) * (0.55 + 0.45 * min(1.0, hh[y][x] * 3))
        col = mix((44, 24, 8), (180, 130, 48), t * 2) if t < 0.5 else mix((180, 130, 48), (250, 226, 150), (t - 0.5) * 2)
        bole = smooth(0.08, 0.0, hh[y][x]) * 0.6 + 0.3 * smooth(0.6, 0.8, wear(x, y))
        return mix(col, (74, 26, 14), min(0.8, bole))
    save(out, grid(n, n, px))


def agua(seed, deep, light, out):
    n = 256
    rip = fbm(int(seed), n, 6, 4)
    deep, light = tuple(int(c) for c in deep.split(",")), tuple(int(c) for c in light.split(","))
    hh = grid(n, n, lambda x, y: rip(x, (y * 3) % n))
    sun = (0.3, -0.8, 0.52)

    def px(x, y):
        gx = (hh[y][(x + 1) % n] - hh[y][(x - 1) % n]) * 40
        gy = (hh[(y + 1) % n][x] - hh[(y - 1) % n][x]) * 40
        norm = math.sqrt(gx * gx + gy * gy + 1)
        lam = max(0.0, (-gx * sun[0] - gy * sun[1] + sun[2]) / norm)
        return mix(deep, light, min(1.0, 0.35 * hh[y][x] + 0.9 * lam ** 6))
    save(out, grid(n, n, px))


def pau_a_pique(clay_path, wood_path, out):
    n = 256
    clay, wood = load(clay_path, n, n), load(wood_path, n, n)
    intact = fbm(141, n, 4, 4)

    def px(x, y):
        m = intact(x, y)
        c = clay[y][x]
        if m > 0.47:
            return shade(c, 0.62 + 0.38 * smooth(0.47, 0.53, m))
        span = (x // 64) % 2
        pole = x % 64 < 14
        lath = (y + span * 12) % 24 < 10
        if pole:
            return shade(wood[y][x], 0.5 + 0.5 * math.sin((x % 64 + 0.5) / 14 * math.pi))
        if lath:
            return shade(wood[x][y], 0.45 + 0.5 * math.sin(((y + span * 12) % 24 + 0.5) / 10 * math.pi))
        return shade(c, 0.42)
    save(out, grid(n, n, px))


def ceu(kind, out):
    w, h = 512, 256
    rnd = random.Random(111)
    cloud = fbm(113, w, 4, 5)
    stars = {}
    if kind == "noite":
        for _ in range(260):
            sx, sy = rnd.randrange(w // 4) * 4, int(h * rnd.random() ** 1.4) // 4 * 4
            stars[(sx // 4, sy // 4)] = 0.35 + 0.65 * rnd.random() ** 3

    def px(x, y):
        v = y / h
        if kind == "noite":
            base = mix((4, 6, 16), (26, 32, 54), v ** 1.6)
            milky = math.exp(-(((x / w + v * 0.35) % 1 - 0.45) / 0.09) ** 2) * cloud(x, y) * 0.35
            base = mix(base, (60, 64, 86), milky)
            star = stars.get((x // 4, y // 4), 0.0) * (1 - 0.6 * v)
            return mix(base, (236, 236, 255), star)
        c = cloud(x, (y * 2) % w)
        dens = smooth(0.38, 0.68, c)
        base = mix((8, 10, 18), (30, 34, 48), v ** 1.4)
        lit = mix((44, 48, 62), (104, 108, 124), smooth(0.5, 0.8, c) * (1 - v * 0.5))
        return mix(base, lit, dens)
    save(out, grid(w, h, px))


def stamp(img, cx, cy, ang, length, width, col):
    ca, sa = math.cos(ang), math.sin(ang)
    reach = int(length) + 2
    for y in range(max(0, int(cy) - reach), min(len(img), int(cy) + reach + 1)):
        for x in range(max(0, int(cx) - reach), min(len(img[0]), int(cx) + reach + 1)):
            a = ((x - cx) * ca + (y - cy) * sa) / length
            b = (-(x - cx) * sa + (y - cy) * ca) / width
            if a * a + b * b < 1:
                k = 0.7 + 0.3 * (1 - abs(b)) + 0.15 * (0.5 - (a + 1) / 4) - (0.25 if abs(b) < 0.12 else 0.0)
                img[y][x] = shade(col, k) + (255,)


def blank(w, h):
    return [[(0, 0, 0, 0) for _ in range(w)] for _ in range(h)]


def folhas(out):
    n = 256
    rnd = random.Random(121)
    blob = periodic_noise(123, n, 6)
    img = blank(n, n)
    leaves = []
    while len(leaves) < 950:
        x, y = rnd.uniform(0, n), rnd.uniform(0, n)
        if math.hypot((x - 128) / 122, (y - 132) / 118) < 0.72 + 0.3 * blob(int(x), int(y)):
            leaves.append((rnd.random(), x, y, rnd.uniform(0, math.pi), rnd.uniform(8, 13)))
    for depth, x, y, ang, size in sorted(leaves):
        tone = 0.25 + 0.6 * depth * (1 - 0.5 * y / n)
        stamp(img, x, y, ang, size, size * 0.4, mix((16, 30, 16), (92, 122, 58), tone))
    save_rgba(out, img)


def bananeira(out):
    w, h = 256, 512
    rnd = random.Random(151)
    img = blank(w, h)
    for y in range(290, h):
        half = 10 + (y - 290) / 222 * 8
        for x in range(int(128 - half), int(128 + half)):
            t = (x - 128 + half) / (2 * half)
            stripe = 0.85 + 0.15 * math.sin(x * 1.3 + y * 0.05)
            img[y][x] = shade(mix((70, 96, 44), (96, 78, 44), (y - 290) / 222), (0.45 + 0.55 * math.sin(t * math.pi)) * stripe) + (255,)
    leaves = [(-1.25, 0.62, 0.95, True), (1.2, 0.66, 0.9, True), (-0.8, 0.85, 0.55, False), (0.75, 0.85, 0.6, False),
              (-0.35, 0.95, 0.3, False), (0.3, 1.0, 0.25, False), (0.05, 0.75, 0.15, False)]
    for ang, scale, droop, dead in leaves:
        bx, by = 128 + ang * 6, 300
        length = 200 * scale
        green = (88, 92, 40) if dead else (54 + rnd.randrange(20), 100 + rnd.randrange(30), 38)
        tears = rnd.uniform(9, 16)
        for i in range(400):
            t = i / 400
            cx = bx + math.sin(ang) * length * t
            cy = by - math.cos(ang) * length * t + droop * length * t * t * 0.9
            dx = math.sin(ang) * length / 400
            dy = -math.cos(ang) * length / 400 + droop * length * 2 * t * 0.9 / 400
            norm = math.hypot(dx, dy)
            nx, ny = -dy / norm, dx / norm
            half = 30 * scale * math.sin(math.pi * min(1.0, t * 1.05)) ** 0.7 * (0.0 if t < 0.12 else 1.0) + 3
            for s in range(-int(half), int(half) + 1):
                f = s / half
                if abs(f) > 0.25 and (t * tears + abs(f) * 0.6) % 1 < (0.12 if dead else 0.05):
                    continue
                x, y = int(cx + nx * s), int(cy + ny * s)
                if 0 <= x < w and 0 <= y < h:
                    edge = (130, 104, 54) if dead and abs(f) > 0.7 else green
                    rib = abs(f) < 0.08
                    k = (0.55 + 0.45 * (1 - abs(f))) * (0.8 + 0.2 * math.sin(s * 0.9 + t * 60)) * (1.0 if f < 0 else 0.82)
                    img[y][x] = shade((150, 160, 90) if rib else edge, 1.0 if rib else k) + (255,)
    for i in range(60):
        t = i / 60
        for s in range(-int(10 * math.sin(math.pi * t) + 2), int(10 * math.sin(math.pi * t) + 2)):
            x, y = int(150 + s * 0.6 + t * 8), int(290 + t * 60)
            img[y][x] = shade((80, 30, 52), 0.7 + 0.3 * math.sin(s * 0.5)) + (255,)
    save_rgba(out, img)


def bambu(out):
    w, h = 256, 512
    rnd = random.Random(161)
    img = blank(w, h)
    culms = [(34, -0.05, 15), (82, 0.03, 18), (128, -0.02, 20), (170, 0.05, 17), (220, -0.04, 14)]
    for base, lean, width in culms:
        top = rnd.randrange(10, 50)
        node0 = rnd.uniform(0, 60)
        for y in range(top, h):
            cx = base + lean * (h - y)
            for x in range(int(cx - width / 2), int(cx + width / 2) + 1):
                if 0 <= x < w:
                    t = (x - cx + width / 2) / width
                    node = (y + node0) % 70
                    col = (118, 128, 52) if node > 4 else (70, 72, 36)
                    k = (0.4 + 0.6 * math.sin(max(0.0, min(1.0, t)) * math.pi)) * (1.12 if 4 < node < 9 else 1.0)
                    img[y][x] = shade(col, k) + (255,)
    for _ in range(420):
        base, lean, _ = rnd.choice(culms)
        y = rnd.uniform(0, 300) ** 1.0
        cx = base + lean * (h - y) + rnd.uniform(-34, 34)
        ang = math.pi / 2 + rnd.uniform(-1.1, 1.1) + (0.6 if cx > base else -0.6)
        stamp(img, cx, y, ang, rnd.uniform(14, 22), rnd.uniform(3.2, 4.4), mix((40, 62, 26), (110, 140, 56), rnd.random()))
    save_rgba(out, img)


def lua(out):
    n = 128
    maria = fbm(171, n, 5, 4)

    def px(x, y):
        r = math.hypot(x - 63.5, y - 63.5) / 58
        if r >= 1:
            return (0, 0, 0, 0)
        k = (0.78 + 0.22 * math.sqrt(1 - r * r)) * (1 - 0.2 * smooth(0.45, 0.7, maria(x, y)))
        return shade((236, 232, 212), k) + (255,)
    save_rgba(out, grid(n, n, px))


def iron(seed, n):
    rust = fbm(seed, n, 8, 3)
    return lambda x, y, k: shade(mix((62, 62, 68), (110, 62, 34), smooth(0.5, 0.75, rust(x, y))), k) + (255,)


def grade(out):
    n = 256
    paint = iron(181, n)
    gap = n / 5

    def px(x, y):
        bx = (x + gap / 2) % gap - gap / 2
        bar = abs(bx) < 4.5 and 30 <= y <= 240
        tip = 6 <= y < 30 and abs(bx) < 4.5 * (y - 6) / 24 + 0.5
        rail = 30 <= y < 42 or 222 <= y < 234
        ring = abs(math.hypot((x % gap) - gap / 2, y - 132) - 15) < 3.2
        if bar or tip:
            return paint(x, y, 0.75 + 0.6 * (0.5 - bx / 9))
        if rail:
            return paint(x, y, 1.15 if y in (30, 31, 222, 223) else 0.85)
        if ring:
            return paint(x, y, 0.9)
        return (0, 0, 0, 0)
    save_rgba(out, grid(n, n, px))


def portao(out):
    w, h = 256, 512
    paint = iron(191, w)

    def px(x, y):
        arch = 80 - 56 * math.sin(math.pi * x / (w - 1))
        bx = (x - 4) % 32 - 16
        frame = (x < 12 or x >= w - 12 or y >= h - 14) and y >= arch
        archrail = arch <= y < arch + 12
        rail = 200 <= y < 212 or 440 <= y < 452
        bar = abs(bx) < 4 and y >= arch
        tip = arch - 26 <= y < arch and abs(bx) < 4 * (y - arch + 26) / 26 + 0.5
        scroll = 212 <= y < 300 and abs(math.hypot(((x - 4) % 32) - 16, y - 240) - 11) < 2.8
        lock = 290 <= y < 334 and w - 46 <= x < w - 12
        if lock:
            hole = math.hypot(x - (w - 29), y - 306) < 4 or (w - 31 <= x <= w - 27 and 306 <= y < 318)
            return (8, 8, 8, 255) if hole else paint(x, y, 1.0)
        if frame or archrail or rail:
            return paint(x, y, 1.12 if y in (200, 201, 440, 441) else 0.88)
        if bar or tip:
            return paint(x, y, 0.75 + 0.6 * (0.5 - bx / 8))
        if scroll:
            return paint(x, y, 0.9)
        return (0, 0, 0, 0)
    save_rgba(out, grid(w, h, px))


def cruz_ferro(out):
    n = 256
    paint = iron(201, n)

    def px(x, y):
        vert = abs(x - 128) < 11 and 30 <= y <= 250
        horiz = abs(y - 86) < 11 and 46 <= x <= 210
        lobes = any(math.hypot(x - ex + dx, y - ey + dy) < 13 for ex, ey, ax, ay in ((128, 28, 0, 1), (44, 86, 1, 0), (212, 86, -1, 0))
                    for dx, dy in ((0, 0), (ay * 14 - ax * 8, ax * 14 - ay * 8), (-ay * 14 - ax * 8, -ax * 14 - ay * 8)))
        ring = abs(math.hypot(x - 128, y - 86) - 36) < 6
        foot = 236 <= y <= 252 and abs(x - 128) < 24 - (y - 236)
        if vert or horiz:
            k = 0.7 + 0.5 * (0.5 - (x - 128) / 22) if vert and not horiz else 0.7 + 0.5 * (0.5 - (y - 86) / 22)
            return paint(x, y, k)
        if lobes or ring or foot:
            return paint(x, y, 0.95)
        return (0, 0, 0, 0)
    save_rgba(out, grid(n, n, px))


KINDS = {
    "taco": taco, "azulejo": azulejo, "floral": floral, "croche": croche, "vidro": vidro, "porta": porta,
    "listrado": listrado, "calcada": calcada, "marmore": marmore, "xadrez": xadrez, "ladrilho": ladrilho,
    "capela": capela, "vitral": vitral, "dourado": dourado, "agua": agua, "pau_a_pique": pau_a_pique, "ceu": ceu,
    "folhas": folhas, "bananeira": bananeira, "bambu": bambu, "lua": lua, "grade": grade, "portao": portao,
    "cruz_ferro": cruz_ferro,
}


def main(argv):
    if len(argv) < 3 or argv[1] not in KINDS:
        sys.exit("usage: synth.py <%s> [inputs...] <out>" % "|".join(KINDS))
    KINDS[argv[1]](*argv[2:])


if __name__ == "__main__":
    main(sys.argv)
