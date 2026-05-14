use crate::config::persist::ConfigStore;
use crate::crypto::dpapi;
use crate::error::{AppError, AppResult};

/// Encrypt and store the global LLM API key.
pub fn set_llm_key(store: &ConfigStore, key: &str) -> AppResult<()> {
    let encrypted = dpapi::encrypt_str(key);
    let len = key.len();
    store.update(|c| {
        c.llm.api_key = encrypted;
        c.llm.api_key_len = len;
    })
}

/// Decrypt and return the global LLM API key.
pub fn get_llm_key(store: &ConfigStore) -> AppResult<String> {
    let cfg = store.read()?;
    if cfg.llm.api_key.is_empty() {
        return Ok(String::new());
    }
    dpapi::decrypt_str(&cfg.llm.api_key).map_err(|e| {
        tracing::error!("failed to decrypt LLM api key: {e}");
        AppError::Internal
    })
}

/// Encrypt and store the global ASR API key.
pub fn set_asr_key(store: &ConfigStore, key: &str) -> AppResult<()> {
    let encrypted = dpapi::encrypt_str(key);
    let len = key.len();
    store.update(|c| {
        c.asr.api_key = encrypted;
        c.asr.api_key_len = len;
    })
}

/// Decrypt and return the global ASR API key.
pub fn get_asr_key(store: &ConfigStore) -> AppResult<String> {
    let cfg = store.read()?;
    if cfg.asr.api_key.is_empty() {
        return Ok(String::new());
    }
    dpapi::decrypt_str(&cfg.asr.api_key).map_err(|e| {
        tracing::error!("failed to decrypt ASR api key: {e}");
        AppError::Internal
    })
}

/// Encrypt and store the global LLM API secret (second key field for ernie, etc.).
pub fn set_llm_secret(store: &ConfigStore, secret: &str) -> AppResult<()> {
    let encrypted = dpapi::encrypt_str(secret);
    let len = secret.len();
    store.update(|c| {
        c.llm.api_secret = encrypted;
        c.llm.api_secret_len = len;
    })
}

/// Decrypt and return the global LLM API secret.
pub fn get_llm_secret(store: &ConfigStore) -> AppResult<String> {
    let cfg = store.read()?;
    if cfg.llm.api_secret.is_empty() {
        return Ok(String::new());
    }
    dpapi::decrypt_str(&cfg.llm.api_secret).map_err(|e| {
        tracing::error!("failed to decrypt LLM api secret: {e}");
        AppError::Internal
    })
}

/// Encrypt and store the global ASR API secret (second key field for volcengine AccessKey, xfyun Secret, etc.).
pub fn set_asr_secret(store: &ConfigStore, secret: &str) -> AppResult<()> {
    let encrypted = dpapi::encrypt_str(secret);
    let len = secret.len();
    store.update(|c| {
        c.asr.api_secret = encrypted;
        c.asr.api_secret_len = len;
    })
}

/// Decrypt and return the global ASR API secret.
pub fn get_asr_secret(store: &ConfigStore) -> AppResult<String> {
    let cfg = store.read()?;
    if cfg.asr.api_secret.is_empty() {
        return Ok(String::new());
    }
    dpapi::decrypt_str(&cfg.asr.api_secret).map_err(|e| {
        tracing::error!("failed to decrypt ASR api secret: {e}");
        AppError::Internal
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::persist::ConfigStore;
    use std::path::PathBuf;

    fn setup(name: &str) -> (ConfigStore, PathBuf) {
        let dir = std::env::temp_dir().join("whisperkey_secrets_test");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join(format!("config_{name}.json"));
        let _ = std::fs::remove_file(&path);
        let store = ConfigStore::load(path.clone()).unwrap();
        (store, path)
    }

    fn teardown(path: &PathBuf) {
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_llm_key_roundtrip() {
        let (store, path) = setup("llm_roundtrip");
        set_llm_key(&store, "sk-test-key-12345").unwrap();
        let key = get_llm_key(&store).unwrap();
        assert_eq!(key, "sk-test-key-12345");
        teardown(&path);
    }

    #[test]
    fn test_asr_key_roundtrip() {
        let (store, path) = setup("asr_roundtrip");
        set_asr_key(&store, "asr-secret-999").unwrap();
        let key = get_asr_key(&store).unwrap();
        assert_eq!(key, "asr-secret-999");
        teardown(&path);
    }

    #[test]
    fn test_empty_key_returns_empty() {
        let (store, path) = setup("empty");
        let key = get_llm_key(&store).unwrap();
        assert!(key.is_empty());
        teardown(&path);
    }

    #[test]
    fn test_encrypted_at_rest() {
        let (store, path) = setup("encrypted");
        set_llm_key(&store, "sk-secret").unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("sk-secret"), "Key should not be stored in plaintext");
        teardown(&path);
    }

    #[test]
    fn test_llm_secret_roundtrip() {
        let (store, path) = setup("llm_secret");
        set_llm_secret(&store, "ernie-sk-12345").unwrap();
        let secret = get_llm_secret(&store).unwrap();
        assert_eq!(secret, "ernie-sk-12345");
        teardown(&path);
    }

    #[test]
    fn test_asr_secret_roundtrip() {
        let (store, path) = setup("asr_secret");
        set_asr_secret(&store, "volc-access-key-999").unwrap();
        let secret = get_asr_secret(&store).unwrap();
        assert_eq!(secret, "volc-access-key-999");
        teardown(&path);
    }

    #[test]
    fn test_empty_secret_returns_empty() {
        let (store, path) = setup("empty_secret");
        let secret = get_asr_secret(&store).unwrap();
        assert!(secret.is_empty());
        teardown(&path);
    }
}
