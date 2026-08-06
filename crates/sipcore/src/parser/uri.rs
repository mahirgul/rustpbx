use crate::error::SipError;
use crate::types::uri::{Scheme, Uri};
use nom::{
    bytes::complete::{take_until, take_while1},
    character::complete::char,
    combinator::opt,
    sequence::tuple,
    IResult,
};
use std::str::FromStr;

/// Parse a SIP URI string into a Uri struct.
pub fn parse_uri(input: &str) -> Result<Uri, SipError> {
    let (_, (scheme_str, _, user_part, host_part, port_part)) =
        parse_uri_components(input).map_err(|e| SipError::InvalidUri(e.to_string()))?;

    let scheme = Scheme::from_str(scheme_str).unwrap_or(Scheme::Sip);

    let mut uri = Uri::new(host_part);
    uri.scheme = scheme;

    if let Some(user) = user_part {
        if let Some((u, p)) = user.split_once(':') {
            uri.user = Some(u.to_string());
            uri.password = Some(p.to_string());
        } else {
            uri.user = Some(user.to_string());
        }
    }

    if let Some(port_str) = port_part {
        if let Ok(p) = port_str.parse::<u16>() {
            uri.port = Some(p);
        }
    }

    Ok(uri)
}

type ParsedUriComponents<'a> = (&'a str, &'a str, Option<&'a str>, &'a str, Option<&'a str>);

fn parse_uri_components(input: &str) -> IResult<&str, ParsedUriComponents<'_>> {
    let (input, scheme) = take_until(":")(input)?;
    let (input, _) = char(':')(input)?;

    let (input, user_part) = opt(tuple((take_until("@"), char('@'))))(input)?;
    let user = user_part.map(|(u, _)| u);

    let (input, host) = take_while1(|c: char| c != ':' && c != ';' && c != '?' && c != ' ')(input)?;

    let (input, port_part) = opt(tuple((
        char(':'),
        take_while1(|c: char| c.is_ascii_digit()),
    )))(input)?;
    let port = port_part.map(|(_, p)| p);

    Ok((input, (scheme, ":", user, host, port)))
}
