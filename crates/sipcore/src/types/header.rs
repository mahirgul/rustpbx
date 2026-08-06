use crate::types::header_name::HeaderName;
use bytes::Bytes;
use std::fmt;

/// Represents a single SIP header line in zero-copy `Bytes` form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub name: HeaderName,
    pub raw_value: Bytes,
}

impl Header {
    pub fn new(name: HeaderName, value: impl Into<Bytes>) -> Self {
        Header {
            name,
            raw_value: value.into(),
        }
    }

    pub fn value_str(&self) -> &str {
        std::str::from_utf8(&self.raw_value).unwrap_or("")
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value_str())
    }
}
