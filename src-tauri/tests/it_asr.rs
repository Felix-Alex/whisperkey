/// IT-03: ASR mock 200 response → text correctly parsed
/// IT-04: ASR 401 → returns E_ASR_AUTH immediately (no retry)
use whisperkey_lib::asr::r#trait::{AsrProvider, AsrResponse};
use whisperkey_lib::asr::openai::OpenAiAsr;
use whisperkey_lib::config::schema::AsrConfig;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn asr_config(base_url: String) -> AsrConfig {
    AsrConfig {
        provider: "openai".into(),
        api_key: "sk-test".into(),
        api_key_len: 7,
        api_secret: String::new(),
        api_secret_len: 0,
        base_url,
        model: "whisper-1".into(),
        language: "auto".into(),
    }
}

#[tokio::test]
async fn it03_asr_200_returns_text() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/audio/transcriptions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"text": "你好世界 Hello World"}),
        ))
        .expect(1)
        .mount(&server)
        .await;

    let provider = OpenAiAsr;
    let cfg = asr_config(server.uri());
    // Minimal valid WAV: 44-byte header + 2 bytes of silence
    let wav = {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&(46u32).to_le_bytes()); // 36 + 10 data
        buf.extend_from_slice(b"WAVE");
        buf.extend_from_slice(b"fmt ");
        buf.extend_from_slice(&16u32.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes()); // PCM
        buf.extend_from_slice(&1u16.to_le_bytes()); // mono
        buf.extend_from_slice(&16000u32.to_le_bytes()); // rate
        buf.extend_from_slice(&32000u32.to_le_bytes()); // byte rate
        buf.extend_from_slice(&2u16.to_le_bytes()); // block align
        buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        buf.extend_from_slice(b"data");
        buf.extend_from_slice(&2u32.to_le_bytes()); // data size
        buf.extend_from_slice(&[0u8, 0u8]); // 1 silent sample
        buf
    };

    let result = provider.transcribe(wav, &cfg).await;
    assert!(result.is_ok(), "Expected Ok, got: {result:?}");
    let resp: AsrResponse = result.unwrap();
    assert_eq!(resp.text, "你好世界 Hello World");
}

#[tokio::test]
async fn it04_asr_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/audio/transcriptions"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;

    let provider = OpenAiAsr;
    let cfg = asr_config(server.uri());
    let wav = {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&(46u32).to_le_bytes());
        buf.extend_from_slice(b"WAVE");
        buf.extend_from_slice(b"fmt ");
        buf.extend_from_slice(&16u32.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&16000u32.to_le_bytes());
        buf.extend_from_slice(&32000u32.to_le_bytes());
        buf.extend_from_slice(&2u16.to_le_bytes());
        buf.extend_from_slice(&16u16.to_le_bytes());
        buf.extend_from_slice(b"data");
        buf.extend_from_slice(&2u32.to_le_bytes());
        buf.extend_from_slice(&[0u8, 0u8]);
        buf
    };

    let result = provider.transcribe(wav, &cfg).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), "E_ASR_AUTH");
}
