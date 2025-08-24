use std::io::BufReader;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use crate::config::{Config, ParseConfig};
use crate::FeedInfo;

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Args;
use feed_rs::model::Entry;
use feed_rs::parser;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use serde::Serialize;
use ureq::{Agent, AgentBuilder};
#[derive(Clone, Debug, Serialize)]

struct FeedOutput {
    #[serde(flatten)]
    meta: FeedInfo,
    slug: String,
    items: Vec<RssItem>,
}

#[derive(Clone, Debug, Serialize)]
struct ItemOutput {
    #[serde(flatten)]
    meta: FeedInfo,
    slug: String,
    #[serde(flatten)]
    item: RssItem,
}

#[derive(Clone, Debug, Serialize)]
struct RssItem {
    title: String,
    item_url: String,
    description: String,
    safe_description: String,
    pub_date: Option<DateTime<Utc>>,
}

#[derive(Args)]
pub struct FetchArgs {
    /// Path to the config file
    #[arg(long, default_value = "./spacefeeder.toml")]
    pub config_path: String,
}

pub fn execute(args: FetchArgs) -> Result<()> {
    let config = Config::from_file(&args.config_path)?;
    run(config)
}

pub fn run(config: Config) -> Result<()> {
    // A channel for transmitting the results of HTTP requests
    let (tx, rx) = channel();

    // Track total feeds and failures for reporting
    let total_feeds = config.feeds.len();
    let mut failed_feeds = Vec::new();

    // Spin off background thread for parallel URL processing
    // TODO use async instead
    thread::spawn(move || {
        let agent: Agent = AgentBuilder::new()
            .timeout_read(Duration::from_secs(10))
            .build();
        config.feeds.par_iter().for_each(|(slug, feed_info)| {
            let slug = slug.clone();
            let feed_info = feed_info.clone();
            match fetch_feed(&agent, &feed_info.url) {
                Some(feed) => {
                    println!("✓ Fetched feed for {slug}");
                    tx.send(Ok((feed, feed_info, slug))).unwrap();
                }
                None => {
                    eprintln!("✗ Failed to load feed for {slug} from {}", feed_info.url);
                    tx.send(Err(slug)).unwrap();
                }
            }
        });
    });

    let re = Regex::new(r"<[^>]*>").unwrap();

    // Collect results, separating successes from failures
    let feed_data: Vec<_> = rx
        .into_iter()
        .filter_map(|result| match result {
            Ok((feed, feed_info, slug)) => {
                println!("Building feed for {slug}");
                Some(build_feed(feed, feed_info, &config.parse_config, &re, slug))
            }
            Err(slug) => {
                failed_feeds.push(slug);
                None
            }
        })
        .collect();

    // Report failures if any
    if !failed_feeds.is_empty() {
        eprintln!(
            "\n⚠ Warning: Failed to fetch {} out of {} feeds:",
            failed_feeds.len(),
            total_feeds
        );
        for slug in &failed_feeds {
            eprintln!("  - {slug}");
        }
        eprintln!();
    }

    write_data_to_file(&config.output_config.feed_data_output_path, &feed_data);

    let mut items: Vec<_> = feed_data.iter().flat_map(Vec::<ItemOutput>::from).collect();
    items.sort_unstable_by_key(|io| io.item.pub_date);
    items.reverse();
    
    // Write all items
    write_data_to_file(&config.output_config.item_data_output_path, &items);
    
    // Write filtered items by tier for better performance in templates
    let loved_items: Vec<_> = items.iter()
        .filter(|item| matches!(item.meta.tier, crate::Tier::Love))
        .cloned()
        .collect();
    write_data_to_file("./content/data/lovedData.json", &loved_items);
    
    let liked_items: Vec<_> = items.iter()
        .filter(|item| matches!(item.meta.tier, crate::Tier::Like))
        .cloned()
        .collect();
    write_data_to_file("./content/data/likedData.json", &liked_items);
    
    let new_items: Vec<_> = items.iter()
        .filter(|item| matches!(item.meta.tier, crate::Tier::New))
        .cloned()
        .collect();
    write_data_to_file("./content/data/newData.json", &new_items);

    println!(
        "\n✓ Successfully processed {} items from {} feeds ({}% success rate)",
        items.len(),
        feed_data.len(),
        (feed_data.len() * 100) / total_feeds.max(1)
    );
    
    // Return Ok even if some feeds failed - the operation as a whole succeeded
    Ok(())
}

impl From<&FeedOutput> for Vec<ItemOutput> {
    fn from(feed: &FeedOutput) -> Self {
        feed.items
            .iter()
            .map(move |item| ItemOutput {
                meta: feed.meta.clone(),
                slug: feed.slug.clone(),
                item: item.clone(),
            })
            .collect::<Vec<_>>()
    }
}
fn write_data_to_file<D: Serialize>(output_path: &str, data: &D) {
    let contents = serde_json::to_string_pretty(data).unwrap();
    std::fs::write(output_path, contents).expect("Unable to write file");
}

fn fetch_feed(agent: &Agent, url: &str) -> Option<feed_rs::model::Feed> {
    let response = agent.get(url).call().ok()?;
    let reader = BufReader::new(response.into_reader());
    parser::parse(reader).ok()
}
fn build_feed(
    feed: feed_rs::model::Feed,
    feed_info: FeedInfo,
    parse_config: &ParseConfig,
    re: &Regex,
    slug: String,
) -> FeedOutput {
    let items = feed
        .entries
        .into_iter()
        .take(parse_config.max_articles)
        .map(|entry| build_item(entry, re, parse_config.description_max_words))
        .collect();
    FeedOutput {
        meta: feed_info,
        slug,
        items,
    }
}

fn build_item(entry: feed_rs::model::Entry, re: &Regex, description_max_words: usize) -> RssItem {
    let title = entry.title.clone().map(|t| t.content).unwrap_or_default();
    let item_url = entry
        .links
        .first()
        .map_or(String::new(), |link| link.href.clone());
    let pub_date = entry.published.or(entry.updated);
    let description = get_description_from_entry(entry).unwrap_or_default();
    let description = get_short_description(description, description_max_words);
    let safe_description = re.replace_all(&description, "").to_string();

    RssItem {
        title,
        item_url,
        description,
        safe_description,
        pub_date,
    }
}

fn get_description_from_entry(entry: Entry) -> Option<String> {
    // Try in the following order
    // 1. Summary
    // 2. Content
    // 3. Media description
    if let Some(summary) = entry.summary {
        return Some(summary.content);
    }
    if let Some(content) = entry.content {
        return content.body;
    }
    if let Some(media) = entry.media.first() {
        if let Some(description) = &media.description {
            return Some(description.content.clone());
        }
    }
    None
}

fn get_short_description(description: String, max_words: usize) -> String {
    description
        .split_whitespace()
        .take(max_words)
        .collect::<Vec<_>>()
        .join(" ")
}
#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const TEST_DATA: &[&str] = &[
        include_str!("../test_data/youtube.xml"),
        include_str!("../test_data/atlassian.xml"),
        include_str!("../test_data/xeiaso.rss"),
    ];

    #[test_case(TEST_DATA[0]; "Import youtube video feed")]
    #[test_case(TEST_DATA[1]; "Import atlassian feed")]
    #[test_case(TEST_DATA[2]; "Import Xe Iaso feed")]
    fn test_feed(feed_xml: &str) {
        let feed = parser::parse(feed_xml.as_bytes());
        assert!(feed.is_ok(), "Feed parsed correctly");
        let feed = feed.unwrap();

        let re = Regex::new(r"<[^>]*>").unwrap();
        let config = Config::default();
        let (slug, feed_info) = config.feeds.into_iter().next().unwrap();
        let feed_data = build_feed(feed, feed_info, &config.parse_config, &re, slug);
        let items: Vec<ItemOutput> = (&feed_data).into();
        assert_eq!(items.len(), config.parse_config.max_articles);
    }

    #[test]
    fn test_get_short_description_exact_words() {
        let description = "This is a test description with exactly ten words here.".to_string();
        let result = get_short_description(description.clone(), 10);
        assert_eq!(result, "This is a test description with exactly ten words here.");
    }

    #[test]
    fn test_get_short_description_truncates() {
        let description = "This is a very long description that should be truncated after exactly five words but continues on and on.".to_string();
        let result = get_short_description(description, 5);
        assert_eq!(result, "This is a very long");
    }

    #[test]
    fn test_get_short_description_empty() {
        let description = "".to_string();
        let result = get_short_description(description, 10);
        assert_eq!(result, "");
    }

    #[test]
    fn test_get_short_description_whitespace() {
        let description = "   Multiple   spaces    between     words   ".to_string();
        let result = get_short_description(description, 3);
        assert_eq!(result, "Multiple spaces between");
    }

    #[test]
    fn test_get_short_description_fewer_words() {
        let description = "Short text".to_string();
        let result = get_short_description(description, 10);
        assert_eq!(result, "Short text");
    }

    #[test]
    fn test_html_tag_removal() {
        let re = Regex::new(r"<[^>]*>").unwrap();
        let html = "<p>This is <strong>bold</strong> and <em>italic</em> text.</p>";
        let result = re.replace_all(html, "").to_string();
        assert_eq!(result, "This is bold and italic text.");
    }

    #[test]
    fn test_html_tag_removal_nested() {
        let re = Regex::new(r"<[^>]*>").unwrap();
        let html = "<div><p>Nested <span>tags</span> here</p></div>";
        let result = re.replace_all(html, "").to_string();
        assert_eq!(result, "Nested tags here");
    }

    #[test]
    fn test_feed_output_to_item_output_conversion() {
        let feed_output = FeedOutput {
            meta: FeedInfo {
                url: "https://example.com/feed".to_string(),
                author: "Test Author".to_string(),
                tier: crate::Tier::New,
            },
            slug: "test_feed".to_string(),
            items: vec![
                RssItem {
                    title: "Article 1".to_string(),
                    item_url: "https://example.com/1".to_string(),
                    description: "Description 1".to_string(),
                    safe_description: "Description 1".to_string(),
                    pub_date: None,
                },
                RssItem {
                    title: "Article 2".to_string(),
                    item_url: "https://example.com/2".to_string(),
                    description: "Description 2".to_string(),
                    safe_description: "Description 2".to_string(),
                    pub_date: None,
                },
            ],
        };

        let items: Vec<ItemOutput> = (&feed_output).into();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].item.title, "Article 1");
        assert_eq!(items[1].item.title, "Article 2");
        assert_eq!(items[0].slug, "test_feed");
        assert_eq!(items[0].meta.author, "Test Author");
    }
}
