pub mod extensions;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use std::sync::Arc;

pub fn create_api_router(pool: Arc<SqlitePool>) -> Router {
    Router::new()
        .route("/api/v1/extensions", get(extensions::list_extensions))
        .route("/api/v1/extensions", post(extensions::create_extension))
        .with_state(pool)
}
