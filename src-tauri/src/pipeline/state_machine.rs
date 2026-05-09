use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineState {
    Idle,
    Recording,
    Processing,
    Injecting,
    Error,
}

#[derive(Debug, Clone)]
pub enum PipelineEvent {
    HotkeyTriggered,
    RecordingStopped { reason: crate::audio::StopReason },
    AsrComplete { text: String, duration_ms: u64 },
    LlmComplete { text: String, tokens: u64 },
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
            // Start recording from idle
            (PipelineState::Idle, PipelineEvent::HotkeyTriggered) => PipelineState::Recording,

            // Stop recording -> processing
            (PipelineState::Recording, PipelineEvent::RecordingStopped { .. }) => PipelineState::Processing,

            // ASR done -> stay in processing (waiting for LLM)
            (PipelineState::Processing, PipelineEvent::AsrComplete { .. }) => PipelineState::Processing,

            // LLM done -> injecting
            (PipelineState::Processing, PipelineEvent::LlmComplete { .. }) => PipelineState::Injecting,

            // Inject done -> idle
            (PipelineState::Injecting, PipelineEvent::InjectDone) => PipelineState::Idle,

            // Error from any state
            (_, PipelineEvent::Error { .. }) => PipelineState::Error,

            // Error recovery -> idle
            (PipelineState::Error, _) => PipelineState::Idle,

            // Invalid transition
            _ => return None,
        };

        self.state = new_state;
        Some(new_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::StopReason;

    #[test]
    fn test_full_flow() {
        let mut sm = StateMachine::new();
        assert_eq!(sm.current_state(), PipelineState::Idle);

        // Start recording
        assert_eq!(
            sm.transition(&PipelineEvent::HotkeyTriggered),
            Some(PipelineState::Recording)
        );

        // Stop recording
        assert_eq!(
            sm.transition(&PipelineEvent::RecordingStopped {
                reason: StopReason::Manual
            }),
            Some(PipelineState::Processing)
        );

        // ASR complete
        assert_eq!(
            sm.transition(&PipelineEvent::AsrComplete {
                text: "hello".into(),
                duration_ms: 1000
            }),
            Some(PipelineState::Processing)
        );

        // LLM complete
        assert_eq!(
            sm.transition(&PipelineEvent::LlmComplete {
                text: "polished".into(),
                tokens: 50
            }),
            Some(PipelineState::Injecting)
        );

        // Inject done
        assert_eq!(
            sm.transition(&PipelineEvent::InjectDone),
            Some(PipelineState::Idle)
        );
    }

    #[test]
    fn test_error_recovery() {
        let mut sm = StateMachine::new();
        assert_eq!(
            sm.transition(&PipelineEvent::Error {
                message: "test error".into()
            }),
            Some(PipelineState::Error)
        );
        // From error, any event goes to idle
        assert_eq!(
            sm.transition(&PipelineEvent::HotkeyTriggered),
            Some(PipelineState::Idle)
        );
    }

    #[test]
    fn test_invalid_transition() {
        let mut sm = StateMachine::new();
        // Can't go to processing from idle without recording
        assert_eq!(
            sm.transition(&PipelineEvent::AsrComplete {
                text: "x".into(),
                duration_ms: 0
            }),
            None
        );
    }
}
