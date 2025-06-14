use crate::prob_manager::engine::models::{engine_state::CoupTransition};
use crate::history_public::ActionObservation;
use super::engine_state::{EngineState, EngineStateName};
use super::game_state::GameData;
#[derive(Copy, Clone)]
pub struct TaxInvitesChallenge {
}
#[derive(Copy, Clone)]
pub struct TaxChallenged {
    player_challenger: u8,
}
#[derive(Copy, Clone)]
pub struct TaxChallengerFailed {
    player_chellenger: u8,
}

impl CoupTransition for TaxInvitesChallenge {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
    
    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for TaxChallenged {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
    
    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for TaxChallengerFailed {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
    
    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}