use crate::types::headers::Headers;
use crate::types::request::RequestLine;
use crate::types::response::ResponseLine;
use bytes::Bytes;
use std::fmt;

/// Represents the type of SIP start line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartLine {
    Request(RequestLine),
    Response(ResponseLine),
}

impl fmt::Display for StartLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StartLine::Request(req) => write!(f, "{}", req),
            StartLine::Response(resp) => write!(f, "{}", resp),
        }
    }
}

/// Core parsed SIP message holding start line, headers, and zero-copy body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SipMessage {
    pub start_line: StartLine,
    pub headers: Headers,
    pub body: Bytes,
}

impl SipMessage {
    pub fn new_request(request_line: RequestLine, headers: Headers, body: Bytes) -> Self {
        SipMessage {
            start_line: StartLine::Request(request_line),
            headers,
            body,
        }
    }

    pub fn new_response(response_line: ResponseLine, headers: Headers, body: Bytes) -> Self {
        SipMessage {
            start_line: StartLine::Response(response_line),
            headers,
            body,
        }
    }

    pub fn is_request(&self) -> bool {
        matches!(self.start_line, StartLine::Request(_))
    }

    pub fn is_response(&self) -> bool {
        matches!(self.start_line, StartLine::Response(_))
    }

    pub fn body_str(&self) -> &str {
        std::str::from_utf8(&self.body).unwrap_or("")
    }
}

impl fmt::Display for SipMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\r\n{}", self.start_line, self.headers)?;
        write!(f, "\r\n")?;
        if !self.body.is_empty() {
            write!(f, "{}", self.body_str())?;
        }
        Ok(())
    }
}
