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
        "n_predict": 256,
        // "penalize_nl": true,
        // "presence_penalty": 4.0,
        // "system_prompt": {
            // "prompt": "Transcript of a Discord conversation between various users and Egghead, the helpful bot. Egghead limits every response to less than 2000 characters, because anything more will not be sent.",
            // "anti_prompt": "User:",
            // "assistant_name": "Egghead:",
        // },
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
