use std::time::Duration;

use anyhow::{anyhow, Result};
use ureq::AgentBuilder;
use url::Url;

const LIKELY_PATHS: &[&str] = &[
    "",
    "/feed",
    "/rss",
    "feed.xml",
    "rss.xml",
    "atom.xml",
    "index.xml",
    "blog.rss",
    ".atom",
];

pub fn run(base_url: &str) -> Result<String> {
    let base_url = Url::parse(base_url)?;
    let agent = AgentBuilder::new()
        .timeout_read(Duration::from_secs(3))
        .build();

    let rss_path = LIKELY_PATHS.iter().find_map(|&path| {
        let url_to_try = base_url
            .join(path)
            .expect("Already verified URL combined with known good pattern");
        let url_str = url_to_try.as_str();
        println!("Trying {url_str}");
        if let Ok(res) = agent.head(url_str).call() {
            if is_feed_content_type(res.header("content-type")) {
                return Some(url_to_try.to_string());
            }
        }
        None
    });
    rss_path.ok_or(anyhow!("Did not find a suitable feed URL"))
}

fn is_feed_content_type(content_type_header: Option<&str>) -> bool {
    if let Some(content_type) = content_type_header {
        let feed_content_types = [
            "application/rss+xml",
            "application/atom+xml",
            "application/xml",
            "text/xml",
        ];

        feed_content_types
            .iter()
            .any(|&feed_type| content_type.starts_with(feed_type))
    } else {
        false
    }
}
