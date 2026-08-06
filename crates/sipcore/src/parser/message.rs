use crate::error::SipError;
use crate::parser::header::parse_headers;
use crate::parser::uri::parse_uri;
use crate::types::message::{SipMessage, StartLine};
use crate::types::method::Method;
use crate::types::request::RequestLine;
use crate::types::response::ResponseLine;
use crate::types::status_code::StatusCode;
use crate::types::version::Version;
use bytes::Bytes;
use std::str::FromStr;

/// Parse raw network buffer into a zero-copy SipMessage.
pub fn parse_message(raw: Bytes) -> Result<SipMessage, SipError> {
    let text = std::str::from_utf8(&raw)
        .map_err(|_| SipError::ParseError("Invalid UTF-8 in message".to_string()))?;

    let mut lines = text.split("\r\n");
    let start_line_str = lines
        .next()
        .ok_or_else(|| SipError::ParseError("Empty SIP message".to_string()))?;

    let start_line = parse_start_line(start_line_str)?;

    let header_start_idx = start_line_str.len() + 2;
    let (headers, header_bytes_len) = parse_headers(&raw[header_start_idx..])?;

    let body_start_idx = header_start_idx + header_bytes_len;
    let body = if body_start_idx < raw.len() {
        raw.slice(body_start_idx..)
    } else {
        Bytes::new()
    };

    Ok(SipMessage {
        start_line,
        headers,
        body,
    })
}

fn parse_start_line(input: &str) -> Result<StartLine, SipError> {
    let parts: Vec<&str> = input.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return Err(SipError::ParseError("Invalid start line".to_string()));
    }

    if parts[0].starts_with("SIP/") {
        // Response line: SIP/2.0 200 OK
        let version = Version::from_str(parts[0])?;
        let code = parts[1]
            .parse::<u16>()
            .map_err(|_| SipError::InvalidStatusCode(0))?;
        let status_code = StatusCode::new(code)?;
        let reason_phrase = parts[2].to_string();

        Ok(StartLine::Response(ResponseLine {
            version,
            status_code,
            reason_phrase,
        }))
    } else {
        // Request line: INVITE sip:user@host SIP/2.0
        let method = Method::from_str(parts[0])?;
        let uri = parse_uri(parts[1])?;
        let version = Version::from_str(parts[2])?;

        Ok(StartLine::Request(RequestLine {
            method,
            uri,
            version,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_invite_request() {
        let raw = Bytes::from(
            "INVITE sip:100@pbx.local SIP/2.0\r\n\
             Via: SIP/2.0/UDP 192.168.1.10:5060;branch=z9hG4bK776asdhds\r\n\
             From: \"Alice\" <sip:100@pbx.local>;tag=1928301774\r\n\
             To: \"Bob\" <sip:200@pbx.local>\r\n\
             Call-ID: a84b4c76e66710@192.168.1.10\r\n\
             CSeq: 314159 INVITE\r\n\
             Contact: <sip:100@192.168.1.10:5060>\r\n\
             Content-Type: application/sdp\r\n\
             Content-Length: 12\r\n\
             \r\n\
             v=0\r\nt=0 0\r\n",
        );

        let msg = parse_message(raw).unwrap();
        assert!(msg.is_request());
        if let StartLine::Request(ref req) = msg.start_line {
            assert_eq!(req.method, Method::Invite);
            assert_eq!(req.uri.host, "pbx.local");
            assert_eq!(req.uri.user.as_deref(), Some("100"));
        } else {
            panic!("Expected request");
        }

        assert_eq!(msg.headers.len(), 8);
        assert_eq!(msg.body_str(), "v=0\r\nt=0 0\r\n");
    }

    #[test]
    fn test_parse_200_ok_response() {
        let raw = Bytes::from(
            "SIP/2.0 200 OK\r\n\
             Via: SIP/2.0/UDP 192.168.1.10:5060;branch=z9hG4bK776asdhds\r\n\
             From: \"Alice\" <sip:100@pbx.local>;tag=1928301774\r\n\
             To: \"Bob\" <sip:200@pbx.local>;tag=a6c85cf\r\n\
             Call-ID: a84b4c76e66710@192.168.1.10\r\n\
             CSeq: 314159 INVITE\r\n\
             Content-Length: 0\r\n\
             \r\n",
        );

        let msg = parse_message(raw).unwrap();
        assert!(msg.is_response());
        if let StartLine::Response(resp) = msg.start_line {
            assert_eq!(resp.status_code.code(), 200);
            assert_eq!(resp.reason_phrase, "OK");
        } else {
            panic!("Expected response");
        }
    }
}
