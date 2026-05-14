use async_trait::async_trait;
use serde_json::json;

use crate::error::{AppError, AppResult};
use crate::llm::r#trait::LlmProvider;

pub struct OpenAiCompatibleLlm;

#[async_trait]
impl LlmProvider for OpenAiCompatibleLlm {
    fn name(&self) -> &'static str {
        "OpenAI Compatible"
    }

    async fn chat(
        &self,
        system_prompt: &str,
        user_text: &str,
        api_key: &str,
        base_url: &str,
        model: &str,
    ) -> AppResult<String> {
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
        tracing::info!("[LLM openai] calling {} model={}", url, model);

        let body = json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_text}
            ],
            "temperature": 0.2,
            "max_tokens": 2048
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
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
                let resp_text = resp.text().await.map_err(|e| {
                    tracing::error!("[LLM] resp.text() failed: {e:?}");
                    AppError::Internal
                })?;
                let body: serde_json::Value =
                    serde_json::from_str(&resp_text).map_err(|e| {
                        tracing::error!(
                            "[LLM] JSON parse failed: {e:?}, body preview: {}",
                            &resp_text[..resp_text.len().min(500)]
                        );
                        AppError::Internal
                    })?;
                let text = body["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                Ok(text)
            }
            401 | 403 => Err(AppError::LlmAuth),
            429 => Err(AppError::LlmQuota),
            _ => {
                let status = resp.status().as_u16();
                let body_preview = resp.text().await.unwrap_or_default();
                tracing::error!(
                    "[LLM] HTTP {status}, body preview: {}",
                    &body_preview[..body_preview.len().min(500)]
                );
                Err(AppError::Network)
            }
        }
    }
}
