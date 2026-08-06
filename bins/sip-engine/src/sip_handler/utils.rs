use sipcore::services::digest::DigestAuth;
use sipcore::types::header_name::HeaderName;
use sipcore::types::headers::Headers;
use sipcore::types::message::{SipMessage, StartLine};

pub fn extract_base_headers(request: &SipMessage) -> Headers {
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

pub fn extract_extension_user(msg: &SipMessage) -> Option<String> {
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

pub fn extract_user_from_sip_str(input: &str) -> Option<String> {
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

pub fn verify_digest_header(header: &str, password: &str, realm: &str) -> bool {
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

pub fn uuid_simple() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}
