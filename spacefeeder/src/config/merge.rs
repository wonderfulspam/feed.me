use super::CategorizationConfig;
use crate::defaults;
use crate::{FeedInfo, UserFeedInfo};
use anyhow::Result;
use std::collections::HashMap;

/// Handles merging of default configuration with user configuration
pub struct ConfigMerger;

impl ConfigMerger {
    /// Merge default tags with user-provided tags
    /// User tags take precedence over defaults
    pub fn merge_tags(user_config: &mut CategorizationConfig) {
        let default_tags = defaults::get_default_tags();
        let user_tag_names: Vec<String> = user_config.tags.iter().map(|t| t.name.clone()).collect();

        // Add default tags that aren't overridden by user
        for default_tag in default_tags {
            if !user_tag_names.contains(&default_tag.name) {
                user_config.tags.push(default_tag);
            }
        }
    }

    /// Merge default feeds with user-provided feeds
    /// Returns a HashMap of fully merged feeds
    pub fn merge_feeds(
        user_feeds: HashMap<String, UserFeedInfo>,
    ) -> Result<HashMap<String, FeedInfo>> {
        let mut merged_feeds = HashMap::new();
        let default_feeds = defaults::get_default_feeds();

        // Process feeds that have defaults
        for (slug, default_feed) in &default_feeds {
            if let Some(user_feed) = user_feeds.get(slug) {
                // User has specified this feed, merge with defaults
                let final_feed = Self::merge_single_feed(default_feed.clone(), user_feed);
                merged_feeds.insert(slug.clone(), final_feed);
            }
            // Don't add default feeds that aren't explicitly mentioned by user
        }

        // Process custom user feeds (without defaults)
        for (slug, user_feed) in user_feeds {
            if !default_feeds.contains_key(&slug) {
                // User-defined feed without defaults, all fields must be provided
                let final_feed = Self::create_custom_feed(&slug, user_feed)?;
                merged_feeds.insert(slug, final_feed);
            }
        }

        Ok(merged_feeds)
    }

    /// Merge a single feed with defaults
    fn merge_single_feed(mut default_feed: FeedInfo, user_feed: &UserFeedInfo) -> FeedInfo {
        // Always use user's tier preference
        default_feed.tier = user_feed.tier;

        // Override with user-specified fields if present
        if let Some(ref user_url) = user_feed.url {
            default_feed.url = user_url.clone();
        }
        if let Some(ref user_author) = user_feed.author {
            default_feed.author = user_author.clone();
        }
        if let Some(ref user_description) = user_feed.description {
            default_feed.description = Some(user_description.clone());
        }
        if let Some(ref user_tags) = user_feed.tags {
            // Merge user tags with default tags
            let mut all_tags = default_feed.tags.unwrap_or_default();
            all_tags.extend(user_tags.iter().cloned());
            all_tags.sort();
            all_tags.dedup();
            default_feed.tags = Some(all_tags);
        }
        if user_feed.auto_tag.is_some() {
            default_feed.auto_tag = user_feed.auto_tag;
        }

        default_feed
    }

    /// Create a custom feed from user configuration (no defaults available)
    fn create_custom_feed(slug: &str, user_feed: UserFeedInfo) -> Result<FeedInfo> {
        Ok(FeedInfo {
            url: user_feed
                .url
                .ok_or_else(|| anyhow::anyhow!("Feed '{}' must specify url", slug))?,
            author: user_feed
                .author
                .ok_or_else(|| anyhow::anyhow!("Feed '{}' must specify author", slug))?,
            description: user_feed.description,
            tier: user_feed.tier,
            tags: user_feed.tags,
            auto_tag: user_feed.auto_tag,
        })
    }

    /// Merge default categorization rules and aliases
    pub fn merge_categorization(user_config: &mut CategorizationConfig) {
        let (default_rules, default_aliases) = defaults::get_default_categorization();

        // Add default rules that aren't already present
        for default_rule in default_rules {
            user_config.rules.push(default_rule);
        }

        // Add default aliases that aren't already present
        for default_alias in default_aliases {
            user_config.aliases.push(default_alias);
        }
    }
}
