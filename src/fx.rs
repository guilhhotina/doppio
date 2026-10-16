use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct Fx(pub i32);

impl Fx {
    pub const SHIFT: u32 = 16;
    pub const ZERO: Fx = Fx(0);
    pub const ONE: Fx = Fx(1 << 16);
    pub const HALF: Fx = Fx(1 << 15);
    pub const MAX: Fx = Fx(i32::MAX);
    pub const MIN: Fx = Fx(i32::MIN);

    pub const fn int(i: i32) -> Fx {
        Fx(i << Self::SHIFT)
    }

    pub const fn ratio(num: i32, den: i32) -> Fx {
        Fx((((num as i64) << Self::SHIFT) / den as i64) as i32)
    }

    pub const fn floor(self) -> i32 {
        self.0 >> Self::SHIFT
    }

    pub const fn round(self) -> i32 {
        self.0.wrapping_add(1 << (Self::SHIFT - 1)) >> Self::SHIFT
    }

    pub const fn mul(self, o: Fx) -> Fx {
        Fx(((self.0 as i64 * o.0 as i64) >> Self::SHIFT) as i32)
    }

    pub const fn div(self, o: Fx) -> Fx {
        Fx((((self.0 as i64) << Self::SHIFT) / o.0 as i64) as i32)
    }

    pub const fn abs(self) -> Fx {
        Fx(self.0.wrapping_abs())
    }

    pub fn clamp(self, lo: Fx, hi: Fx) -> Fx {
        Ord::clamp(self, lo, hi)
    }

    pub fn to_f32(self) -> f32 {
        self.0 as f32 / 65536.0
    }

    pub fn parse(s: &str) -> Option<Fx> {
        let (neg, body) = s.strip_prefix('-').map_or((false, s), |rest| (true, rest));
        let (whole, frac) = body.split_once('.').unwrap_or((body, ""));
        let digits = |t: &str| !t.is_empty() && t.len() <= 9 && t.bytes().all(|b| b.is_ascii_digit());
        let whole_ok = digits(whole) && whole.parse::<i64>().ok().filter(|w| *w < 32768).is_some();
        let frac_ok = frac.is_empty() || digits(frac);
        (whole_ok && frac_ok && !(body.ends_with('.'))).then(|| {
            let w = whole.parse::<i64>().unwrap_or(0) << Self::SHIFT;
            let scale = 10i64.pow(frac.len() as u32);
            let f = frac.parse::<i64>().map_or(0, |f| ((f << Self::SHIFT) + scale / 2) / scale);
            let raw = w + f;
            Fx(if neg { -raw } else { raw } as i32)
        })
    }
}

impl Add for Fx {
    type Output = Fx;
    fn add(self, o: Fx) -> Fx {
        Fx(self.0.wrapping_add(o.0))
    }
}

impl Sub for Fx {
    type Output = Fx;
    fn sub(self, o: Fx) -> Fx {
        Fx(self.0.wrapping_sub(o.0))
    }
}

impl Neg for Fx {
    type Output = Fx;
    fn neg(self) -> Fx {
        Fx(self.0.wrapping_neg())
    }
}

impl Mul for Fx {
    type Output = Fx;
    fn mul(self, o: Fx) -> Fx {
        Fx::mul(self, o)
    }
}

impl Mul<i32> for Fx {
    type Output = Fx;
    fn mul(self, k: i32) -> Fx {
        Fx(self.0.wrapping_mul(k))
    }
}

impl Div for Fx {
    type Output = Fx;
    fn div(self, o: Fx) -> Fx {
        Fx::div(self, o)
    }
}

impl Div<i32> for Fx {
    type Output = Fx;
    fn div(self, k: i32) -> Fx {
        Fx(self.0 / k)
    }
}

impl AddAssign for Fx {
    fn add_assign(&mut self, o: Fx) {
        *self = *self + o;
    }
}

impl SubAssign for Fx {
    fn sub_assign(&mut self, o: Fx) {
        *self = *self - o;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct Ang(pub u16);

const QUARTER: u32 = 1 << 14;
const LUT_LEN: usize = 1024;
const LUT_STEP_SHIFT: u32 = 4;
const HALF_PI_Q62: i128 = 7244019458077122842;

const fn sin_q62(x: i128) -> i128 {
    let x2 = (x * x) >> 62;
    let mut term = x;
    let mut sum = x;
    let mut n = 1;
    while n < 14 {
        term = -((term * x2) >> 62) / ((2 * n) * (2 * n + 1));
        sum += term;
        n += 1;
    }
    sum
}

const SIN_LUT: [i32; LUT_LEN] = {
    let mut t = [0i32; LUT_LEN];
    let mut i = 0;
    while i < LUT_LEN {
        let x = HALF_PI_Q62 * i as i128 / LUT_LEN as i128;
        t[i] = ((sin_q62(x) + (1 << 45)) >> 46) as i32;
        i += 1;
    }
    t
};

const ATAN_TABLE: [u16; 15] = [8192, 4836, 2555, 1297, 651, 326, 163, 81, 41, 20, 10, 5, 3, 1, 1];

const fn lut(i: usize) -> i32 {
    if i >= LUT_LEN { 1 << 16 } else { SIN_LUT[i] }
}

const fn quarter_sin(r: u32) -> i32 {
    let i = (r >> LUT_STEP_SHIFT) as usize;
    let f = (r & ((1 << LUT_STEP_SHIFT) - 1)) as i32;
    let a = lut(i);
    a + (((lut(i + 1) - a) * f) >> LUT_STEP_SHIFT)
}

impl Ang {
    pub const ZERO: Ang = Ang(0);
    pub const QUARTER: Ang = Ang(QUARTER as u16);
    pub const HALF: Ang = Ang(1 << 15);

    pub fn deg(d: Fx) -> Ang {
        let full = 360i64 << Fx::SHIFT;
        let r = (d.0 as i64).rem_euclid(full);
        Ang((((r << 16) + full / 2) / full) as u16)
    }

    pub const fn sin(self) -> Fx {
        let a = self.0 as u32;
        let r = a & (QUARTER - 1);
        let v = match a >> 14 {
            0 => quarter_sin(r),
            1 => quarter_sin(QUARTER - r),
            2 => -quarter_sin(r),
            _ => -quarter_sin(QUARTER - r),
        };
        Fx(v)
    }

    pub const fn cos(self) -> Fx {
        Ang(self.0.wrapping_add(QUARTER as u16)).sin()
    }

    pub fn atan2(y: Fx, x: Fx) -> Ang {
        let (x0, y0, base) = match x.0 < 0 {
            true => (-(x.0 as i64), -(y.0 as i64), 1u16 << 15),
            false => (x.0 as i64, y.0 as i64, 0),
        };
        let bits = 64 - x0.abs().max(y0.abs()).leading_zeros();
        let shift = 40u32.saturating_sub(bits);
        let start = (x0 << shift, y0 << shift, 0i32);
        let (_, _, z) = ATAN_TABLE.iter().enumerate().fold(start, |(x, y, z), (i, &a)| match y > 0 {
            true => (x + (y >> i), y - (x >> i), z + a as i32),
            false => (x - (y >> i), y + (x >> i), z - a as i32),
        });
        Ang(base.wrapping_add(z as u16))
    }

    pub fn to_rad(self) -> f32 {
        self.0 as f32 * (std::f32::consts::TAU / 65536.0)
    }
}

impl Add for Ang {
    type Output = Ang;
    fn add(self, o: Ang) -> Ang {
        Ang(self.0.wrapping_add(o.0))
    }
}

impl Sub for Ang {
    type Output = Ang;
    fn sub(self, o: Ang) -> Ang {
        Ang(self.0.wrapping_sub(o.0))
    }
}

impl Neg for Ang {
    type Output = Ang;
    fn neg(self) -> Ang {
        Ang(self.0.wrapping_neg())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct Vec2 {
    pub x: Fx,
    pub y: Fx,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: Fx::ZERO, y: Fx::ZERO };

    pub const fn new(x: Fx, y: Fx) -> Vec2 {
        Vec2 { x, y }
    }

    pub const fn from_ang(a: Ang) -> Vec2 {
        Vec2 { x: a.sin(), y: a.cos() }
    }

    pub fn scale(self, k: Fx) -> Vec2 {
        Vec2::new(self.x * k, self.y * k)
    }

    pub fn dot(self, o: Vec2) -> Fx {
        Fx(((self.x.0 as i64 * o.x.0 as i64 + self.y.0 as i64 * o.y.0 as i64) >> Fx::SHIFT) as i32)
    }

    pub fn cross(self, o: Vec2) -> Fx {
        Fx(((self.x.0 as i64 * o.y.0 as i64 - self.y.0 as i64 * o.x.0 as i64) >> Fx::SHIFT) as i32)
    }

    pub fn len_sq(self) -> i64 {
        self.x.0 as i64 * self.x.0 as i64 + self.y.0 as i64 * self.y.0 as i64
    }

    pub fn len(self) -> Fx {
        Fx(isqrt(self.len_sq() as u64) as i32)
    }

    pub fn norm(self) -> Vec2 {
        let l = self.len();
        match l.0 {
            0 => Vec2::ZERO,
            _ => Vec2::new(self.x / l, self.y / l),
        }
    }

    pub fn rotate(self, a: Ang) -> Vec2 {
        let (s, c) = (a.sin(), a.cos());
        Vec2::new(self.x * c + self.y * s, self.y * c - self.x * s)
    }

    pub fn ang(self) -> Ang {
        Ang::atan2(self.x, self.y)
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, o: Vec2) {
        *self = *self + o;
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, o: Vec2) {
        *self = *self - o;
    }
}

pub fn isqrt(n: u64) -> u32 {
    (0..32).rev().fold(0u64, |root, bit| {
        let candidate = root | (1 << bit);
        match candidate * candidate <= n {
            true => candidate,
            false => root,
        }
    }) as u32
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rng(pub u32);

impl Rng {
    pub const fn new(seed: u32) -> Rng {
        Rng(if seed == 0 { 0x9E37_79B9 } else { seed })
    }

    pub fn next_u32(&mut self) -> u32 {
        let x = self.0 ^ (self.0 << 13);
        let x = x ^ (x >> 17);
        let x = x ^ (x << 5);
        self.0 = x;
        x
    }

    pub fn below(&mut self, n: u32) -> u32 {
        ((self.next_u32() as u64 * n as u64) >> 32) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul_div_roundtrip() {
        let a = Fx::int(3) + Fx::HALF;
        let b = Fx::int(-2);
        assert_eq!(a * b, Fx::int(-7));
        assert_eq!((a * b) / b, a);
        assert_eq!(Fx::ratio(1, 4), Fx(16384));
        assert_eq!(Fx(-1).floor(), -1);
        assert_eq!((Fx::int(2) + Fx::HALF).round(), 3);
    }

    #[test]
    fn parse_decimals() {
        assert_eq!(Fx::parse("12"), Some(Fx::int(12)));
        assert_eq!(Fx::parse("1.5"), Some(Fx::int(1) + Fx::HALF));
        assert_eq!(Fx::parse("-0.5"), Some(-Fx::HALF));
        assert_eq!(Fx::parse("0.1"), Some(Fx(6554)));
        assert_eq!(Fx::parse("x"), None);
        assert_eq!(Fx::parse("1."), None);
        assert_eq!(Fx::parse(""), None);
        assert_eq!(Fx::parse("40000"), None);
    }

    #[test]
    fn sin_cos_match_reference() {
        let worst = (0..=u16::MAX).step_by(7).map(Ang).fold(0i32, |w, a| {
            let r = a.0 as f64 * std::f64::consts::TAU / 65536.0;
            let es = (a.sin().0 - (r.sin() * 65536.0).round() as i32).abs();
            let ec = (a.cos().0 - (r.cos() * 65536.0).round() as i32).abs();
            w.max(es).max(ec)
        });
        assert!(worst <= 2, "worst error {worst}");
        assert_eq!(Ang(0).sin(), Fx::ZERO);
        assert_eq!(Ang::QUARTER.sin(), Fx::ONE);
        assert_eq!(Ang::HALF.cos(), -Fx::ONE);
    }

    #[test]
    fn atan2_inverts_from_ang() {
        let worst = (0..=u16::MAX).step_by(13).map(Ang).fold(0i32, |w, a| {
            let v = Vec2::from_ang(a).scale(Fx::int(5));
            let d = (v.ang().0.wrapping_sub(a.0) as i16 as i32).abs();
            let r = Vec2::from_ang(Ang(1000)).rotate(a).ang();
            let d = d.max((r.0.wrapping_sub(a.0.wrapping_add(1000)) as i16 as i32).abs());
            w.max(d)
        });
        assert!(worst <= 8, "worst error {worst}");
        assert_eq!(Ang::deg(Fx::int(90)), Ang::QUARTER);
        assert_eq!(Ang::deg(Fx::int(-90)), Ang(49152));
        assert_eq!(Ang::deg(Fx::int(360)), Ang(0));
    }

    #[test]
    fn isqrt_exact() {
        assert!((0..2000u64).chain([u64::MAX, 1 << 40, (1 << 40) - 1]).all(|n| {
            let r = isqrt(n) as u64;
            r * r <= n && (r + 1).checked_mul(r + 1).is_none_or(|s| s > n)
        }));
        assert_eq!(Vec2::new(Fx::int(3), Fx::int(4)).len(), Fx::int(5));
    }

    #[test]
    fn golden_digest() {
        let mut rng = Rng::new(7);
        let digest = (0..4096u32).fold(0u32, |h, i| {
            let a = Ang(rng.next_u32() as u16);
            let v = Vec2::from_ang(a).scale(Fx(rng.below(1 << 22) as i32));
            let mix = [a.sin().0, v.ang().0 as i32, v.len().0, (v.x / Fx::int(3)).0, Fx::ratio(i as i32, 7).0];
            mix.iter().fold(h, |h, &x| (h ^ x as u32).wrapping_mul(0x0100_0193))
        });
        assert_eq!(digest, 367778839);
    }

    #[test]
    fn rng_deterministic() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        let xs: Vec<u32> = (0..100).map(|_| a.next_u32()).collect();
        let ys: Vec<u32> = (0..100).map(|_| b.next_u32()).collect();
        assert_eq!(xs, ys);
        assert!(xs.iter().all(|&x| x != 0));
        assert!((0..1000).all(|_| a.below(6) < 6));
    }
}
