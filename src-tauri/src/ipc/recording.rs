use serde::{Deserialize, Serialize};
use tauri::State;
use crate::app_state::AppState;
use crate::hotkey::HotkeyEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingState {
    pub is_recording: bool,
    pub elapsed_ms: u64,
    pub level: f64,
    pub mode: String,
}

#[tauri::command]
pub async fn cmd_recording_toggle(state: State<'_, AppState>) -> Result<(), String> {
    state.hotkey_tx.send(HotkeyEvent::Triggered)
        .map_err(|e| format!("Failed to trigger: {e}"))?;
    tracing::info!("Recording toggle triggered via IPC");
    Ok(())
}

#[tauri::command]
pub async fn cmd_recording_get_state(state: State<'_, AppState>) -> Result<RecordingState, String> {
    Ok(RecordingState {
        is_recording: false,
        elapsed_ms: 0,
        level: 0.0,
        mode: state.output_mode(),
    })
}

#[tauri::command]
pub async fn cmd_set_output_mode(mode: String, state: State<'_, AppState>) -> Result<(), String> {
    state.set_output_mode(&mode).map_err(|e| e.to_string())
}
