use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DialplanRule {
    pub id: i64,
    pub rule_name: String,
    pub pattern: String,
    pub destination_type: String,
    pub destination_target: String,
    pub priority: i64,
    pub is_active: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateDialplanRuleRequest {
    pub rule_name: String,
    pub pattern: String,
    pub destination_type: String,
    pub destination_target: String,
    pub priority: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDialplanRuleRequest {
    pub rule_name: Option<String>,
    pub pattern: Option<String>,
    pub destination_type: Option<String>,
    pub destination_target: Option<String>,
    pub priority: Option<i64>,
    pub is_active: Option<bool>,
}

pub async fn list_dialplan_rules(
    State(pool): State<Arc<SqlitePool>>,
) -> Result<Json<Vec<DialplanRule>>, StatusCode> {
    let rules = sqlx::query_as::<_, DialplanRule>(
        "SELECT id, rule_name, pattern, destination_type, destination_target, priority, is_active FROM dialplan_rules ORDER BY priority ASC, id ASC",
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rules))
}

pub async fn create_dialplan_rule(
    State(pool): State<Arc<SqlitePool>>,
    Json(payload): Json<CreateDialplanRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let priority = payload.priority.unwrap_or(1);

    let result = sqlx::query(
        "INSERT INTO dialplan_rules (rule_name, pattern, destination_type, destination_target, priority) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&payload.rule_name)
    .bind(&payload.pattern)
    .bind(&payload.destination_type)
    .bind(&payload.destination_target)
    .bind(priority)
    .execute(pool.as_ref())
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": result.last_insert_rowid(),
            "rule_name": payload.rule_name
        })),
    ))
}

pub async fn update_dialplan_rule(
    State(pool): State<Arc<SqlitePool>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(payload): Json<UpdateDialplanRuleRequest>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query(
        "UPDATE dialplan_rules SET rule_name = COALESCE(?, rule_name), pattern = COALESCE(?, pattern), destination_type = COALESCE(?, destination_type), destination_target = COALESCE(?, destination_target), priority = COALESCE(?, priority), is_active = COALESCE(?, is_active) WHERE id = ?"
    )
    .bind(&payload.rule_name)
    .bind(&payload.pattern)
    .bind(&payload.destination_type)
    .bind(&payload.destination_target)
    .bind(payload.priority)
    .bind(payload.is_active.map(|b| if b { 1 } else { 0 }))
    .bind(id)
    .execute(pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn delete_dialplan_rule(
    State(pool): State<Arc<SqlitePool>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM dialplan_rules WHERE id = ?")
        .bind(id)
        .execute(pool.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
