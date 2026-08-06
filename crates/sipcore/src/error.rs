use thiserror::Error;

/// Unified error enum for SIP parsing and formatting.
#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum SipError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid SIP method: {0}")]
    InvalidMethod(String),

    #[error("Invalid status code: {0}")]
    InvalidStatusCode(u16),

    #[error("Invalid SIP version: {0}")]
    InvalidVersion(String),

    #[error("Invalid SIP URI: {0}")]
    InvalidUri(String),

    #[error("Header not found: {0}")]
    HeaderNotFound(String),

    #[error("Invalid header format: {0}")]
    InvalidHeader(String),

    #[error("Unexpected end of buffer")]
    UnexpectedEof,
}

pub type SipResult<T> = Result<T, SipError>;
