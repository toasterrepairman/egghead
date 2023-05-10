use reqwest::Error;
use serde_json::Value;

pub async fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let url = "http://localhost:8080/v1/completions";
    let headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json");

    let response = reqwest::Client::new()
        .post(url)
        .headers(headers)
        .json(&json!({
            "model": "ggml-gpt4all-j.bin",
            "prompt": format!("{}\n{{\n", init, prompt),
            "temperature": temp.parse::<f64>().unwrap()
        }))
        .send()
        .await?;

    let response_json: Value = response.json::<Value>().await?;
    let text_completion = response_json["choices"].as_array().unwrap().first().unwrap();
    let response_text = text_completion["text"].as_str().unwrap().to_owned();

    Ok(response_text)
}
