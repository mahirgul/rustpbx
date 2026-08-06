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
}
