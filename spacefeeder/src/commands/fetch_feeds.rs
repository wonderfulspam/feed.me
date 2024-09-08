use std::io::BufReader;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use crate::config::{Config, ParseConfig};
use crate::FeedInfo;

use anyhow::Result;
use chrono::{DateTime, Utc};
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

pub fn run(config: Config) -> Result<()> {
    // A channel for transmitting the results of HTTP requests
    let (tx, rx) = channel();

    // Spin off background thread for parallel URL processing
    // TODO use async instead
    thread::spawn(move || {
        let agent: Agent = AgentBuilder::new()
            .timeout_read(Duration::from_secs(10))
            .build();
        config.feeds.par_iter().for_each(|(slug, feed_info)| {
            let slug = slug.clone();
            let feed_info = feed_info.clone();
            if let Some(feed) = fetch_feed(&agent, &feed_info.url) {
                println!("Fetched feed for {slug}");
                tx.send((feed, feed_info, slug)).unwrap();
            } else {
                eprintln!("Failed to load feed for {slug}");
            }
        });
    });

    let re = Regex::new(r"<[^>]*>").unwrap();

    let feed_data: Vec<_> = rx
        .into_iter()
        .map(|(feed, feed_info, slug)| {
            println!("Building feed for {slug}");
            build_feed(feed, feed_info, &config.parse_config, &re, slug)
        })
        .collect();

    write_data_to_file(&config.output_config.feed_data_output_path, &feed_data);

    let mut items: Vec<_> = feed_data.iter().flat_map(Vec::<ItemOutput>::from).collect();
    items.sort_unstable_by_key(|io| io.item.pub_date);
    items.reverse();
    write_data_to_file(&config.output_config.item_data_output_path, &items);

    println!(
        "Processed {} items from {} feeds",
        items.len(),
        feed_data.len()
    );
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
    let description = get_description_from_entry(entry, description_max_words);
    let safe_description = re.replace_all(&description, "").to_string();

    RssItem {
        title,
        item_url,
        description,
        safe_description,
        pub_date,
    }
}

fn get_description_from_entry(entry: Entry, max_words: usize) -> String {
    // Try in the following order
    // 1. Summary
    // 2. Content
    // 3. Media description
    // TODO Make this more robust
    let description = entry.summary.map_or_else(
        || {
            entry.content.map_or_else(
                || {
                    entry
                        .media
                        .first()
                        .unwrap()
                        .clone()
                        .description
                        .unwrap()
                        .content
                },
                |c| c.body.unwrap(),
            )
        },
        |s| s.content,
    );
    // If media, take the description from the first item
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
    ];

    #[test_case(TEST_DATA[0]; "Import youtube video feed")]
    #[test_case(TEST_DATA[1]; "Import atlassian feed")]
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
}
