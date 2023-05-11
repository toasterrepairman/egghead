use reqwest::Error;

pub async fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, Error> {
    let body = format!(
        r#"{{"model": "ggml-gpt4all-j.bin", "prompt": "{}\n{}", "temperature": {}}}"#,
        init, prompt, temp
    );

    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:8080/v1/completions")
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?;

    let text = res.text().await?;
    let json: serde_json::Value = serde_json::from_str(&text)?;

    let completion = json["choices"][0]["text"].as_str().unwrap();
    Ok(completion.to_owned())
}
