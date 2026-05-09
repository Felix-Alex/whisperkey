use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub configured: bool,
    pub test_ok: bool,
}

#[tauri::command]
pub async fn cmd_providers_list() -> Result<Vec<ProviderInfo>, String> {
    // List all ASR/LLM providers with their status
    Ok(vec![])
}

#[tauri::command]
pub async fn cmd_providers_test(provider: String) -> Result<bool, String> {
    // Test a specific provider connection
    let _ = provider;
    Ok(true)
}
