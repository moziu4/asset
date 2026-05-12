use std::collections::HashMap;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error, debug};
use crate::core::operation::asset_ops::AssetService;
use crate::core::domain::asset::{AssetVisibility, AssetError, AssetType, ImageVersions};

#[derive(Deserialize)]
pub struct ListParams {
    pub tenant_id: String,
    pub r#type: Option<String>,
}

#[derive(Serialize)]
pub struct AssetVersionResponse {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub r#type: AssetType,
    pub visibility: AssetVisibility,
    pub fallback_url: String,
    pub versions: ImageVersions,
}

impl IntoResponse for AssetError {
    fn into_response(self) -> axum::response::Response {
        error!("AssetError occurred: {:?}", self);
        let (status, error_message) = match self {
            AssetError::AssetNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AssetError::StorageError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AssetError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AssetError::InvalidType => (StatusCode::BAD_REQUEST, self.to_string()),
            AssetError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(serde_json::json!({
            "error": error_message,
            "code": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub async fn upload_asset(
    State(service): State<AssetService>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AssetError> {
    // Por simplicidad, asumimos tenant_id y user_id vienen en el multipart o headers
    // En un caso real vendrían del token JWT. Aquí los sacaremos de campos o pondremos defaults.
    let mut tenant_id = "default".to_string();
    let mut uploaded_by = "system".to_string();
    let mut visibility = AssetVisibility::Public;
    let mut filename = String::new();
    let mut content_type = String::new();
    let mut data = bytes::Bytes::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "tenant_id" => tenant_id = field.text().await.unwrap_or_else(|_| "default".to_string()),
            "user_id" => uploaded_by = field.text().await.unwrap_or_else(|_| "system".to_string()),
            "private" => {
                if field.text().await.unwrap_or_default() == "true" {
                    visibility = AssetVisibility::Private;
                }
            },
            "file" => {
                filename = field.file_name().unwrap_or("unknown").to_string();
                content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
                data = field.bytes().await.unwrap_or_default();
            },
            _ => {}
        }
    }

    if data.is_empty() {
        return Err(AssetError::InvalidType);
    }

    let asset = service.upload_asset(tenant_id, filename, content_type, data, visibility, uploaded_by).await?;
    Ok((StatusCode::CREATED, Json(asset)).into_response())
}

pub async fn get_asset(
    State(service): State<AssetService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AssetError> {
    let asset = service.get_asset(&id).await?;
    Ok((StatusCode::OK, Json(asset)).into_response())
}

pub async fn get_assets_batch(
    State(service): State<AssetService>,
    Json(ids): Json<Vec<String>>,
) -> Result<impl IntoResponse, AssetError> {
    let assets = service.get_assets_batch(ids).await?;
    let response: HashMap<String, crate::core::domain::asset::Asset> = assets
        .into_iter()
        .map(|asset| (asset.id.clone(), asset))
        .collect();
    
    Ok((StatusCode::OK, Json(response)).into_response())
}

pub async fn delete_asset(
    State(service): State<AssetService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AssetError> {
    service.delete_asset(&id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub async fn list_assets(
    State(service): State<AssetService>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, AssetError> {
    let assets = service.list_assets(&params.tenant_id, params.r#type).await?;
    Ok((StatusCode::OK, Json(assets)).into_response())
}

pub async fn get_asset_versions(
    State(service): State<AssetService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AssetError> {
    debug!("get_asset_versions called with id: {}", id);
    let asset = service.get_asset(&id).await.map_err(|e| {
        error!("Error getting asset {}: {:?}", id, e);
        e
    })?;

    info!("Asset found: {}", asset.id);

    let versions = asset.versions.clone().unwrap_or_else(|| ImageVersions {
        original: asset.url.clone(),
        v1200: None,
        v800: None,
        v400: None,
    });

    
    Ok((StatusCode::OK, Json(AssetVersionResponse {
        id: asset.id,
        filename: asset.filename,
        mime_type: asset.mime_type,
        r#type: asset.r#type,
        visibility: asset.visibility,
        fallback_url: asset.url,
        versions,
    })).into_response())
}
