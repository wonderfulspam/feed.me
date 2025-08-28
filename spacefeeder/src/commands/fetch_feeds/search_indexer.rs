use super::types::ItemOutput;
use crate::search::{ArticleDoc, SearchIndex};
use anyhow::Result;
use chrono::Utc;

/// Build search index from processed items
pub fn build_search_index(items: &[ItemOutput]) -> Result<()> {
    let search_index = match SearchIndex::new("./search_index") {
        Ok(index) => index,
        Err(e) => {
            eprintln!("Warning: Failed to initialize search index: {}", e);
            return Ok(()); // Don't fail the whole operation for search indexing issues
        }
    };

    // Clear existing index
    search_index.clear_index()?;

    // Convert items to ArticleDoc format
    let articles: Vec<ArticleDoc> = items
        .iter()
        .map(|item| ArticleDoc {
            title: item.item.title.clone(),
            description: item.item.description.clone(),
            safe_description: item.item.safe_description.clone(),
            author: item.meta.author.clone(),
            tier: format!("{:?}", item.meta.tier).to_lowercase(),
            slug: item.slug.clone(),
            item_url: item.item.item_url.clone(),
            pub_date: item.item.pub_date.unwrap_or_else(Utc::now),
            tags: item.item.tags.clone(),
        })
        .collect();

    // Add articles to search index
    search_index.add_articles(&articles)?;

    // Export search data as JSON for web interface (both locations)
    let search_data_path = "./content/data/searchData.json";
    let static_search_data_path = "./static/data/searchData.json";

    let search_data = serde_json::to_string_pretty(&articles)?;
    std::fs::write(search_data_path, &search_data)?;
    std::fs::write(static_search_data_path, &search_data)?;

    Ok(())
}
