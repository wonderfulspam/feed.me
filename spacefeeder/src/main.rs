use std::io::BufReader;
use std::sync::mpsc::channel;
use std::{collections::HashMap, time::Duration};
use std::{fs, thread};

use chrono::{DateTime, Utc};
use clap::Parser;
use feed_rs::model::Entry;
use feed_rs::parser;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use serde::{Deserialize, Serialize};
use ureq::{Agent, AgentBuilder};

const DESCRIPTION_MAX_WORDS: usize = 150;

#[derive(Parser)]
#[command(name = "Space Feeder", about = "Processes RSS and Atom feeds")]
struct Opt {
    /// Path to the config file
    #[arg(long, default_value = "./spacefeeder.toml")]
    config_path: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    max_articles: usize,
    feeds: HashMap<String, FeedInfo>,
    #[serde(default = "default_feed_data_output_path")]
    feed_data_output_path: String,
    #[serde(default = "default_item_data_output_path")]
    item_data_output_path: String,
}

fn default_feed_data_output_path() -> String {
    "./content/data/feedData.json".to_string()
}

fn default_item_data_output_path() -> String {
    "./content/data/itemData.json".to_string()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct FeedInfo {
    url: String,
    author: String,
    tier: Tier,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum Tier {
    New,
    Like,
    Love,
}

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

fn main() {
    let opt = Opt::parse();
    let config: Config = toml::from_str(&fs::read_to_string(&opt.config_path).unwrap()).unwrap();
    run(config);
}

fn run(config: Config) {
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
            build_feed(feed, feed_info, config.max_articles, &re, slug).unwrap()
        })
        .collect();

    write_data_to_file(&config.feed_data_output_path, &feed_data);

    let mut items: Vec<_> = feed_data.iter().flat_map(Vec::<ItemOutput>::from).collect();
    items.sort_unstable_by_key(|io| io.item.pub_date);
    items.reverse();
    write_data_to_file(&config.item_data_output_path, &items);

    println!(
        "Processed {} items from {} feeds",
        items.len(),
        feed_data.len()
    );
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
    max_articles: usize,
    re: &Regex,
    slug: String,
) -> Option<FeedOutput> {
    let items = feed
        .entries
        .into_iter()
        .take(max_articles)
        .map(|entry| build_item(entry, re))
        .collect();
    Some(FeedOutput {
        meta: feed_info,
        slug,
        items,
    })
}

fn build_item(entry: feed_rs::model::Entry, re: &Regex) -> RssItem {
    let title = entry.title.clone().map(|t| t.content).unwrap_or_default();
    let item_url = entry
        .links
        .first()
        .map_or(String::new(), |link| link.href.clone());
    let pub_date = entry.published.or(entry.updated);
    let description = get_description_from_entry(entry);
    let safe_description = re.replace_all(&description, "").to_string();

    RssItem {
        title,
        item_url,
        description,
        safe_description,
        pub_date,
    }
}

fn get_description_from_entry(entry: Entry) -> String {
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
        .take(DESCRIPTION_MAX_WORDS)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_default_config() -> (FeedInfo, usize, String) {
        let feed_info = FeedInfo {
            url: "www.example.com".to_string(),
            author: "Example Author".to_string(),
            tier: Tier::New,
        };
        let max_articles = 5;
        let slug = "example".to_string();
        (feed_info, max_articles, slug)
    }

    #[test]
    fn test_youtube_feed() {
        let feed_xml = include_str!("test_data/youtube.xml");
        let feed = parser::parse(feed_xml.as_bytes());
        assert!(feed.is_ok(), "Feed parsed correctly");
        let feed = feed.unwrap();
        let first = feed.entries.first().unwrap();
        dbg!(first);

        let re = Regex::new(r"<[^>]*>").unwrap();
        let (feed_info, max_articles, slug) = get_default_config();
        let feed_data = build_feed(feed, feed_info, max_articles, &re, slug);
        assert!(feed_data.is_some(), "Managed to build feed data");
        let feed_data = feed_data.unwrap();

        let items: Vec<ItemOutput> = (&feed_data).into();
        dbg!(items);
    }

    #[test]
    fn test_atlassian_feed() {
        let feed_xml = include_str!("test_data/atlassian.xml");
        let feed = parser::parse(feed_xml.as_bytes());
        assert!(feed.is_ok(), "Feed parsed correctly");
        let feed = feed.unwrap();
        let third = feed.entries.get(2).unwrap();
        dbg!(third);

        let re = Regex::new(r"<[^>]*>").unwrap();
        let (feed_info, max_articles, slug) = get_default_config();
        let feed_data = build_feed(feed, feed_info, max_articles, &re, slug);
        assert!(feed_data.is_some(), "Managed to build feed data");
        let feed_data = feed_data.unwrap();

        let items: Vec<_> = (&feed_data).into();
        dbg!(items);
    }
}
