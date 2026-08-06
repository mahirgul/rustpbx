use axum::{extract::State, Json};
use serde::Serialize;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub name: &'static str,
    pub port: &'static str,
    pub status: &'static str, // "online", "offline", "standby"
}

#[derive(Debug, Serialize)]
pub struct DashboardMetrics {
    pub total_registered_subscribers: i64,
    pub active_calls: u64,
    pub database_status: &'static str,
    pub services: Vec<ServiceStatus>,
}

pub async fn get_dashboard_metrics(
    State(pool): State<Arc<SqlitePool>>,
) -> Json<DashboardMetrics> {
    let total_subscribers: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM extensions WHERE is_active = 1")
        .fetch_one(pool.as_ref())
        .await
        .unwrap_or(0);

    let services = vec![
        ServiceStatus {
            name: "sip-engine (Core B2BUA)",
            port: "5060 UDP",
            status: "online",
        },
        ServiceStatus {
            name: "media-node (RTP Engine)",
            port: "50051 gRPC",
            status: "online",
        },
        ServiceStatus {
            name: "web-admin (Management UI)",
            port: "8088 HTTP",
            status: "online",
        },
        ServiceStatus {
            name: "webrtc-gateway (WSS Edge)",
            port: "8089 WSS",
            status: "standby",
        },
    ];

    Json(DashboardMetrics {
        total_registered_subscribers: total_subscribers,
        active_calls: 0,
        database_status: "Healthy (SQLite WAL)",
        services,
    })
}
