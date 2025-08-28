use super::{CategorizationConfig, Config, SaveCategorizationConfig, SaveConfig};
use crate::defaults;
use crate::{FeedInfo, UserFeedInfo};
use anyhow::Result;
use std::collections::BTreeMap;

/// Handles saving configuration in minimal format
pub struct ConfigSaver;

impl ConfigSaver {
    /// Save config to file with only user-specified overrides
    pub fn save_to_file(config: &Config, config_path: &str) -> Result<()> {
        let user_feeds = Self::extract_user_feeds(&config.feeds);

        // Create minimal save structure
        let save_config = SaveConfig {
            parse_config: config.parse_config.clone(),
            output_config: config.output_config.clone(),
            categorization: Self::extract_user_categorization(&config.categorization),
            feeds: user_feeds,
        };

        // Serialize and save
        let toml_string = toml_edit::ser::to_string_pretty(&save_config)?;
        std::fs::write(config_path, toml_string)?;

        Ok(())
    }

    /// Extract only user-specified feeds (not defaults)
    fn extract_user_feeds(
        feeds: &std::collections::HashMap<String, FeedInfo>,
    ) -> BTreeMap<String, UserFeedInfo> {
        let default_feeds = defaults::get_default_feeds();
        let mut user_feeds = BTreeMap::new();

        for (slug, feed) in feeds {
            if let Some(default_feed) = default_feeds.get(slug) {
                // This is a default feed - only save if user has customized it
                let user_feed = Self::create_user_feed_override(feed, default_feed);
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

        user_feeds
    }

    /// Create minimal user feed with only overridden fields
    fn create_user_feed_override(feed: &FeedInfo, default_feed: &FeedInfo) -> UserFeedInfo {
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

        user_feed
    }

    /// Extract only user-specified categorization (empty for now, as all is default)
    fn extract_user_categorization(config: &CategorizationConfig) -> SaveCategorizationConfig {
        // For now, only save enabled flag since all categorization comes from defaults
        // In the future, this could filter out default rules/tags/aliases
        SaveCategorizationConfig {
            enabled: config.enabled,
        }
    }
}
