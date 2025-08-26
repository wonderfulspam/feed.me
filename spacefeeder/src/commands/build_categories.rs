use std::collections::HashMap;

use anyhow::Result;
use serde::Serialize;
use tera::{Context, Tera};

use crate::search::ArticleDoc;

#[derive(Debug, Serialize, Clone)]
pub struct TagSummary {
    pub name: String,
    pub count: usize,
    pub sample_items: Vec<ArticleDoc>,
}

pub fn build_categories_page(
    articles: &[ArticleDoc],
    templates: &Tera,
    output_dir: &str,
) -> Result<()> {
    // Group articles by tags
    let mut tag_articles: HashMap<String, Vec<&ArticleDoc>> = HashMap::new();

    for article in articles {
        for tag in &article.tags {
            tag_articles.entry(tag.clone()).or_default().push(article);
        }
    }

    // Create tag summaries
    let mut all_tags: Vec<TagSummary> = tag_articles
        .iter()
        .map(|(tag_name, articles)| {
            let mut sorted_articles = articles.clone();
            // Sort by date (most recent first)
            sorted_articles.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));

            let count = sorted_articles.len();

            // Take up to 3 sample articles
            let sample_items: Vec<ArticleDoc> =
                sorted_articles.into_iter().take(3).cloned().collect();

            TagSummary {
                name: tag_name.clone(),
                count,
                sample_items,
            }
        })
        .collect();

    // Sort tags by count (most popular first)
    all_tags.sort_by(|a, b| b.count.cmp(&a.count));

    // Generate main categories page
    let mut context = Context::new();
    context.insert("all_tags", &all_tags);
    context.insert("selected_tag", &None::<String>);
    context.insert("filtered_items", &Vec::<ArticleDoc>::new());

    let rendered = templates.render("categories.html", &context)?;

    // Write categories page
    let categories_dir = format!("{}/categories", output_dir);
    std::fs::create_dir_all(&categories_dir)?;
    std::fs::write(format!("{}/index.html", categories_dir), rendered)?;

    println!("  Generated: {}/categories/index.html", output_dir);

    // Generate individual tag pages
    for (tag_name, tag_articles) in &tag_articles {
        let mut sorted_articles: Vec<ArticleDoc> = tag_articles
            .iter()
            .map(|&article| article.clone())
            .collect();

        // Sort by date (most recent first)
        sorted_articles.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));

        // Create context for individual tag page
        let mut tag_context = Context::new();
        tag_context.insert("all_tags", &all_tags);
        tag_context.insert("selected_tag", tag_name);
        tag_context.insert("filtered_items", &sorted_articles);

        let tag_rendered = templates.render("categories.html", &tag_context)?;

        // Create tag-specific directory and write page
        let tag_dir = format!("{}/categories/{}", output_dir, tag_name);
        std::fs::create_dir_all(&tag_dir)?;
        std::fs::write(format!("{}/index.html", tag_dir), tag_rendered)?;

        println!(
            "  Generated: {}/categories/{}/index.html",
            output_dir, tag_name
        );
    }

    Ok(())
}
