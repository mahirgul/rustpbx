use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::info;

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
        info!(">>> OUTGOING SIP Message to {}:\n{}", target, payload);
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

        // Ignore short non-SIP UDP packets (such as PING keep-alives) silently
        if raw.len() < 10 {
            return Err("Short UDP packet ignored".to_string());
        }

        let raw_str = String::from_utf8_lossy(&raw);
        info!("<<< INCOMING SIP Message from {}:\n{}", src, raw_str);

        parse_message(raw).map(|msg| (msg, src)).map_err(|e| {
            info!("Non-SIP or unparseable UDP packet from {}: {}", src, e);
            e.to_string()
        })
    }
}
