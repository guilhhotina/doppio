use super::level::HEAD;
use super::mat::{self, M4, V3};
use super::mesh::{Faces, Mesh, Pen, Vert};
use super::zones;
use crate::map::{Door, Map};

const SWING: f32 = 1.62;

pub fn mesh(tile: [u8; 4]) -> Mesh {
    let mut m = Mesh::default();
    let mut p = Pen { mesh: &mut m, xf: mat::IDENTITY, density: 1.0 };
    p.cuboid([0.03, 0.0, -0.025], [0.97, HEAD - 0.02, 0.025], Faces(0x07));
    p.panel([0.03, 0.0, 0.0], [0.97, HEAD - 0.02, 0.025]);
    p.xf = mat::rot_y(std::f32::consts::PI);
    p.panel([-0.97, 0.0, 0.0], [-0.03, HEAD - 0.02, 0.025]);
    m.verts.iter_mut().for_each(|v| {
        let leak = if v.pos[1] < 0.1 { 96 } else { 36 };
        *v = v.with_tile(tile).with_lit([42, 36, 30, leak]);
    });
    m
}

pub fn knob(brass: [u8; 4], plate: [u8; 4]) -> Mesh {
    let mut m = Mesh::default();
    [(-0.028f32, Faces(0x20)), (0.028, Faces(0x10))].iter().for_each(|&(z, f)| {
        Pen { mesh: &mut m, xf: mat::IDENTITY, density: 6.0 }.cuboid([0.77, 0.86, z - 0.002], [0.83, 1.02, z + 0.002], f);
    });
    let plates = m.verts.len();
    [(-0.048f32, Faces(0x27)), (0.048, Faces(0x17))].iter().for_each(|&(z, f)| {
        Pen { mesh: &mut m, xf: mat::IDENTITY, density: 8.0 }.cuboid([0.775, 0.94, z - 0.018], [0.825, 0.99, z + 0.018], f);
    });
    m.verts.iter_mut().enumerate().for_each(|(i, v)| {
        let (t, l) = if i < plates { (plate, [40, 35, 30, 65]) } else { (brass, [245, 215, 145, 225]) };
        *v = v.with_tile(t).with_lit(l);
    });
    m
}

pub fn gate(tile: [u8; 4]) -> Mesh {
    let mut m = Mesh::default();
    let q = [[0.03, 0.0, 0.0], [0.97, 0.0, 0.0], [0.97, HEAD, 0.0], [0.03, HEAD, 0.0]];
    let uv = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    m.face(&q, &uv, [0.0, 0.0, 1.0]);
    m.verts.iter_mut().for_each(|v| *v = v.with_tile(tile));
    m
}
pub fn padlock(body: [u8; 4], shackle: [u8; 4]) -> Mesh {
    let mut m = Mesh::default();
    let sides = [-0.045f32, 0.045];
    sides.iter().for_each(|&z| Pen { mesh: &mut m, xf: mat::IDENTITY, density: 8.0 }.cuboid([0.8, 0.95, z - 0.015], [0.86, 1.02, z + 0.015], Faces::ALL));
    let bodies = m.verts.len();
    sides.iter().for_each(|&z| {
        let mut p = Pen { mesh: &mut m, xf: mat::IDENTITY, density: 8.0 };
        p.cuboid([0.81, 1.02, z - 0.005], [0.82, 1.07, z + 0.005], Faces(0x33));
        p.cuboid([0.84, 1.02, z - 0.005], [0.85, 1.07, z + 0.005], Faces(0x33));
        p.cuboid([0.81, 1.07, z - 0.005], [0.85, 1.08, z + 0.005], Faces(0x3c));
    });
    m.verts.iter_mut().enumerate().for_each(|(i, v)| *v = v.with_tile(if i < bodies { body } else { shackle }));
    m
}

pub fn outdoor(map: &Map, z: &zones::Zones, d: &Door) -> bool {
    let sides: Vec<u8> = zones::DIRS.iter().filter_map(|o| z.room(d.x + o[0], d.y + o[1])).collect();
    !sides.is_empty() && sides.iter().all(|&r| map.rooms[r as usize].out)
}

pub fn model(map: &Map, d: &Door, open: u16, eye: V3) -> M4 {
    let (x, y) = (d.x as f32, d.y as f32);
    let a = open as f32 / 65535.0 * SWING;
    let (hinge, base, side) = match zones::solid(map.tile(d.x - 1, d.y)) {
        true => ([x, 0.0, y + 0.5], 0.0, if eye[2] > y + 0.5 { 1.0 } else { -1.0 }),
        false => ([x + 0.5, 0.0, y], -std::f32::consts::FRAC_PI_2, if eye[0] < x + 0.5 { 1.0 } else { -1.0 }),
    };
    mat::mul(&mat::translate(hinge), &mat::rot_y(base + side * a))
}

pub fn emit(mesh: &Mesh, model: &M4, moon: f32, lit: [u8; 4], out: &mut Vec<Vert>) {
    let q = |v: f32| (v * 127.0) as i8;
    out.extend(mesh.verts.iter().map(|v| {
        let n = mat::dir(model, v.normal());
        let l = std::array::from_fn(|k| lit[k].saturating_add(v.lit[k]));
        Vert { pos: mat::point(model, v.pos), uv: v.uv, nrm: [q(n[0]), q(n[1]), q(n[2]), q(moon)], tile: v.tile, lit: l }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;

    #[test]
    fn door_meshes_are_small() {
        assert!(mesh([0; 4]).tris() <= 12);
        assert_eq!(gate([0; 4]).tris(), 2);
        assert!(padlock([0; 4], [1; 4]).tris() <= 72);
    }

    #[test]
    fn open_leaves_swing_away_from_the_eye() {
        let m = map::parse(include_str!("testdata/rua.map")).expect("rua");
        m.doors.iter().for_each(|md| {
            let c = [md.x as f32 + 0.5, 1.0, md.y as f32 + 0.5];
            let plane = if zones::solid(m.tile(md.x - 1, md.y)) { 2 } else { 0 };
            [[2.5, 0.0, 0.0], [-2.5, 0.0, 0.0], [0.0, 0.0, 2.5], [0.0, 0.0, -2.5]].iter().for_each(|o| {
                let eye = mat::add(c, *o);
                let leaf = mat::point(&model(&m, md, 65535, eye), [0.5, 1.0, 0.0]);
                let (to_eye, to_leaf) = (mat::sub(eye, c), mat::sub(leaf, c));
                if to_eye[plane].abs() > 0.1 {
                    assert!(to_eye[plane] * to_leaf[plane] < 0.0, "door {},{} eye {:?} leaf {:?}", md.x, md.y, eye, leaf);
                }
            });
        });
    }
}
