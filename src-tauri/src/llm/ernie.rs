use async_trait::async_trait;
use crate::error::{AppError, AppResult};
use crate::llm::r#trait::{LlmProvider, LlmRequest, LlmResponse};

pub struct ErnieLlm;

#[async_trait]
impl LlmProvider for ErnieLlm {
    fn name(&self) -> &'static str { "ernie" }

    async fn complete(&self, req: LlmRequest) -> AppResult<LlmResponse> {
        let url = "https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/completions";

        let body = serde_json::json!({
            "model": req.model,
            "messages": [{"role": "user", "content": req.prompt}],
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .query(&[("access_token", req.api_key.as_str())])
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
        let text = body["result"].as_str().unwrap_or("").to_string();

        Ok(LlmResponse { text, tokens_used: 0 })
    }
}
