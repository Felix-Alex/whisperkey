pub mod anthropic;
pub mod deepseek;
pub mod doubao;
pub mod ernie;
pub mod gemini;
pub mod openai;
pub mod prompts;
pub mod qwen;
pub mod r#trait;

use std::collections::HashMap;
use std::sync::Arc;
use crate::llm::r#trait::LlmProvider;

pub struct LlmRegistry {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
}

impl Default for LlmRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmRegistry {
    pub fn new() -> Self {
        Self { providers: HashMap::new() }
    }

    pub fn register(&mut self, name: &str, provider: Arc<dyn LlmProvider>) {
        self.providers.insert(name.to_string(), provider);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn LlmProvider>> {
        self.providers.get(name).cloned()
    }
}

/// Mode B (polish) output length check.
/// If output exceeds raw_text * 1.2 + 20 chars, reject and return raw_text.
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
