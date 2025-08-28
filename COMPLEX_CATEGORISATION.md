# Feed Management and Categorization System

## Overview

This document tracks remaining work and known issues for the feed categorization system. The system ships with curated default feeds and categorization rules while maintaining full user customization capabilities.

## Architecture

The system uses a dual-axis model for organizing content:
- **Quality Axis (User-Defined):** Feeds are manually assigned to tiers (`loved`, `liked`, `new`) by the user in `spacefeeder.toml`.
- **Topic Axis (Auto-Generated):** Articles are automatically tagged with topics (e.g., `rust`, `ai`, `devops`) based on a set of rules.

It operates on a "batteries-included" principle, shipping with a default registry of feeds and rules that can be fully customized or extended by the user.

## Critical Issues to Address

### 1. Feed-Level Tag System Overhaul
- **Problem**: Manual tags in `data/feeds.toml` apply to ALL articles from a feed, causing false positives for multi-topic authors
- **Impact**: Simon Willison articles about web development incorrectly get "ai" tags
- **Solution**: Convert feed-level tags from absolute assignments to confidence boosters
- **Implementation**: Weighted tag system where feed tags influence confidence scores without guaranteeing inclusion

### 2. Aggregator Content Handling  
- **Problem**: Hacker News and similar aggregators get inappropriate tags from broad keyword matching
- **Solution**: Implement specialized handling for link aggregators with more restrictive rules or separate pipeline

### 3. Author-Based Rule Flexibility
- **Problem**: `author_with_content` rules require ALL specified keywords (too restrictive)
- **Solution**: Add support for `any` keyword matching in addition to current `all` match mode

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

### Static Analysis Tools
**Status**: Not implemented
**Priority**: Medium

**Goal**: Lightweight Rust-first tools to find categorization mismatches
**Ideas**:
- Sampling tool to test categorization rules against real feed content
- Mismatch detector comparing expected vs actual tags
- Statistical analyzer for tag distribution and anomalies
- Agent slash commands for dynamic testing

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

However, several critical issues remain that impact user experience and system accuracy.