/// IT-07: DPAPI encrypt/decrypt roundtrip — plaintext survives encrypt→decrypt
use whisperkey_lib::crypto::dpapi;

#[test]
fn it07_dpapi_roundtrip_bytes() {
    let input = b"Hello WhisperKey integration test data!";
    let encrypted = dpapi::encrypt(input);
    assert!(!encrypted.is_empty());
    assert_ne!(&encrypted, input);

    let decrypted = dpapi::decrypt(&encrypted);
    assert_eq!(decrypted, input);
}

#[test]
fn it07_dpapi_roundtrip_str() {
    let input = "sk-test-api-key-integration-2026";
    let encrypted = dpapi::encrypt_str(input);
    assert!(!encrypted.is_empty());
    assert_ne!(encrypted.as_bytes(), input.as_bytes());

    let decrypted = dpapi::decrypt_str(&encrypted).unwrap();
    assert_eq!(decrypted, input);
}

#[test]
fn it07_dpapi_different_calls_different_output() {
    let data = b"same data";
    let c1 = dpapi::encrypt(data);
    let c2 = dpapi::encrypt(data);
    assert_ne!(c1, c2);
}

#[test]
fn it07_dpapi_empty_input() {
    let encrypted = dpapi::encrypt(b"");
    assert!(!encrypted.is_empty());
    let decrypted = dpapi::decrypt(&encrypted);
    assert!(decrypted.is_empty());
}

#[test]
fn it07_dpapi_decrypt_bad_data() {
    let result = dpapi::decrypt_result(b"definitely not valid dpapi ciphertext");
    assert!(result.is_err());
}

#[test]
fn it07_dpapi_decrypt_str_bad_base64() {
    let result = dpapi::decrypt_str("not-valid-base64!!!");
    assert!(result.is_err());
}
