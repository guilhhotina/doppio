use super::lamps::MOON_DIR;
use super::mat::{self, V3};
use super::mesh::{Mesh, Vert};
use super::texture;
use crate::map::Sky;
use std::f32::consts::TAU;

const R: f32 = 10.0;
const SEGS: usize = 12;
const WRAPS: f32 = 3.0;
const RINGS: [(f32, f32, f32); 5] = [(-0.35, 0.985, 1.0), (0.0, 0.94, 0.72), (0.26, 0.62, 0.24), (0.62, 0.3, 0.04), (1.1, 0.08, 0.0)];
const MOON_SIZE: f32 = 1.05;
fn vert(p: V3, uv: [f32; 2], fog: f32, tile: [u8; 4]) -> Vert {
    Vert::new(p, uv, [0.0, 1.0, 0.0]).with_light(fog).with_tile(tile)
}

pub fn mesh(sky: Sky, tiles: &[[u8; 4]]) -> Option<Mesh> {
    let tex = match sky {
        Sky::Moon | Sky::Night => "ceu_noite",
        Sky::Overcast => "ceu_nublado",
        Sky::None => return None,
    };
    let tile = tiles[texture::id(tex) as usize];
    let mut m = Mesh::default();
    let at = |e: f32, i: usize| {
        let a = i as f32 / SEGS as f32 * TAU;
        [a.sin() * e.cos() * R, e.sin() * R, a.cos() * e.cos() * R]
    };
    RINGS.iter().enumerate().for_each(|(k, &(e, v, fog))| {
        (0..=SEGS).for_each(|i| m.verts.push(vert(at(e, i), [i as f32 / SEGS as f32 * WRAPS, v], fog, tile)));
        if k > 0 {
            let (a, b) = (((k - 1) * (SEGS + 1)) as u16, (k * (SEGS + 1)) as u16);
            (0..SEGS as u16).for_each(|i| m.idx.extend([a + i, b + i, b + i + 1, a + i, b + i + 1, a + i + 1]));
        }
    });
    let apex = m.verts.len() as u16;
    m.verts.push(vert([0.0, R, 0.0], [0.5, 0.01], 0.0, tile));
    let last = ((RINGS.len() - 1) * (SEGS + 1)) as u16;
    (0..SEGS as u16).for_each(|i| m.idx.extend([last + i, apex, last + i + 1]));
    if sky == Sky::Moon {
        let moon = tiles[texture::id("lua_a") as usize];
        let d = mat::norm([-MOON_DIR[0], MOON_DIR[1] * 0.42, -MOON_DIR[2]]);
        let c = mat::scale(d, R * 0.9);
        let r = mat::scale(mat::norm(mat::cross(d, [0.0, 1.0, 0.0])), MOON_SIZE);
        let u = mat::scale(mat::norm(mat::cross(r, d)), MOON_SIZE);
        let base = m.verts.len() as u16;
        let corners = [(-1.0, -1.0, [0.02, 0.98]), (1.0, -1.0, [0.98, 0.98]), (1.0, 1.0, [0.98, 0.02]), (-1.0, 1.0, [0.02, 0.02])];
        m.verts.extend(corners.iter().map(|&(a, b, uv)| vert(mat::add(c, mat::add(mat::scale(r, a), mat::scale(u, b))), uv, 0.0, moon)));
        m.idx.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    Some(m)
}

#[cfg(test)]
pub fn moon_dir() -> V3 {
    mat::norm([-MOON_DIR[0], MOON_DIR[1] * 0.42, -MOON_DIR[2]])
}

#[cfg(test)]
pub fn above_horizon(dir: V3) -> f32 {
    dir[1].asin() / std::f32::consts::FRAC_PI_2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dome_faces_inward_and_moon_is_up() {
        let tiles = vec![[0, 0, 4, 2]; crate::assets::TEXTURES.len()];
        assert!(mesh(Sky::None, &tiles).is_none());
        let m = mesh(Sky::Moon, &tiles).expect("sky");
        assert!(m.tris() < 140, "{}", m.tris());
        m.idx.chunks(3).for_each(|t| {
            let p = |i: u16| m.verts[i as usize].pos;
            let c = mat::scale(mat::add(mat::add(p(t[0]), p(t[1])), p(t[2])), 1.0 / 3.0);
            let n = mat::cross(mat::sub(p(t[1]), p(t[0])), mat::sub(p(t[2]), p(t[0])));
            assert!(mat::dot(n, c) < 0.0, "triangle faces outward at {c:?}");
        });
        assert!(above_horizon(moon_dir()) > 0.2);
        assert!(mesh(Sky::Overcast, &tiles).expect("overcast").tris() < mesh(Sky::Moon, &tiles).expect("moon").tris());
    }
}
