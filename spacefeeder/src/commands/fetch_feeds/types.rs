use crate::FeedInfo;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct FeedOutput {
    #[serde(flatten)]
    pub meta: FeedInfo,
    pub slug: String,
    pub items: Vec<RssItem>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ItemOutput {
    #[serde(flatten)]
    pub meta: FeedInfo,
    pub slug: String,
    #[serde(flatten)]
    pub item: RssItem,
}

#[derive(Clone, Debug, Serialize)]
pub struct RssItem {
    pub title: String,
    pub item_url: String,
    pub description: String,
    pub safe_description: String,
    pub pub_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

pub struct ProcessedFeed {
    pub display_output: FeedOutput,
    pub all_items: Vec<RssItem>,
    pub meta: FeedInfo,
    pub slug: String,
}

impl From<&FeedOutput> for Vec<ItemOutput> {
    fn from(feed: &FeedOutput) -> Self {
        feed.items
            .iter()
            .map(move |item| ItemOutput {
                meta: feed.meta.clone(),
                slug: feed.slug.clone(),
                item: item.clone(),
            })
            .collect::<Vec<_>>()
    }
}
