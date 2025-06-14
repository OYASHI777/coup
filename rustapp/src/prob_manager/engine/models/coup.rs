use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
pub struct CoupHit {
    player_hit: usize,
}

impl CoupTransition for CoupHit {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}