pub mod image_parser;
mod renderer;

#[macro_use]
extern crate lazy_static;


extern crate clap;

extern crate crossterm;
extern crate image;

use image_parser::ImageParser;
use renderer::TerminalRenderer;

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
    // let img = image::open(image_fn).unwrap_or_else(|_| {
    //     eprintln!("Cannot open image file '{}'.", image_fn);
    //     std::process::exit(1)
    // });

    if let Err(e) = TerminalRenderer::new(std::path::Path::new(image_fn)).handle() {
        eprintln!("{:?}", e);
        std::process::exit(1)
    };
}
