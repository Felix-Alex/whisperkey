use crate::config::schema::Config;

#[tauri::command]
pub async fn cmd_config_get() -> Result<Config, String> {
    // Return current config
    Ok(Config::default())
}

#[tauri::command]
pub async fn cmd_config_set(config: Config) -> Result<(), String> {
    // Save config
    let _ = config;
    Ok(())
}

#[tauri::command]
pub async fn cmd_config_set_provider_key(provider: String, key: String) -> Result<(), String> {
    // Set provider API key (encrypted)
    let _ = (provider, key);
    Ok(())
}

#[tauri::command]
pub async fn cmd_provider_test_connection(provider: String) -> Result<bool, String> {
    // Test connection to provider
    let _ = provider;
    Ok(true)
}
