use async_trait::async_trait;
use crate::asr::r#trait::{AsrProvider, AsrRequest, AsrResponse};
use crate::error::{AppError, AppResult};

pub struct XfyunAsr;

#[async_trait]
impl AsrProvider for XfyunAsr {
    fn name(&self) -> &'static str { "xfyun" }

    async fn transcribe(&self, _req: AsrRequest) -> AppResult<AsrResponse> {
        let client = reqwest::Client::new();
        let url = "https://iat-api.xfyun.cn/v2/iat";
        let resp = client
            .post(url)
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() { AppError::AsrTimeout } else { AppError::Network }
            })?;

        let status = resp.status();
        if status == 401 || status == 403 {
            return Err(AppError::AsrAuth);
        }
        if status == 429 {
            return Err(AppError::AsrQuota);
        }

        let body: serde_json::Value = resp.json().await.map_err(|_| AppError::Internal)?;
        let text = body["data"]["text"].as_str().unwrap_or("").to_string();
        Ok(AsrResponse { text, duration_ms: 0 })
    }
}
