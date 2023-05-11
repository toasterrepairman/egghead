use reqwest::blocking::Client;
use std::time::Duration;

pub fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let url = "http://localhost:8080/v1/completions";
    let input = format!("{}{}\n{}", init, prompt, temp);
    let client = Client::new();
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(300))
        .body(format!(
            r#"{{"model":"ggml-gpt4all-j.bin","prompt":"{}","temperature":{}}}"#,
            input, temp
        ))
        .send()?
        .json::<serde_json::Value>()?;
    let text = response["choices"][0]["text"].as_str().unwrap();
    Ok(text.to_owned())
}
