/// IT-10: Config v1 → v2 migration — fields correct, old file backed up
use whisperkey_lib::config::schema::Config;
use whisperkey_lib::config::persist::ConfigStore;

fn test_path(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("whisperkey_it_config");
    std::fs::create_dir_all(&dir).ok();
    dir.join(format!("config_{name}.json"))
}

fn cleanup(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or(std::path::Path::new("."));
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with(stem.as_ref()) && name.contains("json.bak.") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}

#[test]
fn it10_default_config_creates_v2() {
    let path = test_path("default_v2");
    cleanup(&path);

    let store = ConfigStore::load(path.clone()).unwrap();
    let cfg = store.read().unwrap();
    assert_eq!(cfg.version, 2);
    assert!(path.exists());

    cleanup(&path);
}

#[test]
fn it10_save_and_reload_preserves_values() {
    let path = test_path("save_reload");
    cleanup(&path);

    let store = ConfigStore::load(path.clone()).unwrap();
    store.update(|c| {
        c.audio.max_duration_sec = 90;
        c.hotkey.key = "F2".into();
        c.hotkey.modifiers = vec!["Ctrl".into()];
        c.output_mode = "polish".into();
    }).unwrap();

    let store2 = ConfigStore::load(path.clone()).unwrap();
    let cfg = store2.read().unwrap();
    assert_eq!(cfg.audio.max_duration_sec, 90);
    assert_eq!(cfg.hotkey.key, "F2");
    assert_eq!(cfg.output_mode, "polish");

    cleanup(&path);
}

#[test]
fn it10_v1_to_v2_migration_preserves_fields() {
    let path = test_path("migrate_v1");
    cleanup(&path);

    // Write v1 config
    let v1 = serde_json::json!({
        "version": 1,
        "hotkey": {
            "modifiers": ["Ctrl", "Shift"],
            "key": "Space",
            "paused": false
        },
        "modes": {
            "default": "raw",
            "raw": { "llmProvider": "deepseek", "llmModel": "deepseek-chat" },
            "polish": { "llmProvider": "openai", "llmModel": "gpt-4" }
        },
        "asr": {
            "default": "official",
            "language": "en"
        },
        "providers": {
            "openai": { "apiKey": "old-key", "baseUrl": "https://api.openai.com/v1" },
            "deepseek": { "apiKey": "", "baseUrl": "" }
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
    std::fs::write(&path, serde_json::to_string_pretty(&v1).unwrap()).unwrap();

    // Load triggers migration
    let store = ConfigStore::load(path.clone()).unwrap();
    let cfg = store.read().unwrap();

    assert_eq!(cfg.version, 2);
    assert_eq!(cfg.hotkey.modifiers, vec!["Ctrl", "Shift"]);
    assert_eq!(cfg.hotkey.key, "Space");
    assert_eq!(cfg.audio.max_duration_sec, 30);
    assert!(cfg.audio.silence_auto_stop);
    assert_eq!(cfg.ui.theme, "dark");
    assert!(cfg.system.auto_start);
    assert!(!cfg.history.enabled);
    assert_eq!(cfg.history.retention_days, 14);
    assert_eq!(cfg.advanced.log_level, "debug");

    // v1 → v2 mapping
    assert_eq!(cfg.asr.provider, "official");
    assert_eq!(cfg.asr.language, "en");

    // LLM should be default (v1 per-mode configs are discarded)
    assert_eq!(cfg.llm.provider, "openai");
    assert_eq!(cfg.llm.model, "gpt-4o-mini");

    // Verify v2 on disk has no old keys
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(!raw.contains("modes"));
    assert!(!raw.contains("providers"));
    assert!(raw.contains("\"version\": 2"));

    // Backup file exists
    let stem = path.file_stem().unwrap().to_string_lossy();
    let parent = path.parent().unwrap();
    let backup_exists = std::fs::read_dir(parent).unwrap().any(|e| {
        let name = e.unwrap().file_name().to_string_lossy().into_owned();
        name.starts_with(stem.as_ref()) && name.contains("json.bak.")
    });
    assert!(backup_exists, "v1 backup file should have been created");

    cleanup(&path);
}

#[test]
fn it10_bad_json_rebuilds_default() {
    let path = test_path("bad_json");
    cleanup(&path);
    std::fs::write(&path, "this is not valid json {{{").unwrap();

    let store = ConfigStore::load(path.clone()).unwrap();
    assert_eq!(store.read().unwrap().version, 2);

    cleanup(&path);
}

#[test]
fn it10_deserialize_minimal_v2() {
    let minimal = r#"{"version":2}"#;
    let cfg: Config = serde_json::from_str(minimal).unwrap();
    assert_eq!(cfg.version, 2);
    assert_eq!(cfg.llm.provider, "openai");
    assert_eq!(cfg.asr.language, "auto");
}

#[test]
fn it10_camelcase_roundtrip() {
    let cfg = Config::default();
    let json = serde_json::to_string(&cfg).unwrap();
    let parsed: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.version, 2);
    assert_eq!(parsed.audio.max_duration_sec, 60);
    assert_eq!(parsed.hotkey.key, "J");
}
