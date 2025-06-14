use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
use super::game_state::GameData;
pub struct CoupHit {
    player_hit: usize,
}

impl CoupTransition for CoupHit {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}