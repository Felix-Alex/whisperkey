pub mod codes;
pub mod fingerprint;
pub mod verifier;

use std::sync::RwLock;
use crate::crypto::dpapi;
use crate::error::AppResult;
use crate::license::verifier::LicenseData;
use crate::util::paths::AppPaths;

#[derive(Clone)]
pub struct LicenseStore {
    license: std::sync::Arc<RwLock<Option<LicenseData>>>,
    device_fp: String,
    path: std::path::PathBuf,
}

impl LicenseStore {
    pub fn new(paths: &AppPaths) -> Self {
        let device_fp = fingerprint::device_fingerprint();
        let store = Self {
            license: std::sync::Arc::new(RwLock::new(None)),
            device_fp,
            path: paths.license.clone(),
        };
        let _ = store.load_from_disk();
        store
    }

    /// Read and decrypt license.dat, verify code hash and device fingerprint
    fn load_from_disk(&self) -> AppResult<()> {
        let path = &self.path;
        if !path.exists() {
            return Ok(());
        }

        let encrypted = std::fs::read(path).map_err(|_| crate::error::AppError::Internal)?;
        let json = dpapi::decrypt_result(&encrypted).map_err(|e| {
            tracing::warn!("Failed to decrypt license.dat: {e}");
            crate::error::AppError::Internal
        })?;
        let json_str = String::from_utf8(json).map_err(|_| crate::error::AppError::Internal)?;
        let data: LicenseData =
            serde_json::from_str(&json_str).map_err(|_| crate::error::AppError::Internal)?;

        // Verify code hash is still in whitelist
        if !codes::CODE_HASHES.contains(&data.code_hash.as_str()) {
            tracing::warn!("license.dat code_hash not in whitelist, ignoring");
            return Ok(());
        }

        // Verify device fingerprint matches
        if data.device_fingerprint != self.device_fp {
            tracing::warn!("license.dat device fingerprint mismatch");
            return Ok(());
        }

        *self.license.write().unwrap() = Some(data);
        Ok(())
    }

    /// Activate with a 6-character code. Validates hash, binds to device, writes license.dat.
    pub fn activate(&self, code: &str) -> AppResult<()> {
        let code = code.trim().to_uppercase();
        if code.len() != 6 {
            return Err(crate::error::AppError::LicenseInvalid);
        }

        let code_hash = codes::hash_code(&code);

        if !codes::validate_code(&code) {
            return Err(crate::error::AppError::LicenseInvalid);
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let data = LicenseData {
            version: 1,
            code_hash,
            device_fingerprint: self.device_fp.clone(),
            activated_at: now,
        };

        // Encrypt with DPAPI and write to disk
        let json = serde_json::to_string(&data).map_err(|_| crate::error::AppError::Internal)?;
        let encrypted = dpapi::encrypt(json.as_bytes());
        std::fs::write(&self.path, encrypted).map_err(|_| crate::error::AppError::Internal)?;

        *self.license.write().unwrap() = Some(data);
        tracing::info!("License activated successfully");
        Ok(())
    }

    /// Check if the app is unlocked (any valid activation exists)
    pub fn is_unlocked(&self) -> bool {
        self.license.read().unwrap().is_some()
    }

    pub fn device_fingerprint(&self) -> &str {
        &self.device_fp
    }

    /// Check if the given mode requires activation. Raw is always free.
    pub fn check_mode_access(&self, mode: &str) -> AppResult<()> {
        match mode {
            "raw" => Ok(()),
            "polish" | "markdown" | "quick_ask" | "custom" => {
                if self.is_unlocked() {
                    Ok(())
                } else {
                    Err(crate::error::AppError::LicenseRequired)
                }
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_paths(name: &str) -> AppPaths {
        let dir = std::env::temp_dir().join(format!("whisperkey_license_{name}"));
        std::fs::create_dir_all(&dir).ok();
        AppPaths {
            config: dir.join("config.json"),
            license: dir.join("license.dat"),
            history_db: dir.join("history.db"),
            logs_dir: dir.join("logs"),
            prompts_dir: dir.join("prompts"),
        }
    }

    fn clean(paths: &AppPaths) {
        let _ = std::fs::remove_file(&paths.license);
    }

    #[test]
    fn test_not_unlocked_by_default() {
        let paths = test_paths("not_unlocked");
        clean(&paths);
        let store = LicenseStore::new(&paths);
        assert!(!store.is_unlocked());
        clean(&paths);
    }

    #[test]
    fn test_activate_and_is_unlocked() {
        let paths = test_paths("activate_unlock");
        clean(&paths);
        let store = LicenseStore::new(&paths);
        store.activate("K7mP3x").unwrap();
        assert!(store.is_unlocked());
        clean(&paths);
    }

    #[test]
    fn test_invalid_code_fails() {
        let paths = test_paths("invalid");
        clean(&paths);
        let store = LicenseStore::new(&paths);
        assert!(store.activate("XXXXXX").is_err());
        assert!(!store.is_unlocked());
        clean(&paths);
    }

    #[test]
    fn test_dpapi_persistence() {
        let paths = test_paths("persist");
        clean(&paths);
        let store = LicenseStore::new(&paths);
        store.activate("K7mP3x").unwrap();
        assert!(store.is_unlocked());
        drop(store);

        let store2 = LicenseStore::new(&paths);
        assert!(store2.is_unlocked());
        clean(&paths);
    }

    #[test]
    fn test_short_code_fails() {
        let paths = test_paths("short");
        clean(&paths);
        let store = LicenseStore::new(&paths);
        assert!(store.activate("ABC").is_err());
        clean(&paths);
    }

    #[test]
    fn test_check_mode_access() {
        let paths = test_paths("mode_access");
        clean(&paths);
        let store = LicenseStore::new(&paths);
        // Raw is always free
        assert!(store.check_mode_access("raw").is_ok());
        // All LLM modes require activation
        assert!(store.check_mode_access("polish").is_err());
        assert!(store.check_mode_access("markdown").is_err());
        assert!(store.check_mode_access("quick_ask").is_err());
        assert!(store.check_mode_access("custom").is_err());
        // Activate
        store.activate("K7mP3x").unwrap();
        // All modes should now pass
        assert!(store.check_mode_access("polish").is_ok());
        assert!(store.check_mode_access("markdown").is_ok());
        assert!(store.check_mode_access("quick_ask").is_ok());
        assert!(store.check_mode_access("custom").is_ok());
        clean(&paths);
    }
}
