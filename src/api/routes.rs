use axum::{
    extract::{Path, State},
    http::HeaderMap,
    routing::get,
    Router,
};

use sqlx::SqlitePool;

use super::handlers::health_check;
use crate::media::stream_media;

pub fn create_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/media/{id}/stream", get(stream_media_handler))
        .with_state(pool)
}

/// HTTP handler for media streaming.
async fn stream_media_handler(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    // Read the optional Range header.
    let range = headers
        .get(axum::http::header::RANGE)
        .and_then(|value| value.to_str().ok());

    stream_media(&pool, id, range).await
}