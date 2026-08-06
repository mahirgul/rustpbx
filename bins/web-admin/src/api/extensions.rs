use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Extension {
    pub id: i64,
    pub extension_number: String,
    pub password: String,
    pub display_name: String,
    pub email: Option<String>,
    pub record_calls: i64,
    pub is_active: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateExtensionRequest {
    pub extension_number: String,
    pub password: String,
    pub display_name: String,
    pub email: Option<String>,
    pub record_calls: Option<bool>,
}

pub async fn list_extensions(
    State(pool): State<Arc<SqlitePool>>,
) -> Result<Json<Vec<Extension>>, StatusCode> {
    let rows = sqlx::query_as::<_, Extension>(
        r#"
        SELECT id, extension_number, password, display_name, email, record_calls, is_active
        FROM extensions
        ORDER BY extension_number ASC
        "#,
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows))
}

pub async fn create_extension(
    State(pool): State<Arc<SqlitePool>>,
    Json(payload): Json<CreateExtensionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let record_calls = if payload.record_calls.unwrap_or(false) {
        1
    } else {
        0
    };

    let result = sqlx::query(
        r#"
        INSERT INTO extensions (extension_number, password, display_name, email, record_calls)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(&payload.extension_number)
    .bind(&payload.password)
    .bind(&payload.display_name)
    .bind(&payload.email)
    .bind(record_calls)
    .execute(pool.as_ref())
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": result.last_insert_rowid(),
            "extension_number": payload.extension_number
        })),
    ))
}
