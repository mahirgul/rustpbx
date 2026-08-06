use crate::error::SipError;
use crate::types::header::Header;
use crate::types::header_name::HeaderName;
use crate::types::headers::Headers;
use bytes::Bytes;
use std::str::FromStr;

/// Parse header lines from raw byte input supporting RFC 3261 header folding.
pub fn parse_headers(input: &[u8]) -> Result<(Headers, usize), SipError> {
    let text = std::str::from_utf8(input)
        .map_err(|_| SipError::ParseError("Invalid UTF-8 in headers".to_string()))?;

    let mut headers = Headers::new();
    let mut current_header_name: Option<HeaderName> = None;
    let mut current_value = String::new();
    let mut bytes_consumed = 0;

    for line in text.lines() {
        let line_len = line.len() + 2; // +2 for \r\n
        bytes_consumed += line_len;

        if line.is_empty() {
            // End of headers (\r\n\r\n)
            if let Some(name) = current_header_name.take() {
                headers.push(Header::new(
                    name,
                    Bytes::from(current_value.trim().to_string()),
                ));
            }
            break;
        }

        if line.starts_with(' ') || line.starts_with('\t') {
            // Line folding (continuation line)
            current_value.push(' ');
            current_value.push_str(line.trim());
        } else {
            // New header line
            if let Some(name) = current_header_name.take() {
                headers.push(Header::new(
                    name,
                    Bytes::from(current_value.trim().to_string()),
                ));
                current_value.clear();
            }

            if let Some((name_part, value_part)) = line.split_once(':') {
                let name = HeaderName::from_str(name_part.trim())?;
                current_header_name = Some(name);
                current_value.push_str(value_part.trim());
            } else {
                return Err(SipError::InvalidHeader(line.to_string()));
            }
        }
    }

    if let Some(name) = current_header_name {
        headers.push(Header::new(
            name,
            Bytes::from(current_value.trim().to_string()),
        ));
    }

    Ok((headers, bytes_consumed))
}
