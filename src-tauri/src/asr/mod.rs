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

impl Default for AsrRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the default AsrRegistry with all 4 providers registered.
pub fn default_registry() -> AsrRegistry {
    let mut registry = AsrRegistry::new();
    registry.register("openai", Arc::new(openai::OpenAiAsr));
    registry.register("xfyun", Arc::new(xfyun::XfyunAsr));
    registry.register("volcengine", Arc::new(volcengine::VolcengineAsr));
    registry.register("official", Arc::new(official::OfficialAsr));
    registry
}
