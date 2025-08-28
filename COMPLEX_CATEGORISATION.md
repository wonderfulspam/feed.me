# Feed Management and Categorization System

## Overview

This document outlines the evolution of feed.me's categorization system into a package manager-like experience for RSS/Atom feeds. The system will ship with curated default feeds and categorization rules while maintaining full user customization capabilities.

## Current State

### Categorization System Status
- Basic pattern-based tagging implemented
- Configuration-driven rules stored in `spacefeeder.toml`
- Tag normalization and confidence scoring functional
- # Feed Management and Categorization System

## Overview

This document outlines the evolution of feed.me's categorization system into a package manager-like experience for RSS/Atom feeds. The system ships with curated default feeds and categorization rules while maintaining full user customization capabilities.

## Architecture

The system uses a dual-axis model for organizing content:
- **Quality Axis (User-Defined):** Feeds are manually assigned to tiers (`loved`, `liked`, `new`) by the user in `spacefeeder.toml`.
- **Topic Axis (Auto-Generated):** Articles are automatically tagged with topics (e.g., `rust`, `ai`, `devops`) based on a set of rules.

It operates on a "batteries-included" principle, shipping with a default registry of feeds and rules that can be fully customized or extended by the user.

## Known Issues and Future Work

### 1. Improve Feed-Level Tagging
- **Problem**: Manual tags assigned to a feed in `data/feeds.toml` (e.g., Simon Willison's "ai" tag) apply to *all* articles from that feed, regardless of content. This causes false positives for authors who cover multiple topics.
- **Desired Behavior**: Feed-level tags should act as confidence boosters rather than absolute assignments.
- **Technical Need**: Implement a weighted tag system where feed-level tags influence an article's tag confidence score but don't guarantee inclusion.

### 2. Refine Aggregator Categorization
- **Problem**: Articles from aggregators like Hacker News are sometimes assigned inappropriate tags due to broad keyword matching on diverse content.
- **Desired Behavior**: Reduce false positives for aggregators.
- **Technical Need**: Investigate smarter detection and handling of link aggregators, potentially with more restrictive rules or a separate processing pipeline.

### 3. Fine-Tune Author-Based Rules
- **Problem**: The `author_with_content` rule type can be too restrictive, requiring all specified keywords to be present.
- **Desired Behavior**: More flexible author-based rules.
- **Technical Need**: Allow for `any` keyword match in addition to the current `all` match.

### 4. Build a Community Contribution Framework
- **Goal**: Make it easy for others to contribute feeds and categorization improvements.
- **Tasks**:
    - Document the feed contribution process.
    - Create validation tools to test new feed registry and rule changes.
    - Set up automated CI testing for categorization rules.
    - Create templates for common feed patterns.

- **Issues**: Crude categorization rules producing false positives (e.g., Simon Willison articles incorrectly tagged with "python")

### User Experience Gaps
- Manual feed discovery and configuration required
- Complex `spacefeeder.toml` configuration barrier for new users
- No standardized feed metadata or categorization rules
- Difficult for new contributors to add well-categorized feeds

## Target Architecture

### Package Manager Model
- **Built-in Feed Registry**: Curated collection of feeds with metadata and categorization rules
- **Simplified User Config**: `spacefeeder.toml` contains only feed slugs and personal preferences (tiers)
- **Default Data Sources**: Feed metadata, categorization rules, and aliases built into the application
- **CLI Feed Management**: Commands to search, install, configure, and remove feeds

### Dual-Axis System (Unchanged)
```
Quality Axis (User-Defined)     Topic Axis (Auto-Generated)
├── loved (★★★)                ├── AI/ML
├── liked (★★)                 ├── Rust  
└── new   (★)                  ├── DevOps
                               └── [system + user-defined]
```

### Design Principles
1. **Batteries Included**: Ship with useful defaults for immediate value
2. **Progressive Disclosure**: Simple initial experience, advanced features available
3. **Full Customization**: All defaults can be overridden or extended
4. **Community Driven**: Easy contribution of new feeds and categorization rules

## Implementation Plan

### Phase 1: Built-in Feed Registry

#### Default Data Sources
Store feed registry and categorization rules as embedded resources:

```rust
// Embedded at compile time
const DEFAULT_FEEDS: &str = include_str!("data/feeds.toml");
const DEFAULT_CATEGORIZATION: &str = include_str!("data/categorization.toml");
const DEFAULT_TAGS: &str = include_str!("data/tags.toml");
```

#### Feed Registry Structure
```toml
# data/feeds.toml
[feeds.matklad]
url = "https://matklad.github.io/feed.xml"
author = "Alex Kladov"
description = "Rust compiler development and programming insights"
tags = ["rust", "compilers", "programming"]

[feeds.simonwillison]
url = "https://simonwillison.net/atom/everything/"
author = "Simon Willison"
description = "AI, databases, and web development"
tags = ["ai", "databases", "web"]
```

### Phase 2: CLI Feed Management

#### Core Commands
```bash
# Search available feeds
spacefeeder feeds search "rust"
spacefeeder feeds search --tag "ai"

# Install (add) feeds to user config
spacefeeder feeds add matklad
spacefeeder feeds add simonwillison --tier like

# List installed feeds
spacefeeder feeds list
spacefeeder feeds list --tier love

# Configure existing feeds
spacefeeder feeds configure matklad --tier love
spacefeeder feeds configure simonwillison --tags "+python,-ai"

# Remove feeds
spacefeeder feeds remove matklad

# Show feed information
spacefeeder feeds info matklad
```

#### Simplified User Configuration
```toml
# User's spacefeeder.toml (simplified)
[feeds]
matklad = { tier = "love" }
simonwillison = { tier = "like" }
hackernews = {}  # tier defaults to "new"

# Optional user customizations
[tags.overrides]
# Add custom tags or override defaults
rust.keywords = ["rust", "cargo", "rustc", "oxidize"]

[categorization.rules]
# Additional user-defined rules
[[categorization.rules]]
type = "url_contains"
patterns = ["github.com/rust-lang"]
tag = "rust-official"
```

### Phase 3: Categorization Rule Improvements

#### Current Issues and Solutions
**Problem**: Overly broad pattern matching causes false positives
- Simon Willison articles tagged "python" regardless of content
- Author-based rules too aggressive

**Solutions**:
1. **Content-based matching**: Analyze article title and description, not just author
2. **Confidence thresholds**: Require higher confidence for author-based tags
3. **Negative patterns**: Rules to exclude certain content

#### Improved Rule Structure ✅ IMPLEMENTED
```toml
# data/categorization.toml - More precise rules
[[rules]]
type = "content_analysis"
patterns = ["rust", "cargo", "rustc", "crate"]
tag = "rust"
confidence = 0.8
min_keyword_count = 2  # Require multiple keyword matches

[[rules]]
type = "author_with_content"
patterns = ["Simon Willison"]
required_keywords = ["python", "django", "pip", "pypi"]
tag = "python"
confidence = 0.7  # Lower confidence for author-based rules

# Negative rules to prevent false positives
[[rules]]
type = "exclude_if"
patterns = ["weekly links", "link roundup", "news roundup"]
exclude_tags = ["python", "rust", "ai"]  # Don't auto-tag link roundups
```

## Technical Implementation

### Data Layer Architecture
```rust
struct FeedRegistry {
    feeds: HashMap<String, DefaultFeed>,
    categorization: CategorizationConfig,
    tags: TagConfig,
}

struct DefaultFeed {
    url: String,
    author: String,
    description: String,
    tags: Vec<String>,
    // No tier - that's user-specific
}

struct UserConfig {
    feeds: HashMap<String, UserFeedConfig>,
    overrides: Option<ConfigOverrides>,
}

struct UserFeedConfig {
    tier: Option<Tier>,
    disabled_tags: Vec<String>,
    additional_tags: Vec<String>,
}
```

## Implementation Roadmap

### Milestone 1: Built-in Feed Registry ✅ COMPLETED
**Goal**: Ship with curated default feeds and categorization rules

**Tasks**:
- [x] Create embedded data files (`data/tags.toml`) 
- [x] Create `data/feeds.toml` and `data/categorization.toml`
- [x] Curate initial set of high-quality feeds with proper categorization
- [x] Implement registry loading with `include_str!` macros (for all data)
- [x] Update config parsing to merge defaults with user overrides (for all data)
- [x] Extend merging logic to feeds and categorization rules
- [x] Add description field to FeedInfo for better metadata
- [x] Support minimal user config with tier-only specifications

**Success Criteria**: ✅ New users get useful content immediately without configuration
- 14 curated default feeds with descriptions and tags
- Improved categorization rules with reduced false positives
- Users can specify minimal config: `[feeds.matklad] tier = "love"`

### Milestone 2: CLI Feed Management ✅ COMPLETED
**Goal**: Package manager-like commands for feed discovery and installation

**Tasks**:
- [x] Implement `spacefeeder feeds search <query>` command
- [x] Implement `spacefeeder feeds add <slug>` command  
- [x] Implement `spacefeeder feeds list` and `spacefeeder feeds info <slug>` commands
- [x] Implement `spacefeeder feeds configure <slug>` for tier management
- [x] Implement `spacefeeder feeds remove <slug>` command

**Success Criteria**: ✅ Users can discover, install, and manage feeds without editing TOML files

### Milestone 3: Improved Categorization Rules ✅ COMPLETED
**Goal**: Reduce false positives and improve categorization accuracy

**Tasks**:
- [x] Implement content-based analysis (not just author-based)
- [x] Add confidence scoring and thresholds
- [x] Implement negative pattern matching to exclude certain content
- [x] Add support for multiple keyword requirements
- [x] Test and tune rules against existing feed content

**Success Criteria**: ✅ Categorization accuracy improves, fewer false positives

### Milestone 4: Community Contribution Framework
**Goal**: Make it easy for others to contribute feeds and categorization improvements

**Tasks**:
- [ ] Document feed contribution process
- [ ] Create validation tools for feed registry changes
- [ ] Set up automated testing of categorization rules
- [ ] Create templates for common feed patterns

**Success Criteria**: External contributors can easily add new feeds with proper metadata

## Next Actions

1. **✅ COMPLETED**: Milestone 1 fully implemented with:
   - Built-in feed registry with 14 curated feeds
   - Improved categorization rules with reduced false positives 
   - Support for minimal user configuration
   - All default data embedded at compile time

2. **✅ COMPLETED**: Milestone 2 - CLI Feed Management:
   - Package manager-like `feeds` subcommand with search, add, list, info, configure, remove
   - Feed discovery without config file dependency (search and info commands)
   - Simple feed installation and management workflow
   - Enhanced CLI help descriptions for better user experience

3. **✅ COMPLETED**: Milestone 3 - Advanced Categorization:
   - Extended TagRule structure with advanced fields (exclude_patterns, min_keyword_count, required_keywords, exclude_tags)
   - Added content_analysis rule type with multi-keyword requirements
   - Added author_with_content rule type to reduce author-based false positives  
   - Added exclude_if rule type to prevent tagging of link roundups and announcements
   - Improved keyword matching with word boundaries to prevent false matches ("ai" no longer matches "said")
   - Comprehensive test suite covering realistic false positive scenarios
   - Improved categorization.toml with sophisticated rules demonstrating new features

## Current State and Known Limitations

### What's Working Well ✅
- **Feed Registry**: 14 curated feeds with descriptions and categorization rules
- **CLI Commands**: Package manager-like interface (search, add, list, info, configure, remove)
- **Advanced Rules**: Content analysis, exclusion patterns, multi-keyword requirements
- **Word Boundaries**: Precise keyword matching prevents most false positives
- **Test Coverage**: Comprehensive tests ensure categorization logic works correctly

### Known Issues and Shortcomings ⚠️
1. **Feed-Level Tags Override Content Analysis**: Manual tags assigned to feeds (e.g., Simon Willison's "ai" tag) apply to ALL articles from that feed, regardless of content. This causes false positives for multi-topic authors.
   - **Impact**: Simon Willison articles about web development still get AI tags
   - **Desired Behavior**: Feed-level tags should act as boosters rather than absolute assignments
   - **Technical Need**: Implement weighted tag system where feed tags influence confidence but don't guarantee inclusion

2. **Hacker News Over-Categorization**: Some random articles still get inappropriate tags
   - **Root Cause**: Keyword matching in titles/descriptions of aggregated content
   - **Mitigation**: Exclude rules help but don't cover all cases

3. **Author-Based Rules Need Refinement**: Current author_with_content rules require all keywords, which may be too restrictive for some cases

### Next Priority Improvements
1. **Weighted Tag System**: Convert feed-level tags from absolute assignments to confidence boosters
2. **Smarter Aggregator Handling**: Better detection and handling of link aggregators like Hacker News
3. **Confidence Thresholds**: Fine-tune confidence scoring for different rule types
4. **Content-Only Mode**: Option to ignore feed-level tags for testing accuracy

4. **Future: Milestone 4 - Community Contribution Framework**:
   - Document feed contribution process
   - Create validation tools for feed registry changes  
   - Set up automated testing of categorization rules
   - Create templates for common feed patterns

## Technical Notes

- Use `include_str!` for embedding data files at compile time
- Keep user config minimal - just feed slugs and personal preferences (tiers)
- All default data should be declarative and easily auditable
- Categorization rules should be tunable without code changes
- No external users yet - breaking changes are acceptable