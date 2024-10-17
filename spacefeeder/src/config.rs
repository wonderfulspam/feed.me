use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{FeedInfo, Tier};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(flatten)]
    pub(crate) parse_config: ParseConfig,
    #[serde(flatten)]
    pub(crate) output_config: OutputConfig,
    pub(crate) feeds: HashMap<String, FeedInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ParseConfig {
    pub(crate) max_articles: usize,
    pub(crate) description_max_words: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OutputConfig {
    #[serde(default = "default_feed_data_output_path")]
    pub(crate) feed_data_output_path: String,
    #[serde(default = "default_item_data_output_path")]
    pub(crate) item_data_output_path: String,
}

fn default_feed_data_output_path() -> String {
    "./content/data/feedData.json".to_string()
}

fn default_item_data_output_path() -> String {
    "./content/data/itemData.json".to_string()
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {path}"))?;
        let config = toml_edit::de::from_str(&content)
            .with_context(|| format!("Failed to parse TOML from file: {path}"))?;
        Ok(config)
    }

    pub(crate) fn insert_feed(&mut self, slug: String, feed: FeedInfo) {
        let _ = self.feeds.insert(slug, feed);
    }

    pub fn save(&self, config_path: &str) -> Result<()> {
        let output = toml_edit::ser::to_string_pretty(self)?;
        std::fs::write(config_path, output)
            .with_context(|| format!("Failed to write to {config_path}"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            parse_config: ParseConfig {
                max_articles: 5,
                description_max_words: 150,
            },
            output_config: OutputConfig {
                feed_data_output_path: default_feed_data_output_path(),
                item_data_output_path: default_item_data_output_path(),
            },
            feeds: HashMap::from([(
                "example".to_string(),
                FeedInfo {
                    url: "www.example.com".to_string(),
                    author: "Example Author".to_string(),
                    tier: Tier::New,
                },
            )]),
        }
    }
}
