# Feed Management and Categorization System

## Overview

This document tracks remaining work and known issues for the feed categorization system. The system ships with curated default feeds and categorization rules while maintaining full user customization capabilities.

## Architecture

The system uses a dual-axis model for organizing content:
- **Quality Axis (User-Defined):** Feeds are manually assigned to tiers (`loved`, `liked`, `new`) by the user in `spacefeeder.toml`.
- **Topic Axis (Auto-Generated):** Articles are automatically tagged with topics (e.g., `rust`, `ai`, `devops`) based on a set of rules.

It operates on a "batteries-included" principle, shipping with a default registry of feeds and rules that can be fully customized or extended by the user.

## Critical Issues to Address

### 1. RSS Feed Tag Noise - ROOT CAUSE IDENTIFIED ✅ ANALYZED - HIGH PRIORITY
- **Root Cause**: Most RSS feeds only provide 10-30 recent entries, not comprehensive historical data
- **Evidence**: Feed analysis shows only 2 feeds (smallcultfollowing, danluu) provide >100 entries with historical coverage
- **Impact**: 55.8% singleton tags are not just noise, but result of insufficient training data
- **Secondary Impact**: Quirky tags like "pelican-riding-a-bicycle" appear singleton because we only see 2/57 of Simon Willison's posts with that tag
- **Solution**: Increase feed coverage by configuring larger entry limits or finding paginated feed access

### 2. Multi-word Tag Normalization
- **Problem**: Tags with spaces like "boss politics antitrust" and special characters like "observability 2.0" need normalization
- **Impact**: Inconsistent categorization and poor UX in category browsing
- **Solution**: Expand alias system to normalize multi-word tags to hyphenated forms

### 3. Feed-Level Tag System ✅ RESOLVED
- **Status**: System now uses confidence boosting rather than forced tagging
- **Implementation**: Feed tags provide 1.2x confidence multiplier, capped at 0.95

## Remaining Work

### Community Contribution Framework
**Status**: Not started
**Priority**: Medium

**Tasks**:
- Document feed contribution process with clear guidelines
- Create validation tools for testing feed registry and rule changes
- Set up automated CI testing for categorization rules
- Create templates for common feed patterns
- Build a test suite that validates categorization accuracy

### Version Bumping Automation
**Status**: Manual process, error-prone
**Priority**: High

**Problem**: Frequently forget to bump `spacefeeder/Cargo.toml` and `spacefeeder/Cargo.lock` versions
**Solution Options**:
- Add pre-commit hook to check version consistency
- Create a `just release` command that handles version bumping
- Add CI check that fails if versions are inconsistent

### Feed Promotion Workflow
**Status**: Conceptual
**Priority**: Low

**Goal**: Feeds in `spacefeeder.toml` should "graduate" to `data/feeds.toml` when proven valuable
**Needs**:
- Define criteria for promotion (usage stats, quality metrics)
- Build tooling to suggest feeds for promotion
- Create workflow for reviewing and accepting promotions

### Static Analysis Tools ✅ COMPLETED
**Status**: Rust and Python analysis tools implemented
**Priority**: Medium

**Implementation**: 
- `spacefeeder analyze-feeds` - Rust-based feed coverage analysis using native feed-rs parser
- `tools/analyze_tags.py` - Python-based intelligent statistical analysis of tag quality and distribution
- Detects singleton tags, proper nouns, multi-word tags, and distribution issues
- Provides actionable recommendations for improving categorization
- Integrates with justfile: `just analyze_feeds` and `just analyze_tags [summary|detailed|json]`

**Key Features**:
- Statistical distribution analysis (frequency, concentration, outliers)
- Heuristic quality assessment (length, complexity, proper nouns)
- Semantic similarity detection for potential duplicates
- Co-occurrence analysis for related tags

## Technical Debt

### Configuration Complexity
- Merging logic between built-in data and user config is complex
- Need clearer separation of concerns between system and user data
- Consider moving to a layered configuration model

### Testing Gaps
- No automated tests for categorization accuracy on real feeds
- Missing integration tests for feed graduation workflow
- Need benchmarks for categorization performance on large datasets

## Implementation Status Summary

The feed management system has evolved from a simple RSS aggregator into a package manager-like experience. Core functionality is complete:
- Built-in feed registry with 14 curated feeds
- CLI commands for feed discovery and management
- Advanced categorization rules with confidence scoring
- Word boundary matching to prevent false positives
- Static analysis tooling for tag quality assessment

## Root Cause Analysis

**Core Problem Identified**: Limited historical feed data is the root cause of categorization issues.

Analysis tools reveal most RSS feeds only provide 10-30 recent entries despite authors having extensive publication histories. This means our categorization system operates on insufficient training data, causing legitimate topic tags to appear as singletons.

**Technical Investigation Needed**:
1. Check if spacefeeder's `max_articles_for_search=200` setting is being applied to feed parsing
2. Investigate whether RSS feeds provide pagination or historical archives beyond default entries  
3. Consider alternative data sources (archive.org, feed archives) for comprehensive historical data

Use `just analyze_feeds` and `just analyze_tags` to get current metrics and feed coverage details.