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
    pub qualify_frequency: i64,
    pub nat_mode: String,
    pub min_expires: i64,
    pub max_expires: i64,
    pub auth_required: i64,
    pub max_concurrent_logins: i64,
    pub allowed_transport: String,
}

impl Extension {
    pub fn is_recording_enabled(&self) -> bool {
        self.record_calls == 1
    }

    pub fn is_auth_required(&self) -> bool {
        self.auth_required == 1
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

        // Cleanup any historical duplicate dialplan rules
        let _ = sqlx::query(
            "DELETE FROM dialplan_rules WHERE id NOT IN (SELECT MIN(id) FROM dialplan_rules GROUP BY rule_name, pattern)",
        )
        .execute(&pool)
        .await;

        // Ensure missing columns exist in existing SQLite database file (Auto-migration)
        let _ = sqlx::query(
            "ALTER TABLE extensions ADD COLUMN qualify_frequency INTEGER NOT NULL DEFAULT 60",
        )
        .execute(&pool)
        .await;
        let _ =
            sqlx::query("ALTER TABLE extensions ADD COLUMN nat_mode TEXT NOT NULL DEFAULT 'auto'")
                .execute(&pool)
                .await;
        let _ = sqlx::query(
            "ALTER TABLE extensions ADD COLUMN min_expires INTEGER NOT NULL DEFAULT 60",
        )
        .execute(&pool)
        .await;
        let _ = sqlx::query(
            "ALTER TABLE extensions ADD COLUMN max_expires INTEGER NOT NULL DEFAULT 3600",
        )
        .execute(&pool)
        .await;
        let _ = sqlx::query(
            "ALTER TABLE extensions ADD COLUMN auth_required INTEGER NOT NULL DEFAULT 1",
        )
        .execute(&pool)
        .await;
        let _ = sqlx::query(
            "ALTER TABLE extensions ADD COLUMN max_concurrent_logins INTEGER NOT NULL DEFAULT 1",
        )
        .execute(&pool)
        .await;
        let _ = sqlx::query("ALTER TABLE extensions ADD COLUMN allowed_transport TEXT NOT NULL DEFAULT 'udp,tcp,tls,ws'").execute(&pool).await;

        info!("SQLite database schema verified and sample extensions seeded");

        Ok(DbStore { pool })
    }

    pub async fn load_extensions(&self) -> Result<Vec<Extension>, sqlx::Error> {
        let extensions = sqlx::query_as::<_, Extension>(
            r#"
            SELECT 
                id, extension_number, password, display_name, email, record_calls, is_active,
                qualify_frequency, nat_mode, min_expires, max_expires, auth_required,
                max_concurrent_logins, allowed_transport
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
        let expires_at = current_unix_secs() + expires_secs;
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

    pub async fn get_active_registration(
        &self,
        ext_num: &str,
    ) -> Result<Option<SipRegistration>, sqlx::Error> {
        let now = current_unix_secs();
        let reg = sqlx::query_as::<_, SipRegistration>(
            r#"
            SELECT extension_number, user_agent, contact_uri, source_ip, source_port, CAST(expires_at AS INTEGER) as expires_at
            FROM sip_registrations
            WHERE extension_number = ? AND CAST(expires_at AS INTEGER) > ?
            "#,
        )
        .bind(ext_num)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        Ok(reg)
    }

    pub async fn delete_registration(&self, ext_num: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM sip_registrations WHERE extension_number = ?")
            .bind(ext_num)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct SipRegistration {
    pub extension_number: String,
    pub user_agent: Option<String>,
    pub contact_uri: String,
    pub source_ip: String,
    pub source_port: i64,
    pub expires_at: i64,
}

fn current_unix_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
