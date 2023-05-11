use reqwest::Error;

pub async fn get_chat_response(temp: &str, init: &str, prompt: &str) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let url = "http://localhost:8080/v1/completions";
    let input = format!("{}{}\n{}", init, prompt, temp);

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(format!(
            r#"{{ "model": "ggml-gpt4all-j.bin", "prompt": "{}", "temperature": {} }}"#,
            input, temp
        ))
        .send()
        .await?;

    let body = res.text().await?;
    let json: serde_json::Value = serde_json::from_str(&body)?;

    let text = json["choices"][0]["text"].as_str().unwrap_or("");
    Ok(String::from(text))
}
