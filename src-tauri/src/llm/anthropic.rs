use async_trait::async_trait;
use crate::error::{AppError, AppResult};
use crate::llm::r#trait::{LlmProvider, LlmRequest, LlmResponse};

pub struct AnthropicLlm;

#[async_trait]
impl LlmProvider for AnthropicLlm {
    fn name(&self) -> &'static str { "anthropic" }

    async fn complete(&self, req: LlmRequest) -> AppResult<LlmResponse> {
        let base_url = req.base_url.as_deref().unwrap_or("https://api.anthropic.com");
        let url = format!("{base_url}/v1/messages");

        let body = serde_json::json!({
            "model": req.model,
            "max_tokens": 4096,
            "messages": [{"role": "user", "content": req.prompt}],
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("x-api-key", &req.api_key)
            .header("anthropic-version", "2023-06-01")
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
        let text = body["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(LlmResponse { text, tokens_used: 0 })
    }
}
