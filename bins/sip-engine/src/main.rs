mod api;
mod b2bua;
mod config;
mod db;
mod sbc;
mod sip_handler;

use api::{create_rest_router, AppState};
use b2bua::CallManager;
use config::Config;
use db::DbStore;
use sbc::SbcPipeline;
use sipstack::UdpTransport;
use std::net::SocketAddr;
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
    let sip_addr: SocketAddr = cfg.sip.udp_bind_addr.parse()?;
    let http_addr: SocketAddr = cfg.api.http_bind_addr.parse()?;

    info!("Starting RustPBX Core SIP Engine...");

    // Initialize SQLite Database and load extensions
    std::fs::create_dir_all("data")?;
    let db_store = DbStore::init(&cfg.database.db_path).await?;
    let extensions = db_store.load_extensions().await?;
    for ext in &extensions {
        info!(
            "Loaded Extension {}: {} (Record calls: {})",
            ext.extension_number,
            ext.display_name,
            ext.is_recording_enabled()
        );
    }
    info!("SIP UDP Transport listening on {}", sip_addr);
    info!("HTTP REST Control API listening on http://{}", http_addr);

    let call_manager = Arc::new(CallManager::new());
    let _sbc_pipeline = SbcPipeline::default();

    // Spawn SIP UDP Transport Listener
    let sip_transport = Arc::new(UdpTransport::bind(sip_addr).await?);
    let sip_transport_listener = sip_transport.clone();
    let db_store_listener = db_store.clone();
    let cfg_listener = cfg.clone();
    let _sip_transport_task = tokio::spawn(async move {
        loop {
            match sip_transport_listener.recv_message().await {
                Ok((msg, src)) => {
                    sip_handler::handle_incoming_sip_message(
                        msg,
                        src,
                        &sip_transport_listener,
                        &db_store_listener,
                        &cfg_listener,
                    )
                    .await;
                }
                Err(e) => {
                    tracing::error!("SIP recv error: {}", e);
                }
            }
        }
    });

    // Spawn REST API HTTP Server
    let app_state = Arc::new(AppState {
        call_manager: call_manager.clone(),
    });
    let router = create_rest_router(app_state);

    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
