pub type V3 = [f32; 3];
pub type M4 = [f32; 16];

pub const IDENTITY: M4 = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];

pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn scale(a: V3, k: f32) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

pub fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub fn len(a: V3) -> f32 {
    dot(a, a).sqrt()
}

pub fn norm(a: V3) -> V3 {
    let l = len(a);
    if l > 1e-9 { scale(a, 1.0 / l) } else { [0.0, 1.0, 0.0] }
}

pub fn mul(a: &M4, b: &M4) -> M4 {
    std::array::from_fn(|i| {
        let (c, r) = (i / 4, i % 4);
        (0..4).map(|k| a[k * 4 + r] * b[c * 4 + k]).sum()
    })
}

pub fn translate(t: V3) -> M4 {
    let mut m = IDENTITY;
    m[12..15].copy_from_slice(&t);
    m
}

pub fn rot_x(a: f32) -> M4 {
    let (s, c) = a.sin_cos();
    [1.0, 0.0, 0.0, 0.0, 0.0, c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0]
}

pub fn rot_y(a: f32) -> M4 {
    let (s, c) = a.sin_cos();
    [c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0]
}

pub fn rot_z(a: f32) -> M4 {
    let (s, c) = a.sin_cos();
    [c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
}

pub fn scaling(s: V3) -> M4 {
    [s[0], 0.0, 0.0, 0.0, 0.0, s[1], 0.0, 0.0, 0.0, 0.0, s[2], 0.0, 0.0, 0.0, 0.0, 1.0]
}

pub fn point(m: &M4, p: V3) -> V3 {
    [
        m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12],
        m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13],
        m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14],
    ]
}

pub fn dir(m: &M4, d: V3) -> V3 {
    [
        m[0] * d[0] + m[4] * d[1] + m[8] * d[2],
        m[1] * d[0] + m[5] * d[1] + m[9] * d[2],
        m[2] * d[0] + m[6] * d[1] + m[10] * d[2],
    ]
}

pub fn look_at(eye: V3, target: V3) -> M4 {
    let f = norm(sub(target, eye));
    let up = if f[1].abs() > 0.99 { [0.0, 0.0, -1.0] } else { [0.0, 1.0, 0.0] };
    let s = norm(cross(f, up));
    let u = cross(s, f);
    [
        s[0], u[0], -f[0], 0.0,
        s[1], u[1], -f[1], 0.0,
        s[2], u[2], -f[2], 0.0,
        -dot(s, eye), -dot(u, eye), dot(f, eye), 1.0,
    ]
}

pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> M4 {
    let f = 1.0 / (fovy * 0.5).tan();
    let d = near - far;
    [
        f / aspect, 0.0, 0.0, 0.0,
        0.0, f, 0.0, 0.0,
        0.0, 0.0, (far + near) / d, -1.0,
        0.0, 0.0, 2.0 * far * near / d, 0.0,
    ]
}

#[derive(Clone, Copy)]
pub struct Frustum([[f32; 4]; 6]);

impl Frustum {
    pub fn new(m: &M4) -> Frustum {
        let row = |i: usize| [m[i], m[4 + i], m[8 + i], m[12 + i]];
        let (r0, r1, r2, r3) = (row(0), row(1), row(2), row(3));
        let comb = |a: [f32; 4], b: [f32; 4], s: f32| std::array::from_fn(|k| a[k] + s * b[k]);
        Frustum([
            comb(r3, r0, 1.0),
            comb(r3, r0, -1.0),
            comb(r3, r1, 1.0),
            comb(r3, r1, -1.0),
            comb(r3, r2, 1.0),
            comb(r3, r2, -1.0),
        ])
    }

    pub fn aabb(&self, lo: V3, hi: V3) -> bool {
        self.0.iter().all(|p| {
            let v = [
                if p[0] >= 0.0 { hi[0] } else { lo[0] },
                if p[1] >= 0.0 { hi[1] } else { lo[1] },
                if p[2] >= 0.0 { hi[2] } else { lo[2] },
            ];
            p[0] * v[0] + p[1] * v[1] + p[2] * v[2] + p[3] >= 0.0
        })
    }

    pub fn sphere(&self, c: V3, r: f32) -> bool {
        self.aabb(sub(c, [r; 3]), add(c, [r; 3]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: V3, b: V3) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4)
    }

    #[test]
    fn look_at_puts_target_on_negative_z() {
        let v = look_at([1.0, 2.0, 3.0], [4.0, 2.0, 7.0]);
        assert!(close(point(&v, [4.0, 2.0, 7.0]), [0.0, 0.0, -5.0]));
        assert!(close(point(&v, [1.0, 2.0, 3.0]), [0.0; 3]));
    }

    #[test]
    fn rot_y_maps_front_like_from_ang() {
        let a = 0.7f32;
        assert!(close(dir(&rot_y(a), [0.0, 0.0, 1.0]), [a.sin(), 0.0, a.cos()]));
    }

    #[test]
    fn frustum_culls_behind_camera() {
        let vp = mul(&perspective(1.0, 16.0 / 9.0, 0.1, 30.0), &look_at([0.0, 1.0, 0.0], [0.0, 1.0, 5.0]));
        let f = Frustum::new(&vp);
        assert!(f.aabb([-0.5, 0.0, 4.0], [0.5, 2.0, 6.0]));
        assert!(!f.aabb([-0.5, 0.0, -6.0], [0.5, 2.0, -4.0]));
        assert!(!f.aabb([40.0, 0.0, 4.0], [41.0, 2.0, 6.0]));
    }

    #[test]
    fn mul_composes() {
        let m = mul(&translate([1.0, 0.0, 0.0]), &rot_y(std::f32::consts::FRAC_PI_2));
        assert!(close(point(&m, [0.0, 0.0, 1.0]), [2.0, 0.0, 0.0]));
    }
}
