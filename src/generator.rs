use std::error::Error;
use reqwest::blocking::{Client, Response};
use urlencoding::encode;
use std::time::Duration;

pub fn get_chat_response(prompt: &str) -> Result<String, reqwest::Error> {
    let base_url = "http://localhost:8008/api/chat";
    let model = "ggml-alpaca-7B-q4_0.bin";
    let temperature = "0.6";
    let top_k = "50";
    let top_p = "0.95";
    let max_length = "512";
    let context_window = "512";
    let repeat_last_n = "64";
    let repeat_penalty = "1.3";
    let init_prompt = encode("I am egghead, the world's smartest computer.");
    let n_threads = "2";

    // Get a chat UUID
    let params = format!("?model={}&temperature={}&top_k={}&top_p={}&max_length={}&context_window={}&repeat_last_n={}&repeat_penalty={}&init_prompt={}&n_threads={}",
                         model, temperature, top_k, top_p, max_length, context_window, repeat_last_n, repeat_penalty, init_prompt, n_threads);
    let url = format!("{}{}", base_url, params);
    let uuid = Client::new().post(&url).send()?.text()?.replace("\"", "");
    println!("{}", &uuid);

    // Make a POST request to api/chat/<UUID>/question with the prompt
    let question_url = format!("{}/{}/question?prompt={}", base_url, uuid, encode(prompt));
    println!("{}", &question_url);
    Client::new().post(&question_url).send()?;

    // Block for 60 seconds before returning the response
    std::thread::sleep(Duration::from_secs(60));

    let mut response = Client::new().get(format!("{}/{}", base_url, uuid)).send()?;
    let response_text = response.text()?;

    Ok(response_text)
}
