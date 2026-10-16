#!/bin/sh
set -eu
[ $# -eq 1 ] || { echo "usage: $0 <out.png>" >&2; exit 2; }
out=$1
zssh=${DOPPIO_ZSSH:-/home/guily/work/zedia/scratch/harness/zssh}
raw=$(mktemp)
trap 'rm -f "$raw"' EXIT
info=$("$zssh" 'p=$(cat /tmp/doppio/pid 2>/dev/null)
[ -n "$p" ] && kill -STOP $p 2>/dev/null
f=/sys/class/graphics/fb0
stride=$(cat $f/stride)
y=$(cut -d, -f2 $f/pan)
h=$(cut -d, -f2 $f/virtual_size)
[ $h -gt 1080 ] && h=1080
dd if=/dev/fb0 of=/tmp/doppio-shot.raw bs=$stride skip=$y count=$h 2>/dev/null
[ -n "$p" ] && kill -CONT $p 2>/dev/null
echo "$(cut -d, -f1 $f/virtual_size) $h $stride $y"')
set -- $info
"$zssh" 'cat /tmp/doppio-shot.raw && rm -f /tmp/doppio-shot.raw' > "$raw"
magick -size "$(($3 / 4))x$2" -depth 8 "BGRA:$raw" -crop "${1}x$2+0+0" +repage -alpha off "$out"
echo "$out ${1}x$2 pan_y=$4"
