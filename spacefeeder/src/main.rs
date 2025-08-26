use anyhow::Result;
use clap::{Parser, Subcommand};
use spacefeeder::commands::{
    add_feed::{self, AddFeedArgs},
    build::{self, BuildArgs},
    export_feeds::{self, ExportArgs},
    feeds::{self, FeedsArgs},
    fetch_feeds::{self, FetchArgs},
    find_feed::{self, FindFeedArgs},
    import_feeds::{self, ImportArgs},
    init::{self, InitArgs},
    search::{self, SearchArgs},
    serve::{self, ServeArgs},
};
use spacefeeder::config;

#[derive(Parser)]
#[command(name = "Space Feeder", about = "Processes RSS and Atom feeds")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a single feed to the configuration
    AddFeed(AddFeedArgs),
    /// Fetch feeds and generate complete static site
    Build(BuildArgs),
    /// Package manager-like commands for feed discovery and management
    Feeds(FeedsArgs),
    /// Fetch feeds and update JSON data without building site
    Fetch(FetchArgs),
    /// Find RSS/Atom feed from a website URL
    FindFeed(FindFeedArgs),
    /// Export feeds to OPML format
    Export(ExportArgs),
    /// Import feeds from OPML file
    Import(ImportArgs),
    /// Initialize a new configuration file
    Init(InitArgs),
    /// Search and build search index for articles
    Search(SearchArgs),
    /// Start development server for the generated site
    Serve(ServeArgs),
}

fn get_config_path_if_needed(command: &Commands) -> Option<&str> {
    match command {
        Commands::FindFeed(_) | Commands::Init(_) | Commands::Search(_) => None,
        Commands::AddFeed(args) => Some(&args.config_path),
        Commands::Build(args) => Some(&args.config_path),
        Commands::Export(args) => Some(&args.config_path),
        Commands::Feeds(args) => {
            // Only need config for commands that modify or read user config
            match args.command {
                feeds::FeedsCommands::Search(_) | feeds::FeedsCommands::Info(_) => None,
                _ => Some(&args.config_path),
            }
        }
        Commands::Fetch(args) => Some(&args.config_path),
        Commands::Import(args) => Some(&args.config_path),
        Commands::Serve(args) => Some(&args.config_path),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize config if needed
    if let Some(config_path) = get_config_path_if_needed(&cli.command) {
        config::init_config(config_path)?;
    }

    // Execute the command
    match cli.command {
        Commands::AddFeed(args) => add_feed::execute(args),
        Commands::Build(args) => build::execute(args),
        Commands::Export(args) => export_feeds::execute(args),
        Commands::Feeds(args) => feeds::execute(args),
        Commands::Fetch(args) => fetch_feeds::execute(args),
        Commands::FindFeed(args) => find_feed::execute(args),
        Commands::Import(args) => import_feeds::execute(args),
        Commands::Init(args) => init::execute(args),
        Commands::Search(args) => search::execute(args),
        Commands::Serve(args) => serve::execute(args),
    }
}
