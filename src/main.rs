mod config;
mod engines;
mod renderer;
mod util;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate json;
extern crate clap;

extern crate crossterm;
extern crate image;

// use image_parser::ImageParser;
// use renderer::TerminalRenderer;

fn main() -> Result<(), std::boxed::Box<std::error::Error>> {
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
    engines::Terminal::new(std::path::Path::new(image_fn)).run()?;
    Ok(())
}
