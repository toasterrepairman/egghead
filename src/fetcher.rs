use std::error::Error;
use rss::{Channel, Item};
use reqwest;
use rand::Rng;
use reqwest::header;

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
    let url = "https://hacker-news.firebaseio.com/v0/topstories.json";
    let top_stories = reqwest::get(url).await?.json::<Vec<i32>>().await?;

    let first_story_id = top_stories.first().unwrap();
    let story_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", first_story_id);
    let story = reqwest::get(&story_url).await?.json::<serde_json::Value>().await?;

    let kids = story.get("kids").unwrap().as_array().unwrap();
    let latest_comment_id = kids.last().unwrap().as_i64().unwrap();
    let comment_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", latest_comment_id);
    let comment = reqwest::get(&comment_url).await?.json::<serde_json::Value>().await?;

    let text = comment.get("text").unwrap().as_str().unwrap();
    let first_20_chars = text.chars().take(20).collect();

    Ok(first_20_chars)
}
