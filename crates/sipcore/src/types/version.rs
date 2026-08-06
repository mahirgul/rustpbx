use crate::error::SipError;
use std::fmt;
use std::str::FromStr;

/// Represents SIP protocol version (default "SIP/2.0").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
}

impl Default for Version {
    fn default() -> Self {
        Version { major: 2, minor: 0 }
    }
}

impl Version {
    pub const V2_0: Version = Version { major: 2, minor: 0 };
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SIP/{}.{}", self.major, self.minor)
    }
}

impl FromStr for Version {
    type Err = SipError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 || parts[0] != "SIP" {
            return Err(SipError::InvalidVersion(s.to_string()));
        }
        let numbers: Vec<&str> = parts[1].split('.').collect();
        if numbers.len() != 2 {
            return Err(SipError::InvalidVersion(s.to_string()));
        }
        let major = numbers[0]
            .parse::<u8>()
            .map_err(|_| SipError::InvalidVersion(s.to_string()))?;
        let minor = numbers[1]
            .parse::<u8>()
            .map_err(|_| SipError::InvalidVersion(s.to_string()))?;

        Ok(Version { major, minor })
    }
}
