use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

pub fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;

    let prompt_input = format!("{}\n{}", init, prompt);
    let request_data = json!({
        "model": "ggml-gpt4all-j.bin",
        "prompt": prompt_input,
        "temperature": temp
    });

    let response = client
        .post("http://localhost:8080/v1/completions")
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()?;

    let response_json: serde_json::Value = response.json()?;
    let completion_text = response_json["choices"][0]["text"].as_str().unwrap_or("");
    println!("{}", &completion_text);

    Ok(completion_text.to_string())
}
