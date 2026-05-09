use async_trait::async_trait;
use crate::error::AppResult;
use crate::llm::openai::llm_openai_compat;
use crate::llm::r#trait::{LlmProvider, LlmRequest, LlmResponse};

pub struct DeepSeekLlm;

#[async_trait]
impl LlmProvider for DeepSeekLlm {
    fn name(&self) -> &'static str { "deepseek" }
    async fn complete(&self, req: LlmRequest) -> AppResult<LlmResponse> {
        llm_openai_compat(&req, "https://api.deepseek.com/v1").await
    }
}
