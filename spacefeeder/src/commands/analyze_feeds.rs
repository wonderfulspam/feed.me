use anyhow::Result;
use clap::Args;
use feed_rs::{model::Feed, parser};
use regex::Regex;
use reqwest;
use std::collections::HashMap;
use tokio::time::{timeout, Duration};

use crate::config::Config;
use crate::defaults::get_default_feeds;

#[derive(Debug, Args)]
pub struct AnalyzeArgs {
    #[arg(
        short,
        long,
        default_value = "spacefeeder.toml",
        help = "Path to configuration file"
    )]
    pub config_path: String,
}

#[derive(Debug)]
struct FeedAnalysis {
    slug: String,
    entry_count: usize,
    date_range: Option<(String, String)>,
    error: Option<String>,
}

async fn fetch_and_parse_feed(url: &str, slug: &str, _author: &str) -> FeedAnalysis {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    match timeout(Duration::from_secs(30), client.get(url).send()).await {
        Ok(Ok(response)) => match response.text().await {
            Ok(content) => {
                // Try to parse with feed_rs (same as spacefeeder uses)
                match parser::parse(content.as_bytes()) {
                    Ok(feed) => analyze_feed_content(slug, feed),
                    Err(e) => FeedAnalysis {
                        slug: slug.to_string(),
                        entry_count: 0,
                        date_range: None,
                        error: Some(format!("parse error: {}", e)),
                    },
                }
            }
            Err(e) => FeedAnalysis {
                slug: slug.to_string(),
                entry_count: 0,
                date_range: None,
                error: Some(format!("read error: {}", e)),
            },
        },
        Ok(Err(e)) => FeedAnalysis {
            slug: slug.to_string(),
            entry_count: 0,
            date_range: None,
            error: Some(format!("request error: {}", e)),
        },
        Err(_) => FeedAnalysis {
            slug: slug.to_string(),
            entry_count: 0,
            date_range: None,
            error: Some("timeout".to_string()),
        },
    }
}

fn analyze_feed_content(slug: &str, feed: Feed) -> FeedAnalysis {
    let entry_count = feed.entries.len();
    
    // Extract years from entry dates
    let year_regex = Regex::new(r"(20\d{2}|19\d{2})").unwrap();
    let mut years: Vec<i32> = feed
        .entries
        .iter()
        .filter_map(|entry| {
            let date_str = entry.published
                .or(entry.updated)
                .map(|dt| dt.to_string())
                .unwrap_or_default();
            
            year_regex
                .find(&date_str)
                .and_then(|m| m.as_str().parse::<i32>().ok())
        })
        .collect();
    
    years.sort_unstable();
    years.dedup();
    
    let date_range = if !years.is_empty() {
        let oldest = years.first().unwrap();
        let newest = years.last().unwrap();
        Some((oldest.to_string(), newest.to_string()))
    } else {
        None
    };

    FeedAnalysis {
        slug: slug.to_string(),
        entry_count,
        date_range,
        error: None,
    }
}


fn collect_feeds_for_analysis(config: &Config) -> Result<HashMap<String, (String, String)>> {
    let mut feeds = HashMap::new();
    
    // Load built-in feeds
    let default_feeds = get_default_feeds();
    for (slug, info) in default_feeds {
        feeds.insert(slug, (info.url, info.author));
    }
    
    // Override with user feeds
    for (slug, info) in &config.feeds {
        let url = info.url.clone();
        let author = info.author.clone();
        feeds.insert(slug.clone(), (url, author));
    }
    
    Ok(feeds)
}

pub async fn execute(args: AnalyzeArgs) -> Result<()> {
    let config = Config::from_file(&args.config_path)?;
    let feeds = collect_feeds_for_analysis(&config)?;
    
    println!("Found {} feeds to analyze", feeds.len());
    println!("{}", "=".repeat(80));
    
    let mut results = Vec::new();
    
    for (i, (slug, (url, author))) in feeds.iter().enumerate() {
        println!("\n🔍 Analyzing {} ({}/{}) - {}", slug, i + 1, feeds.len(), author);
        
        let result = fetch_and_parse_feed(url, slug, author).await;
        
        if let Some(error) = &result.error {
            println!("   ❌ {}", error);
        } else {
            let range_str = result.date_range
                .as_ref()
                .map(|(oldest, newest)| format!("{}-{}", oldest, newest))
                .unwrap_or_else(|| "unknown".to_string());
            println!("   📊 {} entries, {}", result.entry_count, range_str);
        }
        
        results.push(result);
    }
    
    // Summary analysis
    println!("\n{}", "=".repeat(80));
    println!("📊 SUMMARY ANALYSIS");
    println!("{}", "=".repeat(80));
    
    let working_feeds: Vec<_> = results.iter().filter(|r| r.error.is_none()).collect();
    let error_feeds: Vec<_> = results.iter().filter(|r| r.error.is_some()).collect();
    
    let total_entries: usize = working_feeds.iter().map(|r| r.entry_count).sum();
    
    println!("📈 Total entries across all feeds: {}", total_entries);
    println!("✅ Working feeds: {}", working_feeds.len());
    println!("❌ Error feeds: {}", error_feeds.len());
    
    if !working_feeds.is_empty() {
        let entry_counts: Vec<_> = working_feeds.iter().map(|r| r.entry_count).collect();
        let min_entries = entry_counts.iter().min().unwrap();
        let max_entries = entry_counts.iter().max().unwrap();
        let avg_entries = total_entries as f64 / working_feeds.len() as f64;
        
        println!("📊 Entry count range: {} - {}", min_entries, max_entries);
        println!("📊 Average entries per feed: {:.1}", avg_entries);
        
        // Find feeds with limited content
        let limited_feeds: Vec<_> = working_feeds.iter().filter(|r| r.entry_count < 30).collect();
        if !limited_feeds.is_empty() {
            println!("\n⚠️  Feeds with limited content (<30 entries):");
            for feed in limited_feeds {
                println!("   • {}: {} entries", feed.slug, feed.entry_count);
            }
        }
        
        // Find feeds with good historical coverage
        let historical_feeds: Vec<_> = working_feeds.iter().filter(|r| r.entry_count > 100).collect();
        if !historical_feeds.is_empty() {
            println!("\n✅ Feeds with good historical coverage (>100 entries):");
            for feed in historical_feeds {
                let range_str = feed.date_range
                    .as_ref()
                    .map(|(oldest, newest)| format!(" ({}-{})", oldest, newest))
                    .unwrap_or_default();
                println!("   • {}: {} entries{}", feed.slug, feed.entry_count, range_str);
            }
        }
    }
    
    if !error_feeds.is_empty() {
        println!("\n❌ Feeds with errors:");
        for feed in error_feeds {
            println!("   • {}: {}", feed.slug, feed.error.as_ref().unwrap());
        }
    }
    
    Ok(())
}