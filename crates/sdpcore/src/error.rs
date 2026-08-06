use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum SdpError {
    #[error("SDP parse error: {0}")]
    ParseError(String),

    #[error("Unsupported codec: {0}")]
    UnsupportedCodec(String),

    #[error("Offer/Answer negotiation failed: {0}")]
    NegotiationFailed(String),
}

pub type SdpResult<T> = Result<T, SdpError>;
