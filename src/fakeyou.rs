use reqwest::Client;
use serde_json::Value;

pub async fn get_audio_url(voice_name: &str, message: &str) -> Result<String, reqwest::Error> {
    let client = Client::new();

    // Search for the specified voice
    let list_response = client
        .get("https://api.fakeyou.com/tts/list")
        .send()
        .await?
        .json::<Value>()
        .await?;

    let models = list_response["models"].as_array().unwrap();
    let model_token = models
        .iter()
        .find(|model| model["title"].as_str().unwrap().contains(voice_name))
        .map(|model| model["model_token"].as_str().unwrap())
        .ok_or_else(|| reqwest::Error::new(reqwest::StatusCode::NOT_FOUND, "Voice not found"))?;

    // Make the inference request
    let uuid_idempotency_token = uuid::Uuid::new_v4().to_string();
    let inference_request_body = json!({
        "uuid_idempotency_token": uuid_idempotency_token,
        "tts_model_token": model_token,
        "inference_text": message
    });

    let inference_response = client
        .post("https://api.fakeyou.com/tts/inference")
        .json(&inference_request_body)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let inference_job_token = inference_response["job_token"].as_str().unwrap();

    // Poll the API until the job is complete
    let mut audio_url = None;
    while audio_url.is_none() {
        let job_response = client
            .get(&format!(
                "https://api.fakeyou.com/tts/job/{}",
                inference_job_token
            ))
            .send()
            .await?
            .json::<Value>()
            .await?;

        if job_response["state"]["status"].as_str().unwrap() == "complete_success" {
            let public_bucket_wav_audio_path = job_response["state"]["maybe_public_bucket_wav_audio_path"].as_str().unwrap();
            audio_url = Some(format!("https://api.fakeyou.com{}", public_bucket_wav_audio_path));
        } else {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    Ok(audio_url.unwrap())
}
