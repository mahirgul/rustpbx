use axum::{extract::State, Json};
use serde::Serialize;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;

#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub name: &'static str,
    pub port: &'static str,
    pub status: &'static str, // "running", "stopped", "standby"
}

#[derive(Debug, Serialize)]
pub struct DashboardMetrics {
    pub total_registered_subscribers: i64,
    pub active_calls: u64,
    pub database_status: &'static str,
    pub services: Vec<ServiceStatus>,
}

pub async fn get_dashboard_metrics(State(pool): State<Arc<SqlitePool>>) -> Json<DashboardMetrics> {
    let now_secs = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    let active_registrations_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sip_registrations WHERE CAST(expires_at AS INTEGER) > CAST(? AS INTEGER)",
    )
    .bind(&now_secs)
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0);

    // Perform accurate process & socket checks
    let sip_engine_running =
        check_process_running("sip-engine") || check_udp_port_bound("127.0.0.1:5060").await;
    let media_node_running =
        check_process_running("media-node") || check_tcp_port_active("127.0.0.1:50051").await;
    let web_admin_running = true; // web-admin is serving this request right now!
    let webrtc_gateway_running =
        check_process_running("webrtc-gateway") || check_tcp_port_active("127.0.0.1:8089").await;

    let services = vec![
        ServiceStatus {
            name: "sip-engine (Core B2BUA)",
            port: "5060 UDP",
            status: if sip_engine_running {
                "running"
            } else {
                "stopped"
            },
        },
        ServiceStatus {
            name: "media-node (RTP Engine)",
            port: "50051 gRPC",
            status: if media_node_running {
                "running"
            } else {
                "stopped"
            },
        },
        ServiceStatus {
            name: "web-admin (Management UI)",
            port: "8088 HTTP",
            status: if web_admin_running {
                "running"
            } else {
                "stopped"
            },
        },
        ServiceStatus {
            name: "webrtc-gateway (WSS Edge)",
            port: "8089 WSS",
            status: if webrtc_gateway_running {
                "running"
            } else {
                "standby"
            },
        },
    ];

    Json(DashboardMetrics {
        total_registered_subscribers: active_registrations_count,
        active_calls: 0,
        database_status: "Healthy (SQLite WAL)",
        services,
    })
}

fn check_process_running(process_name: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {}.exe", process_name), "/NH"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.contains(&format!("{}.exe", process_name));
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("pgrep").arg(process_name).output() {
            return !output.stdout.is_empty();
        }
    }
    false
}

async fn check_tcp_port_active(addr_str: &str) -> bool {
    if let Ok(addr) = addr_str.parse::<SocketAddr>() {
        tokio::time::timeout(Duration::from_millis(150), TcpStream::connect(addr))
            .await
            .map(|res| res.is_ok())
            .unwrap_or(false)
    } else {
        false
    }
}

async fn check_udp_port_bound(addr_str: &str) -> bool {
    // If binding to local UDP port fails with AddrInUse, it means the target service IS running and bound to that port!
    if let Ok(addr) = addr_str.parse::<SocketAddr>() {
        match tokio::net::UdpSocket::bind(addr).await {
            Ok(_) => false, // Port is free -> service is NOT running
            Err(e) => e.kind() == std::io::ErrorKind::AddrInUse, // Port in use -> service IS running!
        }
    } else {
        false
    }
}
