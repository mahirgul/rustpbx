use crate::error::SipError;
use crate::types::header::Header;
use crate::types::header_name::HeaderName;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::fmt;

/// Indexed collection of SIP headers preserving order while providing O(1) lookup.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Headers {
    entries: Vec<Header>,
    index: HashMap<HeaderName, SmallVec<[usize; 2]>>,
}

impl Headers {
    pub fn new() -> Self {
        Headers {
            entries: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn push(&mut self, header: Header) {
        let idx = self.entries.len();
        self.index.entry(header.name.clone()).or_default().push(idx);
        self.entries.push(header);
    }

    pub fn get(&self, name: &HeaderName) -> Option<&Header> {
        let indices = self.index.get(name)?;
        let first_idx = *indices.first()?;
        self.entries.get(first_idx)
    }

    pub fn get_all(&self, name: &HeaderName) -> Vec<&Header> {
        match self.index.get(name) {
            Some(indices) => indices
                .iter()
                .filter_map(|&i| self.entries.get(i))
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn get_value_str(&self, name: &HeaderName) -> Result<&str, SipError> {
        self.get(name)
            .map(|h| h.value_str())
            .ok_or_else(|| SipError::HeaderNotFound(name.to_string()))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Header> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl fmt::Display for Headers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for header in &self.entries {
            write!(f, "{}\r\n", header)?;
        }
        Ok(())
    }
}
