use crate::types::status_code::StatusCode;
use crate::types::version::Version;
use std::fmt;

/// Represents a SIP Response start line (Status-Line).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseLine {
    pub version: Version,
    pub status_code: StatusCode,
    pub reason_phrase: String,
}

impl ResponseLine {
    pub fn new(status_code: StatusCode) -> Self {
        ResponseLine {
            version: Version::default(),
            reason_phrase: status_code.default_reason_phrase().to_string(),
            status_code,
        }
    }
}

impl fmt::Display for ResponseLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.version,
            self.status_code.code(),
            self.reason_phrase
        )
    }
}
