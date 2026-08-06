use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{error, info};

use sipcore::parser::parse_message;
use sipcore::types::SipMessage;

pub struct UdpTransport {
    socket: Arc<UdpSocket>,
}

impl UdpTransport {
    pub async fn bind(bind_addr: SocketAddr) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(bind_addr).await?;
        info!("SIP UDP Transport bound to {}", bind_addr);
        Ok(UdpTransport {
            socket: Arc::new(socket),
        })
    }

    pub async fn send_to(
        &self,
        msg: &SipMessage,
        target: SocketAddr,
    ) -> Result<usize, std::io::Error> {
        let payload = msg.to_string();
        self.socket.send_to(payload.as_bytes(), target).await
    }

    pub async fn recv_message(&self) -> Result<(SipMessage, SocketAddr), String> {
        let mut buf = vec![0u8; 65535];
        let (len, src) = self
            .socket
            .recv_from(&mut buf)
            .await
            .map_err(|e| e.to_string())?;

        buf.truncate(len);
        let raw = Bytes::from(buf);

        parse_message(raw).map(|msg| (msg, src)).map_err(|e| {
            error!("Failed to parse SIP UDP packet from {}: {}", src, e);
            e.to_string()
        })
    }
}
