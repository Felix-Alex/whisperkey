use async_trait::async_trait;
use crate::asr::r#trait::{AsrProvider, AsrRequest, AsrResponse};
use crate::error::{AppError, AppResult};

pub struct OpenAiAsr;

#[async_trait]
impl AsrProvider for OpenAiAsr {
    fn name(&self) -> &'static str {
        "openai"
    }

    async fn transcribe(&self, req: AsrRequest) -> AppResult<AsrResponse> {
        let base_url = req.base_url.as_deref().unwrap_or("https://api.openai.com/v1");
        let url = format!("{base_url}/audio/transcriptions");

        let part = reqwest::multipart::Part::bytes(req.audio_wav)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|_| AppError::Internal)?;

        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", "whisper-1")
            .text("language", req.language.clone());

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .bearer_auth(&req.api_key)
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

        let status = resp.status();
        if status == 401 || status == 403 {
            return Err(AppError::AsrAuth);
        }
        if status == 429 {
            return Err(AppError::AsrQuota);
        }
        if !status.is_success() {
            // Retry once on server errors
            return Err(AppError::Internal);
        }

        let body: serde_json::Value = resp.json().await.map_err(|_| AppError::Internal)?;
        let text = body["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(AsrResponse {
            text,
            duration_ms: 0,
        })
    }
}
