use async_trait::async_trait;
use reqwest::multipart;

use crate::asr::r#trait::{AsrProvider, AsrResponse};
use crate::config::schema::AsrConfig;
use crate::error::{AppError, AppResult};

pub struct OfficialAsr;

#[async_trait]
impl AsrProvider for OfficialAsr {
    fn name(&self) -> &'static str {
        "WhisperKey 官方中转"
    }

    async fn transcribe(&self, wav: Vec<u8>, config: &AsrConfig) -> AppResult<AsrResponse> {
        let url = format!("{}/v1/transcribe", config.base_url.trim_end_matches('/'));

        let part = multipart::Part::bytes(wav)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|_| AppError::Internal)?;

        let form = multipart::Form::new()
            .part("file", part)
            .text("mode", "raw");

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("X-Device-Id", "placeholder-fingerprint") // will be wired to real fingerprint in integration
            .header("X-Client-Version", env!("CARGO_PKG_VERSION"))
            .multipart(form)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AppError::AsrTimeout
                } else {
                    AppError::Network
                }
            })?;

        match resp.status().as_u16() {
            200 => {
                let body: serde_json::Value = resp.json().await.map_err(|_| AppError::Internal)?;
                let text = body["text"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                Ok(AsrResponse {
                    text,
                    confidence: 0.0,
                })
            }
            401 | 403 => Err(AppError::AsrAuth),
            429 => Err(AppError::AsrQuota),
            _ => Err(AppError::Network),
        }
    }
}
