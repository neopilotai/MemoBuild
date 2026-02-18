use crate::server::metadata::MetadataStore;
use crate::server::storage::{ArtifactStorage, LocalStorage};
use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, head, put},
    Router,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

pub mod metadata;
pub mod storage;

pub struct AppState {
    pub metadata: MetadataStore,
    pub storage: Arc<dyn ArtifactStorage>,
}

pub async fn start_server(port: u16, data_dir: PathBuf) -> Result<()> {
    let db_path = data_dir.join("metadata.db");
    let metadata = MetadataStore::new(&db_path)?;
    let storage = Arc::new(LocalStorage::new(&data_dir)?);

    let state = Arc::new(AppState {
        metadata,
        storage,
    });

    let app = Router::new()
        .route("/cache/:hash", head(check_cache))
        .route("/cache/:hash", get(get_artifact))
        .route("/cache/:hash", put(put_artifact))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("🌐 MemoBuild Remote Cache Server running on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn check_cache(
    Path(hash): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.metadata.exists(&hash) {
        Ok(true) => StatusCode::OK,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(e) => {
            eprintln!("Error checking cache: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn get_artifact(
    Path(hash): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.storage.get(&hash) {
        Ok(Some(data)) => (StatusCode::OK, data).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            eprintln!("Error getting artifact: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn put_artifact(
    Path(hash): Path<String>,
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> impl IntoResponse {
    let size = body.len() as u64;
    
    // 1. Store the blob
    match state.storage.put(&hash, &body) {
        Ok(path) => {
            // 2. Update metadata
            if let Err(e) = state.metadata.insert(&hash, &path, size) {
                eprintln!("Error updating metadata: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::CREATED
        }
        Err(e) => {
            eprintln!("Error storing artifact: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
