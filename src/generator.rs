use reqwest::Error;
use serde_json::Value;
use std::error::Error as StdError;

pub async fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, Error> {
    let response = reqwest::Client::new()
        .post("http://localhost:8080/v1/completions")
        .bearer_auth(None)
        .json(&serde_json::json!({
            "model": "ggml-gpt4all-j.bin",
            "prompt": prompt,
            "temperature": temp.parse::<f64>().unwrap()
        }))
        .send()
        .await?;

    if response.status().is_success() {
        let json: Value = response.json::<Value>().await?;
        let text_completion = json["choices"][0]["text"].as_str().unwrap();
        Ok(text_completion.to_owned())
    } else {
        Err(Error::from(response))
    }
}
