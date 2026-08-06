use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[allow(dead_code)]
pub struct RtpRelaySession {
    pub session_id: String,
    pub socket: Arc<UdpSocket>,
    pub local_port: u16,
    pub remote_target: Arc<Mutex<Option<SocketAddr>>>,
    pub is_running: Arc<AtomicBool>,
}

impl RtpRelaySession {
    pub async fn bind(
        session_id: String,
        bind_ip: &str,
        port: u16,
        remote_ip: &str,
        remote_port: u16,
    ) -> Result<Self, std::io::Error> {
        let addr = format!("{}:{}", bind_ip, port);
        let socket = Arc::new(UdpSocket::bind(&addr).await?);
        info!("RTP Relay Session {} bound on {}", session_id, addr);

        let target_addr: Option<SocketAddr> = if !remote_ip.is_empty() && remote_port > 0 {
            format!("{}:{}", remote_ip, remote_port).parse().ok()
        } else {
            None
        };

        let remote_target = Arc::new(Mutex::new(target_addr));
        let is_running = Arc::new(AtomicBool::new(true));

        // Spawn async RTP relay loop
        let socket_clone = socket.clone();
        let target_clone = remote_target.clone();
        let running_clone = is_running.clone();
        let sess_id_clone = session_id.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 2048];
            while running_clone.load(Ordering::Relaxed) {
                match socket_clone.recv_from(&mut buf).await {
                    Ok((len, src)) => {
                        let mut target_guard = target_clone.lock().await;
                        // Auto-learn remote target address if not set (Symmetric RTP)
                        if target_guard.is_none() {
                            *target_guard = Some(src);
                            info!(
                                "RTP Session {} learned remote target address: {}",
                                sess_id_clone, src
                            );
                        }

                        if let Some(dest) = *target_guard {
                            if dest != src {
                                let _ = socket_clone.send_to(&buf[..len], dest).await;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("RTP Relay recv error on session {}: {}", sess_id_clone, e);
                        break;
                    }
                }
            }
            info!("RTP Relay loop stopped for session {}", sess_id_clone);
        });

        Ok(RtpRelaySession {
            session_id,
            socket,
            local_port: port,
            remote_target,
            is_running,
        })
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}
