mod asset_error;
mod asset_type;

pub use asset_error::AssetError;
pub use asset_type::AssetType;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AssetVisibility {
    Public,
    Private,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageVersions {
    pub original: String,
    pub v1200: Option<String>,
    pub v800: Option<String>,
    pub v400: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    #[serde(rename = "_id")]
    pub id: String,
    pub tenant_id: String,
    pub r#type: AssetType,
    pub filename: String,
    pub mime_type: String,
    pub size: u64,
    pub path: String,
    pub url: String,
    pub versions: Option<ImageVersions>,
    pub visibility: AssetVisibility,
    pub uploaded_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
