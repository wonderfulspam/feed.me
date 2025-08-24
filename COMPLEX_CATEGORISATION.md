# Feed Management and Categorization System

## Overview

This document outlines the evolution of feed.me's categorization system into a package manager-like experience for RSS/Atom feeds. The system will ship with curated default feeds and categorization rules while maintaining full user customization capabilities.

## Current State

### Categorization System Status
- Basic pattern-based tagging implemented
- Configuration-driven rules stored in `spacefeeder.toml`
- Tag normalization and confidence scoring functional
- **✅ DONE**: Tags migrated to `data/tags.toml` and embedded at compile time
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

#### Improved Rule Structure
```toml
# data/categorization.toml - More precise rules
[[rules]]
type = "content_analysis"
keywords = ["rust", "cargo", "rustc"]
tag = "rust"
confidence = 0.8
min_keyword_count = 2  # Require multiple keyword matches

[[rules]]
type = "author_with_content"
author = "Simon Willison"
required_keywords = ["python", "django", "pip"]
tag = "python"
confidence = 0.7  # Lower confidence for author-based rules

# Negative rules to prevent false positives
[[rules]]
type = "exclude_if"
patterns = ["AI news roundup", "weekly links"]
exclude_tags = ["python", "rust"]  # Don't auto-tag link roundups
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

### Milestone 3: Improved Categorization Rules
**Goal**: Reduce false positives and improve categorization accuracy

**Tasks**:
- [ ] Implement content-based analysis (not just author-based)
- [ ] Add confidence scoring and thresholds
- [ ] Implement negative pattern matching to exclude certain content
- [ ] Add support for multiple keyword requirements
- [ ] Test and tune rules against existing feed content

**Success Criteria**: Categorization accuracy improves, fewer false positives

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

3. **Next: Milestone 3 - Advanced Categorization**:
   - Implement content-based analysis (beyond pattern matching)
   - Add confidence scoring and thresholds
   - Implement negative pattern matching
   - Add support for multiple keyword requirements

## Technical Notes

- Use `include_str!` for embedding data files at compile time
- Keep user config minimal - just feed slugs and personal preferences (tiers)
- All default data should be declarative and easily auditable
- Categorization rules should be tunable without code changes
- No external users yet - breaking changes are acceptable