pub mod app_state;
pub mod asr;
pub mod audio;
pub mod config;
pub mod crypto;
pub mod error;
pub mod history;
pub mod hotkey;
pub mod indicator;
pub mod inject;
pub mod ipc;
pub mod license;
pub mod llm;
pub mod log;
pub mod pipeline;
pub mod tray;
pub mod updater;
pub mod util;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            ipc::config::cmd_config_get,
            ipc::config::cmd_config_set,
            ipc::config::cmd_config_set_provider_key,
            ipc::config::cmd_provider_test_connection,
            ipc::recording::cmd_recording_toggle,
            ipc::recording::cmd_recording_get_state,
            ipc::providers::cmd_providers_list,
            ipc::providers::cmd_providers_test,
            ipc::history::cmd_history_list,
            ipc::history::cmd_history_delete,
            ipc::history::cmd_history_clear,
            ipc::license::cmd_license_status,
            ipc::license::cmd_license_activate,
            ipc::license::cmd_license_unbind,
            ipc::license::cmd_app_quit,
            ipc::license::cmd_app_open_logs_folder,
        ])
        .setup(|_app| {
            // Global panic hook
            std::panic::set_hook(Box::new(|info| {
                tracing::error!("Panic: {info}");
                let crash_dir = dirs::data_dir()
                    .unwrap_or_default()
                    .join("WhisperKey")
                    .join("logs")
                    .join("crash");
                let _ = std::fs::create_dir_all(&crash_dir);
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let path = crash_dir.join(format!("crash-{ts}.txt"));
                let _ = std::fs::write(&path, format!("{info}\n"));
            }));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
