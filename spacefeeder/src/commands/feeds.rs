use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use crate::config::Config;
use crate::Tier;
use crate::defaults::get_default_feeds;

#[derive(Parser)]
pub struct FeedsArgs {
    #[command(subcommand)]
    pub command: FeedsCommands,
    
    #[arg(long, default_value = "spacefeeder.toml")]
    pub config_path: String,
}

#[derive(Subcommand)]
pub enum FeedsCommands {
    Search(SearchArgs),
    Add(AddArgs),
    List(ListArgs),
    Info(InfoArgs),
    Configure(ConfigureArgs),
    Remove(RemoveArgs),
}

#[derive(Parser)]
pub struct SearchArgs {
    /// Search query (searches in name, author, description, and tags)
    pub query: String,
    
    /// Filter by tag
    #[arg(long)]
    pub tag: Option<String>,
}

#[derive(Parser)]
pub struct AddArgs {
    /// Feed slug to add
    pub slug: String,
    
    /// Tier for the feed (new, like, love)
    #[arg(long)]
    pub tier: Option<String>,
}

#[derive(Parser)]
pub struct ListArgs {
    /// Filter by tier
    #[arg(long)]
    pub tier: Option<String>,
}

#[derive(Parser)]
pub struct InfoArgs {
    /// Feed slug to show info for
    pub slug: String,
}

#[derive(Parser)]
pub struct ConfigureArgs {
    /// Feed slug to configure
    pub slug: String,
    
    /// Set tier (new, like, love)
    #[arg(long)]
    pub tier: Option<String>,
    
    /// Add tags (comma-separated, prefix with + to add)
    #[arg(long)]
    pub tags: Option<String>,
}

#[derive(Parser)]
pub struct RemoveArgs {
    /// Feed slug to remove
    pub slug: String,
}

pub fn execute(args: FeedsArgs) -> Result<()> {
    match args.command {
        FeedsCommands::Search(search_args) => search(search_args),
        FeedsCommands::Add(add_args) => add(add_args, &args.config_path),
        FeedsCommands::List(list_args) => list(list_args, &args.config_path),
        FeedsCommands::Info(info_args) => info(info_args),
        FeedsCommands::Configure(configure_args) => configure(configure_args, &args.config_path),
        FeedsCommands::Remove(remove_args) => remove(remove_args, &args.config_path),
    }
}

fn search(args: SearchArgs) -> Result<()> {
    let default_feeds = get_default_feeds();
    let query = args.query.to_lowercase();
    
    let mut matches = Vec::new();
    
    for (slug, feed) in default_feeds.iter() {
        let mut score = 0;
        let mut match_reasons = Vec::new();
        
        // Search in slug (highest priority)
        if slug.to_lowercase().contains(&query) {
            score += 10;
            match_reasons.push("name");
        }
        
        // Search in author
        if feed.author.to_lowercase().contains(&query) {
            score += 5;
            match_reasons.push("author");
        }
        
        // Search in description
        if let Some(desc) = &feed.description {
            if desc.to_lowercase().contains(&query) {
                score += 3;
                match_reasons.push("description");
            }
        }
        
        // Search in tags
        if let Some(tags) = &feed.tags {
            for tag in tags {
                if tag.to_lowercase().contains(&query) {
                    score += 7;
                    match_reasons.push("tags");
                    break;
                }
            }
        }
        
        // Filter by specific tag if requested
        if let Some(filter_tag) = &args.tag {
            if let Some(tags) = &feed.tags {
                if !tags.iter().any(|tag| tag.to_lowercase() == filter_tag.to_lowercase()) {
                    continue;
                }
            } else {
                continue;
            }
        }
        
        if score > 0 {
            matches.push((slug, feed, score, match_reasons));
        }
    }
    
    if matches.is_empty() {
        println!("No feeds found matching '{}'", args.query);
        return Ok(());
    }
    
    // Sort by score (highest first)
    matches.sort_by(|a, b| b.2.cmp(&a.2));
    
    println!("Found {} feed(s) matching '{}':\n", matches.len(), args.query);
    
    for (slug, feed, _score, reasons) in matches {
        println!("{}", slug);
        println!("  Author: {}", feed.author);
        if let Some(desc) = &feed.description {
            println!("  Description: {}", desc);
        }
        if let Some(tags) = &feed.tags {
            println!("  Tags: {}", tags.join(", "));
        }
        println!("  URL: {}", feed.url);
        println!("  Matched: {}", reasons.join(", "));
        println!();
    }
    
    Ok(())
}

fn add(args: AddArgs, config_path: &str) -> Result<()> {
    let default_feeds = get_default_feeds();
    
    // Check if feed exists in default registry
    let default_feed = default_feeds.get(&args.slug)
        .ok_or_else(|| anyhow!("Feed '{}' not found in registry. Use 'spacefeeder feeds search' to find available feeds.", args.slug))?;
    
    // Parse tier
    let tier = if let Some(tier_str) = &args.tier {
        match tier_str.to_lowercase().as_str() {
            "new" => Tier::New,
            "like" => Tier::Like,
            "love" => Tier::Love,
            _ => return Err(anyhow!("Invalid tier '{}'. Use: new, like, love", tier_str)),
        }
    } else {
        Tier::New
    };
    
    // Load existing config
    let mut config = Config::from_file(config_path)?;
    
    // Check if feed already exists
    if config.feeds.contains_key(&args.slug) {
        println!("Feed '{}' is already configured. Use 'spacefeeder feeds configure' to modify it.", args.slug);
        return Ok(());
    }
    
    // Add feed to config
    let mut feed_info = default_feed.clone();
    feed_info.tier = tier;
    config.feeds.insert(args.slug.clone(), feed_info);
    
    // Save config
    config.save(config_path)?;
    
    println!("Added feed '{}' with tier '{}'", args.slug, tier);
    println!("  Author: {}", default_feed.author);
    if let Some(desc) = &default_feed.description {
        println!("  Description: {}", desc);
    }
    
    Ok(())
}

fn list(args: ListArgs, config_path: &str) -> Result<()> {
    let config = Config::from_file(config_path)?;
    
    // Filter by tier if specified
    let feeds: Vec<_> = if let Some(tier_str) = &args.tier {
        let filter_tier = match tier_str.to_lowercase().as_str() {
            "new" => Tier::New,
            "like" => Tier::Like,
            "love" => Tier::Love,
            _ => return Err(anyhow!("Invalid tier '{}'. Use: new, like, love", tier_str)),
        };
        
        config.feeds.iter()
            .filter(|(_, feed)| feed.tier == filter_tier)
            .collect()
    } else {
        config.feeds.iter().collect()
    };
    
    if feeds.is_empty() {
        if let Some(tier) = &args.tier {
            println!("No feeds found with tier '{}'", tier);
        } else {
            println!("No feeds configured");
        }
        return Ok(());
    }
    
    println!("Configured feeds:\n");
    
    // Group by tier
    let mut by_tier: HashMap<Tier, Vec<_>> = HashMap::new();
    for (slug, feed) in feeds {
        by_tier.entry(feed.tier.clone()).or_default().push((slug, feed));
    }
    
    // Display in order: Love, Like, New
    for tier in [Tier::Love, Tier::Like, Tier::New] {
        if let Some(feeds) = by_tier.get(&tier) {
            println!("{} ({}):", tier, feeds.len());
            for (slug, feed) in feeds {
                println!("  {} - {}", slug, feed.author);
                if let Some(desc) = &feed.description {
                    println!("    {}", desc);
                }
            }
            println!();
        }
    }
    
    Ok(())
}

fn info(args: InfoArgs) -> Result<()> {
    let default_feeds = get_default_feeds();
    
    let feed = default_feeds.get(&args.slug)
        .ok_or_else(|| anyhow!("Feed '{}' not found in registry", args.slug))?;
    
    println!("{}", args.slug);
    println!("  Author: {}", feed.author);
    if let Some(desc) = &feed.description {
        println!("  Description: {}", desc);
    }
    if let Some(tags) = &feed.tags {
        println!("  Tags: {}", tags.join(", "));
    }
    println!("  URL: {}", feed.url);
    
    Ok(())
}

fn configure(args: ConfigureArgs, config_path: &str) -> Result<()> {
    let mut config = Config::from_file(config_path)?;
    
    // Check if feed exists in config
    let feed = config.feeds.get_mut(&args.slug)
        .ok_or_else(|| anyhow!("Feed '{}' not found in configuration. Use 'spacefeeder feeds add' first.", args.slug))?;
    
    let mut changes = Vec::new();
    
    // Update tier if specified
    if let Some(tier_str) = &args.tier {
        let new_tier = match tier_str.to_lowercase().as_str() {
            "new" => Tier::New,
            "like" => Tier::Like,
            "love" => Tier::Love,
            _ => return Err(anyhow!("Invalid tier '{}'. Use: new, like, love", tier_str)),
        };
        
        if feed.tier != new_tier {
            feed.tier = new_tier.clone();
            changes.push(format!("tier -> {}", new_tier));
        }
    }
    
    // Update tags if specified (simplified implementation for now)
    if let Some(_tags_str) = &args.tags {
        changes.push("tags updated".to_string());
        // TODO: Implement tag modification logic
        println!("Note: Tag modification not yet implemented");
    }
    
    if changes.is_empty() {
        println!("No changes made to feed '{}'", args.slug);
        return Ok(());
    }
    
    // Save config
    config.save(config_path)?;
    
    println!("Updated feed '{}': {}", args.slug, changes.join(", "));
    
    Ok(())
}

fn remove(args: RemoveArgs, config_path: &str) -> Result<()> {
    let mut config = Config::from_file(config_path)?;
    
    // Check if feed exists
    if !config.feeds.contains_key(&args.slug) {
        return Err(anyhow!("Feed '{}' not found in configuration", args.slug));
    }
    
    // Remove feed
    let removed_feed = config.feeds.remove(&args.slug).unwrap();
    
    // Save config
    config.save(config_path)?;
    
    println!("Removed feed '{}' ({})", args.slug, removed_feed.author);
    
    Ok(())
}