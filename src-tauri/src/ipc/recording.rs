use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingState {
    pub is_recording: bool,
    pub elapsed_ms: u64,
    pub level: f64,
    pub mode: String,
}

#[tauri::command]
pub async fn cmd_recording_toggle() -> Result<(), String> {
    // Toggle recording start/stop
    Ok(())
}

#[tauri::command]
pub async fn cmd_recording_get_state() -> Result<RecordingState, String> {
    // Get current recording state
    Ok(RecordingState {
        is_recording: false,
        elapsed_ms: 0,
        level: 0.0,
        mode: "raw".into(),
    })
}
