use super::mat::V3;

const PANELS: [f32; 2] = [-0.72, 0.36];
const EDGES: usize = 5;
const STRIP: f32 = 0.09;
const ROD: f32 = 2.3;
const HEM: f32 = 0.25;
const HANG: f32 = 0.08;

pub fn quads(rows: usize, mut emit: impl FnMut([V3; 4], [[f32; 2]; 4])) {
    let drop = ROD - HEM;
    PANELS.iter().for_each(|&x0| {
        let at = |j: usize, k: usize| {
            let d = drop * k as f32 / rows as f32;
            let z = if j % 2 == 0 { 0.05 } else { 0.12 };
            ([x0 + j as f32 * STRIP, ROD - d * HANG.cos(), z + d * HANG.sin()], [j as f32 * 0.2, 4.0 * k as f32 / rows as f32])
        };
        (0..EDGES - 1).flat_map(|j| (0..rows).map(move |k| (j, k))).for_each(|(j, k)| {
            let q = [at(j, k + 1), at(j + 1, k + 1), at(j + 1, k), at(j, k)];
            emit(q.map(|v| v.0), q.map(|v| v.1));
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curtain_hangs_in_front_of_the_wall() {
        let mut n = 0;
        quads(1, |q, _| {
            n += 1;
            assert!(q.iter().all(|v| v[2] >= 0.05 && v[1] >= 0.2 && v[1] <= 2.3));
        });
        assert_eq!(n, PANELS.len() * (EDGES - 1));
    }
}
