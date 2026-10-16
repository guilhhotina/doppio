#!/bin/sh
set -eu

here=$(cd "$(dirname "$0")" && pwd)
src=${DOPPIO_SRC:-/tmp/doppio-src}
syn="$src/syn"
font=/usr/share/fonts/TTF/DejaVuSerif.ttf
fontb=/usr/share/fonts/TTF/DejaVuSerif-Bold.ttf
fontc=/usr/share/fonts/TTF/DejaVuSansCondensed.ttf
fontcb=/usr/share/fonts/TTF/DejaVuSansCondensed-Bold.ttf
fontsb=/usr/share/fonts/TTF/DejaVuSerifCondensed-Bold.ttf
fonti=/usr/share/fonts/TTF/DejaVuSerifCondensed-Italic.ttf
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

t_assoalho() {
    "$tex" "$(ph wood_planks)" assoalho 64 1365x1365+0+0 grime=0.3 streaks=0.1 -modulate 96,85 -brightness-contrast 0x22
}

t_forro_pintado() {
    magick "$(ph white_planks_clean)" -resize 512x512! -modulate 96,40 -fill '#ddd2b4' -colorize 20% -brightness-contrast 0x20 \
        -stroke '#f2ead6' -strokewidth 3 -draw 'line 5,0 5,511' -draw 'line 90,0 90,511' -draw 'line 175,0 175,511' -draw 'line 261,0 261,511' -draw 'line 346,0 346,511' -draw 'line 431,0 431,511' \
        -stroke '#4a4234' -strokewidth 4 -draw 'line 1,0 1,511' -draw 'line 86,0 86,511' -draw 'line 171,0 171,511' -draw 'line 257,0 257,511' -draw 'line 342,0 342,511' -draw 'line 427,0 427,511' \
        "$syn/forro_pintado.png"
    "$tex" "$syn/forro_pintado.png" forro_pintado 64 noseam grime=0.35 streaks=0.25
}

t_asfalto() {
    "$tex" "$(ph asphalt_02)" asfalto 64 900x900+330+0 grime=0.3 -modulate 92,40 -brightness-contrast 0x22
}

t_paralelepipedo() {
    "$tex" "$(ph cobblestone_floor_08)" paralelepipedo 64 1024x1024+0+0 band=0.6 grime=0.3 -modulate 80,70 -brightness-contrast 0x22
}

t_meio_fio() {
    "$tex" "$(ph granite_tile_03)" meio_fio 64x32 640x320+20+20 noseam grime=0.2 streaks=0.3 -colorspace gray -colorspace sRGB -fill '#9a968c' -colorize 40% -brightness-contrast 10x10 \
        -- -fill '#c8c4b8' -draw 'rectangle 0,0 63,1' -fill '#3a3834' -draw 'rectangle 0,30 63,31'
}

t_fachada_rosa() {
    "$tex" "$(ph yellow_plaster)" fachada_rosa 64 1024x1024+1024+1024 grime=0.2 streaks=0.4 -colorspace gray -auto-level -colorspace sRGB +level-colors '#4a2a2c','#f0bcb0' -brightness-contrast 0x12
}

t_fachada_amarela() {
    "$tex" "$(ph yellow_plaster)" fachada_amarela 64 1024x1024+0+0 grime=0.2 streaks=0.4 -modulate 108,135,98 -brightness-contrast 4x18
}

t_fachada_azul() {
    "$tex" "$(ph blue_plaster_weathered)" fachada_azul 64 1024x1024+0+0 grime=0.2 streaks=0.4 -modulate 92,95,106 -brightness-contrast 0x15
}

t_muro_chapiscado() {
    "$tex" "$(ph rough_plaster_03)" muro_chapiscado 64 1365x1365+0+0 grime=0.4 streaks=0.3 -modulate 92,20 -brightness-contrast 0x30
}

t_telha() {
    "$tex" "$(ph roof_tiles)" telha 64 1024x1024+0+0 band=0.7 grime=0.3 -modulate 88,85 -brightness-contrast 0x18
}

t_madeira_crua() {
    "$tex" "$(ph raw_plank_wall)" madeira_crua 64 1095x1095+900+900 grime=0.3 -modulate 82,80 -brightness-contrast 0x15
}

t_ferrugem() {
    "$tex" "$(ph rusty_metal_02)" ferrugem 32 1024x1024+0+0 grime=0.3 -modulate 70,110 -brightness-contrast -5x25
}

t_pedra() {
    "$tex" "$(ph rustic_stone_wall)" pedra 64 1347x1347+0+0 band=0.6 grime=0.3 streaks=0.2 -modulate 85,60 -brightness-contrast 0x20
}

t_cal() {
    "$tex" "$(ph white_plaster_02)" cal 64 grime=0.35 streaks=0.3 -modulate 100,30 -fill '#e4dcc4' -colorize 12% -brightness-contrast 0x20
}

t_madeira_banco() {
    "$tex" "$(ph dark_wood)" madeira_banco 64 1024x1024+0+0 grime=0.15 -modulate 80,62 -brightness-contrast 0x15
}

t_casca() {
    "$tex" "$(ph bark_brown_02)" casca 64 band=0.6 grime=0.2 -modulate 80,90 -brightness-contrast 0x25
}

t_folhas_chao() {
    "$tex" "$(ph leaves_forest_ground)" folhas_chao 64 760x760+0+0 grime=0.35 -modulate 70,55 -brightness-contrast 0x25
}

t_barro() {
    "$tex" "$(ph brown_mud_03)" barro 64 1575x1575+0+0 grime=0.25 -modulate 78,120,98 -fill '#4a2614' -colorize 15% -brightness-contrast 0x25
}

t_madeira_ponte() {
    "$tex" "$(ph weathered_planks)" madeira_ponte 64 1024x1024+0+0 grime=0.3 -modulate 92,70 -brightness-contrast 0x22
}

t_sape() {
    "$tex" "$(ph thatch_roof_angled)" sape 64 1024x1024+0+0 grime=0.3 streaks=0.2 -colorspace gray -auto-level -colorspace sRGB +level-colors '#140e06','#c8a864' -brightness-contrast 0x30
}

t_rocha() {
    "$tex" "$(ph mossy_rock)" rocha 32 1365x1365+0+0 grime=0.3 -modulate 80,70 -brightness-contrast 0x25
}

t_lapide_granito() {
    "$tex" "$(ph granite_tile_03)" lapide_granito 64 360x360+40+40 grime=0.35 streaks=0.35 -colorspace gray -colorspace sRGB -fill '#5c5c62' -colorize 25% -brightness-contrast -5x30
}

t_terra_cova() {
    "$tex" "$(ph brown_mud_rocks_01)" terra_cova 64 1575x1575+0+0 grime=0.3 -modulate 70,90 -brightness-contrast 0x22
}

t_musgo() {
    "$tex" "$(ph moss_wood)" musgo 32 grime=0.35 -modulate 62,75,96 -brightness-contrast 0x20
}

t_bronze() {
    magick -size 256x256 xc: -seed 31 +noise Random -channel G -separate +channel -virtual-pixel tile -motion-blur 0x24+0 -auto-level \
        +level-colors '#3a240c','#d8a84c' \( +clone -fill '#4a7a5a' -colorize 100% \) \
        \( -size 256x256 xc: -seed 32 +noise Random -channel G -separate +channel -virtual-pixel tile -blur 0x14 -auto-level -threshold 76% -blur 0x4 -evaluate multiply 0.55 \) \
        -compose over -composite "$syn/bronze.png"
    "$tex" "$syn/bronze.png" bronze 32 noseam grime=0.3
}

t_papel_listrado() {
    python3 "$here/synth.py" listrado "$(ph decrepit_wallpaper)" "$syn/listrado.png"
    "$tex" "$syn/listrado.png" papel_listrado 64 noseam grime=0.35 streaks=0.3 dither=50 -modulate 92,80
}

t_calcada_portuguesa() {
    python3 "$here/synth.py" calcada "$syn/calcada.png"
    "$tex" "$syn/calcada.png" calcada_portuguesa 64 noseam grime=0.3 dither=40
}

t_marmore_xadrez() {
    python3 "$here/synth.py" xadrez "$syn/xadrez.png"
    "$tex" "$syn/xadrez.png" marmore_xadrez 64 noseam grime=0.25 dither=40
}

t_marmore_sujo() {
    python3 "$here/synth.py" marmore "$syn/marmore.png"
    "$tex" "$syn/marmore.png" marmore_sujo 64 noseam grime=0.45 streaks=0.45 dither=50 -fill '#bcae88' -colorize 14%
}

t_ladrilho_hidraulico() {
    python3 "$here/synth.py" ladrilho "$syn/ladrilho.png"
    "$tex" "$syn/ladrilho.png" ladrilho_hidraulico 64 noseam grime=0.3 dither=40
}

t_azulejo_capela() {
    python3 "$here/synth.py" capela "$syn/capela.png"
    "$tex" "$syn/capela.png" azulejo_capela 64 noseam grime=0.3 streaks=0.2 dither=40
}

t_vitral() {
    python3 "$here/synth.py" vitral "$syn/vitral.png"
    "$tex" "$syn/vitral.png" vitral 64x128 noseam grime=0.2 dither=30
}

t_dourado() {
    python3 "$here/synth.py" dourado "$syn/dourado.png"
    "$tex" "$syn/dourado.png" dourado 64 noseam grime=0.3 dither=40
}

t_agua_rio() {
    python3 "$here/synth.py" agua 7 20,26,22 96,104,98 "$syn/agua_rio.png"
    "$tex" "$syn/agua_rio.png" agua_rio 64 noseam dither=50
}

t_agua() {
    python3 "$here/synth.py" agua 9 12,22,34 84,104,126 "$syn/agua.png"
    "$tex" "$syn/agua.png" agua 64 noseam dither=50
}

t_pau_a_pique() {
    python3 "$here/synth.py" pau_a_pique "$(ph patterned_clay_wall)" "$(ph raw_plank_wall)" "$syn/pau_a_pique.png"
    "$tex" "$syn/pau_a_pique.png" pau_a_pique 64 noseam grime=0.3 streaks=0.2 -modulate 90,80
}

t_ceu_noite() {
    python3 "$here/synth.py" ceu noite "$syn/ceu_noite.png"
    "$tex" "$syn/ceu_noite.png" ceu_noite 128x64 noseam dither=0
}

t_ceu_nublado() {
    python3 "$here/synth.py" ceu nublado "$syn/ceu_nublado.png"
    "$tex" "$syn/ceu_nublado.png" ceu_nublado 128x64 noseam dither=40
}

t_lua_a() {
    python3 "$here/synth.py" lua "$syn/lua.png"
    "$tex" "$syn/lua.png" lua_a 32 alpha dither=30
}

t_folhas_a() {
    python3 "$here/synth.py" folhas "$syn/folhas.png"
    "$tex" "$syn/folhas.png" folhas_a 64 alpha dither=40
}

t_bananeira_a() {
    python3 "$here/synth.py" bananeira "$syn/bananeira.png"
    "$tex" "$syn/bananeira.png" bananeira_a 64x128 alpha dither=40
}

t_bambu_a() {
    python3 "$here/synth.py" bambu "$syn/bambu.png"
    "$tex" "$syn/bambu.png" bambu_a 64x128 alpha dither=40
}

t_grade_a() {
    python3 "$here/synth.py" grade "$syn/grade.png"
    "$tex" "$syn/grade.png" grade_a 64 alpha dither=20
}

t_portao_a() {
    python3 "$here/synth.py" portao "$syn/portao.png"
    "$tex" "$syn/portao.png" portao_a 64x128 alpha dither=20
}

t_cruz_ferro_a() {
    python3 "$here/synth.py" cruz_ferro "$syn/cruz_ferro.png"
    "$tex" "$syn/cruz_ferro.png" cruz_ferro_a 64 alpha dither=20
}

grain() {
    printf '%s' "( $(ph blue_metal_plate) -resize $1! -colorspace gray -auto-level +level $2,100% -colorspace sRGB )"
}

sign() {
    name=$1 bg=$2 fg=$3 face=$4 text=$5 box=$6
    shift 6
    geo=+${box#*+}
    magick -size 512x128 xc:"$bg" $(grain 512x128 60%) -compose multiply -composite \
        "$@" -compose over \
        \( -size "${box%%+*}" -background none -fill "$fg" -font "$face" -gravity center label:"$text" -write mpr:lab +delete \) \
        \( mpr:lab -channel RGB -evaluate set 0 +channel \) -gravity center -geometry "$geo" -composite \
        mpr:lab -geometry "$(echo "$geo" | awk -F+ '{printf "+%d+%d", $2 - 3, $3 - 4}')" -composite \
        -gravity northwest -fill none -stroke "$fg" -strokewidth 5 -draw 'rectangle 10,10 501,117' \
        "$syn/$name.png"
    "$tex" "$syn/$name.png" "$name" 128x32 noseam grime=0.35 streaks=0.35 dither=30
}

t_letreiro_padaria() {
    sign letreiro_padaria '#e6d8b0' '#a82418' "$fontsb" 'PADARIA' 430x96+3+4
}

t_letreiro_bar() {
    sign letreiro_bar '#1e4a2c' '#e8c040' "$fontcb" 'BAR DO ZÉ' 430x96+3+4
}

t_letreiro_farmacia() {
    sign letreiro_farmacia '#e8e8dc' '#1a7a3a' "$fontcb" 'FARMÁCIA' 370x90+50+4 \
        -fill '#1a7a3a' -draw 'rectangle 22,48 86,80' -draw 'rectangle 38,32 70,96'
}

t_letreiro_rodoviaria() {
    sign letreiro_rodoviaria '#1c3c78' '#ecece4' "$fontcb" 'RODOVIÁRIA' 430x96+3+4
}

t_placa_rua() {
    magick -size 512x256 xc:'#d8d8d0' -fill '#e8e8e0' -draw 'roundrectangle 0,0 511,255 30,30' -fill '#1e3e8a' -draw 'roundrectangle 16,16 495,239 22,22' \
        -fill '#e8e8e0' -font "$fontcb" -gravity north -pointsize 52 -annotate +0+28 'RUA DO' -pointsize 104 -annotate +0+92 'ROSÁRIO' \
        \( -size 512x256 xc:'#3a2a20' \( -size 512x256 xc: -seed 211 +noise Random -channel G -separate +channel -blur 0x6 -auto-level -threshold 88% \) \
        -alpha off -compose copy-opacity -composite \) -gravity northwest -compose over -composite \
        "$syn/placa_rua.png"
    "$tex" "$syn/placa_rua.png" placa_rua 64x32 noseam grime=0.3 streaks=0.3 dither=20
}

t_cartaz_festa() {
    f=$(commons santo.jpg https://upload.wikimedia.org/wikipedia/commons/4/4f/SacredHeartBatoni.jpg)
    magick -size 512x512 xc:'#dcc46c' \
        \( "$f" -crop 900x760+70+80 +repage -resize 300x250! -colorspace gray -level 8%,85% -ordered-dither h4x4a -colorspace sRGB \) -gravity north -geometry +0+152 -compose multiply -composite -compose over \
        -fill '#a01818' -font "$fontcb" -pointsize 84 -annotate +0+18 'PROCISSÃO' \
        -fill '#181410' -font "$fontsb" -pointsize 40 -annotate +0+104 'DO SENHOR MORTO' \
        -fill '#a01818' -font "$fontcb" -pointsize 38 -annotate +0+410 'SEXTA-FEIRA SANTA 19H' \
        -fill '#181410' -font "$fontc" -pointsize 28 -annotate +0+460 'IGREJA MATRIZ DE PEDRA SECA' \
        -gravity northwest -fill '#7c766a' -draw 'polygon 512,512 512,446 488,456 470,486 446,500 430,512' -draw 'polygon 0,0 70,0 52,22 30,26 0,48' \
        "$syn/cartaz.png"
    "$tex" "$syn/cartaz.png" cartaz_festa 64 noseam grime=0.35 streaks=0.3 dither=30
}

t_quadro_negro() {
    magick -size 512x256 xc:'#5a3a1e' -fill '#1c2622' -draw 'rectangle 18,18 493,229' \
        \( -size 476x212 xc: -seed 221 +noise Random -channel G -separate +channel -blur 0x16 -auto-level -level 55%,100% -evaluate multiply 0.3 \) -geometry +18+18 -compose screen -composite -compose over \
        -fill '#d4d8cc' -font "$fonti" -pointsize 46 -annotate +44+74 'Catecismo' -pointsize 28 -annotate +52+122 '1. Pai Nosso' -annotate +52+160 '2. Ave Maria' -annotate +52+198 '3. Credo' \
        -stroke '#d4d8cc' -strokewidth 5 -draw 'line 400,60 400,170' -draw 'line 366,92 434,92' -stroke none \
        -fill '#6a4a2a' -draw 'rectangle 18,228 493,244' -fill '#e8e8e0' -draw 'rectangle 330,222 366,230' \
        "$syn/quadro_negro.png"
    "$tex" "$syn/quadro_negro.png" quadro_negro 64x32 noseam grime=0.25 dither=30
}

t_calendario() {
    f=$(commons santo.jpg https://upload.wikimedia.org/wikipedia/commons/4/4f/SacredHeartBatoni.jpg)
    python3 -c "
print('fill \'#3a3430\'')
for r in range(5):
    for c in range(7):
        print(\"fill '%s' rectangle %d,%d %d,%d\" % ('#a82020' if c == 0 else '#3a3430', 26 + c * 30, 166 + r * 17, 44 + c * 30, 175 + r * 17))
" > "$syn/cal.mvg"
    magick -size 256x256 xc:'#e8e0cc' \( "$f" -crop 1000x500+40+120 +repage -resize 216x108! -modulate 110,70 \) -geometry +20+16 -composite \
        -fill '#a82020' -draw 'rectangle 20,128 236,156' -fill '#f0ece0' -font "$fontcb" -pointsize 24 -gravity north -annotate +0+129 'ABRIL 1977' \
        -gravity northwest -draw "@$syn/cal.mvg" -fill '#201810' -draw 'circle 128,7 128,11' \
        "$syn/calendario.png"
    "$tex" "$syn/calendario.png" calendario 32 noseam grime=0.3 dither=30
}

t_quadro_fusivel() {
    magick -size 256x256 xc:'#5a5c58' -fill '#8a8c86' -draw 'rectangle 12,12 243,243' \
        -fill '#e8c030' -draw 'polygon 128,16 156,62 100,62' -fill '#181818' -draw 'polygon 132,26 118,46 130,46 122,58 140,38 128,38' \
        -fill '#e0dcd0' -draw 'circle 64,110 64,140' -draw 'circle 128,110 128,140' -draw 'circle 192,110 192,140' -draw 'circle 64,190 64,220' -draw 'circle 128,190 128,220' -draw 'circle 192,190 192,220' \
        -fill '#b89850' -draw 'circle 64,110 64,128' -draw 'circle 128,110 128,128' -draw 'circle 192,110 192,128' -draw 'circle 64,190 64,208' -draw 'circle 192,190 192,208' \
        -fill '#0a0a0a' -draw 'circle 128,190 128,208' -fill '#d84030' -draw 'circle 64,110 64,116' -draw 'circle 128,110 128,116' -draw 'circle 192,110 192,116' -draw 'circle 64,190 64,196' -draw 'circle 192,190 192,196' \
        "$syn/quadro_fusivel.png"
    "$tex" "$syn/quadro_fusivel.png" quadro_fusivel 32 noseam grime=0.35 dither=30
}

t_vitrine() {
    python3 "$here/synth.py" vidro "$syn/vidro.png"
    magick "$syn/vidro.png" -modulate 100,70 -fill '#3a3020' -colorize 10% \
        -fill '#4a3a2888' -draw 'rectangle 20,150 236,158' -draw 'rectangle 20,212 236,220' \
        -fill '#6a4a2c99' -draw 'ellipse 60,140 26,11 0,360' -draw 'ellipse 120,141 24,10 0,360' -draw 'ellipse 184,140 28,11 0,360' -draw 'ellipse 80,202 30,11 0,360' -draw 'ellipse 168,203 26,10 0,360' \
        \( -size 256x256 gradient:'#00000000'-'#a09078a0' \) -compose over -composite \
        -fill none -stroke '#3a2a1a' -strokewidth 18 -draw 'rectangle 0,0 255,255' -stroke '#1a120a' -strokewidth 3 -draw 'rectangle 10,10 245,245' \
        "$syn/vitrine.png"
    "$tex" "$syn/vitrine.png" vitrine 64 noseam grime=0.35 streaks=0.3 dither=40
}

t_onibus_lataria() {
    python3 -c "
for y in (98, 142):
    for x in range(8, 256, 32):
        print('circle %d,%d %d,%d' % (x, y, x, y + 3))
" > "$syn/rivets.mvg"
    magick -size 256x256 xc:'#d8ccaa' -fill '#a02a20' -draw 'rectangle 0,108 255,132' -fill '#26467e' -draw 'rectangle 0,132 255,255' \
        -fill '#2a2a2a' -draw 'rectangle 0,104 255,107' -fill '#c8c8c0' -draw "@$syn/rivets.mvg" \
        $(grain 256x256 68%) -compose multiply -composite "$syn/onibus_lataria.png"
    "$tex" "$syn/onibus_lataria.png" onibus_lataria 64 noseam grime=0.35 streaks=0.45 dither=30
}

t_onibus_janela() {
    magick -size 256x256 xc:'#d8ccaa' -fill '#141414' -draw 'roundrectangle 14,16 241,214 20,20' -fill '#1c2838' -draw 'roundrectangle 24,26 231,204 14,14' \
        -fill '#3a4a62' -draw 'polygon 40,204 90,26 120,26 70,204' -fill '#2c3a50' -draw 'polygon 150,204 190,26 202,26 162,204' \
        -fill '#b8b8b0' -draw 'rectangle 122,26 133,204' -fill '#a02a20' -draw 'rectangle 0,232 255,255' \
        $(grain 256x256 72%) -compose multiply -composite "$syn/onibus_janela.png"
    "$tex" "$syn/onibus_janela.png" onibus_janela 64 noseam grime=0.3 streaks=0.3 dither=30
}

t_fusca_lataria() {
    magick -size 256x256 gradient:'#9cbccc'-'#56788c' $(grain 256x256 70%) -compose multiply -composite \
        \( -size 256x256 xc:'#6a3a1e' \) \( -size 256x256 xc: -seed 231 +noise Random -channel G -separate +channel -virtual-pixel tile -blur 0x5 -auto-level -threshold 90% -blur 0x1 \) \
        -compose over -composite "$syn/fusca.png"
    "$tex" "$syn/fusca.png" fusca_lataria 32 noseam grime=0.35 streaks=0.25 dither=30
}

t_kombi_lataria() {
    magick -size 256x256 xc:'#dcd8cc' -fill '#a42c22' -draw 'rectangle 0,128 255,255' -fill '#c8c8c4' -draw 'rectangle 0,120 255,127' -fill '#3a3a3a' -draw 'rectangle 0,127 255,129' \
        -fill '#7a2018' -draw 'rectangle 0,200 255,203' $(grain 256x256 72%) -compose multiply -composite "$syn/kombi.png"
    "$tex" "$syn/kombi.png" kombi_lataria 64 noseam grime=0.35 streaks=0.4 dither=30
}

t_bilhete() {
    magick -size 256x256 xc:'#d8cc9c' $(grain 256x256 78%) -compose multiply -composite -compose over \
        -fill '#1a120c' -font "${fonti%Italic.ttf}BoldItalic.ttf" -pointsize 34 -draw "text 18,58 'Não abra'" -draw "text 18,100 'a porta do'" -draw "text 18,142 'porão de'" -draw "text 18,184 'noite...'" \
        -stroke '#2a2018' -strokewidth 3 -draw 'line 22,212 150,206' -draw 'line 22,234 196,230' -stroke '#b8a878' -strokewidth 4 -draw 'line 0,128 255,124' \
        -stroke none -fill '#a02020' -draw 'circle 128,14 128,6' "$syn/bilhete.png"
    "$tex" "$syn/bilhete.png" bilhete 32 noseam grime=0.35 streaks=0.2 dither=20
}

item() {
    n=$1
    shift
    magick -size 256x256 xc:none "$@" -virtual-pixel edge \
        \( +clone -alpha extract -morphology Dilate Disk:7 \( +clone -fill '#120c08' -colorize 100% \) +swap -alpha off -compose copy-opacity -composite \) \
        +swap -compose over -composite "$syn/item_$n.png"
    "$tex" "$syn/item_$n.png" "item_${n}_a" 32 alpha dither=0
}

t_item_chave_a() {
    item chave -fill none -stroke '#b8902c' -strokewidth 22 -draw 'circle 62,128 62,94' -stroke none -fill '#b8902c' \
        -draw 'rectangle 94,118 236,138' -draw 'rectangle 196,138 206,184' -draw 'rectangle 214,138 228,172' \
        -fill '#ecc860' -draw 'rectangle 100,120 232,124' -fill none -stroke '#ecc860' -strokewidth 5 -draw 'arc 30,96 94,160 190,280' \
        -virtual-pixel transparent -distort SRT -40
}

t_item_alicate_a() {
    item alicate -fill none -stroke '#b02a22' -strokewidth 24 -draw 'stroke-linecap round line 132,120 238,86' -draw 'stroke-linecap round line 132,136 238,170' \
        -stroke '#e05a48' -strokewidth 6 -draw 'line 140,114 232,84' -stroke none \
        -fill '#8a8c94' -draw 'polygon 14,122 120,106 136,128 120,150 14,134' -fill '#c8c8d0' -draw 'polygon 18,123 116,110 118,116 18,127' \
        -fill '#4a4c54' -draw 'circle 126,128 126,112' -virtual-pixel transparent -distort SRT -35
}

t_item_martelo_a() {
    item martelo -fill '#9a6a38' -draw 'roundrectangle 114,70 142,246 10,10' -fill '#c8945a' -draw 'rectangle 118,76 124,238' \
        -fill '#6a6c74' -draw 'polygon 50,28 162,28 162,84 50,84' -draw 'rectangle 34,36 50,76' -draw 'polygon 160,32 208,22 238,62 202,48 160,80' \
        -fill '#b0b2ba' -draw 'rectangle 54,32 158,40' -virtual-pixel transparent -distort SRT 25
}

t_item_pe_de_cabra_a() {
    item pe_de_cabra -fill none -stroke '#a82a20' -strokewidth 22 -draw "stroke-linecap round stroke-linejoin round path 'M 204,28 L 86,196 Q 64,236 104,234'" \
        -stroke '#e05a48' -strokewidth 5 -draw 'line 198,32 92,184' -stroke '#a82a20' -strokewidth 9 -draw 'line 204,28 230,16' -draw 'line 204,28 220,6'
}

t_item_chave_de_fenda_a() {
    item chave_de_fenda -fill '#d8a020' -draw 'roundrectangle 16,100 120,156 24,24' -fill '#a87810' -draw 'rectangle 40,104 46,152' -draw 'rectangle 64,104 70,152' -draw 'rectangle 88,104 94,152' \
        -fill '#f0d070' -draw 'roundrectangle 24,104 112,113 4,4' -fill '#9a9aa2' -draw 'rectangle 118,114 134,142' -draw 'rectangle 134,120 228,136' -draw 'polygon 228,120 248,124 248,132 228,136' \
        -fill '#d8d8e0' -draw 'rectangle 136,121 226,125' -virtual-pixel transparent -distort SRT -40
}

t_item_fusivel_a() {
    item fusivel -fill '#e8e4d8' -draw 'roundrectangle 66,112 190,236 16,16' -fill '#b4b0a4' -draw 'roundrectangle 152,112 190,236 12,12' \
        -fill '#b89850' -draw 'rectangle 80,44 176,114' -fill '#7a6230' -draw 'rectangle 80,58 176,65' -draw 'rectangle 80,76 176,83' -draw 'rectangle 80,94 176,101' \
        -fill '#dcc070' -draw 'rectangle 108,22 148,44' -fill '#d03020' -draw 'circle 128,172 128,154'
}

t_item_sal_a() {
    item sal -fill '#e2dac4' -draw 'polygon 62,92 194,92 216,238 40,238' -draw 'polygon 84,94 172,94 150,48 106,48' \
        -fill '#b8b09a' -draw 'polygon 170,92 194,92 216,238 186,238' -fill '#8a6a3a' -draw 'rectangle 96,58 160,72' \
        -fill '#2a4a9a' -draw 'rectangle 62,134 194,194' -fill '#f0f0e8' -font "$fontcb" -pointsize 50 -draw "text 86,184 'SAL'"
}

t_item_vela_a() {
    item vela -fill '#ece4cc' -draw 'rectangle 100,96 156,244' -fill '#c0b89e' -draw 'rectangle 140,96 156,244' \
        -fill '#ece4cc' -draw 'roundrectangle 92,94 110,152 8,8' -draw 'roundrectangle 146,94 162,130 7,7' -fill '#d8d0b6' -draw 'ellipse 128,96 28,8 0,360' \
        -stroke '#201810' -strokewidth 5 -draw 'line 128,96 128,76' -stroke none -fill '#ff9a30' -draw 'ellipse 128,50 20,32 0,360' -fill '#ffe080' -draw 'ellipse 128,56 10,18 0,360'
}

t_item_fosforo_a() {
    item fosforo -fill '#c8a040' -draw 'rectangle 26,104 214,198' -fill '#2a4a8a' -draw 'rectangle 26,104 214,132' -fill '#b02a20' -draw 'rectangle 26,172 214,198' \
        -fill '#5a3a22' -draw 'rectangle 26,198 214,214' -fill '#f0e8c0' -draw 'circle 120,152 120,138' \
        -stroke '#d8c08a' -strokewidth 9 -draw 'line 150,98 222,44' -stroke none -fill '#b02020' -draw 'circle 226,40 226,26'
}

t_item_tabaco_a() {
    item tabaco -fill '#3a200e' -draw 'circle 128,128 128,22' -fill none -stroke '#7a4a22' -strokewidth 13 \
        -draw 'circle 128,128 128,38' -draw 'circle 128,128 128,60' -draw 'circle 128,128 128,82' -draw 'circle 128,128 128,104' \
        -stroke '#a8703a' -strokewidth 4 -draw 'arc 32,32 224,224 200,290' -draw 'arc 76,76 180,180 200,290' -stroke none -fill '#5a3418' -draw 'circle 128,128 128,116'
}

t_item_balde_a() {
    item balde -fill none -stroke '#5a6066' -strokewidth 8 -draw 'arc 48,18 208,178 190,350' -stroke none \
        -fill '#8a9096' -draw 'polygon 52,98 204,98 184,236 72,236' -fill '#6a7076' -draw 'polygon 160,98 204,98 184,236 156,236' \
        -fill '#bcc2c8' -draw 'polygon 64,102 84,102 92,232 80,232' -fill '#5a6066' -draw 'rectangle 58,142 198,148' -draw 'rectangle 66,192 190,198' \
        -fill '#3a3e42' -draw 'ellipse 128,98 76,16 0,360' -fill '#3a6a88' -draw 'ellipse 128,100 64,10 0,360'
}

t_item_freio_a() {
    item freio -fill none -stroke '#5a3418' -strokewidth 14 -draw 'line 54,118 72,26' -draw 'line 202,118 184,26' -draw 'line 72,26 184,26' \
        -stroke '#6a6a72' -strokewidth 15 -draw 'circle 52,152 52,120' -draw 'circle 204,152 204,120' -draw 'line 84,152 124,170' -draw 'line 132,170 172,152' \
        -stroke '#b0a060' -strokewidth 6 -draw 'rectangle 118,14 140,38' -stroke none -fill '#6a6a72' -draw 'circle 128,170 128,158'
}

t_item_ossos_a() {
    item ossos -stroke '#c8bc98' -strokewidth 26 -draw 'line 194,62 62,194' -stroke none -fill '#c8bc98' \
        -draw 'circle 210,62 210,44' -draw 'circle 194,46 194,28' -draw 'circle 62,210 62,192' -draw 'circle 46,194 46,176' \
        -stroke '#e6dcbc' -strokewidth 26 -draw 'line 62,62 194,194' -stroke none -fill '#e6dcbc' \
        -draw 'circle 46,62 46,44' -draw 'circle 62,46 62,28' -draw 'circle 194,210 194,192' -draw 'circle 210,194 210,176'
}

t_item_cruz_a() {
    item cruz -fill '#6a3e1a' -draw 'rectangle 108,14 148,246' -draw 'rectangle 52,60 204,100' -fill '#9a6a3a' -draw 'rectangle 112,18 119,242' -draw 'rectangle 56,64 200,70' \
        -fill '#e8e0c0' -draw 'rectangle 112,24 144,42' -fill '#dcc48c' -draw 'circle 128,80 128,67' -draw 'polygon 120,92 136,92 140,172 116,172' \
        -stroke '#dcc48c' -strokewidth 9 -draw 'line 122,94 70,84' -draw 'line 134,94 186,84' -draw 'line 124,170 126,212' -draw 'line 132,170 130,212'
}

t_item_bilhete_onibus_a() {
    item bilhete_onibus -fill '#e6d6a6' -draw 'rectangle 22,66 234,190' -fill '#b02a20' -draw 'rectangle 22,66 234,100' \
        -fill '#f0e8d0' -font "$fontcb" -pointsize 26 -draw "text 34,93 'PEDRA SECA'" \
        -fill '#3a3028' -draw 'rectangle 38,114 180,123' -draw 'rectangle 38,136 204,145' -draw 'rectangle 38,158 150,167' -fill '#201810' -draw 'circle 206,164 206,152' \
        -virtual-pixel transparent -distort SRT -12
}

t_item_manivela_a() {
    item manivela -fill none -stroke '#6a6c74' -strokewidth 18 -draw "stroke-linecap round stroke-linejoin round path 'M 40,212 L 40,70 L 190,70 L 190,138'" \
        -stroke '#b8bac2' -strokewidth 4 -draw 'line 34,74 34,206' -draw 'line 44,64 186,64' -stroke none \
        -fill '#4a4c54' -draw 'rectangle 24,198 56,244' -fill '#8a5a2a' -draw 'roundrectangle 172,128 208,240 14,14' -fill '#b07a44' -draw 'rectangle 179,136 186,230'
}

t_item_remo_a() {
    item remo -fill '#a87a40' -draw 'rectangle 120,14 136,160' -draw 'rectangle 98,6 158,22' -draw 'ellipse 128,192 38,60 0,360' \
        -fill '#8a5e2e' -draw 'ellipse 142,192 22,54 0,360' -fill '#d4a464' -draw 'rectangle 122,18 126,158' -draw 'ellipse 114,190 10,44 0,360' \
        -virtual-pixel transparent -distort SRT 35
}

t_item_lanterna_pilha_a() {
    item lanterna_pilha -fill '#24242a' -draw 'roundrectangle 76,50 180,238 10,10' -fill '#d8a030' -draw 'rectangle 76,50 180,128' \
        -fill '#f4cc64' -draw 'rectangle 84,54 98,124' -fill '#4a4a54' -draw 'rectangle 84,132 98,232' -fill '#b8b8c0' -draw 'rectangle 110,30 146,50' \
        -fill '#24242a' -draw 'rectangle 120,70 136,112' -draw 'rectangle 108,83 148,99'
}

t_item_foto_a() {
    item foto -fill '#ece4d0' -draw 'rectangle 30,40 226,216' -fill '#7a6040' -draw 'rectangle 44,54 212,180' -fill '#9a8058' -draw 'rectangle 44,54 212,104' \
        -fill '#2a2018' -draw 'circle 98,104 98,88' -draw 'roundrectangle 78,118 118,180 12,12' -draw 'circle 158,98 158,80' -draw 'roundrectangle 136,114 180,180 14,14' \
        -stroke '#ece4d0' -strokewidth 5 -draw 'line 146,84 170,110' -draw 'line 170,84 146,110' -stroke none -fill '#5a5040' -draw 'rectangle 56,194 170,201' \
        -virtual-pixel transparent -distort SRT -10
}

v1="taco lajota azulejo ceramica reboco reboco_mofo papel_floral madeira_porta madeira_escura forro tijolo concreto grama terra tecido_sofa colcha croche metal ceramica_barro vidro_noite quadro_santo relogio papel_jornal"
casa="assoalho papel_listrado forro_pintado calendario quadro_fusivel bilhete"
cidade="asfalto paralelepipedo calcada_portuguesa meio_fio fachada_rosa fachada_amarela fachada_azul muro_chapiscado grade_a portao_a placa_rua letreiro_padaria letreiro_bar letreiro_farmacia letreiro_rodoviaria cartaz_festa telha vitrine onibus_lataria onibus_janela fusca_lataria kombi_lataria madeira_crua ferrugem"
igreja="pedra cal madeira_banco marmore_xadrez vitral dourado quadro_negro ladrilho_hidraulico bronze"
mata="folhas_a bananeira_a bambu_a casca folhas_chao barro agua_rio madeira_ponte sape pau_a_pique rocha"
cemiterio="lapide_granito marmore_sujo terra_cova cruz_ferro_a musgo azulejo_capela"
ceu="ceu_noite ceu_nublado lua_a agua"
items="chave alicate martelo pe_de_cabra chave_de_fenda fusivel sal vela fosforo tabaco balde freio ossos cruz bilhete_onibus manivela remo lanterna_pilha foto"
all="$v1 $casa $cidade $igreja $mata $cemiterio $ceu $(for i in $items; do printf 'item_%s_a ' "$i"; done)"
for n in ${*:-$all}; do
    "t_$n"
done
