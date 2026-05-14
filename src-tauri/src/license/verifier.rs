use serde::{Deserialize, Serialize};

/// Stored in license.dat (DPAPI-encrypted JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseData {
    pub version: u32,
    /// SHA-256(uppercase(code) + SALT) — same hash as in codes.rs whitelist
    pub code_hash: String,
    /// Device fingerprint at activation time
    pub device_fingerprint: String,
    /// Unix timestamp of activation
    pub activated_at: u64,
}
