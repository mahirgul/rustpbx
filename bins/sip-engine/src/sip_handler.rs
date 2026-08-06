use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};

use sipcore::types::header::Header;
use sipcore::types::header_name::HeaderName;
use sipcore::types::headers::Headers;
use sipcore::types::message::{SipMessage, StartLine};
use sipcore::types::method::Method;
use sipcore::types::response::ResponseLine;
use sipcore::types::status_code::StatusCode;
use sipcore::types::version::Version;
use sipstack::UdpTransport;

use crate::db::DbStore;

pub async fn handle_incoming_sip_message(
    msg: SipMessage,
    src: SocketAddr,
    transport: &Arc<UdpTransport>,
    db: &DbStore,
) {
    match &msg.start_line {
        StartLine::Request(req) => {
            info!("Received SIP Request {} from {}", req.method, src);

            match req.method {
                Method::Register => {
                    handle_register(msg, src, transport, db).await;
                }
                Method::Options => {
                    handle_options(msg, src, transport).await;
                }
                _ => {
                    info!("Method {} received, sending 200 OK placeholder", req.method);
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
    let user_found = if let StartLine::Request(ref req) = msg.start_line {
        if let Some(user) = &req.uri.user {
            extensions.iter().any(|e| e.extension_number == *user)
        } else {
            false
        }
    } else {
        false
    };

    if user_found {
        info!("Registration successful for endpoint at {}", src);
        send_simple_response(msg, StatusCode::new(200).unwrap(), src, transport).await;
    } else {
        warn!(
            "Registration failed: Extension not found for endpoint at {}",
            src
        );
        send_simple_response(msg, StatusCode::new(404).unwrap(), src, transport).await;
    }
}

async fn handle_options(msg: SipMessage, src: SocketAddr, transport: &Arc<UdpTransport>) {
    send_simple_response(msg, StatusCode::new(200).unwrap(), src, transport).await;
}

async fn send_simple_response(
    request: SipMessage,
    status_code: StatusCode,
    dest: SocketAddr,
    transport: &Arc<UdpTransport>,
) {
    let mut resp_headers = Headers::new();

    if let Some(via) = request.headers.get(&HeaderName::Via) {
        resp_headers.push(via.clone());
    }
    if let Some(from) = request.headers.get(&HeaderName::From) {
        resp_headers.push(from.clone());
    }
    if let Some(to) = request.headers.get(&HeaderName::To) {
        resp_headers.push(to.clone());
    }
    if let Some(call_id) = request.headers.get(&HeaderName::CallId) {
        resp_headers.push(call_id.clone());
    }
    if let Some(cseq) = request.headers.get(&HeaderName::CSeq) {
        resp_headers.push(cseq.clone());
    }

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
