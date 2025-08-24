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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.parse_config.max_articles, 5);
        assert_eq!(config.parse_config.description_max_words, 150);
        assert_eq!(config.output_config.feed_data_output_path, "./content/data/feedData.json");
        assert_eq!(config.output_config.item_data_output_path, "./content/data/itemData.json");
        assert_eq!(config.feeds.len(), 1);
        assert!(config.feeds.contains_key("example"));
    }

    #[test]
    fn test_insert_feed() {
        let mut config = Config::default();
        let feed = FeedInfo {
            url: "https://test.com/feed".to_string(),
            author: "Test Author".to_string(),
            tier: Tier::Love,
        };
        config.insert_feed("test_feed".to_string(), feed);
        assert_eq!(config.feeds.len(), 2);
        assert!(config.feeds.contains_key("test_feed"));
    }

    #[test]
    fn test_config_from_file() {
        let toml_content = r#"
max_articles = 10
description_max_words = 200

[feeds.test_feed]
url = "https://example.com/feed"
author = "Test Author"
tier = "love"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.parse_config.max_articles, 10);
        assert_eq!(config.parse_config.description_max_words, 200);
        assert_eq!(config.feeds.len(), 1);
        assert!(config.feeds.contains_key("test_feed"));
        
        let feed = &config.feeds["test_feed"];
        assert_eq!(feed.url, "https://example.com/feed");
        assert_eq!(feed.author, "Test Author");
        assert!(matches!(feed.tier, Tier::Love));
    }

    #[test]
    fn test_config_save() {
        let config = Config::default();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        
        config.save(path).unwrap();
        
        // Read back and verify
        let loaded_config = Config::from_file(path).unwrap();
        assert_eq!(loaded_config.parse_config.max_articles, config.parse_config.max_articles);
        assert_eq!(loaded_config.parse_config.description_max_words, config.parse_config.description_max_words);
        assert_eq!(loaded_config.feeds.len(), config.feeds.len());
    }

    #[test]
    fn test_default_output_paths() {
        assert_eq!(default_feed_data_output_path(), "./content/data/feedData.json");
        assert_eq!(default_item_data_output_path(), "./content/data/itemData.json");
    }
}
