use async_trait::async_trait;
use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct AsrRequest {
    pub audio_wav: Vec<u8>,
    pub language: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AsrResponse {
    pub text: String,
    pub duration_ms: u64,
}

#[async_trait]
pub trait AsrProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn transcribe(&self, req: AsrRequest) -> AppResult<AsrResponse>;
}
