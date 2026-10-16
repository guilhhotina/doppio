#!/bin/sh
set -eu

here=$(cd "$(dirname "$0")" && pwd)
src=${DOPPIO_SRC:-/tmp/doppio-src}
syn="$src/syn"
font=/usr/share/fonts/TTF/DejaVuSerif.ttf
fontb=/usr/share/fonts/TTF/DejaVuSerif-Bold.ttf
fontc=/usr/share/fonts/TTF/DejaVuSansCondensed.ttf
mkdir -p "$src/ph" "$syn"
tex="$here/tex.sh"

ph() {
    f="$src/ph/$1.jpg"
    if [ ! -s "$f" ]; then
        url=$(curl -sf "https://api.polyhaven.com/files/$1" | python3 -c "import json,sys; print(json.load(sys.stdin)['Diffuse']['2k']['jpg']['url'])")
        curl -sfL "$url" -o "$f"
    fi
    echo "$f"
}

commons() {
    f="$src/$1"
    [ -s "$f" ] || curl -sfL -A 'doppio-assets/1.0' "$2" -o "$f"
    echo "$f"
}

t_taco() {
    python3 "$here/synth.py" taco "$(ph black_walnut_veneer_02)" "$syn/taco.png"
    "$tex" "$syn/taco.png" taco 64 noseam grime=0.3 -modulate 100,25 -brightness-contrast 0x12 +level-colors '#22120a','#c8905e'
}

t_lajota() {
    "$tex" "$(ph terracotta_floor_tiles)" lajota 64 910x916+0+2 noseam grime=0.3 -modulate 118,115,99 -brightness-contrast 5x18
}

t_azulejo() {
    python3 "$here/synth.py" azulejo "$syn/azulejo.png"
    "$tex" "$syn/azulejo.png" azulejo 128 noseam grime=0.3 streaks=0.15 dither=40 -attenuate 0.4 +noise Gaussian -modulate 92,85
}

t_ceramica() {
    "$tex" "$(ph floor_tiles_08)" ceramica 64 1637x1641+19+3 noseam grime=0.35 -brightness-contrast 0x25
}

t_reboco() {
    "$tex" "$(ph painted_plaster_wall)" reboco 64 noseam grime=0.35 streaks=0.3 -modulate 100,40 -level 15%,95% -fill '#d8d0b0' -colorize 14% -brightness-contrast 0x15
}

t_reboco_mofo() {
    magick "$(ph painted_plaster_wall)" -resize 512x512 -modulate 100,40 -level 15%,95% -fill '#d8d0b0' -colorize 12% "$syn/mofo_a.png"
    magick "$(ph worn_mossy_plasterwall)" -resize 512x512 -modulate 90,80 -fill '#3c4228' -colorize 25% "$syn/mofo_b.png"
    magick -size 512x512 xc: -seed 5 +noise Random -channel G -separate +channel -virtual-pixel tile -blur 0x10 -auto-level "$syn/mofo_n.png"
    magick -size 512x512 xc: -seed 6 +noise Random -channel G -separate +channel -virtual-pixel tile -blur 0x4 -auto-level "$syn/mofo_m.png"
    magick "$syn/mofo_a.png" "$syn/mofo_b.png" "$syn/mofo_n.png" "$syn/mofo_m.png" \
        -fx "qh=j/h; qe=qh-0.5+(u[2]-0.5)*0.7; qm=clamp(qe*4)*(0.35+0.65*clamp((u[3]-0.3)*2.5)); qt=exp(-pow(qe*12,2)); (u*(1-qm)+v*qm*(1-0.3*qh))*(1-0.3*qt)" "$syn/mofo.png"
    "$tex" "$syn/mofo.png" reboco_mofo 64 noseam grime=0.3 streaks=0.35
}

t_papel_floral() {
    python3 "$here/synth.py" floral "$(ph decrepit_wallpaper)" "$syn/floral.png"
    "$tex" "$syn/floral.png" papel_floral 64 noseam grime=0.35 streaks=0.3 dither=50 -modulate 95,80
}

t_madeira_porta() {
    python3 "$here/synth.py" porta "$(ph wood_table_worn)" "$syn/porta.png"
    "$tex" "$syn/porta.png" madeira_porta 64x128 noseam grime=0.25 -modulate 112,85 -brightness-contrast 5x25
}

t_madeira_escura() {
    "$tex" "$(ph wood_table_worn)" madeira_escura 64 noseam grime=0.2 -modulate 95,85 -brightness-contrast 0x28
}

t_forro() {
    "$tex" "$(ph japanese_cedar_planks)" forro 64 noseam grime=0.3 streaks=0.2 -modulate 82,52,96 -brightness-contrast 0x30
}

t_tijolo() {
    "$tex" "$(ph red_brick)" tijolo 64 1024x1024+512+95 band=0.6 grime=0.2 -brightness-contrast 0x20
}

t_concreto() {
    "$tex" "$(ph concrete_floor_worn_001)" concreto 32 noseam grime=0.25 streaks=0.15 -level 8%,92% -fill '#807868' -colorize 15%
}

t_grama() {
    "$tex" "$(ph grass_ground)" grama 32 1024x1024+0+0 grime=0.3 -modulate 85,115 -brightness-contrast 0x25
}

t_terra() {
    "$tex" "$(ph red_dirt_mud_01)" terra 32 noseam grime=0.3 dither=50 -brightness-contrast 0x20
}

t_tecido_sofa() {
    "$tex" "$(ph floral_jacquard)" tecido_sofa 32 1024x777+512+0 band=0.6 grime=0.15 dither=40 -colorspace gray -equalize -blur 0x3 -colorspace sRGB +level-colors '#2e2612','#8e7a46'
}

t_colcha() {
    "$tex" "$(ph floral_jacquard)" colcha 64 2048x1553+0+0 noseam grime=0.25 dither=50 -colorspace gray -equalize -blur 0x3 -colorspace sRGB +level-colors '#3e1c1e','#c89c8c'
}

t_croche() {
    python3 "$here/synth.py" croche "$syn/croche.png"
    "$tex" "$syn/croche.png" croche 32 noseam dither=0
}

t_metal() {
    "$tex" "$(ph blue_metal_plate)" metal 32 1024x1024+0+0 grime=0.3 -modulate 140,15 -level 0%,80%
}

t_ceramica_barro() {
    "$tex" "$(ph clay_plaster)" ceramica_barro 32 noseam grime=0.15 -modulate 95,150,94 -brightness-contrast 0x20
}

t_vidro_noite() {
    python3 "$here/synth.py" vidro "$syn/vidro.png"
    "$tex" "$syn/vidro.png" vidro_noite 32 noseam grime=0.2 dither=50
}

t_quadro_santo() {
    f=$(commons santo.jpg https://upload.wikimedia.org/wikipedia/commons/4/4f/SacredHeartBatoni.jpg)
    magick \( -size 512x512 xc:'#4a3418' -fill none -stroke '#a07c38' -strokewidth 16 -draw 'rectangle 8,8 503,503' -stroke '#2a1c0c' -strokewidth 6 -draw 'rectangle 24,24 487,487' \) \
        \( "$f" -crop 1034x1034+0+40 +repage -resize 452x452! -modulate 108,50 -fill '#6b5428' -colorize 20% \
        \( -size 452x452 radial-gradient:white-gray40 \) -compose multiply -composite \) \
        -gravity center -compose over -composite "$syn/santo.png"
    "$tex" "$syn/santo.png" quadro_santo 64 noseam grime=0.4 streaks=0.3 dither=60 -brightness-contrast 5x18
}

t_relogio() {
    magick -size 512x512 xc:'#2a1a0e' -fill '#d8c99a' -stroke '#5a4020' -strokewidth 10 -draw 'circle 256,256 256,24' \
        -fill none -stroke '#1a120a' -strokewidth 3 -draw 'circle 256,256 256,58' \
        -font "$font" -pointsize 42 -fill '#1a120a' -stroke none -gravity center \
        $(python3 -c "
import math
for i, s in enumerate('XII I II III IIII V VI VII VIII IX X XI'.split()):
    a = math.radians(i * 30)
    print('-annotate %+d%+d %s' % (round(172 * math.sin(a)), round(-172 * math.cos(a)), s))
") -gravity northwest \
        -stroke '#100a05' -strokewidth 12 -draw 'line 256,256 352,256' -strokewidth 7 -draw 'line 256,256 256,104' \
        -fill '#100a05' -draw 'circle 256,256 256,270' \
        \( -size 512x512 radial-gradient:'#ffffff'-'#606060' \) -compose multiply -composite "$syn/relogio.png"
    "$tex" "$syn/relogio.png" relogio 32 noseam grime=0.35 dither=40
}

t_papel_jornal() {
    [ -s "$syn/porta.png" ] || python3 "$here/synth.py" porta "$(ph wood_table_worn)" "$syn/porta.png"
    magick -size 512x512 xc:'#cbbf98' -font "$fontb" -pointsize 54 -fill '#181410' -draw "text 34,70 'O ESTADO'" \
        -stroke '#181410' -strokewidth 3 -draw 'line 20,86 492,86' -draw 'line 20,94 492,94' -stroke none \
        -font "$fontb" -pointsize 30 -draw "text 24,136 'FAMILIA DESAPARECE'" -font "$fontc" -pointsize 13 \
        \( "$syn/porta.png" -crop 256x176+0+200 +repage -resize 220x150! -colorspace gray -level 0%,55% -sigmoidal-contrast 4x50% -ordered-dither h4x4a -colorspace sRGB -fill '#cbbf98' -tint 60 \) -geometry +270+150 -composite "$syn/jornal_a.png"
    python3 -c "
import random
r = random.Random(3)
words = 'a casa noite de que em o da do para com os sem porta janela velha familia vizinhos policia nada ninguem viu ouviram passos corredor madrugada luz'.split()
print('\n'.join(' '.join(r.choice(words) for _ in range(9)) for _ in range(60)))
" > "$syn/jornal.txt"
    magick "$syn/jornal_a.png" \
        \( -size 230x350 -background '#cbbf98' -fill '#2a241c' -font "$fontc" -pointsize 11 caption:@"$syn/jornal.txt" \) -geometry +24+152 -composite \
        \( -size 220x190 -background '#cbbf98' -fill '#2a241c' -font "$fontc" -pointsize 11 caption:@"$syn/jornal.txt" \) -geometry +270+312 -composite \
        "$syn/jornal.png"
    "$tex" "$syn/jornal.png" papel_jornal 32 noseam grime=0.4 streaks=0.2 dither=40
}

all="taco lajota azulejo ceramica reboco reboco_mofo papel_floral madeira_porta madeira_escura forro tijolo concreto grama terra tecido_sofa colcha croche metal ceramica_barro vidro_noite quadro_santo relogio papel_jornal"
for n in ${*:-$all}; do
    "t_$n"
done
