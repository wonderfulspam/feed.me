use std::str::FromStr;

use anyhow::{Context, Result};

use crate::{config::Config, FeedInfo, Tier};

pub fn run(
    config: &mut Config,
    slug: String,
    url: String,
    author: String,
    tier: String,
) -> Result<()> {
    let tier = Tier::from_str(&tier).with_context(|| format!("Not a valid tier: {tier}"))?;
    let feed = FeedInfo { url, author, tier };
    config.insert_feed(slug, feed);
    Ok(())
}
