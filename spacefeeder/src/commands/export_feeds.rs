use crate::config::Config;
use anyhow::Result;
use opml::OPML;

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
