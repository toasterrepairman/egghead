use reqwest::blocking::Client;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn get_chat_response(temp: &str, init: &str, prompt: &str, images: Option<Vec<String>>) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(360))
        .build()?;

    let prompt_input = format!("{}{}\n\n", init, prompt);

    let mut request_data = json!({
        "model": "gemma3:270m",
        "prompt": format!("{}", prompt_input),
        // "temperature": temp.parse::<f64>().unwrap(),
        "stream": false,
        "stop": ["\n"],
    });

    // Add images array if provided
    if let Some(img_list) = images {
        if !img_list.is_empty() {
            request_data["images"] = json!(img_list);
            println!("Sending request with {} image(s)", img_list.len());
        }
    }

    let response = client
        .post("http://localhost:11434/api/generate")
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()?;

    let response_json: serde_json::Value = response.json().unwrap();
    let completion_text = response_json["response"].as_str().unwrap_or("Prompt machine broke");
    Ok(completion_text.to_string())
}
