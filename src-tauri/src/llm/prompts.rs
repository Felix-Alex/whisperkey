//! Prompt templates loaded from files at runtime.
//! Built-in defaults are embedded at compile time from resources/prompts/.
//! User overrides live in %APPDATA%/WhisperKey/prompts/ and take precedence.

use std::collections::HashMap;
use std::sync::RwLock;
use crate::error::{AppError, AppResult};

static PROMPT_CACHE: RwLock<Option<HashMap<String, String>>> = RwLock::new(None);

const RAW_DEFAULT: &str = include_str!("../../resources/prompts/raw.md");
const POLISH_DEFAULT: &str = include_str!("../../resources/prompts/polish.md");
const MARKDOWN_DEFAULT: &str = include_str!("../../resources/prompts/markdown.md");
const QUICK_ASK_DEFAULT: &str = include_str!("../../resources/prompts/quick_ask.md");
const CUSTOM_DEFAULT: &str = include_str!("../../resources/prompts/custom.md");

fn builtin_default(mode: &str) -> &'static str {
    match mode {
        "raw" => RAW_DEFAULT,
        "polish" => POLISH_DEFAULT,
        "markdown" => MARKDOWN_DEFAULT,
        "quick_ask" => QUICK_ASK_DEFAULT,
        "custom" => CUSTOM_DEFAULT,
        _ => "",
    }
}

/// Initialize the prompt cache from user prompt files.
/// Seeds missing files from built-in defaults, then loads all into memory.
/// Call once at startup before any pipeline processing.
pub fn init_prompts(prompts_dir: &std::path::Path) {
    let mut cache = HashMap::new();
    for mode in &["raw", "polish", "markdown", "quick_ask", "custom"] {
        let file_path = prompts_dir.join(format!("{mode}.md"));
        let content = if file_path.exists() {
            std::fs::read_to_string(&file_path)
                .unwrap_or_else(|_| builtin_default(mode).to_string())
        } else {
            let default = builtin_default(mode);
            if let Err(e) = std::fs::write(&file_path, default) {
                tracing::warn!("[prompts] Failed to seed {file_path:?}: {e}");
            }
            default.to_string()
        };
        cache.insert(mode.to_string(), content);
    }
    *PROMPT_CACHE.write().unwrap() = Some(cache);
}

/// Hot-reload the custom prompt after user edits it.
/// Writes content to disk and updates the in-memory cache immediately.
pub fn reload_custom_prompt(prompts_dir: &std::path::Path, content: &str) -> AppResult<()> {
    let file_path = prompts_dir.join("custom.md");
    std::fs::write(&file_path, content).map_err(|e| {
        tracing::error!("[prompts] Failed to write custom prompt to {file_path:?}: {e}");
        AppError::Internal
    })?;
    if let Some(ref mut cache) = *PROMPT_CACHE.write().unwrap() {
        cache.insert("custom".to_string(), content.to_string());
    }
    Ok(())
}

/// Read the current custom prompt content from disk.
pub fn read_custom_prompt(prompts_dir: &std::path::Path) -> String {
    let file_path = prompts_dir.join("custom.md");
    if file_path.exists() {
        std::fs::read_to_string(&file_path).unwrap_or_else(|_| builtin_default("custom").to_string())
    } else {
        builtin_default("custom").to_string()
    }
}

/// Return the system prompt for the given mode.
/// The prompt is a pure instruction template; the user's text is sent
/// separately as the user message in the LLM chat request.
pub fn render_prompt(mode: &crate::llm::r#trait::OutputMode, _raw_text: &str) -> String {
    let cache = PROMPT_CACHE
        .read()
        .unwrap()
        .as_ref()
        .expect("prompts not initialized — call init_prompts first")
        .clone();
    cache
        .get(mode.as_str())
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::r#trait::OutputMode;

    fn ensure_init() {
        if PROMPT_CACHE.read().unwrap().is_none() {
            let tmp = std::env::temp_dir().join("whisperkey_test_prompts");
            let _ = std::fs::create_dir_all(&tmp);
            init_prompts(&tmp);
        }
    }

    #[test]
    fn test_all_modes_in_cache_and_render_non_empty() {
        ensure_init();

        let cache = PROMPT_CACHE.read().unwrap();
        let cache = cache.as_ref().unwrap();
        for mode in &["raw", "polish", "markdown", "quick_ask", "custom"] {
            assert!(cache.contains_key(*mode), "Cache missing mode: {mode}");
        }

        for mode in [
            OutputMode::Raw,
            OutputMode::Polish,
            OutputMode::Markdown,
            OutputMode::QuickAsk,
            OutputMode::Custom,
        ] {
            let result = render_prompt(&mode, "unused text");
            assert!(!result.is_empty(), "Mode {:?} returned empty prompt", mode);
        }
    }

    #[test]
    fn test_custom_prompt_reload_and_read() {
        ensure_init();
        let tmp = std::env::temp_dir().join("whisperkey_test_prompts");
        let _ = std::fs::create_dir_all(&tmp);

        // Test hot reload via cache
        let original = render_prompt(&OutputMode::Custom, "");
        reload_custom_prompt(&tmp, "新自定义内容").unwrap();
        let updated = render_prompt(&OutputMode::Custom, "");
        assert_eq!(updated, "新自定义内容");
        assert_ne!(original, updated);

        // Test disk read
        let disk_content = read_custom_prompt(&tmp);
        assert_eq!(disk_content, "新自定义内容");

        // Restore default
        reload_custom_prompt(&tmp, builtin_default("custom")).unwrap();
        let restored = render_prompt(&OutputMode::Custom, "");
        assert_eq!(restored, builtin_default("custom"));
    }
}
