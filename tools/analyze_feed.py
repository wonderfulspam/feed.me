#!/usr/bin/env python3
"""
Feed Analysis Tool

Analyzes RSS/Atom feeds to understand their content coverage, 
entry counts, and date ranges.

Usage:
    python analyze_feed.py <feed_url>
    python analyze_feed.py --all-feeds [config_path]
"""

import xml.etree.ElementTree as ET
import sys
import re
import argparse
import requests
from datetime import datetime
from urllib.parse import urlparse
import toml
from pathlib import Path


def parse_feed(feed_url, verbose=False):
    """Parse a feed and extract metadata"""
    try:
        if verbose:
            print(f"   Fetching {feed_url}...")
        response = requests.get(feed_url, timeout=10)
        response.raise_for_status()
        
        data = response.text[:500000]  # Limit to first 500KB to avoid huge feeds
        
        if verbose:
            print(f"   Processing {len(data)} chars...")
            
        # Parse XML and handle namespaces properly
        try:
            root = ET.fromstring(data)
        except ET.ParseError:
            # Only do expensive regex cleanup if parsing fails
            if verbose:
                print("   Cleaning malformed XML...")
            data = re.sub(r'\s+xmlns[^=]*=\"[^\"]*\"', '', data)
            data = re.sub(r'<([a-zA-Z0-9]+):[a-zA-Z0-9]+\b', r'<\1', data)
            data = re.sub(r'</([a-zA-Z0-9]+):[a-zA-Z0-9]+>', r'</\1>', data)
            root = ET.fromstring(data)
        
        # Try both RSS and Atom formats with namespace awareness
        published_dates = []
        
        if verbose:
            print(f"   Root tag: {root.tag}")
        
        # RSS format (pubDate)
        pubdates = [elem.text for elem in root.iter() if elem.tag.endswith('pubDate') and elem.text]
        if pubdates:
            published_dates = pubdates
        
        # Atom format (published)
        if not published_dates:
            published = [elem.text for elem in root.iter() if elem.tag.endswith('published') and elem.text]
            if published:
                published_dates = published
        
        # Fallback to updated for Atom
        if not published_dates:
            updated = [elem.text for elem in root.iter() if elem.tag.endswith('updated') and elem.text]
            if updated:
                published_dates = updated
        
        # Get feed title
        title_elem = root.find('.//title')
        feed_title = title_elem.text if title_elem is not None else urlparse(feed_url).netloc
        
        return {
            'title': feed_title,
            'url': feed_url,
            'entry_count': len(published_dates),
            'dates': published_dates,
            'newest': published_dates[0] if published_dates else None,
            'oldest': published_dates[-1] if published_dates else None,
            'error': None
        }
        
    except Exception as e:
        return {
            'title': urlparse(feed_url).netloc,
            'url': feed_url,
            'entry_count': 0,
            'dates': [],
            'newest': None,
            'oldest': None,
            'error': str(e)
        }


def parse_date(date_str):
    """Parse various date formats to extract year"""
    if not date_str:
        return None
        
    # Extract year from various formats
    year_match = re.search(r'(20\d{2}|19\d{2})', date_str)
    if year_match:
        return int(year_match.group(1))
    return None


def analyze_single_feed(feed_url, verbose=False):
    """Analyze a single feed"""
    print(f"Analyzing feed: {feed_url}")
    print("-" * 60)
    
    result = parse_feed(feed_url, verbose)
    
    if result['error']:
        print(f"❌ Error: {result['error']}")
        return
    
    print(f"📰 Feed: {result['title']}")
    print(f"🔗 URL: {result['url']}")
    print(f"📊 Entries: {result['entry_count']}")
    
    if result['dates']:
        newest_year = parse_date(result['newest'])
        oldest_year = parse_date(result['oldest'])
        
        print(f"📅 Date Range:")
        print(f"   Newest: {result['newest']}")
        print(f"   Oldest: {result['oldest']}")
        
        if newest_year and oldest_year:
            span_years = newest_year - oldest_year
            print(f"   Span: {span_years} years ({oldest_year}-{newest_year})")
            
            if result['entry_count'] > 0:
                entries_per_year = result['entry_count'] / max(span_years, 1)
                print(f"   Average: {entries_per_year:.1f} entries/year")
    
    # Coverage assessment
    if result['entry_count'] < 10:
        print("⚠️  Very few entries - may be incomplete feed")
    elif result['entry_count'] > 100:
        print("✅ Good historical coverage")
    
    newest_year = parse_date(result['newest'])
    if newest_year and newest_year < 2025:
        print(f"⚠️  Feed may be stale (newest entry from {newest_year})")


def load_feeds_from_config(config_path):
    """Load feeds from spacefeeder.toml configuration"""
    feeds = {}
    
    try:
        with open(config_path, 'r') as f:
            config = toml.load(f)
        
        # Get built-in feeds from data/feeds.toml
        data_dir = Path(config_path).parent / "data"
        feeds_toml_path = data_dir / "feeds.toml"
        
        if feeds_toml_path.exists():
            with open(feeds_toml_path, 'r') as f:
                built_in = toml.load(f)
                if 'feeds' in built_in:
                    for slug, info in built_in['feeds'].items():
                        feeds[slug] = {
                            'url': info['url'],
                            'author': info.get('author', 'Unknown'),
                            'source': 'built-in'
                        }
        
        # Get user feeds from config
        if 'feeds' in config:
            for slug, info in config['feeds'].items():
                if 'url' in info:  # User-defined feed
                    feeds[slug] = {
                        'url': info['url'],
                        'author': info.get('author', 'Unknown'),
                        'source': 'user'
                    }
                elif slug in feeds:  # Override built-in feed
                    feeds[slug]['source'] = 'user-configured'
        
        return feeds
        
    except Exception as e:
        print(f"Error loading config: {e}")
        return {}


def write_output(message, output_file=None):
    """Write to file or print to stdout"""
    if output_file:
        output_file.write(message + "\n")
        output_file.flush()
    else:
        print(message)


def analyze_all_feeds(config_path, output_file=None, verbose=False):
    """Analyze all feeds from configuration"""
    feeds = load_feeds_from_config(config_path)
    
    if not feeds:
        write_output("No feeds found in configuration", output_file)
        return
    
    write_output(f"Found {len(feeds)} feeds to analyze", output_file)
    write_output("=" * 80, output_file)
    
    results = []
    
    for i, (slug, info) in enumerate(feeds.items(), 1):
        progress = f"({i}/{len(feeds)})"
        write_output(f"\n🔍 Analyzing {slug} {progress} - {info['author']}", output_file)
        
        result = parse_feed(info['url'], verbose)
        result['slug'] = slug
        result['author'] = info['author']
        result['source'] = info['source']
        results.append(result)
        
        if result['error']:
            write_output(f"   ❌ {result['error']}", output_file)
        else:
            newest_year = parse_date(result['newest'])
            oldest_year = parse_date(result['oldest'])
            span = f"{oldest_year}-{newest_year}" if oldest_year and newest_year else "unknown"
            write_output(f"   📊 {result['entry_count']} entries, {span}", output_file)
    
    # Summary analysis
    write_output("\n" + "=" * 80, output_file)
    write_output("📊 SUMMARY ANALYSIS", output_file)
    write_output("=" * 80, output_file)
    
    total_entries = sum(r['entry_count'] for r in results if not r['error'])
    working_feeds = [r for r in results if not r['error']]
    error_feeds = [r for r in results if r['error']]
    
    write_output(f"📈 Total entries across all feeds: {total_entries}", output_file)
    write_output(f"✅ Working feeds: {len(working_feeds)}", output_file)
    write_output(f"❌ Error feeds: {len(error_feeds)}", output_file)
    
    if working_feeds:
        entry_counts = [r['entry_count'] for r in working_feeds]
        write_output(f"📊 Entry count range: {min(entry_counts)} - {max(entry_counts)}", output_file)
        write_output(f"📊 Average entries per feed: {sum(entry_counts) / len(entry_counts):.1f}", output_file)
        
        # Find feeds with very limited content
        limited_feeds = [r for r in working_feeds if r['entry_count'] < 30]
        if limited_feeds:
            write_output(f"\n⚠️  Feeds with limited content (<30 entries):", output_file)
            for feed in limited_feeds:
                write_output(f"   • {feed['slug']}: {feed['entry_count']} entries", output_file)
        
        # Find feeds with good historical coverage  
        historical_feeds = [r for r in working_feeds if r['entry_count'] > 100]
        if historical_feeds:
            write_output(f"\n✅ Feeds with good historical coverage (>100 entries):", output_file)
            for feed in historical_feeds:
                oldest_year = parse_date(feed['oldest'])
                newest_year = parse_date(feed['newest'])
                span = f"({oldest_year}-{newest_year})" if oldest_year and newest_year else ""
                write_output(f"   • {feed['slug']}: {feed['entry_count']} entries {span}", output_file)
    
    if error_feeds:
        write_output(f"\n❌ Feeds with errors:", output_file)
        for feed in error_feeds:
            write_output(f"   • {feed['slug']}: {feed['error']}", output_file)
    
    # Additional analysis for documentation
    if output_file:
        write_output(f"\n## Key Findings for Documentation", output_file)
        write_output(f"", output_file)
        write_output(f"**Feed Coverage Issues:**", output_file)
        
        very_limited = [r for r in working_feeds if r['entry_count'] <= 20]
        if very_limited:
            write_output(f"- {len(very_limited)} feeds provide ≤20 entries (very recent content only)", output_file)
        
        recent_only = [r for r in working_feeds if r['entry_count'] < 50]
        write_output(f"- {len(recent_only)} of {len(working_feeds)} feeds provide <50 entries", output_file)
        
        total_possible_tags = sum(r['entry_count'] for r in working_feeds)
        write_output(f"- Current categorization based on only {total_entries} articles", output_file)
        write_output(f"- Many singleton tags likely due to insufficient historical data", output_file)


def main():
    parser = argparse.ArgumentParser(description="Analyze RSS/Atom feed coverage")
    parser.add_argument('feed_url', nargs='?', help='Feed URL to analyze')
    parser.add_argument('--all-feeds', action='store_true', help='Analyze all feeds from config')
    parser.add_argument('--config', default='spacefeeder.toml', help='Config file path')
    parser.add_argument('--report', help='Write report to file instead of stdout')
    parser.add_argument('--verbose', action='store_true', help='Show verbose progress')
    
    args = parser.parse_args()
    
    if args.all_feeds:
        if args.report:
            # Create reports directory if it doesn't exist
            Path("reports").mkdir(exist_ok=True)
            report_path = Path("reports") / args.report
            print(f"Writing feed analysis report to {report_path}")
            with open(report_path, 'w') as f:
                f.write(f"# Feed Analysis Report\n")
                f.write(f"Generated: {datetime.now().isoformat()}\n\n")
                analyze_all_feeds(args.config, f, args.verbose)
            print(f"Report written to {report_path}")
        else:
            analyze_all_feeds(args.config, None, args.verbose)
    elif args.feed_url:
        analyze_single_feed(args.feed_url, args.verbose)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()