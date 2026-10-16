use crate::assets::TEXTURES;
use crate::gl::{self, Gl};

pub fn id(name: &str) -> u8 {
    TEXTURES
        .binary_search_by(|(n, _)| (*n).cmp(name))
        .unwrap_or_else(|_| panic!("texture {name} missing"))
        as u8
}

pub fn decode(tex: &[u8]) -> (i32, i32, Vec<u16>) {
    let (w, h) = (1i32 << tex[0], 1i32 << tex[1]);
    let pal: [u16; 16] = std::array::from_fn(|i| u16::from_le_bytes([tex[2 + i * 2], tex[3 + i * 2]]));
    let px = tex[34..].iter().flat_map(|b| [pal[(b & 15) as usize], pal[(b >> 4) as usize]]).collect();
    (w, h, px)
}

fn params(gl: &Gl, wrap: gl::GLenum) {
    gl.tex_parameter(gl::TEXTURE_MIN_FILTER, gl::NEAREST);
    gl.tex_parameter(gl::TEXTURE_MAG_FILTER, gl::NEAREST);
    gl.tex_parameter(gl::TEXTURE_WRAP_S, wrap);
    gl.tex_parameter(gl::TEXTURE_WRAP_T, wrap);
}

pub fn upload_rgb565(gl: &Gl, w: i32, h: i32, px: &[u16]) -> u32 {
    let t = gl.gen_texture();
    gl.bind_texture(0, t);
    gl.tex_image(gl::RGB, w, h, gl::UNSIGNED_SHORT_5_6_5, Some(px));
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

pub struct Textures {
    pub ids: Vec<u32>,
    pub dither: u32,
    pub noise: u32,
    pub bytes: usize,
}

impl Textures {
    pub fn new(gl: &Gl) -> Textures {
        let (ids, bytes) = TEXTURES.iter().fold((Vec::with_capacity(TEXTURES.len()), 0), |(mut ids, bytes), (_, t)| {
            let (w, h, px) = decode(t);
            ids.push(upload_rgb565(gl, w, h, &px));
            (ids, bytes + px.len() * 2)
        });
        let dither = upload_luminance(gl, 4, 4, &dither());
        let noise = upload_luminance(gl, 64, 64, &noise(64 * 64, 0x5eed));
        gl.bind_texture(0, 0);
        Textures { ids, dither, noise, bytes: bytes + 16 + 64 * 64 }
    }

    pub fn delete(&self, gl: &Gl) {
        self.ids.iter().chain([&self.dither, &self.noise]).for_each(|&t| gl.delete_texture(t));
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
}
