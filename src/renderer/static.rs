use super::super::config::Config;
use super::traits::Renderer;
use image::DynamicImage;

use std::path::Path;
use std::time::Duration;

use super::super::util::img_to_ascii_color;

pub struct StaticRenderer {
    img: DynamicImage,
    config: Config,
    cache: String,
}

impl StaticRenderer {
    pub fn new(path: &Path, config: Config) -> StaticRenderer {
        StaticRenderer {
            img: image::open(path).unwrap(),
            config: config,
            cache: String::new(),
        }
    }

}

impl Renderer for StaticRenderer {
    fn next(&mut self) -> Option<(String, Duration)> {
        Some((self.cache.to_owned(), Duration::from_millis(100)))
    }
    fn resize(&mut self, width: usize, height: usize) {
        self.cache = img_to_ascii_color(&self.img, height, width, &self.config);
    }
}