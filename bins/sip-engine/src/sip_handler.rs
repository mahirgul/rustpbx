use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};

use sipcore::services::digest::DigestAuth;
use sipcore::types::header::Header;
use sipcore::types::header_name::HeaderName;
use sipcore::types::headers::Headers;
use sipcore::types::message::{SipMessage, StartLine};
use sipcore::types::method::Method;
use sipcore::types::response::ResponseLine;
use sipcore::types::status_code::StatusCode;
use sipcore::types::version::Version;
use sipstack::UdpTransport;

use crate::config::Config;
use crate::db::DbStore;

pub async fn handle_incoming_sip_message(
    msg: SipMessage,
    src: SocketAddr,
    transport: &Arc<UdpTransport>,
    db: &DbStore,
    cfg: &Config,
) {
    match &msg.start_line {
        StartLine::Request(req) => {
            info!("Received SIP Request {} from {}", req.method, src);

            match req.method {
                Method::Register => {
                    handle_register(msg, src, transport, db, cfg).await;
                }
                Method::Options => {
                    handle_options(msg, src, transport).await;
                }
                _ => {
                    send_simple_response(msg, StatusCode::new(200).unwrap(), src, transport).await;
                }
            }
        }
        StartLine::Response(resp) => {
            info!(
                "Received SIP Response {} from {}",
                resp.status_code.code(),
                src
            );
        }
    }
}

async fn handle_register(
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

    // Extract user from Request URI or To header
    let target_user = if let StartLine::Request(ref req) = msg.start_line {
        req.uri.user.clone()
    } else {
        None
    };

    let target_user = match target_user {
        Some(u) => u,
        None => {
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

    // Check if Digest Auth is enabled
    if cfg.sip.require_digest_auth {
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
                    send_simple_response(msg, StatusCode::new(200).unwrap(), src, transport).await;
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
            "Digest Auth disabled. Granting 200 OK for Ext {} from {}",
            ext.extension_number, src
        );
        send_simple_response(msg, StatusCode::new(200).unwrap(), src, transport).await;
    }
}

async fn handle_options(msg: SipMessage, src: SocketAddr, transport: &Arc<UdpTransport>) {
    send_simple_response(msg, StatusCode::new(200).unwrap(), src, transport).await;
}

async fn send_digest_challenge(
    request: SipMessage,
    realm: &str,
    dest: SocketAddr,
    transport: &Arc<UdpTransport>,
) {
    let mut resp_headers = extract_base_headers(&request);
    let nonce = format!("nonce-{}", uuid_simple());

    let challenge_val = format!(
        "Digest realm=\"{}\", nonce=\"{}\", algorithm=MD5",
        realm, nonce
    );

    resp_headers.push(Header::new(
        HeaderName::WwwAuthenticate,
        Bytes::from(challenge_val),
    ));
    resp_headers.push(Header::new(
        HeaderName::UserAgent,
        Bytes::from("RustPBX/0.1.0"),
    ));
    resp_headers.push(Header::new(HeaderName::ContentLength, Bytes::from("0")));

    let status_code = StatusCode::new(401).unwrap();
    let resp_line = ResponseLine {
        version: Version::default(),
        status_code,
        reason_phrase: "Unauthorized".to_string(),
    };

    let resp_msg = SipMessage::new_response(resp_line, resp_headers, Bytes::new());
    let _ = transport.send_to(&resp_msg, dest).await;
}

async fn send_simple_response(
    request: SipMessage,
    status_code: StatusCode,
    dest: SocketAddr,
    transport: &Arc<UdpTransport>,
) {
    let mut resp_headers = extract_base_headers(&request);

    resp_headers.push(Header::new(
        HeaderName::UserAgent,
        Bytes::from("RustPBX/0.1.0"),
    ));
    resp_headers.push(Header::new(HeaderName::ContentLength, Bytes::from("0")));

    let resp_line = ResponseLine {
        version: Version::default(),
        status_code,
        reason_phrase: status_code.default_reason_phrase().to_string(),
    };

    let resp_msg = SipMessage::new_response(resp_line, resp_headers, Bytes::new());

    if let Err(e) = transport.send_to(&resp_msg, dest).await {
        warn!("Failed to send SIP response to {}: {}", dest, e);
    } else {
        info!("Sent SIP {} to {}", status_code.code(), dest);
    }
}

fn extract_base_headers(request: &SipMessage) -> Headers {
    let mut headers = Headers::new();
    if let Some(via) = request.headers.get(&HeaderName::Via) {
        headers.push(via.clone());
    }
    if let Some(from) = request.headers.get(&HeaderName::From) {
        headers.push(from.clone());
    }
    if let Some(to) = request.headers.get(&HeaderName::To) {
        headers.push(to.clone());
    }
    if let Some(call_id) = request.headers.get(&HeaderName::CallId) {
        headers.push(call_id.clone());
    }
    if let Some(cseq) = request.headers.get(&HeaderName::CSeq) {
        headers.push(cseq.clone());
    }
    headers
}

fn verify_digest_header(header: &str, password: &str, realm: &str) -> bool {
    let mut username = "";
    let mut nonce = "";
    let mut uri = "";
    let mut response = "";

    for part in header.trim_start_matches("Digest ").split(',') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            let val = v.trim_matches('"');
            match k {
                "username" => username = val,
                "nonce" => nonce = val,
                "uri" => uri = val,
                "response" => response = val,
                _ => {}
            }
        }
    }

    if username.is_empty() || nonce.is_empty() || uri.is_empty() || response.is_empty() {
        return false;
    }

    DigestAuth::verify(username, realm, password, nonce, "REGISTER", uri, response)
}

fn uuid_simple() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}
