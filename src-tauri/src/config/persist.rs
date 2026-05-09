use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::OnceCell;

use crate::config::schema::Config;
use crate::error::{AppError, AppResult};

static CONFIG_STORE: OnceCell<ConfigStore> = OnceCell::new();

pub fn init_global(path: PathBuf) -> AppResult<()> {
    let store = ConfigStore::load(path)?;
    CONFIG_STORE.set(store).map_err(|_| AppError::Internal)
}

pub fn global() -> &'static ConfigStore {
    CONFIG_STORE.get().expect("ConfigStore not initialized")
}

pub struct ConfigStore {
    config: RwLock<Config>,
    path: PathBuf,
}

impl ConfigStore {
    pub fn load(path: PathBuf) -> AppResult<Self> {
        let config = match fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<Config>(&content) {
                    Ok(cfg) => {
                        // Version migration: if version mismatch, backup and reset
                        if cfg.version != 1 {
                            Self::backup_file(&path)?;
                            tracing::warn!(
                                "config version {} != 1, backed up and reset to default",
                                cfg.version
                            );
                            Config::default()
                        } else {
                            cfg
                        }
                    }
                    Err(e) => {
                        // Bad JSON: backup and create default
                        Self::backup_file(&path)?;
                        tracing::warn!("failed to parse config: {e}, reset to default");
                        Config::default()
                    }
                }
            }
            Err(_) => {
                // File doesn't exist: create with default
                let cfg = Config::default();
                Self::write_atomic(&path, &cfg)?;
                cfg
            }
        };

        Ok(Self {
            config: RwLock::new(config),
            path,
        })
    }

    pub fn save(&self) -> AppResult<()> {
        let cfg = self.config.read().map_err(|_| AppError::Internal)?;
        Self::write_atomic(&self.path, &cfg)
    }

    pub fn read(&self) -> AppResult<std::sync::RwLockReadGuard<'_, Config>> {
        self.config.read().map_err(|_| AppError::Internal)
    }

    pub fn update<F>(&self, f: F) -> AppResult<()>
    where
        F: FnOnce(&mut Config),
    {
        {
            let mut cfg = self.config.write().map_err(|_| AppError::Internal)?;
            f(&mut cfg);
        }
        let cfg = self.config.read().map_err(|_| AppError::Internal)?;
        Self::write_atomic(&self.path, &cfg)
    }

    fn backup_file(path: &PathBuf) -> AppResult<()> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let backup = path.with_extension(format!("json.bak.{ts}"));
        if path.exists() {
            fs::rename(path, &backup).map_err(|_| AppError::Internal)?;
        }
        Ok(())
    }

    fn write_atomic(path: &PathBuf, config: &Config) -> AppResult<()> {
        let json = serde_json::to_string_pretty(config).map_err(|_| AppError::Internal)?;
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, json).map_err(|_| AppError::Internal)?;
        fs::rename(&tmp, path).map_err(|_| AppError::Internal)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn test_dir() -> PathBuf {
        std::env::temp_dir().join("whisperkey_test")
    }

    fn test_path(name: &str) -> PathBuf {
        let dir = test_dir();
        fs::create_dir_all(&dir).ok();
        dir.join(format!("config_{name}.json"))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_load_creates_default() {
        let path = test_path("load_default");
        cleanup(&path);

        let store = ConfigStore::load(path.clone()).unwrap();
        assert_eq!(store.read().unwrap().version, 1);
        assert!(path.exists());

        cleanup(&path);
    }

    #[test]
    fn test_save_and_reload() {
        let path = test_path("save_reload");
        cleanup(&path);

        let store = ConfigStore::load(path.clone()).unwrap();
        store
            .update(|c| {
                c.audio.max_duration_sec = 120;
            })
            .unwrap();

        let store2 = ConfigStore::load(path.clone()).unwrap();
        assert_eq!(store2.read().unwrap().audio.max_duration_sec, 120);

        cleanup(&path);
    }

    #[test]
    fn test_bad_json_rebuilds() {
        let path = test_path("bad_json");
        cleanup(&path);
        fs::write(&path, "not valid json").unwrap();

        let store = ConfigStore::load(path.clone()).unwrap();
        assert_eq!(store.read().unwrap().version, 1);

        cleanup(&path);
    }

    #[test]
    fn test_write_then_read_consistent() {
        let path = test_path("write_read");
        cleanup(&path);

        let store = ConfigStore::load(path.clone()).unwrap();
        store
            .update(|c| {
                c.hotkey.key = "F2".into();
                c.hotkey.modifiers = vec!["Alt".into()];
            })
            .unwrap();

        let raw = fs::read_to_string(&path).unwrap();
        let parsed: Config = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed.hotkey.key, "F2");

        cleanup(&path);
    }
}
