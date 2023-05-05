use reqwest::Client;
use serde_json::Value;
use std::error::Error;
use uuid::Uuid;

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

pub async fn get_audio_url(voice_name: &str, message: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();

    // Search for the specified voice from the unified list
    let voice_list = client
        .get("https://api.fakeyou.com/tts/list")
        .send()
        .await?
        .json::<Value>()
        .await?;

    let model_token = voice_list
        .as_array()
        .unwrap()
        .iter()
        .find(|voice| voice["title"].as_str().unwrap() == voice_name)
        .map(|voice| voice["model_token"].as_str().unwrap().to_string())
        .ok_or("Voice not found")?;

    // Pass the model_token to the inferencing call
    let uuid_idempotency_token = Uuid::new_v4().to_string();
    let data = json!({
        "uuid_idempotency_token": uuid_idempotency_token,
        "tts_model_token": model_token,
        "inference_text": message,
    });

    let inference_job = client
        .post("https://api.fakeyou.com/tts/inference")
        .json(&data)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let inference_job_token = inference_job["job_token"].as_str().unwrap();

    let mut audio_url = String::new();
    let mut success = false;

    while !success {
        let job_result = client
            .get(&format!(
                "https://api.fakeyou.com/tts/job/{}",
                inference_job_token
            ))
            .send()
            .await?
            .json::<Value>()
            .await?;

        success = job_result["success"].as_bool().unwrap();

        if success {
            audio_url = format!(
                "https://api.fakeyou.com{}",
                job_result["state"]["maybe_public_bucket_wav_audio_path"]
                    .as_str()
                    .unwrap()
            );
        }
    }

    Ok(audio_url)
}
