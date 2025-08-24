# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

This is a dual-component static site generator for RSS/Atom feeds:

- **spacefeeder**: Rust CLI tool that fetches RSS/Atom feeds and generates JSON data files
- **Zola website**: Static site generator that consumes the JSON data to create a personal RSS reader homepage

The workflow: spacefeeder processes feeds from `spacefeeder.toml` → generates JSON in `content/data/` → Zola builds static site using templates.

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

# Development server with auto-reload
just serve

# Fetch feeds only (updates JSON data)
just fetch_feeds

# Add a new feed
just add_feed <slug> <url> <author> <tier>

# Export feeds to OPML
just export_feeds

# Find RSS/Atom feed from a website URL
just find_feed <base_url>
```

## Project Structure

- `spacefeeder.toml`: Feed configuration with tiers (new/like/love)
- `content/data/`: JSON output from spacefeeder (feedData.json, itemData.json)
- `templates/`: Zola HTML templates
- `static/css/`: Stylesheets
- `spacefeeder/src/commands/`: CLI command implementations (add_feed, fetch_feeds, etc.)
- `spacefeeder/src/config.rs`: TOML configuration handling

## Configuration

- Feeds are organized by tiers: "new", "like", "love"
- `max_articles = 50` limits per-feed items
- `description_max_words = 150` truncates descriptions
- Feed data outputs to `content/data/` for Zola consumption

## Testing

The Rust component uses standard Cargo testing with `test-case` for parameterized tests. Test data is in `spacefeeder/src/test_data/`.