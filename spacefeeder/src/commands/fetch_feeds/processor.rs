use crate::categorization::{CategorizationEngine, ItemContext};
use crate::config::Config;
use crate::FeedInfo;
use regex::Regex;

use super::{
    text_utils::{extract_first_paragraph, get_description_from_entry, get_short_description},
    types::{FeedOutput, ProcessedFeed, RssItem},
};

/// Build a complete feed with processed items
pub fn build_feed(
    feed: feed_rs::model::Feed,
    feed_info: FeedInfo,
    config: &Config,
    html_strip_regex: &Regex,
    slug: String,
) -> ProcessedFeed {
    let categorization_engine = CategorizationEngine::from_config(&config.categorization);

    let items: Vec<RssItem> = feed
        .entries
        .into_iter()
        .take(config.parse_config.max_articles_for_search)
        .map(|entry| {
            build_item(
                entry,
                &feed_info,
                &slug,
                config,
                html_strip_regex,
                &categorization_engine,
            )
        })
        .collect();

    let display_items = items
        .iter()
        .take(config.parse_config.max_articles)
        .cloned()
        .collect();

    ProcessedFeed {
        display_output: FeedOutput {
            meta: feed_info.clone(),
            slug: slug.clone(),
            items: display_items,
        },
        all_items: items,
        meta: feed_info,
        slug,
    }
}

/// Build a single RSS item with categorization and text processing
pub fn build_item(
    entry: feed_rs::model::Entry,
    feed_info: &FeedInfo,
    feed_slug: &str,
    config: &Config,
    html_strip_regex: &Regex,
    categorization_engine: &CategorizationEngine,
) -> RssItem {
    let title = entry
        .title
        .as_ref()
        .map(|t| t.content.clone())
        .unwrap_or_default();
    let item_url = entry
        .links
        .first()
        .map(|link| link.href.clone())
        .unwrap_or_default();

    // Get and process description
    let raw_description = get_description_from_entry(entry.clone()).unwrap_or_default();
    let stripped_description = html_strip_regex
        .replace_all(&raw_description, "")
        .to_string();
    let safe_description = get_short_description(
        stripped_description.clone(),
        config.parse_config.description_max_words,
    );

    // Try to get a clean description for display
    let description =
        extract_first_paragraph(&stripped_description).unwrap_or_else(|| safe_description.clone());

    // Get RSS categories as potential tags
    let rss_categories: Vec<String> = entry.categories.iter().map(|c| c.term.clone()).collect();
    let rss_categories_slice = if rss_categories.is_empty() {
        None
    } else {
        Some(rss_categories.as_slice())
    };

    // Apply categorization
    let mut tags = Vec::new();
    if categorization_engine.is_enabled() {
        let context = ItemContext {
            title: &title,
            description: Some(&description),
            link: Some(&item_url),
            author: Some(&feed_info.author),
            feed_slug,
            feed_tags: feed_info.tags.as_deref(),
            rss_categories: rss_categories_slice,
        };

        let generated_tags = categorization_engine.generate_tags_for_item(&context);
        tags = generated_tags.into_iter().map(|t| t.name).collect();
    }

    RssItem {
        title,
        item_url,
        description,
        safe_description,
        pub_date: entry.published.or(entry.updated),
        tags,
    }
}
