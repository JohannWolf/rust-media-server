use axum::{
    routing::get,
    Router,
};
use sqlx::SqlitePool;

use crate::api::handlers;

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
        .route("/playlist.m3u", axum::routing::get(handlers::get_playlist))
        .with_state(pool)
}