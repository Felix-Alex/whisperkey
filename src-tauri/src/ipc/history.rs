use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: i64,
    pub created_at: i64,
    pub mode: String,
    pub raw_text: String,
    pub processed_text: String,
    pub duration_ms: i64,
    pub app_name: Option<String>,
    pub injected: bool,
}

#[tauri::command]
pub async fn cmd_history_list(
    mode: Option<String>,
    search: Option<String>,
    page: Option<u32>,
) -> Result<Vec<HistoryItem>, String> {
    let _ = (mode, search, page);
    Ok(vec![])
}

#[tauri::command]
pub async fn cmd_history_delete(id: i64) -> Result<(), String> {
    let _ = id;
    Ok(())
}

#[tauri::command]
pub async fn cmd_history_clear() -> Result<(), String> {
    Ok(())
}
