use serde::Deserialize;
use crate::config::TagDefinition;

// Embed default tags at compile time
const DEFAULT_TAGS_TOML: &str = include_str!("../../data/tags.toml");

#[derive(Debug, Deserialize)]
struct DefaultTags {
    tags: Vec<TagDefinition>,
}

pub fn get_default_tags() -> Vec<TagDefinition> {
    let default_tags: DefaultTags = toml_edit::de::from_str(DEFAULT_TAGS_TOML)
        .expect("Failed to parse embedded tags.toml");
    default_tags.tags
}