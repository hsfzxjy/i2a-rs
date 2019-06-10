mod traits;
mod animation;
mod r#static;


use animation::AnimationRenderer;
use r#static::StaticRenderer;
pub use traits::Renderer;

use super::config::Config;
use std::boxed::Box;
use std::path::Path;
pub fn get_renderer(path: &Path, config: Config) -> Box<dyn Renderer> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map_or("".to_string(), |s| s.to_ascii_lowercase());

    match &ext[..] {
        "gif" => Box::new(AnimationRenderer::new(path, config)),
        _format => Box::new(StaticRenderer::new(path, config)),
    }
}