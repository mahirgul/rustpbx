use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Trunk {
    pub id: i64,
    pub trunk_name: String,
    pub sip_server: String,
    pub port: i64,
    pub auth_username: Option<String>,
    pub auth_password: Option<String>,
    pub is_active: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateTrunkRequest {
    pub trunk_name: String,
    pub sip_server: String,
    pub port: Option<i64>,
    pub auth_username: Option<String>,
    pub auth_password: Option<String>,
}

pub async fn list_trunks(
    State(pool): State<Arc<SqlitePool>>,
) -> Result<Json<Vec<Trunk>>, StatusCode> {
    let trunks = sqlx::query_as::<_, Trunk>(
        "SELECT id, trunk_name, sip_server, port, auth_username, auth_password, is_active FROM trunks ORDER BY id DESC",
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(trunks))
}

pub async fn create_trunk(
    State(pool): State<Arc<SqlitePool>>,
    Json(payload): Json<CreateTrunkRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let port = payload.port.unwrap_or(5060);

    let result = sqlx::query(
        "INSERT INTO trunks (trunk_name, sip_server, port, auth_username, auth_password) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&payload.trunk_name)
    .bind(&payload.sip_server)
    .bind(port)
    .bind(&payload.auth_username)
    .bind(&payload.auth_password)
    .execute(pool.as_ref())
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": result.last_insert_rowid(),
            "trunk_name": payload.trunk_name
        })),
    ))
}

pub async fn delete_trunk(
    State(pool): State<Arc<SqlitePool>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM trunks WHERE id = ?")
        .bind(id)
        .execute(pool.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
