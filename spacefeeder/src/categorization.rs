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

pub struct ItemContext<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub link: Option<&'a str>,
    pub author: Option<&'a str>,
    pub feed_slug: &'a str,
    pub feed_tags: Option<&'a [String]>,
    pub rss_categories: Option<&'a [String]>,
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

    pub fn generate_tags_for_item(&self, context: &ItemContext) -> Vec<Tag> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut tags = Vec::new();
        let mut seen = HashSet::new();

        // Store feed tags for later use as confidence boosters
        let feed_tag_hints: HashSet<String> = if let Some(feed_tags) = context.feed_tags {
            feed_tags.iter().map(|t| self.normalize_tag(t)).collect()
        } else {
            HashSet::new()
        };

        // 1. Add RSS/Atom category tags (these are from the content itself, so keep high confidence)
        if let Some(categories) = context.rss_categories {
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

        // 2. Check for exclusion rules first
        let mut excluded_tags = HashSet::new();
        for rule in &self.config.rules {
            if rule.rule_type == "exclude_if" {
                let content = format!(
                    "{} {}",
                    context.title.to_lowercase(),
                    context.description.unwrap_or("").to_lowercase()
                );

                if rule
                    .patterns
                    .iter()
                    .any(|p| self.matches_keyword(&content, &p.to_lowercase()))
                {
                    // If this exclude rule matches, mark its exclude_tags for exclusion
                    for tag in &rule.exclude_tags {
                        excluded_tags.insert(tag.clone());
                    }
                }
            }
        }

        // 3. Apply rule-based tagging (skipping exclude_if rules)
        for rule in &self.config.rules {
            if rule.rule_type != "exclude_if" {
                if let Some(matched_tags) =
                    self.apply_rule(rule, context.title, context.description, context.link, context.author, context.feed_slug)
                {
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

        // 4. Apply keyword-based tagging
        if self.config.auto_tag_new_articles {
            let content = format!("{} {}", context.title, context.description.unwrap_or(""));
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

        // 5. Apply confidence boost for tags that match feed hints
        // Feed tags act as hints that increase confidence when content supports them
        for tag in &mut tags {
            if feed_tag_hints.contains(&tag.name) {
                // Boost confidence by 20% if this tag was hinted by feed tags
                // but cap at 0.95 to show it's still derived from content
                tag.confidence = (tag.confidence * 1.2).min(0.95);
            }
        }

        // 6. Add feed tags that weren't found in content with very low confidence
        // This ensures feed categorization is preserved but with appropriate skepticism
        for feed_tag in &feed_tag_hints {
            if !seen.contains(feed_tag) && !excluded_tags.contains(feed_tag) {
                // Check if there's at least some weak signal in the content
                let content = format!(
                    "{} {}",
                    context.title.to_lowercase(),
                    context.description.unwrap_or("").to_lowercase()
                );

                // Only add if the tag keyword appears somewhere in content
                // This prevents completely unrelated tags
                if let Some(tag_def) = self
                    .config
                    .tags
                    .iter()
                    .find(|t| self.normalize_tag(&t.name) == *feed_tag)
                {
                    if !tag_def.keywords.is_empty() {
                        let has_any_keyword = tag_def
                            .keywords
                            .iter()
                            .any(|kw| self.matches_keyword(&content, &kw.to_lowercase()));

                        if has_any_keyword && seen.insert(feed_tag.clone()) {
                            tags.push(Tag {
                                name: feed_tag.clone(),
                                confidence: 0.25, // Very low confidence for feed hints without strong content support
                                source: TagSource::Manual,
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
                if self.matches_keyword(&content, &exclude_pattern.to_lowercase()) {
                    return None; // Rule excluded
                }
            }
        }

        let matches = match rule.rule_type.as_str() {
            "title_contains" => {
                let title_lower = title.to_lowercase();
                rule.patterns
                    .iter()
                    .any(|p| self.matches_keyword(&title_lower, &p.to_lowercase()))
            }
            "content_contains" => {
                let content = format!(
                    "{} {}",
                    title.to_lowercase(),
                    description.unwrap_or("").to_lowercase()
                );
                rule.patterns
                    .iter()
                    .any(|p| self.matches_keyword(&content, &p.to_lowercase()))
            }
            "content_analysis" => {
                // Advanced content analysis with keyword count requirements
                let content = format!(
                    "{} {}",
                    title.to_lowercase(),
                    description.unwrap_or("").to_lowercase()
                );

                let matched_keywords = rule
                    .patterns
                    .iter()
                    .filter(|p| self.matches_keyword(&content, &p.to_lowercase()))
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
                    let author_matches = rule
                        .patterns
                        .iter()
                        .any(|p| author_str.to_lowercase().contains(&p.to_lowercase()));

                    if author_matches {
                        // If there are required keywords, ALL must be present
                        if !rule.required_keywords.is_empty() {
                            let content = format!(
                                "{} {}",
                                title.to_lowercase(),
                                description.unwrap_or("").to_lowercase()
                            );

                            rule.required_keywords
                                .iter()
                                .all(|kw| self.matches_keyword(&content, &kw.to_lowercase()))
                        } else {
                            // No required keywords means this rule shouldn't fire
                            // (use regular author_contains instead)
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            "url_contains" => {
                if let Some(url) = link {
                    let url_lower = url.to_lowercase();
                    rule.patterns
                        .iter()
                        .any(|p| url_lower.contains(&p.to_lowercase()))
                } else {
                    false
                }
            }
            "author_contains" => {
                if let Some(author_str) = author {
                    let author_lower = author_str.to_lowercase();
                    rule.patterns
                        .iter()
                        .any(|p| author_lower.contains(&p.to_lowercase()))
                } else {
                    false
                }
            }
            "feed_slug" => rule.patterns.iter().any(|p| feed_slug == p),
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
            if self.matches_keyword(&content_lower, &keyword.to_lowercase()) {
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

    /// Check if keyword matches with word boundaries or phrase matching
    fn matches_keyword(&self, content: &str, keyword: &str) -> bool {
        // For multi-word phrases, use exact substring matching
        if keyword.contains(' ') {
            return content.contains(keyword);
        }

        // For single words, check word boundaries to avoid false matches
        // e.g., "ai" shouldn't match "said" or "wait"

        // Use byte positions since find() returns byte positions
        if let Some(byte_pos) = content.find(keyword) {
            let keyword_byte_len = keyword.len();

            // Check character before (if any)
            let before_ok = if byte_pos == 0 {
                true
            } else {
                // Get the character just before the match
                let before_slice = &content[..byte_pos];
                if let Some(last_char) = before_slice.chars().last() {
                    !last_char.is_alphabetic()
                } else {
                    true
                }
            };

            // Check character after (if any)
            let after_byte_pos = byte_pos + keyword_byte_len;
            let after_ok = if after_byte_pos >= content.len() {
                true
            } else {
                // Get the character just after the match
                let after_slice = &content[after_byte_pos..];
                if let Some(first_char) = after_slice.chars().next() {
                    !first_char.is_alphabetic()
                } else {
                    true
                }
            };

            before_ok && after_ok
        } else {
            false
        }
    }

    fn normalize_tag(&self, tag: &str) -> String {
        let tag_lower = tag.to_lowercase();
        self.alias_map
            .get(&tag_lower)
            .cloned()
            .unwrap_or(tag_lower)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CategorizationConfig, TagAlias, TagDefinition, TagRule};

    fn create_test_context<'a>(
        title: &'a str,
        description: Option<&'a str>,
        link: Option<&'a str>,
        author: Option<&'a str>,
        feed_slug: &'a str,
        feed_tags: Option<&'a [String]>,
        rss_categories: Option<&'a [String]>,
    ) -> ItemContext<'a> {
        ItemContext {
            title,
            description,
            link,
            author,
            feed_slug,
            feed_tags,
            rss_categories,
        }
    }

    fn create_test_engine() -> CategorizationEngine {
        let config = CategorizationConfig {
            enabled: true,
            auto_tag_new_articles: true,
            max_tags_per_item: 5,
            confidence_threshold: 0.3,
            tags: vec![
                TagDefinition {
                    name: "ai".to_string(),
                    description: "AI and ML".to_string(),
                    keywords: vec![
                        "artificial intelligence".to_string(),
                        "machine learning".to_string(),
                        "neural network".to_string(),
                    ],
                },
                TagDefinition {
                    name: "rust".to_string(),
                    description: "Rust programming".to_string(),
                    keywords: vec!["rust".to_string(), "cargo".to_string(), "rustc".to_string()],
                },
                TagDefinition {
                    name: "python".to_string(),
                    description: "Python programming".to_string(),
                    keywords: vec![
                        "python".to_string(),
                        "django".to_string(),
                        "pip".to_string(),
                    ],
                },
            ],
            rules: vec![
                // Test rule: Simon Willison + AI content
                TagRule {
                    rule_type: "author_with_content".to_string(),
                    patterns: vec!["Simon Willison".to_string()],
                    tag: "ai".to_string(),
                    tags: vec![],
                    confidence: 0.8,
                    exclude_patterns: vec![],
                    min_keyword_count: None,
                    required_keywords: vec![
                        "artificial intelligence".to_string(),
                        "machine learning".to_string(),
                    ],
                    exclude_tags: vec![],
                },
                // Test rule: Title contains AI
                TagRule {
                    rule_type: "title_contains".to_string(),
                    patterns: vec!["AI".to_string()],
                    tag: "ai".to_string(),
                    tags: vec![],
                    confidence: 0.9,
                    exclude_patterns: vec![],
                    min_keyword_count: None,
                    required_keywords: vec![],
                    exclude_tags: vec![],
                },
                // Test rule: Content analysis for Rust
                TagRule {
                    rule_type: "content_analysis".to_string(),
                    patterns: vec!["rust".to_string(), "cargo".to_string(), "rustc".to_string()],
                    tag: "rust".to_string(),
                    tags: vec![],
                    confidence: 0.8,
                    exclude_patterns: vec![],
                    min_keyword_count: Some(2),
                    required_keywords: vec![],
                    exclude_tags: vec![],
                },
                // Test exclusion rule
                TagRule {
                    rule_type: "exclude_if".to_string(),
                    patterns: vec!["weekly links".to_string(), "news roundup".to_string()],
                    tag: "".to_string(),
                    tags: vec![],
                    confidence: 1.0,
                    exclude_patterns: vec![],
                    min_keyword_count: None,
                    required_keywords: vec![],
                    exclude_tags: vec!["ai".to_string(), "rust".to_string(), "python".to_string()],
                },
            ],
            aliases: vec![TagAlias {
                from: vec!["artificial-intelligence".to_string(), "ml".to_string()],
                to: "ai".to_string(),
            }],
        };

        CategorizationEngine::from_config(&config)
    }

    #[test]
    fn test_keyword_word_boundaries() {
        let engine = create_test_engine();

        // "ai" should NOT match "said", "wait", "maintain"
        assert!(!engine.matches_keyword("i said hello", "ai"));
        assert!(!engine.matches_keyword("please wait here", "ai"));
        assert!(!engine.matches_keyword("maintain the system", "ai"));

        // "ai" should match when it's a standalone word
        assert!(engine.matches_keyword("ai is powerful", "ai"));
        assert!(engine.matches_keyword("the ai system", "ai"));
        assert!(engine.matches_keyword("talk about ai.", "ai"));
        assert!(engine.matches_keyword("ai", "ai"));

        // Multi-word phrases should work
        assert!(engine.matches_keyword(
            "artificial intelligence is growing",
            "artificial intelligence"
        ));
        assert!(!engine.matches_keyword("partially intelligent systems", "artificial intelligence"));
    }

    #[test]
    fn test_author_with_content_requires_keywords() {
        let engine = create_test_engine();

        // Simon Willison article WITHOUT AI keywords should NOT get AI tag
        let context = create_test_context(
            "Static Sites with Python, uv, Caddy, and Docker",
            Some("A technical tutorial about building static websites using Python tooling and Docker containers for deployment."),
            Some("https://simonwillison.net/2025/Aug/24/example/"),
            Some("Simon Willison"),
            "simonwillison",
            None,
            None,
        );
        let tags = engine.generate_tags_for_item(&context);

        let ai_tags: Vec<_> = tags.iter().filter(|t| t.name == "ai").collect();
        assert!(
            ai_tags.is_empty(),
            "Should not tag non-AI content from Simon Willison with AI"
        );

        // Simon Willison article WITH AI keywords should get AI tag
        let context = create_test_context(
            "Evaluating Large Language Models for Code Generation",
            Some("A deep dive into machine learning approaches for automatic code generation using artificial intelligence and neural networks."),
            Some("https://simonwillison.net/2025/Aug/24/llm-code/"),
            Some("Simon Willison"),
            "simonwillison",
            None,
            None,
        );
        let tags = engine.generate_tags_for_item(&context);

        let ai_tags: Vec<_> = tags.iter().filter(|t| t.name == "ai").collect();
        assert!(
            !ai_tags.is_empty(),
            "Should tag AI content from Simon Willison with AI"
        );
        assert!(
            ai_tags[0].confidence > 0.5,
            "Should have reasonable confidence for AI content"
        );
    }

    #[test]
    fn test_title_ai_matching_precision() {
        let engine = create_test_engine();

        // Title with "AI" word should match
        let feed_tags = ["news".to_string(), "tech".to_string()];
        let context = create_test_context(
            "Show HN: ClearCam – Add AI object detection to your IP CCTV cameras",
            Some("This runs YOLOv8 + bytetrack with Tinygrad for computer vision detection."),
            None,
            Some("Hacker News"),
            "hackernews",
            Some(&feed_tags),
            None,
        );
        let tags = engine.generate_tags_for_item(&context);

        let ai_tags: Vec<_> = tags.iter().filter(|t| t.name == "ai").collect();
        assert!(
            !ai_tags.is_empty(),
            "Should tag articles with 'AI' in title"
        );

        // Title with "said" should NOT trigger AI tag via keyword matching
        let feed_tags = ["news".to_string(), "tech".to_string()];
        let context = create_test_context(
            "CEO said the company will focus on renewable energy",
            Some("The executive announced a major shift toward solar and wind power investments."),
            None,
            Some("Hacker News"),
            "hackernews",
            Some(&feed_tags),
            None,
        );
        let tags = engine.generate_tags_for_item(&context);

        let ai_tags: Vec<_> = tags.iter().filter(|t| t.name == "ai").collect();
        assert!(
            ai_tags.is_empty(),
            "Should not tag articles just because they contain 'said'"
        );
    }

    #[test]
    fn test_content_analysis_multi_keyword_requirement() {
        let engine = create_test_engine();

        // Article mentioning only "rust" (1 keyword) gets lower confidence from keyword match
        // but the content_analysis rule (requires 2) shouldn't fire
        let context = create_test_context(
            "Programming languages overview",
            Some("A comparison of various languages including Go, Python, and rust for different use cases."),
            None,
            Some("Tech Blog"),
            "techblog",
            None,
            None,
        );
        let tags = engine.generate_tags_for_item(&context);

        // The keyword-based tagging will find "rust" and tag it (confidence ~0.33)
        // The content_analysis rule won't fire because it needs 2 keywords
        let rust_tags: Vec<_> = tags.iter().filter(|t| t.name == "rust").collect();
        if !rust_tags.is_empty() {
            // If tagged, it should be from keyword matching with lower confidence, not the rule
            assert!(
                rust_tags[0].confidence < 0.7,
                "Should have low confidence from keyword match, not high confidence from rule"
            );
        }

        // Article mentioning "rust" and "cargo" (2 keywords) gets content_analysis rule match
        let context = create_test_context(
            "Building Rust applications with Cargo",
            Some("A comprehensive guide to using cargo for rust project management and building."),
            None,
            Some("Tech Blog"),
            "techblog",
            None,
            None,
        );
        let tags = engine.generate_tags_for_item(&context);

        let rust_tags: Vec<_> = tags.iter().filter(|t| t.name == "rust").collect();
        assert!(!rust_tags.is_empty(), "Should tag with 2 rust keywords");
        // Should have higher confidence from the rule
        assert!(
            rust_tags[0].confidence >= 0.7,
            "Should have high confidence from content_analysis rule"
        );
    }

    #[test]
    fn test_exclude_if_prevents_tagging() {
        let engine = create_test_engine();

        // Weekly links article should NOT get programming language tags
        let context = create_test_context(
            "Weekly Links Roundup - August 2025", 
            Some("This week's collection includes articles about rust programming, python tutorials, and artificial intelligence breakthroughs."),
            None,
            Some("Tech Newsletter"),
            "newsletter",
            None,
            None,
        );
        let tags = engine.generate_tags_for_item(&context);

        let ai_tags: Vec<_> = tags.iter().filter(|t| t.name == "ai").collect();
        let rust_tags: Vec<_> = tags.iter().filter(|t| t.name == "rust").collect();
        let python_tags: Vec<_> = tags.iter().filter(|t| t.name == "python").collect();

        assert!(ai_tags.is_empty(), "Should not tag AI in weekly links");
        assert!(rust_tags.is_empty(), "Should not tag Rust in weekly links");
        assert!(
            python_tags.is_empty(),
            "Should not tag Python in weekly links"
        );

        // Regular article should get tags normally
        let context = create_test_context(
            "Advanced Rust Programming Techniques",
            Some("Deep dive into cargo workspaces and rustc optimization techniques for better performance."),
            None,
            Some("Programming Blog"),
            "progblog", 
            None,
            None,
        );
        let tags = engine.generate_tags_for_item(&context);

        let rust_tags: Vec<_> = tags.iter().filter(|t| t.name == "rust").collect();
        assert!(!rust_tags.is_empty(), "Should tag regular rust articles");
    }

    #[test]
    fn test_feed_level_tags_should_not_override_content() {
        let engine = create_test_engine();

        // Article about Python/Docker from an "AI-focused" feed should NOT get AI tag
        // if the content isn't about AI
        let feed_tags = vec!["ai".to_string(), "database".to_string()];
        let context = create_test_context(
            "Static Sites with Python, uv, Caddy, and Docker",
            Some("A tutorial about building static websites using Python tooling and Docker containers."),
            None,
            Some("Simon Willison"),
            "simonwillison",
            Some(&feed_tags),
            None,
        );
        let tags = engine.generate_tags_for_item(&context);

        // Should get python and docker tags from content
        let tag_names: Vec<_> = tags.iter().map(|t| t.name.clone()).collect();
        assert!(
            tag_names.contains(&"python".to_string()),
            "Should tag Python content"
        );

        // Should NOT get AI tag just because feed has it
        let ai_tags: Vec<_> = tags.iter().filter(|t| t.name == "ai").collect();
        if !ai_tags.is_empty() {
            // If AI tag exists, it should have lower confidence (from feed hint)
            // not high confidence (from absolute assignment)
            assert!(ai_tags[0].confidence < 0.8,
                "Feed tags should provide hints (low confidence), not absolute assignments (high confidence). Got confidence: {}", 
                ai_tags[0].confidence);
        }
    }

    #[test]
    fn test_realistic_hacker_news_false_positives() {
        let engine = create_test_engine();

        // Renewable energy article should NOT get AI tag
        let feed_tags = ["news".to_string(), "tech".to_string()];
        let context = create_test_context(
            "US attack on renewables will lead to power crunch that spikes electricity prices",
            Some("Analysis of renewable energy policy impacts on electricity pricing and grid stability."),
            Some("https://www.cnbc.com/2025/08/24/solar-wind-renewable-trump-tariff.html"),
            Some("Hacker News"),
            "hackernews",
            Some(&feed_tags),
            None,
        );
        let tags = engine.generate_tags_for_item(&context);

        let ai_tags: Vec<_> = tags.iter().filter(|t| t.name == "ai").collect();
        assert!(
            ai_tags.is_empty(),
            "Renewable energy articles should not get AI tag"
        );

        // Valve employee handbook should NOT get AI tag
        let feed_tags = ["news".to_string(), "tech".to_string()];
        let context = create_test_context(
            "Valve Software handbook for new employees [pdf] (2012)",
            Some("Internal company documentation about Valve's flat organizational structure and hiring practices."),
            Some("https://cdn.akamai.steamstatic.com/apps/valve/Valve_NewEmployeeHandbook.pdf"),
            Some("Hacker News"),
            "hackernews",
            Some(&feed_tags),
            None,
        );
        let tags = engine.generate_tags_for_item(&context);

        let ai_tags: Vec<_> = tags.iter().filter(|t| t.name == "ai").collect();
        assert!(
            ai_tags.is_empty(),
            "Corporate handbooks should not get AI tag"
        );

        // Actual AI article should get AI tag
        let feed_tags = ["news".to_string(), "tech".to_string()];
        let context = create_test_context(
            "Evaluating LLMs for my personal use case",
            Some("Comprehensive comparison of different large language models including GPT-4, Claude, and open source alternatives for machine learning applications."),
            Some("https://darkcoding.net/software/personal-ai-evals-aug-2025/"),
            Some("Hacker News"),
            "hackernews",
            Some(&feed_tags),
            None,
        );
        let tags = engine.generate_tags_for_item(&context);

        let ai_tags: Vec<_> = tags.iter().filter(|t| t.name == "ai").collect();
        assert!(!ai_tags.is_empty(), "Actual LLM articles should get AI tag");
    }

    #[test]
    fn test_tag_normalization() {
        let engine = create_test_engine();

        // Check that aliases work correctly
        let normalized = engine.normalize_tag("ml");
        assert_eq!(normalized, "ai", "ML should normalize to AI");

        let normalized = engine.normalize_tag("artificial-intelligence");
        assert_eq!(
            normalized, "ai",
            "artificial-intelligence should normalize to AI"
        );

        let normalized = engine.normalize_tag("rust");
        assert_eq!(normalized, "rust", "rust should stay as rust");
    }
}
