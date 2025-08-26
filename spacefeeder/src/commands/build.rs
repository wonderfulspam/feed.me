use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use clap::Args;
use serde_json::Value;
use tera::{Context as TeraContext, Tera};
use walkdir::WalkDir;

use crate::commands::build_categories;
use crate::commands::fetch_feeds::{self, FetchArgs};
use crate::config;
use crate::search::ArticleDoc;

#[derive(Args)]
pub struct BuildArgs {
    /// Path to the config file
    #[arg(long, default_value = "./spacefeeder.toml")]
    pub config_path: String,
}

pub fn execute(args: BuildArgs) -> Result<()> {
    println!("Building site...");

    // Step 1: Fetch feeds (reuse existing functionality)
    let fetch_args = FetchArgs {
        config_path: args.config_path.clone(),
    };
    fetch_feeds::execute(fetch_args)?;

    // Step 2: Initialize template engine
    let mut tera = setup_templates()?;

    // Step 3: Generate HTML pages
    generate_pages(&mut tera)?;

    // Step 4: Copy static assets
    copy_static_assets()?;

    println!("✅ Site build complete!");
    Ok(())
}

fn setup_templates() -> Result<Tera> {
    let mut tera = Tera::new("templates/**/*")?;

    // Add custom filters to match Zola's behavior
    tera.register_filter("slugify", slugify_filter);
    tera.register_filter("date", date_filter);

    // Add custom functions to match Zola's behavior
    tera.register_function("load_data", load_data_function);
    tera.register_function("now", now_function);

    Ok(tera)
}

fn generate_pages(tera: &mut Tera) -> Result<()> {
    // Clean and recreate public directory
    if Path::new("public").exists() {
        fs::remove_dir_all("public")?;
    }
    fs::create_dir_all("public")?;

    // Load JSON data files
    let loved_data = load_json_data("content/data/lovedData.json")?;
    let liked_data = load_json_data("content/data/likedData.json")?;
    let item_data = load_json_data("content/data/itemData.json")?;

    // Generate index page
    generate_page(
        tera,
        "index.html",
        "public/index.html",
        &[("loved_data", &loved_data), ("liked_data", &liked_data)],
    )?;

    // Generate loved page
    fs::create_dir_all("public/loved")?;
    generate_page(
        tera,
        "loved.html",
        "public/loved/index.html",
        &[("item_data", &loved_data)],
    )?;

    // Generate all page
    fs::create_dir_all("public/all")?;
    generate_page(
        tera,
        "all.html",
        "public/all/index.html",
        &[("item_data", &item_data)],
    )?;

    // Generate search page
    fs::create_dir_all("public/search")?;
    generate_page(tera, "search.html", "public/search/index.html", &[])?;

    // Copy search data for JavaScript
    fs::create_dir_all("public/data")?;
    fs::copy(
        "content/data/searchData.json",
        "public/data/searchData.json",
    )?;

    // Generate categories page
    generate_categories_page(tera)?;

    // Generate basic 404 page
    fs::write(
        "public/404.html",
        "<!doctype html>\n<title>404 Not Found</title>\n<h1>404 Not Found</h1>\n",
    )?;

    // Generate robots.txt and sitemap.xml
    generate_robots_txt()?;
    generate_sitemap()?;

    Ok(())
}

fn generate_page(
    tera: &Tera,
    template_name: &str,
    output_path: &str,
    data: &[(&str, &Value)],
) -> Result<()> {
    let mut context = TeraContext::new();

    // Add data to context
    for (key, value) in data {
        context.insert(*key, value);
    }

    // Render template
    let rendered = tera
        .render(template_name, &context)
        .with_context(|| format!("Failed to render template: {}", template_name))?;

    // Write to file
    fs::write(output_path, rendered)
        .with_context(|| format!("Failed to write output file: {}", output_path))?;

    println!("  Generated: {}", output_path);
    Ok(())
}

fn load_json_data(path: &str) -> Result<Value> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read data file: {}", path))?;
    let value: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON in: {}", path))?;
    Ok(value)
}

fn copy_static_assets() -> Result<()> {
    let static_dir = Path::new("static");
    let public_dir = Path::new("public");

    if !static_dir.exists() {
        return Ok(());
    }

    // Walk through static directory and copy all files
    for entry in WalkDir::new(static_dir) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            // Calculate relative path from static/
            let relative_path = path.strip_prefix(static_dir)?;
            let dest_path = public_dir.join(relative_path);

            // Create parent directories if they don't exist
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Copy the file
            fs::copy(path, &dest_path)?;
            println!(
                "  Copied: static/{} → public/{}",
                relative_path.display(),
                relative_path.display()
            );
        }
    }

    Ok(())
}

// Custom filter to match Zola's slugify behavior
fn slugify_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("slugify filter can only be used on strings"))?;
    let slug = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        // Remove multiple consecutive dashes
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .trim_matches('-')
        .to_string();
    Ok(Value::String(slug))
}

// Custom function to match Zola's load_data behavior
fn load_data_function(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let path = args
        .get("path")
        .ok_or_else(|| tera::Error::msg("load_data function requires a 'path' argument"))?
        .as_str()
        .ok_or_else(|| tera::Error::msg("load_data path must be a string"))?;

    let content = fs::read_to_string(path)
        .map_err(|e| tera::Error::msg(format!("Failed to read file '{}': {}", path, e)))?;

    let value: Value = serde_json::from_str(&content)
        .map_err(|e| tera::Error::msg(format!("Failed to parse JSON in '{}': {}", path, e)))?;

    Ok(value)
}

// Custom function to match Zola's now() behavior
fn now_function(_args: &HashMap<String, Value>) -> tera::Result<Value> {
    let now = Utc::now();
    // Return timestamp for use with date filter
    Ok(Value::Number(serde_json::Number::from(now.timestamp())))
}

// Custom filter to match Zola's date behavior
fn date_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    // Parse timestamp (either from now() function or article pub_date)
    let timestamp = if let Some(timestamp) = value.as_i64() {
        timestamp
    } else if let Some(date_str) = value.as_str() {
        // Try parsing ISO date string (for article dates)
        let dt = DateTime::parse_from_rfc3339(date_str)
            .map_err(|_| tera::Error::msg("Invalid date format"))?;
        dt.timestamp()
    } else {
        return Err(tera::Error::msg(
            "date filter requires a timestamp or date string",
        ));
    };

    let datetime = Utc
        .timestamp_opt(timestamp, 0)
        .single()
        .ok_or_else(|| tera::Error::msg("Invalid timestamp"))?;

    // Get timezone (default to UTC)
    let timezone_str = args
        .get("timezone")
        .and_then(|v| v.as_str())
        .unwrap_or("UTC");

    let tz: Tz = timezone_str
        .parse()
        .map_err(|_| tera::Error::msg(format!("Invalid timezone: {}", timezone_str)))?;

    let local_datetime = datetime.with_timezone(&tz);

    // Get format (default to Zola's default)
    let format = args
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("%Y-%m-%d");

    let formatted = local_datetime.format(format).to_string();
    Ok(Value::String(formatted))
}

fn generate_robots_txt() -> Result<()> {
    let base_url = config::get_config().base_url().trim_end_matches('/');
    let robots_content = format!(
        r#"User-agent: *
Disallow:
Allow: /
Sitemap: {}/sitemap.xml
"#,
        base_url
    );

    fs::write("public/robots.txt", robots_content)?;
    println!("  Generated: public/robots.txt");
    Ok(())
}

fn generate_categories_page(tera: &Tera) -> Result<()> {
    // Load all articles from itemData.json
    let item_data = load_json_data("content/data/itemData.json")?;
    let articles: Vec<ArticleDoc> = serde_json::from_value(item_data)?;

    // Generate categories page using the build_categories module
    build_categories::build_categories_page(&articles, tera, "public")?;

    Ok(())
}

fn generate_sitemap() -> Result<()> {
    let base_url = config::get_config().base_url().trim_end_matches('/');
    let sitemap_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <url>
        <loc>{}/</loc>
    </url>
    <url>
        <loc>{}/all/</loc>
    </url>
    <url>
        <loc>{}/loved/</loc>
    </url>
    <url>
        <loc>{}/categories/</loc>
    </url>
    <url>
        <loc>{}/search/</loc>
    </url>
</urlset>
"#,
        base_url, base_url, base_url, base_url, base_url
    );

    fs::write("public/sitemap.xml", sitemap_content)?;
    println!("  Generated: public/sitemap.xml");
    Ok(())
}
