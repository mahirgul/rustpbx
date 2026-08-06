use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};

use sipcore::types::header_name::HeaderName;
use sipcore::types::message::SipMessage;
use sipcore::types::status_code::StatusCode;
use sipstack::UdpTransport;

use crate::config::Config;
use crate::db::DbStore;

use super::response::{send_digest_challenge, send_register_200_ok, send_simple_response};
use super::utils::{extract_extension_user, verify_digest_header};

pub async fn handle_register(
    msg: SipMessage,
    src: SocketAddr,
    transport: &Arc<UdpTransport>,
    db: &DbStore,
    cfg: &Config,
) {
    let extensions = match db.load_extensions().await {
        Ok(exts) => exts,
        Err(e) => {
            warn!("Failed to query DB for registration: {}", e);
            send_simple_response(msg, StatusCode::new(500).unwrap(), src, transport).await;
            return;
        }
    };

    // Extract extension user from Request-URI or To header (RFC 3261 §10.2)
    let target_user = extract_extension_user(&msg);

    let target_user = match target_user {
        Some(u) => u,
        None => {
            warn!("Registration failed: Could not determine extension number from Request-URI or To header");
            send_simple_response(msg, StatusCode::new(400).unwrap(), src, transport).await;
            return;
        }
    };

    let ext = match extensions
        .iter()
        .find(|e| e.extension_number == target_user)
    {
        Some(e) => e,
        None => {
            warn!(
                "Registration failed: Extension {} not found for {}",
                target_user, src
            );
            send_simple_response(msg, StatusCode::new(404).unwrap(), src, transport).await;
            return;
        }
    };

    // Check if Digest Auth is enabled (globally AND for this specific extension)
    if cfg.sip.require_digest_auth && ext.is_auth_required() {
        let auth_header = msg.headers.get(&HeaderName::Authorization);

        match auth_header {
            None => {
                info!(
                    "Sending 401 Unauthorized Digest Challenge for Ext {}",
                    ext.extension_number
                );
                send_digest_challenge(msg, &cfg.sip.domain, src, transport).await;
            }
            Some(header) => {
                let header_str = String::from_utf8_lossy(&header.raw_value);
                let is_valid = verify_digest_header(&header_str, &ext.password, &cfg.sip.domain);

                if is_valid {
                    info!(
                        "Digest Auth SUCCESS for Ext {} from {}",
                        ext.extension_number, src
                    );
                    let ua = msg
                        .headers
                        .get(&HeaderName::UserAgent)
                        .map(|h| String::from_utf8_lossy(&h.raw_value).to_string());
                    let contact = msg
                        .headers
                        .get(&HeaderName::Contact)
                        .map(|h| String::from_utf8_lossy(&h.raw_value).to_string())
                        .unwrap_or_else(|| src.to_string());

                    // Enforce extension's min/max expires limits
                    let granted_expires = ext.max_expires.clamp(ext.min_expires, 86400);

                    let _ = db
                        .upsert_registration(
                            &ext.extension_number,
                            ua.as_deref(),
                            &contact,
                            &src.ip().to_string(),
                            src.port() as i32,
                            granted_expires,
                        )
                        .await;
                    send_register_200_ok(msg, src, transport, granted_expires).await;
                } else {
                    warn!(
                        "Digest Auth FAILED for Ext {} from {}",
                        ext.extension_number, src
                    );
                    send_simple_response(msg, StatusCode::new(403).unwrap(), src, transport).await;
                }
            }
        }
    } else {
        info!(
            "Digest Auth disabled for Ext {}. Granting 200 OK from {}",
            ext.extension_number, src
        );
        let granted_expires = ext.max_expires.clamp(ext.min_expires, 86400);
        send_register_200_ok(msg, src, transport, granted_expires).await;
    }
}
