use serde::Deserialize;
use reqwest::Client;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use serde_with::skip_serializing_none;
use closestmatch::ClosestMatch;
use unicase::UniCase;
use unidecode::unidecode;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::{fuzzy_indices};
use fuzzy_matcher::skim::SkimMatcherV2;

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
    inference_job_token: Option<String>,
    inference_job_token_type: Option<String>,
    #[serde(default)]
    state: Option<InferenceJobState>,
}

#[derive(Deserialize)]
struct InferenceJobState {
    job_token: String,
    status: String,
    maybe_public_bucket_wav_audio_path: Option<String>,
}

pub async fn fuzzy_search_voices(query: String) -> String {
    let client = Client::new();

    let voice_list_url = "https://api.fakeyou.com/tts/list";
    let voices: VoiceListResponse = client.get(voice_list_url).send().await.unwrap().json().await.unwrap();


    let matcher = SkimMatcherV2::default();
    let mut matches = Vec::new();

    for voice in voices {
        if let Some((score, _)) = matcher.fuzzy_indices(&query, &voice.name) {
            matches.push((score, voice));
        }
    }

    // Sort the matches by their score
    matches.sort_by(|a, b| a.0.cmp(&b.0));

    // Create a newline-separated string of the voice names
    let result = matches
        .into_iter()
        .map(|(_, voice)| voice.name)
        .collect::<Vec<String>>()
        .join("\n");

    result
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
    println!("Got past the voice search! {}", &model_token);

    // Create the inference job
    let inference_url = "https://api.fakeyou.com/tts/inference";
    let idempotency_token = uuid::Uuid::new_v4().to_string();
    let job_payload = json!({
        "inference_text": message,
        "tts_model_token": model_token,
        "uuid_idempotency_token": idempotency_token,
    });
    println!("Debug payload: {}", job_payload);
    let job_response: InferenceJobResponse = client.post(inference_url)
        .json(&job_payload)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .send().await?
        .json().await?;
    println!("Got past the job creation");

    let job_token = job_response.inference_job_token;

    // Poll the API for the audio file URL
    let mut audio_url = None;
    while audio_url.is_none() {
        sleep(Duration::from_secs(5)).await; // Wait before polling
        let job_status_url = format!("https://api.fakeyou.com/tts/job/{}", job_token.as_ref().unwrap());
        println!("[ Sent another API job request check to {}] ", &job_status_url);
        let status_response: InferenceJobResponse = client
            .get(&job_status_url)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .send().await?
            .json().await?;
        if status_response.state.is_some() {
            if status_response.state.as_ref().unwrap().status == "complete_success" {
                audio_url = status_response.state.unwrap().maybe_public_bucket_wav_audio_path;
            } else {

            }
        } else {

        }
    }
    println!("It's over now!");

    Ok(format!("https://storage.googleapis.com/vocodes-public{}", audio_url.unwrap()))
}