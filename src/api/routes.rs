use axum::{
    routing::get,
    Router,
};
use sqlx::SqlitePool;

use super::handlers::{
    get_media_handler,
    health_check,
    stream_media_handler,
};

pub fn create_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/media", get(get_media_handler))
        .route("/api/media/{id}/stream", get(stream_media_handler))
        .with_state(pool)
}