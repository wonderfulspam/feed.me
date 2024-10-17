use anyhow::Result;
use clap::{arg, command, Parser, Subcommand};
use spacefeeder::{
    commands::{add_feed, export_feeds, fetch_feeds, find_feed, import_feeds},
    config,
};

#[derive(Parser)]
#[command(name = "Space Feeder", about = "Processes RSS and Atom feeds")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    AddFeed {
        #[arg(long)]
        slug: String,
        #[arg(long)]
        url: String,
        #[arg(long)]
        author: String,
        #[arg(long)]
        tier: String,
        #[arg(long, default_value = "./spacefeeder.toml")]
        config_path: String,
    },
    Fetch {
        /// Path to the config file
        #[arg(long, default_value = "./spacefeeder.toml")]
        config_path: String,
    },
    FindFeed {
        #[arg(long)]
        base_url: String,
    },
    Export {
        /// Path to the config file
        #[arg(long, default_value = "./spacefeeder.toml")]
        config_path: String,
        #[arg(long, default_value = "./spacefeeder_export.opml")]
        output_path: String,
    },
    Import {
        /// Path to the OPML file to import
        #[arg(long)]
        input_path: String,
        /// Default tier for imported feeds
        #[arg(long, default_value = "new")]
        tier: String,
        /// Path to the config file
        #[arg(long, default_value = "./spacefeeder.toml")]
        config_path: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::AddFeed {
            slug,
            url,
            author,
            tier,
            config_path,
        } => {
            let mut config = config::Config::from_file(&config_path)?;
            add_feed::run(&mut config, slug, url, author, tier)?;
            config.save(&config_path)
        }
        Commands::Fetch { config_path } => {
            let config = config::Config::from_file(&config_path)?;
            fetch_feeds::run(config)
        }
        Commands::FindFeed { base_url } => {
            let url_match = find_feed::run(&base_url)?;
            println!("{url_match}");
            Ok(())
        }
        Commands::Export {
            config_path,
            output_path,
        } => {
            let config = config::Config::from_file(&config_path)?;
            export_feeds::run(config, output_path)
        }
        Commands::Import {
            input_path,
            tier,
            config_path,
        } => {
            let mut config = config::Config::from_file(&config_path)?;
            import_feeds::run(&mut config, input_path, tier)?;
            config.save(&config_path)
        }
    }
}
