use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
pub struct CoupHit {
    game_state: GameState,
    player_hit: usize,
}

impl CoupTransition for CoupHit {
    fn next(self, action: &ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation) -> EngineState {
        todo!()
    }
}