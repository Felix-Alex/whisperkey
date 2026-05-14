use async_trait::async_trait;

use crate::asr::r#trait::{AsrProvider, AsrResponse};
use crate::config::schema::AsrConfig;
use crate::error::{AppError, AppResult};

pub struct XfyunAsr;

#[async_trait]
impl AsrProvider for XfyunAsr {
    fn name(&self) -> &'static str {
        "讯飞极速听写"
    }

    async fn transcribe(&self, wav: Vec<u8>, config: &AsrConfig) -> AppResult<AsrResponse> {
        // Xfyun ASR uses HmacSHA1(appId + ts) signature + multipart upload
        // The api_key field holds appId, and base_url holds the API endpoint
        let api_key = &config.api_key;
        if api_key.is_empty() {
            return Err(AppError::AsrAuth);
        }

        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();

        // Simple signature: SHA1(appId + ts) — Xfyun uses this for auth
        let sign_input = format!("{api_key}{ts}");
        let signa = {
            use ring::digest;
            let hash = digest::digest(&digest::SHA1_FOR_LEGACY_USE_ONLY, sign_input.as_bytes());
            hash.as_ref().iter().map(|b| format!("{b:02x}")).collect::<String>()
        };

        let base = config.base_url.trim_end_matches('/');
        let url = format!(
            "{}/v2/api/upload?signa={}&ts={}&appId={}&fileSize={}&duration={}",
            base,
            signa,
            ts,
            api_key,
            wav.len(),
            wav.len() / 32, // rough duration estimate for 16kHz 16bit mono
        );

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .body(wav)
            .header("Content-Type", "application/octet-stream")
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
                let text = body["data"]
                    .as_str()
                    .or_else(|| body["text"].as_str())
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
