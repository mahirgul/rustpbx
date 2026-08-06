use crate::error::SdpError;
use crate::types::{Codec, Connection, MediaDescription, MediaDirection, SessionDescription};

/// Parse raw SDP string into a SessionDescription.
pub fn parse_sdp(input: &str) -> Result<SessionDescription, SdpError> {
    let mut sdp = SessionDescription {
        version: 0,
        origin_username: "-".to_string(),
        session_id: 0,
        session_version: 0,
        session_name: "RustPBX".to_string(),
        connection: None,
        media: Vec::new(),
    };

    let mut current_media: Option<MediaDescription> = None;

    for line in input.lines() {
        let line = line.trim();
        if line.len() < 2 || !line.as_bytes()[1] == b'=' {
            continue;
        }

        let key = &line[0..1];
        let val = &line[2..];

        match key {
            "v" => {
                sdp.version = val
                    .parse::<u32>()
                    .map_err(|_| SdpError::ParseError("Invalid version".to_string()))?;
            }
            "c" => {
                let parts: Vec<&str> = val.split_whitespace().collect();
                if parts.len() >= 3 {
                    let conn = Connection {
                        net_type: parts[0].to_string(),
                        addr_type: parts[1].to_string(),
                        address: parts[2].to_string(),
                    };
                    if let Some(m) = &mut current_media {
                        // Connection per media
                        let _ = m;
                    } else {
                        sdp.connection = Some(conn);
                    }
                }
            }
            "m" => {
                if let Some(m) = current_media.take() {
                    sdp.media.push(m);
                }
                let parts: Vec<&str> = val.split_whitespace().collect();
                if parts.len() >= 3 {
                    let port = parts[1]
                        .parse::<u16>()
                        .map_err(|_| SdpError::ParseError("Invalid media port".to_string()))?;

                    let mut codecs = Vec::new();
                    for pt_str in &parts[3..] {
                        if let Ok(pt) = pt_str.parse::<u8>() {
                            let name = match pt {
                                0 => "PCMU",
                                8 => "PCMA",
                                9 => "G722",
                                111 => "opus",
                                _ => "UNKNOWN",
                            };
                            let clock = match pt {
                                111 => 48000,
                                _ => 8000,
                            };
                            codecs.push(Codec {
                                payload_type: pt,
                                name: name.to_string(),
                                clock_rate: clock,
                                channels: None,
                            });
                        }
                    }

                    current_media = Some(MediaDescription {
                        media_type: parts[0].to_string(),
                        port,
                        protocol: parts[2].to_string(),
                        codecs,
                        direction: MediaDirection::SendRecv,
                    });
                }
            }
            "a" => {
                if let Some(m) = &mut current_media {
                    if val == "sendrecv" {
                        m.direction = MediaDirection::SendRecv;
                    } else if val == "sendonly" {
                        m.direction = MediaDirection::SendOnly;
                    } else if val == "recvonly" {
                        m.direction = MediaDirection::RecvOnly;
                    } else if val == "inactive" {
                        m.direction = MediaDirection::Inactive;
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(m) = current_media {
        sdp.media.push(m);
    }

    Ok(sdp)
}
