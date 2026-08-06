use crate::error::SipError;
use std::fmt;
use std::str::FromStr;

/// SIP Header Name enum supporting full names, compact names (RFC 3261 §7.3.3), and custom headers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HeaderName {
    From,
    To,
    Via,
    CallId,
    CSeq,
    Contact,
    ContentType,
    ContentLength,
    MaxForwards,
    RecordRoute,
    Route,
    UserAgent,
    Server,
    Allow,
    Accept,
    Supported,
    Require,
    ProxyRequire,
    Authorization,
    ProxyAuthorization,
    WwwAuthenticate,
    ProxyAuthenticate,
    Expires,
    MinExpires,
    Event,
    SubscriptionState,
    ReferTo,
    ReferredBy,
    Replaces,
    RAck,
    RSeq,
    PAssertedIdentity,
    PreferredIdentity,
    Privacy,
    Other(String),
}

impl HeaderName {
    pub fn as_str(&self) -> &str {
        match self {
            HeaderName::From => "From",
            HeaderName::To => "To",
            HeaderName::Via => "Via",
            HeaderName::CallId => "Call-ID",
            HeaderName::CSeq => "CSeq",
            HeaderName::Contact => "Contact",
            HeaderName::ContentType => "Content-Type",
            HeaderName::ContentLength => "Content-Length",
            HeaderName::MaxForwards => "Max-Forwards",
            HeaderName::RecordRoute => "Record-Route",
            HeaderName::Route => "Route",
            HeaderName::UserAgent => "User-Agent",
            HeaderName::Server => "Server",
            HeaderName::Allow => "Allow",
            HeaderName::Accept => "Accept",
            HeaderName::Supported => "Supported",
            HeaderName::Require => "Require",
            HeaderName::ProxyRequire => "Proxy-Require",
            HeaderName::Authorization => "Authorization",
            HeaderName::ProxyAuthorization => "Proxy-Authorization",
            HeaderName::WwwAuthenticate => "WWW-Authenticate",
            HeaderName::ProxyAuthenticate => "Proxy-Authenticate",
            HeaderName::Expires => "Expires",
            HeaderName::MinExpires => "Min-Expires",
            HeaderName::Event => "Event",
            HeaderName::SubscriptionState => "Subscription-State",
            HeaderName::ReferTo => "Refer-To",
            HeaderName::ReferredBy => "Referred-By",
            HeaderName::Replaces => "Replaces",
            HeaderName::RAck => "RAck",
            HeaderName::RSeq => "RSeq",
            HeaderName::PAssertedIdentity => "P-Asserted-Identity",
            HeaderName::PreferredIdentity => "P-Preferred-Identity",
            HeaderName::Privacy => "Privacy",
            HeaderName::Other(s) => s.as_str(),
        }
    }
}

impl fmt::Display for HeaderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for HeaderName {
    type Err = SipError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(SipError::InvalidHeader("Empty header name".to_string()));
        }

        match trimmed.to_lowercase().as_str() {
            "from" | "f" => Ok(HeaderName::From),
            "to" | "t" => Ok(HeaderName::To),
            "via" | "v" => Ok(HeaderName::Via),
            "call-id" | "i" => Ok(HeaderName::CallId),
            "cseq" => Ok(HeaderName::CSeq),
            "contact" | "m" => Ok(HeaderName::Contact),
            "content-type" | "c" => Ok(HeaderName::ContentType),
            "content-length" | "l" => Ok(HeaderName::ContentLength),
            "max-forwards" => Ok(HeaderName::MaxForwards),
            "record-route" => Ok(HeaderName::RecordRoute),
            "route" => Ok(HeaderName::Route),
            "user-agent" => Ok(HeaderName::UserAgent),
            "server" => Ok(HeaderName::Server),
            "allow" => Ok(HeaderName::Allow),
            "accept" => Ok(HeaderName::Accept),
            "supported" | "k" => Ok(HeaderName::Supported),
            "require" => Ok(HeaderName::Require),
            "proxy-require" => Ok(HeaderName::ProxyRequire),
            "authorization" => Ok(HeaderName::Authorization),
            "proxy-authorization" => Ok(HeaderName::ProxyAuthorization),
            "www-authenticate" => Ok(HeaderName::WwwAuthenticate),
            "proxy-authenticate" => Ok(HeaderName::ProxyAuthenticate),
            "expires" => Ok(HeaderName::Expires),
            "min-expires" => Ok(HeaderName::MinExpires),
            "event" | "o" => Ok(HeaderName::Event),
            "subscription-state" => Ok(HeaderName::SubscriptionState),
            "refer-to" | "r" => Ok(HeaderName::ReferTo),
            "referred-by" => Ok(HeaderName::ReferredBy),
            "replaces" => Ok(HeaderName::Replaces),
            "rack" => Ok(HeaderName::RAck),
            "rseq" => Ok(HeaderName::RSeq),
            "p-asserted-identity" => Ok(HeaderName::PAssertedIdentity),
            "p-preferred-identity" => Ok(HeaderName::PreferredIdentity),
            "privacy" => Ok(HeaderName::Privacy),
            other => Ok(HeaderName::Other(other.to_string())),
        }
    }
}
