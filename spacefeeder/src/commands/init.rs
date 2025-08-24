use anyhow::{Context, Result};
use clap::Args;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::{FeedInfo, Tier};

#[derive(Args)]
pub struct InitArgs {
    /// Path to create the config file (defaults to spacefeeder.toml in current directory)
    #[arg(short, long)]
    pub config: Option<String>,

    /// Create config in user's home config directory instead
    #[arg(long)]
    pub global: bool,

    /// Force overwrite existing config file
    #[arg(short, long)]
    pub force: bool,

    /// Skip interactive wizard
    #[arg(short, long)]
    pub quiet: bool,
}

pub fn execute(args: InitArgs) -> Result<()> {
    let config_path = determine_config_path(&args)?;

    // Check if config already exists
    if Path::new(&config_path).exists() && !args.force {
        println!("Configuration file already exists at: {}", config_path);
        println!("Use --force to overwrite or specify a different path with --config");
        return Ok(());
    }

    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(&config_path).parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let mut config = create_starter_config();

    if !args.quiet {
        println!("🚀 Welcome to SpaceFeeder!");
        println!("Let's set up your personal RSS feed reader.\n");
        
        config = run_interactive_setup(config)?;
    }

    config.save(&config_path)?;

    println!("\n✅ Configuration created successfully!");
    println!("📄 Config file: {}", config_path);
    println!("\n🎯 Next steps:");
    println!("  1. Run 'spacefeeder fetch' to download your feeds");
    println!("  2. Run 'just build' to build your feed reader");
    println!("  3. Run 'just serve' to start the development server");
    
    if !args.quiet {
        println!("\n💡 Tips:");
        println!("  • Add more feeds: spacefeeder add-feed <slug> <url> <author> <tier>");
        println!("  • Find feeds: spacefeeder find-feed <website-url>");
        println!("  • Export to OPML: spacefeeder export");
    }

    Ok(())
}

fn determine_config_path(args: &InitArgs) -> Result<String> {
    if let Some(path) = &args.config {
        return Ok(path.clone());
    }

    if args.global {
        let home = dirs::home_dir()
            .context("Could not determine home directory")?;
        let config_dir = home.join(".config").join("feed.me");
        return Ok(config_dir.join("spacefeeder.toml").to_string_lossy().to_string());
    }

    Ok("spacefeeder.toml".to_string())
}

fn create_starter_config() -> Config {
    let mut config = Config::default();
    
    // Clear the example feed and add better starter feeds
    config.feeds.clear();
    
    // Add curated starter feeds
    let starter_feeds = vec![
        ("rust-blog", "https://blog.rust-lang.org/feed.xml", "Rust Team", Tier::Love),
        ("github-blog", "https://github.blog/feed/", "GitHub", Tier::Like),
        ("hacker-news", "https://hnrss.org/frontpage", "Hacker News", Tier::New),
        ("dev-to", "https://dev.to/feed", "DEV Community", Tier::New),
        ("lobsters", "https://lobste.rs/rss", "Lobsters", Tier::Like),
    ];

    for (slug, url, author, tier) in starter_feeds {
        config.insert_feed(slug.to_string(), FeedInfo {
            url: url.to_string(),
            author: author.to_string(),
            tier,
            tags: None,
            auto_tag: None,
        });
    }

    // Set reasonable defaults
    config.parse_config.max_articles = 50;
    config.parse_config.description_max_words = 150;

    config
}

fn run_interactive_setup(mut config: Config) -> Result<Config> {
    println!("📝 I've included some popular tech feeds to get you started:");
    
    for (slug, feed) in &config.feeds {
        println!("  • {} - {} ({:?} tier)", slug, feed.author, feed.tier);
    }

    println!("\n🔧 Configuration options:");
    
    // Ask about max articles
    print!("📄 Maximum articles per feed [{}]: ", config.parse_config.max_articles);
    std::io::Write::flush(&mut std::io::stdout())?;
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if let Ok(max_articles) = input.trim().parse::<usize>() {
        if max_articles > 0 {
            config.parse_config.max_articles = max_articles;
        }
    }

    // Ask about description length
    print!("✂️  Maximum description words [{}]: ", config.parse_config.description_max_words);
    std::io::Write::flush(&mut std::io::stdout())?;
    
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    
    if let Ok(desc_words) = input.trim().parse::<usize>() {
        if desc_words > 0 {
            config.parse_config.description_max_words = desc_words;
        }
    }

    println!("\n🎨 Feed tiers help organize your reading:");
    println!("  • Love: Your absolute favorites (shown prominently)");
    println!("  • Like: Regular reads (balanced display)");
    println!("  • New: Testing/discovering (compact display)");

    Ok(config)
}