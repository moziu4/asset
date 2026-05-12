use axum::{Router, routing::{get, post, delete}, extract::DefaultBodyLimit};
use std::net::SocketAddr;
use mongodb::Client;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::data::access::mongodb::AssetRepository;
use crate::core::operation::asset_ops::AssetService;

mod core;
mod data;
mod handlers;
mod storage;
mod utils;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "asset=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Configuración MinIO
    let main_bucket = std::env::var("MINIO_BUCKET").unwrap_or_else(|_| "images".to_string());
    let storage = storage::minio::Storage::new(
        std::env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT must be set"),
        std::env::var("MINIO_ACCESS_KEY").expect("MINIO_ACCESS_KEY must be set"),
        std::env::var("MINIO_SECRET_KEY").expect("MINIO_SECRET_KEY must be set"),
        main_bucket.clone(),
    ).await;

    // Asegurarse de que el bucket principal existe
    if let Err(e) = storage.ensure_bucket_exists(&main_bucket).await {
        error!("Error: Could not ensure bucket {} exists: {:?}", main_bucket, e);
    } else {
        info!("Bucket {} verified/created successfully.", main_bucket);
    }

    // Configuración MongoDB
    let mongo_uri = std::env::var("MONGO_URI").expect("MONGO_URI must be set");
    let mongo_client = Client::with_uri_str(&mongo_uri).await.expect("Failed to connect to MongoDB");
    
    // Intentar obtener el nombre de la DB de la URI o usar 'asset' como defecto
    let db_name = mongo_uri.split('/').last()
        .map(|s| s.split('?').next().unwrap_or(s))
        .filter(|s| !s.is_empty())
        .unwrap_or("asset");

    let db = mongo_client.database(db_name);
    info!("Connected to MongoDB database: {}", db_name);
    let repository = AssetRepository::new(&db);

    let base_url = std::env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
    let asset_service = AssetService::new(repository, storage.clone(), base_url);

    let app = Router::new()
        // Nuevos Endpoints
        .route("/assets", post(handlers::http::asset::upload_asset))
        .route("/assets/batch", post(handlers::http::asset::get_assets_batch))
        .route("/assets", get(handlers::http::asset::list_assets))
        .route("/assets/:id", get(handlers::http::asset::get_asset))
        .route("/assets/:id/versions", get(handlers::http::asset::get_asset_versions))
        .route("/assets/:id", delete(handlers::http::asset::delete_asset))
        
        // Mantener compatibilidad (o migrar gradualmente)
        .route("/products", post(handlers::upload::upload_products))
        .route("/images", post(handlers::upload::upload_images))
        .route("/contents", post(handlers::upload::upload_contents))
        
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // Límite de 10MB
        .with_state(asset_service);

    let bind_addr = std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let addr: SocketAddr = bind_addr.parse().expect("Invalid HTTP_BIND address");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}