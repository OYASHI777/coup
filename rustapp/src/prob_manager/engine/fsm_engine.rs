use super::models::turn_start::TurnStart;
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
    state: GameState,
}
impl FSMEngine {
    /// Generates an FSMEngine
    pub fn new() -> Self {
        FSMEngine { 
            history: Vec::with_capacity(128), 
            state: GameState::new(),
        }
    }
}
impl CoupTraversal for FSMEngine {
    fn start_public(&mut self) {
        todo!()
    }

    fn start_private(&mut self, player: usize, cards: &[crate::history_public::Card; 2]) {
        todo!()
    }
    /// Update's Engine's state
    fn push_ao_public(&mut self, action: &ActionObservation) {
        self.state.push(action);
    }

    fn push_ao_public_lazy(&mut self, action: &ActionObservation) {
        self.state.push(action);
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
            self.state.pop(&action);
        }
    }

    fn reset(&mut self) {
        self.history.clear();
        self.state.reset();
    }
}