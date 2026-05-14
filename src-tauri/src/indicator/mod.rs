use serde::{Deserialize, Serialize};
use tauri::WebviewWindow;
use tauri::Emitter;
use tracing;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IndicatorState {
    Idle,
    Recording,
    Processing,
    AsrTranscribing,
    LlmProcessing,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorEvent {
    pub state: IndicatorState,
    pub mode: Option<String>,
    pub level: Option<f32>,
}

pub struct Indicator {
    window: WebviewWindow,
    visible: bool,
}

impl Indicator {
    pub fn new(window: WebviewWindow) -> Self {
        tracing::info!("[INDICATOR] Created with window label={}", window.label());
        Self { window, visible: false }
    }

    pub fn show(&mut self, mode: &str) {
        tracing::info!("[INDICATOR] show() called, mode={mode}");
        self.visible = true;
        if let Err(e) = self.window.show() {
            tracing::error!("[INDICATOR] Failed to show window: {e}");
        }
        if let Err(e) = self.window.set_focus() {
            tracing::warn!("[INDICATOR] set_focus failed (may be expected): {e}");
        }
    }

    pub fn hide(&mut self) {
        tracing::info!("[INDICATOR] hide() called");
        self.visible = false;
        if let Err(e) = self.window.hide() {
            tracing::error!("[INDICATOR] Failed to hide window: {e}");
        }
    }

    pub fn set_state(&self, state: IndicatorState) {
        self.set_state_msg(state, "");
    }

    pub fn set_state_msg(&self, state: IndicatorState, message: &str) {
        tracing::info!("[INDICATOR] set_state({state:?}, msg=\"{message}\")");
        let mut payload = serde_json::json!({ "state": state });
        if !message.is_empty() {
            payload["message"] = serde_json::Value::String(message.to_string());
        }
        if let Err(e) = self.window.emit("indicator-state", &payload) {
            tracing::error!("[INDICATOR] emit indicator-state failed: {e}");
        }
    }

    pub fn update_level(&self, level: f32) {
        if let Err(e) = self.window.emit("indicator-level", level) {
            tracing::error!("[INDICATOR] emit indicator-level failed: {e}");
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

impl Default for Indicator {
    fn default() -> Self {
        // Panic-safe: only used as fallback, never in production path
        panic!("Indicator must be created with a WebviewWindow");
    }
}
