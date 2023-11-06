use rand::Rng;
use reqwest;
use std::error::Error;

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
        format!(
            "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
            article_name
        )
    } else {
        "https://en.wikipedia.org/api/rest_v1/page/random/summary".to_string()
    };

    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("reqwest"),
    );
    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let title = response["title"].as_str().unwrap_or("Unknown").to_owned();

    Ok(format!("{}", title))
}

pub async fn get_latest_hn_comment() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = "http://hn.algolia.com/api/v1/search_by_date?tags=comment";
    let response = client
        .get(url)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    let index: usize = rand::thread_rng().gen_range(0..15);
    let latest_comment = response["hits"][index]["comment_text"]
        .as_str()
        .unwrap_or("");
    Ok(latest_comment.chars().take(30).collect())
}
