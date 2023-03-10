use std::error::Error;
use rss::{Channel, Item};
use reqwest;
use rand::{random, Rng};
use reqwest::header;
use reqwest::Client;

pub async fn get_random_headline_from_rss_link(rss_link: &str) -> Result<String, Box<dyn Error>> {
    // Send an HTTP GET request to the RSS link using reqwest
    let body = reqwest::get(rss_link).await?.text().await?;

    // Parse the RSS feed using the rss crate
    let channel = rss::Channel::read_from(&body.as_bytes()[..])?;

    // Get a random index within the range of the number of items in the RSS feed
    let index = rand::thread_rng().gen_range(0..channel.items().len());

    // Extract the item at the random index
    let item = channel.items()[index].clone();

    // Return the title of the item as a string
    Ok(item.title().unwrap_or_default().to_string())
}

pub async fn get_wikipedia_summary(article: Option<&str>) -> Result<String, reqwest::Error> {
    let url = if let Some(article_name) = article {
        format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", article_name)
    } else {
        "https://en.wikipedia.org/api/rest_v1/page/random/summary".to_string()
    };

    let client = reqwest::Client::new();
    let mut headers = header::HeaderMap:: new();
    headers.insert(header::USER_AGENT, header::HeaderValue::from_static("reqwest"));
    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let title = response["title"].as_str().unwrap_or("Unknown").to_owned();
    let summary = response["extract"].as_str().unwrap_or("").chars().take(40).collect::<String>();

    Ok(format!("{}\n{}", title, summary))
}

pub async fn get_latest_hn_comment() -> Result<String, reqwest::Error> {
    let client = Client::new();
    let url = "http://hn.algolia.com/api/v1/search_by_date?tags=comment";
    let response = client.get(url).send().await?.json::<serde_json::Value>().await?;
    let index: usize = rand::thread_rng().gen_range(0..15);
    let latest_comment = response["hits"][index]["comment_text"].as_str().unwrap_or("");
    Ok(latest_comment.chars().take(30).collect())
}

pub async fn process_tts_job(job_token: &str) -> String {
    let mut audio_url = String::new();
    let mut job_status = String::new();

    let client = reqwest::Client::new();
    while audio_url.is_empty() {
        let job_status_response = client
            .get(&format!("https://api.fakeyou.com/tts/job/{}", job_token))
            .send()
            .await
            .unwrap();

        if job_status_response.status().is_success() {
            let job_status_json = job_status_response.json::<serde_json::Value>().await.unwrap();
            job_status = job_status_json["state"].as_str().unwrap().to_string();
            println!("Current job status: {}", job_status);

            if job_status == "completed" {
                audio_url = job_status_json["maybe_public_bucket_wav_audio_path"]
                    .as_str()
                    .unwrap()
                    .to_string();
            }
        } else {
            let status = job_status_response.status();
            let error_message = job_status_response.text().await.unwrap();
            panic!("Received error {}: {}", status, error_message);
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    let full_audio_url = format!("https://storage.googleapis.com/vocodes-public/{}", audio_url);
    println!("Final audio URL: {}", full_audio_url);
    full_audio_url
}
