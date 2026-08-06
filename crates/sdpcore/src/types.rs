use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaDirection {
    SendRecv,
    SendOnly,
    RecvOnly,
    Inactive,
}

impl MediaDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaDirection::SendRecv => "sendrecv",
            MediaDirection::SendOnly => "sendonly",
            MediaDirection::RecvOnly => "recvonly",
            MediaDirection::Inactive => "inactive",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    pub net_type: String,  // IN
    pub addr_type: String, // IP4 or IP6
    pub address: String,   // IP address
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Codec {
    pub payload_type: u8,
    pub name: String,
    pub clock_rate: u32,
    pub channels: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaDescription {
    pub media_type: String, // audio, video
    pub port: u16,
    pub protocol: String, // RTP/AVP, RTP/SAVPF
    pub codecs: Vec<Codec>,
    pub direction: MediaDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDescription {
    pub version: u32,
    pub origin_username: String,
    pub session_id: u64,
    pub session_version: u64,
    pub session_name: String,
    pub connection: Option<Connection>,
    pub media: Vec<MediaDescription>,
}

impl fmt::Display for SessionDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v={}\r\n", self.version)?;
        write!(
            f,
            "o={} {} {} IN IP4 127.0.0.1\r\n",
            self.origin_username, self.session_id, self.session_version
        )?;
        write!(f, "s={}\r\n", self.session_name)?;
        if let Some(conn) = &self.connection {
            write!(
                f,
                "c={} {} {}\r\n",
                conn.net_type, conn.addr_type, conn.address
            )?;
        }
        write!(f, "t=0 0\r\n")?;

        for m in &self.media {
            write!(f, "m={} {} {} ", m.media_type, m.port, m.protocol)?;
            for (i, c) in m.codecs.iter().enumerate() {
                if i > 0 {
                    write!(f, " ")?;
                }
                write!(f, "{}", c.payload_type)?;
            }
            write!(f, "\r\na={}\r\n", m.direction.as_str())?;
            for c in &m.codecs {
                write!(
                    f,
                    "a=rtpmap:{} {}/{}\r\n",
                    c.payload_type, c.name, c.clock_rate
                )?;
            }
        }
        Ok(())
    }
}
