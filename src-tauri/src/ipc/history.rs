use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app_state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub id: i64,
    pub created_at: i64,
    pub mode: String,
    pub raw_text: String,
    pub processed_text: String,
    pub duration_ms: i64,
    pub app_name: Option<String>,
    pub asr_provider: Option<String>,
    pub llm_provider: Option<String>,
    pub injected: bool,
}

#[tauri::command]
pub async fn cmd_history_list(
    state: State<'_, AppState>,
    mode: Option<String>,
    search: Option<String>,
    page: Option<u32>,
) -> Result<Vec<HistoryItem>, String> {
    let db = state.history_db.lock().map_err(|e| e.to_string())?;
    let entries = db
        .list(mode.as_deref(), search.as_deref(), page.unwrap_or(0), 50)
        .map_err(|e| e.to_string())?;

    let items: Vec<HistoryItem> = entries
        .into_iter()
        .map(|e| HistoryItem {
            id: e.id,
            created_at: e.created_at,
            mode: e.mode,
            raw_text: e.raw_text,
            processed_text: e.processed_text,
            duration_ms: e.duration_ms,
            app_name: e.app_name,
            asr_provider: e.asr_provider,
            llm_provider: e.llm_provider,
            injected: e.injected,
        })
        .collect();
    Ok(items)
}

#[tauri::command]
pub async fn cmd_history_delete(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.history_db.lock().map_err(|e| e.to_string())?;
    db.delete(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_history_clear(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.history_db.lock().map_err(|e| e.to_string())?;
    db.clear().map_err(|e| e.to_string())
}
