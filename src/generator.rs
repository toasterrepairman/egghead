use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref CLIENT: Mutex<Client> = Mutex::new(Client::new());
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    prompt: String,
    temperature: f32,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    text: String,
}

pub fn get_chat_response(
    temp: &str,
    init: &str,
    prompt: &str,
) -> Result<String, reqwest::Error> {
    let temperature = temp.parse::<f32>().unwrap_or(0.7);
    let request = ChatRequest {
        model: init.to_string(),
        prompt: prompt.to_string(),
        temperature,
    };

    let client = CLIENT.lock().unwrap();
    let response = client
        .post("http://localhost:8080/v1/completions")
        .json(&request)
        .send()
        .and_then(|resp| resp.json::<ChatResponse>());

    match response {
        Ok(chat_response) => {
            if let Some(choice) = chat_response.choices.into_iter().next() {
                Ok(choice.text)
            } else {
                Err(reqwest::Error::new(
                    reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                    "No choices returned",
                ))
            }
        }
        Err(error) => Err(error),
    }
}
