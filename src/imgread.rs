use std::fs::File;
use std::io::prelude::*;
use base64;

fn base64_encode_image_file(file_path: &str) -> Result<String, std::io::Error> {
    let mut f = File::open(file_path)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    let encoded = base64::encode(contents.as_bytes());
    Ok(encoded)
}
