use async_trait::async_trait;
use crate::asr::r#trait::{AsrProvider, AsrRequest, AsrResponse};
use crate::error::{AppError, AppResult};

pub struct OfficialAsr {
    pub device_fingerprint: String,
}

#[async_trait]
impl AsrProvider for OfficialAsr {
    fn name(&self) -> &'static str { "official" }

    async fn transcribe(&self, req: AsrRequest) -> AppResult<AsrResponse> {
        let url = "https://api.whisperkey.app/v1/transcribe";
        let part = reqwest::multipart::Part::bytes(req.audio_wav)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|_| AppError::Internal)?;

        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("language", req.language.clone());

        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .header("X-Device-Fingerprint", &self.device_fingerprint)
            .multipart(form)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() { AppError::AsrTimeout } else { AppError::Network }
            })?;

        let status = resp.status();
        if status == 429 {
            return Err(AppError::AsrQuota);
        }
        if !status.is_success() {
            return Err(AppError::Internal);
        }

        let body: serde_json::Value = resp.json().await.map_err(|_| AppError::Internal)?;
        let text = body["text"].as_str().unwrap_or("").to_string();
        Ok(AsrResponse { text, duration_ms: 0 })
    }
}
