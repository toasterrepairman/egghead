use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct RequestBody<'a> {
    model: &'a str,
    prompt: &'a str,
    temperature: f64,
}

#[derive(Deserialize)]
struct ResponseBody {
    object: String,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Deserialize)]
struct Choice {
    text: String,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

pub fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let body = RequestBody {
        model: "ggml-gpt4all-j.bin",
        prompt: &format!("{}{}\n{}", init, prompt, temp),
        temperature: 0.7,
    };
    let response = client
        .post("http://localhost:8080/v1/completions")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()?
        .json::<ResponseBody>()?;
    println!("{:?}", response.choices[0].text.clone());
    Ok(response.choices[0].text.clone())
}
