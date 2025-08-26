use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, TantivyDocument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleDoc {
    pub title: String,
    pub description: String,
    pub safe_description: String,
    pub author: String,
    pub tier: String,
    pub slug: String,
    pub item_url: String,
    pub pub_date: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub description: String,
    pub safe_description: String,
    pub author: String,
    pub tier: String,
    pub slug: String,
    pub item_url: String,
    pub pub_date: DateTime<Utc>,
    pub tags: Vec<String>,
    pub score: f32,
}

pub struct SearchIndex {
    index: Index,
    title_field: Field,
    description_field: Field,
    author_field: Field,
    tier_field: Field,
    slug_field: Field,
    url_field: Field,
    date_field: Field,
    tags_field: Field,
}

impl SearchIndex {
    pub fn new<P: AsRef<Path>>(index_path: P) -> Result<Self> {
        let mut schema_builder = Schema::builder();

        // Searchable text fields (title gets higher boost)
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let description_field = schema_builder.add_text_field("description", TEXT | STORED);
        let tags_field = schema_builder.add_text_field("tags", TEXT | STORED);

        // Filterable/facet fields
        let author_field = schema_builder.add_text_field("author", STORED | STRING);
        let tier_field = schema_builder.add_text_field("tier", STORED | STRING);
        let slug_field = schema_builder.add_text_field("slug", STORED | STRING);
        let url_field = schema_builder.add_text_field("url", STORED | STRING);

        // Date field for sorting/filtering (stored as timestamp)
        let date_field = schema_builder.add_i64_field("date", STORED | INDEXED);

        let schema = schema_builder.build();

        // Create index directory if it doesn't exist
        std::fs::create_dir_all(&index_path)?;

        let index = Index::create_in_dir(index_path, schema)?;

        Ok(SearchIndex {
            index,
            title_field,
            description_field,
            author_field,
            tier_field,
            slug_field,
            url_field,
            date_field,
            tags_field,
        })
    }

    pub fn open<P: AsRef<Path>>(index_path: P) -> Result<Self> {
        let index = Index::open_in_dir(index_path)?;
        let schema = index.schema();

        let title_field = schema.get_field("title").unwrap();
        let description_field = schema.get_field("description").unwrap();
        let author_field = schema.get_field("author").unwrap();
        let tier_field = schema.get_field("tier").unwrap();
        let slug_field = schema.get_field("slug").unwrap();
        let url_field = schema.get_field("url").unwrap();
        let date_field = schema.get_field("date").unwrap();
        let tags_field = schema.get_field("tags").unwrap();

        Ok(SearchIndex {
            index,
            title_field,
            description_field,
            author_field,
            tier_field,
            slug_field,
            url_field,
            date_field,
            tags_field,
        })
    }

    pub fn add_articles(&self, articles: &[ArticleDoc]) -> Result<()> {
        let mut index_writer: IndexWriter<TantivyDocument> = self.index.writer(50_000_000)?;

        for article in articles {
            let doc = doc!(
                self.title_field => article.title.clone(),
                self.description_field => article.description.clone(),
                self.author_field => article.author.clone(),
                self.tier_field => article.tier.clone(),
                self.slug_field => article.slug.clone(),
                self.url_field => article.item_url.clone(),
                self.date_field => article.pub_date.timestamp(),
                self.tags_field => article.tags.join(" ")
            );
            index_writer.add_document(doc)?;
        }

        index_writer.commit()?;
        Ok(())
    }

    pub fn clear_index(&self) -> Result<()> {
        let mut index_writer: IndexWriter<TantivyDocument> = self.index.writer(50_000_000)?;
        index_writer.delete_all_documents()?;
        index_writer.commit()?;
        Ok(())
    }

    pub fn search(&self, query_text: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let reader = self.index.reader()?;

        let searcher = reader.searcher();

        // Create query parser for title, description, and tags fields
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.title_field, self.description_field, self.tags_field],
        );

        let query = query_parser.parse_query(query_text)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();

        for (score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

            let title = retrieved_doc
                .get_first(self.title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let description = retrieved_doc
                .get_first(self.description_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let author = retrieved_doc
                .get_first(self.author_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let tier = retrieved_doc
                .get_first(self.tier_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let slug = retrieved_doc
                .get_first(self.slug_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let item_url = retrieved_doc
                .get_first(self.url_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let pub_date = retrieved_doc
                .get_first(self.date_field)
                .and_then(|v| v.as_i64())
                .map(|timestamp| DateTime::from_timestamp(timestamp, 0).unwrap_or_default())
                .unwrap_or_default();

            let tags_str = retrieved_doc
                .get_first(self.tags_field)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let tags: Vec<String> = if tags_str.is_empty() {
                Vec::new()
            } else {
                tags_str.split_whitespace().map(|s| s.to_string()).collect()
            };

            results.push(SearchResult {
                title,
                description: description.clone(),
                safe_description: description, // For now, use description as safe_description
                author,
                tier,
                slug,
                item_url,
                pub_date,
                tags,
                score,
            });
        }

        Ok(results)
    }

    pub fn search_with_filters(
        &self,
        query_text: &str,
        author_filter: Option<&str>,
        tier_filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // For now, implement basic search and filter results in memory
        // A more sophisticated implementation would use tantivy's filtering capabilities
        let results = self.search(query_text, limit * 2)?; // Get more results to account for filtering

        let filtered_results: Vec<SearchResult> = results
            .into_iter()
            .filter(|result| {
                let author_matches = author_filter.is_none_or(|filter| {
                    result
                        .author
                        .to_lowercase()
                        .contains(&filter.to_lowercase())
                });
                let tier_matches = tier_filter.is_none_or(|filter| result.tier == filter);

                author_matches && tier_matches
            })
            .take(limit)
            .collect();

        Ok(filtered_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_articles() -> Vec<ArticleDoc> {
        vec![
            ArticleDoc {
                title: "Rust Programming Language".to_string(),
                description: "A systems programming language focused on safety and performance"
                    .to_string(),
                safe_description:
                    "A systems programming language focused on safety and performance".to_string(),
                author: "Rust Team".to_string(),
                tier: "love".to_string(),
                slug: "rust-blog".to_string(),
                item_url: "https://blog.rust-lang.org/article1".to_string(),
                pub_date: DateTime::parse_from_rfc3339("2025-08-24T10:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                tags: vec!["rust".to_string(), "programming".to_string()],
            },
            ArticleDoc {
                title: "Getting Started with Tantivy".to_string(),
                description: "A fast full-text search engine library written in Rust".to_string(),
                safe_description: "A fast full-text search engine library written in Rust"
                    .to_string(),
                author: "Tantivy Team".to_string(),
                tier: "like".to_string(),
                slug: "tantivy-docs".to_string(),
                item_url: "https://docs.rs/tantivy/article2".to_string(),
                pub_date: DateTime::parse_from_rfc3339("2025-08-23T15:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                tags: vec!["rust".to_string(), "search".to_string()],
            },
        ]
    }

    #[test]
    fn test_search_index_creation_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");

        let search_index = SearchIndex::new(&index_path).unwrap();
        let articles = create_test_articles();

        search_index.add_articles(&articles).unwrap();

        // Test search
        let results = search_index.search("rust", 10).unwrap();
        assert!(results.len() >= 1);
        assert!(results.iter().any(|r| r.title.contains("Rust")));

        // Test search with different query
        let results = search_index.search("tantivy", 10).unwrap();
        assert!(results.len() >= 1);
        assert!(results.iter().any(|r| r.title.contains("Tantivy")));
    }

    #[test]
    fn test_search_with_filters() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");

        let search_index = SearchIndex::new(&index_path).unwrap();
        let articles = create_test_articles();

        search_index.add_articles(&articles).unwrap();

        // Test tier filter
        let results = search_index
            .search_with_filters("rust", None, Some("love"), 10)
            .unwrap();
        assert!(results.iter().all(|r| r.tier == "love"));

        // Test author filter
        let results = search_index
            .search_with_filters("rust", Some("Rust"), None, 10)
            .unwrap();
        assert!(results.iter().all(|r| r.author.contains("Rust")));
    }
}
