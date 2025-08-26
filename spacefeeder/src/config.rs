use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::defaults;
use crate::{FeedInfo, Tier, UserFeedInfo};

static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(flatten)]
    pub(crate) parse_config: ParseConfig,
    #[serde(flatten)]
    pub(crate) output_config: OutputConfig,
    #[serde(default)]
    pub(crate) categorization: CategorizationConfig,
    pub(crate) feeds: HashMap<String, FeedInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParseConfig {
    pub(crate) max_articles: usize,
    pub(crate) description_max_words: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputConfig {
    #[serde(default = "default_feed_data_output_path")]
    pub(crate) feed_data_output_path: String,
    #[serde(default = "default_item_data_output_path")]
    pub(crate) item_data_output_path: String,
    #[serde(default = "default_base_url")]
    pub(crate) base_url: String,
}

fn default_feed_data_output_path() -> String {
    "./content/data/feedData.json".to_string()
}

fn default_item_data_output_path() -> String {
    "./content/data/itemData.json".to_string()
}

fn default_base_url() -> String {
    "http://localhost:8000/".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CategorizationConfig {
    #[serde(default = "default_categorization_enabled")]
    pub enabled: bool,
    #[serde(default = "default_auto_tag_new_articles")]
    pub auto_tag_new_articles: bool,
    #[serde(default = "default_max_tags_per_item")]
    pub max_tags_per_item: usize,
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f32,
    #[serde(default)]
    pub tags: Vec<TagDefinition>,
    #[serde(default)]
    pub rules: Vec<TagRule>,
    #[serde(default)]
    pub aliases: Vec<TagAlias>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TagDefinition {
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TagRule {
    #[serde(rename = "type")]
    pub rule_type: String,
    pub patterns: Vec<String>,
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub confidence: f32,
    /// Negative patterns that exclude this rule if matched
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    /// Minimum number of keywords that must match (for content_analysis rules)
    #[serde(default)]
    pub min_keyword_count: Option<usize>,
    /// Required keywords that must all be present (for author_with_content rules)
    #[serde(default)]
    pub required_keywords: Vec<String>,
    /// Tags to exclude if this rule matches (for exclude_if rules)
    #[serde(default)]
    pub exclude_tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TagAlias {
    pub from: Vec<String>,
    pub to: String,
}

impl Default for CategorizationConfig {
    fn default() -> Self {
        Self {
            enabled: default_categorization_enabled(),
            auto_tag_new_articles: default_auto_tag_new_articles(),
            max_tags_per_item: default_max_tags_per_item(),
            confidence_threshold: default_confidence_threshold(),
            tags: Vec::new(),
            rules: Vec::new(),
            aliases: Vec::new(),
        }
    }
}

fn default_categorization_enabled() -> bool {
    true
}

fn default_auto_tag_new_articles() -> bool {
    true
}

fn default_max_tags_per_item() -> usize {
    5
}

fn default_confidence_threshold() -> f32 {
    0.3
}

// Temporary struct for parsing user config that can handle minimal feed definitions
#[derive(Debug, Deserialize)]
struct ParsedConfig {
    #[serde(flatten)]
    parse_config: ParseConfig,
    #[serde(flatten)]
    output_config: OutputConfig,
    #[serde(default)]
    categorization: CategorizationConfig,
    #[serde(default)]
    feeds: HashMap<String, UserFeedInfo>,
}

// Minimal struct for saving user config without defaults
#[derive(Debug, Serialize)]
struct SaveConfig {
    #[serde(flatten)]
    parse_config: ParseConfig,
    #[serde(flatten)]
    output_config: OutputConfig,
    categorization: SaveCategorizationConfig,
    feeds: BTreeMap<String, UserFeedInfo>,
}

// Minimal categorization config for saving
#[derive(Debug, Serialize)]
struct SaveCategorizationConfig {
    enabled: bool,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {path}"))?;
        let parsed_config: ParsedConfig = toml_edit::de::from_str(&content)
            .with_context(|| format!("Failed to parse TOML from file: {path}"))?;

        let mut config = Config {
            parse_config: parsed_config.parse_config,
            output_config: parsed_config.output_config,
            categorization: parsed_config.categorization,
            feeds: HashMap::new(),
        };

        // Merge default tags with user-provided tags
        let default_tags = defaults::get_default_tags();
        let user_tag_names: Vec<String> = config
            .categorization
            .tags
            .iter()
            .map(|t| t.name.clone())
            .collect();

        // Add default tags that aren't overridden by user
        for default_tag in default_tags {
            if !user_tag_names.contains(&default_tag.name) {
                config.categorization.tags.push(default_tag);
            }
        }

        // Merge default feeds with user-provided feeds
        let default_feeds = defaults::get_default_feeds();
        for (slug, default_feed) in default_feeds {
            if let Some(user_feed) = parsed_config.feeds.get(&slug) {
                // User has specified this feed, merge with defaults
                // User config only overrides certain fields, everything else comes from defaults
                let mut final_feed = default_feed;
                final_feed.tier = user_feed.tier; // Always use user's tier preference

                // Override with user-specified fields if present
                if let Some(ref user_url) = user_feed.url {
                    final_feed.url = user_url.clone();
                }
                if let Some(ref user_author) = user_feed.author {
                    final_feed.author = user_author.clone();
                }
                if let Some(ref user_description) = user_feed.description {
                    final_feed.description = Some(user_description.clone());
                }
                if let Some(ref user_tags) = user_feed.tags {
                    // Merge user tags with default tags
                    let mut all_tags = final_feed.tags.unwrap_or_default();
                    all_tags.extend(user_tags.iter().cloned());
                    all_tags.sort();
                    all_tags.dedup();
                    final_feed.tags = Some(all_tags);
                }
                if user_feed.auto_tag.is_some() {
                    final_feed.auto_tag = user_feed.auto_tag;
                }

                config.feeds.insert(slug, final_feed);
            }
            // Don't add default feeds that aren't explicitly mentioned by user
        }

        // Handle user feeds that don't have defaults (custom feeds)
        for (slug, user_feed) in parsed_config.feeds {
            if let std::collections::hash_map::Entry::Vacant(e) = config.feeds.entry(slug) {
                // User-defined feed without defaults, all fields must be provided
                let final_feed = FeedInfo {
                    url: user_feed
                        .url
                        .ok_or_else(|| anyhow::anyhow!("Feed '{}' must specify url", e.key()))?,
                    author: user_feed
                        .author
                        .ok_or_else(|| anyhow::anyhow!("Feed '{}' must specify author", e.key()))?,
                    description: user_feed.description,
                    tier: user_feed.tier,
                    tags: user_feed.tags,
                    auto_tag: user_feed.auto_tag,
                };
                e.insert(final_feed);
            }
        }

        // Merge default categorization rules and aliases
        let (default_rules, default_aliases) = defaults::get_default_categorization();

        // Add default rules that aren't already present
        for default_rule in default_rules {
            config.categorization.rules.push(default_rule);
        }

        // Add default aliases that aren't already present
        for default_alias in default_aliases {
            config.categorization.aliases.push(default_alias);
        }

        Ok(config)
    }

    pub(crate) fn insert_feed(&mut self, slug: String, feed: FeedInfo) {
        let _ = self.feeds.insert(slug, feed);
    }

    pub fn base_url(&self) -> &str {
        &self.output_config.base_url
    }

    pub fn save(&self, config_path: &str) -> Result<()> {
        // Create minimal config for saving - only user-specified data
        let default_feeds = defaults::get_default_feeds();
        let mut user_feeds = BTreeMap::new();

        // Only include feeds that are either:
        // 1. Custom feeds (not in defaults)
        // 2. Default feeds with non-default tier or other overrides
        for (slug, feed) in &self.feeds {
            if let Some(default_feed) = default_feeds.get(slug) {
                // This is a default feed - only save if user has customized it
                let mut user_feed = UserFeedInfo {
                    tier: feed.tier,
                    url: None,
                    author: None,
                    description: None,
                    tags: None,
                    auto_tag: feed.auto_tag,
                };

                // Only include overridden fields
                if feed.url != default_feed.url {
                    user_feed.url = Some(feed.url.clone());
                }
                if feed.author != default_feed.author {
                    user_feed.author = Some(feed.author.clone());
                }
                if feed.description != default_feed.description {
                    user_feed.description = feed.description.clone();
                }
                if feed.tags != default_feed.tags {
                    user_feed.tags = feed.tags.clone();
                }

                user_feeds.insert(slug.clone(), user_feed);
            } else {
                // Custom feed - include all required fields
                let user_feed = UserFeedInfo {
                    url: Some(feed.url.clone()),
                    author: Some(feed.author.clone()),
                    description: feed.description.clone(),
                    tier: feed.tier,
                    tags: feed.tags.clone(),
                    auto_tag: feed.auto_tag,
                };
                user_feeds.insert(slug.clone(), user_feed);
            }
        }

        // Create minimal save structure
        let save_config = SaveConfig {
            parse_config: self.parse_config.clone(),
            output_config: self.output_config.clone(),
            categorization: SaveCategorizationConfig {
                enabled: self.categorization.enabled,
            },
            feeds: user_feeds,
        };

        let output = toml_edit::ser::to_string_pretty(&save_config)?;
        std::fs::write(config_path, output)
            .with_context(|| format!("Failed to write to {config_path}"))
    }
}

// Global config management functions
pub fn init_config(config_path: &str) -> Result<()> {
    let config = Config::from_file(config_path)?;
    GLOBAL_CONFIG
        .set(config)
        .map_err(|_| anyhow::anyhow!("Global config was already initialized"))?;
    Ok(())
}

pub fn get_config() -> &'static Config {
    GLOBAL_CONFIG
        .get()
        .expect("Config must be initialized before use")
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
                base_url: default_base_url(),
            },
            categorization: CategorizationConfig::default(),
            feeds: HashMap::from([(
                "example".to_string(),
                FeedInfo {
                    url: "www.example.com".to_string(),
                    author: "Example Author".to_string(),
                    description: Some("Example feed for testing".to_string()),
                    tier: Tier::New,
                    tags: None,
                    auto_tag: None,
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
        assert_eq!(
            config.output_config.feed_data_output_path,
            "./content/data/feedData.json"
        );
        assert_eq!(
            config.output_config.item_data_output_path,
            "./content/data/itemData.json"
        );
        assert_eq!(config.feeds.len(), 1);
        assert!(config.feeds.contains_key("example"));
    }

    #[test]
    fn test_insert_feed() {
        let mut config = Config::default();
        let feed = FeedInfo {
            url: "https://test.com/feed".to_string(),
            author: "Test Author".to_string(),
            description: Some("Test feed".to_string()),
            tier: Tier::Love,
            tags: None,
            auto_tag: None,
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
        assert_eq!(
            loaded_config.parse_config.max_articles,
            config.parse_config.max_articles
        );
        assert_eq!(
            loaded_config.parse_config.description_max_words,
            config.parse_config.description_max_words
        );
        assert_eq!(loaded_config.feeds.len(), config.feeds.len());
    }

    #[test]
    fn test_default_output_paths() {
        assert_eq!(
            default_feed_data_output_path(),
            "./content/data/feedData.json"
        );
        assert_eq!(
            default_item_data_output_path(),
            "./content/data/itemData.json"
        );
    }

    #[test]
    fn test_default_tags_loaded() {
        let toml_content = r#"
max_articles = 10
description_max_words = 200

[categorization]
enabled = true

[feeds.test_feed]
url = "https://example.com/feed"
author = "Test Author"
tier = "love"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();

        // Verify default tags are loaded
        assert!(
            !config.categorization.tags.is_empty(),
            "Should have loaded default tags"
        );

        let tag_names: Vec<String> = config
            .categorization
            .tags
            .iter()
            .map(|t| t.name.clone())
            .collect();

        // Check for some expected default tags
        assert!(
            tag_names.contains(&"rust".to_string()),
            "Should contain rust tag"
        );
        assert!(
            tag_names.contains(&"ai".to_string()),
            "Should contain ai tag"
        );
        assert!(
            tag_names.contains(&"python".to_string()),
            "Should contain python tag"
        );
        assert!(
            tag_names.contains(&"devops".to_string()),
            "Should contain devops tag"
        );
    }

    #[test]
    fn test_user_tags_override_defaults() {
        let toml_content = r#"
max_articles = 10
description_max_words = 200

[categorization]
enabled = true

[[categorization.tags]]
name = "rust"
description = "Custom Rust description"
keywords = ["custom", "rust"]

[feeds.test_feed]
url = "https://example.com/feed"
author = "Test Author"
tier = "love"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();

        // Find the rust tag
        let rust_tag = config
            .categorization
            .tags
            .iter()
            .find(|t| t.name == "rust")
            .expect("Should have rust tag");

        // Verify user version is used, not default
        assert_eq!(rust_tag.description, "Custom Rust description");
        assert_eq!(rust_tag.keywords, vec!["custom", "rust"]);

        // Verify other default tags are still loaded
        let tag_names: Vec<String> = config
            .categorization
            .tags
            .iter()
            .map(|t| t.name.clone())
            .collect();
        assert!(
            tag_names.contains(&"ai".to_string()),
            "Should still have other default tags"
        );
    }
}
