use serde::{Deserialize, Serialize};

/// Pipeline states following the ASR + LLM two-step architecture.
/// Raw mode skips LlmProcessing and goes directly from AsrTranscribing to Injecting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineState {
    Idle,
    Recording,
    AsrTranscribing,
    /// Only entered for non-raw modes (polish/markdown/quick_ask/custom)
    LlmProcessing,
    Injecting,
    Error,
}

#[derive(Debug, Clone)]
pub enum PipelineEvent {
    HotkeyTriggered,
    RecordingStopped,
    AsrComplete { text: String },
    LlmComplete { text: String },
    InjectDone,
    Error { message: String },
}

pub struct StateMachine {
    state: PipelineState,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: PipelineState::Idle,
        }
    }

    pub fn current_state(&self) -> PipelineState {
        self.state
    }

    pub fn transition(&mut self, event: &PipelineEvent) -> Option<PipelineState> {
        let new_state = match (self.state, event) {
            (PipelineState::Idle, PipelineEvent::HotkeyTriggered) => PipelineState::Recording,

            (PipelineState::Recording, PipelineEvent::RecordingStopped) => {
                PipelineState::AsrTranscribing
            }

            // Raw mode: ASR → Injecting (skip LLM)
            (PipelineState::AsrTranscribing, PipelineEvent::AsrComplete { .. }) => {
                // Caller decides whether to go to LlmProcessing or Injecting based on mode.
                // Default transition: ASR → LLM. Raw mode handled externally.
                PipelineState::LlmProcessing
            }

            (PipelineState::LlmProcessing, PipelineEvent::LlmComplete { .. }) => {
                PipelineState::Injecting
            }

            (PipelineState::Injecting, PipelineEvent::InjectDone) => PipelineState::Idle,

            (_, PipelineEvent::Error { .. }) => PipelineState::Error,

            (PipelineState::Error, _) => PipelineState::Idle,

            _ => return None,
        };

        self.state = new_state;
        Some(new_state)
    }

    /// Force transition to Injecting (used by raw mode to skip LlmProcessing).
    pub fn force_injecting(&mut self) -> PipelineState {
        self.state = PipelineState::Injecting;
        PipelineState::Injecting
    }

    /// Force transition to LlmProcessing (for non-raw modes after ASR).
    pub fn force_llm_processing(&mut self) -> PipelineState {
        self.state = PipelineState::LlmProcessing;
        PipelineState::LlmProcessing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_flow_raw_mode() {
        let mut sm = StateMachine::new();
        assert_eq!(sm.current_state(), PipelineState::Idle);

        // Hotkey → Recording
        assert_eq!(
            sm.transition(&PipelineEvent::HotkeyTriggered),
            Some(PipelineState::Recording)
        );

        // Stop → AsrTranscribing
        assert_eq!(
            sm.transition(&PipelineEvent::RecordingStopped),
            Some(PipelineState::AsrTranscribing)
        );

        // Raw mode: force skip to Injecting
        assert_eq!(sm.force_injecting(), PipelineState::Injecting);

        // Inject done → Idle
        assert_eq!(
            sm.transition(&PipelineEvent::InjectDone),
            Some(PipelineState::Idle)
        );
    }

    #[test]
    fn test_full_flow_llm_mode() {
        let mut sm = StateMachine::new();

        sm.transition(&PipelineEvent::HotkeyTriggered);
        sm.transition(&PipelineEvent::RecordingStopped);

        // Default ASR → LlmProcessing
        assert_eq!(
            sm.transition(&PipelineEvent::AsrComplete {
                text: "hello".into()
            }),
            Some(PipelineState::LlmProcessing)
        );

        // LLM completes → Injecting
        assert_eq!(
            sm.transition(&PipelineEvent::LlmComplete {
                text: "polished".into()
            }),
            Some(PipelineState::Injecting)
        );

        // Inject done → Idle
        assert_eq!(
            sm.transition(&PipelineEvent::InjectDone),
            Some(PipelineState::Idle)
        );
    }

    #[test]
    fn test_asr_failure() {
        let mut sm = StateMachine::new();
        sm.transition(&PipelineEvent::HotkeyTriggered);
        sm.transition(&PipelineEvent::RecordingStopped);

        assert_eq!(
            sm.transition(&PipelineEvent::Error {
                message: "ASR timeout".into()
            }),
            Some(PipelineState::Error)
        );

        // Error → Idle
        assert_eq!(
            sm.transition(&PipelineEvent::HotkeyTriggered),
            Some(PipelineState::Idle)
        );
    }

    #[test]
    fn test_llm_failure() {
        let mut sm = StateMachine::new();
        sm.transition(&PipelineEvent::HotkeyTriggered);
        sm.transition(&PipelineEvent::RecordingStopped);
        sm.transition(&PipelineEvent::AsrComplete {
            text: "hello".into(),
        });

        // LLM fails
        assert_eq!(
            sm.transition(&PipelineEvent::Error {
                message: "LLM auth error".into()
            }),
            Some(PipelineState::Error)
        );
    }

    #[test]
    fn test_invalid_transition() {
        let mut sm = StateMachine::new();
        // Cannot inject from Idle
        assert_eq!(sm.transition(&PipelineEvent::InjectDone), None);
    }
}
