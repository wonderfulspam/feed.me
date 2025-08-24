use anyhow::Result;
use clap::{Parser, Subcommand};
use spacefeeder::config;
use spacefeeder::commands::{
    add_feed::{self, AddFeedArgs},
    build::{self, BuildArgs},
    export_feeds::{self, ExportArgs},
    fetch_feeds::{self, FetchArgs},
    find_feed::{self, FindFeedArgs},
    import_feeds::{self, ImportArgs},
    init::{self, InitArgs},
    search::{self, SearchArgs},
    serve::{self, ServeArgs},
};

#[derive(Parser)]
#[command(name = "Space Feeder", about = "Processes RSS and Atom feeds")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    AddFeed(AddFeedArgs),
    Build(BuildArgs),
    Fetch(FetchArgs),
    FindFeed(FindFeedArgs),
    Export(ExportArgs),
    Import(ImportArgs),
    Init(InitArgs),
    Search(SearchArgs),
    Serve(ServeArgs),
}

fn get_config_path_if_needed(command: &Commands) -> Option<&str> {
    match command {
        Commands::FindFeed(_) | Commands::Init(_) | Commands::Search(_) => None,
        Commands::AddFeed(args) => Some(&args.config_path),
        Commands::Build(args) => Some(&args.config_path),
        Commands::Export(args) => Some(&args.config_path),
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
        Commands::Fetch(args) => fetch_feeds::execute(args),
        Commands::FindFeed(args) => find_feed::execute(args),
        Commands::Import(args) => import_feeds::execute(args),
        Commands::Init(args) => init::execute(args),
        Commands::Search(args) => search::execute(args),
        Commands::Serve(args) => serve::execute(args),
    }
}
