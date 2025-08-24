use anyhow::{anyhow, Result};
use clap::Args;
use crate::search::SearchIndex;

#[derive(Args)]
pub struct SearchArgs {
    /// Search query
    pub query: String,
    
    /// Filter by author (partial match, case-insensitive)
    #[arg(long)]
    pub author: Option<String>,
    
    /// Filter by tier (new, like, love)
    #[arg(long)]
    pub tier: Option<String>,
    
    /// Maximum number of results to return
    #[arg(long, default_value = "10")]
    pub limit: usize,
}

pub fn execute(args: SearchArgs) -> Result<()> {
    let index_path = "./search_index";
    
    if !std::path::Path::new(index_path).exists() {
        return Err(anyhow!("Search index not found. Run 'spacefeeder fetch' first to build the index."));
    }
    
    let search_index = SearchIndex::open(index_path)?;
    
    let results = search_index.search_with_filters(
        &args.query,
        args.author.as_deref(),
        args.tier.as_deref(),
        args.limit,
    )?;
    
    if results.is_empty() {
        println!("No articles found matching your search criteria.");
        return Ok(());
    }
    
    println!("Found {} result{}:\n", results.len(), if results.len() == 1 { "" } else { "s" });
    
    for (i, result) in results.iter().enumerate() {
        println!("{}. {} (score: {:.2})", i + 1, result.title, result.score);
        println!("   Author: {} | Tier: {} | Date: {}", 
                 result.author, result.tier, result.pub_date.format("%Y-%m-%d"));
        println!("   URL: {}", result.item_url);
        
        // Show description preview (first 100 chars)
        let description = if result.description.len() > 100 {
            format!("{}...", &result.description[..100])
        } else {
            result.description.clone()
        };
        println!("   {}\n", description);
    }
    
    Ok(())
}