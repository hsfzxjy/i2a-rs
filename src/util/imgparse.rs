use super::super::config::Config;
use super::grayscale::grayscale_to_char;
use super::size::adjust_size;
use image::{DynamicImage, GenericImageView};

pub fn img_to_ascii_color(
    img: &DynamicImage,
    out_height: usize,
    out_width: usize,
    config: &Config,
) -> String {
    let mut s = String::new();
    img_to_ascii(
        img,
        out_height,
        out_width,
        config.dup,
        config.padding,
        config.grayscale,
        |color, ch| {
            if ch == '\n' {
                return;
            }
            let colored = format!(
                "\x1b[38;2;{};{};{}m{}{}",
                color[0],
                color[1],
                color[2],
                ch,
                crossterm::Attribute::Reset,
            );
            s.push_str(colored.as_str());
        },
    );
    s
}

pub fn img_to_ascii<F>(
    img: &DynamicImage,
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
        let (w, h) = img.dimensions();
        w as f64 / h as f64
    };
    let (tb_width, tb_height, padding_x_l, padding_x_r, padding_y_l, padding_y_r) = adjust_size(
        aspect_ratio,
        out_width as usize,
        out_height as usize,
        dup,
        padding,
    );

    let mut thumbnail = img.thumbnail_exact(tb_width as u32, tb_height as u32);

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

