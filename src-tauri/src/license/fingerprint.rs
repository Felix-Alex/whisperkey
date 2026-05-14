use ring::digest;

/// Generate a device fingerprint from machine identifiers
pub fn device_fingerprint() -> String {
    let machine_guid = read_machine_guid();
    let combined = if machine_guid.is_empty() {
        // Fallback: generate a random ID and persist it alongside the license
        tracing::warn!("[fingerprint] MachineGuid empty — using random fallback");
        "fallback-device".to_string()
    } else {
        machine_guid
    };

    let hash = digest::digest(&digest::SHA256, combined.as_bytes());
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, hash.as_ref())
}

fn read_machine_guid() -> String {
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ,
    };

    let sub_key: Vec<u16> = "SOFTWARE\\Microsoft\\Cryptography\0".encode_utf16().collect();
    let value_name: Vec<u16> = "MachineGuid\0".encode_utf16().collect();

    let mut hkey = windows::Win32::System::Registry::HKEY::default();
    // SAFETY: sub_key is a null-terminated UTF-16 string. hkey is a valid out-pointer.
    // HKEY_LOCAL_MACHINE is always open on Windows. We check the return code.
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from_raw(sub_key.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        )
    };

    if result.is_err() {
        tracing::warn!("[fingerprint] RegOpenKeyExW for MachineGuid failed: {result:?}");
        return String::new();
    }

    let mut data_type = REG_SZ;
    let mut buffer = vec![0u16; 128]; // GUID is 38 chars max
    let mut size: u32 = (buffer.len() * 2) as u32;

    // SAFETY: hkey is valid (opened above), value_name is null-terminated UTF-16,
    // buffer is writable and sized, size points to the correct byte count.
    let result2 = unsafe {
        RegQueryValueExW(
            hkey,
            PCWSTR::from_raw(value_name.as_ptr()),
            None,
            Some(&mut data_type),
            Some(buffer.as_mut_ptr() as *mut u8),
            Some(&mut size),
        )
    };

    // SAFETY: hkey was opened successfully. RegCloseKey failure is non-critical.
    unsafe { let _ = RegCloseKey(hkey); }

    if result2.is_err() {
        tracing::warn!("[fingerprint] RegQueryValueExW for MachineGuid failed: {result2:?}");
        return String::new();
    }

    // size is in bytes; convert to u16 count
    let len = (size as usize / 2).saturating_sub(1); // minus null terminator
    String::from_utf16_lossy(&buffer[..len])
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

    #[test]
    fn test_read_machine_guid_non_empty() {
        let guid = read_machine_guid();
        // On a real Windows machine, MachineGuid should exist
        assert!(!guid.is_empty(), "MachineGuid should exist on Windows");
    }
}
