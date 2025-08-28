# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

This is a single-component static site generator for RSS/Atom feeds:

- **spacefeeder**: Rust CLI tool that fetches RSS/Atom feeds and generates a complete static website

The workflow: spacefeeder processes feeds from `spacefeeder.toml` → fetches feed data → generates HTML using templates → outputs complete static site to `public/`.

## Source Control (Jujutsu)

**CRITICAL: Commit changes in small increments. Never skip commits.**

```bash
# View status
jj st

# Stage and commit with message
jj commit -m "Your commit message"

# Update main bookmark after committing
jj bookmark move main --to @-

# Push to GitHub
jj git push

# Describe current change without committing
jj describe -m "Description of work in progress"
```

## Core Commands

All development tasks use the `justfile` (Just task runner):

```bash
# Build entire site (fetches feeds + builds static site)
just build

# Development server
just serve

# Fetch feeds only (updates data)
just fetch_feeds

# Search the site
just search <query>

# --- Feed Management ---

# Add a feed from the built-in registry to your config
just spacefeeder feeds add <slug>

# List all feeds in your config
just spacefeeder feeds list

# Search for feeds in the built-in registry
just spacefeeder feeds search <query>

# Find a feed URL from a website
just find_feed <base_url>
```

## Project Structure

- `spacefeeder.toml`: User-specific feed configuration (tiers).
- `data/feeds.toml`: The built-in registry of default feeds.
- `data/categorization.toml`: The built-in rules for auto-tagging articles.
- `data/tags.toml`: The built-in tag definitions.
- `content/data/`: JSON output from spacefeeder (feedData.json, itemData.json).
- `templates/`: Tera templates for HTML generation.
- `spacefeeder/src/commands/`: CLI command implementations.
- `spacefeeder/src/config/`: Modular configuration handling (merging, loading, saving).

## Configuration

- The system uses a dual-config model:
    1. **Built-in data**: `data/feeds.toml` and `data/categorization.toml` provide a curated starting point.
    2. **User config**: `spacefeeder.toml` allows users to specify which feeds they want and assign them to tiers (`new`, `like`, `love`).
- The application merges these sources, with user configuration taking precedence.

### Categorization System

The system uses a flexible, rule-based engine to automatically categorize articles. Key features include:

- **Rule Types**: Matching on title, content, URL, author, and feed slug.
- **Advanced Rules**: Can require multiple keywords or combine author matching with content analysis to reduce false positives.
- **Exclusion Rules**: Prevent tagging on certain articles (e.g., link roundups).
- **Confidence Scoring**: Filters out low-confidence tag matches.
- **Aliases**: Normalize tags (e.g., "rustlang" → "rust").

Example rule from `data/categorization.toml`:
```toml
[[rules]]
type = "author_with_content"
patterns = ["Simon Willison"]
required_keywords = ["ai", "llm", "gpt", "claude", "machine learning"]
tag = "ai"
confidence = 0.8
```

## Testing

The Rust component uses standard Cargo testing with `test-case` for parameterized tests. Test data is in `spacefeeder/src/test_data/`.
- Run cargo fmt and cargo clippy after editing rust code