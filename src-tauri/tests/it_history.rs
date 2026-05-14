/// IT-09: 7-day history auto-cleanup — old records deleted, new ones preserved
use whisperkey_lib::history::db::{HistoryDb, NewHistoryEntry};

fn test_db(name: &str) -> (HistoryDb, std::path::PathBuf) {
    let dir = std::env::temp_dir().join("whisperkey_it_history");
    std::fs::create_dir_all(&dir).ok();
    let path = dir.join(format!("{name}.db"));
    let _ = std::fs::remove_file(&path);
    let db = HistoryDb::open(&path).unwrap();
    (db, path)
}

fn entry(mode: &str, raw: &str, processed: &str) -> NewHistoryEntry {
    NewHistoryEntry {
        mode: mode.into(),
        raw_text: raw.into(),
        processed_text: processed.into(),
        duration_ms: 5000,
        app_name: "notepad.exe".into(),
        app_title: "Notepad".into(),
        asr_provider: "openai".into(),
        llm_provider: "openai".into(),
        injected: true,
    }
}

#[test]
fn it09_add_and_list() {
    let (db, path) = test_db("add_list");

    let id1 = db.add(&entry("raw", "今天天气不错", "今天天气不错")).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1100)); // Ensure distinct created_at timestamps
    let id2 = db.add(&entry("polish", "嗯那个我想去吃饭", "我想去吃饭")).unwrap();
    assert!(id1 > 0);
    assert!(id2 > 0);
    assert!(id2 > id1);

    let list = db.list(None, None, 0, 50).unwrap();
    assert_eq!(list.len(), 2);
    // Most recent first
    assert_eq!(list[0].id, id2);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn it09_filter_by_mode() {
    let (db, path) = test_db("filter_mode");

    db.add(&entry("raw", "raw text 1", "raw text 1")).unwrap();
    db.add(&entry("polish", "polish text", "polished text")).unwrap();
    db.add(&entry("markdown", "md text", "md processed")).unwrap();

    let raw_list = db.list(Some("raw"), None, 0, 50).unwrap();
    assert_eq!(raw_list.len(), 1);
    assert_eq!(raw_list[0].mode, "raw");

    let polish_list = db.list(Some("polish"), None, 0, 50).unwrap();
    assert_eq!(polish_list.len(), 1);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn it09_search_text() {
    let (db, path) = test_db("search");

    db.add(&entry("raw", "Hello World", "Hello World")).unwrap();
    db.add(&entry("raw", "你好世界", "你好世界")).unwrap();
    db.add(&entry("polish", "Goodbye World", "Goodbye World")).unwrap();

    let results = db.list(None, Some("Hello"), 0, 50).unwrap();
    assert_eq!(results.len(), 1);

    let results_cn = db.list(None, Some("世界"), 0, 50).unwrap();
    assert_eq!(results_cn.len(), 1);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn it09_pagination() {
    let (db, path) = test_db("pagination");

    for i in 0..5 {
        db.add(&entry("raw", &format!("text {i}"), &format!("text {i}")))
            .unwrap();
    }

    let page0 = db.list(None, None, 0, 2).unwrap();
    assert_eq!(page0.len(), 2);

    let page1 = db.list(None, None, 1, 2).unwrap();
    assert_eq!(page1.len(), 2);

    // Verify no overlap
    let ids0: Vec<i64> = page0.iter().map(|e| e.id).collect();
    let ids1: Vec<i64> = page1.iter().map(|e| e.id).collect();
    for id in ids0 {
        assert!(!ids1.contains(&id));
    }

    let _ = std::fs::remove_file(&path);
}

#[test]
fn it09_delete_single() {
    let (db, path) = test_db("delete");

    let id = db.add(&entry("raw", "to delete", "to delete")).unwrap();
    assert_eq!(db.list(None, None, 0, 50).unwrap().len(), 1);

    db.delete(id).unwrap();
    assert_eq!(db.list(None, None, 0, 50).unwrap().len(), 0);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn it09_clear_all() {
    let (db, path) = test_db("clear");

    db.add(&entry("raw", "a", "a")).unwrap();
    db.add(&entry("raw", "b", "b")).unwrap();
    assert_eq!(db.list(None, None, 0, 50).unwrap().len(), 2);

    db.clear().unwrap();
    assert_eq!(db.list(None, None, 0, 50).unwrap().len(), 0);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn it09_purge_old_records() {
    let (db, path) = test_db("purge");

    // Add a record, then manually set its created_at to 14 days ago
    let id_new = db.add(&entry("raw", "new record", "new record")).unwrap();
    let id_old = db.add(&entry("raw", "old record", "old record")).unwrap();

    // Manually update the old record's created_at to 14 days ago
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        - 14 * 86400;
    db.conn
        .execute(
            "UPDATE history SET created_at = ?1 WHERE id = ?2",
            rusqlite::params![cutoff, id_old],
        )
        .unwrap();

    // Purge records older than 7 days
    db.purge_old(7).unwrap();

    let remaining = db.list(None, None, 0, 50).unwrap();
    let ids: Vec<i64> = remaining.iter().map(|e| e.id).collect();

    assert!(ids.contains(&id_new), "New record should be kept");
    assert!(!ids.contains(&id_old), "Old record should be purged");

    let _ = std::fs::remove_file(&path);
}
