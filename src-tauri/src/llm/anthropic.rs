use async_trait::async_trait;
use serde_json::json;

use crate::error::{AppError, AppResult};
use crate::llm::r#trait::LlmProvider;

pub struct AnthropicLlm;

#[async_trait]
impl LlmProvider for AnthropicLlm {
    fn name(&self) -> &'static str {
        "Anthropic Claude"
    }

    async fn chat(
        &self,
        system_prompt: &str,
        user_text: &str,
        api_key: &str,
        base_url: &str,
        model: &str,
    ) -> AppResult<String> {
        let url = format!("{}/v1/messages", base_url.trim_end_matches('/'));

        let body = json!({
            "model": model,
            "max_tokens": 2048,
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_text}
            ]
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AppError::LlmTimeout
                } else {
                    AppError::Network
                }
            })?;

        match resp.status().as_u16() {
            200 => {
                let body: serde_json::Value = resp.json().await.map_err(|_| AppError::Internal)?;
                let text = body["content"][0]["text"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                Ok(text)
            }
            401 | 403 => Err(AppError::LlmAuth),
            429 => Err(AppError::LlmQuota),
            _ => Err(AppError::Network),
        }
    }
}
