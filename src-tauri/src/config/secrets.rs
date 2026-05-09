use crate::config::persist::ConfigStore;
use crate::crypto::dpapi;
use crate::error::{AppError, AppResult};

pub fn set_provider_key(store: &ConfigStore, provider: &str, key: &str) -> AppResult<()> {
    let encrypted = dpapi::encrypt_str(key);

    store.update(|c| {
        let providers = &mut c.providers;
        match provider {
            "openai" => providers.openai.api_key = encrypted,
            "anthropic" => providers.anthropic.api_key = encrypted,
            "deepseek" => providers.deepseek.api_key = encrypted,
            "qwen" => providers.qwen.api_key = encrypted,
            "gemini" => providers.gemini.api_key = encrypted,
            "ernie" => providers.ernie.api_key = encrypted,
            "doubao" => providers.doubao.api_key = encrypted,
            "xfyun" => providers.xfyun.api_key = encrypted,
            "volcengine" => providers.volcengine.access_key = encrypted,
            _ => {}
        }
    })
}

pub fn get_provider_key(store: &ConfigStore, provider: &str) -> AppResult<String> {
    let cfg = store.read()?;
    let encrypted = match provider {
        "openai" => &cfg.providers.openai.api_key,
        "anthropic" => &cfg.providers.anthropic.api_key,
        "deepseek" => &cfg.providers.deepseek.api_key,
        "qwen" => &cfg.providers.qwen.api_key,
        "gemini" => &cfg.providers.gemini.api_key,
        "ernie" => &cfg.providers.ernie.api_key,
        "doubao" => &cfg.providers.doubao.api_key,
        "xfyun" => &cfg.providers.xfyun.api_key,
        "volcengine" => &cfg.providers.volcengine.access_key,
        _ => "",
    };

    if encrypted.is_empty() {
        return Ok(String::new());
    }

    dpapi::decrypt_str(encrypted).map_err(|e| {
        tracing::error!("failed to decrypt api key for {provider}: {e}");
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
    fn test_set_and_get_roundtrip() {
        let (store, path) = setup("roundtrip");
        set_provider_key(&store, "openai", "sk-test-key-12345").unwrap();
        let key = get_provider_key(&store, "openai").unwrap();
        assert_eq!(key, "sk-test-key-12345");
        teardown(&path);
    }

    #[test]
    fn test_multiple_providers() {
        let (store, path) = setup("multi");
        set_provider_key(&store, "openai", "sk-openai").unwrap();
        set_provider_key(&store, "deepseek", "sk-deepseek").unwrap();
        assert_eq!(get_provider_key(&store, "openai").unwrap(), "sk-openai");
        assert_eq!(get_provider_key(&store, "deepseek").unwrap(), "sk-deepseek");
        teardown(&path);
    }

    #[test]
    fn test_unknown_provider_returns_empty() {
        let (store, path) = setup("unknown");
        let key = get_provider_key(&store, "nonexistent").unwrap();
        assert!(key.is_empty());
        teardown(&path);
    }

    #[test]
    fn test_encrypted_at_rest() {
        let (store, path) = setup("encrypted");
        set_provider_key(&store, "openai", "sk-secret").unwrap();

        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("sk-secret"), "Key should not be stored in plaintext");
        teardown(&path);
    }
}
