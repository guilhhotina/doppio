use crate::assets::TEXTURES;
use crate::gl::{self, Gl};

pub fn id(name: &str) -> u8 {
    TEXTURES
        .binary_search_by(|(n, _)| (*n).cmp(name))
        .unwrap_or_else(|_| panic!("texture {name} missing"))
        as u8
}

pub fn alpha(name: &str) -> bool {
    name.ends_with("_a")
}

#[cfg(test)]
pub fn decode(tex: &[u8]) -> (i32, i32, Vec<u16>) {
    let (w, h) = (1i32 << tex[0], 1i32 << tex[1]);
    let pal: [u16; 16] = std::array::from_fn(|i| u16::from_le_bytes([tex[2 + i * 2], tex[3 + i * 2]]));
    let px = tex[34..].iter().flat_map(|b| [pal[(b & 15) as usize], pal[(b >> 4) as usize]]).collect();
    (w, h, px)
}

pub fn rgba5551(c: u16, clear: bool) -> u16 {
    let (r, g, b) = (c >> 11, c >> 6 & 31, c & 31);
    match clear {
        true => 0,
        false => r << 11 | g << 6 | b << 1 | 1,
    }
}

fn decode_rgba(name: &str, tex: &[u8]) -> (i32, i32, Vec<u16>) {
    let (w, h) = (1i32 << tex[0], 1i32 << tex[1]);
    let pal: [u16; 16] = std::array::from_fn(|i| u16::from_le_bytes([tex[2 + i * 2], tex[3 + i * 2]]));
    let keyed = alpha(name);
    let px = |i: u8| rgba5551(pal[i as usize], keyed && i == 0);
    (w, h, tex[34..].iter().flat_map(|b| [px(b & 15), px(b >> 4)]).collect())
}

fn params(gl: &Gl, wrap: gl::GLenum) {
    gl.tex_parameter(gl::TEXTURE_MIN_FILTER, gl::NEAREST);
    gl.tex_parameter(gl::TEXTURE_MAG_FILTER, gl::NEAREST);
    gl.tex_parameter(gl::TEXTURE_WRAP_S, wrap);
    gl.tex_parameter(gl::TEXTURE_WRAP_T, wrap);
}

pub fn upload_rgba5551(gl: &Gl, w: i32, h: i32, px: &[u16]) -> u32 {
    let t = gl.gen_texture();
    gl.bind_texture(0, t);
    gl.tex_image(gl::RGBA, w, h, gl::UNSIGNED_SHORT_5_5_5_1, Some(px));
    params(gl, gl::REPEAT);
    t
}

pub fn upload_luminance(gl: &Gl, w: i32, h: i32, px: &[u8]) -> u32 {
    let t = gl.gen_texture();
    gl.bind_texture(0, t);
    gl.pixel_store(gl::UNPACK_ALIGNMENT, 1);
    gl.tex_image(gl::LUMINANCE, w, h, gl::UNSIGNED_BYTE, Some(px));
    gl.pixel_store(gl::UNPACK_ALIGNMENT, 4);
    params(gl, gl::REPEAT);
    t
}

pub fn target(gl: &Gl, w: i32, h: i32) -> u32 {
    let t = gl.gen_texture();
    gl.bind_texture(0, t);
    gl.tex_image::<u16>(gl::RGB, w, h, gl::UNSIGNED_SHORT_5_6_5, None);
    params(gl, gl::CLAMP_TO_EDGE);
    t
}

pub const BAYER: [u8; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

pub fn dither() -> [u8; 16] {
    BAYER.map(|b| b * 16 + 8)
}

pub fn noise(n: usize, seed: u32) -> Vec<u8> {
    std::iter::successors(Some(seed | 1), |s| {
        let x = s ^ s << 13;
        let x = x ^ x >> 17;
        Some(x ^ x << 5)
    })
    .skip(1)
    .take(n)
    .map(|v| (v >> 24) as u8)
    .collect()
}

pub const ATLAS_W: i32 = 512;
pub const ATLAS_H: i32 = 1024;
pub const CELL: i32 = 32;

pub struct Atlas {
    pub px: Vec<u16>,
    pub tiles: Vec<[u8; 4]>,
}

fn size(tex: &[u8]) -> (i32, i32) {
    (1i32 << tex[0], 1i32 << tex[1])
}

pub fn layout() -> Vec<[u8; 4]> {
    let cells: Vec<(usize, usize)> = TEXTURES.iter().map(|(_, t)| size(t)).map(|(w, h)| ((w / CELL) as usize, (h / CELL) as usize)).collect();
    let mut order: Vec<usize> = (0..cells.len()).collect();
    order.sort_by_key(|&i| (std::cmp::Reverse(cells[i].1), std::cmp::Reverse(cells[i].0)));
    let (cols, rows) = ((ATLAS_W / CELL) as usize, (ATLAS_H / CELL) as usize);
    let mut sky = vec![0usize; cols];
    let mut tiles = vec![[0u8; 4]; cells.len()];
    order.iter().for_each(|&i| {
        let (w, h) = cells[i];
        let top = |sky: &[usize], x: usize| sky[x..x + w].iter().copied().max().unwrap_or(0);
        let x = (0..=cols - w).min_by_key(|&x| (top(&sky, x), x)).unwrap_or(0);
        let y = top(&sky, x);
        assert!(y + h <= rows, "texture atlas overflow");
        sky[x..x + w].iter_mut().for_each(|s| *s = y + h);
        tiles[i] = [x, y, w, h].map(|v| v as u8);
    });
    tiles
}

pub fn atlas() -> Atlas {
    let tiles = layout();
    let mut px = vec![0u16; (ATLAS_W * ATLAS_H) as usize];
    TEXTURES.iter().zip(&tiles).for_each(|((n, t), tile)| {
        let (w, _, src) = decode_rgba(n, t);
        let (x, y) = (tile[0] as i32 * CELL, tile[1] as i32 * CELL);
        src.chunks(w as usize).enumerate().for_each(|(row, line)| {
            let at = ((y + row as i32) * ATLAS_W + x) as usize;
            px[at..at + w as usize].copy_from_slice(line);
        });
    });
    Atlas { px, tiles }
}

pub struct Textures {
    pub atlas: u32,
    pub tiles: Vec<[u8; 4]>,
    pub dither: u32,
    pub noise: u32,
    pub bytes: usize,
}

impl Textures {
    pub fn new(gl: &Gl) -> Textures {
        let Atlas { px, tiles } = atlas();
        let atlas = upload_rgba5551(gl, ATLAS_W, ATLAS_H, &px);
        let dither = upload_luminance(gl, 4, 4, &dither());
        let noise = upload_luminance(gl, 64, 64, &noise(64 * 64, 0x5eed));
        gl.bind_texture(0, 0);
        Textures { atlas, tiles, dither, noise, bytes: px.len() * 2 + 16 + 64 * 64 }
    }

    pub fn tile(&self, name: &str) -> [u8; 4] {
        self.tiles[id(name) as usize]
    }

    pub fn delete(&self, gl: &Gl) {
        [self.atlas, self.dither, self.noise].iter().for_each(|&t| gl.delete_texture(t));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_expands_nibbles_low_first() {
        let mut t = vec![5u8, 5];
        t.extend((0..16u16).flat_map(|i| (i * 0x1111).to_le_bytes()));
        t.extend([0x21u8; 512]);
        let (w, h, px) = decode(&t);
        assert_eq!((w, h, px.len()), (32, 32, 1024));
        assert_eq!(&px[..2], &[0x1111, 0x2222]);
    }

    #[test]
    fn bayer_is_permutation() {
        let mut b = BAYER;
        b.sort();
        assert!(b.iter().enumerate().all(|(i, &v)| v as usize == i));
    }

    #[test]
    fn every_texture_resolves() {
        assert!(TEXTURES.iter().enumerate().all(|(i, (n, _))| id(n) as usize == i));
    }

    #[test]
    fn atlas_holds_every_texture_without_overlap() {
        let a = atlas();
        let cells = (ATLAS_W / CELL * ATLAS_H / CELL) as usize;
        let mut used = vec![false; cells];
        a.tiles.iter().zip(TEXTURES).for_each(|(t, (name, tex))| {
            let (w, h, px) = decode(tex);
            assert_eq!([t[2] as i32 * CELL, t[3] as i32 * CELL], [w, h], "{name}");
            (0..t[3] as i32).flat_map(|dy| (0..t[2] as i32).map(move |dx| (dx, dy))).for_each(|(dx, dy)| {
                let c = ((t[1] as i32 + dy) * (ATLAS_W / CELL) + t[0] as i32 + dx) as usize;
                assert!(!used[c], "{name} overlaps");
                used[c] = true;
            });
            let corner = ((t[1] as i32 * CELL + h - 1) * ATLAS_W + t[0] as i32 * CELL + w - 1) as usize;
            let last = px[px.len() - 1];
            assert_eq!(a.px[corner], rgba5551(last, alpha(name) && last == 0xF81F), "{name}");
        });
    }
}
