use rusqlite::Connection;
use crate::error::{AppError, AppResult};

/// Run pending migrations based on schema_version.
pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    // Ensure schema_version table exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL PRIMARY KEY,
            applied_at INTEGER NOT NULL
        )",
        [],
    ).map_err(|_| AppError::Internal)?;

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // v1: initial schema
    if current < 1 {
        conn.execute_batch(
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
            );
            CREATE INDEX IF NOT EXISTS idx_history_created_at ON history(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_history_mode ON history(mode);
            INSERT INTO schema_version VALUES (1, strftime('%s','now'));"
        ).map_err(|_| AppError::Internal)?;
    }

    // v2: widen mode CHECK constraint to 5 values
    if current < 2 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history_v2 (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at    INTEGER NOT NULL,
                mode          TEXT    NOT NULL CHECK(mode IN ('raw','polish','markdown','humor','venomous')),
                raw_text      TEXT    NOT NULL,
                processed_text TEXT   NOT NULL,
                duration_ms   INTEGER NOT NULL DEFAULT 0,
                app_name      TEXT,
                app_title     TEXT,
                asr_provider  TEXT,
                llm_provider  TEXT,
                injected      INTEGER NOT NULL DEFAULT 1
            );
            INSERT OR IGNORE INTO history_v2 SELECT * FROM history;
            DROP TABLE IF EXISTS history;
            ALTER TABLE history_v2 RENAME TO history;
            CREATE INDEX IF NOT EXISTS idx_history_created_at ON history(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_history_mode ON history(mode);
            INSERT OR IGNORE INTO schema_version VALUES (2, strftime('%s','now'));"
        ).map_err(|_| AppError::Internal)?;
    }

    // v3: replace humor/venomous with custom mode
    if current < 3 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history_v3 (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at    INTEGER NOT NULL,
                mode          TEXT    NOT NULL CHECK(mode IN ('raw','polish','markdown','custom')),
                raw_text      TEXT    NOT NULL,
                processed_text TEXT   NOT NULL,
                duration_ms   INTEGER NOT NULL DEFAULT 0,
                app_name      TEXT,
                app_title     TEXT,
                asr_provider  TEXT,
                llm_provider  TEXT,
                injected      INTEGER NOT NULL DEFAULT 1
            );
            INSERT INTO history_v3
                SELECT id, created_at,
                    CASE WHEN mode IN ('humor','venomous') THEN 'custom' ELSE mode END,
                    raw_text, processed_text, duration_ms,
                    app_name, app_title, asr_provider, llm_provider, injected
                FROM history;
            DROP TABLE IF EXISTS history;
            ALTER TABLE history_v3 RENAME TO history;
            CREATE INDEX IF NOT EXISTS idx_history_created_at ON history(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_history_mode ON history(mode);
            INSERT OR IGNORE INTO schema_version VALUES (3, strftime('%s','now'));"
        ).map_err(|_| AppError::Internal)?;
    }

    // v4: add quick_ask mode
    if current < 4 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history_v4 (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at    INTEGER NOT NULL,
                mode          TEXT    NOT NULL CHECK(mode IN ('raw','polish','markdown','quick_ask','custom')),
                raw_text      TEXT    NOT NULL,
                processed_text TEXT   NOT NULL,
                duration_ms   INTEGER NOT NULL DEFAULT 0,
                app_name      TEXT,
                app_title     TEXT,
                asr_provider  TEXT,
                llm_provider  TEXT,
                injected      INTEGER NOT NULL DEFAULT 1
            );
            INSERT INTO history_v4 SELECT * FROM history;
            DROP TABLE IF EXISTS history;
            ALTER TABLE history_v4 RENAME TO history;
            CREATE INDEX IF NOT EXISTS idx_history_created_at ON history(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_history_mode ON history(mode);
            INSERT OR IGNORE INTO schema_version VALUES (4, strftime('%s','now'));"
        ).map_err(|_| AppError::Internal)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn in_memory() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    #[test]
    fn test_new_db_has_5_mode_check() {
        let conn = in_memory();
        run_migrations(&conn).unwrap();

        // Inserting valid modes should work
        for mode in &["raw", "polish", "markdown", "quick_ask", "custom"] {
            conn.execute(
                "INSERT INTO history (created_at, mode, raw_text, processed_text) VALUES (1, ?1, '', '')",
                [mode],
            ).unwrap();
        }

        // Invalid mode should fail
        let err = conn.execute(
            "INSERT INTO history (created_at, mode, raw_text, processed_text) VALUES (1, 'invalid_mode', '', '')",
            [],
        );
        assert!(err.is_err(), "Invalid mode should be rejected by CHECK constraint");
    }

    #[test]
    fn test_migration_from_v1_preserves_data() {
        let conn = in_memory();

        // Manually create v1 table with 3-mode CHECK
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL PRIMARY KEY,
                applied_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS history (
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
            );
            INSERT INTO history (created_at, mode, raw_text, processed_text, duration_ms)
            VALUES (1000, 'raw', 'hello', 'hello', 500),
                   (2000, 'polish', 'world', 'world!', 600),
                   (3000, 'markdown', 'prompt', '# prompt', 700);
            INSERT INTO schema_version VALUES (1, 12345);"
        ).unwrap();

        // Run migration to v4
        run_migrations(&conn).unwrap();

        // Verify old data preserved
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 3);

        // Verify quick_ask mode can be inserted
        conn.execute(
            "INSERT INTO history (created_at, mode, raw_text, processed_text) VALUES (4000, 'quick_ask', 'question', 'answer')",
            [],
        ).unwrap();

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn test_migration_from_v2_converts_humor_venomous_to_custom() {
        let conn = in_memory();

        // Create v2 database with humor and venomous data
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL PRIMARY KEY,
                applied_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS history (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at    INTEGER NOT NULL,
                mode          TEXT    NOT NULL CHECK(mode IN ('raw','polish','markdown','humor','venomous')),
                raw_text      TEXT    NOT NULL,
                processed_text TEXT   NOT NULL,
                duration_ms   INTEGER NOT NULL DEFAULT 0,
                app_name      TEXT,
                app_title     TEXT,
                asr_provider  TEXT,
                llm_provider  TEXT,
                injected      INTEGER NOT NULL DEFAULT 1
            );
            INSERT INTO history (created_at, mode, raw_text, processed_text)
            VALUES (1000, 'humor', 'joke', 'funny'),
                   (2000, 'venomous', 'angry', 'rage');
            INSERT INTO schema_version VALUES (2, 12345);"
        ).unwrap();

        // Run migration to v4
        run_migrations(&conn).unwrap();

        // Verify both rows converted to custom
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM history WHERE mode = 'custom'", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 2, "humor and venomous rows should be converted to custom");
    }

    #[test]
    fn test_idempotent_migration() {
        let conn = in_memory();
        run_migrations(&conn).unwrap();
        // Running again should not error
        run_migrations(&conn).unwrap();

        conn.execute(
            "INSERT INTO history (created_at, mode, raw_text, processed_text) VALUES (1, 'raw', '', '')",
            [],
        ).unwrap();
    }
}
