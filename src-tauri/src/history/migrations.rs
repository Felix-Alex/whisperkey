/// SQL statements for database initialization
pub const MIGRATIONS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS schema_version (
        version INTEGER NOT NULL PRIMARY KEY,
        applied_at INTEGER NOT NULL
    )",
    "INSERT OR IGNORE INTO schema_version VALUES (1, strftime('%s','now'))",
    "CREATE TABLE IF NOT EXISTS history (
        id            INTEGER PRIMARY KEY AUTOINCREMENT,
        created_at    INTEGER NOT NULL,
        mode          TEXT    NOT NULL CHECK(mode IN ('raw','polish','markdown')),
        raw_text      TEXT    NOT NULL,
        processed_text TEXT   NOT NULL,
        duration_ms   INTEGER NOT NULL DEFAULT 0,
        app_name      TEXT,
        app_title     TEXT,
        asr_provider  TEXT,
        llm_provider  TEXT,
        injected      INTEGER NOT NULL DEFAULT 1
    )",
    "CREATE INDEX IF NOT EXISTS idx_history_created_at ON history(created_at DESC)",
    "CREATE INDEX IF NOT EXISTS idx_history_mode ON history(mode)",
];
