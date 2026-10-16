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


def main(argv):
    kind = argv[1]
    if kind == "taco":
        taco(argv[2], argv[3])
    elif kind == "azulejo":
        azulejo(argv[2])
    elif kind == "floral":
        floral(argv[2], argv[3])
    elif kind == "croche":
        croche(argv[2])
    elif kind == "vidro":
        vidro(argv[2])
    elif kind == "porta":
        porta(argv[2], argv[3])
    else:
        sys.exit("usage: synth.py taco <grain> <out> | azulejo <out> | floral <paper> <out> | croche <out> | vidro <out> | porta <grain> <out>")


if __name__ == "__main__":
    main(sys.argv)
