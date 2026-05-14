use tauri::State;

use crate::app_state::AppState;
use crate::config::schema::Config;

#[tauri::command]
pub async fn cmd_config_get(state: State<'_, AppState>) -> Result<Config, String> {
    state.config_store.read().map(|c| c.clone()).map_err(|e| e.to_string())
}

/// Restart the OS-level hotkey listener (e.g. after user changes the binding).
#[tauri::command]
pub async fn cmd_hotkey_restart(state: State<'_, AppState>) -> Result<(), String> {
    state.restart_hotkey().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_config_set(config: Config, state: State<'_, AppState>) -> Result<(), String> {
    state.config_store.update(|c| *c = config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_llm_set_config(
    provider: String,
    api_key: String,
    api_secret: Option<String>,
    base_url: String,
    model: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !api_key.is_empty() {
        crate::config::secrets::set_llm_key(&state.config_store, &api_key).map_err(|e| e.to_string())?;
    }
    if let Some(ref secret) = api_secret {
        if !secret.is_empty() {
            crate::config::secrets::set_llm_secret(&state.config_store, secret).map_err(|e| e.to_string())?;
        }
    }
    state.config_store.update(|c| {
        c.llm.provider = provider;
        if !base_url.is_empty() { c.llm.base_url = base_url; }
        if !model.is_empty() { c.llm.model = model; }
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_asr_set_config(
    provider: String,
    api_key: String,
    api_secret: Option<String>,
    base_url: String,
    model: String,
    language: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !api_key.is_empty() {
        crate::config::secrets::set_asr_key(&state.config_store, &api_key).map_err(|e| e.to_string())?;
    }
    if let Some(ref secret) = api_secret {
        if !secret.is_empty() {
            crate::config::secrets::set_asr_secret(&state.config_store, secret).map_err(|e| e.to_string())?;
        }
    }
    state.config_store.update(|c| {
        c.asr.provider = provider;
        if !base_url.is_empty() { c.asr.base_url = base_url; }
        if !model.is_empty() { c.asr.model = model; }
        if !language.is_empty() { c.asr.language = language; }
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_config_save(state: State<'_, AppState>) -> Result<(), String> {
    state.config_store.save().map_err(|e| e.to_string())
}

/// Test LLM connection with a minimal chat completion request.
#[tauri::command]
pub async fn cmd_llm_test_connection(
    provider: String,
    api_key: String,
    _api_secret: Option<String>,
    base_url: String,
    model: String,
) -> Result<String, String> {
    if api_key.is_empty() {
        return Err("请先填写 API Key".into());
    }

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let test_model = if model.is_empty() { "gpt-4o-mini".into() } else { model };

    let body = serde_json::json!({
        "model": test_model,
        "messages": [{"role": "user", "content": "hi"}],
        "max_tokens": 1,
    });

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let mut req = client
        .post(&url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(15));

    match provider.as_str() {
        "anthropic" => {
            req = req
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01");
        }
        "gemini" => {
            let url = format!(
                "{}/v1beta/models/{}:generateContent?key={}",
                base_url.trim_end_matches('/'),
                test_model,
                api_key
            );
            let body = serde_json::json!({
                "contents": [{"role": "user", "parts": [{"text": "hi"}]}],
                "generationConfig": {"maxOutputTokens": 1},
            });
            req = client.post(&url).json(&body).timeout(std::time::Duration::from_secs(15));
        }
        _ => {
            req = req.header("Authorization", format!("Bearer {api_key}"));
        }
    }

    let resp = req.send().await.map_err(|e| {
        if e.is_timeout() { "连接超时，请检查网络或 Base URL 是否正确".into() }
        else if e.is_connect() { format!("无法连接到服务器: {e}") }
        else { format!("网络错误: {e}") }
    })?;

    let status = resp.status();
    // Always read body even on 2xx — some proxies return errors in 200 responses
    let body_text = resp.text().await.unwrap_or_default();

    if status == 401 || status == 403 {
        return Err("API Key 无效，请检查 Key 是否正确".into());
    }
    if status == 404 {
        return Err(format!("接口不存在 (404)，Base URL 或模型名可能有误 (model={test_model})"));
    }
    if status == 429 {
        return Err("请求频率超限 (429)，请稍后重试".into());
    }
    if !status.is_success() {
        let snippet: String = body_text.chars().take(200).collect();
        return Err(format!("服务器错误 (HTTP {status}): {snippet}"));
    }

    // HTTP 2xx — validate response body for embedded API errors
    let json: serde_json::Value = serde_json::from_str(&body_text).unwrap_or_default();
    if let Some(err) = json.get("error") {
        let msg = err.get("message")
            .and_then(|m| m.as_str())
            .or_else(|| err.as_str())
            .unwrap_or("unknown error");
        return Err(format!("API 返回错误: {msg}"));
    }
    // Verify response looks like a plausible chat completion
    let has_choices = json.get("choices").is_some();
    let has_candidates = json.get("candidates").is_some();
    let has_content = json.get("content").is_some();
    if !has_choices && !has_candidates && !has_content {
        let snippet: String = body_text.chars().take(200).collect();
        return Err(format!("服务器返回异常响应: {snippet}"));
    }

    Ok(format!("✓ 连接成功 — {provider} ({test_model})"))
}

/// Test ASR connection with a connectivity/auth check.
#[tauri::command(rename_all = "camelCase")]
pub async fn cmd_asr_test_connection(
    provider: String,
    api_key: String,
    api_secret: Option<String>,
    base_url: String,
    model: String,
) -> Result<String, String> {
    if provider == "official" {
        return Ok("✓ 官方中转服务 — 无需额外配置".into());
    }

    if api_key.is_empty() {
        return Err("请先填写 API Key / App Key".into());
    }

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    match provider.as_str() {
        "openai" => {
            let url = format!("{}/models", base_url.trim_end_matches('/'));
            let resp = client
                .get(&url)
                .header("Authorization", format!("Bearer {api_key}"))
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() { "连接超时".into() }
                    else if e.is_connect() { format!("无法连接到服务器: {e}") }
                    else { format!("网络错误: {e}") }
                })?;
            let http_status = resp.status().as_u16();
            // Always read body — some proxies return 200 with error JSON
            let resp_text = resp.text().await.unwrap_or_default();

            if http_status == 401 || http_status == 403 {
                return Err("API Key 无效".into());
            }
            if http_status == 429 {
                return Err("请求频率超限 (429)".into());
            }
            if http_status == 200 {
                // Verify response is actually a models list, not an error object
                let json: serde_json::Value =
                    serde_json::from_str(&resp_text).unwrap_or_default();
                if json.get("error").is_some() {
                    let msg = json["error"]["message"]
                        .as_str()
                        .unwrap_or("unknown error");
                    return Err(format!("API 返回错误: {msg}"));
                }
                if json.get("data").is_some() || json.get("object").is_some() {
                    return Ok("✓ 连接成功 — OpenAI Whisper".into());
                }
                // Unexpected 200 body — show snippet
                let snippet: String = resp_text.chars().take(200).collect();
                return Err(format!("服务器返回异常响应: {snippet}"));
            }
            let snippet: String = resp_text.chars().take(200).collect();
            Err(format!("服务器返回 HTTP {http_status}: {snippet}"))
        }
        "volcengine" => {
            // Volcengine v3 streaming ASR — 新版控制台只需 X-Api-Key
            // Auth is validated during WebSocket handshake (HTTP Upgrade):
            //   101 = credentials valid   |   401/403 = credentials invalid
            // Ref: https://www.volcengine.com/docs/6561/1354869
            let host = base_url
                .trim_end_matches('/')
                .trim_start_matches("https://")
                .trim_start_matches("http://");
            let resource_ids = [
                "volc.seedasr.sauc.duration",   // 豆包流式语音识别模型2.0 小时版
            ];

            let mut last_err_detail = String::new();

            for rid in &resource_ids {
                let ws_url = format!("wss://{host}/api/v3/sauc/bigmodel_async");
                let connect_id = uuid::Uuid::new_v4().to_string();
                let request_id = uuid::Uuid::new_v4().to_string();

                use tokio_tungstenite::tungstenite::client::IntoClientRequest;
                let mut req = ws_url.as_str().into_client_request()
                    .map_err(|e| format!("构造请求失败: {e}"))?;
                let headers = req.headers_mut();
                headers.insert("X-Api-Key", api_key.as_str().parse().unwrap());
                headers.insert("X-Api-Resource-Id", (*rid).parse().unwrap());
                headers.insert("X-Api-Request-Id", request_id.as_str().parse().unwrap());
                headers.insert("X-Api-Connect-Id", connect_id.as_str().parse().unwrap());
                headers.insert("X-Api-Sequence", "-1".parse().unwrap());

                match tokio_tungstenite::connect_async(req).await {
                    Ok((_ws, _resp)) => {
                        return Ok(format!(
                            "✓ 连接成功 — 火山引擎语音识别 (resource={rid})"
                        ));
                    }
                    Err(e) => {
                        use tokio_tungstenite::tungstenite::Error as WsErr;
                        let detail = match &e {
                            WsErr::Http(resp) => {
                                let status = resp.status().as_u16();
                                let logid = resp
                                    .headers()
                                    .get("X-Tt-Logid")
                                    .and_then(|v| v.to_str().ok())
                                    .unwrap_or("none");
                                let body_snippet: String = resp
                                    .body()
                                    .as_ref()
                                    .and_then(|b| String::from_utf8(b.clone()).ok())
                                    .unwrap_or_default()
                                    .chars()
                                    .take(300)
                                    .collect();
                                format!("HTTP {status} logid={logid} body={body_snippet}")
                            }
                            WsErr::Tls(e) => format!("TLS握手失败: {e}"),
                            WsErr::Io(e) => format!("网络错误: {e}"),
                            WsErr::Url(e) => format!("URL错误: {e}"),
                            WsErr::Protocol(e) => format!("协议错误: {e}"),
                            WsErr::HttpFormat(e) => format!("HTTP格式错误: {e}"),
                            other => format!("{other}"),
                        };
                        let is_auth_fail = matches!(
                            &e,
                            WsErr::Http(resp)
                                if resp.status().as_u16() == 401
                                    || resp.status().as_u16() == 403
                        );
                        if is_auth_fail {
                            // Auth rejected by server — try next resource id
                            last_err_detail = format!("{rid}: {detail}");
                        } else {
                            // Infrastructure error — don't treat as auth success,
                            // tell user exactly what went wrong
                            return Err(format!(
                                "无法连接到火山引擎服务器 (resource={rid}): {detail}"
                            ));
                        }
                    }
                }
            }

            Err(format!("鉴权失败 — App Key 或 Access Key 无效: {last_err_detail}"))
        }
        "xfyun" => {
            let secret = api_secret.unwrap_or_default();
            let secret_len = secret.len();
            if secret.is_empty() {
                return Err("请先填写 API Secret".into());
            }

            // Build HMAC-SHA1 signature (same algorithm as real transcribe)
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string();
            let sign_input = format!("{}{}", api_key, ts);
            let signa = {
                use ring::digest;
                let hash = digest::digest(
                    &digest::SHA1_FOR_LEGACY_USE_ONLY,
                    sign_input.as_bytes(),
                );
                hash.as_ref()
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>()
            };

            // Send a properly signed request (no audio body — just auth check)
            let url = format!(
                "{}/v2/api/upload?signa={}&ts={}&appId={}&fileSize=0&duration=0",
                base_url.trim_end_matches('/'),
                signa,
                ts,
                api_key,
            );

            let resp = client
                .post(&url)
                .header("Content-Type", "application/octet-stream")
                .timeout(std::time::Duration::from_secs(15))
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() { "连接超时".into() }
                    else if e.is_connect() { format!("无法连接到服务器: {e}") }
                    else { format!("网络错误: {e}") }
                })?;

            let http_status = resp.status().as_u16();
            let resp_text = resp.text().await.unwrap_or_default();
            let resp_json: serde_json::Value =
                serde_json::from_str(&resp_text).unwrap_or_default();

            // Xfyun returns "code" as either number or string
            let code_num = resp_json["code"].as_i64();
            let code_str = resp_json["code"].as_str();
            let is_success = code_num == Some(0)
                || code_str == Some("0");
            let is_error = code_num.map_or(false, |c| c != 0)
                || code_str.map_or(false, |c| c != "0");

            if is_success {
                return Ok("✓ 连接成功 — 讯飞极速听写".into());
            }
            if is_error {
                let msg = resp_json["message"]
                    .as_str()
                    .or_else(|| resp_json["desc"].as_str())
                    .unwrap_or("未知错误");
                let code = code_num.map(|c| c.to_string())
                    .or_else(|| code_str.map(|s| s.to_string()))
                    .unwrap_or_default();
                return Err(format!(
                    "鉴权失败: {msg} (code={code}) [DIAG: key_len={} secret_len={secret_len}]",
                    api_key.len(),
                ));
            }

            // Fallback: check HTTP status
            if http_status == 401 || http_status == 403 {
                return Err("AppId 或 API Secret 无效".into());
            }
            let snippet: String = resp_text.chars().take(200).collect();
            Err(format!(
                "无法解析服务器响应 (HTTP {http_status}): {snippet} [DIAG: key_len={} secret_len={secret_len}]",
                api_key.len(),
            ))
        }
        _ => Err(format!("厂商 {provider} 暂不支持测试连接")),
    }
}

/// Read the current custom prompt content.
#[tauri::command]
pub async fn cmd_custom_prompt_get() -> Result<String, String> {
    let prompts_dir = dirs::data_dir()
        .unwrap_or_default()
        .join("WhisperKey")
        .join("prompts");
    Ok(crate::llm::prompts::read_custom_prompt(&prompts_dir))
}

/// Write custom prompt content and hot-reload.
#[tauri::command]
pub async fn cmd_custom_prompt_set(content: String) -> Result<(), String> {
    let prompts_dir = dirs::data_dir()
        .unwrap_or_default()
        .join("WhisperKey")
        .join("prompts");
    crate::llm::prompts::reload_custom_prompt(&prompts_dir, &content)
        .map_err(|e| format!("保存自定义 Prompt 失败: {e}"))?;
    Ok(())
}
