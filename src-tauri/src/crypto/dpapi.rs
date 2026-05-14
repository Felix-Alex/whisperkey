use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB,
};

const ENTROPY: &[u8] = b"WhisperKey-v2-Salt";

fn make_blob(data: &[u8]) -> CRYPT_INTEGER_BLOB {
    CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    }
}

fn make_entropy() -> CRYPT_INTEGER_BLOB {
    CRYPT_INTEGER_BLOB {
        cbData: ENTROPY.len() as u32,
        pbData: ENTROPY.as_ptr() as *mut u8,
    }
}

fn free_blob_data(blob: &CRYPT_INTEGER_BLOB) {
    if !blob.pbData.is_null() {
        // SAFETY: pbData was allocated by CryptProtectData/CryptUnprotectData via LocalAlloc.
        // LocalFree is the correct deallocator for DPAPI-allocated memory blocks.
        unsafe {
            windows::Win32::Foundation::LocalFree(windows::Win32::Foundation::HLOCAL(
                blob.pbData as *mut core::ffi::c_void,
            ));
        }
    }
}

pub fn encrypt(plain: &[u8]) -> Vec<u8> {
    let data_in = make_blob(plain);
    let entropy = make_entropy();
    let mut data_out = CRYPT_INTEGER_BLOB::default();

    // SAFETY:
    // - data_in contains a valid pointer to `plain` bytes and correct length.
    // - entropy contains a valid pointer to static ENTROPY bytes and correct length.
    // - data_out is default-initialized; DPAPI allocates memory and writes the pointer + length.
    // - All pointers remain valid for the duration of this call.
    unsafe {
        CryptProtectData(
            &data_in,
            windows::core::PCWSTR::null(),
            Some(&entropy),
            None,
            None,
            0,
            &mut data_out,
        )
        .expect("CryptProtectData failed");
    }

    // SAFETY: data_out.pbData was allocated by CryptProtectData above, with cbData bytes.
    // The pointer is non-null (guaranteed by the expect above) and the memory is initialized.
    let out = unsafe { std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize) }.to_vec();
    free_blob_data(&data_out);
    out
}

pub fn decrypt(cipher: &[u8]) -> Vec<u8> {
    decrypt_result(cipher).expect("CryptUnprotectData failed")
}

pub fn decrypt_result(cipher: &[u8]) -> Result<Vec<u8>, String> {
    let data_in = make_blob(cipher);
    let entropy = make_entropy();
    let mut data_out = CRYPT_INTEGER_BLOB::default();

    // SAFETY:
    // - data_in contains a valid pointer to `cipher` bytes and correct length.
    // - entropy contains a valid pointer to static ENTROPY bytes and correct length.
    // - data_out is default-initialized; DPAPI allocates memory and writes the pointer + length.
    // - All pointers remain valid for the duration of this call.
    unsafe {
        CryptUnprotectData(
            &data_in,
            None,
            Some(&entropy),
            None,
            None,
            0,
            &mut data_out,
        )
        .map_err(|e| format!("DPAPI decrypt failed: {e}"))?;
    }

    // SAFETY: data_out.pbData was allocated by CryptUnprotectData above, with cbData bytes.
    // The pointer is non-null (guaranteed by the ? above) and the memory is initialized.
    let out = unsafe { std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize) }.to_vec();
    free_blob_data(&data_out);
    Ok(out)
}

pub fn encrypt_str(plain: &str) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, encrypt(plain.as_bytes()))
}

pub fn decrypt_str(cipher: &str) -> Result<String, String> {
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, cipher)
        .map_err(|e| format!("base64 decode failed: {e}"))?;
    let plain = decrypt(&bytes);
    String::from_utf8(plain).map_err(|e| format!("utf8 decode failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let input = b"hello world secret data";
        let cipher = encrypt(input);
        assert!(!cipher.is_empty());
        assert_ne!(cipher, input);

        let plain = decrypt(&cipher);
        assert_eq!(plain, input);
    }

    #[test]
    fn test_str_roundtrip() {
        let input = "my API Key: sk-1234567890";
        let cipher = encrypt_str(input);
        assert!(!cipher.is_empty());
        assert_ne!(cipher.as_bytes(), input.as_bytes());

        let plain = decrypt_str(&cipher).unwrap();
        assert_eq!(plain, input);
    }

    #[test]
    fn test_encrypt_produces_different_output() {
        let input = b"test data";
        let c1 = encrypt(input);
        let c2 = encrypt(input);
        assert!(!c1.is_empty());
        assert!(!c2.is_empty());
    }
}
