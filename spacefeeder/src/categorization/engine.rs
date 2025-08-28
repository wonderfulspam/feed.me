use std::collections::{HashMap, HashSet};

use super::{
    matching::StringMatcher,
    rules::RuleApplicator,
    types::{ItemContext, Tag, TagSource},
};
use crate::config::CategorizationConfig;

pub struct CategorizationEngine {
    config: CategorizationConfig,
    matcher: StringMatcher,
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
            matcher: StringMatcher::new(alias_map),
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
            feed_tags
                .iter()
                .map(|t| self.matcher.normalize_tag(t))
                .collect()
        } else {
            HashSet::new()
        };

        // 1. Add RSS/Atom category tags (these are from the content itself, so keep high confidence)
        if let Some(categories) = context.rss_categories {
            for category in categories {
                let normalized = self.matcher.normalize_tag(category);
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
                    .any(|p| self.matcher.matches_keyword(&content, &p.to_lowercase()))
                {
                    // If this exclude rule matches, mark its exclude_tags for exclusion
                    for tag in &rule.exclude_tags {
                        excluded_tags.insert(tag.clone());
                    }
                }
            }
        }

        // 3. Apply rule-based tagging (skipping exclude_if rules)
        let rule_applicator = RuleApplicator::new(&self.matcher);
        for rule in &self.config.rules {
            if rule.rule_type != "exclude_if" {
                if let Some(matched_tags) = rule_applicator.apply_rule(
                    rule,
                    context.title,
                    context.description,
                    context.link,
                    context.author,
                    context.feed_slug,
                ) {
                    for tag in matched_tags {
                        let normalized = self.matcher.normalize_tag(&tag.name);
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
                if let Some(confidence) = self.matcher.check_keywords(&content, &tag_def.keywords) {
                    if confidence >= self.config.confidence_threshold {
                        let normalized = self.matcher.normalize_tag(&tag_def.name);
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
                    .find(|t| self.matcher.normalize_tag(&t.name) == *feed_tag)
                {
                    if !tag_def.keywords.is_empty() {
                        let has_any_keyword = tag_def
                            .keywords
                            .iter()
                            .any(|kw| self.matcher.matches_keyword(&content, &kw.to_lowercase()));

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
}
