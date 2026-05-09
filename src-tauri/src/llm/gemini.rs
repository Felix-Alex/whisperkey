use async_trait::async_trait;
use crate::error::{AppError, AppResult};
use crate::llm::r#trait::{LlmProvider, LlmRequest, LlmResponse};

pub struct GeminiLlm;

#[async_trait]
impl LlmProvider for GeminiLlm {
    fn name(&self) -> &'static str { "gemini" }

    async fn complete(&self, req: LlmRequest) -> AppResult<LlmResponse> {
        let base_url = req.base_url.as_deref().unwrap_or("https://generativelanguage.googleapis.com");
        let url = format!("{base_url}/v1beta/models/{}:generateContent", req.model);

        let body = serde_json::json!({
            "contents": [{"parts": [{"text": req.prompt}]}],
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .query(&[("key", req.api_key.as_str())])
            .json(&body)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() { AppError::LlmTimeout } else { AppError::Network }
            })?;

        let status = resp.status();
        if status == 401 || status == 403 {
            return Err(AppError::LlmAuth);
        }

        let body: serde_json::Value = resp.json().await.map_err(|_| AppError::Internal)?;
        let text = body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(LlmResponse { text, tokens_used: 0 })
    }
}
