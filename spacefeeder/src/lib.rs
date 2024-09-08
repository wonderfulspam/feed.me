pub mod commands;
pub mod config;

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
struct FeedInfo {
    url: String,
    author: String,
    tier: Tier,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum Tier {
    New,
    Like,
    Love,
}
