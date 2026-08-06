mod config;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use config::Config;
use std::net::SocketAddr;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cfg = Config::default();
    let wss_addr: SocketAddr = cfg.server.wss_bind_addr.parse()?;

    info!("Starting RustPBX WebRTC Edge Border Gateway...");
    info!("WebSocket (WSS) Signaling server listening on {}", wss_addr);
    info!(
        "Bridging to Core SIP Engine UDP target {}",
        cfg.sip_engine.sip_udp_target
    );

    let app = Router::new().route("/ws", get(ws_handler));

    let listener = tokio::net::TcpListener::bind(wss_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    info!("New WebRTC Browser WebSocket client connected");

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                info!("Received WebRTC Signaling Message: {}", text);

                // Echo back WebRTC SDP Answer / Candidate acknowledgment for negotiation
                let response = serde_json::json!({
                    "type": "ack",
                    "status": "connected",
                    "message": "RustPBX WebRTC Gateway ready for SDP offer"
                });

                if socket
                    .send(Message::Text(response.to_string()))
                    .await
                    .is_err()
                {
                    warn!("Failed to send WebSocket message to client");
                    break;
                }
            }
            Message::Close(_) => {
                info!("WebRTC Client disconnected");
                break;
            }
            _ => {}
        }
    }
}
