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
    pub is_registered: bool,
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
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let rows = sqlx::query(
        r#"
        SELECT 
            e.id, 
            e.extension_number, 
            e.password, 
            e.display_name, 
            e.email, 
            e.record_calls, 
            e.is_active,
            CASE WHEN r.extension_number IS NOT NULL AND CAST(r.expires_at AS INTEGER) > ? THEN 1 ELSE 0 END AS is_registered
        FROM extensions e
        LEFT JOIN sip_registrations r ON e.extension_number = r.extension_number
        ORDER BY e.extension_number ASC
        "#,
    )
    .bind(now_secs)
    .fetch_all(pool.as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    use sqlx::Row;
    let extensions = rows
        .into_iter()
        .map(|r| Extension {
            id: r.get("id"),
            extension_number: r.get("extension_number"),
            password: r.get("password"),
            display_name: r.get("display_name"),
            email: r.get("email"),
            record_calls: r.get("record_calls"),
            is_active: r.get("is_active"),
            is_registered: r.get::<i64, _>("is_registered") == 1,
        })
        .collect();

    Ok(Json(extensions))
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

#[derive(Debug, Deserialize)]
pub struct UpdateExtensionRequest {
    pub password: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub record_calls: Option<bool>,
    pub is_active: Option<bool>,
}

pub async fn update_extension(
    State(pool): State<Arc<SqlitePool>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(payload): Json<UpdateExtensionRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut query_builder = String::from("UPDATE extensions SET ");
    let mut updates = Vec::new();

    if payload.display_name.is_some() {
        updates.push("display_name = ?");
    }
    if payload.password.is_some() {
        updates.push("password = ?");
    }
    if payload.email.is_some() {
        updates.push("email = ?");
    }
    if payload.record_calls.is_some() {
        updates.push("record_calls = ?");
    }
    if payload.is_active.is_some() {
        updates.push("is_active = ?");
    }

    if updates.is_empty() {
        return Ok(StatusCode::OK);
    }

    query_builder.push_str(&updates.join(", "));
    query_builder.push_str(" WHERE id = ?");

    let mut query = sqlx::query(&query_builder);

    if let Some(name) = &payload.display_name {
        query = query.bind(name);
    }
    if let Some(pass) = &payload.password {
        query = query.bind(pass);
    }
    if let Some(email) = &payload.email {
        query = query.bind(email);
    }
    if let Some(rec) = payload.record_calls {
        query = query.bind(if rec { 1 } else { 0 });
    }
    if let Some(active) = payload.is_active {
        query = query.bind(if active { 1 } else { 0 });
    }

    query = query.bind(id);

    query
        .execute(pool.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn delete_extension(
    State(pool): State<Arc<SqlitePool>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM extensions WHERE id = ?")
        .bind(id)
        .execute(pool.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
