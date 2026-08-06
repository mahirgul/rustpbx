use crate::error::{SdpError, SdpResult};
use crate::types::{Codec, MediaDescription, MediaDirection, SessionDescription};

/// Performs RFC 3264 Offer/Answer SDP negotiation to produce a matching SDP answer.
pub fn negotiate_answer(
    offer: &SessionDescription,
    supported_codecs: &[&str],
    local_media_ip: &str,
    local_media_port: u16,
) -> SdpResult<SessionDescription> {
    let mut answer = SessionDescription {
        version: 0,
        origin_username: "RustPBX".to_string(),
        session_id: offer.session_id,
        session_version: offer.session_version + 1,
        session_name: "RustPBX Bridge".to_string(),
        connection: Some(crate::types::Connection {
            net_type: "IN".to_string(),
            addr_type: "IP4".to_string(),
            address: local_media_ip.to_string(),
        }),
        media: Vec::new(),
    };

    for offer_media in &offer.media {
        let mut matched_codecs = Vec::new();

        for codec in &offer_media.codecs {
            if supported_codecs
                .iter()
                .any(|&sc| sc.eq_ignore_ascii_case(&codec.name))
            {
                matched_codecs.push(codec.clone());
            }
        }

        if matched_codecs.is_empty() {
            // Default fallback if unknown payload type
            matched_codecs.push(Codec {
                payload_type: 0,
                name: "PCMU".to_string(),
                clock_rate: 8000,
                channels: None,
            });
        }

        let direction = match offer_media.direction {
            MediaDirection::SendRecv => MediaDirection::SendRecv,
            MediaDirection::SendOnly => MediaDirection::RecvOnly,
            MediaDirection::RecvOnly => MediaDirection::SendOnly,
            MediaDirection::Inactive => MediaDirection::Inactive,
        };

        answer.media.push(MediaDescription {
            media_type: offer_media.media_type.clone(),
            port: local_media_port,
            protocol: offer_media.protocol.clone(),
            codecs: matched_codecs,
            direction,
        });
    }

    if answer.media.is_empty() {
        return Err(SdpError::NegotiationFailed(
            "No compatible media streams found".to_string(),
        ));
    }

    Ok(answer)
}
