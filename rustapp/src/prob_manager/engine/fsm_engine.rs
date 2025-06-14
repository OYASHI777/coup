use super::models::turn_start::TurnStart;
use crate::prob_manager::engine::models::engine_state::{CoupTransition, EngineStateName};
use crate::prob_manager::engine::models::game_state::GameState;
use crate::traits::prob_manager::coup_analysis::CoupTraversal;
use crate::history_public::{ActionObservation};
use super::models::engine_state::EngineState;

pub trait Node {
    fn dispatch(&self) -> bool;
}
// TODO: Write test for same resources after push() then pop()
pub struct FSMEngine {
    history: Vec<ActionObservation>,
    history_state: Vec<EngineState>,
    state: GameState,
}
impl FSMEngine {
    /// Generates an FSMEngine
    pub fn new() -> Self {
        FSMEngine { 
            history: Vec::with_capacity(128), 
            history_state: Vec::with_capacity(128),
            state: GameState::new(),
        }
    }
}
impl CoupTraversal for FSMEngine {
    fn start_public(&mut self) {
        self.reset();
        *self = Self::new();
        self.history_state.push(EngineState::TurnStart(TurnStart {  }));
    }

    fn start_private(&mut self, player: usize, cards: &[crate::history_public::Card; 2]) {
        todo!()
    }
    /// Update's Engine's state
    fn push_ao_public(&mut self, action: &ActionObservation) {
        self.state.engine_state = self.state.engine_state.state_update(action, &mut self.state.game_data);
        EngineState::action_update(action, &mut self.state.game_data);
        self.history_state.push(self.state.engine_state);
        self.history.push(*action);
    }

    fn push_ao_public_lazy(&mut self, action: &ActionObservation) {
        self.push_ao_public(action);
    }
    /// Update's Engine's state with private information
    fn push_ao_private(&mut self, action: &ActionObservation) {
        todo!()
    }

    fn push_ao_private_lazy(&mut self, action: &ActionObservation) {
        todo!()
    }

    fn pop(&mut self) {
        // Case when history is empty => Off or ignore
        if let Some(action) = self.history.pop() {
            self.history_state.pop();
            // This must come first so Discard can add the influence back
            EngineState::reverse_action_update(&action, &mut self.state.game_data);
            if let Some(prev_state) = self.history_state.last() {
                self.state.engine_state = *prev_state;
                // This is affected by Discard and must come after
                self.state.engine_state.reverse_state_update(&action, &mut self.state.game_data);
            } else {
                panic!("Pop not working")
            }
        }
    }

    fn reset(&mut self) {
        self.history.clear();
        self.state.reset();
    }
}