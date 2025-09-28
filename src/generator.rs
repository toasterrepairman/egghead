use reqwest::blocking::Client;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn get_chat_response(temp: &str, init: &str, prompt: &str, imagedata: Option<&str>) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(360))
        .build()?;

    let image_data = match imagedata {
        Some(data) => json!({ "data": data }),
        None => json!({ "data": "" }),
    };

    println!("{:?}", image_data);

    let prompt_input = format!("{}{}\n\n", init, prompt);
    let request_data = json!({
        "model": "huihui_ai/gemma3-abliterated:270m",
        "prompt": format!("{}", prompt_input),
        // "temperature": temp.parse::<f64>().unwrap(),
        "stream": false,
        "stop": ["\n"],
        "image_data": image_data,
    });

    let response = client
        .post("http://localhost:11434/api/generate")
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()?;

    let response_json: serde_json::Value = response.json().unwrap();
    let completion_text = response_json["response"].as_str().unwrap_or("Prompt machine broke");
    Ok(completion_text.to_string())
}
