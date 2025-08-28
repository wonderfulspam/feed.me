use feed_rs::parser;
use ureq::Agent;

/// Fetch a feed from URL with timeout and error handling  
pub fn fetch_feed(agent: &Agent, url: &str) -> Option<feed_rs::model::Feed> {
    let mut response = agent.get(url).call().ok()?;
    let content = response.body_mut().read_to_string().ok()?;
    parser::parse(content.as_bytes()).ok()
}
