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
    pub qualify_frequency: i64,
    pub nat_mode: String,
    pub min_expires: i64,
    pub max_expires: i64,
    pub auth_required: i64,
    pub max_concurrent_logins: i64,
    pub allowed_transport: String,
    pub is_registered: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateExtensionRequest {
    pub extension_number: String,
    pub password: Option<String>,
    pub display_name: String,
    pub email: Option<String>,
    pub record_calls: Option<bool>,
    pub qualify_frequency: Option<i64>,
    pub nat_mode: Option<String>,
    pub min_expires: Option<i64>,
    pub max_expires: Option<i64>,
    pub auth_required: Option<bool>,
    pub max_concurrent_logins: Option<i64>,
    pub allowed_transport: Option<String>,
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
            e.qualify_frequency,
            e.nat_mode,
            e.min_expires,
            e.max_expires,
            e.auth_required,
            e.max_concurrent_logins,
            e.allowed_transport,
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
            qualify_frequency: r.get("qualify_frequency"),
            nat_mode: r.get("nat_mode"),
            min_expires: r.get("min_expires"),
            max_expires: r.get("max_expires"),
            auth_required: r.get("auth_required"),
            max_concurrent_logins: r.get("max_concurrent_logins"),
            allowed_transport: r.get("allowed_transport"),
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
    let qualify_frequency = payload.qualify_frequency.unwrap_or(60);
    let nat_mode = payload.nat_mode.unwrap_or_else(|| "auto".to_string());
    let min_expires = payload.min_expires.unwrap_or(60);
    let max_expires = payload.max_expires.unwrap_or(3600);
    let auth_required = if payload.auth_required.unwrap_or(true) {
        1
    } else {
        0
    };
    let max_concurrent_logins = payload.max_concurrent_logins.unwrap_or(1);
    let allowed_transport = payload
        .allowed_transport
        .unwrap_or_else(|| "udp,tcp,tls,ws".to_string());

    let password = payload
        .password
        .filter(|p| !p.trim().is_empty())
        .unwrap_or_else(|| format!("{}100", payload.extension_number));

    let result = sqlx::query(
        r#"
        INSERT INTO extensions (
            extension_number, password, display_name, email, record_calls,
            qualify_frequency, nat_mode, min_expires, max_expires, auth_required,
            max_concurrent_logins, allowed_transport
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&payload.extension_number)
    .bind(&password)
    .bind(&payload.display_name)
    .bind(&payload.email)
    .bind(record_calls)
    .bind(qualify_frequency)
    .bind(nat_mode)
    .bind(min_expires)
    .bind(max_expires)
    .bind(auth_required)
    .bind(max_concurrent_logins)
    .bind(allowed_transport)
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
    pub qualify_frequency: Option<i64>,
    pub nat_mode: Option<String>,
    pub min_expires: Option<i64>,
    pub max_expires: Option<i64>,
    pub auth_required: Option<bool>,
    pub max_concurrent_logins: Option<i64>,
    pub allowed_transport: Option<String>,
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
    if payload.qualify_frequency.is_some() {
        updates.push("qualify_frequency = ?");
    }
    if payload.nat_mode.is_some() {
        updates.push("nat_mode = ?");
    }
    if payload.min_expires.is_some() {
        updates.push("min_expires = ?");
    }
    if payload.max_expires.is_some() {
        updates.push("max_expires = ?");
    }
    if payload.auth_required.is_some() {
        updates.push("auth_required = ?");
    }
    if payload.max_concurrent_logins.is_some() {
        updates.push("max_concurrent_logins = ?");
    }
    if payload.allowed_transport.is_some() {
        updates.push("allowed_transport = ?");
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
    if let Some(q) = payload.qualify_frequency {
        query = query.bind(q);
    }
    if let Some(nat) = &payload.nat_mode {
        query = query.bind(nat);
    }
    if let Some(min_exp) = payload.min_expires {
        query = query.bind(min_exp);
    }
    if let Some(max_exp) = payload.max_expires {
        query = query.bind(max_exp);
    }
    if let Some(auth) = payload.auth_required {
        query = query.bind(if auth { 1 } else { 0 });
    }
    if let Some(logins) = payload.max_concurrent_logins {
        query = query.bind(logins);
    }
    if let Some(transports) = &payload.allowed_transport {
        query = query.bind(transports);
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
