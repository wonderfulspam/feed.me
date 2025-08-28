#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub confidence: f32,
    pub source: TagSource,
}

#[derive(Debug, Clone)]
pub enum TagSource {
    Manual,
    Feed,
    Rule,
    Keyword,
}

pub struct ItemContext<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub link: Option<&'a str>,
    pub author: Option<&'a str>,
    pub feed_slug: &'a str,
    pub feed_tags: Option<&'a [String]>,
    pub rss_categories: Option<&'a [String]>,
}
