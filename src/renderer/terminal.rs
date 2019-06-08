extern crate image;
use super::super::image_parser::ImageParser;
use std::path::Path;
trait Ticker {
    fn get_duration(&self) -> std::time::Duration;
    fn tick(&mut self, term: &crossterm::Terminal) -> std::io::Result<()>;
}

struct GifTicker {
    frame_parsers: Vec<(ImageParser, std::time::Duration)>,
    current_frame: i32,
}

impl GifTicker {
    fn new(name: &Path) -> GifTicker {
        use super::gifparse;
        let parsers: Vec<(ImageParser, std::time::Duration)> = gifparse::gif_to_frames(name)
            .unwrap_or_else(|e| panic!(e))
            .into_iter()
            .map(|f| (ImageParser::new(f), std::time::Duration::from_millis(100)))
            .collect();
        println!("{}", parsers.len());

        GifTicker {
            frame_parsers: parsers,
            current_frame: 0,
        }
    }
}

impl Ticker for GifTicker {
    fn get_duration(&self) -> std::time::Duration {
        self.frame_parsers[self.current_frame as usize].1
    }
    fn tick(&mut self, term: &crossterm::Terminal) -> std::io::Result<()> {
        // term.clear(crossterm::ClearType::All)?;
        // let mut parser = &self.frame_parsers[self.current_frame as usize].0;
        (self.frame_parsers[self.current_frame as usize].0).render_terminal(&term, false)?;
        self.current_frame = (self.current_frame + 1) % self.frame_parsers.len() as i32;
        Ok(())
    }
}

struct StaticTicker {
    img_parser: ImageParser,
    old_size: (usize, usize),
}

impl StaticTicker {
    fn new(name: &Path) -> StaticTicker {
        let img = image::open(name).unwrap_or_else(|_| {
            eprintln!("Cannot open image file '{:?}'.", name);
            std::process::exit(1)
        });
        StaticTicker {
            img_parser: ImageParser::new(img),
            old_size: (0, 0),
        }
    }
}

impl Ticker for StaticTicker {
    fn get_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(100)
    }
    fn tick(&mut self, term: &crossterm::Terminal) -> std::io::Result<()> {
        let new_size = term_size::dimensions().unwrap();
        if new_size == self.old_size {
            return Ok(());
        }

        self.old_size = new_size;
        self.img_parser.render_terminal(&term, true)?;
        Ok(())
    }
}

enum DynamicTicker {
    Static(StaticTicker),
    Gif(GifTicker),
}

impl DynamicTicker {
    fn from_filename(filename: &Path) -> DynamicTicker {
        let ext = filename
            .extension()
            .and_then(|s| s.to_str())
            .map_or("".to_string(), |s| s.to_ascii_lowercase());


        match &ext[..] {
            "gif" => DynamicTicker::Gif(GifTicker::new(filename)),
            _format => DynamicTicker::Static(StaticTicker::new(filename)),
        }
    }
}

impl Ticker for DynamicTicker {
    fn get_duration(&self) -> std::time::Duration {
        match self {
            DynamicTicker::Static(t) => t.get_duration(),
            DynamicTicker::Gif(t) => t.get_duration(),
        }
    }
    fn tick(&mut self, term: &crossterm::Terminal) -> std::io::Result<()> {
        match self {
            DynamicTicker::Static(t) => t.tick(term),
            DynamicTicker::Gif(t) => t.tick(term),
        }
    }
}

pub struct TerminalRenderer {
    ticker: DynamicTicker,
}


impl TerminalRenderer {
    pub fn new(filename: &Path) -> TerminalRenderer {
        TerminalRenderer {
            ticker: DynamicTicker::from_filename(filename),
        }
    }
    pub fn handle(&mut self) -> std::io::Result<()> {
        if let Ok(_alternate) = crossterm::AlternateScreen::to_alternate(true) {
            let ct = crossterm::Crossterm::new();
            let terminal = ct.terminal();
            self.start_main(&terminal)?;

        }
        Ok(())
    }

    fn start_main(&mut self, terminal: &crossterm::Terminal) -> std::io::Result<()> {
        let input = crossterm::input();
        let mut sync_stdin = input.read_async();
        loop {
            if let Some(event) = sync_stdin.next() {
                match event {
                    crossterm::InputEvent::Keyboard(crossterm::KeyEvent::Char('q')) => {
                        break;
                    }
                    _ => (),
                }
            }

            self.ticker.tick(&terminal);
            std::thread::sleep(self.ticker.get_duration());
        }
        Ok(())
    }
}
