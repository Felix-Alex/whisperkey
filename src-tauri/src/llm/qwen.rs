use async_trait::async_trait;
use crate::error::AppResult;
use crate::llm::openai::llm_openai_compat;
use crate::llm::r#trait::{LlmProvider, LlmRequest, LlmResponse};

pub struct QwenLlm;

#[async_trait]
impl LlmProvider for QwenLlm {
    fn name(&self) -> &'static str { "qwen" }
    async fn complete(&self, req: LlmRequest) -> AppResult<LlmResponse> {
        llm_openai_compat(&req, "https://dashscope.aliyuncs.com/compatible-mode/v1").await
    }
}
