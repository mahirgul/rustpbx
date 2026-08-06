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
                Method::Invite => {
                    handle_invite(msg, src, transport, db, cfg).await;
                }
                Method::Bye => {
                    handle_bye(msg, src, transport).await;
                }
                Method::Cancel => {
                    handle_cancel(msg, src, transport).await;
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

async fn handle_options(msg: SipMessage, src: SocketAddr, transport: &Arc<UdpTransport>) {
    let mut resp_headers = extract_base_headers(&msg);
    resp_headers.push(Header::new(
        HeaderName::Allow,
        Bytes::from("INVITE, ACK, CANCEL, BYE, OPTIONS, REGISTER"),
    ));
    resp_headers.push(Header::new(
        HeaderName::Accept,
        Bytes::from("application/sdp"),
    ));
    resp_headers.push(Header::new(
        HeaderName::UserAgent,
        Bytes::from("RustPBX/0.1.0"),
    ));
    resp_headers.push(Header::new(HeaderName::ContentLength, Bytes::from("0")));

    let status_code = StatusCode::new(200).unwrap();
    let resp_line = ResponseLine {
        version: Version::default(),
        status_code,
        reason_phrase: "OK".to_string(),
    };

    let resp_msg = SipMessage::new_response(resp_line, resp_headers, Bytes::new());
    let _ = transport.send_to(&resp_msg, src).await;
}

async fn handle_invite(
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
                        if !verify_digest_header(&header_str, &ext.password, &cfg.sip.domain) {
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

    // 4. Send 180 Ringing to caller
    send_simple_response(msg.clone(), StatusCode::new(180).unwrap(), src, transport).await;
    info!(
        "INVITE call routed to target extension {} from {}",
        target_user, src
    );
}

async fn handle_bye(msg: SipMessage, src: SocketAddr, transport: &Arc<UdpTransport>) {
    info!("Processing BYE call teardown request from {}", src);
    send_simple_response(msg, StatusCode::new(200).unwrap(), src, transport).await;
}

async fn handle_cancel(msg: SipMessage, src: SocketAddr, transport: &Arc<UdpTransport>) {
    info!("Processing CANCEL call cancellation request from {}", src);
    send_simple_response(msg.clone(), StatusCode::new(200).unwrap(), src, transport).await;
    send_simple_response(msg, StatusCode::new(487).unwrap(), src, transport).await;
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

async fn send_register_200_ok(
    request: SipMessage,
    dest: SocketAddr,
    transport: &Arc<UdpTransport>,
    granted_expires: i64,
) {
    let mut resp_headers = extract_base_headers(&request);

    if let Some(contact) = request.headers.get(&HeaderName::Contact) {
        resp_headers.push(contact.clone());
    }
    resp_headers.push(Header::new(
        HeaderName::Expires,
        Bytes::from(format!("{}", granted_expires)),
    ));
    resp_headers.push(Header::new(
        HeaderName::UserAgent,
        Bytes::from("RustPBX/0.1.0"),
    ));
    resp_headers.push(Header::new(HeaderName::ContentLength, Bytes::from("0")));

    let status_code = StatusCode::new(200).unwrap();
    let resp_line = ResponseLine {
        version: Version::default(),
        status_code,
        reason_phrase: "OK".to_string(),
    };

    let resp_msg = SipMessage::new_response(resp_line, resp_headers, Bytes::new());
    if let Err(e) = transport.send_to(&resp_msg, dest).await {
        warn!("Failed to send REGISTER 200 OK to {}: {}", dest, e);
    } else {
        info!(
            "Sent REGISTER 200 OK (Expires: {}) to {}",
            granted_expires, dest
        );
    }
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

fn extract_extension_user(msg: &SipMessage) -> Option<String> {
    // 1. Check Request-URI
    if let StartLine::Request(ref req) = msg.start_line {
        if let Some(user) = &req.uri.user {
            if !user.is_empty() {
                return Some(user.clone());
            }
        }
    }

    // 2. Check To Header (e.g. To: <sip:100@pbx.local>)
    if let Some(to) = msg.headers.get(&HeaderName::To) {
        let to_str = String::from_utf8_lossy(&to.raw_value);
        if let Some(user) = extract_user_from_sip_str(&to_str) {
            return Some(user);
        }
    }

    // 3. Check From Header
    if let Some(from) = msg.headers.get(&HeaderName::From) {
        let from_str = String::from_utf8_lossy(&from.raw_value);
        if let Some(user) = extract_user_from_sip_str(&from_str) {
            return Some(user);
        }
    }

    None
}

fn extract_user_from_sip_str(input: &str) -> Option<String> {
    if let Some(start) = input.find("sip:") {
        let rest = &input[start + 4..];
        if let Some(at) = rest.find('@') {
            let user = &rest[..at];
            let clean_user = user.trim_start_matches('"').trim_start_matches('<');
            if !clean_user.is_empty() {
                return Some(clean_user.to_string());
            }
        }
    }
    None
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
