use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::schema::AsrConfig;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrResponse {
    pub text: String,
    pub confidence: f32,
}

#[async_trait]
pub trait AsrProvider: Send + Sync {
    /// Transcribe WAV audio bytes to text using the given ASR configuration.
    async fn transcribe(&self, wav: Vec<u8>, config: &AsrConfig) -> AppResult<AsrResponse>;

    fn name(&self) -> &'static str;
}
