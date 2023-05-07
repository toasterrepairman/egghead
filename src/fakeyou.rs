use serde::Deserialize;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Deserialize)]
struct VoiceListResponse {
    success: bool,
    models: Vec<VoiceModel>,
}

#[derive(Deserialize)]
struct VoiceModel {
    model_token: String,
    title: String,
}

#[derive(Deserialize)]
struct InferenceJobResponse {
    success: bool,
    state: InferenceJobState,
}

#[derive(Deserialize)]
struct InferenceJobState {
    job_token: String,
    status: String,
    maybe_public_bucket_wav_audio_path: Option<String>,
}

pub async fn get_audio_url(voice_name: &str, message: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();

    // Get the voice list
    let voice_list_url = "https://api.fakeyou.com/tts/list";
    let response: VoiceListResponse = client.get(voice_list_url).send().await?.json().await?;

    // Find the model_token for the specified voice_name
    let model_token = response.models.iter()
        .find(|model| model.title == voice_name)
        .ok_or("Voice not found")?
        .model_token
        .clone();

    // Create the inference job
    let inference_url = "https://api.fakeyou.com/tts/inference";
    let idempotency_token = uuid::Uuid::new_v4().to_string();
    let job_payload = json!({
        "uuid_idempotency_token": idempotency_token,
        "tts_model_token": model_token,
        "inference_text": message
    });
    let job_response: InferenceJobResponse = client.post(inference_url)
        .json(&job_payload)
        .send()
        .await?
        .json()
        .await?;

    let job_token = job_response.state.job_token;

    // Poll the API for the audio file URL
    let mut audio_url = None;
    while audio_url.is_none() {
        sleep(Duration::from_secs(1)).await; // Wait before polling
        let job_status_url = format!("https://api.fakeyou.com/tts/job/{}", job_token);
        let status_response: InferenceJobResponse = client.get(&job_status_url).send().await?.json().await?;
        if status_response.state.status == "complete_success" {
            audio_url = status_response.state.maybe_public_bucket_wav_audio_path;
        }
    }

    Ok(format!("https://api.fakeyou.com{}", audio_url.unwrap()))
}