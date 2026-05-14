/// IT-05/IT-06 foundation: LLM provider chat endpoint integration
/// Tests the ASR → LLM HTTP chain with wiremock
use whisperkey_lib::llm::openai::OpenAiCompatibleLlm;
use whisperkey_lib::llm::r#trait::LlmProvider;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn it05_llm_200_returns_content() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({
                "choices": [{
                    "message": {
                        "content": "这是优化后的书面语文本。"
                    }
                }]
            }),
        ))
        .expect(1)
        .mount(&server)
        .await;

    let provider = OpenAiCompatibleLlm;
    let result = provider
        .chat(
            "你是一个口语转书面语优化器。",
            "嗯那个就是说我想去吃饭然后",
            "sk-test",
            &server.uri(),
            "gpt-4o-mini",
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "这是优化后的书面语文本。");
}

#[tokio::test]
async fn it06_llm_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;

    let provider = OpenAiCompatibleLlm;
    let result = provider
        .chat(
            "系统提示词",
            "用户输入文本",
            "sk-bad-key",
            &server.uri(),
            "gpt-4o-mini",
        )
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), "E_LLM_AUTH");
}

#[tokio::test]
async fn it06_llm_429_returns_quota_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(429))
        .expect(1)
        .mount(&server)
        .await;

    let provider = OpenAiCompatibleLlm;
    let result = provider
        .chat("system", "user text", "sk-test", &server.uri(), "gpt-4o-mini")
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), "E_LLM_QUOTA");
}
