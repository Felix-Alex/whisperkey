use serde::{Deserialize, Serialize};
use tauri::State;
use crate::app_state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatus {
    pub activated: bool,
}

#[tauri::command]
pub async fn cmd_license_status(state: State<'_, AppState>) -> Result<LicenseStatus, String> {
    Ok(LicenseStatus {
        activated: state.license_store.is_unlocked(),
    })
}

#[tauri::command]
pub async fn cmd_license_activate(
    code: String,
    state: State<'_, AppState>,
) -> Result<LicenseStatus, String> {
    state.license_store.activate(&code).map_err(|e| e.to_string())?;
    Ok(LicenseStatus {
        activated: state.license_store.is_unlocked(),
    })
}

#[tauri::command]
pub async fn cmd_app_quit(app: tauri::AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub async fn cmd_app_open_logs_folder() -> Result<(), String> {
    let logs_dir = crate::util::paths::AppPaths::new().logs_dir;
    std::process::Command::new("explorer")
        .arg(logs_dir)
        .spawn()
        .map_err(|e| format!("Failed to open logs folder: {e}"))?;
    Ok(())
}
