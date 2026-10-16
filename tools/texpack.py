#!/usr/bin/env python3
import struct
import sys


def expand565(c):
    r, g, b = (c >> 11) & 31, (c >> 5) & 63, c & 31
    return ((r << 3) | (r >> 2), (g << 2) | (g >> 4), (b << 3) | (b >> 2))


def to565(rgb):
    r, g, b = rgb
    return ((min(31, (r * 31 + 127) // 255)) << 11) | ((min(63, (g * 63 + 127) // 255)) << 5) | min(31, (b * 31 + 127) // 255)


def pixels(raw):
    return [tuple(raw[i:i + 3]) for i in range(0, len(raw), 3)]


def ppm(w, h, rgbs):
    return b"P6\n%d %d\n255\n" % (w, h) + bytes(v for p in rgbs for v in p)


def log2(n):
    if n not in (8, 16, 32, 64, 128, 256):
        sys.exit("size %d not a supported power of two" % n)
    return n.bit_length() - 1


def luma(rgb):
    return rgb[0] * 299 + rgb[1] * 587 + rgb[2] * 114


def pal565(raw):
    colors = sorted({expand565(to565(p)) for p in pixels(raw)}, key=luma)
    return ppm(len(colors), 1, colors)


KEY = 0xF81F


def opaque(raw, mask):
    px = [p for p, m in zip(pixels(raw), mask) if m >= 128]
    return ppm(len(px), 1, px)


def pack(w, h, raw, mask=None):
    px = pixels(raw)
    if len(px) != w * h:
        sys.exit("expected %d pixels, got %d" % (w * h, len(px)))
    keep = [m >= 128 for m in mask] if mask is not None else [True] * len(px)
    if len(keep) != len(px):
        sys.exit("mask has %d pixels, expected %d" % (len(keep), len(px)))
    code = [min(to565(p), KEY - 1) if to565(p) == KEY else to565(p) for p in px]
    solid = sorted({c for c, k in zip(code, keep) if k}, key=lambda c: luma(expand565(c)))
    codes = solid if mask is None else [KEY] + solid
    if len(codes) > 16:
        sys.exit("%d colors after 565 rounding, need <= 16" % len(codes))
    index = {c: i for i, c in enumerate(codes)}
    idx = [index[c] if k else 0 for c, k in zip(code, keep)]
    head = struct.pack("<BB16H", log2(w), log2(h), *(codes + [0] * (16 - len(codes))))
    body = bytes(idx[i] | (idx[i + 1] << 4) for i in range(0, len(idx), 2))
    return head + body


def read(path):
    with open(path, "rb") as f:
        return f.read()


def selftest():
    raw = bytes([255, 0, 255, 10, 20, 30, 200, 100, 50, 10, 20, 30])
    t = pack(8, 8, raw * 16, bytes([0, 255, 255, 255]) * 16)
    pal = struct.unpack_from("<16H", t, 2)
    assert pal[0] == KEY and KEY not in pal[1:] and t[34] & 15 == 0 and t[34] >> 4 != 0
    assert KEY not in struct.unpack_from("<16H", pack(8, 8, raw * 16), 2)


def unpack(data):
    w, h = 1 << data[0], 1 << data[1]
    pal = [expand565(c) for c in struct.unpack_from("<16H", data, 2)]
    body = data[34:34 + w * h // 2]
    if len(body) != w * h // 2:
        sys.exit("truncated tex")
    return ppm(w, h, [pal[n] for b in body for n in (b & 15, b >> 4)])


def main(argv):
    mode = argv[1] if len(argv) > 1 else ""
    if mode == "pal565":
        sys.stdout.buffer.write(pal565(sys.stdin.buffer.read()))
    elif mode == "pack":
        mask = read(argv[4]) if len(argv) > 4 else None
        sys.stdout.buffer.write(pack(int(argv[2]), int(argv[3]), sys.stdin.buffer.read(), mask))
    elif mode == "opaque":
        sys.stdout.buffer.write(opaque(sys.stdin.buffer.read(), read(argv[2])))
    elif mode == "unpack":
        sys.stdout.buffer.write(unpack(read(argv[2])))
    elif mode == "selftest":
        selftest()
    else:
        sys.exit("usage: texpack.py pal565 < rgb | pack W H [mask.gray] < rgb | opaque mask.gray < rgb | unpack file.tex > ppm | selftest")


if __name__ == "__main__":
    main(sys.argv)
