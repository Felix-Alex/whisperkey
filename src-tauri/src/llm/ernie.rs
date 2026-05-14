use async_trait::async_trait;
use serde_json::json;
use std::sync::Mutex;

use crate::error::{AppError, AppResult};
use crate::llm::r#trait::LlmProvider;

pub struct ErnieLlm {
    /// Cached OAuth2 access token + expiry
    token_cache: Mutex<Option<(String, std::time::Instant)>>,
}

impl ErnieLlm {
    pub fn new() -> Self {
        Self {
            token_cache: Mutex::new(None),
        }
    }

    /// Exchange API key + secret key for an OAuth2 access token (cached 25 min).
    async fn get_access_token(&self, api_key: &str, secret_key: &str, base_url: &str) -> AppResult<String> {
        // Check cache
        {
            let cache = self.token_cache.lock().unwrap();
            if let Some((token, expiry)) = cache.as_ref() {
                if std::time::Instant::now() < *expiry {
                    return Ok(token.clone());
                }
            }
        }

        let url = format!("{}/oauth/2.0/token", base_url.trim_end_matches('/'));
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .query(&[
                ("grant_type", "client_credentials"),
                ("client_id", api_key),
                ("client_secret", secret_key),
            ])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AppError::LlmTimeout
                } else {
                    AppError::Network
                }
            })?;

        let body: serde_json::Value = resp.json().await.map_err(|_| AppError::LlmAuth)?;
        let token = body["access_token"]
            .as_str()
            .ok_or(AppError::LlmAuth)?
            .to_string();

        // Cache for 25 minutes
        *self.token_cache.lock().unwrap() = Some((
            token.clone(),
            std::time::Instant::now() + std::time::Duration::from_secs(25 * 60),
        ));

        Ok(token)
    }
}

impl Default for ErnieLlm {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for ErnieLlm {
    fn name(&self) -> &'static str {
        "百度文心一言"
    }

    async fn chat(
        &self,
        system_prompt: &str,
        user_text: &str,
        api_key: &str,
        base_url: &str,
        model: &str,
    ) -> AppResult<String> {
        // api_key doubles as client_id; base_url must contain secret_key in a real impl
        // For now, use api_key directly as token if it looks like a token (starts with "sk-")
        let token = if api_key.len() > 40 {
            api_key.to_string()
        } else {
            // Assume base_url contains `apiKey:secretKey` pair encoded
            let parts: Vec<&str> = base_url.rsplit('/').collect();
            let secret = parts.first().map(|s| s.to_string()).unwrap_or_default();
            self.get_access_token(api_key, &secret, "https://aip.baidubce.com").await?
        };

        let url = format!(
            "https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/{model}?access_token={token}",
        );

        let body = json!({
            "messages": [
                {"role": "user", "content": format!("{system_prompt}\n\n{user_text}")}
            ]
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AppError::LlmTimeout
                } else {
                    AppError::Network
                }
            })?;

        match resp.status().as_u16() {
            200 => {
                let body: serde_json::Value = resp.json().await.map_err(|_| AppError::Internal)?;
                let text = body["result"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                Ok(text)
            }
            401 | 403 => Err(AppError::LlmAuth),
            429 => Err(AppError::LlmQuota),
            _ => Err(AppError::Network),
        }
    }
}
