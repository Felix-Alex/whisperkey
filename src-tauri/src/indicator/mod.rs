use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IndicatorState {
    Idle,
    Recording,
    Processing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorEvent {
    pub state: IndicatorState,
    pub mode: Option<String>,
    pub level: Option<f32>,
}

pub struct Indicator {
    visible: bool,
}

impl Indicator {
    pub fn new() -> Self {
        Self { visible: false }
    }

    pub fn show(&mut self, _mode: &str) {
        self.visible = true;
        // Show the indicator window via Tauri window API
    }

    pub fn hide(&mut self) {
        self.visible = false;
        // Hide the indicator window
    }

    pub fn set_state(&self, _state: IndicatorState) {
        // Emit event to indicator window frontend
    }

    pub fn update_level(&self, _level: f32) {
        // Emit audio level to indicator window frontend
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

impl Default for Indicator {
    fn default() -> Self {
        Self::new()
    }
}
