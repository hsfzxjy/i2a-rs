extern crate clap;
extern crate image;
#[macro_use]
extern crate lazy_static;

use image::GenericImageView;
use pancurses::{endwin, initscr, noecho, Input};
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

fn grayscale_to_char(gs: f64) -> char {
    GSCALE[(gs * (*GSCALE_NUM - 1) as f64) as usize]
}

struct Renderer<'a> {
    img: image::DynamicImage,
    window: &'a pancurses::Window,
}

impl<'a> Renderer<'a> {
    fn new(img: image::DynamicImage, window: &pancurses::Window) -> Renderer {
        Renderer {
            img: img,
            window: window,
        }
    }

    fn img_to_ascii(
        &self,
        out_height: usize,
        out_width: usize,
        dup: bool,
        padding: bool,
    ) -> String {
        let aspect_ratio = {
            let (w, h) = self.img.dimensions();
            w as f64 / h as f64
        };
        let (out_width, out_height, padding_x, padding_y, newline) = adjust_size(
            aspect_ratio,
            out_width as usize,
            out_height as usize,
            dup,
            padding,
        );

        let thumbnail = self
            .img
            .thumbnail_exact(out_width as u32, out_height as u32);
        let mut result = String::new();

        result.push_str(&"\n".repeat(padding_y as usize));
        for (x, _y, pixel) in thumbnail.grayscale().to_rgba().enumerate_pixels() {
            if x == 0 {
                result.push_str(&" ".repeat(padding_x as usize));
            }

            let cvalue = pixel[0] as f64 / 255.0;
            let ch = grayscale_to_char(cvalue);
            result.push(ch);
            if dup {
                result.push(ch);
            }

            if x as usize == out_width - 1 {
                result.push_str(&" ".repeat(padding_x as usize));
                if newline {
                    result.push('\n');
                }
            }
        }
        result
    }

    fn render(&self) {
        let s = self.img_to_ascii(
            self.window.get_max_y() as usize,
            self.window.get_max_x() as usize,
            true,
            true,
        );
        self.window.erase();
        self.window.mvaddstr(0, 0, s);
        self.window.refresh();
    }

}

fn adjust_size(
    aspect_ratio: f64,
    w: usize,
    h: usize,
    dup: bool,
    padding: bool,
) -> (usize, usize, usize, usize, bool) {
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
    let padding_y = if !padding { 0 } else { (h - new_h) / 2 };
    let newline = !padding || w != if dup { new_w * 2 } else { new_w } + padding_x * 2;
    (new_w, new_h, padding_x, padding_y, newline)
}

fn main() {
    let args = clap::App::new("i2a-rs")
        .author("hsfzxjy")
        .about("Show images in terminal using ASCII characters")
        .arg(
            clap::Arg::with_name("IMAGE")
                .help("The input image file")
                .required(true)
                .index(1),
        )
        .get_matches();

    let image_fn = args.value_of("IMAGE").unwrap();
    let img = image::open(image_fn).unwrap_or_else(|_| {
        eprintln!("Cannot open image file '{}'.", image_fn);
        std::process::exit(1)
    });

    let window = initscr();
    noecho();
    window.keypad(true);
    window.refresh();

    let renderer = Renderer::new(img, &window);
    renderer.render();

    let result = std::panic::catch_unwind(|| loop {
        pancurses::napms(50);
        match window.getch() {
            Some(Input::Character(x)) => match x {
                'q' => {
                    break;
                }
                _ => (),
            },
            Some(Input::KeyResize) => {
                renderer.render();
            }
            _ => (),
        }

    });
    endwin();
    pancurses::echo();


    if let Err(e) = result {
        if let Some(e) = e.downcast_ref::<&'static str>() {
            eprintln!("Error: {}", e);
        } else {
            eprintln!("Unknown error: {:?}", e);
        }
        std::process::exit(1);
    }
}
