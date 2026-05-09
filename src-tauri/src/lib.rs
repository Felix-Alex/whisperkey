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
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
