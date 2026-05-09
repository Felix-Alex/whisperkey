use async_trait::async_trait;
use crate::error::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmMode {
    Raw,
    Polish,
    Markdown,
}

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub mode: LlmMode,
    pub raw_text: String,
    pub prompt: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub text: String,
    pub tokens_used: u64,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn complete(&self, req: LlmRequest) -> AppResult<LlmResponse>;
}
