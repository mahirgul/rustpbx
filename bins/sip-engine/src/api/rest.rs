use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::b2bua::CallManager;

pub struct AppState {
    pub call_manager: Arc<CallManager>,
}

#[derive(Debug, Deserialize)]
pub struct OriginateRequest {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize)]
pub struct CallResponse {
    pub call_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub active_calls: usize,
    pub version: &'static str,
}

pub fn create_rest_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/calls", post(originate_call))
        .route("/api/v1/calls", get(list_calls))
        .route("/api/v1/calls/:call_id", delete(hangup_call))
        .route("/api/v1/system/info", get(system_info))
        .with_state(state)
}

async fn originate_call(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<OriginateRequest>,
) -> (StatusCode, Json<CallResponse>) {
    let call_id = format!("call-{}", uuid_simple());
    tracing::info!(
        "REST API: Originate call from {} to {}",
        payload.from,
        payload.to
    );

    (
        StatusCode::CREATED,
        Json(CallResponse {
            call_id,
            status: "initiating".to_string(),
        }),
    )
}

async fn list_calls(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let count = state.call_manager.active_call_count();
    Json(serde_json::json!({ "total": count }))
}

async fn hangup_call(
    State(state): State<Arc<AppState>>,
    Path(call_id): Path<String>,
) -> StatusCode {
    tracing::info!("REST API: Hangup call {}", call_id);
    if state.call_manager.remove_call(&call_id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn system_info(State(state): State<Arc<AppState>>) -> Json<SystemInfo> {
    Json(SystemInfo {
        active_calls: state.call_manager.active_call_count(),
        version: env!("CARGO_PKG_VERSION"),
    })
}

fn uuid_simple() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}
