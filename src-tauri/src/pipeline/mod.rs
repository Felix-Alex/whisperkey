pub mod state_machine;

use tokio::sync::watch;
use crate::pipeline::state_machine::{PipelineEvent, PipelineState, StateMachine};

pub struct Pipeline {
    state_machine: StateMachine,
    state_tx: watch::Sender<PipelineState>,
    pub state_rx: watch::Receiver<PipelineState>,
}

impl Pipeline {
    pub fn new() -> Self {
        let (state_tx, state_rx) = watch::channel(PipelineState::Idle);
        Self {
            state_machine: StateMachine::new(),
            state_tx,
            state_rx,
        }
    }

    pub fn handle_event(&mut self, event: PipelineEvent) {
        if let Some(new_state) = self.state_machine.transition(&event) {
            let _ = self.state_tx.send(new_state);
        }
    }

    pub fn state(&self) -> PipelineState {
        self.state_machine.current_state()
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
