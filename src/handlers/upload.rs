use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use mime::Mime;
use crate::utils::images::resize_image;

use crate::core::operation::asset_ops::AssetService;

pub async fn upload_products(
    State(service): State<AssetService>,
    multipart: Multipart,
) -> impl IntoResponse {
    let bucket = std::env::var("MINIO_BUCKET_PRODUCTS").unwrap_or_else(|_| "images".to_string());
    process_upload(service, multipart, &bucket, "products").await
}

pub async fn upload_images(
    State(service): State<AssetService>,
    multipart: Multipart,
) -> impl IntoResponse {
    let bucket = std::env::var("MINIO_BUCKET_IMAGES").unwrap_or_else(|_| "images".to_string());
    process_upload(service, multipart, &bucket, "images").await
}

pub async fn upload_contents(
    State(service): State<AssetService>,
    multipart: Multipart,
) -> impl IntoResponse {
    let bucket = std::env::var("MINIO_BUCKET_CONTENTS").unwrap_or_else(|_| "contents".to_string());
    process_upload(service, multipart, &bucket, "contents").await
}

async fn process_upload(
    service: AssetService,
    mut multipart: Multipart,
    bucket: &str,
    folder: &str,
) -> impl IntoResponse {
    let mut field = match multipart.next_field().await {
        Ok(Some(field)) => field,
        Ok(None) => {
            return (axum::http::StatusCode::BAD_REQUEST, "No fields found (ensure you send a 'file' field)").into_response();
        },
        Err(e) => {
            if e.to_string().contains("boundary") {
                return (axum::http::StatusCode::BAD_REQUEST, 
                    "Invalid boundary. In Postman, do NOT set 'Content-Type' manually in Headers. Let Postman handle it automatically when you select 'form-data' in the Body tab.").into_response();
            }
            return (axum::http::StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)).into_response();
        }
    };

    loop {
        let field_name = field.name().unwrap_or("unknown").to_string();

        if field_name == "file" {
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            let mime: Mime = match content_type.parse() {
                Ok(m) => m,
                Err(_) => return (axum::http::StatusCode::BAD_REQUEST, "Invalid Content-Type").into_response(),
            };

            if mime.type_() != mime::IMAGE {
                return (axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE, format!("Not an image (received: {})", mime)).into_response();
            }

            let data = match field.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Error al leer los bytes del campo 'file': {}", e);
                    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read bytes: {}", e)).into_response();
                }
            };
            
            let ext = mime.subtype().as_str();
            let file_uuid = Uuid::new_v4();

            // Definimos las carpetas y tamaños
            let versions = [
                ("original", None, None),
                ("medium", Some(800), Some(800)),
                ("thumbnail", Some(200), Some(200)),
            ];

            let mut uploaded_keys = Vec::new();

            for (version_name, width, height) in versions {
                let key = format!("{}/{}/{}.{}", folder, version_name, file_uuid, ext);
                
                let upload_data = if let (Some(w), Some(h)) = (width, height) {
                    match resize_image(&data, w, h) {
                        Ok(resized) => resized,
                        Err(e) => {
                            eprintln!("Error al redimensionar imagen para {}: {}", version_name, e);
                            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to process image").into_response();
                        }
                    }
                } else {
                    data.clone()
                };

                if let Err(e) = service.storage.upload_to_bucket(bucket, key.clone(), upload_data, &content_type).await {
                    eprintln!("Error al subir al storage (MinIO) en bucket {} (versión {}): {}", bucket, version_name, e);
                    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to upload to storage").into_response();
                }
                
                uploaded_keys.push(key);
            }

            println!("Imagen y sus versiones subidas con éxito a {}: {:?}", bucket, uploaded_keys);
            return Json(serde_json::json!({ "keys": uploaded_keys, "bucket": bucket })).into_response();
        }

        field = match multipart.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => return (axum::http::StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)).into_response(),
        };
    }

    (axum::http::StatusCode::BAD_REQUEST, "file field missing (ensure the field name in Postman is exactly 'file')").into_response()
}
