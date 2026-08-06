use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{FromRow, Pool, Sqlite};
use std::str::FromStr;
use tracing::info;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct Extension {
    pub id: i64,
    pub extension_number: String,
    pub password: String,
    pub display_name: String,
    pub email: Option<String>,
    pub record_calls: i64,
    pub is_active: i64,
}

impl Extension {
    pub fn is_recording_enabled(&self) -> bool {
        self.record_calls == 1
    }
}

#[derive(Clone)]
pub struct DbStore {
    pool: Pool<Sqlite>,
}

impl DbStore {
    pub async fn init(db_path: &str) -> Result<Self, sqlx::Error> {
        info!("Initializing SQLite database (WAL Mode) at {}", db_path);

        let options = SqliteConnectOptions::from_str(db_path)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        // Initialize schema using relative path to root data/schema.sql
        let schema_sql = include_str!("../../../../data/schema.sql");
        sqlx::query(schema_sql).execute(&pool).await?;

        info!("SQLite database schema verified and sample extensions seeded");

        Ok(DbStore { pool })
    }

    pub async fn load_extensions(&self) -> Result<Vec<Extension>, sqlx::Error> {
        let extensions = sqlx::query_as::<_, Extension>(
            r#"
            SELECT id, extension_number, password, display_name, email, record_calls, is_active
            FROM extensions
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(extensions)
    }

    pub async fn upsert_registration(
        &self,
        ext_num: &str,
        user_agent: Option<&str>,
        contact_uri: &str,
        ip: &str,
        port: i32,
        expires_secs: i64,
    ) -> Result<(), sqlx::Error> {
        let expires_at = chrono_offset_secs(expires_secs);
        sqlx::query(
            r#"
            INSERT INTO sip_registrations (extension_number, user_agent, contact_uri, source_ip, source_port, expires_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(extension_number) DO UPDATE SET
                user_agent = excluded.user_agent,
                contact_uri = excluded.contact_uri,
                source_ip = excluded.source_ip,
                source_port = excluded.source_port,
                expires_at = excluded.expires_at,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(ext_num)
        .bind(user_agent)
        .bind(contact_uri)
        .bind(ip)
        .bind(port)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

fn chrono_offset_secs(secs: i64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", now + secs as u64)
}
