use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};

use sipcore::types::header_name::HeaderName;
use sipcore::types::message::SipMessage;
use sipcore::types::status_code::StatusCode;
use sipstack::UdpTransport;

use crate::config::Config;
use crate::db::DbStore;

use super::response::{send_digest_challenge, send_simple_response};
use super::utils::{extract_extension_user, extract_user_from_sip_str, verify_digest_header};

pub async fn handle_invite(
    msg: SipMessage,
    src: SocketAddr,
    transport: &Arc<UdpTransport>,
    db: &DbStore,
    cfg: &Config,
) {
    info!("Processing INVITE request from {}", src);

    // 1. Send 100 Trying to caller immediately (RFC 3261 §17.2.1)
    send_simple_response(msg.clone(), StatusCode::new(100).unwrap(), src, transport).await;

    // 2. Extract destination extension number
    let target_user = match extract_extension_user(&msg) {
        Some(u) => u,
        None => {
            send_simple_response(msg, StatusCode::new(400).unwrap(), src, transport).await;
            return;
        }
    };

    // 3. Digest Auth Check for Caller if enabled
    if cfg.sip.require_digest_auth {
        if let Some(auth_header) = msg.headers.get(&HeaderName::Authorization) {
            let header_str = String::from_utf8_lossy(&auth_header.raw_value);
            // Verify caller identity against DB
            let caller_user = extract_user_from_sip_str(&header_str).unwrap_or_default();
            if !caller_user.is_empty() {
                if let Ok(exts) = db.load_extensions().await {
                    if let Some(ext) = exts.iter().find(|e| e.extension_number == caller_user) {
                        if !verify_digest_header(
                            &header_str,
                            &ext.password,
                            &cfg.sip.domain,
                            "INVITE",
                        ) {
                            warn!("INVITE Digest Auth FAILED for caller {}", caller_user);
                            send_simple_response(
                                msg,
                                StatusCode::new(403).unwrap(),
                                src,
                                transport,
                            )
                            .await;
                            return;
                        }
                    }
                }
            }
        } else {
            info!("Sending 401 Digest Challenge for INVITE from {}", src);
            send_digest_challenge(msg, &cfg.sip.domain, src, transport).await;
            return;
        }
    }

    // 4. Query Callee Registration Status & Forward INVITE
    match db.get_active_registration(&target_user).await {
        Ok(Some(reg)) => {
            let target_addr: Result<SocketAddr, _> =
                format!("{}:{}", reg.source_ip, reg.source_port).parse();
            if let Ok(dest_sock) = target_addr {
                info!(
                    "Forwarding INVITE for Ext {} to registered target endpoint {}",
                    target_user, dest_sock
                );
                let _ = transport.send_to(&msg, dest_sock).await;
                send_simple_response(msg.clone(), StatusCode::new(180).unwrap(), src, transport)
                    .await;
            } else {
                warn!("Invalid SocketAddr for callee Ext {}", target_user);
                send_simple_response(msg, StatusCode::new(480).unwrap(), src, transport).await;
            }
        }
        _ => {
            warn!(
                "Callee extension {} is offline or not registered",
                target_user
            );
            send_simple_response(msg, StatusCode::new(480).unwrap(), src, transport).await;
        }
    }
}

pub async fn handle_bye(msg: SipMessage, src: SocketAddr, transport: &Arc<UdpTransport>) {
    info!("Processing BYE call teardown request from {}", src);
    send_simple_response(msg, StatusCode::new(200).unwrap(), src, transport).await;
}

pub async fn handle_cancel(msg: SipMessage, src: SocketAddr, transport: &Arc<UdpTransport>) {
    info!("Processing CANCEL call cancellation request from {}", src);
    send_simple_response(msg.clone(), StatusCode::new(200).unwrap(), src, transport).await;
    send_simple_response(msg, StatusCode::new(487).unwrap(), src, transport).await;
}
