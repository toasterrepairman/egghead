use reqwest::blocking::Client;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ChatResponse {
    content: String,
    probs: Vec<Probability>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Probability {
    prob: f64,
    tok_str: String,
}

pub fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    // Prepare the request data
    let request_data = serde_json::json!({
        "prompt": prompt,
        "n_predict": 128,
    });

    // Create a reqwest client and send the POST request
    let client = Client::new();
    let response = client
        .post("http://localhost:8080/completion")
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()?;

    // Check if the request was successful
    response.error_for_status()?;

    // Parse the JSON response
    let chat_response: ChatResponse = response.json()?;

    // Return the content field from the response
    Ok(chat_response.content)
}

pub fn get_short_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(180))
        .build()?;

    let prompt_input = format!("{}{}\n\n", init, prompt);
    let request_data = json!({
        "model": "Wizard-Vicuna-7B-Uncensored.ggmlv3.q4_1.bin",
        "prompt": prompt_input,
        "temperature": temp.parse::<f64>().unwrap(),
        "stream": false,
        "max_tokens": 64,
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
        "model": "phi-2.Q3_K_S.gguf",
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
