# Future Work and Technical Improvements

This document tracks remaining improvements and feature ideas for the `feed.me` project.

## Guiding Principles

- **Incremental Improvement**: The current architecture is solid. Focus on additive features rather than major refactoring.
- **Simplicity**: Prioritize simple, robust solutions. For example, use `localStorage` for client-side state to avoid server-side complexity.
- **Unix Philosophy**: Keep the CLI tool focused and composable.

## High-Priority Improvements

### 1. Package Manager Distribution
- **Problem**: Installation currently requires cloning the repository and building from source (`git clone` + `cargo build`).
- **Goal**: Lower the barrier to entry for new users with a one-command installation.
- **Implementation**: Publish the `spacefeeder` crate to `crates.io` and investigate creating a Homebrew tap.

## Medium-Priority Improvements

### 1. Enhanced Feed Discovery
- **Problem**: Adding feeds requires manually finding and entering the feed URL.
- **Goal**: Make feed discovery and management easier.
- **Ideas**:
    - Improve the `find-feed` command to reliably auto-discover feeds from a website URL.
    - Enhance OPML import/export functionality.
    - Add a feature to recommend popular or curated feeds.

### 2. Web-Based Management UI
- **Problem**: All configuration is done by editing the `spacefeeder.toml` file.
- **Goal**: Provide a more user-friendly configuration experience.
- **Implementation**: A web-based UI for managing feeds, tiers, and settings. This would require UX design input.

## Low-Priority / Investigated Ideas

The following ideas were previously considered but are deemed low-priority as they provide minimal practical benefit for the project's current scope. This is recorded here to avoid re-litigating settled decisions.

- **Feed Health Monitoring**: The current error reporting during `fetch` is sufficient.
- **Duplicate Content Detection**: No duplicates have been observed in practice.
- **Performance Caching**: Current performance (~2 seconds for 23 feeds) is acceptable.
