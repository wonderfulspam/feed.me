use super::{matching::StringMatcher, Tag, TagSource};
use crate::config::TagRule;

pub struct RuleApplicator<'a> {
    matcher: &'a StringMatcher,
}

impl<'a> RuleApplicator<'a> {
    pub fn new(matcher: &'a StringMatcher) -> Self {
        Self { matcher }
    }

    /// Apply a single tagging rule to content
    pub fn apply_rule(
        &self,
        rule: &TagRule,
        title: &str,
        description: Option<&str>,
        link: Option<&str>,
        author: Option<&str>,
        feed_slug: &str,
    ) -> Option<Vec<Tag>> {
        match rule.rule_type.as_str() {
            "title_contains" => self.apply_title_rule(rule, title),
            "content_contains" => self.apply_content_rule(rule, title, description),
            "url_contains" => self.apply_url_rule(rule, link),
            "feed_slug" => self.apply_feed_slug_rule(rule, feed_slug),
            "author_with_content" => {
                self.apply_author_content_rule(rule, title, description, author)
            }
            "content_analysis" => self.apply_content_analysis_rule(rule, title, description),
            _ => None,
        }
    }

    fn apply_title_rule(&self, rule: &TagRule, title: &str) -> Option<Vec<Tag>> {
        let title_lower = title.to_lowercase();

        for pattern in &rule.patterns {
            if self
                .matcher
                .matches_keyword(&title_lower, &pattern.to_lowercase())
            {
                // Check exclude patterns
                if self.has_exclude_patterns(rule, title, None) {
                    return None;
                }

                return Some(vec![Tag {
                    name: rule.tag.clone(),
                    confidence: rule.confidence,
                    source: TagSource::Rule,
                }]);
            }
        }
        None
    }

    fn apply_content_rule(
        &self,
        rule: &TagRule,
        title: &str,
        description: Option<&str>,
    ) -> Option<Vec<Tag>> {
        let content = format!("{} {}", title, description.unwrap_or(""));
        let content_lower = content.to_lowercase();

        for pattern in &rule.patterns {
            if self
                .matcher
                .matches_keyword(&content_lower, &pattern.to_lowercase())
            {
                // Check exclude patterns
                if self.has_exclude_patterns(rule, title, description) {
                    return None;
                }

                return Some(vec![Tag {
                    name: rule.tag.clone(),
                    confidence: rule.confidence,
                    source: TagSource::Rule,
                }]);
            }
        }
        None
    }

    fn apply_url_rule(&self, rule: &TagRule, link: Option<&str>) -> Option<Vec<Tag>> {
        if let Some(url) = link {
            let url_lower = url.to_lowercase();
            for pattern in &rule.patterns {
                if url_lower.contains(&pattern.to_lowercase()) {
                    return Some(vec![Tag {
                        name: rule.tag.clone(),
                        confidence: rule.confidence,
                        source: TagSource::Rule,
                    }]);
                }
            }
        }
        None
    }

    fn apply_feed_slug_rule(&self, rule: &TagRule, feed_slug: &str) -> Option<Vec<Tag>> {
        let slug_lower = feed_slug.to_lowercase();
        for pattern in &rule.patterns {
            if slug_lower.contains(&pattern.to_lowercase()) {
                return Some(vec![Tag {
                    name: rule.tag.clone(),
                    confidence: rule.confidence,
                    source: TagSource::Rule,
                }]);
            }
        }
        None
    }

    fn apply_author_content_rule(
        &self,
        rule: &TagRule,
        title: &str,
        description: Option<&str>,
        author: Option<&str>,
    ) -> Option<Vec<Tag>> {
        // Check if author matches
        if let Some(author_name) = author {
            for pattern in &rule.patterns {
                if author_name.to_lowercase().contains(&pattern.to_lowercase()) {
                    // Author matches, now check if required keywords are present
                    if !rule.required_keywords.is_empty() {
                        let content = format!("{} {}", title, description.unwrap_or(""));
                        let content_lower = content.to_lowercase();

                        // Check if any of the required keywords are present
                        let has_required_keyword = rule.required_keywords.iter().any(|keyword| {
                            self.matcher
                                .matches_keyword(&content_lower, &keyword.to_lowercase())
                        });

                        if !has_required_keyword {
                            return None;
                        }
                    }

                    return Some(vec![Tag {
                        name: rule.tag.clone(),
                        confidence: rule.confidence,
                        source: TagSource::Rule,
                    }]);
                }
            }
        }
        None
    }

    fn apply_content_analysis_rule(
        &self,
        rule: &TagRule,
        title: &str,
        description: Option<&str>,
    ) -> Option<Vec<Tag>> {
        let content = format!("{} {}", title, description.unwrap_or(""));
        let content_lower = content.to_lowercase();

        let mut matched_keywords = 0;
        for pattern in &rule.patterns {
            if self
                .matcher
                .matches_keyword(&content_lower, &pattern.to_lowercase())
            {
                matched_keywords += 1;
            }
        }

        // Check minimum keyword count requirement
        let min_required = rule.min_keyword_count.unwrap_or(1);
        if matched_keywords >= min_required {
            // Check exclude patterns
            if self.has_exclude_patterns(rule, title, description) {
                return None;
            }

            return Some(vec![Tag {
                name: rule.tag.clone(),
                confidence: rule.confidence,
                source: TagSource::Rule,
            }]);
        }

        None
    }

    fn has_exclude_patterns(&self, rule: &TagRule, title: &str, description: Option<&str>) -> bool {
        if rule.exclude_patterns.is_empty() {
            return false;
        }

        let content = format!("{} {}", title, description.unwrap_or(""));
        let content_lower = content.to_lowercase();

        rule.exclude_patterns.iter().any(|pattern| {
            self.matcher
                .matches_keyword(&content_lower, &pattern.to_lowercase())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TagRule;
    use std::collections::HashMap;

    fn create_test_applicator() -> RuleApplicator<'static> {
        static MATCHER: std::sync::OnceLock<StringMatcher> = std::sync::OnceLock::new();
        let matcher = MATCHER.get_or_init(|| StringMatcher::new(HashMap::new()));
        RuleApplicator::new(matcher)
    }

    #[test]
    fn test_title_rule() {
        let applicator = create_test_applicator();
        let rule = TagRule {
            rule_type: "title_contains".to_string(),
            patterns: vec!["Rust".to_string()],
            tag: "rust".to_string(),
            tags: vec![],
            confidence: 0.9,
            exclude_patterns: vec![],
            min_keyword_count: None,
            required_keywords: vec![],
            exclude_tags: vec![],
        };

        let result = applicator.apply_rule(&rule, "Introduction to Rust", None, None, None, "blog");
        assert!(result.is_some());
        let tags = result.unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "rust");
        assert_eq!(tags[0].confidence, 0.9);
    }

    #[test]
    fn test_author_with_content_rule() {
        let applicator = create_test_applicator();
        let rule = TagRule {
            rule_type: "author_with_content".to_string(),
            patterns: vec!["Simon Willison".to_string()],
            tag: "ai".to_string(),
            tags: vec![],
            confidence: 0.8,
            exclude_patterns: vec![],
            min_keyword_count: None,
            required_keywords: vec!["ai".to_string(), "llm".to_string()],
            exclude_tags: vec![],
        };

        // Should match when author + required keyword present
        let result = applicator.apply_rule(
            &rule,
            "Building AI applications",
            Some("Using LLM models"),
            None,
            Some("Simon Willison"),
            "blog",
        );
        assert!(result.is_some());

        // Should not match when required keyword missing
        let result = applicator.apply_rule(
            &rule,
            "Building web applications",
            Some("Using Django"),
            None,
            Some("Simon Willison"),
            "blog",
        );
        assert!(result.is_none());
    }
}
