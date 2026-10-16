use super::mat::{self, M4, V3};
use super::mesh::{Mesh, Vert};

const SIDES: u16 = 10;
const RINGS: [(f32, f32, f32); 2] = [(0.75, 0.32, 0.07), (1.8, 0.75, 0.0)];
const APEX: f32 = 0.2;

pub fn cone() -> Mesh {
    let step = std::f32::consts::TAU / SIDES as f32;
    let ring = |k: usize, i: u16| {
        let (len, r, glow) = RINGS[k];
        let a = i as f32 * step;
        Vert::new([a.sin() * r, a.cos() * r, len], [glow, 0.0], mat::norm([a.sin(), a.cos(), -r / len]))
    };
    let mut m = Mesh::default();
    m.verts.push(Vert::new([0.0; 3], [APEX, 1.0], [0.0, 0.0, 1.0]));
    (0..RINGS.len()).for_each(|k| m.verts.extend((0..SIDES).map(|i| ring(k, i))));
    let at = |k: u16, i: u16| 1 + k * SIDES + i % SIDES;
    (0..SIDES).for_each(|i| {
        m.idx.extend([0, at(0, i), at(0, i + 1)]);
        m.idx.extend([at(0, i), at(1, i), at(1, i + 1), at(0, i), at(1, i + 1), at(0, i + 1)]);
    });
    m
}

pub fn model(pos: V3, dir: V3) -> M4 {
    mat::mul(&mat::mul(&mat::translate(pos), &mat::rot_y(dir[0].atan2(dir[2]))), &mat::rot_x((-dir[1]).asin()))
}

pub fn emit(mesh: &Mesh, model: &M4, out: &mut Vec<Vert>) {
    let q = |v: f32| (v * 127.0) as i8;
    out.extend(mesh.verts.iter().map(|v| {
        let n = mat::dir(model, v.normal());
        Vert { pos: mat::point(model, v.pos), nrm: [q(n[0]), q(n[1]), q(n[2]), 0], tile: [255; 4], ..*v }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::mesh::UV_ONE;

    #[test]
    fn cone_fades_out_and_points_along_dir() {
        let c = cone();
        assert_eq!(c.tris(), 3 * SIDES as usize);
        assert!(c.idx.iter().all(|&i| (i as usize) < c.verts.len()));
        let glow = |v: &Vert| v.uv[0] as f32 / UV_ONE;
        assert!(c.verts[1..].iter().all(|v| glow(v) < glow(&c.verts[0])));
        assert!(c.verts.iter().filter(|v| v.pos[2] > 1.0).all(|v| glow(v) == 0.0));
        let mut out = Vec::new();
        emit(&c, &model([1.0, 1.0, 1.0], mat::norm([1.0, -0.3, 0.0])), &mut out);
        let tip = out.iter().map(|v| v.pos[0]).fold(f32::MIN, f32::max);
        assert!(tip > 2.5, "{tip}");
    }
}
