use async_trait::async_trait;

/// Five output modes. Raw skips LLM entirely; the other four are LLM-processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Raw,
    Polish,
    QuickAsk,
    Markdown,
    Custom,
}

impl OutputMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "raw" => Some(Self::Raw),
            "polish" => Some(Self::Polish),
            "markdown" => Some(Self::Markdown),
            "quick_ask" => Some(Self::QuickAsk),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Polish => "polish",
            Self::Markdown => "markdown",
            Self::QuickAsk => "quick_ask",
            Self::Custom => "custom",
        }
    }

    /// Whether this mode requires LLM processing (all except Raw).
    pub fn requires_llm(&self) -> bool {
        !matches!(self, Self::Raw)
    }
}

/// Global LLM Provider trait. A single provider handles all LLM modes.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a chat request with system prompt and user text.
    async fn chat(
        &self,
        system_prompt: &str,
        user_text: &str,
        api_key: &str,
        base_url: &str,
        model: &str,
    ) -> Result<String, crate::error::AppError>;

    fn name(&self) -> &'static str;
}
