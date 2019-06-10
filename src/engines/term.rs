
use super::super::config::Config;
use super::super::renderer::{get_renderer, Renderer};

use std::io::Result;
use std::path::Path;
pub struct Terminal {
    renderer: Box<dyn Renderer>,
    old_size: (usize, usize),
}

impl Terminal {
    pub fn new(path: &Path) -> Terminal {
        Terminal {
            renderer: get_renderer(
                path,
                Config {
                    padding: true,
                    dup: true,
                    grayscale: false,
                },
            ),
            old_size: (0, 0),
        }
    }

    fn resize(&mut self, terminal: &crossterm::Terminal) -> Result<()> {
        let (width, height) = term_size::dimensions().unwrap();

        if self.old_size == (width, height) {
            return Ok(());
        }

        terminal.clear(crossterm::ClearType::All)?;
        println!("Parsing...");

        self.old_size = (width, height);

        self.renderer.resize(width, height);
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        if let Ok(_alternate) = crossterm::AlternateScreen::to_alternate(true) {
            let terminal = crossterm::Crossterm::new().terminal();

            let mut sync_input = crossterm::input().read_async();

            loop {
                if let Some(event) = sync_input.next() {
                    match event {
                        crossterm::InputEvent::Keyboard(crossterm::KeyEvent::Char('q')) => {
                            break;
                        }
                        _ => (),
                    }
                }

                self.resize(&terminal)?;
                terminal.clear(crossterm::ClearType::All)?;
                eprintln!("next");
                let (content, duration) = self.renderer.next();
                print!("{}", content);
                std::thread::sleep(duration);
            }
        }
        Ok(())
    }
}