use reqwest::Error;
use serde_json::Value;

pub async fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let api_url = "http://localhost:8080/v1/completions";
    let response = reqwest::Client::new()
        .post(api_url)
        .json(&serde_json::json!({
            "model": "ggml-gpt4all-j.bin",
            "prompt": prompt,
            "temperature": temp.parse::<f64>().unwrap()
        }))
        .send()
        .await?;

    let response_json: Value = response.json::<Value>().await?;
    let text_completion = response_json["choices"][0]["text"].as_str().unwrap();

    Ok(text_completion.to_string())
}
