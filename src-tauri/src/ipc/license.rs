use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatus {
    pub activated: bool,
    pub products: Vec<String>,
    pub expires_at: Option<u64>,
}

#[tauri::command]
pub async fn cmd_license_status() -> Result<LicenseStatus, String> {
    Ok(LicenseStatus {
        activated: false,
        products: vec![],
        expires_at: None,
    })
}

#[tauri::command]
pub async fn cmd_license_activate(code: String) -> Result<LicenseStatus, String> {
    let _ = code;
    Ok(LicenseStatus {
        activated: true,
        products: vec!["polish".into(), "markdown".into()],
        expires_at: None,
    })
}

#[tauri::command]
pub async fn cmd_license_unbind() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn cmd_app_quit() -> Result<(), String> {
    std::process::exit(0);
}

#[tauri::command]
pub async fn cmd_app_open_logs_folder() -> Result<(), String> {
    let logs_dir = crate::util::paths::AppPaths::new().logs_dir;
    // Open logs folder in File Explorer
    std::process::Command::new("explorer")
        .arg(logs_dir)
        .spawn()
        .map_err(|e| format!("Failed to open logs folder: {e}"))?;
    Ok(())
}
