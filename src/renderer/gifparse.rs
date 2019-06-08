
use std::io::Result;
use std::path::Path;
extern crate data_encoding;
extern crate ring;

use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};

fn sha256_digest<R: std::io::Read>(mut reader: R) -> Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

pub fn gif_to_frames(filename: &Path) -> Result<Vec<image::DynamicImage>> {
    let input = std::fs::File::open(filename)?;
    let hash_value = HEXUPPER.encode(sha256_digest(input)?.as_ref());
    let mut dirname = std::env::temp_dir();
    dirname.push(format!("i2a-rs_{}", hash_value));
    std::fs::create_dir_all(dirname.to_str().unwrap())?;

    let output = std::process::Command::new("ffmpeg")
        .args(&[
            "-i",
            filename.to_str().unwrap(),
            format!("{}/%04d.jpg", dirname.to_str().unwrap()).as_str(),
        ])
        .output()
        .expect("Error while executing `ffmpeg`");
    if !output.status.success() {
        panic!("`ffmpeg` did not exit successfully.");
    }

    let mut result = Vec::new();

    for i in 1.. {
        let filename = dirname.join(format!("{:04}.jpg", i));
        if !filename.exists() {
            break;
        }
        result.push(image::open(filename).unwrap_or_else(|_| panic!("Cannot open temporary file!")))
    }

    Ok(result)
}