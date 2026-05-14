/// IT-08: License signature validation — valid / tampered / expired
/// IT-11: Unactivated → non-raw mode blocked, activated → all modes open
use whisperkey_lib::license::codes;
use whisperkey_lib::license::LicenseStore;
use whisperkey_lib::util::paths::AppPaths;

fn test_paths(name: &str) -> AppPaths {
    let dir = std::env::temp_dir().join(format!("whisperkey_it_license_{name}"));
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

// ── IT-08: License code validation ──

#[test]
fn it08_valid_activation_code() {
    assert!(codes::validate_code("K7mP3x"));
    assert!(codes::validate_code("N9fA2d"));
    assert!(codes::validate_code("B8qW5z"));
}

#[test]
fn it08_valid_lowercase_and_whitespace() {
    assert!(codes::validate_code("k7mp3x"));
    assert!(codes::validate_code("  K7mP3x  "));
}

#[test]
fn it08_invalid_random_code() {
    assert!(!codes::validate_code("XXXXXX"));
    assert!(!codes::validate_code("000000"));
    assert!(!codes::validate_code("ABC123"));
}

#[test]
fn it08_invalid_empty_or_short() {
    assert!(!codes::validate_code(""));
    assert!(!codes::validate_code("AB"));
    assert!(!codes::validate_code("ABC"));
}

#[test]
fn it08_invalid_too_long() {
    assert!(!codes::validate_code("K7mP3x1"));
    assert!(!codes::validate_code("K7mP3x12"));
}

// ── IT-08: License persistence (DPAPI encrypted) ──

#[test]
fn it08_license_persists_across_store_instances() {
    let paths = test_paths("persist");
    clean(&paths);

    let store = LicenseStore::new(&paths);
    assert!(!store.is_unlocked());
    store.activate("K7mP3x").unwrap();
    assert!(store.is_unlocked());
    drop(store);

    // New store instance loads from disk
    let store2 = LicenseStore::new(&paths);
    assert!(store2.is_unlocked());

    clean(&paths);
}

#[test]
fn it08_invalid_code_does_not_activate() {
    let paths = test_paths("invalid");
    clean(&paths);

    let store = LicenseStore::new(&paths);
    let result = store.activate("XXXXXX");
    assert!(result.is_err());
    assert!(!store.is_unlocked());

    clean(&paths);
}

#[test]
fn it08_short_code_returns_error() {
    let paths = test_paths("short");
    clean(&paths);

    let store = LicenseStore::new(&paths);
    let result = store.activate("ABC");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), "E_LICENSE_INVALID");

    clean(&paths);
}

// ── IT-11: License gating — mode access ──

#[test]
fn it11_raw_mode_always_free() {
    let paths = test_paths("raw_free");
    clean(&paths);

    let store = LicenseStore::new(&paths);
    assert!(!store.is_unlocked());
    // Raw mode must work even without activation
    assert!(store.check_mode_access("raw").is_ok());

    clean(&paths);
}

#[test]
fn it11_non_raw_modes_blocked_when_unactivated() {
    let paths = test_paths("blocked");
    clean(&paths);

    let store = LicenseStore::new(&paths);
    assert!(!store.is_unlocked());

    for mode in &["polish", "markdown", "quick_ask", "custom"] {
        let result = store.check_mode_access(mode);
        assert!(result.is_err(), "mode '{mode}' should be blocked when unactivated");
        assert_eq!(result.unwrap_err().code(), "E_LICENSE_REQUIRED");
    }

    clean(&paths);
}

#[test]
fn it11_all_modes_allowed_after_activation() {
    let paths = test_paths("all_open");
    clean(&paths);

    let store = LicenseStore::new(&paths);
    store.activate("K7mP3x").unwrap();
    assert!(store.is_unlocked());

    for mode in &["raw", "polish", "markdown", "quick_ask", "custom"] {
        assert!(
            store.check_mode_access(mode).is_ok(),
            "mode '{mode}' should pass after activation"
        );
    }

    clean(&paths);
}

#[test]
fn it08_default_not_activated() {
    let paths = test_paths("default");
    clean(&paths);

    let store = LicenseStore::new(&paths);
    assert!(!store.is_unlocked());

    clean(&paths);
}
