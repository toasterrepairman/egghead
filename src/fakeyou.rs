use reqwest::Client;
use serde_json::Value;
use std::error::Error;
use uuid::Uuid;
use serde_json::json;
use serde::Deserialize;
/*
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let voice_name = "Agent Pleakley (Lilo & Stitch, Kevin McDonald)";
    let message = "Hello, I am Agent Pleakley!";

    let audio_url = get_audio_url(voice_name, message).await?;
    println!("Audio URL: {}", audio_url);
    Ok(())
}
*/

#[derive(Debug, Deserialize)]
struct Voice {
    model_token: String,
    title: String,
}

#[derive(Deserialize)]
struct VoicesResponse {
    results: Vec<Voice>,
}

#[derive(Deserialize)]
struct JobResponse {
    success: bool,
    state: Option<JobState>,
}

#[derive(Deserialize)]
struct JobState {
    job_token: String,
    status: String,
    maybe_public_bucket_wav_audio_path: Option<String>,
}

pub async fn get_audio_url(voice_name: &str, message: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = Client::new();

    // Get the list of voices
    let voices_url = "https://api.fakeyou.com/tts/list";
    let response: Vec<Voice> = client.get(voices_url).send().await?.json().await?;

    // Find the voice with the requested name
    let voice = response
        .into_iter()
        .find(|v| v.title.to_lowercase() == voice_name.to_lowercase())
        .ok_or("Voice not found")?;

    // Create the inference request
    let inference_url = "https://api.fakeyou.com/tts/inference";
    let uuid_idempotency_token = Uuid::new_v4();
    let json_data = serde_json::json!({
        "uuid_idempotency_token": uuid_idempotency_token.to_string(),
        "tts_model_token": voice.model_token,
        "inference_text": message
    });
    let job_response: JobResponse = client
        .post(inference_url)
        .json(&json_data)
        .send()
        .await?
        .json()
        .await?;

    // Poll for the inference job completion
    let job_token = job_response.state.ok_or("Job state not found")?.job_token;
    let job_status_url = format!("https://api.fakeyou.com/tts/job/{}", job_token);
    loop {
        let job_response: JobResponse = client.get(&job_status_url).send().await?.json().await?;
        let state = job_response.state.ok_or("Job state not found")?;
        if state.status == "complete_success" {
            let audio_path = state.maybe_public_bucket_wav_audio_path.ok_or("Audio path not found")?;
            let audio_url = format!("https://api.fakeyou.com{}", audio_path);
            return Ok(audio_url);
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await; // Wait before polling again
    }
}