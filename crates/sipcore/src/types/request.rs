use crate::types::method::Method;
use crate::types::uri::Uri;
use crate::types::version::Version;
use std::fmt;

/// Represents a SIP Request start line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestLine {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
}

impl fmt::Display for RequestLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.method, self.uri, self.version)
    }
}
