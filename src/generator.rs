use reqwest::blocking::Client;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use base64::encode; // For encoding image data to base64
use std::fs::File;
use std::io::Read;
use serde_json::json;

pub fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(360))
        .build()?;

    // Assume you have an image file path, adjust as needed
    let imagedata_path = "path/to/your/image.jpg";

    let mut image_file = File::open(imagedata_path)?;
    let mut imagedata = String::new();
    image_file.read_to_string(&mut imagedata)?;

    // Encode the image data to base64
    let encoded_image_data = encode(imagedata.as_bytes());

    let prompt_input = format!("{}{}\n\n", init, prompt);
    let request_data = json!({
        "model": "ollama",
        "prompt": prompt_input,
        "temperature": temp.parse::<f64>().unwrap(),
        "stream": false,
        "stop": ["\n"],
        "images": [
            encoded_image_data
        ],
        "system_prompt": {
          "prompt": "You are Egghead, the world's smartest computer. Answer the following user query:",
          // Update any additional fields as needed for ollama
       },
    });

    let response = client
        .post("http://localhost:11434/api/generate")
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()?;

    let response_json: serde_json::Value = response.json()?;
    println!("{}", &response_json);
    let completion_text = response_json["response"].as_str().unwrap_or("Prompt machine broke");
    println!("{}", &completion_text);

    Ok(completion_text.to_string())
}
