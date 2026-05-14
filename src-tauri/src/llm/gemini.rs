use async_trait::async_trait;
use serde_json::json;

use crate::error::{AppError, AppResult};
use crate::llm::r#trait::LlmProvider;

pub struct GeminiLlm;

#[async_trait]
impl LlmProvider for GeminiLlm {
    fn name(&self) -> &'static str {
        "Google Gemini"
    }

    async fn chat(
        &self,
        system_prompt: &str,
        user_text: &str,
        api_key: &str,
        base_url: &str,
        model: &str,
    ) -> AppResult<String> {
        let url = format!(
            "{}/v1beta/models/{model}:generateContent?key={api_key}",
            base_url.trim_end_matches('/'),
        );

        let body = json!({
            "system_instruction": {
                "parts": [{"text": system_prompt}]
            },
            "contents": [
                {
                    "role": "user",
                    "parts": [{"text": user_text}]
                }
            ],
            "generationConfig": {
                "temperature": 0.2
            }
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
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
                let text = body["candidates"][0]["content"]["parts"][0]["text"]
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
