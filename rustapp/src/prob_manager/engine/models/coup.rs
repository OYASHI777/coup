use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::{EngineState, EngineStateName};
use super::game_state::GameData;
#[derive(Copy, Clone)]
pub struct CoupHit {
    player_hit: usize,
}

impl CoupTransition for CoupHit {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}