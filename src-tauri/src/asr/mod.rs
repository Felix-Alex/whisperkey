pub mod official;
pub mod openai;
pub mod r#trait;
pub mod volcengine;
pub mod xfyun;

use std::collections::HashMap;
use std::sync::Arc;
use crate::asr::r#trait::AsrProvider;

pub struct AsrRegistry {
    providers: HashMap<String, Arc<dyn AsrProvider>>,
}

impl Default for AsrRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AsrRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, provider: Arc<dyn AsrProvider>) {
        self.providers.insert(name.to_string(), provider);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn AsrProvider>> {
        self.providers.get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::error::AppResult;
    use crate::asr::r#trait::{AsrRequest, AsrResponse};

    struct MockAsr;

    #[async_trait]
    impl AsrProvider for MockAsr {
        fn name(&self) -> &'static str { "mock" }
        async fn transcribe(&self, _req: AsrRequest) -> AppResult<AsrResponse> {
            Ok(AsrResponse { text: "mock".into(), duration_ms: 0 })
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = AsrRegistry::new();
        registry.register("mock", Arc::new(MockAsr));
        let p = registry.get("mock").unwrap();
        assert_eq!(p.name(), "mock");
    }
}
