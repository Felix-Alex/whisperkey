use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub hotkey: HotkeyConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub asr: AsrConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub system: SystemConfig,
    #[serde(default)]
    pub history: HistoryConfig,
    #[serde(default)]
    pub advanced: AdvancedConfig,
    #[serde(default = "default_output_mode")]
    pub output_mode: String,
}

fn default_version() -> u32 {
    2
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: default_version(),
            hotkey: HotkeyConfig::default(),
            llm: LlmConfig::default(),
            asr: AsrConfig::default(),
            audio: AudioConfig::default(),
            ui: UiConfig::default(),
            system: SystemConfig::default(),
            history: HistoryConfig::default(),
            advanced: AdvancedConfig::default(),
            output_mode: default_output_mode(),
        }
    }
}

// ── Hotkey ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    #[serde(default = "default_modifiers")]
    pub modifiers: Vec<String>,
    #[serde(default = "default_key")]
    pub key: String,
    #[serde(default)]
    pub paused: bool,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifiers: default_modifiers(),
            key: default_key(),
            paused: false,
        }
    }
}

fn default_modifiers() -> Vec<String> {
    vec!["Alt".into()]
}

fn default_key() -> String {
    "J".into()
}

// ── LLM (single global provider) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    #[serde(default = "default_llm_provider")]
    pub provider: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_key: String,
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub api_key_len: usize,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_secret: String,
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub api_secret_len: usize,
    #[serde(default = "default_openai_base_url")]
    pub base_url: String,
    #[serde(default = "default_llm_model")]
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_llm_provider(),
            api_key: String::new(),
            api_key_len: 0,
            api_secret: String::new(),
            api_secret_len: 0,
            base_url: default_openai_base_url(),
            model: default_llm_model(),
        }
    }
}

fn default_llm_provider() -> String {
    "openai".into()
}

fn default_llm_model() -> String {
    "gpt-4o-mini".into()
}

// ── ASR ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsrConfig {
    #[serde(default = "default_asr_provider")]
    pub provider: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_key: String,
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub api_key_len: usize,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_secret: String,
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub api_secret_len: usize,
    #[serde(default = "default_openai_base_url")]
    pub base_url: String,
    #[serde(default = "default_asr_model")]
    pub model: String,
    #[serde(default = "default_language")]
    pub language: String,
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            provider: default_asr_provider(),
            api_key: String::new(),
            api_key_len: 0,
            api_secret: String::new(),
            api_secret_len: 0,
            base_url: default_openai_base_url(),
            model: default_asr_model(),
            language: default_language(),
        }
    }
}

fn default_asr_provider() -> String {
    "openai".into()
}

fn default_asr_model() -> String {
    "whisper-1".into()
}

fn default_language() -> String {
    "auto".into()
}

fn default_openai_base_url() -> String {
    "https://api.openai.com/v1".into()
}

// ── Audio ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioConfig {
    #[serde(default = "default_max_duration")]
    pub max_duration_sec: u32,
    #[serde(default)]
    pub silence_auto_stop: bool,
    #[serde(default = "default_silence_timeout")]
    pub silence_timeout_ms: u32,
    #[serde(default = "default_input_device")]
    pub input_device: String,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            max_duration_sec: default_max_duration(),
            silence_auto_stop: false,
            silence_timeout_ms: default_silence_timeout(),
            input_device: default_input_device(),
        }
    }
}

fn default_max_duration() -> u32 {
    60
}

fn default_silence_timeout() -> u32 {
    3000
}

fn default_input_device() -> String {
    "default".into()
}

// ── UI ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_lang")]
    pub language: String,
    #[serde(default = "default_indicator_position")]
    pub indicator_position: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            language: default_lang(),
            indicator_position: default_indicator_position(),
        }
    }
}

fn default_theme() -> String {
    "auto".into()
}

fn default_lang() -> String {
    "zh-CN".into()
}

fn default_indicator_position() -> String {
    "bottom-center".into()
}

// ── System ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemConfig {
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    #[serde(default = "default_true")]
    pub check_updates: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            auto_start: false,
            minimize_to_tray: true,
            check_updates: true,
        }
    }
}

fn default_true() -> bool {
    true
}

// ── History ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_retention")]
    pub retention_days: u32,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: default_retention(),
        }
    }
}

fn default_retention() -> u32 {
    7
}

// ── Advanced ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub telemetry: bool,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            telemetry: false,
        }
    }
}

fn default_log_level() -> String {
    "info".into()
}

fn default_output_mode() -> String {
    "raw".into()
}

fn is_zero_usize(n: &usize) -> bool {
    *n == 0
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_version_2() {
        let cfg = Config::default();
        assert_eq!(cfg.version, 2, "default config must be version 2");
    }

    #[test]
    fn test_default_config_fields() {
        let cfg = Config::default();
        let json = serde_json::to_string_pretty(&cfg).unwrap();

        // v2 key fields
        assert!(json.contains("\"version\": 2"));
        assert!(json.contains("\"modifiers\""));
        assert!(json.contains("\"key\": \"J\""));
        // LLM global config
        assert!(json.contains("\"llm\""));
        assert!(json.contains("\"provider\": \"openai\""));
        assert!(json.contains("\"model\": \"gpt-4o-mini\""));
        // ASR config
        assert!(json.contains("\"asr\""));
        assert!(json.contains("\"whisper-1\""));
        assert!(json.contains("\"language\": \"auto\""));
        // Other sections preserved
        assert!(json.contains("\"maxDurationSec\": 60"));
        assert!(json.contains("\"theme\": \"auto\""));
        assert!(json.contains("\"retentionDays\": 7"));
        assert!(json.contains("\"logLevel\": \"info\""));
    }

    #[test]
    fn test_default_config_roundtrip() {
        let cfg = Config::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, 2);
        assert_eq!(parsed.hotkey.modifiers, vec!["Alt"]);
        assert_eq!(parsed.hotkey.key, "J");
        assert_eq!(parsed.llm.provider, "openai");
        assert_eq!(parsed.llm.model, "gpt-4o-mini");
        assert_eq!(parsed.asr.provider, "openai");
        assert_eq!(parsed.asr.model, "whisper-1");
        assert_eq!(parsed.asr.language, "auto");
        assert_eq!(parsed.audio.max_duration_sec, 60);
    }

    #[test]
    fn test_camelcase_serialization() {
        let cfg = Config::default();
        let json = serde_json::to_string(&cfg).unwrap();
        // camelCase keys must be present
        assert!(json.contains("maxDurationSec"));
        assert!(json.contains("silenceTimeoutMs"));
        assert!(json.contains("silenceAutoStop"));
        assert!(json.contains("inputDevice"));
        assert!(json.contains("indicatorPosition"));
        assert!(json.contains("autoStart"));
        assert!(json.contains("minimizeToTray"));
        assert!(json.contains("checkUpdates"));
        assert!(json.contains("retentionDays"));
        assert!(json.contains("logLevel"));
        // snake_case keys must NOT be present in serialized output
        assert!(!json.contains("max_duration_sec"));
        assert!(!json.contains("indicator_position"));
    }

    #[test]
    fn test_llm_config_camelcase() {
        let mut cfg = Config::default();
        cfg.llm.api_key = "sk-test".into();
        cfg.llm.base_url = "https://custom.api.com/v1".into();
        let json = serde_json::to_string(&cfg).unwrap();
        assert!(json.contains("apiKey"));
        assert!(json.contains("baseUrl"));
        // snake_case must not appear
        assert!(!json.contains("api_key"));
        assert!(!json.contains("base_url"));
    }

    #[test]
    fn test_asr_config_camelcase() {
        let mut cfg = Config::default();
        cfg.asr.api_key = "asr-key".into();
        let json = serde_json::to_string(&cfg).unwrap();
        // ASR section uses camelCase
        assert!(json.contains("\"asr\""));
        assert!(json.contains("apiKey"));
    }

    #[test]
    fn test_old_modes_and_providers_absent() {
        let cfg = Config::default();
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        // These old v1 keys must NOT appear
        assert!(!json.contains("modes"));
        assert!(!json.contains("providers"));
        assert!(!json.contains("ModeAssignment"));
        assert!(!json.contains("llmProvider"));
        assert!(!json.contains("llmModel"));
    }

    #[test]
    fn test_empty_api_key_skipped() {
        let cfg = Config::default();
        let json = serde_json::to_string(&cfg).unwrap();
        // Default empty apiKey should be skipped (clean JSON)
        assert!(!json.contains("\"apiKey\": \"\""));
    }

    #[test]
    fn test_can_deserialize_minimal_v2_json() {
        let minimal = r#"{"version":2}"#;
        let cfg: Config = serde_json::from_str(minimal).unwrap();
        assert_eq!(cfg.version, 2);
        assert_eq!(cfg.llm.provider, "openai");
        assert_eq!(cfg.asr.language, "auto");
    }
}
