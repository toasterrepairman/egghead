use reqwest::blocking::Client;

pub fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let client = Client::new();

    let payload = format!(
        r#"{{"model": "ggml-gpt4all-j.bin", "prompt": "{}\n{}\n", "temperature": {}}}"#,
        init, prompt, temp
    );

    let response = client
        .post("http://localhost:8080/v1/completions")
        .header("Content-Type", "application/json")
        .body(payload)
        .send()?
        .text()?;

    Ok(response)
}
