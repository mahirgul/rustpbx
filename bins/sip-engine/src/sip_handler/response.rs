use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};

use sipcore::types::header::Header;
use sipcore::types::header_name::HeaderName;
use sipcore::types::message::SipMessage;
use sipcore::types::response::ResponseLine;
use sipcore::types::status_code::StatusCode;
use sipcore::types::version::Version;
use sipstack::UdpTransport;

use super::utils::{extract_base_headers, uuid_simple};

pub async fn send_digest_challenge(
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

pub async fn send_register_200_ok(
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

pub async fn send_simple_response(
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
