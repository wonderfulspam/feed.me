use std::str::FromStr;

use anyhow::{Context, Result};
use clap::Args;

use crate::{config::Config, FeedInfo, Tier};

#[derive(Args)]
pub struct AddFeedArgs {
    #[arg(long)]
    pub slug: String,
    #[arg(long)]
    pub url: String,
    #[arg(long)]
    pub author: String,
    #[arg(long)]
    pub tier: String,
    #[arg(long, default_value = "./spacefeeder.toml")]
    pub config_path: String,
}

pub fn execute(args: AddFeedArgs) -> Result<()> {
    let mut config = Config::from_file(&args.config_path)?;
    run(&mut config, args.slug, args.url, args.author, args.tier)?;
    config.save(&args.config_path)
}

pub fn run(
    config: &mut Config,
    slug: String,
    url: String,
    author: String,
    tier: String,
) -> Result<()> {
    let tier = Tier::from_str(&tier).with_context(|| format!("Not a valid tier: {tier}"))?;
    let feed = FeedInfo { url, author, tier, tags: None, auto_tag: None };
    config.insert_feed(slug, feed);
    Ok(())
}
