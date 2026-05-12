use crate::core::domain::asset::{Asset, AssetType, AssetVisibility, AssetError, ImageVersions};
use crate::data::access::mongodb::AssetRepository;
use crate::storage::minio::Storage;
use crate::utils::images::resize_image;
use chrono::Utc;
use uuid::Uuid;
use bytes::Bytes;

#[derive(Clone)]
pub struct AssetService {
    pub repository: AssetRepository,
    pub storage: Storage,
    pub base_url: String,
}

impl AssetService {
    pub fn new(repository: AssetRepository, storage: Storage, base_url: String) -> Self {
        Self {
            repository,
            storage,
            base_url,
        }
    }

    pub async fn upload_asset(
        &self,
        tenant_id: String,
        filename: String,
        content_type: String,
        data: Bytes,
        visibility: AssetVisibility,
        uploaded_by: String,
    ) -> Result<Asset, AssetError> {
        let asset_id = Uuid::new_v4().to_string();
        let asset_type = if content_type.starts_with("image/") {
            AssetType::Image
        } else if content_type == "application/pdf" || content_type.contains("word") {
            AssetType::Document
        } else if content_type.starts_with("video/") {
            AssetType::Video
        } else {
            AssetType::Document // Default
        };

        let bucket = std::env::var("MINIO_BUCKET").unwrap_or_else(|_| "images".to_string());
        let mut versions = None;
        let main_path = format!("{}/{}/{}", tenant_id, asset_id, filename);

        if asset_type == AssetType::Image {
            let mut img_versions = ImageVersions {
                original: format!("{}",  main_path),
                v1200: None,
                v800: None,
                v400: None,
            };

            // Original
            self.storage.upload_to_bucket(&bucket, main_path.clone(), data.clone(), &content_type)
                .await
                .map_err(|e| AssetError::StorageError(e.to_string()))?;

            let avif_content_type = "image/avif";

            // v1200
            let v1200_key = format!("{}/{}/1200_{}.avif", tenant_id, asset_id, filename);
            match resize_image(&data, 1200, 1200) {
                Ok(v1200_data) => {
                    if let Err(e) = self.storage.upload_to_bucket(&bucket, v1200_key.clone(), v1200_data, avif_content_type).await {
                        eprintln!("Error uploading v1200: {:?}", e);
                    } else {
                        img_versions.v1200 = Some(format!("{}", v1200_key));
                    }
                }
                Err(e) => {
                    eprintln!("Error resizing v1200: {:?}", e);
                }
            }

            // v800
            let v800_key = format!("{}/{}/800_{}.avif", tenant_id, asset_id, filename);
            match resize_image(&data, 800, 800) {
                Ok(v800_data) => {
                    if let Err(e) = self.storage.upload_to_bucket(&bucket, v800_key.clone(), v800_data, avif_content_type).await {
                        eprintln!("Error uploading v800: {:?}", e);
                    } else {
                        img_versions.v800 = Some(format!("{}", v800_key));
                    }
                }
                Err(e) => {
                    eprintln!("Error resizing v800: {:?}", e);
                }
            }

            // v400
            let v400_key = format!("{}/{}/400_{}.avif", tenant_id, asset_id, filename);
            match resize_image(&data, 400, 400) {
                Ok(v400_data) => {
                    if let Err(e) = self.storage.upload_to_bucket(&bucket, v400_key.clone(), v400_data, avif_content_type).await {
                        eprintln!("Error uploading v400: {:?}", e);
                    } else {
                        img_versions.v400 = Some(format!("{}", v400_key));
                    }
                }
                Err(e) => {
                    eprintln!("Error resizing v400: {:?}", e);
                }
            }

            versions = Some(img_versions);
        } else {
            // No es imagen, subida normal
            self.storage.upload_to_bucket(&bucket, main_path.clone(), data.clone(), &content_type)
                .await
                .map_err(|e| AssetError::StorageError(e.to_string()))?;
        }

        let asset = Asset {
            id: asset_id,
            tenant_id,
            r#type: asset_type,
            filename,
            mime_type: content_type,
            size: data.len() as u64,
            path: main_path.clone(),
            url: format!("{}/{}", self.base_url, main_path),
            versions,
            visibility,
            uploaded_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Save metadata to MongoDB
        self.repository.create(&asset).await.map_err(|e| {
            eprintln!("Error saving to MongoDB: {:?}", e);
            AssetError::DatabaseError(e.to_string())
        })?;

        Ok(asset)
    }

    pub async fn get_asset(&self, id: &str) -> Result<Asset, AssetError> {
        self.repository.get_by_id(id).await?
            .ok_or(AssetError::AssetNotFound)
    }

    pub async fn get_assets_batch(&self, ids: Vec<String>) -> Result<Vec<Asset>, AssetError> {
        Ok(self.repository.get_by_ids(&ids).await.map_err(|e| {
            eprintln!("Error fetching batch from MongoDB: {:?}", e);
            AssetError::DatabaseError(e.to_string())
        })?)
    }

    pub async fn delete_asset(&self, id: &str) -> Result<(), AssetError> {
        let asset = self.get_asset(id).await?;
        let bucket = std::env::var("MINIO_BUCKET").unwrap_or_else(|_| "images".to_string());
        
        self.storage.delete_from_bucket(&bucket, &asset.path)
            .await
            .map_err(|e| AssetError::StorageError(e.to_string()))?;
            
        self.repository.delete(id).await?;
        Ok(())
    }

    pub async fn list_assets(&self, tenant_id: &str, asset_type: Option<String>) -> Result<Vec<Asset>, AssetError> {
        Ok(self.repository.list_by_tenant(tenant_id, asset_type.as_deref()).await?)
    }
}
