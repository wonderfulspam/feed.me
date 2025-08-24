use std::collections::{HashMap, HashSet};

use crate::config::{CategorizationConfig, TagRule};

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub confidence: f32,
    pub source: TagSource,
}

#[derive(Debug, Clone)]
pub enum TagSource {
    Manual,
    Feed,
    Rule,
    Keyword,
}

pub struct CategorizationEngine {
    config: CategorizationConfig,
    alias_map: HashMap<String, String>,
}

impl CategorizationEngine {
    pub fn from_config(config: &CategorizationConfig) -> Self {
        let mut alias_map = HashMap::new();
        for alias in &config.aliases {
            for from in &alias.from {
                alias_map.insert(from.to_lowercase(), alias.to.clone());
            }
        }

        Self {
            config: config.clone(),
            alias_map,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn generate_tags_for_item(
        &self,
        title: &str,
        description: Option<&str>,
        link: Option<&str>,
        author: Option<&str>,
        feed_slug: &str,
        feed_tags: Option<&[String]>,
        rss_categories: Option<&[String]>,
    ) -> Vec<Tag> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut tags = Vec::new();
        let mut seen = HashSet::new();

        // 1. Add manual feed tags
        if let Some(feed_tags) = feed_tags {
            for tag in feed_tags {
                let normalized = self.normalize_tag(tag);
                if seen.insert(normalized.clone()) {
                    tags.push(Tag {
                        name: normalized,
                        confidence: 1.0,
                        source: TagSource::Manual,
                    });
                }
            }
        }

        // 2. Add RSS/Atom category tags
        if let Some(categories) = rss_categories {
            for category in categories {
                let normalized = self.normalize_tag(category);
                if seen.insert(normalized.clone()) {
                    tags.push(Tag {
                        name: normalized,
                        confidence: 0.9,
                        source: TagSource::Feed,
                    });
                }
            }
        }

        // 3. Check for exclusion rules first
        let mut excluded_tags = HashSet::new();
        for rule in &self.config.rules {
            if rule.rule_type == "exclude_if" {
                let content = format!(
                    "{} {}",
                    title.to_lowercase(),
                    description.unwrap_or("").to_lowercase()
                );
                
                if rule.patterns.iter().any(|p| content.contains(&p.to_lowercase())) {
                    // If this exclude rule matches, mark its exclude_tags for exclusion
                    for tag in &rule.exclude_tags {
                        excluded_tags.insert(tag.clone());
                    }
                }
            }
        }

        // 4. Apply rule-based tagging (skipping exclude_if rules)
        for rule in &self.config.rules {
            if rule.rule_type != "exclude_if" {
                if let Some(matched_tags) = self.apply_rule(
                    rule,
                    title,
                    description,
                    link,
                    author,
                    feed_slug,
                ) {
                    for tag in matched_tags {
                        let normalized = self.normalize_tag(&tag.name);
                        // Skip excluded tags
                        if !excluded_tags.contains(&normalized) && seen.insert(normalized.clone()) {
                            tags.push(Tag {
                                name: normalized,
                                confidence: tag.confidence,
                                source: tag.source,
                            });
                        }
                    }
                }
            }
        }

        // 5. Apply keyword-based tagging
        if self.config.auto_tag_new_articles {
            let content = format!(
                "{} {}",
                title,
                description.unwrap_or("")
            );
            for tag_def in &self.config.tags {
                if let Some(confidence) = self.check_keywords(&content, &tag_def.keywords) {
                    if confidence >= self.config.confidence_threshold {
                        let normalized = self.normalize_tag(&tag_def.name);
                        // Skip excluded tags
                        if !excluded_tags.contains(&normalized) && seen.insert(normalized.clone()) {
                            tags.push(Tag {
                                name: normalized,
                                confidence,
                                source: TagSource::Keyword,
                            });
                        }
                    }
                }
            }
        }

        // Sort by confidence and limit
        tags.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        tags.truncate(self.config.max_tags_per_item);

        tags
    }

    fn apply_rule(
        &self,
        rule: &TagRule,
        title: &str,
        description: Option<&str>,
        link: Option<&str>,
        author: Option<&str>,
        feed_slug: &str,
    ) -> Option<Vec<Tag>> {
        // First check if any exclude patterns match - if so, skip this rule
        if !rule.exclude_patterns.is_empty() {
            let content = format!(
                "{} {}",
                title.to_lowercase(),
                description.unwrap_or("").to_lowercase()
            );
            
            for exclude_pattern in &rule.exclude_patterns {
                if content.contains(&exclude_pattern.to_lowercase()) {
                    return None; // Rule excluded
                }
            }
        }

        let matches = match rule.rule_type.as_str() {
            "title_contains" => {
                let title_lower = title.to_lowercase();
                rule.patterns.iter().any(|p| title_lower.contains(&p.to_lowercase()))
            }
            "content_contains" => {
                let content = format!(
                    "{} {}",
                    title.to_lowercase(),
                    description.unwrap_or("").to_lowercase()
                );
                rule.patterns.iter().any(|p| content.contains(&p.to_lowercase()))
            }
            "content_analysis" => {
                // Advanced content analysis with keyword count requirements
                let content = format!(
                    "{} {}",
                    title.to_lowercase(),
                    description.unwrap_or("").to_lowercase()
                );
                
                let matched_keywords = rule.patterns.iter()
                    .filter(|p| content.contains(&p.to_lowercase()))
                    .count();
                
                if let Some(min_count) = rule.min_keyword_count {
                    matched_keywords >= min_count
                } else {
                    matched_keywords > 0
                }
            }
            "author_with_content" => {
                // Author-based rule that also requires content keywords
                if let Some(author_str) = author {
                    let author_matches = rule.patterns.iter()
                        .any(|p| author_str.to_lowercase().contains(&p.to_lowercase()));
                    
                    if author_matches && !rule.required_keywords.is_empty() {
                        let content = format!(
                            "{} {}",
                            title.to_lowercase(),
                            description.unwrap_or("").to_lowercase()
                        );
                        
                        // All required keywords must be present
                        rule.required_keywords.iter()
                            .all(|kw| content.contains(&kw.to_lowercase()))
                    } else {
                        author_matches
                    }
                } else {
                    false
                }
            }
            "url_contains" => {
                if let Some(url) = link {
                    let url_lower = url.to_lowercase();
                    rule.patterns.iter().any(|p| url_lower.contains(&p.to_lowercase()))
                } else {
                    false
                }
            }
            "author_contains" => {
                if let Some(author_str) = author {
                    let author_lower = author_str.to_lowercase();
                    rule.patterns.iter().any(|p| author_lower.contains(&p.to_lowercase()))
                } else {
                    false
                }
            }
            "feed_slug" => {
                rule.patterns.iter().any(|p| feed_slug == p)
            }
            "exclude_if" => {
                // This is a negative rule - if it matches, it prevents other tags
                // This should be handled at a higher level, but we return false here
                false
            }
            _ => false,
        };

        if matches {
            let mut tags = Vec::new();
            
            // Handle single tag
            if !rule.tag.is_empty() {
                tags.push(Tag {
                    name: rule.tag.clone(),
                    confidence: rule.confidence,
                    source: TagSource::Rule,
                });
            }
            
            // Handle multiple tags
            for tag in &rule.tags {
                tags.push(Tag {
                    name: tag.clone(),
                    confidence: rule.confidence,
                    source: TagSource::Rule,
                });
            }
            
            if !tags.is_empty() {
                Some(tags)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn check_keywords(&self, content: &str, keywords: &[String]) -> Option<f32> {
        let content_lower = content.to_lowercase();
        let mut matches = 0;
        
        for keyword in keywords {
            if content_lower.contains(&keyword.to_lowercase()) {
                matches += 1;
            }
        }
        
        if matches > 0 {
            let confidence = (matches as f32) / (keywords.len() as f32).min(3.0);
            Some(confidence.min(1.0))
        } else {
            None
        }
    }

    fn normalize_tag(&self, tag: &str) -> String {
        let tag_lower = tag.to_lowercase();
        self.alias_map.get(&tag_lower)
            .cloned()
            .unwrap_or_else(|| tag_lower)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{TagDefinition, TagAlias};

    fn create_test_config() -> CategorizationConfig {
        CategorizationConfig {
            enabled: true,
            auto_tag_new_articles: true,
            max_tags_per_item: 5,
            confidence_threshold: 0.3,
            tags: vec![
                TagDefinition {
                    name: "rust".to_string(),
                    description: "Rust programming".to_string(),
                    keywords: vec!["rust".to_string(), "cargo".to_string()],
                },
                TagDefinition {
                    name: "ai".to_string(),
                    description: "Artificial Intelligence".to_string(),
                    keywords: vec!["ai".to_string(), "llm".to_string(), "gpt".to_string()],
                },
            ],
            rules: vec![
                TagRule {
                    rule_type: "title_contains".to_string(),
                    patterns: vec!["Rust".to_string()],
                    tag: "rust".to_string(),
                    tags: vec![],
                    confidence: 0.9,
                },
                TagRule {
                    rule_type: "author_contains".to_string(),
                    patterns: vec!["Simon Willison".to_string()],
                    tag: String::new(),
                    tags: vec!["ai".to_string(), "python".to_string()],
                    confidence: 0.7,
                },
            ],
            aliases: vec![
                TagAlias {
                    from: vec!["rustlang".to_string()],
                    to: "rust".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_categorization_engine_creation() {
        let config = create_test_config();
        let engine = CategorizationEngine::from_config(&config);
        assert!(engine.is_enabled());
        assert_eq!(engine.config.tags.len(), 2);
        assert_eq!(engine.alias_map.len(), 1);
    }

    #[test]
    fn test_title_rule_matching() {
        let config = create_test_config();
        let engine = CategorizationEngine::from_config(&config);
        
        let tags = engine.generate_tags_for_item(
            "Learning Rust Programming",
            Some("A guide to Rust"),
            None,
            None,
            "test_feed",
            None,
            None,
        );
        
        assert!(!tags.is_empty());
        assert_eq!(tags[0].name, "rust");
        assert_eq!(tags[0].confidence, 0.9);
    }

    #[test]
    fn test_author_rule_matching() {
        let config = create_test_config();
        let engine = CategorizationEngine::from_config(&config);
        
        let tags = engine.generate_tags_for_item(
            "Machine Learning Basics",
            Some("Introduction to ML"),
            None,
            Some("Simon Willison"),
            "test_feed",
            None,
            None,
        );
        
        assert!(tags.len() >= 2);
        let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
        assert!(tag_names.contains(&"ai".to_string()));
        assert!(tag_names.contains(&"python".to_string()));
    }

    #[test]
    fn test_keyword_matching() {
        let config = create_test_config();
        let engine = CategorizationEngine::from_config(&config);
        
        let tags = engine.generate_tags_for_item(
            "Building with Cargo",
            Some("How to use cargo for Rust projects"),
            None,
            None,
            "test_feed",
            None,
            None,
        );
        
        assert!(!tags.is_empty());
        let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
        assert!(tag_names.contains(&"rust".to_string()));
    }

    #[test]
    fn test_alias_normalization() {
        let config = create_test_config();
        let engine = CategorizationEngine::from_config(&config);
        
        let normalized = engine.normalize_tag("rustlang");
        assert_eq!(normalized, "rust");
        
        let normalized = engine.normalize_tag("RUSTLANG");
        assert_eq!(normalized, "rust");
    }

    #[test]
    fn test_max_tags_limit() {
        let mut config = create_test_config();
        config.max_tags_per_item = 2;
        let engine = CategorizationEngine::from_config(&config);
        
        let tags = engine.generate_tags_for_item(
            "Rust and AI: Building LLM with Cargo",
            Some("GPT implementation in Rust programming language"),
            None,
            Some("Simon Willison"),
            "test_feed",
            Some(&vec!["manual1".to_string(), "manual2".to_string(), "manual3".to_string()]),
            None,
        );
        
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_disabled_categorization() {
        let mut config = create_test_config();
        config.enabled = false;
        let engine = CategorizationEngine::from_config(&config);
        
        let tags = engine.generate_tags_for_item(
            "Rust Programming",
            Some("About Rust"),
            None,
            None,
            "test_feed",
            None,
            None,
        );
        
        assert!(tags.is_empty());
    }
}