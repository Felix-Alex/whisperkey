const RAW_PROMPT: &str = "仅输出转录原文，不添加任何解释或修饰。\n输入：{{TEXT}}";
const POLISH_PROMPT: &str = "你是一个文本润色助手。优化以下文本的语法和流畅度，保持原意不变，不添加额外内容。\n输入：{{TEXT}}";
const MARKDOWN_PROMPT: &str = "你是一个 Markdown 格式化助手。将以下文本转换为结构良好的 Markdown 格式。\n输入：{{TEXT}}";

pub fn render_prompt(mode: &crate::llm::r#trait::LlmMode, raw_text: &str) -> String {
    let template = match mode {
        crate::llm::r#trait::LlmMode::Raw => RAW_PROMPT,
        crate::llm::r#trait::LlmMode::Polish => POLISH_PROMPT,
        crate::llm::r#trait::LlmMode::Markdown => MARKDOWN_PROMPT,
    };
    template.replace("{{TEXT}}", raw_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::r#trait::LlmMode;

    #[test]
    fn test_render_raw() {
        let result = render_prompt(&LlmMode::Raw, "hello");
        assert!(result.contains("hello"));
        assert!(!result.contains("{{TEXT}}"));
    }

    #[test]
    fn test_render_polish() {
        let result = render_prompt(&LlmMode::Polish, "test text");
        assert!(result.contains("test text"));
        assert!(result.contains("润色"));
        assert!(!result.contains("{{TEXT}}"));
    }

    #[test]
    fn test_render_markdown() {
        let result = render_prompt(&LlmMode::Markdown, "content");
        assert!(result.contains("content"));
        assert!(!result.contains("{{TEXT}}"));
    }
}
