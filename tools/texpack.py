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


def pack(w, h, raw):
    px = pixels(raw)
    if len(px) != w * h:
        sys.exit("expected %d pixels, got %d" % (w * h, len(px)))
    codes = sorted({to565(p) for p in px}, key=lambda c: luma(expand565(c)))
    if len(codes) > 16:
        sys.exit("%d colors after 565 rounding, need <= 16" % len(codes))
    index = {c: i for i, c in enumerate(codes)}
    idx = [index[to565(p)] for p in px]
    head = struct.pack("<BB16H", log2(w), log2(h), *(codes + [0] * (16 - len(codes))))
    body = bytes(idx[i] | (idx[i + 1] << 4) for i in range(0, len(idx), 2))
    return head + body


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
        sys.stdout.buffer.write(pack(int(argv[2]), int(argv[3]), sys.stdin.buffer.read()))
    elif mode == "unpack":
        with open(argv[2], "rb") as f:
            sys.stdout.buffer.write(unpack(f.read()))
    else:
        sys.exit("usage: texpack.py pal565 < rgb | pack W H < rgb | unpack file.tex > ppm")


if __name__ == "__main__":
    main(sys.argv)
