use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::{EngineState, EngineStateName};
use super::game_state::GameData;
#[derive(Copy, Clone)]
pub struct ExchangeInvitesChallenge {
}
#[derive(Copy, Clone)]
pub struct ExchangeDrawn {
}
#[derive(Copy, Clone)]
pub struct ExchangeChallenged {
    player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct ExchangeChallengerFailed {
    player_challenger: usize,
}

impl CoupTransition for ExchangeInvitesChallenge {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for ExchangeDrawn {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for ExchangeChallenged {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for ExchangeChallengerFailed {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}