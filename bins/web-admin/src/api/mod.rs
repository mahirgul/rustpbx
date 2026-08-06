pub mod dialplan;
pub mod extensions;
pub mod system;
pub mod trunks;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::SqlitePool;
use std::sync::Arc;

pub fn create_api_router(pool: Arc<SqlitePool>) -> Router {
    Router::new()
        .route(
            "/api/v1/system/dashboard",
            get(system::get_dashboard_metrics),
        )
        // Extension Routes
        .route("/api/v1/extensions", get(extensions::list_extensions))
        .route("/api/v1/extensions", post(extensions::create_extension))
        .route("/api/v1/extensions/:id", put(extensions::update_extension))
        .route(
            "/api/v1/extensions/:id",
            delete(extensions::delete_extension),
        )
        // Trunk Routes
        .route("/api/v1/trunks", get(trunks::list_trunks))
        .route("/api/v1/trunks", post(trunks::create_trunk))
        .route("/api/v1/trunks/:id", delete(trunks::delete_trunk))
        // Dialplan Routes
        .route("/api/v1/dialplan", get(dialplan::list_dialplan_rules))
        .route("/api/v1/dialplan", post(dialplan::create_dialplan_rule))
        .route(
            "/api/v1/dialplan/:id",
            delete(dialplan::delete_dialplan_rule),
        )
        .with_state(pool)
}
