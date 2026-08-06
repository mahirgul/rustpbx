/// Generates and verifies RFC 2617 SIP Digest Authentication response hashes.
pub struct DigestAuth;

impl DigestAuth {
    /// Compute HA1 = MD5(username:realm:password)
    pub fn compute_ha1(username: &str, realm: &str, password: &str) -> String {
        let digest = md5::compute(format!("{}:{}:{}", username, realm, password).as_bytes());
        format!("{:x}", digest)
    }

    /// Compute HA2 = MD5(method:digest_uri)
    pub fn compute_ha2(method: &str, uri: &str) -> String {
        let digest = md5::compute(format!("{}:{}", method, uri).as_bytes());
        format!("{:x}", digest)
    }

    /// Compute Response = MD5(HA1:nonce:HA2)
    pub fn compute_response(ha1: &str, nonce: &str, ha2: &str) -> String {
        let digest = md5::compute(format!("{}:{}:{}", ha1, nonce, ha2).as_bytes());
        format!("{:x}", digest)
    }

    /// Verify an incoming Digest authorization header response value
    pub fn verify(
        username: &str,
        realm: &str,
        password: &str,
        nonce: &str,
        method: &str,
        uri: &str,
        expected_response: &str,
    ) -> bool {
        let ha1 = Self::compute_ha1(username, realm, password);
        let ha2 = Self::compute_ha2(method, uri);
        let computed_response = Self::compute_response(&ha1, nonce, &ha2);

        computed_response.eq_ignore_ascii_case(expected_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_auth_rfc2617_example() {
        // MD5("MAMON:biloxi.com:gidget") verification
        let ha1 = DigestAuth::compute_ha1("MAMON", "biloxi.com", "gidget");
        assert_eq!(ha1, "405326a824dbce84c1d9555c0fe951ca");
    }
}
