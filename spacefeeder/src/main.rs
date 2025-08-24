use anyhow::Result;
use clap::{Parser, Subcommand};
use spacefeeder::commands::{
    add_feed::{self, AddFeedArgs},
    export_feeds::{self, ExportArgs},
    fetch_feeds::{self, FetchArgs},
    find_feed::{self, FindFeedArgs},
    import_feeds::{self, ImportArgs},
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
    Fetch(FetchArgs),
    FindFeed(FindFeedArgs),
    Export(ExportArgs),
    Import(ImportArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::AddFeed(args) => add_feed::execute(args),
        Commands::Fetch(args) => fetch_feeds::execute(args),
        Commands::FindFeed(args) => find_feed::execute(args),
        Commands::Export(args) => export_feeds::execute(args),
        Commands::Import(args) => import_feeds::execute(args),
    }
}
