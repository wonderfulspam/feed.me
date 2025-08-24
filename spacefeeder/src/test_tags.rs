// Simple test to verify tags are loaded
use spacefeeder::config::Config;

fn main() {
    let config = Config::from_file("../test_config.toml").unwrap();
    
    println!("Total tags loaded: {}", config.categorization.tags.len());
    println!("\nTags:");
    for tag in &config.categorization.tags {
        println!("  - {}: {}", tag.name, tag.description);
    }
    
    // Verify we have the expected default tags
    let tag_names: Vec<&str> = config.categorization.tags.iter()
        .map(|t| t.name.as_str())
        .collect();
    
    let expected_tags = vec!["rust", "ai", "llm", "anthropic", "openai", 
                             "google-ai", "devops", "security", "linux", 
                             "python", "cloud", "javascript", "web-dev", 
                             "go", "database"];
    
    for expected in expected_tags {
        if tag_names.contains(&expected) {
            println!("✓ Found expected tag: {}", expected);
        } else {
            println!("✗ Missing expected tag: {}", expected);
        }
    }
}