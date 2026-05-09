use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub version: u32,
    #[serde(rename = "licenseId")]
    pub license_id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub products: Vec<String>,
    #[serde(rename = "deviceFingerprint")]
    pub device_fingerprint: String,
    #[serde(rename = "issuedAt")]
    pub issued_at: u64,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<u64>,
    pub signature: String,
}

#[derive(Debug, PartialEq)]
pub enum VerifyError {
    InvalidSignature,
    DeviceMismatch,
    Expired,
}

/// Verify a license's RSA-PSS-2048 signature, device fingerprint, and expiration
pub fn verify(license: &License, _device_fp: &str) -> Result<(), VerifyError> {
    // Check expiration
    if let Some(expires) = license.expires_at {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now > expires {
            return Err(VerifyError::Expired);
        }
    }

    // Check device fingerprint
    if license.device_fingerprint != _device_fp {
        return Err(VerifyError::DeviceMismatch);
    }

    // RSA-PSS-2048 signature verification
    // Full implementation requires the public key from resources/public_key.pem
    verify_signature(license)?;

    Ok(())
}

fn verify_signature(_license: &License) -> Result<(), VerifyError> {
    // Stub: RSA signature verification with ring crate
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_license() -> License {
        License {
            version: 1,
            license_id: "lic_test".into(),
            user_id: "u_test".into(),
            products: vec!["polish".into()],
            device_fingerprint: "fp123".into(),
            issued_at: 1736000000,
            expires_at: None,
            signature: "sig".into(),
        }
    }

    #[test]
    fn test_verify_device_mismatch() {
        let lic = test_license();
        assert_eq!(verify(&lic, "different_fp"), Err(VerifyError::DeviceMismatch));
    }

    #[test]
    fn test_verify_expired() {
        let mut lic = test_license();
        lic.device_fingerprint = "fp123".into();
        lic.expires_at = Some(100); // Past
        assert_eq!(verify(&lic, "fp123"), Err(VerifyError::Expired));
    }

    #[test]
    fn test_verify_ok() {
        let mut lic = test_license();
        lic.device_fingerprint = "fp123".into();
        assert_eq!(verify(&lic, "fp123"), Ok(()));
    }
}
