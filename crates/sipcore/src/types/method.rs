use crate::error::SipError;
use std::fmt;
use std::str::FromStr;

/// Standard SIP methods defined in RFC 3261 and extensions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Method {
    Invite,
    Ack,
    Bye,
    Cancel,
    Options,
    Register,
    Subscribe,
    Notify,
    Refer,
    Update,
    Prack,
    Info,
    Message,
    Publish,
    Other(String),
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Method::Invite => "INVITE",
            Method::Ack => "ACK",
            Method::Bye => "BYE",
            Method::Cancel => "CANCEL",
            Method::Options => "OPTIONS",
            Method::Register => "REGISTER",
            Method::Subscribe => "SUBSCRIBE",
            Method::Notify => "NOTIFY",
            Method::Refer => "REFER",
            Method::Update => "UPDATE",
            Method::Prack => "PRACK",
            Method::Info => "INFO",
            Method::Message => "MESSAGE",
            Method::Publish => "PUBLISH",
            Method::Other(s) => s.as_str(),
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Method {
    type Err = SipError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "INVITE" => Ok(Method::Invite),
            "ACK" => Ok(Method::Ack),
            "BYE" => Ok(Method::Bye),
            "CANCEL" => Ok(Method::Cancel),
            "OPTIONS" => Ok(Method::Options),
            "REGISTER" => Ok(Method::Register),
            "SUBSCRIBE" => Ok(Method::Subscribe),
            "NOTIFY" => Ok(Method::Notify),
            "REFER" => Ok(Method::Refer),
            "UPDATE" => Ok(Method::Update),
            "PRACK" => Ok(Method::Prack),
            "INFO" => Ok(Method::Info),
            "MESSAGE" => Ok(Method::Message),
            "PUBLISH" => Ok(Method::Publish),
            other => {
                if other.is_empty() {
                    Err(SipError::InvalidMethod("Empty method string".to_string()))
                } else {
                    Ok(Method::Other(other.to_string()))
                }
            }
        }
    }
}
