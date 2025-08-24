# Complex Categorization System - IMPLEMENTED ✓

## Executive Summary

**STATUS: FULLY IMPLEMENTED** ✅

This document described the plan to evolve feed.me's categorization system from the current three-tier model (new/liked/loved) to a more sophisticated dual-axis system combining quality tiers with topical tags. The implementation is complete and provides better content discovery, filtering, and organization while maintaining backward compatibility.

## Implementation Summary

The categorization system has been successfully implemented with the following features:

✅ **Configuration-driven approach** - All rules stored in `spacefeeder.toml`  
✅ **Pattern-based tagging** - Title, content, URL, author, and feed slug patterns  
✅ **Keyword extraction** - Auto-tagging based on configured keywords  
✅ **Tag normalization** - Aliases handle variations like "rustlang" → "rust"  
✅ **Confidence scoring** - Filter low-quality tags with thresholds  
✅ **Visual display** - Tags shown with semantic colors in the UI  
✅ **Search integration** - Tags are searchable through the search index  

**Key Results:**
- Articles now display relevant tags (e.g., "rust", "ai", "devops")
- Tags are colored semantically (Rust = amber, AI = purple, etc.)
- No hardcoded author/source handling - all rules are configurable
- Backward compatible - existing tier system unchanged

## Current State Analysis

### Existing System
- **Tiers**: new, liked, loved (representing quality/curation level)
- **Limitations**: 
  - No topical organization
  - Confusion between quality rating and starring individual articles
  - Limited filtering capabilities

### Feed Content Analysis
Based on search analysis of current feeds, the dominant topics are:
- **AI/ML**: LLMs, AI agents, Claude, GPT (30+ articles)
- **Programming**: Rust (25+ articles), Python, general development
- **DevOps/Infrastructure**: CI/CD, cloud, containers (20+ articles)
- **Security**: Vulnerabilities, bug bounties, privacy (10+ articles)
- **Linux/Open Source**: NixOS, Linux desktop, FOSS tools (15+ articles)
- **Tech Industry**: Company news, products, analysis (20+ articles)

### RSS/Atom Category Support
- Some feeds (e.g., Atlassian) include `<category>` tags
- Most feeds lack category metadata
- YouTube/custom feeds have no category information
- **Conclusion**: Cannot rely solely on feed-provided categories

## Proposed Architecture

### Dual-Axis System

```
Quality Axis (Tiers)          Topic Axis (Tags)
├── loved (★★★)               ├── AI/ML
├── liked (★★)                ├── Rust
└── new   (★)                 ├── DevOps
                              ├── Security
                              ├── Cloud
                              ├── Linux
                              └── [user-defined]
```

### Key Design Principles
1. **Orthogonal Concerns**: Tiers and tags serve different purposes
2. **Progressive Enhancement**: Start simple, add sophistication over time
3. **User Control**: Allow manual override of automatic tagging
4. **Feed & Article Granularity**: Tags can apply to both feeds and individual articles

## Implementation Strategy

### Phase 1: Data Model Enhancement
```rust
// Current
struct Feed {
    tier: Tier,  // new/liked/loved
    // ...
}

// Proposed
struct Feed {
    tier: Tier,           // Quality rating (unchanged)
    tags: Vec<String>,    // Feed-level tags
    auto_tag: bool,       // Enable auto-tagging for this feed
    // ...
}

struct Article {
    // ... existing fields
    tags: Vec<String>,         // Article-specific tags
    inferred_tags: Vec<String>, // Auto-detected tags (kept separate)
    starred: bool,             // Individual article starring
}
```

### Phase 2: Tagging Strategies

#### 2.1 Manual Feed Tagging
- Add tags to feeds in `spacefeeder.toml`
- Example:
```toml
[feeds.matklad]
url = "https://matklad.github.io/feed.xml"
author = "Alex Kladov"
tier = "like"
tags = ["rust", "programming", "compilers"]
```

#### 2.2 RSS Category Import
- Extract `<category>` tags from RSS/Atom feeds when available
- Map common variations (e.g., "DevOps" → "devops")
- Store as initial tag suggestions

#### 2.3 Keyword-Based Auto-Tagging
Simple implementation using `keyword_extraction` crate:
```rust
use keyword_extraction::rake::Rake;

fn extract_tags(title: &str, description: &str) -> Vec<String> {
    let text = format!("{} {}", title, description);
    let rake = Rake::new();
    let keywords = rake.extract_keywords(&text, 5);
    
    // Match against predefined tag list
    match_to_known_tags(keywords)
}
```

#### 2.4 Advanced NLP Tagging (Future)
Options for more sophisticated classification:
- **rust-bert**: Pre-trained models for text classification
- **OpenAI API**: Use GPT for tag suggestions (requires API key)
- **Local LLM**: Run Llama.cpp with small model for privacy

### Phase 3: Migration Path

#### Step 1: Backward Compatible Addition
1. Add tag fields to data structures
2. Keep tier system unchanged
3. Deploy without breaking existing functionality

#### Step 2: Initial Tag Population
```rust
// Auto-populate tags based on feed characteristics
fn migrate_feed_tags(feed: &Feed) -> Vec<String> {
    let mut tags = Vec::new();
    
    // Analyze feed URL/author for hints
    if feed.url.contains("rust") || feed.author.contains("rust") {
        tags.push("rust".to_string());
    }
    if feed.url.contains("devops") || feed.slug.contains("devops") {
        tags.push("devops".to_string());
    }
    
    tags
}
```

#### Step 3: User Interface Updates
- Add tag filters to web interface
- Show tags alongside tier badges
- Allow tag-based search/filtering

## User-Defined Categories

### Configuration
```toml
[tags]
# Predefined tags with descriptions
predefined = [
    { name = "ai", description = "AI, ML, LLMs, and neural networks" },
    { name = "rust", description = "Rust programming language" },
    { name = "devops", description = "CI/CD, deployment, infrastructure" },
    { name = "security", description = "Security, vulnerabilities, privacy" },
]

# Tag aliases for normalization
aliases = [
    { from = ["artificial-intelligence", "machine-learning"], to = "ai" },
    { from = ["rustlang", "rust-lang"], to = "rust" },
]

# Auto-tagging rules
[tags.rules]
title_keywords = [
    { keywords = ["rust", "cargo", "crate"], tag = "rust" },
    { keywords = ["ai", "llm", "gpt", "claude"], tag = "ai" },
]
```

## Technical Implementation

### Search Integration
Extend existing search to support tag queries:
```bash
just search "rust" --tags "ai,security"  # Articles about Rust in AI or Security contexts
just search --tag "devops"               # All DevOps articles
```

### Database Schema
```sql
-- New tables for tag support
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT
);

CREATE TABLE feed_tags (
    feed_id TEXT,
    tag_id INTEGER,
    confidence REAL,  -- For auto-tagged items
    FOREIGN KEY (feed_id) REFERENCES feeds(slug),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);

CREATE TABLE article_tags (
    article_id TEXT,
    tag_id INTEGER,
    source TEXT,  -- 'manual', 'feed', 'auto'
    confidence REAL,
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);
```

## Performance Considerations

### Caching Strategy
- Cache tag extraction results
- Batch process new articles
- Use background job for auto-tagging

### Optimization
```rust
// Use bloom filter for quick tag matching
use bloom::BloomFilter;

struct TagMatcher {
    filters: HashMap<String, BloomFilter>,
}

impl TagMatcher {
    fn might_match(&self, text: &str, tag: &str) -> bool {
        self.filters.get(tag)
            .map(|f| f.might_contain(text))
            .unwrap_or(false)
    }
}
```

## UI/UX Considerations

### Visual Hierarchy
```
┌─────────────────────────────────────┐
│ 📰 Article Title                    │
│ Author Name | ★★ liked | 🏷️ rust, ai │
│ Description text...                 │
└─────────────────────────────────────┘
```

### Filtering Interface
- Checkbox list for tags
- Keep tier filter separate
- "AND" vs "OR" tag logic toggle
- Tag cloud visualization

## Rollout Plan

### Week 1-2: Foundation
- [ ] Update data models
- [ ] Add database migrations
- [ ] Implement basic tag storage

### Week 3-4: Auto-tagging
- [ ] Keyword extraction implementation
- [ ] RSS category import
- [ ] Initial tag population script

### Week 5-6: UI Integration
- [ ] Tag display in feed items
- [ ] Filter interface
- [ ] Search integration

### Week 7-8: Advanced Features
- [ ] User-defined tags
- [ ] Tag management interface
- [ ] Bulk tagging tools

## Success Metrics

1. **Adoption**: % of feeds with at least one tag
2. **Accuracy**: User satisfaction with auto-tagging
3. **Discovery**: Increase in cross-feed article discovery
4. **Performance**: No degradation in page load times

## Risk Mitigation

### Over-tagging
- Limit to 3-5 tags per article
- Confidence threshold for auto-tags
- User ability to hide/remove tags

### Under-tagging
- Suggest tags based on similar articles
- Periodic review of untagged content
- Community tagging features (future)

### Performance Impact
- Lazy-load tags
- Cache aggressively
- Background processing for auto-tagging

## Future Enhancements

### Phase 2 Features
- Tag hierarchies (e.g., "programming" > "rust")
- Tag relationships (e.g., "kubernetes" relates to "devops")
- Personal tag vocabularies
- Tag-based RSS feed generation

### Machine Learning Pipeline
```python
# Future: Training custom classifier
from sklearn.naive_bayes import MultinomialNB
from sklearn.feature_extraction.text import TfidfVectorizer

def train_classifier(articles_with_tags):
    vectorizer = TfidfVectorizer(max_features=1000)
    X = vectorizer.fit_transform([a.text for a in articles])
    y = [a.tags for a in articles]
    
    classifier = MultinomialNB()
    classifier.fit(X, y)
    return classifier, vectorizer
```

## Conclusion

This phased approach allows us to:
1. Maintain backward compatibility
2. Progressively enhance the system
3. Gather user feedback at each stage
4. Build a solid foundation for future ML-based categorization

The dual-axis system (tiers + tags) provides the flexibility users need while keeping the simplicity of the current tier system. By treating these as orthogonal concerns, we avoid confusion and enable more powerful filtering and discovery features.

## Appendix: Automatic Tag Derivation

### Configuration-Driven Tag System

**Key Principle**: No hardcoded handling of specific authors/sources. All categorization rules are stored in configuration and loaded at startup.

#### Default Configuration Structure

The system initializes with sensible defaults stored in `spacefeeder.toml`:

```toml
# Categorization configuration (loaded at startup)
[categorization]
enabled = true
auto_tag_new_articles = true
max_tags_per_item = 5
confidence_threshold = 0.3

# Predefined tags with descriptions
[[categorization.tags]]
name = "rust"
description = "Rust programming language"
keywords = ["rust", "cargo", "rustc", "rustup", "crate", "tokio"]

[[categorization.tags]]
name = "ai"
description = "AI, ML, LLMs, and neural networks"
keywords = ["ai", "artificial intelligence", "ml", "machine learning", "llm", "gpt", "claude"]

[[categorization.tags]]
name = "devops"
description = "CI/CD, deployment, infrastructure"
keywords = ["devops", "ci/cd", "cicd", "pipeline", "deployment", "kubernetes", "docker"]

[[categorization.tags]]
name = "security"
description = "Security, vulnerabilities, encryption"
keywords = ["security", "vulnerability", "cve", "encryption", "authentication", "exploit"]

[[categorization.tags]]
name = "linux"
description = "Linux, Unix, system administration"
keywords = ["linux", "unix", "nixos", "kernel", "systemd", "bash"]

# Pattern-based rules for automatic tagging
[[categorization.rules]]
type = "title_contains"
patterns = ["Rust", "rustc", "cargo"]
tag = "rust"
confidence = 0.9

[[categorization.rules]]
type = "url_contains"
patterns = ["rust-lang.org", "rustacean"]
tag = "rust"
confidence = 0.95

[[categorization.rules]]
type = "author_is"
patterns = ["Simon Willison"]
tags = ["ai", "python"]
confidence = 0.7

# Tag aliases for normalization
[[categorization.aliases]]
from = ["artificial-intelligence", "machine-learning"]
to = "ai"

[[categorization.aliases]]
from = ["rustlang", "rust-lang"]
to = "rust"
```

### Declarative Tag Generation

```rust
/// Load categorization rules from configuration
struct CategorizationEngine {
    config: CategorizationConfig,
    tag_matchers: Vec<TagMatcher>,
}

impl CategorizationEngine {
    /// Initialize from configuration at startup
    pub fn from_config(config: &Config) -> Self {
        let categorization = &config.categorization;
        let mut matchers = Vec::new();
        
        // Build matchers from configuration rules
        for rule in &categorization.rules {
            matchers.push(TagMatcher::from_rule(rule));
        }
        
        Self {
            config: categorization.clone(),
            tag_matchers: matchers,
        }
    }
    
    /// Apply all configured rules to generate tags
    pub fn generate_tags(&self, item: &FeedItem) -> Vec<Tag> {
        let mut tags = Vec::new();
        let mut seen = HashSet::new();
        
        for matcher in &self.tag_matchers {
            if let Some(matched_tags) = matcher.apply(item) {
                for tag in matched_tags {
                    // Apply aliases from config
                    let normalized = self.normalize_tag(&tag.name);
                    if seen.insert(normalized.clone()) {
                        tags.push(Tag {
                            name: normalized,
                            confidence: tag.confidence,
                            source: tag.source,
                        });
                    }
                }
            }
        }
        
        // Sort by confidence, limit to max_tags_per_item
        tags.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        tags.truncate(self.config.max_tags_per_item);
        
        tags
    }
    
    /// Normalize tag using configured aliases
    fn normalize_tag(&self, tag: &str) -> String {
        for alias in &self.config.aliases {
            if alias.from.contains(&tag.to_string()) {
                return alias.to.clone();
            }
        }
        tag.to_lowercase()
    }
}
```

### Tag Discovery Pipeline

1. **Initial Scan**: Analyze all existing articles for term frequency
2. **Pattern Matching**: Look for technology-specific keywords
3. **Threshold Filtering**: Only create tags with sufficient content
4. **Confidence Scoring**: Rate tag quality based on:
   - Frequency (how often it appears)
   - Specificity (how unique the term is)
   - Consistency (appears across multiple feeds)

### Avoiding Over-Broad Tags

These terms are TOO BROAD and should be avoided:
- ❌ `software` - Too generic
- ❌ `code` - Too generic  
- ❌ `technology` - Too generic
- ❌ `web` - Too generic
- ❌ `data` - Too generic
- ❌ `system` - Too generic
- ❌ `application` - Too generic
- ❌ `development` - Too generic

### Feed-Level Tag Configuration

Tags can be specified at the feed level in `spacefeeder.toml`:

```toml
# Example feed with explicit tags
[feeds.matklad]
url = "https://matklad.github.io/feed.xml"
author = "Alex Kladov"
tier = "like"
tags = ["rust", "compilers", "programming"]  # Manual tags
auto_tag = true  # Also apply automatic tagging rules

# Feed that relies entirely on automatic tagging
[feeds.simonwillison]
url = "https://simonwillison.net/atom/everything/"
author = "Simon Willison"
tier = "like"
auto_tag = true  # Will match configured rules for this author
```

The system applies tags in this priority order:
1. Explicit tags from feed configuration
2. Tags from RSS/Atom `<category>` elements
3. Tags from configured pattern matching rules
4. Tags from keyword extraction (if enabled)