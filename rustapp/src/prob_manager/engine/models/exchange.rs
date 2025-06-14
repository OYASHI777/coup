use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
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
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ExchangeDrawn {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ExchangeChallenged {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ExchangeChallengerFailed {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}