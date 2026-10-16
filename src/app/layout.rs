use crate::render::{CANVAS_H, CANVAS_W};

pub fn rects(n: usize) -> [[u16; 4]; 4] {
    let (w, h) = (CANVAS_W, CANVAS_H);
    let (hw, hh) = (w / 2, h / 2);
    match n {
        0 | 1 => [[0, 0, w, h], [0; 4], [0; 4], [0; 4]],
        2 => [[0, 0, hw, h], [hw, 0, w - hw, h], [0; 4], [0; 4]],
        _ => [[0, 0, hw, hh], [hw, 0, w - hw, hh], [0, hh, hw, h - hh], [hw, hh, w - hw, h - hh]],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn views_tile_the_canvas() {
        [1usize, 2, 4].iter().for_each(|&n| {
            let area: u32 = rects(n).iter().take(n).map(|r| r[2] as u32 * r[3] as u32).sum();
            assert_eq!(area, CANVAS_W as u32 * CANVAS_H as u32);
        });
        assert_eq!(rects(3)[2], [0, 108, 192, 108]);
    }
}
