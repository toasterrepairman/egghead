use std::fs;
use base64::{encode};

pub fn encode_image_to_base64(file_path: &str) -> Result<String, std::io::Error> {
    // Read the image file as bytes
    let image_data = fs::read(file_path)?;

    // Encode the bytes to a base64 string
    let encoded = encode(&image_data);

    // Return the encoded string
    Ok(encoded)
}
