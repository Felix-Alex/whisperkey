use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("快捷键被占用，请重新设置")]
    HotkeyRegister,

    #[error("快捷键必须包含至少一个修饰键（Ctrl/Alt/Shift/Win），且为 2-3 键组合")]
    HotkeyInvalid,

    #[error("未检测到麦克风")]
    AudioDevice,

    #[error("请到系统设置授予麦克风权限")]
    AudioPermission,

    #[error("识别超时，请检查网络")]
    AsrTimeout,

    #[error("API Key 无效")]
    AsrAuth,

    #[error("调用频率/配额超限")]
    AsrQuota,

    #[error("AI 处理超时")]
    LlmTimeout,

    #[error("API Key 无效")]
    LlmAuth,

    #[error("调用频率/配额超限")]
    LlmQuota,

    #[error("已复制到剪贴板，请手动 Ctrl+V")]
    InjectFailed,

    #[error("激活码无效")]
    LicenseInvalid,

    #[error("许可证不属于当前设备")]
    LicenseDevice,

    #[error("该模式需激活后使用")]
    LicenseRequired,

    #[error("网络连接失败")]
    Network,

    #[error("内部错误，请查看日志")]
    Internal,
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::HotkeyRegister => "E_HOTKEY_REGISTER",
            AppError::HotkeyInvalid => "E_HOTKEY_INVALID",
            AppError::AudioDevice => "E_AUDIO_DEVICE",
            AppError::AudioPermission => "E_AUDIO_PERMISSION",
            AppError::AsrTimeout => "E_ASR_TIMEOUT",
            AppError::AsrAuth => "E_ASR_AUTH",
            AppError::AsrQuota => "E_ASR_QUOTA",
            AppError::LlmTimeout => "E_LLM_TIMEOUT",
            AppError::LlmAuth => "E_LLM_AUTH",
            AppError::LlmQuota => "E_LLM_QUOTA",
            AppError::InjectFailed => "E_INJECT_FAILED",
            AppError::LicenseInvalid => "E_LICENSE_INVALID",
            AppError::LicenseDevice => "E_LICENSE_DEVICE",
            AppError::LicenseRequired => "E_LICENSE_REQUIRED",
            AppError::Network => "E_NETWORK",
            AppError::Internal => "E_INTERNAL",
        }
    }

    pub fn message(&self) -> &'static str {
        // thiserror::Error provides .to_string(), but for serialization we match
        match self {
            AppError::HotkeyRegister => "快捷键被占用，请重新设置",
            AppError::HotkeyInvalid => "快捷键必须包含至少一个修饰键（Ctrl/Alt/Shift/Win），且为 2-3 键组合",
            AppError::AudioDevice => "未检测到麦克风",
            AppError::AudioPermission => "请到系统设置授予麦克风权限",
            AppError::AsrTimeout => "识别超时，请检查网络",
            AppError::AsrAuth => "API Key 无效",
            AppError::AsrQuota => "调用频率/配额超限",
            AppError::LlmTimeout => "AI 处理超时",
            AppError::LlmAuth => "API Key 无效",
            AppError::LlmQuota => "调用频率/配额超限",
            AppError::InjectFailed => "已复制到剪贴板，请手动 Ctrl+V",
            AppError::LicenseInvalid => "激活码无效",
            AppError::LicenseDevice => "许可证不属于当前设备",
            AppError::LicenseRequired => "该模式需激活后使用",
            AppError::Network => "网络连接失败",
            AppError::Internal => "内部错误，请查看日志",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", self.message())?;
        s.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_register_serialization() {
        let err = AppError::HotkeyRegister;
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(
            json,
            r#"{"code":"E_HOTKEY_REGISTER","message":"快捷键被占用，请重新设置"}"#
        );
    }

    #[test]
    fn test_all_error_variants_serialize_to_code_message() {
        let variants: Vec<AppError> = vec![
            AppError::HotkeyRegister,
            AppError::HotkeyInvalid,
            AppError::AudioDevice,
            AppError::AudioPermission,
            AppError::AsrTimeout,
            AppError::AsrAuth,
            AppError::AsrQuota,
            AppError::LlmTimeout,
            AppError::LlmAuth,
            AppError::LlmQuota,
            AppError::InjectFailed,
            AppError::LicenseInvalid,
            AppError::LicenseDevice,
            AppError::LicenseRequired,
            AppError::Network,
            AppError::Internal,
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            let obj = parsed.as_object().unwrap();
            assert!(obj.contains_key("code"), "Missing 'code' in {}", json);
            assert!(obj.contains_key("message"), "Missing 'message' in {}", json);
            assert_eq!(obj.len(), 2);
        }
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            AppError::HotkeyRegister.to_string(),
            "快捷键被占用，请重新设置"
        );
        assert_eq!(AppError::Network.to_string(), "网络连接失败");
    }
}
