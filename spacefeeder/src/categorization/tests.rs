use super::*;
use crate::config::{CategorizationConfig, TagDefinition, TagRule};

fn create_test_context_with_feed_tags<'a>(
    title: &'a str,
    description: Option<&'a str>,
    author: Option<&'a str>,
    feed_slug: &'a str,
    feed_tags: Option<&'a [String]>,
) -> ItemContext<'a> {
    ItemContext {
        title,
        description,
        link: None,
        author,
        feed_slug,
        feed_tags,
        rss_categories: None,
    }
}

fn create_simon_willison_engine() -> CategorizationEngine {
    let config = CategorizationConfig {
        enabled: true,
        confidence_threshold: 0.3,
        max_tags_per_item: 10,
        auto_tag_new_articles: true,
        tags: vec![
            TagDefinition {
                name: "ai".to_string(),
                description: "Artificial Intelligence".to_string(),
                keywords: vec![
                    "ai".to_string(),
                    "artificial intelligence".to_string(),
                    "machine learning".to_string(),
                    "llm".to_string(),
                    "gpt".to_string(),
                ],
            },
            TagDefinition {
                name: "python".to_string(),
                description: "Python programming".to_string(),
                keywords: vec![
                    "python".to_string(),
                    "django".to_string(),
                    "flask".to_string(),
                    "pip".to_string(),
                ],
            },
            TagDefinition {
                name: "web".to_string(),
                description: "Web development".to_string(),
                keywords: vec![
                    "javascript".to_string(),
                    "html".to_string(),
                    "css".to_string(),
                    "web".to_string(),
                ],
            },
        ],
        rules: vec![
            // Author-based rule for Simon Willison + AI content
            TagRule {
                rule_type: "author_with_content".to_string(),
                patterns: vec!["Simon Willison".to_string()],
                tag: "ai".to_string(),
                tags: vec![],
                confidence: 0.8,
                exclude_patterns: vec![],
                min_keyword_count: None,
                required_keywords: vec![
                    "ai".to_string(),
                    "llm".to_string(),
                    "gpt".to_string(),
                    "machine learning".to_string(),
                ],
                exclude_tags: vec![],
            },
        ],
        aliases: vec![],
    };

    CategorizationEngine::from_config(&config)
}

#[test]
fn test_feed_tags_boost_confidence_when_content_matches() {
    let engine = create_simon_willison_engine();
    
    let feed_tags = vec!["ai".to_string()];
    let context = create_test_context_with_feed_tags(
        "Understanding GPT-4's capabilities",
        Some("A deep dive into how GPT-4 works and its applications in AI"),
        Some("Simon Willison"),
        "simonwillison",
        Some(&feed_tags),
    );

    let tags = engine.generate_tags_for_item(&context);
    
    // Should find AI tag with boosted confidence
    let ai_tag = tags.iter().find(|t| t.name == "ai");
    assert!(ai_tag.is_some(), "AI tag should be present");
    
    let ai_tag = ai_tag.unwrap();
    assert!(ai_tag.confidence > 0.8, "AI tag should have high confidence due to content match and feed boost");
    assert!(ai_tag.confidence <= 0.95, "AI tag confidence should be capped at 0.95");
}

#[test]
fn test_feed_tags_low_confidence_without_content_match() {
    let engine = create_simon_willison_engine();
    
    let feed_tags = vec!["ai".to_string()];
    let context = create_test_context_with_feed_tags(
        "Building a Django application",
        Some("Tutorial on creating a web application with Django and Python"),
        Some("Simon Willison"),
        "simonwillison",
        Some(&feed_tags),
    );

    let tags = engine.generate_tags_for_item(&context);
    
    // AI tag should NOT be present or have very low confidence
    // because content doesn't support it
    let ai_tag = tags.iter().find(|t| t.name == "ai");
    assert!(ai_tag.is_none(), "AI tag should not be present when content doesn't match");
    
    // Should find Python tag instead
    let python_tag = tags.iter().find(|t| t.name == "python");
    assert!(python_tag.is_some(), "Python tag should be present based on content");
}

#[test]
fn test_feed_tags_require_keyword_match_for_low_confidence_addition() {
    let engine = create_simon_willison_engine();
    
    let feed_tags = vec!["python".to_string()];
    let context = create_test_context_with_feed_tags(
        "Introduction to Rust",
        Some("Learn the basics of Rust programming language"),
        Some("Simon Willison"),
        "simonwillison",
        Some(&feed_tags),
    );

    let tags = engine.generate_tags_for_item(&context);
    
    // Python tag should NOT be added even as feed hint
    // because no python keywords appear in content
    let python_tag = tags.iter().find(|t| t.name == "python");
    assert!(python_tag.is_none(), "Python tag should not be added without any keyword match");
}

#[test]
fn test_feed_tags_weak_signal_gets_low_confidence() {
    let engine = create_simon_willison_engine();
    
    let feed_tags = vec!["python".to_string()];
    let context = create_test_context_with_feed_tags(
        "Quick note about pip",
        Some("Just a brief mention about package management"),
        Some("Simon Willison"),
        "simonwillison",
        Some(&feed_tags),
    );

    let tags = engine.generate_tags_for_item(&context);
    
    // Python tag might be added with very low confidence
    // since "pip" is a python keyword - but it's actually matching as a regular keyword
    // with base confidence 0.33, then boosted by feed tag to 0.4
    let python_tag = tags.iter().find(|t| t.name == "python");
    if let Some(tag) = python_tag {
        println!("Python tag confidence: {}", tag.confidence);
        assert!(tag.confidence <= 0.41, "Feed hint should boost confidence slightly, got: {}", tag.confidence);
    }
}

#[test]
fn test_multi_topic_author_correct_tagging() {
    let engine = create_simon_willison_engine();
    
    // Test case 1: AI article from Simon Willison
    let feed_tags = vec!["ai".to_string(), "python".to_string(), "web".to_string()];
    let context = create_test_context_with_feed_tags(
        "Building LLM applications",
        Some("How to integrate GPT models into your Python applications"),
        Some("Simon Willison"),
        "simonwillison",
        Some(&feed_tags),
    );

    let tags = engine.generate_tags_for_item(&context);
    
    // Should tag as both AI and Python based on content
    assert!(tags.iter().any(|t| t.name == "ai"), "AI tag should be present");
    assert!(tags.iter().any(|t| t.name == "python"), "Python tag should be present");
    
    // Web tag should NOT be present as content doesn't support it
    let web_tag = tags.iter().find(|t| t.name == "web");
    assert!(web_tag.is_none(), "Web tag should not be present without content match");
}

#[test]
fn test_exclusion_rules_override_feed_tags() {
    let config = CategorizationConfig {
        enabled: true,
        confidence_threshold: 0.3,
        max_tags_per_item: 10,
        auto_tag_new_articles: true,
        tags: vec![
            TagDefinition {
                name: "ai".to_string(),
                description: "AI".to_string(),
                keywords: vec!["ai".to_string(), "artificial".to_string()],
            },
        ],
        rules: vec![
            TagRule {
                rule_type: "exclude_if".to_string(),
                patterns: vec!["weekly links".to_string(), "link roundup".to_string()],
                tag: "".to_string(),
                tags: vec![],
                confidence: 1.0,
                exclude_patterns: vec![],
                min_keyword_count: None,
                required_keywords: vec![],
                exclude_tags: vec!["ai".to_string()],
            },
        ],
        aliases: vec![],
    };
    
    let engine = CategorizationEngine::from_config(&config);
    
    let feed_tags = vec!["ai".to_string()];
    let context = create_test_context_with_feed_tags(
        "Weekly links about AI",
        Some("My weekly roundup of artificial intelligence news"),
        None,
        "blog",
        Some(&feed_tags),
    );

    let tags = engine.generate_tags_for_item(&context);
    
    // AI tag should be excluded despite being in feed tags and content
    let ai_tag = tags.iter().find(|t| t.name == "ai");
    assert!(ai_tag.is_none(), "AI tag should be excluded by exclusion rule");
}