use std::error::Error;
use reqwest::blocking::{Client, Response};
use urlencoding::encode;
use std::time::Duration;
use serde_json::{json, Value};

pub fn get_chat_response(prompt: &str) -> Result<String, reqwest::Error> {
    let base_url = "http://localhost:8008/api/chat";
    let model = "ggml-alpaca-7B-q4_0.bin";
    let temperature = "0.7";
    let top_k = "50";
    let top_p = "0.95";
    let max_length = "2048";
    let context_window = "1024";
    let repeat_last_n = "64";
    let repeat_penalty = "1.3";
    let init_prompt = encode("I am egghead, the world's smartest computer. I will write a response that appropriately completes the request. The response must be accurate, concise and evidence-based whenever possible. A complete answer is always ended by [end of text].");
    let n_threads = "3";

    // Get a chat UUID
    let params = format!("?model={}&temperature={}&top_k={}&top_p={}&max_length={}&context_window={}&repeat_last_n={}&repeat_penalty={}&init_prompt={}&n_threads={}",
                         model, temperature, top_k, top_p, max_length, context_window, repeat_last_n, repeat_penalty, init_prompt, n_threads);
    let url = format!("{}{}", base_url, params);
    let uuid = Client::new().post(&url).send()?.text()?.replace("\"", "");
    println!("{}", &uuid);

    // Make a POST request to api/chat/<UUID>/question with the prompt
    let question_url = format!("{}/{}/question?prompt={}", base_url, uuid, encode(prompt));
    println!("{}", &question_url);
    Client::new().post(&question_url).timeout(std::time::Duration::from_secs(75)).send()?;

    // Block for 60 seconds before returning the response
    std::thread::sleep(Duration::from_secs(50));

    let mut response = Client::new().get(format!("{}/{}", base_url, uuid)).send()?;
    let response_text = response.text()?;

    let answer = extract_answer(&response_text).unwrap();

    Ok(answer)
}

fn extract_answer(json_string: &str) -> Option<String> {
    if let Some(start_index) = json_string.find(r#"answer":"#) {
        let end_index = json_string[start_index..].find("\",\"")? + start_index;
        Some(json_string[(start_index + r#"answer":"#.len())..end_index].to_owned())
    } else {
        None
    }
}
