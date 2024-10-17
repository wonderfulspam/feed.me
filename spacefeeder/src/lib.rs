pub mod commands;
pub mod config;

use std::str::FromStr;

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

impl FromStr for Tier {
    type Err = std::fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "new" | "New" => Ok(Tier::New),
            "like" | "Like" => Ok(Tier::Like),
            "love" | "Love" => Ok(Tier::Love),
            _ => Err(std::fmt::Error),
        }
    }
}
