use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AssetError {
    #[error("Asset not found")]
    AssetNotFound,
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Invalid asset type")]
    InvalidType,
    
    #[error("Internal server error")]
    Internal,
}

impl From<mongodb::error::Error> for AssetError {
    fn from(err: mongodb::error::Error) -> Self {
        match err.kind.as_ref() {
            mongodb::error::ErrorKind::Command(e) if e.code == 11000 => {
                AssetError::DatabaseError("Duplicate key".to_string())
            }
            _ => AssetError::DatabaseError(err.to_string()),
        }
    }
}

impl From<anyhow::Error> for AssetError {
    fn from(err: anyhow::Error) -> Self {
        eprintln!("Internal error: {:?}", err);
        AssetError::Internal
    }
}