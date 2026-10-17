use super::art::{self, PAPER};
use super::cards::{self, Card};
use super::{Item, Party, PartySeat, RoomRow, Cues, credits, howto, hud, menu, overlay, party, rooms, title};
use crate::map::campaign;
use crate::render::{CANVAS_H, CANVAS_W, Ui2d, UiVert};
use crate::sim::{Dial, Setup, World};

const W: usize = CANVAS_W as usize;
const H: usize = CANVAS_H as usize;

fn texel(atlas: &[u16], u: i32, v: i32) -> [f32; 4] {
    let c = atlas[(v.clamp(0, 1023) * 512 + u.clamp(0, 511)) as usize];
    let e = |x: u16| (x & 31) as f32 / 31.0;
    [e(c >> 11), e(c >> 6), e(c >> 1), (c & 1) as f32]
}

fn raster(px: &mut [[f32; 3]], atlas: &[u16], q: &[UiVert]) {
    let (x0, y0, x1, y1) = (q[0].pos[0] as i32, q[0].pos[1] as i32, q[2].pos[0] as i32, q[2].pos[1] as i32);
    let (u0, v0, u1, v1) = (q[0].uv[0] as f32, q[0].uv[1] as f32, q[2].uv[0] as f32, q[2].uv[1] as f32);
    let col = q[0].col.map(|c| c as f32 / 255.0);
    (y0.max(0)..y1.min(H as i32)).for_each(|y| {
        (x0.max(0)..x1.min(W as i32)).for_each(|x| {
            let u = u0 + (u1 - u0) * ((x - x0) as f32 + 0.5) / (x1 - x0) as f32;
            let v = v0 + (v1 - v0) * ((y - y0) as f32 + 0.5) / (y1 - y0) as f32;
            let t = texel(atlas, u.floor() as i32, v.floor() as i32);
            let a = t[3] * col[3];
            let p = &mut px[y as usize * W + x as usize];
            (0..3).for_each(|k| p[k] = p[k] * (1.0 - a) + t[k] * col[k] * a);
        });
    });
}

fn shoot(name: &str, ui: &Ui2d, atlas: &[u16]) {
    let Ok(dir) = std::env::var("DOPPIO_UI_SHOTS") else { return };
    let bg = std::fs::read(format!("{dir}/bg.rgb")).ok().filter(|b| b.len() == W * H * 3);
    let mut px: Vec<[f32; 3]> = (0..W * H).map(|i| bg.as_ref().map_or([0.07, 0.06, 0.055], |b| [0, 1, 2].map(|k| b[i * 3 + k] as f32 / 255.0))).collect();
    ui.verts().chunks(4).for_each(|q| raster(&mut px, atlas, q));
    let mut out = format!("P6\n{W} {H}\n255\n").into_bytes();
    out.extend(px.iter().flat_map(|p| p.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)));
    std::fs::write(format!("{dir}/{name}.ppm"), out).expect("write shot");
}

fn check(name: &str, ui: &Ui2d, atlas: &[u16]) {
    assert!(ui.quads() > 0 && ui.quads() < crate::render::MAX_UI_QUADS * 3 / 4, "{name}: {} quads", ui.quads());
    assert!(ui.verts().iter().all(|v| (-64..=CANVAS_W as i16 + 64).contains(&v.pos[0]) && (-64..=CANVAS_H as i16 + 64).contains(&v.pos[1])), "{name}");
    shoot(name, ui, atlas);
}

#[test]
fn every_screen_draws_within_budget() {
    let atlas = crate::render::atlas_pixels();
    let world = World::new(campaign()[0], Setup { seed: 1, phase: 0, players: 1, characters: [0, 1, 2, 3] });
    let screen = |name: &str, f: &dyn Fn(&mut Ui2d)| {
        let mut ui = Ui2d::new();
        f(&mut ui);
        check(name, &ui, &atlas);
    };
    let items = [
        Item { note: Some("uma pessoa, fase a fase"), ..Item::new("Campanha") },
        Item { note: Some("dois a quatro controles nesta TV"), ..Item::new("Jogar no sofá") },
        Item { enabled: false, note: Some("não disponível nesta caixinha"), ..Item::new("Jogar online") },
        Item::new("Como jogar"),
        Item::new("Créditos"),
    ];
    screen("title", &|ui| title(ui, 0, "v0.9"));
    screen("menu", &|ui| {
        art::logo(ui, (CANVAS_W as i16 - art::LOGO_SIZE[0]) / 2, 2, crate::render::WHITE);
        menu(ui, 128, 76, &items, 1, 0);
    });
    screen("howto", &|ui| howto(ui, 0));
    screen("credits", &|ui| credits(ui, 0));
    screen("credits-lenda", &|ui| credits(ui, 1));
    let rows = [RoomRow { nick: "guily", players: 2, country: "BR" }, RoomRow { nick: "tia_nena", players: 1, country: "PT" }];
    screen("salas", &|ui| rooms(ui, &rows, 0, None, 0));
    let seats = [PartySeat { character: 0, nick: "controle 1", here: true, ready: false }, PartySeat { character: 2, nick: "controle 2", here: false, ready: true }];
    let party_items = [Item { value: Some("Benedita"), ..Item::new("Personagem") }, Item::new("Começar"), Item::new("Voltar")];
    screen("sofa", &|ui| party(ui, &Party { head: "Xique-Xique - o sobrado", sub: "fase 1 - lenda: Corpo-seco", seats: &seats, items: &party_items, sel: 0, focus: 0, footer: "outro controle: START entra, B sai" }, 0));
    [(Card::Intro, "intro"), (Card::Outro, "outro"), (Card::Restart, "restart"), (Card::Ending, "ending")].iter().for_each(|&(c, n)| {
        screen(&format!("card-{n}"), &|ui| cards::card(ui, c, &world, 0, 5));
    });
    let pause_items = [Item::new("Continuar"), Item::new("Sair")];
    screen("pause", &|ui| cards::pause(ui, &pause_items, 0, false, 0, &world, super::goal::goal(&world)));
    screen("notice", &|ui| cards::notice(ui, "Conexão perdida", "a outra TV saiu ou a rede caiu"));
    let cues = Cues::new(&world, u32::MAX, [true, false, false, false]);
    screen("hud", &|ui| hud(ui, &world, &cues, [0, 0, CANVAS_W, CANVAS_H], 0, 0));
    let dial = Dial { lock: 0, digits: [1, 9, 3, 7], len: 4, cursor: 1, wrong: 0 };
    screen("cadeado", &|ui| overlay::dial(ui, [0, 0, CANVAS_W, CANVAS_H], dial));
    screen("bilhete", &|ui| overlay::note(ui, [0, 0, CANVAS_W, CANVAS_H], "Não abra a porta do porão de noite. A senha do cadeado é o ano em que a igreja foi erguida, está na placa da praça."));
    screen("papel", &|ui| art::paper(ui, [40, 40, 300, 120], PAPER));
}
