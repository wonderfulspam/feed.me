use std::time::Duration;

use anyhow::{anyhow, Result};
use clap::Args;
use ureq::Agent;
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

#[derive(Args)]
pub struct FindFeedArgs {
    #[arg(long)]
    pub base_url: String,
}

pub fn execute(args: FindFeedArgs) -> Result<()> {
    let url_match = run(&args.base_url)?;
    println!("{url_match}");
    Ok(())
}

pub fn run(base_url: &str) -> Result<String> {
    let base_url = Url::parse(base_url)?;
    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(3)))
        .build()
        .into();

    let rss_path = LIKELY_PATHS.iter().find_map(|&path| {
        let url_to_try = base_url
            .join(path)
            .expect("Already verified URL combined with known good pattern");
        let url_str = url_to_try.as_str();
        println!("Trying {url_str}");
        if let Ok(res) = agent.head(url_str).call() {
            let content_type = res
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok());
            if is_feed_content_type(content_type) {
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
