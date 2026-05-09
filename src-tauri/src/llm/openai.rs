use async_trait::async_trait;
use crate::error::{AppError, AppResult};
use crate::llm::r#trait::{LlmProvider, LlmRequest, LlmResponse};

pub struct OpenAiLlm;

#[async_trait]
impl LlmProvider for OpenAiLlm {
    fn name(&self) -> &'static str { "openai" }

    async fn complete(&self, req: LlmRequest) -> AppResult<LlmResponse> {
        llm_openai_compat(&req, "https://api.openai.com/v1").await
    }
}

// Shared OpenAI-compatible API implementation
pub async fn llm_openai_compat(req: &LlmRequest, default_base: &str) -> AppResult<LlmResponse> {
    let base_url = req.base_url.as_deref().unwrap_or(default_base);
    let url = format!("{base_url}/chat/completions");

    let body = serde_json::json!({
        "model": req.model,
        "messages": [{"role": "user", "content": req.prompt}],
        "max_tokens": 4096,
        "temperature": 0.3,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .bearer_auth(&req.api_key)
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
    let text = body["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let tokens = body["usage"]["total_tokens"].as_u64().unwrap_or(0);

    Ok(LlmResponse {
        text,
        tokens_used: tokens,
    })
}
