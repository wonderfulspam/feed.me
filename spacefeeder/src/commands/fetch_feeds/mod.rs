mod fetcher;
mod processor;
mod search_indexer;
mod text_utils;
mod types;

use fetcher::fetch_feed;
use processor::build_feed;
use search_indexer::build_search_index;
pub use types::*;

use std::thread;
use std::time::Duration;

use anyhow::Result;
use clap::Args;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use serde::Serialize;
use ureq::Agent;

use crate::config::Config;

#[derive(Args)]
pub struct FetchArgs {
    /// Path to the config file
    #[arg(long, default_value = "./spacefeeder.toml")]
    pub config_path: String,
}

pub fn execute(_args: FetchArgs) -> Result<()> {
    let config = crate::config::get_config().clone();
    run(config)
}

pub fn run(config: Config) -> Result<()> {
    let agent = Agent::new_with_defaults();

    println!("Fetching {} feeds...", config.feeds.len());

    let html_strip_regex = Regex::new(r"<[^>]*>").unwrap();

    // Use rayon for parallel processing
    let processed_feeds: Vec<ProcessedFeed> = config
        .feeds
        .par_iter()
        .filter_map(|(slug, feed_info)| {
            println!("  Fetching: {}", slug);

            match fetch_feed_with_retry(&agent, &feed_info.url, 3) {
                Some(feed) => {
                    let processed = build_feed(
                        feed,
                        feed_info.clone(),
                        &config,
                        &html_strip_regex,
                        slug.clone(),
                    );
                    Some(processed)
                }
                None => {
                    eprintln!("  ✗ Failed to fetch: {}", slug);
                    None
                }
            }
        })
        .collect();

    if processed_feeds.is_empty() {
        return Err(anyhow::anyhow!("No feeds could be fetched"));
    }

    println!("✓ Successfully fetched {} feeds", processed_feeds.len());

    // Convert to display format for JSON output
    let feed_outputs: Vec<FeedOutput> = processed_feeds
        .iter()
        .map(|pf| pf.display_output.clone())
        .collect();

    // Write feed data
    write_data_to_file(&config.output_config.feed_data_output_path, &feed_outputs);
    write_data_to_file("./content/data/feedData.json", &feed_outputs);

    // Create item data
    let all_search_items: Vec<ItemOutput> = processed_feeds
        .iter()
        .flat_map(|pf| {
            pf.all_items.iter().map(move |item| ItemOutput {
                meta: pf.meta.clone(),
                slug: pf.slug.clone(),
                item: item.clone(),
            })
        })
        .collect();

    // Write item data for templates
    write_data_to_file("./content/data/itemData.json", &all_search_items);

    // Build tier-specific data from all items
    let (loved_data, liked_data, new_data) = split_items_by_tier(&all_search_items);
    write_data_to_file("./content/data/lovedData.json", &loved_data);
    write_data_to_file("./content/data/likedData.json", &liked_data);
    write_data_to_file("./content/data/newData.json", &new_data);

    if let Err(e) = build_search_index(&all_search_items) {
        eprintln!("⚠ Warning: Failed to build search index: {}", e);
    } else {
        println!(
            "✓ Search index updated with {} items",
            all_search_items.len()
        );
    }

    // Return Ok even if some feeds failed - the operation as a whole succeeded
    Ok(())
}

fn write_data_to_file<D: Serialize>(output_path: &str, data: &D) {
    match std::fs::create_dir_all(std::path::Path::new(output_path).parent().unwrap()) {
        Ok(_) => {}
        Err(e) => eprintln!("Warning: Failed to create directory: {}", e),
    }

    match std::fs::write(output_path, serde_json::to_string_pretty(data).unwrap()) {
        Ok(_) => {}
        Err(e) => eprintln!("Warning: Failed to write to {}: {}", output_path, e),
    }
}

fn fetch_feed_with_retry(agent: &Agent, url: &str, retries: u32) -> Option<feed_rs::model::Feed> {
    for attempt in 1..=retries {
        match fetch_feed(agent, url) {
            Some(feed) => return Some(feed),
            None => {
                if attempt < retries {
                    // Add exponential backoff
                    let delay = Duration::from_millis(100 * (1 << attempt));
                    thread::sleep(delay);
                }
            }
        }
    }
    None
}

fn split_items_by_tier(items: &[ItemOutput]) -> (Vec<&ItemOutput>, Vec<&ItemOutput>, Vec<&ItemOutput>) {
    let mut loved = Vec::new();
    let mut liked = Vec::new();
    let mut new = Vec::new();

    for item in items {
        match item.meta.tier {
            crate::Tier::Love => loved.push(item),
            crate::Tier::Like => liked.push(item),
            crate::Tier::New => new.push(item),
        }
    }

    (loved, liked, new)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use feed_rs::parser;
    use regex::Regex;

    #[test]
    fn test_fetch_and_build_feed() {
        let feed_xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
            <channel>
                <title>Test Feed</title>
                <item>
                    <title>Test Item</title>
                    <link>http://example.com/item</link>
                    <description>Test description</description>
                </item>
            </channel>
        </rss>
        "#;

        let feed = parser::parse(feed_xml.as_bytes());
        assert!(feed.is_ok(), "Feed parsed correctly");
        let feed = feed.unwrap();

        let re = Regex::new(r"<[^>]*>").unwrap();
        let config = Config::default();
        let (slug, feed_info) = config.feeds.clone().into_iter().next().unwrap();
        let feed_data = build_feed(feed, feed_info, &config, &re, slug);
        let items: Vec<ItemOutput> = (&feed_data.display_output).into();
        assert_eq!(items.len(), 1); // Test feed has only 1 item
    }
}
