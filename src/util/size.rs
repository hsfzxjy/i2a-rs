use std::f64;


pub fn adjust_size(
    aspect_ratio: f64,
    w: usize,
    h: usize,
    dup: bool,
    padding: bool,
) -> (usize, usize, usize, usize, usize, usize) {
    let adjusted_w = if dup { w / 2 } else { w };
    let tmp_w = (aspect_ratio * h as f64) as usize;
    let mut new_w = adjusted_w;
    let mut new_h = h;

    if tmp_w < adjusted_w {
        new_w = tmp_w;
    } else {
        new_h = (adjusted_w as f64 / aspect_ratio) as usize;
    };
    let padding_x = if !padding {
        0
    } else if dup {
        (w - new_w * 2) / 2
    } else {
        (w - new_w) / 2
    };
    let padding_x_r = if dup {
        w - new_w * 2 - padding_x
    } else {
        w - new_w - padding_x
    };
    let padding_y = if !padding { 0 } else { (h - new_h) / 2 };
    (
        new_w,
        new_h,
        padding_x,
        padding_x_r,
        padding_y,
        h - new_h - padding_y,
    )
}

