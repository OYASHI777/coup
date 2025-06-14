use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::{EngineState, EngineStateName};
use super::game_state::GameData;
#[derive(Copy, Clone)]
pub struct End {
}
// TODO: Mate put all of these in GameState not vice versa
impl End {
    pub fn new() -> Self {
        todo!()
    }
}

impl CoupTransition for End {
    fn state_update(&self, action: &crate::history_public::ActionObservation, game_data: &mut GameData) -> EngineState {
        panic!("Game has ended")
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}