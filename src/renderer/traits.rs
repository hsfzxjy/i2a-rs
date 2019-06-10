use std::time::Duration;

pub trait Renderer {
    fn next(&mut self) -> (String, Duration);
    fn resize(&mut self, width: usize, height: usize);
}