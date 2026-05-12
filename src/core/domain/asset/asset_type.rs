use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Image,
    Document,
    Video,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetType::Image => write!(f, "image"),
            AssetType::Document => write!(f, "document"),
            AssetType::Video => write!(f, "video"),
        }
    }
}
