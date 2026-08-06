use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;

use sipcore::types::header::Header;
use sipcore::types::header_name::HeaderName;
use sipcore::types::message::SipMessage;
use sipcore::types::response::ResponseLine;
use sipcore::types::status_code::StatusCode;
use sipcore::types::version::Version;
use sipstack::UdpTransport;

use super::utils::extract_base_headers;

pub async fn handle_options(msg: SipMessage, src: SocketAddr, transport: &Arc<UdpTransport>) {
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
