use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::OnceCell;
use serde::Deserialize;

use crate::config::schema::{AudioConfig, AdvancedConfig, Config, HistoryConfig, HotkeyConfig, SystemConfig, UiConfig};
use crate::error::{AppError, AppResult};

static CONFIG_STORE: OnceCell<ConfigStore> = OnceCell::new();

pub fn init_global(path: PathBuf) -> AppResult<()> {
    let store = ConfigStore::load(path)?;
    CONFIG_STORE.set(store).map_err(|_| AppError::Internal)
}

pub fn global() -> &'static ConfigStore {
    CONFIG_STORE.get().expect("ConfigStore not initialized")
}

#[derive(Clone)]
pub struct ConfigStore {
    config: std::sync::Arc<RwLock<Config>>,
    path: PathBuf,
}

// ── v1 schema types (migration only) ──

#[derive(Debug, Deserialize)]
struct V1Config {
    #[serde(default)]
    hotkey: Option<HotkeyConfig>,
    #[serde(default)]
    asr: Option<V1AsrConfig>,
    #[serde(default)]
    audio: Option<AudioConfig>,
    #[serde(default)]
    ui: Option<UiConfig>,
    #[serde(default)]
    system: Option<SystemConfig>,
    #[serde(default)]
    history: Option<HistoryConfig>,
    #[serde(default)]
    advanced: Option<AdvancedConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct V1AsrConfig {
    #[serde(default)]
    default: Option<String>,
    #[serde(default)]
    language: Option<String>,
}

/// Migrate v1 config JSON to v2 Config struct.
fn migrate_v1_to_v2(v1_json: &str) -> AppResult<Config> {
    let v1: V1Config = serde_json::from_str(v1_json).map_err(|e| {
        tracing::error!("v1 config parse error during migration: {e}");
        AppError::Internal
    })?;

    let mut cfg = Config::default();

    // Preserve compatible fields
    if let Some(h) = v1.hotkey { cfg.hotkey = h; }
    if let Some(a) = v1.audio { cfg.audio = a; }
    if let Some(u) = v1.ui { cfg.ui = u; }
    if let Some(s) = v1.system { cfg.system = s; }
    if let Some(h) = v1.history { cfg.history = h; }
    if let Some(a) = v1.advanced { cfg.advanced = a; }

    // Map v1 asr.default → v2 asr.provider
    if let Some(v1_asr) = v1.asr {
        if let Some(provider) = v1_asr.default {
            cfg.asr.provider = provider;
        }
        if let Some(lang) = v1_asr.language {
            cfg.asr.language = lang;
        }
    }

    // LLM starts with default — v1 per-mode provider configs are discarded
    tracing::info!("v1→v2 migration: asr.provider={}, asr.language={}", cfg.asr.provider, cfg.asr.language);

    Ok(cfg)
}

impl ConfigStore {
    pub fn load(path: PathBuf) -> AppResult<Self> {
        let config = match fs::read_to_string(&path) {
            Ok(content) => {
                // Detect version from raw JSON to decide parse strategy
                let version = serde_json::from_str::<serde_json::Value>(&content)
                    .ok()
                    .and_then(|v| v.get("version")?.as_u64())
                    .unwrap_or(0) as u32;

                match version {
                    2 => match serde_json::from_str::<Config>(&content) {
                        Ok(cfg) => cfg,
                        Err(e) => {
                            Self::backup_file(&path)?;
                            tracing::warn!("failed to parse v2 config: {e}, reset to default");
                            Config::default()
                        }
                    },
                    1 => {
                        tracing::info!("migrating config from v1 to v2");
                        Self::backup_file(&path)?;
                        match migrate_v1_to_v2(&content) {
                            Ok(cfg) => {
                                // Persist the migrated config immediately
                                Self::write_atomic(&path, &cfg)?;
                                cfg
                            }
                            Err(e) => {
                                tracing::warn!("v1 migration failed: {e}, reset to default");
                                Config::default()
                            }
                        }
                    }
                    v => {
                        Self::backup_file(&path)?;
                        tracing::warn!(
                            "config version {v} unknown, backed up and reset to default"
                        );
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
            config: std::sync::Arc::new(RwLock::new(config)),
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
        // Also clean up backup files for this specific test file
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let parent = path.parent().unwrap_or(Path::new("."));
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                // Only remove backups belonging to this test (same file stem)
                if name.starts_with(stem.as_ref()) && name.contains("json.bak.") {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }

    #[test]
    fn test_load_creates_default() {
        let path = test_path("load_default");
        cleanup(&path);

        let store = ConfigStore::load(path.clone()).unwrap();
        assert_eq!(store.read().unwrap().version, 2);
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
        assert_eq!(store.read().unwrap().version, 2);

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

    #[test]
    fn test_v1_to_v2_migration() {
        let path = test_path("migrate_v1");
        cleanup(&path);

        // Write a v1 config to disk
        let v1_json = serde_json::json!({
            "version": 1,
            "hotkey": {
                "modifiers": ["Ctrl", "Shift"],
                "key": "Space",
                "paused": false
            },
            "modes": {
                "default": "raw",
                "raw": { "llmProvider": "qwen-audio", "llmModel": "qwen-omni-turbo" },
                "polish": { "llmProvider": "qwen-audio", "llmModel": "qwen-omni-turbo" },
                "markdown": { "llmProvider": "qwen-audio", "llmModel": "qwen-omni-turbo" }
            },
            "asr": {
                "default": "official",
                "language": "zh-CN"
            },
            "providers": {
                "openai": { "apiKey": "encrypted-sk-old", "baseUrl": "https://api.openai.com/v1" },
                "qwen": { "apiKey": "", "baseUrl": "" },
                "gemini": { "apiKey": "", "baseUrl": "" },
                "doubao": { "apiKey": "", "baseUrl": "" },
                "deepseek": { "apiKey": "", "baseUrl": "" },
                "anthropic": { "apiKey": "", "baseUrl": "" },
                "ernie": { "apiKey": "", "secretKey": "" },
                "xfyun": { "appId": "", "apiKey": "", "apiSecret": "" },
                "volcengine": { "appKey": "", "accessKey": "" },
                "official": {}
            },
            "audio": {
                "maxDurationSec": 30,
                "silenceAutoStop": true,
                "silenceTimeoutMs": 2000,
                "inputDevice": "default"
            },
            "ui": {
                "theme": "dark",
                "language": "zh-CN",
                "indicatorPosition": "bottom-center"
            },
            "system": {
                "autoStart": true,
                "minimizeToTray": true,
                "checkUpdates": false
            },
            "history": {
                "enabled": false,
                "retentionDays": 14
            },
            "advanced": {
                "logLevel": "debug",
                "telemetry": true
            }
        });
        fs::write(&path, serde_json::to_string_pretty(&v1_json).unwrap()).unwrap();

        // Load — should trigger migration
        let store = ConfigStore::load(path.clone()).unwrap();
        let cfg = store.read().unwrap();

        // Check version is now 2
        assert_eq!(cfg.version, 2);

        // Preserved fields
        assert_eq!(cfg.hotkey.modifiers, vec!["Ctrl", "Shift"]);
        assert_eq!(cfg.hotkey.key, "Space");
        assert_eq!(cfg.audio.max_duration_sec, 30);
        assert!(cfg.audio.silence_auto_stop);
        assert_eq!(cfg.audio.silence_timeout_ms, 2000);
        assert_eq!(cfg.ui.theme, "dark");
        assert!(cfg.system.auto_start);
        assert!(!cfg.system.check_updates);
        assert!(!cfg.history.enabled);
        assert_eq!(cfg.history.retention_days, 14);
        assert_eq!(cfg.advanced.log_level, "debug");
        assert!(cfg.advanced.telemetry);

        // Mapped fields
        assert_eq!(cfg.asr.provider, "official");
        assert_eq!(cfg.asr.language, "zh-CN");

        // LLM should be default (no mapping from v1 per-mode config)
        assert_eq!(cfg.llm.provider, "openai");
        assert_eq!(cfg.llm.model, "gpt-4o-mini");

        // Verify the file on disk is now v2 (no modes/providers keys)
        let raw = fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("modes"));
        assert!(!raw.contains("providers"));
        assert!(!raw.contains("llmProvider"));
        assert!(raw.contains("\"version\": 2"));

        // Verify a backup file was created with v1 content
        let stem = path.file_stem().unwrap().to_string_lossy();
        let parent = path.parent().unwrap();
        let backup_exists = fs::read_dir(parent).unwrap().any(|e| {
            let name = e.unwrap().file_name().to_string_lossy().into_owned();
            name.starts_with(stem.as_ref()) && name.contains("json.bak.")
        });
        assert!(backup_exists, "v1 backup file should have been created");

        cleanup(&path);
    }
}
