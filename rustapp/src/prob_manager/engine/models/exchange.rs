use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
pub struct ExchangeInvitesChallenge {
    game_state: GameState,
}
pub struct ExchangeDrawn {
    game_state: GameState,
}
pub struct ExchangeChallenged {
    game_state: GameState,
    player_challenger: usize,
}
pub struct ExchangeChallengerFailed {
    game_state: GameState,
    player_challenger: usize,
}

impl CoupTransition for ExchangeInvitesChallenge {
    fn next(self, action: &ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ExchangeDrawn {
    fn next(self, action: &ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ExchangeChallenged {
    fn next(self, action: &ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ExchangeChallengerFailed {
    fn next(self, action: &ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation) -> EngineState {
        todo!()
    }
}