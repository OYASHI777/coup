use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
use super::game_state::GameData;
pub struct ExchangeInvitesChallenge {
}
pub struct ExchangeDrawn {
}
pub struct ExchangeChallenged {
    player_challenger: usize,
}
pub struct ExchangeChallengerFailed {
    player_challenger: usize,
}

impl CoupTransition for ExchangeInvitesChallenge {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ExchangeDrawn {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ExchangeChallenged {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ExchangeChallengerFailed {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}