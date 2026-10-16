use super::level;
use super::mat::{self, M4, V3};
use super::mesh::{Faces, Mesh, Pen, Vert};
use crate::map::{Map, Tile};
use crate::sim::Door;

const SWING: f32 = 1.62;

pub fn mesh(tile: [u8; 4]) -> Mesh {
    let mut m = Mesh::default();
    let mut p = Pen { mesh: &mut m, xf: mat::IDENTITY, density: 1.0 };
    p.cuboid([0.03, 0.0, -0.025], [0.97, level::HEAD - 0.02, 0.025], Faces(0x0f));
    p.panel([0.03, 0.0, 0.0], [0.97, level::HEAD - 0.02, 0.025]);
    p.xf = mat::rot_y(std::f32::consts::PI);
    p.panel([-0.97, 0.0, 0.0], [-0.03, level::HEAD - 0.02, 0.025]);
    m.verts.iter_mut().for_each(|v| *v = v.with_tile(tile));
    m
}

pub fn model(map: &Map, d: &Door, eye: V3) -> M4 {
    let (x, y) = (d.x as f32, d.y as f32);
    let open = d.open as f32 / 65535.0 * SWING;
    let (hinge, base, side) = match matches!(map.tile(d.x as i32 - 1, d.y as i32), Tile::Wall | Tile::Window) {
        true => ([x, 0.0, y + 0.5], 0.0, if eye[2] > y + 0.5 { 1.0 } else { -1.0 }),
        false => ([x + 0.5, 0.0, y], -std::f32::consts::FRAC_PI_2, if eye[0] < x + 0.5 { 1.0 } else { -1.0 }),
    };
    mat::mul(&mat::translate(hinge), &mat::rot_y(base + side * open))
}

pub fn emit(mesh: &Mesh, model: &M4, light: f32, out: &mut Vec<Vert>) {
    let q = |v: f32| (v * 127.0) as i8;
    out.extend(mesh.verts.iter().map(|v| {
        let n = mat::dir(model, v.normal());
        Vert { pos: mat::point(model, v.pos), uv: v.uv, nrm: [q(n[0]), q(n[1]), q(n[2]), q(light)], tile: v.tile }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    #[test]
    fn door_mesh_is_small() {
        assert!(mesh([0; 4]).tris() <= 12);
    }

    #[test]
    fn open_leaves_swing_away_from_the_eye() {
        let m = map::sobrado();
        m.doors.iter().for_each(|md| {
            let d = Door { x: md.x as u8, y: md.y as u8, vertical: false, open: 65535, locked: false };
            let c = [md.x as f32 + 0.5, 1.0, md.y as f32 + 0.5];
            let leaf = mat::point(&model(m, &d, c), [0.5, 1.0, 0.0]);
            let plane = if matches!(m.tile(md.x - 1, md.y), Tile::Wall | Tile::Window) { 2 } else { 0 };
            [[2.5, 0.0, 0.0], [-2.5, 0.0, 0.0], [0.0, 0.0, 2.5], [0.0, 0.0, -2.5]].iter().for_each(|o| {
                let eye = mat::add(c, *o);
                let leaf = mat::point(&model(m, &d, eye), [0.5, 1.0, 0.0]);
                let (to_eye, to_leaf) = (mat::sub(eye, c), mat::sub(leaf, c));
                if to_eye[plane].abs() > 0.1 {
                    assert!(to_eye[plane] * to_leaf[plane] < 0.0, "door {},{} eye {:?} leaf {:?}", md.x, md.y, eye, leaf);
                }
            });
            assert!(mat::len(mat::sub(leaf, c)) < 1.2);
        });
    }
}
