use super::mat::{self, M4, V3};

pub const UV_ONE: f32 = 2048.0;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Vert {
    pub pos: V3,
    pub uv: [i16; 2],
    pub nrm: [i8; 4],
    pub tile: [u8; 4],
}

impl Vert {
    pub fn new(pos: V3, uv: [f32; 2], n: V3) -> Vert {
        let q = |v: f32| (v * 127.0).round().clamp(-127.0, 127.0) as i8;
        let t = |v: f32| (v * UV_ONE).round().clamp(-32767.0, 32767.0) as i16;
        Vert { pos, uv: [t(uv[0]), t(uv[1])], nrm: [q(n[0]), q(n[1]), q(n[2]), 127], tile: [0; 4] }
    }

    pub fn normal(&self) -> V3 {
        [self.nrm[0] as f32 / 127.0, self.nrm[1] as f32 / 127.0, self.nrm[2] as f32 / 127.0]
    }

    pub fn with_light(self, l: f32) -> Vert {
        let mut v = self;
        v.nrm[3] = (l * 127.0).round().clamp(0.0, 127.0) as i8;
        v
    }

    pub fn with_tile(self, tile: [u8; 4]) -> Vert {
        Vert { tile, ..self }
    }
}

#[derive(Default, Clone)]
pub struct Mesh {
    pub verts: Vec<Vert>,
    pub idx: Vec<u16>,
}

impl Mesh {
    pub fn tris(&self) -> usize {
        self.idx.len() / 3
    }

    pub fn face(&mut self, p: &[V3], uv: &[[f32; 2]], n: V3) {
        let base = self.verts.len() as u16;
        self.verts.extend(p.iter().zip(uv).map(|(&p, &t)| Vert::new(p, t, n)));
        self.idx.extend((1..p.len() as u16 - 1).flat_map(|i| [base, base + i, base + i + 1]));
    }

    pub fn quad_facing(&mut self, p: [V3; 4], uv: [[f32; 2]; 4], n: V3) {
        let c = mat::cross(mat::sub(p[1], p[0]), mat::sub(p[2], p[0]));
        match mat::dot(c, n) >= 0.0 {
            true => self.face(&p, &uv, n),
            false => self.face(&[p[3], p[2], p[1], p[0]], &[uv[3], uv[2], uv[1], uv[0]], n),
        }
    }

    pub fn append(&mut self, other: &Mesh) {
        let base = self.verts.len() as u16;
        self.verts.extend_from_slice(&other.verts);
        self.idx.extend(other.idx.iter().map(|i| i + base));
    }
}

#[derive(Clone, Copy)]
pub struct Faces(pub u8);

impl Faces {
    pub const ALL: Faces = Faces(0x3f);
    pub const NO_BOTTOM: Faces = Faces(0x3f & !0x08);
    pub const NO_BACK: Faces = Faces(0x3f & !0x20);
    pub const NO_BOTTOM_BACK: Faces = Faces(0x3f & !0x28);

    fn has(self, i: usize) -> bool {
        self.0 >> i & 1 != 0
    }
}

pub struct Pen<'a> {
    pub mesh: &'a mut Mesh,
    pub xf: M4,
    pub density: f32,
}

impl Pen<'_> {
    fn emit(&mut self, local: &[V3], uv: &[[f32; 2]]) {
        let p: [V3; 4] = std::array::from_fn(|i| mat::point(&self.xf, local[i.min(local.len() - 1)]));
        let n = mat::norm(mat::cross(mat::sub(p[1], p[0]), mat::sub(p[2], p[0])));
        self.mesh.face(&p[..local.len()], uv, n);
    }

    pub fn quad(&mut self, p: [V3; 4], uv: [[f32; 2]; 4]) {
        self.emit(&p, &uv);
    }

    pub fn tri(&mut self, p: [V3; 3], uv: [[f32; 2]; 3]) {
        self.emit(&p, &uv);
    }

    pub fn cuboid(&mut self, lo: V3, hi: V3, faces: Faces) {
        let d = self.density;
        let (sx, sy, sz) = ((hi[0] - lo[0]) * d, (hi[1] - lo[1]) * d, (hi[2] - lo[2]) * d);
        let c = |x: usize, y: usize, z: usize| [[lo[0], hi[0]][x], [lo[1], hi[1]][y], [lo[2], hi[2]][z]];
        let quads: [([V3; 4], f32, f32); 6] = [
            ([c(1, 0, 1), c(1, 0, 0), c(1, 1, 0), c(1, 1, 1)], sz, sy),
            ([c(0, 0, 0), c(0, 0, 1), c(0, 1, 1), c(0, 1, 0)], sz, sy),
            ([c(0, 1, 1), c(1, 1, 1), c(1, 1, 0), c(0, 1, 0)], sx, sz),
            ([c(0, 0, 0), c(1, 0, 0), c(1, 0, 1), c(0, 0, 1)], sx, sz),
            ([c(0, 0, 1), c(1, 0, 1), c(1, 1, 1), c(0, 1, 1)], sx, sy),
            ([c(1, 0, 0), c(0, 0, 0), c(0, 1, 0), c(1, 1, 0)], sx, sy),
        ];
        quads.iter().enumerate().filter(|(i, _)| faces.has(*i)).for_each(|(_, (p, u, v))| {
            self.quad(*p, [[0.0, *v], [*u, *v], [*u, 0.0], [0.0, 0.0]]);
        });
    }

    pub fn panel(&mut self, lo: V3, hi: V3) {
        self.quad(
            [[lo[0], lo[1], hi[2]], [hi[0], lo[1], hi[2]], [hi[0], hi[1], hi[2]], [lo[0], hi[1], hi[2]]],
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        );
    }

    pub fn cylinder(&mut self, c: V3, r0: f32, r1: f32, h: f32, sides: usize, caps: bool) {
        let d = self.density;
        let ring = |i: usize, r: f32, y: f32| {
            let a = i as f32 / sides as f32 * std::f32::consts::TAU;
            [c[0] + a.sin() * r, c[1] + y, c[2] + a.cos() * r]
        };
        let circ = std::f32::consts::TAU * r0.max(r1) * d;
        (0..sides).for_each(|i| {
            let (u0, u1) = (i as f32 / sides as f32 * circ, (i + 1) as f32 / sides as f32 * circ);
            self.quad(
                [ring(i, r0, 0.0), ring(i + 1, r0, 0.0), ring(i + 1, r1, h), ring(i, r1, h)],
                [[u0, h * d], [u1, h * d], [u1, 0.0], [u0, 0.0]],
            );
        });
        if caps && r1 > 0.0 {
            let top: Vec<V3> = (0..sides).map(|i| ring(i, r1, h)).collect();
            let uv: Vec<[f32; 2]> = (0..sides)
                .map(|i| {
                    let a = i as f32 / sides as f32 * std::f32::consts::TAU;
                    [0.5 + a.sin() * 0.5, 0.5 + a.cos() * 0.5]
                })
                .collect();
            let p: Vec<V3> = top.iter().map(|&v| mat::point(&self.xf, v)).collect();
            self.mesh.face(&p, &uv, mat::norm(mat::dir(&self.xf, [0.0, 1.0, 0.0])));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuboid_faces_point_outward() {
        let mut m = Mesh::default();
        Pen { mesh: &mut m, xf: mat::IDENTITY, density: 1.0 }.cuboid([-1.0; 3], [1.0; 3], Faces::ALL);
        assert_eq!(m.tris(), 12);
        m.idx.chunks(3).for_each(|t| {
            let v = |i: u16| m.verts[i as usize];
            let c = mat::scale(mat::add(mat::add(v(t[0]).pos, v(t[1]).pos), v(t[2]).pos), 1.0 / 3.0);
            let n = mat::norm(mat::cross(mat::sub(v(t[1]).pos, v(t[0]).pos), mat::sub(v(t[2]).pos, v(t[0]).pos)));
            assert!(mat::dot(n, c) > 0.0);
            assert!(mat::dot(v(t[0]).normal(), n) > 0.9);
        });
    }

    #[test]
    fn cylinder_counts() {
        let mut m = Mesh::default();
        Pen { mesh: &mut m, xf: mat::IDENTITY, density: 1.0 }.cylinder([0.0; 3], 0.5, 0.5, 1.0, 8, true);
        assert_eq!(m.tris(), 16 + 6);
    }
}
