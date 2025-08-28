use anyhow::{Context, Result};
use std::collections::HashMap;

use super::{
    types::{Config, ParsedConfig},
    ConfigMerger,
};

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {path}"))?;
        let parsed_config: ParsedConfig = toml_edit::de::from_str(&content)
            .with_context(|| format!("Failed to parse TOML from file: {path}"))?;

        let mut config = Config {
            parse_config: parsed_config.parse_config,
            output_config: parsed_config.output_config,
            categorization: parsed_config.categorization,
            feeds: HashMap::new(),
        };

        // Use dedicated merger for complex configuration merging
        ConfigMerger::merge_tags(&mut config.categorization);
        config.feeds = ConfigMerger::merge_feeds(parsed_config.feeds)?;
        ConfigMerger::merge_categorization(&mut config.categorization);

        Ok(config)
    }
}
