use rusqlite::{params, Connection};
use crate::error::{AppError, AppResult};

pub struct HistoryDb {
    pub conn: Connection,
}

impl HistoryDb {
    pub fn open(path: &std::path::Path) -> AppResult<Self> {
        let conn = Connection::open(path).map_err(|_| AppError::Internal)?;
        crate::history::migrations::run_migrations(&conn)?;
        Ok(Self { conn })
    }

    pub fn add(&self, entry: &NewHistoryEntry) -> AppResult<i64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.conn
            .execute(
                "INSERT INTO history (created_at, mode, raw_text, processed_text, duration_ms, app_name, app_title, asr_provider, llm_provider, injected)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    now,
                    entry.mode,
                    entry.raw_text,
                    entry.processed_text,
                    entry.duration_ms as i64,
                    entry.app_name,
                    entry.app_title,
                    entry.asr_provider,
                    entry.llm_provider,
                    entry.injected as i64
                ],
            )
            .map_err(|_| AppError::Internal)?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn list(
        &self,
        filter_mode: Option<&str>,
        search: Option<&str>,
        page: u32,
        page_size: u32,
    ) -> AppResult<Vec<HistoryEntry>> {
        let offset = (page as i64) * (page_size as i64);
        let limit = page_size as i64;

        let mut sql = String::from("SELECT id, created_at, mode, raw_text, processed_text, duration_ms, app_name, app_title, asr_provider, llm_provider, injected FROM history WHERE 1=1");
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(mode) = filter_mode {
            sql.push_str(" AND mode = ?");
            param_values.push(Box::new(mode.to_string()));
        }
        if let Some(q) = search {
            sql.push_str(" AND (raw_text LIKE ? OR processed_text LIKE ?)");
            let like = format!("%{q}%");
            param_values.push(Box::new(like.clone()));
            param_values.push(Box::new(like));
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        param_values.push(Box::new(limit));
        param_values.push(Box::new(offset));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql).map_err(|_| AppError::Internal)?;
        let rows = stmt
            .query_map(params_ref.as_slice(), |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    mode: row.get(2)?,
                    raw_text: row.get(3)?,
                    processed_text: row.get(4)?,
                    duration_ms: row.get(5)?,
                    app_name: row.get(6)?,
                    app_title: row.get(7)?,
                    asr_provider: row.get(8)?,
                    llm_provider: row.get(9)?,
                    injected: row.get::<_, i64>(10)? != 0,
                })
            })
            .map_err(|_| AppError::Internal)?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|_| AppError::Internal)?);
        }
        Ok(entries)
    }

    pub fn delete(&self, id: i64) -> AppResult<()> {
        self.conn
            .execute("DELETE FROM history WHERE id = ?1", params![id])
            .map_err(|_| AppError::Internal)?;
        Ok(())
    }

    pub fn clear(&self) -> AppResult<()> {
        self.conn
            .execute("DELETE FROM history", [])
            .map_err(|_| AppError::Internal)?;
        Ok(())
    }

    pub fn purge_old(&self, retention_days: u32) -> AppResult<()> {
        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
            - (retention_days as i64 * 86400);

        self.conn
            .execute("DELETE FROM history WHERE created_at < ?1", params![cutoff])
            .map_err(|_| AppError::Internal)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NewHistoryEntry {
    pub mode: String,
    pub raw_text: String,
    pub processed_text: String,
    pub duration_ms: u64,
    pub app_name: String,
    pub app_title: String,
    pub asr_provider: String,
    pub llm_provider: String,
    pub injected: bool,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: i64,
    pub created_at: i64,
    pub mode: String,
    pub raw_text: String,
    pub processed_text: String,
    pub duration_ms: i64,
    pub app_name: Option<String>,
    pub app_title: Option<String>,
    pub asr_provider: Option<String>,
    pub llm_provider: Option<String>,
    pub injected: bool,
}
