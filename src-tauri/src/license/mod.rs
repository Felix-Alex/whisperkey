pub mod activator;
pub mod fingerprint;
pub mod verifier;

use std::path::PathBuf;
use std::sync::RwLock;
use crate::license::verifier::{License, verify};
use crate::error::AppResult;
use crate::util::paths::AppPaths;

pub struct LicenseStore {
    license: RwLock<Option<License>>,
    device_fp: String,
    path: PathBuf,
}

impl LicenseStore {
    pub fn new(paths: &AppPaths) -> Self {
        let device_fp = fingerprint::device_fingerprint();
        let store = Self {
            license: RwLock::new(None),
            device_fp,
            path: paths.license.clone(),
        };
        // Try loading existing license on startup
        let _ = store.load_from_disk();
        store
    }

    fn load_from_disk(&self) -> AppResult<()> {
        if let Ok(data) = std::fs::read(&self.path) {
            if let Ok(lic) = serde_json::from_slice::<License>(&data) {
                if verify(&lic, &self.device_fp).is_ok() {
                    *self.license.write().unwrap() = Some(lic);
                }
            }
        }
        Ok(())
    }

    pub fn save_license(&self, license: License) -> AppResult<()> {
        let data = serde_json::to_vec(&license).map_err(|_| crate::error::AppError::Internal)?;
        std::fs::write(&self.path, data).map_err(|_| crate::error::AppError::Internal)?;
        *self.license.write().unwrap() = Some(license);
        Ok(())
    }

    pub fn is_unlocked(&self, product: &str) -> bool {
        self.license
            .read()
            .unwrap()
            .as_ref()
            .map(|lic| lic.products.contains(&product.to_string()))
            .unwrap_or(false)
    }

    pub fn device_fingerprint(&self) -> &str {
        &self.device_fp
    }

    /// Check if mode B/C is accessible (requires license)
    pub fn check_mode_access(&self, mode: &str) -> AppResult<()> {
        match mode {
            "raw" => Ok(()),
            "polish" | "markdown" => {
                if self.is_unlocked(mode) {
                    Ok(())
                } else {
                    Err(crate::error::AppError::LicenseInvalid)
                }
            }
            _ => Ok(()),
        }
    }
}

/// Background task: silent license refresh every 30 days
pub async fn refresh_license_task() {
    // Stub: check license periodically and refresh from server
    tracing::debug!("License refresh task running (stub)");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_paths() -> AppPaths {
        let dir = std::env::temp_dir().join("whisperkey_license_test");
        std::fs::create_dir_all(&dir).ok();
        AppPaths {
            config: dir.join("config.json"),
            license: dir.join("license.dat"),
            history_db: dir.join("history.db"),
            logs_dir: dir.join("logs"),
            prompts_dir: dir.join("prompts"),
        }
    }

    #[test]
    fn test_is_unlocked_false_when_no_license() {
        let paths = test_paths();
        let _ = std::fs::remove_file(&paths.license);
        let store = LicenseStore::new(&paths);
        assert!(!store.is_unlocked("polish"));
        let _ = std::fs::remove_file(&paths.license);
    }

    #[test]
    fn test_is_unlocked_true() {
        let paths = test_paths();
        let _ = std::fs::remove_file(&paths.license);
        let store = LicenseStore::new(&paths);
        let lic = License {
            version: 1,
            license_id: "test".into(),
            user_id: "u1".into(),
            products: vec!["polish".into(), "markdown".into()],
            device_fingerprint: store.device_fingerprint().to_string(),
            issued_at: 1736000000,
            expires_at: None,
            signature: "sig".into(),
        };
        store.save_license(lic).unwrap();
        assert!(store.is_unlocked("polish"));
        assert!(store.is_unlocked("markdown"));
        assert!(!store.is_unlocked("raw"));
        let _ = std::fs::remove_file(&paths.license);
    }
}
