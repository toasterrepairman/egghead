use reqwest::blocking::Client;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(360))
        .build()?;

    let prompt_input = format!("{}{}\n\n", init, prompt);
    let request_data = json!({
        "prompt": prompt_input,
        "temperature": temp.parse::<f64>().unwrap(),
        "stream": false,
        // "n_predict": 325,
        // "penalize_nl": true,
        // "presence_penalty": 4.0,
        // "batch_size": 512,
        // "image_data": [{"data": Some(imagedata), "id": 42}],
        "system_prompt": {
         "prompt": "You are Egghead, the world's smartest computer.",
         // "anti_prompt": "User: React and respond to any images or messages in this channel.",
         // "assistant_name": "Egghead:",
        },
    });

    let response = client
        .post("http://localhost:8080/completion")
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()?;

    let response_json: serde_json::Value = response.json()?;
    println!("{}", &response_json);
    let completion_text = response_json["content"].as_str().unwrap_or("Prompt machine broke");
    println!("{}", &completion_text);

    Ok(completion_text.to_string())
}
