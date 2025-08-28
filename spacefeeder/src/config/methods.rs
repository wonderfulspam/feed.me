use super::{types::Config, ConfigSaver};
use crate::FeedInfo;
use anyhow::Result;

impl Config {
    pub(crate) fn insert_feed(&mut self, slug: String, feed: FeedInfo) {
        let _ = self.feeds.insert(slug, feed);
    }

    pub fn base_url(&self) -> &str {
        &self.output_config.base_url
    }

    pub fn save(&self, config_path: &str) -> Result<()> {
        ConfigSaver::save_to_file(self, config_path)
    }
}
