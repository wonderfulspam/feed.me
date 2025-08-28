use regex::Regex;
use std::collections::HashMap;

pub struct StringMatcher {
    alias_map: HashMap<String, String>,
}

impl StringMatcher {
    pub fn new(alias_map: HashMap<String, String>) -> Self {
        Self { alias_map }
    }

    /// Check if keywords match content and return confidence score
    pub fn check_keywords(&self, content: &str, keywords: &[String]) -> Option<f32> {
        if keywords.is_empty() {
            return None;
        }

        let content_lower = content.to_lowercase();
        let mut matched_keywords = 0;

        for keyword in keywords {
            if self.matches_keyword(&content_lower, &keyword.to_lowercase()) {
                matched_keywords += 1;
            }
        }

        if matched_keywords > 0 {
            // Simple confidence calculation based on keyword density
            let confidence = (matched_keywords as f32 / keywords.len() as f32).clamp(0.0, 1.0);
            Some(confidence.max(0.33)) // Minimum confidence threshold
        } else {
            None
        }
    }

    /// Check if a keyword matches in content using word boundaries
    pub fn matches_keyword(&self, content: &str, keyword: &str) -> bool {
        if keyword.is_empty() {
            return false;
        }

        // For single words, use word boundaries to avoid partial matches
        if !keyword.contains(' ') {
            // Create regex pattern with word boundaries
            let pattern = format!(r"\b{}\b", regex::escape(keyword));
            if let Ok(re) = Regex::new(&pattern) {
                return re.is_match(content);
            }
        }

        // For multi-word phrases, check if the phrase exists
        // Also use word boundaries at the start and end
        let words: Vec<&str> = keyword.split_whitespace().collect();
        if words.len() > 1 {
            // Check if all words appear in sequence with word boundaries
            let pattern = words
                .iter()
                .map(|word| regex::escape(word))
                .collect::<Vec<_>>()
                .join(r"\s+");
            let full_pattern = format!(r"\b{}\b", pattern);

            if let Ok(re) = Regex::new(&full_pattern) {
                return re.is_match(content);
            }
        }

        // Fallback to simple contains check
        content.contains(keyword)
    }

    /// Normalize tag using aliases
    pub fn normalize_tag(&self, tag: &str) -> String {
        let tag_lower = tag.to_lowercase();
        self.alias_map.get(&tag_lower).cloned().unwrap_or(tag_lower)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_matcher() -> StringMatcher {
        let mut alias_map = HashMap::new();
        alias_map.insert("ml".to_string(), "ai".to_string());
        alias_map.insert("artificial-intelligence".to_string(), "ai".to_string());
        StringMatcher::new(alias_map)
    }

    #[test]
    fn test_keyword_word_boundaries() {
        let matcher = create_test_matcher();

        // "ai" should NOT match "said", "wait", "maintain"
        assert!(!matcher.matches_keyword("i said hello", "ai"));
        assert!(!matcher.matches_keyword("please wait here", "ai"));
        assert!(!matcher.matches_keyword("maintain the system", "ai"));

        // "ai" should match when it's a standalone word
        assert!(matcher.matches_keyword("ai is powerful", "ai"));
        assert!(matcher.matches_keyword("the ai system", "ai"));
        assert!(matcher.matches_keyword("talk about ai.", "ai"));
        assert!(matcher.matches_keyword("ai", "ai"));

        // Multi-word phrases should work
        assert!(matcher.matches_keyword(
            "artificial intelligence is growing",
            "artificial intelligence"
        ));
        assert!(
            !matcher.matches_keyword("partially intelligent systems", "artificial intelligence")
        );
    }

    #[test]
    fn test_tag_normalization() {
        let matcher = create_test_matcher();

        assert_eq!(matcher.normalize_tag("ml"), "ai");
        assert_eq!(matcher.normalize_tag("ML"), "ai");
        assert_eq!(matcher.normalize_tag("artificial-intelligence"), "ai");
        assert_eq!(matcher.normalize_tag("rust"), "rust"); // No alias
    }

    #[test]
    fn test_check_keywords() {
        let matcher = create_test_matcher();
        let keywords = vec!["rust".to_string(), "cargo".to_string()];

        // Should match with good confidence
        assert!(
            matcher
                .check_keywords("rust and cargo are great", &keywords)
                .unwrap()
                > 0.5
        );

        // Should match with lower confidence
        assert!(matcher.check_keywords("rust is great", &keywords).unwrap() >= 0.33);

        // Should not match
        assert!(matcher
            .check_keywords("python is also good", &keywords)
            .is_none());
    }
}
