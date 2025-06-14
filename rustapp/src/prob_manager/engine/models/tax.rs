use crate::prob_manager::engine::models::{engine_state::CoupTransition};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
use super::game_state::GameData;
pub struct TaxInvitesChallenge {
}
pub struct TaxChallenged {
    player_challenger: u8,
}
pub struct TaxChallengerFailed {
    player_chellenger: u8,
}

impl CoupTransition for TaxInvitesChallenge {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for TaxChallenged {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for TaxChallengerFailed {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}