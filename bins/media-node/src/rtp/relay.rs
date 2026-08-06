use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::info;

#[allow(dead_code)]
pub struct RtpRelaySession {
    pub session_id: String,
    pub socket: Arc<UdpSocket>,
    pub remote_target: Option<SocketAddr>,
}

impl RtpRelaySession {
    pub async fn bind(
        session_id: String,
        bind_ip: &str,
        port: u16,
    ) -> Result<Self, std::io::Error> {
        let addr = format!("{}:{}", bind_ip, port);
        let socket = UdpSocket::bind(&addr).await?;
        info!("RTP Relay Session {} bound on {}", session_id, addr);

        Ok(RtpRelaySession {
            session_id,
            socket: Arc::new(socket),
            remote_target: None,
        })
    }
}
