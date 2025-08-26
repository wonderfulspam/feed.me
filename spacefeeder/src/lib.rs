pub mod categorization;
pub mod commands;
pub mod config;
pub mod defaults;
pub mod search;

use std::str::FromStr;

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeedInfo {
    pub url: String,
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub tier: Tier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_tag: Option<bool>,
}

// User-defined feed info that can be minimal (only tier required)
// Other fields are merged from defaults if available
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserFeedInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub tier: Tier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_tag: Option<bool>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    New,
    Like,
    Love,
}

impl FromStr for Tier {
    type Err = std::fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "new" | "New" => Ok(Tier::New),
            "like" | "Like" => Ok(Tier::Like),
            "love" | "Love" => Ok(Tier::Love),
            _ => Err(std::fmt::Error),
        }
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tier::New => write!(f, "new"),
            Tier::Like => write!(f, "like"),
            Tier::Love => write!(f, "love"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_from_str_lowercase() {
        assert!(matches!(Tier::from_str("new").unwrap(), Tier::New));
        assert!(matches!(Tier::from_str("like").unwrap(), Tier::Like));
        assert!(matches!(Tier::from_str("love").unwrap(), Tier::Love));
    }

    #[test]
    fn test_tier_from_str_capitalized() {
        assert!(matches!(Tier::from_str("New").unwrap(), Tier::New));
        assert!(matches!(Tier::from_str("Like").unwrap(), Tier::Like));
        assert!(matches!(Tier::from_str("Love").unwrap(), Tier::Love));
    }

    #[test]
    fn test_tier_from_str_invalid() {
        assert!(Tier::from_str("invalid").is_err());
        assert!(Tier::from_str("").is_err());
        assert!(Tier::from_str("NEW").is_err());
        assert!(Tier::from_str("LOVE").is_err());
    }

    #[test]
    fn test_tier_serialization() {
        // Test that the serde rename_all = "lowercase" works
        let tier = Tier::New;
        let json = serde_json::to_string(&tier).unwrap();
        assert_eq!(json, "\"new\"");

        let tier = Tier::Love;
        let json = serde_json::to_string(&tier).unwrap();
        assert_eq!(json, "\"love\"");
    }

    #[test]
    fn test_tier_deserialization() {
        let tier: Tier = serde_json::from_str("\"new\"").unwrap();
        assert!(matches!(tier, Tier::New));

        let tier: Tier = serde_json::from_str("\"love\"").unwrap();
        assert!(matches!(tier, Tier::Love));
    }
}
