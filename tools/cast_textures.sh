#!/bin/sh
set -eu

here=$(cd "$(dirname "$0")" && pwd)
src=${DOPPIO_SRC:-/tmp/doppio-src}/cast
work=$src/work
tex="$here/tex.sh"
mkdir -p "$src" "$work"

commons() {
    f="$src/$1"
    [ -s "$f" ] || curl -sfL -A 'doppio-assets/1.0' "https://commons.wikimedia.org/wiki/Special:FilePath/$2?width=1280" -o "$f"
    echo "$f"
}

crop() {
    python3 - "$1" "$2" "$3" "$4" "$(magick identify -format '%w %h' "$5")" <<'EOF'
import sys
x0, y0, x1, y1 = map(float, sys.argv[1:5])
w, h = map(int, sys.argv[5].split()[:2])
print(f"{int((x1 - x0) * w)}x{int((y1 - y0) * h)}+{int(x0 * w)}+{int(y0 * h)}")
EOF
}

noise() {
    magick -size "$1" xc: -seed "$2" +noise Random -channel G -separate +channel -virtual-pixel tile -blur "0x$3" -auto-level "$4"
}

strands() {
    noise 256x128 "$1" 0.6 "$work/s1.png"
    magick "$work/s1.png" -virtual-pixel tile -motion-blur 0x14+90 -auto-level -sigmoidal-contrast 4x50% +level-colors "$2","$3" "$4"
}

face() {
    magick "$1" -crop "$(crop "$2" "$3" "$4" "$5" "$1")" +repage -colorspace gray -normalize $8 +level-colors "$6","$7" -resize 124x112! "$work/face.png"
    magick -size 124x112 xc:black -fill white -draw "ellipse 62,58 50,50 0,360" -blur 0x8 "$work/fmask.png"
}

compose() {
    magick "$1" \( "$work/face.png" "$work/fmask.png" -alpha off -compose copy-opacity -composite \) -geometry +66+20 -compose over -composite "$2" -geometry +0+0 -composite "$3"
    magick "$3" -brightness-contrast -4x12 -fill '#211811' \
        -draw "ellipse 109,58 6,2 0,360 ellipse 147,58 6,2 0,360 polygon 101,50 118,51 118,54 103,53 polygon 138,51 155,50 153,53 138,54" \
        -fill '#644331' -draw "line 125,63 123,77 line 123,77 130,79" \
        -fill '#38231b' -draw "line 118,92 138,92" "$3"
}

skin() {
    noise 256x128 "$1" 2 "$work/sk.png"
    magick "$work/sk.png" -level 30%,70% +level-colors "$2","$3" "$4"
}

hairmask() {
    magick -size 256x128 xc:white -fill black -draw "$1" -blur 0x4 "$work/hmask.png"
    echo "$work/hmask.png"
}

t_benedita() {
    p=$(commons benedita.jpg "Portrait_of_an_elderly_woman_(I0027806).tiff")
    skin 3 '#4a2a1a' '#8a5a3c' "$work/b_skin.png"
    face "$p" 0.405 0.375 0.625 0.545 '#24120a' '#c89470' "-gamma 1.1"
    strands 5 '#4c4a48' '#d4d2ca' "$work/b_hair.png"
    m=$(hairmask "ellipse 128,104 58,72 0,360 rectangle 0,92 256,128")
    magick "$work/b_hair.png" "$m" -alpha off -compose copy-opacity -composite "$work/b_hairA.png"
    compose "$work/b_skin.png" "$work/b_hairA.png" "$work/benedita.png"
    "$tex" "$work/benedita.png" cast_benedita 64 noseam dither=15
}

t_tonho() {
    p=$(commons tonho.jpg "Unidentified_boy_photograph_-_DPLA_-_b33c835fd845cfa3b6f70b994a4edd3e.jpg")
    skin 4 '#4a2818' '#8c5c3c' "$work/t_skin.png"
    face "$p" 0.415 0.205 0.53 0.295 '#26130a' '#d09a74' "-gamma 1.05"
    strands 6 '#080706' '#3a2c22' "$work/t_hair.png"
    m=$(hairmask "ellipse 128,104 60,74 0,360 rectangle 0,96 256,128")
    magick "$work/t_hair.png" "$m" -alpha off -compose copy-opacity -composite "$work/t_hairA.png"
    compose "$work/t_skin.png" "$work/t_hairA.png" "$work/tonho.png"
    "$tex" "$work/tonho.png" cast_tonho 64 noseam dither=15
}

t_jairo() {
    p=$(commons jairo.jpg "Unidentified_man_with_mustache,_Portland,_Oregon,_ca_1890_(PORTRAITS_546).jpg")
    skin 7 '#6a3c22' '#b07850' "$work/j_skin.png"
    face "$p" 0.36 0.30 0.58 0.50 '#2e180c' '#d29a6c' "-gamma 1.15"
    strands 8 '#120c08' '#4a3426' "$work/j_hair.png"
    m=$(hairmask "ellipse 128,108 60,72 0,360 rectangle 0,98 256,128")
    magick "$work/j_hair.png" "$m" -alpha off -compose copy-opacity -composite "$work/j_hairA.png"
    compose "$work/j_skin.png" "$work/j_hairA.png" "$work/jairo.png"
    "$tex" "$work/jairo.png" cast_jairo 64 noseam dither=15
}

t_bia() {
    p=$(commons bia.jpg "A_young_woman_in_the_1920s_to_1930s_-_DPLA_-_58890cae980aeef52defcd865ba423c4.jpg")
    skin 9 '#9a6a50' '#e8b898' "$work/g_skin.png"
    face "$p" 0.315 0.30 0.575 0.55 '#4a281a' '#f6d0b0' "-gamma 1.1"
    strands 10 '#0e0806' '#5a3820' "$work/g_hair.png"
    m=$(hairmask "ellipse 128,104 52,72 0,360")
    magick "$work/g_hair.png" "$m" -alpha off -compose copy-opacity -composite "$work/g_hairA.png"
    compose "$work/g_skin.png" "$work/g_hairA.png" "$work/bia.png"
    "$tex" "$work/bia.png" cast_bia 64 noseam dither=15
}

t_corpo() {
    p=$(commons corpo.png "Ahmose-mummy-head.png")
    magick "$p" -crop "$(crop 0.30 0.08 0.70 0.24 "$p")" +repage -colorspace gray -normalize -resize 256x128! \
        +level-colors '#0e0a06','#6e6046' "$work/c_skin.png"
    face "$p" 0.32 0.26 0.70 0.58 '#060403' '#b4a27a' "-sigmoidal-contrast 4x45%"
    magick -size 256x128 xc:none "$work/c_none.png"
    compose "$work/c_skin.png" "$work/c_none.png" "$work/corpo.png"
    "$tex" "$work/corpo.png" cast_corpo 64 noseam dither=25 -- -modulate 92,80
}

t_matinta() {
    p=$(commons matinta.jpg "Unknown_(American)_-_Elderly_Woman_-_55.80_-_Detroit_Institute_of_Arts.jpg")
    skin 11 '#4a4c40' '#8a8c7a' "$work/m_skin.png"
    face "$p" 0.41 0.24 0.575 0.40 '#14160e' '#b4b49c' "-gamma 0.9"
    strands 12 '#1a1a18' '#c4c4bc' "$work/m_hair.png"
    m=$(hairmask "ellipse 128,108 46,76 0,360")
    magick "$work/m_hair.png" "$m" -alpha off -compose copy-opacity -composite "$work/m_hairA.png"
    compose "$work/m_skin.png" "$work/m_hairA.png" "$work/matinta.png"
    "$tex" "$work/matinta.png" cast_matinta 64 noseam dither=20
}

t_cabra() {
    p=$(commons cabra.jpg "Goat_Anatomy_Head.jpg")
    magick "$p" -crop "$(crop 0.30 0.55 0.75 0.80 "$p")" +repage -modulate 70,30 -resize 256x256! "$work/k_fur.png"
    magick "$work/k_fur.png" -fill '#3c3632' -draw "rectangle 0,232 256,256" -fill '#060404' -draw "ellipse 104,246 10,7 0,360 ellipse 152,246 10,7 0,360" "$work/cabra.png"
    "$tex" "$work/cabra.png" cast_cabra 64 noseam dither=40 -- -brightness-contrast -5x10
}

t_florido() {
    noise 128x128 21 3 "$work/fl_n.png"
    magick "$work/fl_n.png" -level 20%,80% +level-colors '#141a38','#26305c' "$work/fl_base.png"
    draw=$(python3 - <<'EOF'
import math, random
random.seed(7)
out = []
for i in range(11):
    cx, cy = random.uniform(0, 128), random.uniform(0, 128)
    col = random.choice(['#e07898', '#ece4d4', '#e0a8c0', '#f0d070'])
    for dx in (-128, 0, 128):
        for dy in (-128, 0, 128):
            x, y = cx + dx, cy + dy
            if -16 < x < 144 and -16 < y < 144:
                out.append(f"fill '#3c6a34' ellipse {x + 9:.1f},{y + 5:.1f} 6,3 0,360")
                out += [f"fill '{col}' circle {x + 5 * math.cos(a):.1f},{y + 5 * math.sin(a):.1f} {x + 5 * math.cos(a) + 3.6:.1f},{y + 5 * math.sin(a):.1f}" for a in (k * 2 * math.pi / 5 for k in range(5))]
                out.append(f"fill '#f2c84a' circle {x:.1f},{y:.1f} {x + 2.2:.1f},{y:.1f}")
print(' '.join(out))
EOF
)
    eval magick "$work/fl_base.png" -draw "\"$draw\"" "$work/florido.png"
    "$tex" "$work/florido.png" cast_florido 32 noseam dither=30
}

t_xale() {
    magick -size 16x16 xc:'#6a2430' -fill '#3e1018' -draw "circle 8,8 11,8" -fill '#8c3a44' -draw "line 0,0 16,16 line 16,0 0,16" "$work/xale_cell.png"
    magick -size 128x128 tile:"$work/xale_cell.png" "$work/xale_t.png"
    noise 128x128 23 2 "$work/xale_n.png"
    magick "$work/xale_t.png" "$work/xale_n.png" -compose multiply -composite -level 0%,85% "$work/xale.png"
    "$tex" "$work/xale.png" cast_xale 32 noseam dither=30
}

t_alva() {
    noise 128x128 25 1 "$work/alva_n.png"
    magick "$work/alva_n.png" -virtual-pixel tile -motion-blur 0x30+90 -auto-level +level-colors '#8e8a80','#f2eee4' "$work/alva.png"
    "$tex" "$work/alva.png" cast_alva 32 noseam dither=30
}

t_xadrez() {
    magick -size 128x128 xc:'#8c1e16' \
        -fill '#3a0a08' -draw "rectangle 0,40 128,64 rectangle 40,0 64,128" \
        -fill '#5a1410' -draw "rectangle 0,96 128,104 rectangle 96,0 104,128" \
        -fill '#d8b048' -draw "rectangle 0,80 128,82 rectangle 80,0 82,128" "$work/xad_a.png"
    noise 128x128 27 0.8 "$work/xad_n.png"
    magick "$work/xad_a.png" \( "$work/xad_n.png" -level 0%,160% \) -compose multiply -composite "$work/xadrez.png"
    "$tex" "$work/xadrez.png" cast_xadrez 32 noseam dither=30
}

t_jeans() {
    noise 128x128 29 0.7 "$work/je_n.png"
    magick "$work/je_n.png" -virtual-pixel tile -motion-blur 0x6+45 -auto-level +level-colors '#16223c','#5e7898' "$work/jeans.png"
    "$tex" "$work/jeans.png" cast_jeans 32 noseam dither=30
}

t_moletom() {
    noise 128x128 31 0.8 "$work/mo_n.png"
    magick "$work/mo_n.png" -virtual-pixel tile -motion-blur 0x3+90 -auto-level -level 15%,85% +level-colors '#8a6414','#dcae3a' "$work/moletom.png"
    "$tex" "$work/moletom.png" cast_moletom 32 noseam dither=30
}

t_trapo() {
    p=$(commons corpo.png "Ahmose-mummy-head.png")
    magick "$p" -crop "$(crop 0.2 0.62 0.8 0.92 "$p")" +repage -colorspace gray -normalize +level-colors '#2a2418','#b4a888' "$work/trapo.png"
    "$tex" "$work/trapo.png" cast_trapo 32 grime=0.5 streaks=0.3 dither=40
}

t_couro() {
    p=$(commons corpo.png "Ahmose-mummy-head.png")
    magick "$p" -crop "$(crop 0.36 0.22 0.62 0.34 "$p")" +repage -colorspace gray -normalize +level-colors '#140e08','#8c7a58' "$work/couro.png"
    "$tex" "$work/couro.png" cast_couro 32 dither=40
}

t_preto() {
    noise 128x128 33 0.6 "$work/pr_n.png"
    magick "$work/pr_n.png" -virtual-pixel tile -motion-blur 0x10+90 -auto-level +level-colors '#060508','#3c3644' "$work/preto.png"
    "$tex" "$work/preto.png" cast_preto 32 noseam grime=0.3 dither=30
}

t_pelo() {
    noise 128x128 35 0.5 "$work/pe_n.png"
    magick "$work/pe_n.png" -virtual-pixel tile -motion-blur 0x12+80 -auto-level -sigmoidal-contrast 5x55% +level-colors '#040404','#4a3e36' "$work/pelo.png"
    "$tex" "$work/pelo.png" cast_pelo 32 noseam dither=30
}

t_misc() {
    draw=$(python3 - <<'EOF'
cols = ['#141414', '#4a2c18', '#b02020', '#e8e4d8', '#6b4630', '#a87454', '#d8a888', '#8c8c7c',
        '#2c3c6c', '#3c4030', '#303034', '#c8b060', '#6c4a2a', '#d8ccb0', '#ff6010', '#282420']
print(' '.join(f"fill '{c}' rectangle {i % 4 * 32},{i // 4 * 32} {i % 4 * 32 + 31},{i // 4 * 32 + 31}" for i, c in enumerate(cols)))
EOF
)
    eval magick -size 128x128 xc:black -draw "\"$draw\"" "$work/misc.png"
    "$tex" "$work/misc.png" cast_misc 32 noseam dither=0
}

all="benedita tonho jairo bia corpo matinta cabra florido xale alva xadrez jeans moletom trapo couro preto pelo misc"
for t in ${*:-$all}; do
    "t_$t"
done
