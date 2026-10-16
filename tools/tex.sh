#!/bin/sh
set -eu

usage() {
    echo "usage: tex.sh <src-image> <name> <size 32|64|128|WxH> [crop WxH+X+Y] [noseam] [alpha] [band=0.5] [seed=N] [colors=16] [dither=75] [grime=0..1] [streaks=0..1] [pre magick ops...] [-- post magick ops...]" >&2
    exit 2
}

[ $# -ge 3 ] || usage
src=$1
name=$2
size=$3
shift 3

here=$(cd "$(dirname "$0")" && pwd)
out="$here/../assets/tex"
preview=/tmp/doppio-tex-preview
work=$(mktemp -d /tmp/doppio-tex.XXXXXX)
trap 'rm -rf "$work"' EXIT
mkdir -p "$out" "$preview"

case $size in
    *x*) w=${size%x*} h=${size#*x} ;;
    *) w=$size h=$size ;;
esac
ww=$((w * 4))
wh=$((h * 4))

crop=""
seam=1
band=0.5
seed=1
colors=16
dither=75
grime=0
streaks=0
alpha=0
case ${1:-} in
    *x*+*+*) crop=$1; shift ;;
esac
while [ $# -gt 0 ]; do
    case $1 in
        noseam) seam=0 ;;
        alpha) alpha=1 seam=0 ;;
        band=*) band=${1#band=} ;;
        seed=*) seed=${1#seed=} ;;
        colors=*) colors=${1#colors=} ;;
        dither=*) dither=${1#dither=} ;;
        grime=*) grime=${1#grime=} ;;
        streaks=*) streaks=${1#streaks=} ;;
        *) break ;;
    esac
    shift
done

quote() {
    printf "'%s'" "$(printf %s "$1" | sed "s/'/'\\\\''/g")"
}

pre=""
post=""
inpost=0
for a in "$@"; do
    if [ "$a" = "--" ]; then
        inpost=1
    elif [ $inpost = 1 ]; then
        post="$post $(quote "$a")"
    else
        pre="$pre $(quote "$a")"
    fi
done

cropop=""
[ -z "$crop" ] || cropop="-crop $crop +repage"
alphaop="-alpha off"
[ $alpha = 0 ] || alphaop="-alpha set"
eval magick "$(quote "$src")" -colorspace sRGB $alphaop $cropop $pre -resize "${ww}x${wh}!" "$work/a.png"

noise() {
    magick -size "${ww}x${wh}" xc: -seed "$1" +noise Random -channel G -separate +channel \
        -virtual-pixel tile -blur "0x$2" -auto-level "$3"
}

if [ $seam = 1 ]; then
    noise "$seed" $((ww / 24 + 1)) "$work/n.png"
    blend() {
        magick "$1" \( "$1" -roll "$2" \) "$work/n.png" -fx "qd=$3; qt=clamp((qd-$band)/(1-$band)); qt=qt*qt*(3-2*qt); qk=clamp((qt+(u[2]-0.5)*0.6-0.5)*4+0.5); u*(1-qk)+v*qk" "$4"
    }
    blend "$work/a.png" "+$((ww / 2))+0" "abs(i/(w-1)*2-1)" "$work/h.png"
    blend "$work/h.png" "+0+$((wh / 2))" "abs(j/(h-1)*2-1)" "$work/s.png"
else
    cp "$work/a.png" "$work/s.png"
fi

if [ "$grime" != 0 ]; then
    noise $((seed + 11)) $((ww / 10 + 1)) "$work/g1.png"
    noise $((seed + 12)) $((ww / 40 + 1)) "$work/g2.png"
    magick "$work/s.png" "$work/g1.png" "$work/g2.png" \
        -channel RGB -fx "qg=u[1]*0.65+u[2]*0.35; u*(1-$grime*pow(clamp((0.62-qg)*2.2),1.5))" +channel "$work/s2.png"
    mv "$work/s2.png" "$work/s.png"
fi

if [ "$streaks" != 0 ]; then
    magick -size "${ww}x16" xc: -seed $((seed + 21)) +noise Random -channel G -separate +channel \
        -virtual-pixel tile -blur 0x1.2 -auto-level -filter point -resize "${ww}x${wh}!" "$work/c.png"
    noise $((seed + 22)) $((wh / 8 + 1)) "$work/c2.png"
    magick "$work/s.png" "$work/c.png" "$work/c2.png" \
        -channel RGB -fx "u*(1-$streaks*clamp((0.55-u[1])*3)*(0.5+0.5*u[2]))" +channel "$work/s2.png"
    mv "$work/s2.png" "$work/s.png"
fi

eval magick "'$work/s.png'" -filter Lanczos -resize "${w}x${h}!" $post -depth 8 "'$work/r.png'"
if [ $alpha = 1 ]; then
    magick "$work/r.png" -alpha extract -threshold 50% -depth 8 gray:"$work/m.gray"
    magick "$work/r.png" -alpha off -depth 8 rgb:"$work/c.rgb"
    python3 "$here/texpack.py" opaque "$work/m.gray" < "$work/c.rgb" \
        | magick ppm:- -dither None -colors "$((colors > 15 ? 15 : colors))" -unique-colors -depth 8 rgb:- \
        | python3 "$here/texpack.py" pal565 > "$work/pal.ppm"
else
    magick "$work/r.png" -dither None -colors "$colors" -unique-colors -depth 8 rgb:- \
        | python3 "$here/texpack.py" pal565 > "$work/pal.ppm"
fi
magick "$work/r.png" -alpha off -dither FloydSteinberg -define "dither:diffusion-amount=$dither%" -remap "$work/pal.ppm" -depth 8 rgb:- \
    | python3 "$here/texpack.py" pack "$w" "$h" $([ $alpha = 0 ] || echo "$work/m.gray") > "$work/t.tex"
mv "$work/t.tex" "$out/$name.tex"

python3 "$here/texpack.py" unpack "$out/$name.tex" | magick ppm:- -fill '#28303a' -opaque '#ff00ff' "$work/u.ppm"
magick "$work/u.ppm" -filter point -resize 400% "$preview/$name.png"
magick "$work/u.ppm" -write mpr:t +delete -size "$((w * 3))x$((h * 3))" tile:mpr:t -filter point -resize 200% "$preview/${name}_tiled.png"
echo "$name ${w}x${h} $(wc -c < "$out/$name.tex") bytes"
