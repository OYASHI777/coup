use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use super::engine_state::EngineState;
pub struct TaxInvitesChallenge {
    game_state: GameState,
}
pub struct TaxChallenged {
    game_state: GameState,
    player_challenger: u8,
}
pub struct TaxChallengerFailed {
    game_state: GameState,
    player_chellenger: u8,
}

impl CoupTransition for TaxInvitesChallenge {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for TaxChallenged {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for TaxChallengerFailed {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}