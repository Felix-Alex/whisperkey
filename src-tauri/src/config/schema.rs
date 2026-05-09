use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub hotkey: HotkeyConfig,
    #[serde(default)]
    pub modes: ModesConfig,
    #[serde(default)]
    pub asr: AsrConfig,
    #[serde(default)]
    pub providers: ProvidersConfig,
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
}

fn default_version() -> u32 {
    1
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            hotkey: HotkeyConfig::default(),
            modes: ModesConfig::default(),
            asr: AsrConfig::default(),
            providers: ProvidersConfig::default(),
            audio: AudioConfig::default(),
            ui: UiConfig::default(),
            system: SystemConfig::default(),
            history: HistoryConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }
}

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
    vec!["Ctrl".into(), "Shift".into()]
}

fn default_key() -> String {
    "Space".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModesConfig {
    #[serde(default = "default_mode")]
    pub default: String,
    #[serde(default)]
    pub raw: ModeAssignment,
    #[serde(default)]
    pub polish: ModeAssignment,
    #[serde(default)]
    pub markdown: ModeAssignment,
}

fn default_mode() -> String {
    "raw".into()
}

impl Default for ModesConfig {
    fn default() -> Self {
        Self {
            default: default_mode(),
            raw: ModeAssignment {
                llm_provider: "official".into(),
                llm_model: "default".into(),
            },
            polish: ModeAssignment {
                llm_provider: "deepseek".into(),
                llm_model: "deepseek-chat".into(),
            },
            markdown: ModeAssignment {
                llm_provider: "anthropic".into(),
                llm_model: "claude-3-5-haiku-20241022".into(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeAssignment {
    pub llm_provider: String,
    pub llm_model: String,
}

impl Default for ModeAssignment {
    fn default() -> Self {
        Self {
            llm_provider: "official".into(),
            llm_model: "default".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsrConfig {
    #[serde(default = "default_asr_provider")]
    pub default: String,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_asr_provider() -> String {
    "official".into()
}

fn default_language() -> String {
    "auto".into()
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            default: default_asr_provider(),
            language: default_language(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProvidersConfig {
    #[serde(default)]
    pub openai: ProviderCredential,
    #[serde(default)]
    pub anthropic: ProviderCredential,
    #[serde(default)]
    pub deepseek: ProviderCredential,
    #[serde(default)]
    pub qwen: ProviderCredential,
    #[serde(default)]
    pub ernie: ErnieCredential,
    #[serde(default)]
    pub doubao: DoubaoCredential,
    #[serde(default)]
    pub gemini: ProviderCredential,
    #[serde(default)]
    pub xfyun: XfyunCredential,
    #[serde(default)]
    pub volcengine: VolcengineCredential,
    #[serde(default)]
    pub official: OfficialCredential,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCredential {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_key: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ErnieCredential {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_key: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DoubaoCredential {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_key: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub endpoint_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XfyunCredential {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub app_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_key: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VolcengineCredential {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub app_key: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub access_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OfficialCredential {}

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
            max_duration_sec: 60,
            silence_auto_stop: false,
            silence_timeout_ms: 3000,
            input_device: "default".into(),
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
            theme: "auto".into(),
            language: "zh-CN".into(),
            indicator_position: "bottom-center".into(),
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
            retention_days: 7,
        }
    }
}

fn default_retention() -> u32 {
    7
}

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
            log_level: "info".into(),
            telemetry: false,
        }
    }
}

fn default_log_level() -> String {
    "info".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_fields() {
        let cfg = Config::default();
        let json = serde_json::to_string_pretty(&cfg).unwrap();

        // Verify key fields from TECH_SPEC §6.1 are present
        assert!(json.contains("\"version\": 1"));
        assert!(json.contains("\"modifiers\""));
        assert!(json.contains("\"Ctrl\""));
        assert!(json.contains("\"Shift\""));
        assert!(json.contains("\"key\": \"Space\""));
        assert!(json.contains("\"default\": \"raw\""));
        assert!(json.contains("\"maxDurationSec\": 60"));
        assert!(json.contains("\"silenceTimeoutMs\": 3000"));
        assert!(json.contains("\"theme\": \"auto\""));
        assert!(json.contains("\"language\": \"zh-CN\""));
        assert!(json.contains("\"logLevel\": \"info\""));
        assert!(json.contains("\"retentionDays\": 7"));
    }

    #[test]
    fn test_default_config_roundtrip() {
        let cfg = Config::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.hotkey.modifiers, vec!["Ctrl", "Shift"]);
        assert_eq!(parsed.hotkey.key, "Space");
        assert_eq!(parsed.audio.max_duration_sec, 60);
        assert_eq!(parsed.modes.default, "raw");
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
        // snake_case keys must NOT be present
        assert!(!json.contains("max_duration_sec"));
        assert!(!json.contains("silence_timeout_ms"));
        assert!(!json.contains("log_level"));
    }

    #[test]
    fn test_provider_camelcase_serialization() {
        let mut cfg = Config::default();
        cfg.providers.openai.api_key = "test-key".into();
        cfg.providers.openai.base_url = "https://test.com".into();
        let json = serde_json::to_string(&cfg).unwrap();
        assert!(json.contains("apiKey"));
        assert!(json.contains("baseUrl"));
        assert!(json.contains("llmProvider"));
        assert!(json.contains("llmModel"));
    }
}
