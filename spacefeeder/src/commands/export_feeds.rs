use crate::config::Config;
use anyhow::Result;
use clap::Args;
use opml::OPML;

#[derive(Args)]
pub struct ExportArgs {
    /// Path to the config file
    #[arg(long, default_value = "./spacefeeder.toml")]
    pub config_path: String,
    #[arg(long, default_value = "./spacefeeder_export.opml")]
    pub output_path: String,
}

pub fn execute(args: ExportArgs) -> Result<()> {
    let config = Config::from_file(&args.config_path)?;
    run(config, args.output_path)
}

pub fn run(config: Config, output_path: String) -> Result<()> {
    let feeds = config.feeds;
    let mut opml = OPML::default();
    for (title, feed) in feeds {
        opml.add_feed(&title, &feed.url);
    }
    let output = opml.to_string()?;
    std::fs::write(output_path, output)?;
    Ok(())
}
