use axum::{
    routing::get,
    Router,
};

use super::handlers::health_check;

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
}