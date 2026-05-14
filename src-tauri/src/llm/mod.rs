pub mod anthropic;
pub mod ernie;
pub mod gemini;
pub mod openai;
pub mod prompts;
pub mod r#trait;

use std::collections::HashMap;
use std::sync::Arc;

use crate::config::schema::LlmConfig;
use crate::llm::r#trait::LlmProvider;

pub struct LlmRegistry {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
}

impl LlmRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, provider: Arc<dyn LlmProvider>) {
        self.providers.insert(name.to_string(), provider);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn LlmProvider>> {
        self.providers.get(name).cloned()
    }
}

impl Default for LlmRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the default LlmRegistry with all providers registered.
/// OpenAI-compatible protocol providers share the same implementation.
pub fn default_registry() -> LlmRegistry {
    let mut registry = LlmRegistry::new();
    let openai_compat = Arc::new(openai::OpenAiCompatibleLlm);
    registry.register("openai", openai_compat.clone());
    registry.register("deepseek", openai_compat.clone());
    registry.register("qwen", openai_compat.clone());
    registry.register("doubao", openai_compat);
    registry.register("anthropic", Arc::new(anthropic::AnthropicLlm));
    registry.register("gemini", Arc::new(gemini::GeminiLlm));
    registry.register("ernie", Arc::new(ernie::ErnieLlm::new()));
    registry
}

/// Resolve a provider from config and return the provider + effective model.
pub fn resolve_provider<'a>(
    config: &'a LlmConfig,
    registry: &'a LlmRegistry,
) -> Option<(Arc<dyn LlmProvider>, &'a str)> {
    let provider = registry.get(&config.provider)?;
    Some((provider, &config.model))
}

/// Mode B (polish) output length check.
/// Ensures LLM output doesn't exceed 120% of input length + 20 chars.
pub fn validate_polish_output(raw_text: &str, output: &str) -> String {
    let max_len = (raw_text.chars().count() as f64 * 1.2) as usize + 20;
    if output.chars().count() > max_len {
        tracing::warn!("Polish output exceeded length limit, returning raw text");
        raw_text.to_string()
    } else {
        output.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_acceptable_output() {
        let result = validate_polish_output("hello world", "hello world!");
        assert_eq!(result, "hello world!");
    }

    #[test]
    fn test_validate_excessive_output() {
        let raw = "hi";
        let long_output = "x".repeat(100);
        let result = validate_polish_output(raw, &long_output);
        assert_eq!(result, "hi");
    }
}
