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

    // Extract requested expires duration (RFC 3261 §10.2)
    let req_expires = extract_requested_expires(&msg);

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
                    process_registration_grant(msg, src, transport, db, ext, req_expires).await;
                } else {
                    let log_msg = format!(
                        "Digest Auth FAILED for Ext {} from {}",
                        ext.extension_number, src
                    );
                    warn!("{}", log_msg);
                    crate::logger::log_auth_audit(&log_msg);
                    send_simple_response(msg, StatusCode::new(403).unwrap(), src, transport).await;
                }
            }
        }
    } else {
        process_registration_grant(msg, src, transport, db, ext, req_expires).await;
    }
}

async fn process_registration_grant(
    msg: SipMessage,
    src: SocketAddr,
    transport: &Arc<UdpTransport>,
    db: &DbStore,
    ext: &crate::db::Extension,
    req_expires: Option<i64>,
) {
    // If client requested 0 seconds, unregister/logout extension (RFC 3261 §10.2.2)
    if req_expires == Some(0) {
        let log_msg = format!(
            "Ext {} UNREGISTERED (Expires: 0) from {}",
            ext.extension_number, src
        );
        info!("{}", log_msg);
        crate::logger::log_auth_audit(&log_msg);

        let _ = db.delete_registration(&ext.extension_number).await;
        send_register_200_ok(msg, src, transport, 0).await;
        return;
    }

    let granted_expires = req_expires
        .unwrap_or(ext.max_expires)
        .clamp(ext.min_expires, ext.max_expires);

    let log_msg = format!(
        "Digest Auth SUCCESS for Ext {} from {} (Granted Expires: {})",
        ext.extension_number, src, granted_expires
    );
    info!("{}", log_msg);
    crate::logger::log_auth_audit(&log_msg);

    let ua = msg
        .headers
        .get(&HeaderName::UserAgent)
        .map(|h| String::from_utf8_lossy(&h.raw_value).to_string());
    let contact = msg
        .headers
        .get(&HeaderName::Contact)
        .map(|h| String::from_utf8_lossy(&h.raw_value).to_string())
        .unwrap_or_else(|| src.to_string());

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
}

fn extract_requested_expires(msg: &SipMessage) -> Option<i64> {
    if let Some(exp_header) = msg.headers.get(&HeaderName::Expires) {
        let val_str = String::from_utf8_lossy(&exp_header.raw_value);
        if let Ok(val) = val_str.trim().parse::<i64>() {
            return Some(val);
        }
    }
    if let Some(contact) = msg.headers.get(&HeaderName::Contact) {
        let c_str = String::from_utf8_lossy(&contact.raw_value);
        if let Some(pos) = c_str.find("expires=") {
            let rest = &c_str[pos + 8..];
            let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(val) = num_str.parse::<i64>() {
                return Some(val);
            }
        }
    }
    None
}
