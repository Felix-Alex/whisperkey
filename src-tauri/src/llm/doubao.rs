use async_trait::async_trait;
use crate::error::AppResult;
use crate::llm::openai::llm_openai_compat;
use crate::llm::r#trait::{LlmProvider, LlmRequest, LlmResponse};

pub struct DoubaoLlm;

#[async_trait]
impl LlmProvider for DoubaoLlm {
    fn name(&self) -> &'static str { "doubao" }
    async fn complete(&self, req: LlmRequest) -> AppResult<LlmResponse> {
        llm_openai_compat(&req, "https://ark.cn-beijing.volces.com/api/v3").await
    }
}
