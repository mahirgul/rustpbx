use std::collections::HashMap;
use std::fmt;

/// Supported URI schemes (sip, sips, tel).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scheme {
    Sip,
    Sips,
    Tel,
    Other(String),
}

impl Scheme {
    pub fn as_str(&self) -> &str {
        match self {
            Scheme::Sip => "sip",
            Scheme::Sips => "sips",
            Scheme::Tel => "tel",
            Scheme::Other(s) => s.as_str(),
        }
    }
}

impl std::str::FromStr for Scheme {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sip" => Ok(Scheme::Sip),
            "sips" => Ok(Scheme::Sips),
            "tel" => Ok(Scheme::Tel),
            other => Ok(Scheme::Other(other.to_string())),
        }
    }
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a parsed SIP or SIPS URI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uri {
    pub scheme: Scheme,
    pub user: Option<String>,
    pub password: Option<String>,
    pub host: String,
    pub port: Option<u16>,
    pub params: HashMap<String, Option<String>>,
    pub headers: HashMap<String, String>,
}

impl Uri {
    pub fn new(host: impl Into<String>) -> Self {
        Uri {
            scheme: Scheme::Sip,
            user: None,
            password: None,
            host: host.into(),
            port: None,
            params: HashMap::new(),
            headers: HashMap::new(),
        }
    }

    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
}

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.scheme)?;
        if let Some(user) = &self.user {
            write!(f, "{}", user)?;
            if let Some(pass) = &self.password {
                write!(f, ":{}", pass)?;
            }
            write!(f, "@")?;
        }
        write!(f, "{}", self.host)?;
        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }
        for (k, v) in &self.params {
            write!(f, ";{}", k)?;
            if let Some(val) = v {
                write!(f, "={}", val)?;
            }
        }
        if !self.headers.is_empty() {
            write!(f, "?")?;
            let mut first = true;
            for (k, v) in &self.headers {
                if !first {
                    write!(f, "&")?;
                }
                write!(f, "{}={}", k, v)?;
                first = false;
            }
        }
        Ok(())
    }
}
