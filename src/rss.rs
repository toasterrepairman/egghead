use std::error::Error;
use rss::Channel;

async fn example_feed() -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get("http://example.com/feed.xml")
        .await?
        .bytes()
        .await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}