use super::super::config::Config;
use super::traits::Renderer;
use std::path::{Path, PathBuf};
use std::time::Duration;

type Frame = String;

pub struct AnimationRenderer {
    frames: Vec<Frame>,
    config: Config,
    filename: PathBuf,
    duration: Duration,
    current_index: usize,
}

impl AnimationRenderer {
    pub fn new(path: &Path, config: Config) -> AnimationRenderer {
        AnimationRenderer {
            frames: Vec::new(),
            config: config,
            filename: path.to_path_buf(),
            duration: Duration::from_millis(100),
            current_index: 0,
        }
    }
}

impl Renderer for AnimationRenderer {
    fn next(&mut self) -> Option<(String, Duration)> {
        let result = (self.frames[self.current_index].to_owned(), self.duration);
        self.current_index = (self.current_index + 1) % self.frames.len();

        if self.current_index == 0 && self.config.once {
            None
        } else {
            Some(result)
        }
    }
    fn resize(&mut self, width: usize, height: usize) {
        use super::super::util::Gif;
        let (frames, duration) = Gif::new(self.filename.as_path(), width, height, &self.config)
            .unwrap()
            .into_parse()
            .unwrap();
        self.frames = frames;
        self.duration = duration;
        self.current_index = 0;
    }
}