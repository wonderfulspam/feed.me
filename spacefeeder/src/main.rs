use anyhow::Result;
use clap::{arg, command, Parser, Subcommand};
use spacefeeder::{
    commands::{fetch_feeds, find_feed},
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
    Fetch {
        /// Path to the config file
        #[arg(long, default_value = "./spacefeeder.toml")]
        config_path: String,
    },
    FindFeed {
        #[arg(long)]
        base_url: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Fetch { config_path } => {
            let config = config::Config::from_file(&config_path)?;
            fetch_feeds::run(config)
        }
        Commands::FindFeed { base_url } => {
            let url_match = find_feed::run(&base_url)?;
            println!("{url_match}");
            Ok(())
        }
    }
}
