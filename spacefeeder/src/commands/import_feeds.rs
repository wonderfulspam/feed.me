use std::str::FromStr;

use anyhow::{Context, Result};
use opml::OPML;

use crate::{config::Config, FeedInfo, Tier};

pub fn run(config: &mut Config, input_path: String, default_tier: String) -> Result<()> {
    let tier = Tier::from_str(&default_tier)
        .with_context(|| format!("Not a valid tier: {default_tier}"))?;
    
    let opml_content = std::fs::read_to_string(&input_path)
        .with_context(|| format!("Failed to read OPML file: {input_path}"))?;
    
    let opml = OPML::from_str(&opml_content)
        .with_context(|| format!("Failed to parse OPML file: {input_path}"))?;
    
    for outline in opml.body.outlines {
        if let Some(xml_url) = outline.xml_url {
            let title = outline.text;
            let slug = title.to_lowercase().replace(' ', "_").replace('-', "_");
            let feed = FeedInfo {
                url: xml_url,
                author: title.clone(),
                tier: tier.clone(),
            };
            println!("Added feed: {} -> {}", slug, title);
            config.insert_feed(slug, feed);
        }
    }
    
    Ok(())
}