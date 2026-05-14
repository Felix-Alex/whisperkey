use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use tokio::sync::broadcast;
use crate::audio::device_cache::AudioDeviceCache;
use crate::config::persist::ConfigStore;
use crate::error::{AppError, AppResult};
use crate::history::db::HistoryDb;
use crate::hotkey::{HotkeyConfig, HotkeyEvent};
use crate::hotkey::registrar::HotkeyHandle;
use crate::license::LicenseStore;
use tracing;

pub struct AppState {
    pub config_store: ConfigStore,
    pub license_store: LicenseStore,
    pub hotkey_tx: broadcast::Sender<HotkeyEvent>,
    pub asr_registry: Arc<crate::asr::AsrRegistry>,
    pub llm_registry: Arc<crate::llm::LlmRegistry>,
    pub device_cache: Arc<AudioDeviceCache>,
    pub history_db: Arc<Mutex<HistoryDb>>,
    output_mode: RwLock<String>,
    hotkey_handle: RwLock<Option<HotkeyHandle>>,
}

impl AppState {
    pub fn new(
        config_store: ConfigStore,
        license_store: LicenseStore,
        hotkey_tx: broadcast::Sender<HotkeyEvent>,
        device_cache: Arc<AudioDeviceCache>,
        history_db: Arc<Mutex<HistoryDb>>,
    ) -> Self {
        let output_mode = config_store
            .read()
            .map(|c| c.output_mode.clone())
            .unwrap_or_else(|_| "raw".into());
        Self {
            config_store,
            license_store,
            hotkey_tx,
            asr_registry: Arc::new(crate::asr::default_registry()),
            llm_registry: Arc::new(crate::llm::default_registry()),
            device_cache,
            history_db,
            output_mode: RwLock::new(output_mode),
            hotkey_handle: RwLock::new(None),
        }
    }

    pub fn output_mode(&self) -> String {
        self.output_mode.read().unwrap().clone()
    }

    pub fn set_output_mode(&self, mode: &str) -> AppResult<()> {
        match mode {
            "raw" | "polish" | "markdown" | "quick_ask" | "custom" => {
                *self.output_mode.write().unwrap() = mode.to_string();
                Ok(())
            }
            _ => Err(AppError::Internal),
        }
    }

    /// Restart the OS hotkey listener after config change.
    /// Drops the old handle (which posts WM_QUIT to the pump thread and
    /// unregisters the old hotkey), then spawns a fresh listener.
    pub fn restart_hotkey(&self) -> AppResult<()> {
        tracing::info!("[HOTKEY] Restart requested — dropping old handle");
        // Drop the old handle: sets stop_flag + posts WM_QUIT → thread exits
        *self.hotkey_handle.write().unwrap() = None;

        let cfg = self.config_store.read().map_err(|_| AppError::Internal)?;
        let hotkey = &cfg.hotkey;
        let hk = HotkeyConfig::from_string(
            &format!("{}+{}", hotkey.modifiers.join("+"), hotkey.key)
        ).unwrap_or_else(|| HotkeyConfig::new(vec![crate::hotkey::Modifier::Alt], "J"));

        tracing::info!("[HOTKEY] Restart: new hotkey = {hk}");
        let handle = crate::hotkey::registrar::start(hk, self.hotkey_tx.clone());
        *self.hotkey_handle.write().unwrap() = Some(handle);
        Ok(())
    }

    /// Store the initial hotkey handle after startup.
    pub fn set_hotkey_handle(&self, handle: HotkeyHandle) {
        *self.hotkey_handle.write().unwrap() = Some(handle);
    }
}
