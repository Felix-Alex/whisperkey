use ring::digest;

/// Generate a device fingerprint from machine identifiers
pub fn device_fingerprint() -> String {
    let machine_guid = read_machine_guid();
    let board_serial = read_board_serial();

    let combined = format!("{machine_guid}:{board_serial}");
    let hash = digest::digest(&digest::SHA256, combined.as_bytes());
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, hash.as_ref())
}

fn read_machine_guid() -> String {
    // Read HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid
    // Stub: return a placeholder
    String::from("placeholder-guid")
}

fn read_board_serial() -> String {
    // WMI Win32_BaseBoard.SerialNumber (first 8 chars)
    String::from("00000000")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_consistent() {
        let f1 = device_fingerprint();
        let f2 = device_fingerprint();
        assert_eq!(f1, f2);
    }

    #[test]
    fn test_fingerprint_non_empty() {
        let fp = device_fingerprint();
        assert!(!fp.is_empty());
    }
}
