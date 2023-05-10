use std::io::Read;
use reqwest::blocking::Client;

pub fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let prompt = format!("{}{}\n", init, prompt);

    let client = Client::new();
    let res = client.post("http://localhost:8080/v1/completions")
        .header("Content-Type", "application/json")
        .body(format!(r#"{{"model": "ggml-gpt4all-j.bin", "prompt": "{}", "temperature": {}}}"#, prompt, temp))
        .send()?
        .text()?;

    let json: serde_json::Value = serde_json::from_str(&res)?;
    let text = json["choices"][0]["text"].as_str().unwrap_or_default();

    Ok(String::from(text))
}
