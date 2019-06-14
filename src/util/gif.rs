extern crate data_encoding;
extern crate json;
extern crate regex;

use super::super::config::Config;
use super::imgparse::img_to_ascii_color;
use data_encoding::HEXUPPER;
use ring::digest::{Context, SHA256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

type Result<T> = std::result::Result<T, std::boxed::Box<std::error::Error>>;

fn sha256_digest(path: &Path) -> Result<String> {
    let mut reader = std::fs::File::open(path)?;
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    let result = HEXUPPER.encode(context.finish().as_ref());
    Ok(result)
}

pub struct Gif<'a> {
    filename: PathBuf,
    dirname: PathBuf,
    width: usize,
    height: usize,
    config: &'a Config,
}

type GifParseResult = Result<(Vec<String>, Duration)>;

#[derive(Debug, Clone)]
struct CacheReadError;

impl std::fmt::Display for CacheReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cannot parse cache file.")
    }
}

impl std::error::Error for CacheReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl<'a> Gif<'a> {
    pub fn new(
        filename: &'a Path,
        width: usize,
        height: usize,
        config: &'a Config,
    ) -> Result<Gif<'a>> {
        Ok(Gif {
            filename: filename.to_path_buf(),
            dirname: get_cache_dirname(filename, width, height, config)?,
            width: width,
            height: height,
            config: config,
        })
    }

    fn from_json(&self) -> GifParseResult {
        let content = std::fs::read_to_string(self.dirname.join("content.json"))?;
        if let Ok(data) = json::parse(content.as_str()) {
            let duration = data["duration"].as_u64().ok_or(CacheReadError)?;
            if let json::JsonValue::Array(caches) = &data["frames"] {
                let mut arr = Vec::new();
                for value in caches.into_iter() {
                    arr.push(String::from(value.as_str().ok_or(CacheReadError)?))
                }
                Ok((arr, Duration::from_millis(duration)))
            } else {
                None.ok_or(CacheReadError)?
            }
        } else {
            None.ok_or(CacheReadError)?
        }
    }

    fn parse_ffmpeg_output(&self, output: &str) -> Result<u64> {
        let re = regex::Regex::new(r"frame=?\s*(\d+)").unwrap();

        let frames = re
            .captures(&output)
            .unwrap()
            .get(1)
            .unwrap_or_else(|| panic!("Cannot read frames information from output."))
            .as_str()
            .parse::<f64>()?;

        let re = regex::Regex::new(r"time=(\d{2}):(\d{2}):(\d{2}.\d{2})").unwrap();
        let cap = re.captures(output).unwrap();
        let hours = cap.get(1).unwrap().as_str().parse::<f64>()?;
        let minutes = cap.get(2).unwrap().as_str().parse::<f64>()?;
        let seconds = cap.get(3).unwrap().as_str().parse::<f64>()?;
        let total_msecs = 1000. * ((hours * 60. + minutes) * 60. + seconds);
        Ok((total_msecs / frames) as u64)
    }

    fn run_ffmpeg(&self) -> Result<u64> {
        let output = std::process::Command::new("ffmpeg")
            .args(&[
                "-i",
                self.filename.to_str().unwrap(),
                "-vf",
                format!(
                    "scale=w={}:h={}:force_original_aspect_ratio=decrease",
                    self.width, self.height
                )
                .as_str(),
                format!("{}/%05d.jpg", self.dirname.to_str().unwrap()).as_str(),
            ])
            .output()?;

        if !output.status.success() {
            panic!(
                "`ffmpeg` did not exit successfully.\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        eprintln!("{}", String::from_utf8_lossy(&output.stderr));

        self.parse_ffmpeg_output(&String::from_utf8_lossy(&output.stderr))
    }

    fn get_frames_from_file(&self) -> Result<Vec<String>> {
        let mut result = Vec::new();
        for i in 1.. {
            let filename = self.dirname.join(format!("{:05}.jpg", i));
            if let Ok(image) = image::open(filename) {
                result.push(img_to_ascii_color(
                    &image,
                    self.height,
                    self.width,
                    &self.config,
                ))
            } else {
                break;
            }
        }
        Ok(result)
    }

    fn save_cache(&self, frames: &Vec<String>, duration: u64) -> Result<()> {
        let frames_json: Vec<json::JsonValue> = frames
            .iter()
            .map(|s| json::JsonValue::String(s.to_owned()))
            .collect();
        let data = object! {
            "duration" => duration,
            "frames" => frames_json,
        };
        std::fs::write(self.dirname.join("content.json"), json::stringify(data))?;
        Ok(())
    }

    fn use_ffmpeg(&self) -> GifParseResult {
        let msecs = self.run_ffmpeg()?;
        let duration = Duration::from_millis(msecs);
        let frames = self.get_frames_from_file()?;
        self.save_cache(&frames, msecs)?;
        Ok((frames, duration))
    }

    pub fn into_parse(self) -> GifParseResult {
        if let Ok(result) = self.from_json() {
            return Ok(result);
        }

        self.use_ffmpeg()
    }
}

fn get_cache_dirname(
    filename: &Path,
    width: usize,
    height: usize,
    config: &Config,
) -> Result<PathBuf> {
    let hash_value = sha256_digest(filename)?;
    let mut dirname = std::env::temp_dir();
    dirname.push(format!("i2a-rs_{}", hash_value));
    dirname.push(format!(
        "{}x{}x{}x{}",
        width, height, config.grayscale, config.dup
    ));

    if !dirname.exists() {
        std::fs::create_dir_all(&dirname)?;
    }

    Ok(dirname)
}
