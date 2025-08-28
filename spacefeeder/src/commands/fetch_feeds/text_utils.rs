use feed_rs::model::Entry;

/// Get description from RSS entry, trying different fields
pub fn get_description_from_entry(entry: Entry) -> Option<String> {
    // Try content first (usually the full content)
    if let Some(content) = entry.content {
        if let Some(ref body) = content.body {
            if !body.is_empty() {
                return Some(body.clone());
            }
        }
    }

    // Then try summary (usually a description/excerpt)
    if let Some(summary) = entry.summary {
        if !summary.content.is_empty() {
            return Some(summary.content);
        }
    }

    None
}

/// Truncate description to specified word count
pub fn get_short_description(description: String, max_words: usize) -> String {
    let words: Vec<&str> = description.split_whitespace().collect();
    if words.len() <= max_words {
        description
    } else {
        let truncated = words[..max_words].join(" ");
        format!("{}...", truncated)
    }
}

/// Extract first paragraph from text, useful for descriptions
pub fn extract_first_paragraph(text: &str) -> Option<String> {
    // First try to find first sentence ending
    if let Some(pos) = text.find(". ") {
        let sentence = &text[..pos + 1];
        if sentence.len() > 20 {
            // Only use if it's a reasonable length
            return Some(sentence.trim().to_string());
        }
    }

    // Then try paragraph breaks
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    if let Some(first) = paragraphs.first() {
        let trimmed = first.trim();
        if !trimmed.is_empty() && trimmed != text {
            // Only return if we actually found a paragraph break
            return Some(trimmed.to_string());
        }
    }

    // Last resort: take first reasonable chunk
    let words: Vec<&str> = text.split_whitespace().take(30).collect();
    if !words.is_empty() {
        Some(words.join(" "))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_short_description_exact_words() {
        let description = "This is a test description with exactly ten words here.".to_string();
        let result = get_short_description(description.clone(), 10);
        assert_eq!(
            result,
            "This is a test description with exactly ten words here."
        );
    }

    #[test]
    fn test_get_short_description_truncation() {
        let description = "This is a much longer description that definitely exceeds the word limit and should be truncated.".to_string();
        let result = get_short_description(description, 5);
        assert_eq!(result, "This is a much longer...");
    }

    #[test]
    fn test_extract_first_paragraph() {
        let text = "First paragraph here.\n\nSecond paragraph follows.";
        let result = extract_first_paragraph(text);
        assert_eq!(result, Some("First paragraph here.".to_string()));
    }

    #[test]
    fn test_extract_first_paragraph_sentence_ending() {
        let text = "This is the first sentence. This is the second sentence.";
        let result = extract_first_paragraph(text);
        assert_eq!(result, Some("This is the first sentence.".to_string()));
    }
}
