use std::collections::HashMap;
use serde::Deserialize;
use crate::config::{TagDefinition, TagRule, TagAlias};
use crate::{FeedInfo, Tier};

// Embed default data at compile time
const DEFAULT_TAGS_TOML: &str = include_str!("../../data/tags.toml");
const DEFAULT_FEEDS_TOML: &str = include_str!("../../data/feeds.toml");
const DEFAULT_CATEGORIZATION_TOML: &str = include_str!("../../data/categorization.toml");

#[derive(Debug, Deserialize)]
struct DefaultTags {
    tags: Vec<TagDefinition>,
}

#[derive(Debug, Deserialize)]
struct DefaultFeeds {
    feeds: HashMap<String, DefaultFeedInfo>,
}

#[derive(Debug, Deserialize)]
struct DefaultFeedInfo {
    url: String,
    author: String,
    description: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DefaultCategorization {
    rules: Vec<TagRule>,
    aliases: Vec<TagAlias>,
}

pub fn get_default_tags() -> Vec<TagDefinition> {
    let default_tags: DefaultTags = toml_edit::de::from_str(DEFAULT_TAGS_TOML)
        .expect("Failed to parse embedded tags.toml");
    default_tags.tags
}

pub fn get_default_feeds() -> HashMap<String, FeedInfo> {
    let default_feeds: DefaultFeeds = toml_edit::de::from_str(DEFAULT_FEEDS_TOML)
        .expect("Failed to parse embedded feeds.toml");
    
    default_feeds.feeds
        .into_iter()
        .map(|(slug, feed_info)| {
            (slug, FeedInfo {
                url: feed_info.url,
                author: feed_info.author,
                description: Some(feed_info.description),
                tier: Tier::New, // Default tier for built-in feeds
                tags: Some(feed_info.tags),
                auto_tag: None,
            })
        })
        .collect()
}

pub fn get_default_categorization() -> (Vec<TagRule>, Vec<TagAlias>) {
    let default_cat: DefaultCategorization = toml_edit::de::from_str(DEFAULT_CATEGORIZATION_TOML)
        .expect("Failed to parse embedded categorization.toml");
    (default_cat.rules, default_cat.aliases)
}