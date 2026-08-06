mod api;
mod config;
mod static_files;

use api::create_api_router;
use config::Config;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use static_files::static_handler;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cfg = Config::default();
    let http_addr: SocketAddr = cfg.server.http_bind_addr.parse()?;

    info!("Starting RustPBX Web Admin Interface...");
    info!("Web Admin Dashboard listening at http://{}", http_addr);

    // Connect to shared SQLite database
    let options = SqliteConnectOptions::from_str(&cfg.database.db_path)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = Arc::new(
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?,
    );

    let app = create_api_router(pool).fallback(static_handler);

    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
