use super::types::{CategorizationConfig, Config, OutputConfig, ParseConfig};
use crate::{FeedInfo, Tier};
use std::collections::HashMap;

impl Default for Config {
    fn default() -> Self {
        Self {
            parse_config: ParseConfig {
                max_articles: 5,
                max_articles_for_search: 200,
                description_max_words: 150,
            },
            output_config: OutputConfig {
                feed_data_output_path: "./content/data/feedData.json".to_string(),
                item_data_output_path: "./content/data/itemData.json".to_string(),
                base_url: "http://localhost:8000/".to_string(),
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
