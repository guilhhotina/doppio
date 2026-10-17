#!/usr/bin/env python3
import http.cookiejar
import io
import json
import math
import os
import random
import re
import struct
import sys
import urllib.parse
import urllib.request
import zipfile

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageFont, ImageOps

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import texpack  # noqa: E402

ROOT = os.path.dirname(HERE)
OUT = os.path.join(ROOT, "assets", "tex")
FONTS = os.path.join(ROOT, "assets", "ui", "fonts.bin")
SRC = os.environ.get("DOPPIO_SRC", "/tmp/doppio-src")
PREVIEW = "/tmp/doppio-tex-preview"

EXTRA = "ªº°«»ÀÁÂÃÄÇÈÉÊÍÑÓÔÕÖÚÜàáâãäçèéêíñóôõöúü"
CHARSET = "".join(chr(c) for c in range(0x20, 0x7F)) + EXTRA
FALLBACK = {"ª": "a", "º": "o", "°": "o", "«": '"', "»": '"'}

CONSOLE = [int(v, 16) for v in """
0x0000000000000000 0x0030003030787830 0x0000000000006c6c 0x006c6cfe6cfe6c6c 0x0030f80c78c07c30 0x00c6663018ccc600 0x0076ccdc76386c38 0x0000000000603030
0x0018306060603018 0x0060301818183060 0x0000663cff3c6600 0x00003030fc303000 0x6030300000000000 0x00000000fc000000 0x0030300000000000 0x00c06030180c0600
0x007cc6c6d6c6c67c 0x00fc303030307030 0x00fccc60380ccc78 0x0078cc0c380ccc78 0x001e0cfecc6c3c1c 0x0078cc0c0cf8c0fc 0x0078ccccf8c06038 0x00303030180cccfc
0x0078cccc78cccc78 0x0070180c7ccccc78 0x0030300000303000 0x6030300000303000 0x00183060c0603018 0x0000fc0000fc0000 0x006030180c183060 0x00300030180ccc78
0x0078c0dededec67c 0x00ccccfccccc7830 0x00fc66667c6666fc 0x003c66c0c0c0663c 0x00f86c6666666cf8 0x00fe6268786862fe 0x00f06068786862fe 0x003e66cec0c0663c
0x00ccccccfccccccc 0x0078303030303078 0x0078cccc0c0c0c1e 0x00e6666c786c66e6 0x00fe6662606060f0 0x00c6c6d6fefeeec6 0x00c6c6cedef6e6c6 0x00386cc6c6c66c38
0x00f060607c6666fc 0x001c78dccccccc78 0x00e6666c7c6666fc 0x0078cc1c70e0cc78 0x007830303030b4fc 0x0078cccccccccccc 0x003078cccccccccc 0x00c6eefed6c6c6c6
0x00c6c66c386cc6c6 0x0078303078cccccc 0x00fe6630180cc6fe 0x0078606060606078 0x00060c183060c000 0x0078181818181878 0x0000000000663c18 0xff00000000000000
0x0000000000183030 0x0076cc7c0c780000 0x00dc66667c6060e0 0x0078ccc0cc780000 0x0076cccc7c0c0c1c 0x0078c0fccc780000 0x00f06060f0606c38 0xf80c7ccccc760000
0x00e66666766c60e0 0x0078303030700030 0x78cccc0c0c0c000c 0x00e66c786c6660e0 0x0078303030303070 0x00c6d6fefecc0000 0x00ccccccccf80000 0x0078cccccc780000
0xf0607c6666dc0000 0x1e0c7ccccc760000 0x00f0606676dc0000 0x00f80c78c07c0000 0x00183430307c3010 0x0076cccccccc0000 0x003078cccccc0000 0x006cfefed6c60000
0x00c66c386cc60000 0xf80c7ccccccc0000 0x00fc643098fc0000 0x001c3030e030301c 0x0030303030303030 0x00e030301c3030e0 0x000000000000dc76 0xffffffffffffffff
0x0000000000000000 0x3078783030003000 0x003078ccc0cc7830 0x00fce660f0646c38 0x0000c67c6c7cc600 0x3030fc30fc78cccc 0x0030303000303030 0x7c863c66663c613e
0x000000000000006c 0x7c829aa2a29a827c 0x0000007e003e6c3c 0x00003366cc663300 0x00000c0cfc000000 0x000000007c000000 0x7c82aab2aab2827c 0x00000000000000ff
0x000000000070d870 0x00fc003030fc3030 0x000000f86030d870 0x00000070d830d870 0x0000000000003018 0xc0f6cccccccc0000 0x001b1b1b3b7bdb7f 0x0000001818000000
0x78cc180000000000 0x000000f06060e060 0x0000007c00386c38 0x0000cc663366cc00 0x03cf6f37dbccc6c3 0x0fcc6633deccc6c3 0xc36f35fb3c6633e0 0x3c66603018001800
0x00ccccfccc783060 0x00ccccfccc783018 0x00ccccfccc788478 0x00ccccfccc78dc76 0x00ccccfccc7800cc 0x00ccccfccc784830 0x00ded8d8fcd8783e 0x780c3c66c0c0663c
0x00fe627862fe3060 0x00fe627862fe180c 0x00fe627862fe6c38 0x00fe627862fe006c 0x0078303030783060 0x0078303030783018 0x007830303078cc78 0x00783030307800cc
0x00f86c66f6666cf8 0x00c6cedef6e6dc76 0x0078cccccc783060 0x0078cccccc783018 0x0078cccccc78cc78 0x0078cccccc78dc76 0x0078cccccc7800cc 0x0000006c38386c00
0x00f86cf6d6de6c3e 0x0078cccccccc3060 0x0078cccccccc3018 0x0078cccccc00cc78 0x0078cccccccc00cc 0x00783078cccc3018 0x00f0607c667c60f0 0x80dcd6c6ccc6c67c
0x0076cc7c0c783060 0x0076cc7c0c783018 0x0076cc7c0c788478 0x0076cc7c0c78dc76 0x0076cc7c0c78006c 0x0076cc7c0c786c38 0x007ed87e1b7e0000 0x701878ccc0cc7800
0x0078c0fccc783060 0x0078c0fccc78180c 0x0078c0fccc788478 0x0078c0fccc7800cc 0x0078303070003060 0x0078303070003018 0x007830307000d870 0x007830307000d800
0x0078cccc7c187078 0x00ccccccccf8dc76 0x0078cccc78003060 0x0078cccc78003018 0x0078cccc7800cc78 0x0078cccc7800dc76 0x0078cccc7800cc00 0x00003000fc003000
0x00f8ecfcdc7c0000 0x0076cccccccc3060 0x0076cccccccc3018 0x0076cccccc00cc78 0x0076cccccccc00cc 0xf80c7ccccccc3018 0xf060786c6c7860f0 0xf80c7ccccccc00cc
""".split()]


def ph(name):
    path = os.path.join(SRC, "ph", name + ".jpg")
    if not os.path.exists(path):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        files = json.load(urllib.request.urlopen("https://api.polyhaven.com/files/" + name))
        with open(path, "wb") as f:
            f.write(urllib.request.urlopen(files["Diffuse"]["2k"]["jpg"]["url"]).read())
    return Image.open(path).convert("RGB")


def itch(page, upload, member):
    path = os.path.join(SRC, "fonts", os.path.basename(member))
    if os.path.exists(path):
        return path
    jar = http.cookiejar.CookieJar()
    web = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
    token = lambda html: re.search(r'csrf_token" value="([^"]*)"', html).group(1)
    post = lambda url, tok: json.load(web.open(url, urllib.parse.urlencode({"csrf_token": tok}).encode()))["url"]
    landing = post(page + "/download_url", token(web.open(page).read().decode()))
    archive = post(page + "/file/%d?source=game_download" % upload, token(web.open(landing).read().decode()))
    blob = zipfile.ZipFile(io.BytesIO(urllib.request.urlopen(archive).read()))
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as f:
        f.write(blob.read(member))
    return path


def serif_font():
    ttf = itch("https://not-jam.itch.io/not-jam-font-pack", 8327656, "NotJamFontPack/Not Jam Old Style 11/NotJamOldStyle14.ttf")
    return ImageFont.truetype(ttf, 14)


def save(name, img, colors=16, dither=True):
    img = img.convert("RGBA")
    w, h = img.size
    alpha = name.endswith("_a")
    mask = img.getchannel("A").point(lambda a: 255 if a >= 128 else 0)
    rgb = Image.new("RGB", img.size, (0, 0, 0))
    rgb.paste(img.convert("RGB"), mask=mask if alpha else None)
    n = min(colors, 15) if alpha else colors
    if dither:
        opaque = [p for p, m in zip(rgb.getdata(), mask.getdata()) if m or not alpha]
        strip = Image.new("RGB", (len(opaque), 1))
        strip.putdata(opaque)
        pal = strip.quantize(colors=n, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
        q = rgb.quantize(palette=pal, dither=Image.Dither.FLOYDSTEINBERG).convert("RGB")
    else:
        q = rgb
    data = texpack.pack(w, h, q.tobytes(), mask.tobytes() if alpha else None)
    with open(os.path.join(OUT, name + ".tex"), "wb") as f:
        f.write(data)
    os.makedirs(PREVIEW, exist_ok=True)
    view = Image.open(io.BytesIO(texpack.unpack(data))).convert("RGB")
    if alpha:
        bg = Image.new("RGB", view.size, (40, 48, 58))
        idx = [b for byte in data[34:] for b in (byte & 15, byte >> 4)]
        keep = Image.new("L", view.size)
        keep.putdata([0 if i == 0 else 255 for i in idx])
        bg.paste(view, mask=keep)
        view = bg
    view.resize((w * 4, h * 4), Image.NEAREST).save(os.path.join(PREVIEW, name + ".png"))
    print("%s %dx%d %d bytes" % (name, w, h, len(data)))


INK = [255, 238, 214, 190, 166]
RING = (10, 8, 7, 255)


def outlined(bits, w, h, band, edge=True):
    pad = 1 if edge else 0
    out = Image.new("RGBA", (w + 2 * pad, h + 2 * pad), (0, 0, 0, 0))
    px = out.load()
    on = lambda x, y: 0 <= x < w and 0 <= y < h and bits[y][x]
    for y in range(h + 2 * pad):
        for x in range(w + 2 * pad):
            if on(x - pad, y - pad):
                v = INK[min(band(y - pad), len(INK) - 1)]
                px[x, y] = (v, v, v, 255)
            elif edge and any(on(x - 1 + dx, y - 1 + dy) for dx in (-1, 0, 1) for dy in (-1, 0, 1)):
                px[x, y] = RING
    return out


def console_glyph(c):
    code = ord(c)
    i = code - 0x20 if code < 0x7F else code - 0xA0 + 96
    g = CONSOLE[i]
    return [[(g >> (r * 8)) >> (7 - x) & 1 for x in range(8)] for r in range(8)]


def trim(bits):
    cols = [x for x in range(len(bits[0])) if any(row[x] for row in bits)]
    if not cols:
        return [], 0
    return [row[cols[0]:cols[-1] + 1] for row in bits], cols[-1] - cols[0] + 1


def serif_glyph(font, c):
    asc, _ = font.getmetrics()
    im = Image.new("1", (32, 32), 0)
    d = ImageDraw.Draw(im)
    d.fontmode = "1"
    d.text((4, 4), c, font=font, fill=1)
    box = im.getbbox()
    adv = round(font.getlength(c))
    if not box:
        return [], 0, 0, adv
    bits = [[im.getpixel((x, y)) for x in range(box[0], box[2])] for y in range(box[1], box[3])]
    return bits, box[2] - box[0], box[1] - 4 - (asc - 11), adv


def fonts():
    sheet = Image.new("RGBA", (256, 256), (0, 0, 0, 0))
    meta = []
    x, y, shelf = 0, 0, 0

    def place(img):
        nonlocal x, y, shelf
        w, h = img.size
        if x + w > 256:
            x, y, shelf = 0, y + shelf, 0
        assert y + h <= 256, "font sheet overflow"
        sheet.paste(img, (x, y))
        at = (x, y)
        x, shelf = x + w, max(shelf, h)
        return at

    serif = serif_font()
    missing = serif_glyph(serif, "\ue000")[0]

    def serif_bits(c):
        g = serif_glyph(serif, c)
        return serif_glyph(serif, FALLBACK.get(c, "?")) if g[0] == missing and c != " " else g

    for edge in (True, False):
        pad = 1 if edge else 0
        for c in CHARSET:
            bits, w = trim(console_glyph(c))
            if w == 0:
                meta.append((0, 0, 0, 0, 0, 4))
                continue
            sx, sy = place(outlined(bits, w, 8, lambda r: r // 2, edge))
            meta.append((sx, sy, w + 2 * pad, 8 + 2 * pad, -pad, w + 1))
        x, y, shelf = 0, y + shelf, 0
        for c in CHARSET:
            bits, w, top, adv = serif_bits(c)
            if w == 0:
                meta.append((0, 0, 0, 0, 0, max(adv, 4)))
                continue
            sx, sy = place(outlined(bits, w, len(bits), lambda r, t=top: (r + t) // 3, edge))
            meta.append((sx, sy, w + 2 * pad, len(bits) + 2 * pad, top - pad, max(adv, w + 1)))
        x, y, shelf = 0, y + shelf, 0
    solid = place(Image.new("RGBA", (8, 8), (255, 255, 255, 255)))
    os.makedirs(os.path.dirname(FONTS), exist_ok=True)
    with open(FONTS, "wb") as f:
        f.write(bytes(solid))
        f.write(b"".join(struct.pack("<BBBBbB", *m) for m in meta))
    save("ui_font_a", sheet, dither=False)


S = 4


def tex_image(name):
    with open(os.path.join(OUT, name + ".tex"), "rb") as f:
        return Image.open(io.BytesIO(texpack.unpack(f.read()))).convert("RGB")


def material(photo, size, colors, seed=0, zoom=1.0):
    img = ph(photo) if isinstance(photo, str) else photo
    w, h = size
    rnd = random.Random(seed)
    cw, ch = int(img.width * 0.35 / zoom), int(img.width * 0.35 / zoom * h / w)
    x, y = rnd.randint(0, img.width - cw), rnd.randint(0, img.height - ch)
    g = ImageOps.autocontrast(img.crop((x, y, x + cw, y + ch)).resize((w, h), Image.LANCZOS).convert("L"), cutoff=2)
    if len(colors) == 3:
        return ImageOps.colorize(g, colors[0], colors[2], colors[1])
    return ImageOps.colorize(g, colors[0], colors[1])


def tiled(img, size):
    out = Image.new("RGB", size)
    for y in range(0, size[1], img.height):
        for x in range(0, size[0], img.width):
            out.paste(img, (x, y))
    return out


def blur(mask, r):
    return mask.filter(ImageFilter.GaussianBlur(r))


def scale(l, k):
    return l.point(lambda v: min(255, int(v * k)))


def lit(rgb, mask, d=2 * S, hi=1.2, lo=1.4, r=1.5 * S):
    b = blur(mask, r)
    sh = ImageChops.offset(b, int(d), int(d))
    up = ImageChops.subtract(b, sh)
    dn = ImageChops.subtract(sh, b)
    rgb = ImageChops.add(rgb, Image.merge("RGB", [scale(up, hi)] * 3))
    return ImageChops.subtract(rgb, Image.merge("RGB", [scale(dn, lo)] * 3))


def ring(mask, px):
    return mask.filter(ImageFilter.MaxFilter(px * 2 + 1))


def layer(canvas, rgb, mask, at=(0, 0)):
    canvas.paste(rgb, at, mask)


def shape(size, draw):
    m = Image.new("L", size, 0)
    draw(ImageDraw.Draw(m))
    return m


def rough(mask, seed, amount=0.5, grain=3 * S):
    w, h = mask.size
    rnd = random.Random(seed)
    n = Image.new("L", (max(1, w // grain), max(1, h // grain)))
    n.putdata([rnd.randint(0, 255) for _ in range(n.width * n.height)])
    n = n.resize((w, h), Image.BICUBIC)
    b = blur(mask, grain * 0.6)
    noise = n.point(lambda v: int(v * amount))
    return Image.eval(ImageChops.add(b, noise, 1.0, -int(127 * amount)), lambda v: 255 if v >= 128 else 0)


def down(img, w, h):
    rgb = img.convert("RGB").resize((w, h), Image.LANCZOS)
    a = img.getchannel("A").resize((w, h), Image.LANCZOS)
    out = rgb.convert("RGBA")
    out.putalpha(a)
    return out


BRASS = ("#1e1206", "#9a7428", "#f4dc8c")
IVORY = ("#3a3020", "#c8bc98", "#f6f0dc")
WAX = ("#5a4a34", "#e0d6bc", "#fbf6e6")


def brass(size, seed, grime=0.35):
    w, h = size
    smooth = material("blue_metal_plate", size, BRASS, seed)
    spots = material("rusty_metal_02", size, ("#2a1a08", "#6a4a1a", "#9a7a3a"), seed + 1)
    mask = material("rusty_metal_02", size, ("#000000", "#ffffff"), seed + 2).convert("L").point(lambda v: 255 if v < 255 * grime else 0)
    sheen = Image.linear_gradient("L").resize(size).point(lambda v: int(170 + 85 * math.cos(math.pi * (v / 255 - 0.3))))
    out = ImageChops.multiply(smooth, Image.merge("RGB", [sheen] * 3))
    out.paste(spots, (0, 0), blur(mask, S * 0.7))
    return out


def padlock():
    w, h = 80 * S, 96 * S
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    shackle = shape((w, h), lambda d: d.rounded_rectangle((14 * S, 1 * S, 66 * S, 60 * S), radius=24 * S, outline=255, width=8 * S))
    steel = material("rusty_metal", (w, h), ("#141414", "#7c7a74", "#e2e0d8"), 3)
    layer(img, Image.new("RGB", (w, h), (8, 6, 4)), ring(shackle, S))
    layer(img, lit(steel, shackle, hi=1.6, lo=1.2), shackle)
    body = shape((w, h), lambda d: d.rounded_rectangle((2 * S, 32 * S, 78 * S, 95 * S), radius=7 * S, fill=255))
    face = lit(brass((w, h), 7), body, d=3 * S, hi=1.5, lo=1.8, r=2.5 * S)
    layer(img, Image.new("RGB", (w, h), (10, 7, 3)), ring(body, S))
    layer(img, face, body)
    for y in (38, 88):
        line = shape((w, h), lambda d, y=y: d.rectangle((8 * S, y * S, 72 * S, y * S + S), fill=255))
        layer(img, Image.new("RGB", (w, h), (60, 40, 12)), line)
    slot = shape((w, h), lambda d: d.rounded_rectangle((7 * S, 49 * S, 73 * S, 79 * S), radius=3 * S, fill=255))
    layer(img, lit(Image.new("RGB", (w, h), (30, 20, 8)), ImageOps.invert(slot), d=2 * S, hi=0.0, lo=0.0), slot)
    hole = shape((w, h), lambda d: [d.ellipse((38 * S, 81 * S, 42 * S, 85 * S), fill=255), d.polygon([(39 * S, 84 * S), (41 * S, 84 * S), (41.6 * S, 88 * S), (38.4 * S, 88 * S)], fill=255)])
    layer(img, Image.new("RGB", (w, h), (10, 6, 3)), hole)
    rivets = shape((w, h), lambda d: [d.ellipse(((x - 1.5) * S, (y - 1.5) * S, (x + 1.5) * S, (y + 1.5) * S), fill=255) for x in (7, 73) for y in (37, 90)])
    layer(img, lit(material("rusty_metal_02", (w, h), ("#3a2a10", "#c8a050", "#fff0b0"), 9), rivets, d=S, r=S), rivets)
    return down(img, 80, 96)


def wheel(on):
    w, h = 14 * S, 22 * S
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    m = shape((w, h), lambda d: d.rounded_rectangle((0, 0, w - 1, h - 1), radius=3 * S, fill=255))
    tones = ("#2a1a08", "#c89c40", "#fff4c0") if on else ("#1a1006", "#8a6a2a", "#e8cc80")
    base = material("rusty_metal_02", (w, h), tones, 11 if on else 12)
    shade = Image.linear_gradient("L").rotate(90).resize((w, h))
    curve = shade.point(lambda v: int(255 * math.sin(math.pi * v / 255) ** 0.8))
    base = ImageChops.multiply(base, Image.merge("RGB", [curve] * 3))
    for y in range(0, h, 3 * S):
        ImageDraw.Draw(base).rectangle((0, y, w, y + S - 1), fill=(30, 20, 8) if not on else (60, 40, 12))
    face = shape((w, h), lambda d: d.rectangle((2 * S, 5 * S, w - 2 * S - 1, h - 5 * S - 1), fill=255))
    base.paste(material("rusty_metal_02", (w, h), ("#4a3410", "#d8c08a", "#fffbe8") if on else ("#3a2a10", "#b89c62", "#efe2b8"), 13), (0, 0), face)
    layer(img, base, m)
    return down(img, 14, 22)


def compass():
    w = h = 32 * S
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    disc = shape((w, h), lambda d: d.ellipse((0, 0, w - 1, h - 1), fill=255))
    layer(img, Image.new("RGB", (w, h), (10, 7, 3)), disc)
    rim = shape((w, h), lambda d: d.ellipse((S, S, w - S - 1, h - S - 1), fill=255))
    layer(img, lit(brass((w, h), 21), rim, d=2 * S, hi=1.5, lo=1.6), rim)
    face = shape((w, h), lambda d: d.ellipse((5 * S, 5 * S, w - 5 * S - 1, h - 5 * S - 1), fill=255))
    paper = material(tex_image("ui_papel").resize((256, 256), Image.NEAREST), (w, h), ("#4a3c28", "#b8a880", "#e8dcb8"), 4, 2.0)
    layer(img, paper, face)
    marks = shape((w, h), lambda d: [d.line((w / 2 + math.sin(a) * 10 * S, h / 2 - math.cos(a) * 10 * S, w / 2 + math.sin(a) * 12 * S, h / 2 - math.cos(a) * 12 * S), fill=255, width=S) for a in [k * math.pi / 4 for k in range(8)]])
    layer(img, Image.new("RGB", (w, h), (40, 30, 20)), marks)
    north = shape((w, h), lambda d: d.polygon([(w / 2, 6 * S), (w / 2 - 2.5 * S, 11 * S), (w / 2 + 2.5 * S, 11 * S)], fill=255))
    layer(img, Image.new("RGB", (w, h), (150, 24, 18)), north)
    glare = shape((w, h), lambda d: d.arc((7 * S, 7 * S, w - 7 * S, h - 7 * S), 200, 250, fill=255, width=S))
    layer(img, Image.new("RGB", (w, h), (250, 246, 230)), glare)
    return down(img, 32, 32)


def battery():
    w, h = 30 * S, 12 * S
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    shell = shape((w, h), lambda d: d.rounded_rectangle((0, 0, 26 * S, h - 1), radius=2 * S, fill=255))
    cap = shape((w, h), lambda d: d.rectangle((26 * S, 3 * S, 29 * S, 8 * S), fill=255))
    layer(img, lit(material("rusty_metal", (w, h), ("#060606", "#3a3a3a", "#9a9a90"), 5), shell, hi=1.4), shell)
    layer(img, lit(brass((w, h), 6), cap, d=S, r=S), cap)
    slot = shape((w, h), lambda d: d.rectangle((2 * S, 2 * S, 24 * S - 1, h - 2 * S - 1), fill=255))
    layer(img, Image.new("RGB", (w, h), (4, 3, 2)), slot)
    return down(img, 30, 12)


def candle(frame):
    w, h = 8 * S, 18 * S
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    stick = shape((w, h), lambda d: d.rectangle((2 * S, 7 * S, 6 * S - 1, h - 1), fill=255))
    drip = shape((w, h), lambda d: [d.rectangle((1 * S, 7 * S, 2 * S, 10 * S), fill=255), d.rectangle((5 * S, 7 * S, 6 * S, 12 * S), fill=255)])
    m = ImageChops.lighter(stick, drip)
    layer(img, Image.new("RGB", (w, h), (20, 14, 8)), ring(m, S // 2))
    wax = material("white_plaster_02", (w, h), WAX, 3)
    shade = Image.linear_gradient("L").rotate(90).resize((w, h)).point(lambda v: int(150 + 105 * math.sin(math.pi * v / 255)))
    layer(img, ImageChops.multiply(wax, Image.merge("RGB", [shade] * 3)), m)
    wick = shape((w, h), lambda d: d.rectangle((3.5 * S, 5 * S, 4.5 * S, 7 * S), fill=255))
    layer(img, Image.new("RGB", (w, h), (24, 16, 10)), wick)
    if frame < 3:
        lean = (-0.6, 0.0, 0.6)[frame] * S
        tall = (5.5, 6.5, 5.0)[frame] * S
        outer = shape((w, h), lambda d: d.ellipse((4 * S - 1.6 * S + lean, 5 * S - tall, 4 * S + 1.6 * S + lean, 5.6 * S), fill=255))
        inner = shape((w, h), lambda d: d.ellipse((4 * S - 0.8 * S + lean / 2, 5 * S - tall * 0.55, 4 * S + 0.8 * S + lean / 2, 5.2 * S), fill=255))
        layer(img, Image.new("RGB", (w, h), (232, 110, 30)), outer)
        layer(img, Image.new("RGB", (w, h), (255, 236, 150)), inner)
    else:
        smoke = shape((w, h), lambda d: d.line((4 * S, 4.5 * S, 3 * S, 2.5 * S, 4.5 * S, 0.5 * S), fill=255, width=S))
        layer(img, Image.new("RGB", (w, h), (90, 86, 80)), smoke)
    return down(img, 8, 18)


def button(letter, tone):
    w = h = 13 * S
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    disc = shape((w, h), lambda d: d.ellipse((0, 0, w - 1, h - 1), fill=255))
    layer(img, Image.new("RGB", (w, h), (8, 6, 4)), disc)
    face = shape((w, h), lambda d: d.ellipse((S, S, w - S - 1, h - S - 1), fill=255))
    layer(img, lit(material("white_plaster_02", (w, h), tone, ord(letter[0])), face, d=1.5 * S, hi=1.6, lo=2.0, r=S), face)
    out = down(img, 13, 13)
    bits, gw = trim(console_glyph(letter))
    px = out.load()
    for y, row in enumerate(bits[:7]):
        for x, b in enumerate(row):
            if b:
                px[(13 - gw) // 2 + x, 3 + y] = (24, 16, 10, 255)
    return out


def key(direction):
    w = h = 11 * S
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    box = shape((w, h), lambda d: d.rounded_rectangle((0, 0, w - 1, h - 1), radius=2 * S, fill=255))
    layer(img, Image.new("RGB", (w, h), (8, 6, 4)), box)
    face = shape((w, h), lambda d: d.rounded_rectangle((S, S, w - S - 1, h - S - 1), radius=S, fill=255))
    layer(img, lit(material("white_plaster_02", (w, h), IVORY, 17), face, d=1.5 * S, hi=1.6, lo=2.0, r=S), face)
    tri = [(w / 2, 2.5 * S), (2.8 * S, 7.5 * S), (w - 2.8 * S, 7.5 * S)]
    arrow = shape((w, h), lambda d: d.polygon(tri, fill=255)).rotate({"up": 0, "left": 90}[direction])
    layer(img, Image.new("RGB", (w, h), (26, 18, 12)), arrow)
    return down(img, 11, 11)


def start_key():
    w, h = 26 * S, 11 * S
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    pill = shape((w, h), lambda d: d.rounded_rectangle((0, 0, w - 1, h - 1), radius=5 * S, fill=255))
    layer(img, Image.new("RGB", (w, h), (8, 6, 4)), pill)
    face = shape((w, h), lambda d: d.rounded_rectangle((S, S, w - S - 1, h - S - 1), radius=4 * S, fill=255))
    layer(img, lit(material("white_plaster_02", (w, h), IVORY, 19), face, d=1.5 * S, hi=1.6, lo=2.0, r=S), face)
    out = down(img, 26, 11)
    tiny = {"S": ["111", "100", "111", "001", "111"], "T": ["111", "010", "010", "010", "010"], "A": ["010", "101", "111", "101", "101"], "R": ["110", "101", "110", "101", "101"]}
    px = out.load()
    for i, c in enumerate("START"):
        for y, row in enumerate(tiny[c]):
            for x, b in enumerate(row):
                if b == "1":
                    px[4 + i * 4 + x, 3 + y] = (24, 16, 10, 255)
    return out


def pointer():
    w, h = 12 * S, 11 * S
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    blade = shape((w, h), lambda d: d.polygon([(0, 2 * S), (7 * S, 2 * S), (11.5 * S, 5.5 * S), (7 * S, 9 * S), (0, 9 * S), (3 * S, 5.5 * S)], fill=255))
    layer(img, Image.new("RGB", (w, h), (8, 6, 4)), ring(blade, S))
    layer(img, lit(brass((w, h), 23), blade, d=1.5 * S, hi=1.6, lo=1.8, r=S), blade)
    return down(img, 12, 11)


LATAO = {
    "padlock": (0, 0),
    "wheel": (80, 0),
    "wheel_on": (94, 0),
    "compass": (80, 22),
    "battery": (80, 54),
    "candle0": (80, 66),
    "candle1": (88, 66),
    "candle2": (96, 66),
    "candle_out": (104, 66),
    "btn_a": (0, 96),
    "btn_b": (13, 96),
    "btn_x": (26, 96),
    "btn_y": (39, 96),
    "key_up": (52, 96),
    "key_left": (63, 96),
    "start": (74, 96),
    "pointer": (100, 96),
}


def latao():
    sheet = Image.new("RGBA", (128, 128), (0, 0, 0, 0))
    parts = {
        "padlock": padlock(),
        "wheel": wheel(False),
        "wheel_on": wheel(True),
        "compass": compass(),
        "battery": battery(),
        "candle0": candle(0),
        "candle1": candle(1),
        "candle2": candle(2),
        "candle_out": candle(3),
        "btn_a": button("A", ("#24301a", "#8aa070", "#e6f0d0")),
        "btn_b": button("B", ("#3a1810", "#b07060", "#f4dcd0")),
        "btn_x": button("X", ("#18243a", "#7088b0", "#d8e2f4")),
        "btn_y": button("Y", ("#3a3010", "#b8a050", "#f6eccc")),
        "key_up": key("up"),
        "key_left": key("left"),
        "start": start_key(),
        "pointer": pointer(),
    }
    for name, img in parts.items():
        sheet.alpha_composite(img, LATAO[name])
    save("ui_latao_a", sheet)


def torn(w, h, seed, vertical=False):
    rnd = random.Random(seed)
    W, H = w * S, h * S
    paper = tiled(tex_image("ui_papel"), (64, 64)).resize((64 * S, 64 * S), Image.NEAREST).crop((0, 0, W, H))
    edge, walk = [], H * 0.45
    for _ in range(W if not vertical else H):
        walk = min(H * 0.85 if not vertical else W * 0.85, max(S * 0.5, walk + rnd.uniform(-1.6, 1.6) * S * 0.6))
        edge.append(walk)
    span = W if not vertical else H
    mask = Image.new("L", (W, H), 0)
    d = ImageDraw.Draw(mask)
    fringe = Image.new("L", (W, H), 0)
    f = ImageDraw.Draw(fringe)
    for i in range(span):
        e = edge[i] if not vertical else edge[i] * W / H
        if not vertical:
            d.line((i, e, i, H), fill=255)
            f.line((i, e, i, e + S * 1.2), fill=255)
        else:
            d.line((e, i, W, i), fill=255)
            f.line((e, i, e + S * 1.2, i), fill=255)
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    layer(img, paper, mask)
    layer(img, Image.new("RGB", (W, H), (120, 100, 70)), ImageChops.multiply(fringe, mask))
    return down(img, w, h)


def tape():
    w, h = 28 * S, 11 * S
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    m = shape((w, h), lambda d: d.polygon([(S, S), (w - 2 * S, 0), (w - S, h * 0.4), (w - 2 * S, h - S), (S * 2, h - 1), (0, h * 0.6)], fill=255))
    base = material("white_plaster_02", (w, h), ("#8a7a50", "#d8c888", "#f4ecc0"), 31)
    bayer = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5]
    out = down(Image.merge("RGBA", (*base.split(), m)), 28, 11)
    a = out.getchannel("A").load()
    for y in range(11):
        for x in range(28):
            a[x, y] = 255 if a[x, y] >= 128 and bayer[(y % 4) * 4 + x % 4] < 12 else 0
    return out


def pin(color):
    w = h = 8 * S
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    head = shape((w, h), lambda d: d.ellipse((S, 0, w - S - 1, h - 2 * S - 1), fill=255))
    shadow = shape((w, h), lambda d: d.ellipse((2 * S, 2 * S, w - 1, h - 1), fill=255))
    layer(img, Image.new("RGB", (w, h), (12, 8, 6)), shadow)
    layer(img, lit(material("rusty_metal_02", (w, h), color, 41), head, d=1.5 * S, hi=1.8, lo=1.4, r=S), head)
    return down(img, 8, 8)


def vignette():
    n = 48
    bayer = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5]
    img = Image.new("RGBA", (n, n), (0, 0, 0, 0))
    px = img.load()
    for y in range(n):
        for x in range(n):
            r = math.hypot(x / n, y / n * 1.15)
            dark = max(0.0, min(1.0, (1.0 - r) * 1.5 - 0.15)) ** 1.6
            if dark * 16 > bayer[(y % 4) * 4 + x % 4] + 0.5:
                px[x, y] = (255, 255, 255, 255)
    return img


def photo_corner():
    img = Image.new("RGBA", (10, 10), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.polygon([(0, 0), (9, 0), (0, 9)], fill=(18, 14, 12, 255))
    d.line((1, 8, 8, 1), fill=(60, 52, 44, 255))
    return img


def stain(seed):
    w = h = 24 * S
    rnd = random.Random(seed)
    ringm = shape((w, h), lambda d: d.arc((2 * S, 3 * S, w - 2 * S, h - 2 * S), 20, 330, fill=255, width=int(1.4 * S)))
    inner = shape((w, h), lambda d: d.ellipse((4 * S, 5 * S, w - 4 * S, h - 4 * S), fill=255))
    speck = Image.new("L", (w, h), 0)
    speck.putdata([255 if rnd.random() < 0.18 else 0 for _ in range(w * h)])
    m = ImageChops.lighter(ringm, ImageChops.multiply(inner, blur(speck, S).point(lambda v: 255 if v > 70 else 0)))
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    layer(img, material("rusty_metal", (w, h), ("#5a3818", "#8a6438", "#a87c4c"), seed), m)
    return down(img, 24, 24)


MOLDURA = {
    "torn_h": (0, 0),
    "torn_v": (0, 8),
    "tape": (8, 8),
    "pin": (36, 8),
    "pin_red": (44, 8),
    "corner": (52, 8),
    "stain": (8, 20),
    "vignette": (64, 8),
    "blood": (32, 20),
}


def blood():
    w, h = 32 * S, 44 * S
    rnd = random.Random(5)
    m = Image.new("L", (w, h), 0)
    d = ImageDraw.Draw(m)
    cx, cy = 15 * S, 12 * S
    d.ellipse((cx - 7 * S, cy - 6 * S, cx + 8 * S, cy + 7 * S), fill=255)
    for _ in range(40):
        a = rnd.uniform(0, math.tau)
        r = rnd.uniform(7, 15) * S
        s_ = rnd.uniform(0.4, 1.8) * S * (1.4 - r / (15 * S))
        x, y = cx + math.cos(a) * r, cy + math.sin(a) * r * 0.8
        d.ellipse((x - s_, y - s_, x + s_, y + s_), fill=255)
    for _ in range(4):
        x = cx + rnd.uniform(-6, 6) * S
        ln = rnd.uniform(10, 28) * S
        wd = rnd.uniform(0.6, 1.3) * S
        d.rounded_rectangle((x - wd, cy, x + wd, min(h - 2 * S, cy + ln)), radius=wd, fill=255)
        d.ellipse((x - wd * 1.6, min(h - 2 * S, cy + ln) - wd * 2, x + wd * 1.6, min(h - 2 * S, cy + ln) + wd), fill=255)
    m = rough(m, 6, 0.25, S)
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    layer(img, lit(material("rusty_metal", (w, h), ("#160202", "#5a0806", "#a82014"), 8), m, d=S, hi=0.9, lo=0.7, r=S), m)
    return down(img, 32, 44)


def moldura():
    sheet = Image.new("RGBA", (128, 64), (0, 0, 0, 0))
    parts = {
        "torn_h": torn(128, 6, 1),
        "torn_v": torn(6, 56, 2, vertical=True),
        "tape": tape(),
        "pin": pin(BRASS),
        "pin_red": pin(("#2a0604", "#a02018", "#f08070")),
        "corner": photo_corner(),
        "stain": stain(3),
        "vignette": vignette(),
        "blood": blood(),
    }
    for name, img in parts.items():
        sheet.alpha_composite(img, MOLDURA[name])
    save("ui_moldura_a", sheet)


def logo():
    path = os.path.join(SRC, "fonts", "IMFeENsc28P.ttf")
    if not os.path.exists(path):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "wb") as f:
            f.write(urllib.request.urlopen("https://github.com/google/fonts/raw/main/ofl/imfellenglishsc/IMFeENsc28P.ttf").read())
    font = ImageFont.truetype(path, 58 * S)
    w, h = 256 * S, 64 * S
    text = shape((w, h), lambda d: d.text((w / 2, h / 2 - 1 * S), "DOPPIO", font=font, fill=255, anchor="mm"))
    text = rough(text, 9, 0.35, 2 * S)
    rnd = random.Random(12)
    scratches = shape((w, h), lambda d: [d.line((x, rnd.uniform(0, h), x + rnd.uniform(-20, 20) * S, rnd.uniform(0, h)), fill=255, width=S) for x in [rnd.uniform(0, w) for _ in range(14)]])
    body = ImageChops.subtract(text, scratches)
    bone = material(tiled(tex_image("ui_papel"), (128, 128)).resize((512, 512), Image.NEAREST), (w, h), ("#2a2016", "#b8a888", "#f2ead2"), 2, 1.4)
    bone = lit(bone, body, d=2 * S, hi=1.3, lo=1.8, r=2 * S)
    drips = Image.new("L", (w, h), 0)
    d = ImageDraw.Draw(drips)
    tp = text.load()
    for x in [int(rnd.uniform(30, 226) * S) for _ in range(11)]:
        rows = [y for y in range(h) if tp[x, y]]
        if not rows:
            continue
        top = rows[-1] - 2 * S
        ln = rnd.uniform(3, 13) * S
        wd = rnd.uniform(0.7, 1.4) * S
        d.rounded_rectangle((x - wd, top, x + wd, min(h - 2 * S, top + ln)), radius=wd, fill=255)
        d.ellipse((x - wd * 1.5, min(h - 2 * S, top + ln) - wd * 2.2, x + wd * 1.5, min(h - 2 * S, top + ln) + wd * 0.6), fill=255)
    smear = ImageChops.multiply(text, Image.linear_gradient("L").resize((w, h)).point(lambda v: 255 if v > 150 else 0))
    red = material("rusty_metal", (w, h), ("#1a0302", "#6a0c08", "#b02818"), 8)
    img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    edge = ring(ImageChops.lighter(text, drips), 2 * S)
    layer(img, Image.new("RGB", (w, h), (6, 4, 3)), edge)
    layer(img, bone, body)
    layer(img, ImageChops.multiply(bone, red), ImageChops.multiply(smear, rough(smear, 4, 0.6, 3 * S)))
    layer(img, lit(red, drips, d=S, hi=0.8, lo=0.6, r=S), drips)
    save("ui_logo_a", down(img, 256, 64))


def main(argv):
    jobs = {"fonts": fonts, "latao": latao, "moldura": moldura, "logo": logo}
    for name in argv[1:] or list(jobs):
        jobs[name]()


if __name__ == "__main__":
    main(sys.argv)
