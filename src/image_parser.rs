extern crate term_size;
use image::GenericImageView;
use std::f64;

lazy_static! {
    static ref GSCALE: Vec<char> = {
        let tables = r#"$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\|()1{}[]?-_+~i!lI;:,"^` "#;
        let mut vec: Vec<char> = tables.chars().collect();
        vec.reverse();
        vec
    };
    static ref GSCALE_NUM: usize = GSCALE.len();
}

fn adjust_size(
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

fn grayscale_to_char(gs: f64) -> char {
    GSCALE[(gs * (*GSCALE_NUM - 1) as f64) as usize]
}

pub struct ImageParser {
    img: image::DynamicImage,
    cache: String,
}

impl ImageParser {
    pub fn new(img: image::DynamicImage) -> ImageParser {
        ImageParser {
            img: img,
            cache: String::from(""),
        }
    }

    pub fn img_to_ascii<F>(
        &self,
        out_height: usize,
        out_width: usize,
        dup: bool,
        padding: bool,
        grayscale: bool,
        mut f: F,
    ) where
        F: FnMut([u8; 4], char) -> (),
    {
        let aspect_ratio = {
            let (w, h) = self.img.dimensions();
            w as f64 / h as f64
        };
        let (tb_width, tb_height, padding_x_l, padding_x_r, padding_y_l, padding_y_r) = adjust_size(
            aspect_ratio,
            out_width as usize,
            out_height as usize,
            dup,
            padding,
        );

        let mut thumbnail = self.img.thumbnail_exact(tb_width as u32, tb_height as u32);


        for _ in 0..padding_y_l {
            for _ in 0..out_width {
                f([0; 4], ' ');
            }
            f([0; 4], '\n');
        }

        if grayscale {
            thumbnail = thumbnail.grayscale();
        }

        for (x, _y, pixel) in thumbnail.to_rgba().enumerate_pixels() {
            if x == 0 {
                for _ in 0..padding_x_l {
                    f([0; 4], ' ');
                }
            }

            let ch = if grayscale {
                let cvalue = pixel[0] as f64 / 255.0;
                grayscale_to_char(cvalue)
            } else {
                '■'
            };

            f(pixel.data, ch);
            if dup {
                f(pixel.data, ch);
            }

            if x as usize == tb_width - 1 {
                for _ in 0..padding_x_r {
                    f([0; 4], ' ');
                }
                f([0; 4], '\n');
            }
        }

        for _ in 0..padding_y_r {
            for _ in 0..out_width {
                f([0; 4], ' ');
            }
            f([0; 4], '\n');
        }

    }

    pub fn render_terminal(
        &mut self,
        term: &crossterm::Terminal,
        rerender: bool,
    ) -> std::io::Result<()> {
        if self.cache.len() == 0 || rerender {
            let (w, h) = term_size::dimensions().unwrap();
            let mut cache = String::new();

            self.img_to_ascii(h as usize, w as usize, true, true, false, |color, ch| {
                if ch == '\n' {
                    return;
                }
                let s = format!(
                    "\x1b[38;2;{};{};{}m{}{}",
                    color[0],
                    color[1],
                    color[2],
                    ch,
                    crossterm::Attribute::Reset,
                );
                cache.push_str(s.as_str());
            });
            self.cache = cache;
        }
        term.clear(crossterm::ClearType::All)?;

        print!("{}", self.cache);
        // std::fs::write("google.txt", &self.cache);
        Ok(())
    }
}

