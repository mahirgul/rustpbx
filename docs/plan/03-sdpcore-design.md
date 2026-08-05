# 03 — `sdpcore` Design — SDP Parser & Negotiation

## Purpose

Zero-I/O SDP parsing, generation, and offer/answer negotiation library.

**Dependencies:** `nom`, `bytes` only. **NO dependency on `sipcore`.**

**Input/Output:** Takes `&[u8]` or `Bytes` (raw SDP body), outputs structured `SessionDescription`. The glue between the SIP message body and SDP parsing lives in `sipstack` or `sip-engine`, not in either core crate.

---

## Core Types

```rust
/// sdpcore operates on raw bytes — no SIP awareness needed
pub fn parse_sdp(input: &[u8]) -> Result<SessionDescription, SdpError> { ... }

pub struct SessionDescription {
    pub version: u32,
    pub origin: Origin,
    pub session_name: String,
    pub connection: Option<Connection>,
    pub timing: Vec<Timing>,
    pub media: Vec<MediaDescription>,
    pub attributes: Vec<Attribute>,
}

pub struct MediaDescription {
    pub media_type: MediaType,         // audio, video, application
    pub port: u16,
    pub protocol: TransportProtocol,   // RTP/AVP, RTP/SAVPF, UDP/TLS/RTP/SAVPF
    pub formats: Vec<u8>,              // Payload type numbers
    pub codecs: Vec<Codec>,            // Parsed from a=rtpmap
    pub direction: Direction,          // sendrecv, sendonly, recvonly, inactive
    pub ice_candidates: Vec<IceCandidate>,
    pub dtls_fingerprint: Option<DtlsFingerprint>,
    pub attributes: Vec<Attribute>,
}
```

---

## Glue Code inside `sipstack`

Keeping `sdpcore` independent of `sipcore` prevents circular dependencies. The glue code combining both lives in `sipstack`:

```rust
/// inside sipstack: Extracting SDP from a SIP message
use sipcore::types::SipMessage;
use sdpcore::{parse_sdp, SessionDescription};

pub fn extract_sdp(msg: &SipMessage) -> Option<Result<SessionDescription, SdpError>> {
    let ct = msg.headers().get_typed::<ContentType>().ok()?;
    if ct.media_type() != "application/sdp" {
        return None;
    }
    Some(parse_sdp(msg.body()))
}
```
