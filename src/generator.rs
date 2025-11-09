use reqwest::blocking::Client;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn get_chat_response(temp: &str, init: &str, prompt: &str, images: Option<Vec<String>>, conversation_history: Option<Vec<serde_json::Value>>) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(360))
        .build()?;

    // Build messages array with system message
    let mut messages = vec![
        json!({
            "role": "system",
            "content": init
        })
    ];

    // Add conversation history if provided
    if let Some(history) = conversation_history {
        messages.extend(history);
    }

    // Build user message content
    let user_message = if let Some(img_list) = images {
        if !img_list.is_empty() {
            // OpenAI vision format: content is an array of text and image_url objects
            let mut content_parts = vec![
                json!({
                    "type": "text",
                    "text": prompt
                })
            ];

            for img_base64 in img_list {
                content_parts.push(json!({
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:image/jpeg;base64,{}", img_base64)
                    }
                }));
            }

            println!("Sending request with {} image(s)", content_parts.len() - 1);

            json!({
                "role": "user",
                "content": content_parts
            })
        } else {
            json!({
                "role": "user",
                "content": prompt
            })
        }
    } else {
        json!({
            "role": "user",
            "content": prompt
        })
    };

    messages.push(user_message);

    let request_data = json!({
        "model": "riven/smolvlm",
        "messages": messages,
        // "temperature": temp.parse::<f64>().unwrap(),
        "stream": false,
    });

    let response = client
        .post("http://localhost:11434/v1/chat/completions")
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()?;

    let status = response.status();
    println!("Response status: {}", status);

    let response_json: serde_json::Value = response.json().unwrap();
    println!("Response JSON: {:?}", response_json);

    // Check for error in response
    if let Some(error) = response_json.get("error") {
        eprintln!("OpenAI API error: {}", error);
        return Ok(format!("Error from API: {}", error));
    }

    // Extract response from OpenAI format: choices[0].message.content
    let completion_text = response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_else(|| {
            eprintln!("No 'choices[0].message.content' field in JSON: {:?}", response_json);
            "Prompt machine broke - no response field"
        });

    Ok(completion_text.to_string())
}
