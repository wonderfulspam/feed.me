#[cfg(test)]
mod tests {
    use super::super::types::Config;
    use crate::{FeedInfo, Tier};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.parse_config.max_articles, 5);
        assert_eq!(config.parse_config.description_max_words, 150);
        assert_eq!(
            config.output_config.feed_data_output_path,
            "./content/data/feedData.json"
        );
        assert_eq!(
            config.output_config.item_data_output_path,
            "./content/data/itemData.json"
        );
        assert_eq!(config.feeds.len(), 1);
        assert!(config.feeds.contains_key("example"));
    }

    #[test]
    fn test_insert_feed() {
        let mut config = Config::default();
        let feed = FeedInfo {
            url: "https://test.com/feed".to_string(),
            author: "Test Author".to_string(),
            description: Some("Test feed".to_string()),
            tier: Tier::Love,
            tags: None,
            auto_tag: None,
        };
        config.insert_feed("test_feed".to_string(), feed);
        assert_eq!(config.feeds.len(), 2);
        assert!(config.feeds.contains_key("test_feed"));
    }

    #[test]
    fn test_config_from_file() {
        let toml_content = r#"
max_articles = 10
description_max_words = 200

[feeds.test_feed]
url = "https://example.com/feed"
author = "Test Author"
tier = "love"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.parse_config.max_articles, 10);
        assert_eq!(config.parse_config.description_max_words, 200);
        assert_eq!(config.feeds.len(), 1);
        assert!(config.feeds.contains_key("test_feed"));

        let feed = &config.feeds["test_feed"];
        assert_eq!(feed.url, "https://example.com/feed");
        assert_eq!(feed.author, "Test Author");
        assert!(matches!(feed.tier, Tier::Love));
    }

    #[test]
    fn test_config_save() {
        let config = Config::default();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        config.save(path).unwrap();

        // Read back and verify
        let loaded_config = Config::from_file(path).unwrap();
        assert_eq!(
            loaded_config.parse_config.max_articles,
            config.parse_config.max_articles
        );
        assert_eq!(
            loaded_config.parse_config.description_max_words,
            config.parse_config.description_max_words
        );
        assert_eq!(loaded_config.feeds.len(), config.feeds.len());
    }

    #[test]
    fn test_default_tags_loaded() {
        let toml_content = r#"
max_articles = 10
description_max_words = 200

[categorization]
enabled = true

[feeds.test_feed]
url = "https://example.com/feed"
author = "Test Author"
tier = "love"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();

        // Verify default tags are loaded
        assert!(
            !config.categorization.tags.is_empty(),
            "Should have loaded default tags"
        );

        let tag_names: Vec<String> = config
            .categorization
            .tags
            .iter()
            .map(|t| t.name.clone())
            .collect();

        // Check for some expected default tags
        assert!(
            tag_names.contains(&"rust".to_string()),
            "Should contain rust tag"
        );
        assert!(
            tag_names.contains(&"ai".to_string()),
            "Should contain ai tag"
        );
        assert!(
            tag_names.contains(&"python".to_string()),
            "Should contain python tag"
        );
        assert!(
            tag_names.contains(&"devops".to_string()),
            "Should contain devops tag"
        );
    }

    #[test]
    fn test_default_output_paths() {
        let config = Config::default();
        assert_eq!(
            config.output_config.feed_data_output_path,
            "./content/data/feedData.json"
        );
        assert_eq!(
            config.output_config.item_data_output_path,
            "./content/data/itemData.json"
        );
    }

    #[test]
    fn test_user_tags_override_defaults() {
        let toml_content = r#"
max_articles = 10
description_max_words = 200

[categorization]
enabled = true

[[categorization.tags]]
name = "rust"
description = "Custom Rust description"
keywords = ["custom", "rust"]

[feeds.test_feed]
url = "https://example.com/feed"
author = "Test Author"
tier = "love"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();

        // Find the rust tag
        let rust_tag = config
            .categorization
            .tags
            .iter()
            .find(|t| t.name == "rust")
            .expect("Should have rust tag");

        // Verify user version is used, not default
        assert_eq!(rust_tag.description, "Custom Rust description");
        assert_eq!(rust_tag.keywords, vec!["custom", "rust"]);

        // Verify other default tags are still loaded
        let tag_names: Vec<String> = config
            .categorization
            .tags
            .iter()
            .map(|t| t.name.clone())
            .collect();
        assert!(
            tag_names.contains(&"ai".to_string()),
            "Should still have other default tags"
        );
    }
}
