use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

pub fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;

    let prompt_input = format!("{}{}\n\n", init, prompt);
    let request_data = json!({
        "model": "mistral-7b-v0.1.Q4_K_M.gguf",
        "prompt": prompt_input,
        "temperature": temp.parse::<f64>().unwrap(),
        "stream": false,
        "max_tokens": 128,
    });

    let response = client
        .post("http://localhost:8080/v1/completions")
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()?;

    let response_json: serde_json::Value = response.json()?;
    println!("{}", &response_json);
    let completion_text = response_json["choices"][0]["text"].as_str().unwrap_or("Prompt machine broke");
    println!("{}", &completion_text);

    Ok(completion_text.to_string())
}

pub fn get_smart_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;

    let prompt_input = format!("{}{}\n", init, prompt);
    let request_data = json!({
        "model": "wizardlm-13b-v1.2.ggmlv3.q4_0.bin",
        "prompt": prompt_input,
        "temperature": temp.parse::<f64>().unwrap(),
        // "stream": false,
        // "max_tokens": 64,
    });

    let response = client
        .post("http://localhost:8080/v1/completions")
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()?;

    let response_json: serde_json::Value = response.json()?;
    println!("{}", &response_json);
    let completion_text = response_json["choices"][0]["text"].as_str().unwrap_or("Prompt machine broke");
    println!("{}", &completion_text);

    Ok(completion_text.to_string())
}

pub fn get_code_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;

    let prompt_input = format!("{}{}\n", init, prompt);
    let request_data = json!({
        "model": "codellama-7b.ggmlv3.Q4_1.bin",
        "prompt": prompt_input,
        "temperature": temp.parse::<f64>().unwrap(),
        // "stream": false,
        // "max_tokens": 64,
    });

    let response = client
        .post("http://localhost:8080/v1/completions")
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()?;

    let response_json: serde_json::Value = response.json()?;
    println!("{}", &response_json);
    let completion_text = response_json["choices"][0]["text"].as_str().unwrap_or("Prompt machine broke");
    println!("{}", &completion_text);

    Ok(completion_text.to_string())
}
