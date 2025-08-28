use crate::{FeedInfo, UserFeedInfo};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

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
    #[serde(default = "default_max_articles_for_search")]
    pub(crate) max_articles_for_search: usize,
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

// Temporary struct for parsing user config that can handle minimal feed definitions
#[derive(Debug, Deserialize)]
pub(super) struct ParsedConfig {
    #[serde(flatten)]
    pub parse_config: ParseConfig,
    #[serde(flatten)]
    pub output_config: OutputConfig,
    #[serde(default)]
    pub categorization: CategorizationConfig,
    #[serde(default)]
    pub feeds: HashMap<String, UserFeedInfo>,
}

// Minimal struct for saving user config without defaults
#[derive(Debug, Serialize)]
pub struct SaveConfig {
    #[serde(flatten)]
    pub parse_config: ParseConfig,
    #[serde(flatten)]
    pub output_config: OutputConfig,
    pub categorization: SaveCategorizationConfig,
    pub feeds: BTreeMap<String, UserFeedInfo>,
}

// Minimal categorization config for saving
#[derive(Debug, Serialize)]
pub struct SaveCategorizationConfig {
    pub enabled: bool,
}

// Default functions
fn default_feed_data_output_path() -> String {
    "./content/data/feedData.json".to_string()
}

fn default_item_data_output_path() -> String {
    "./content/data/itemData.json".to_string()
}

fn default_base_url() -> String {
    "http://localhost:8000/".to_string()
}

fn default_max_articles_for_search() -> usize {
    200
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
