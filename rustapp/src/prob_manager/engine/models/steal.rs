use super::game_state::GameState;
use super::engine_state::EngineState;
use crate::{prob_manager::engine::models::engine_state::CoupTransition};
use crate::history_public::{ActionObservation, Card};

pub struct StealInvitesChallenge {
}
pub struct StealChallenged {
    player_blocking: usize,
    player_challenger: usize,
}
pub struct StealChallengerFailed {
    player_blocking: usize,
    player_challenger: usize,
}
pub struct StealInvitesBlock {
    player_blocking: u8,
}
pub struct StealBlockInvitesChallenge {
    player_blocking: u8,
    card_blocker: Card,
}
pub struct StealBlockChallenged {
    player_blocking: u8,
    player_challenger: u8,
    card_blocker: Card,
}
pub struct StealBlockChallengerFailed {
    player_challenger: u8,
}

impl CoupTransition for StealInvitesChallenge {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}
impl CoupTransition for StealChallenged {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}
impl CoupTransition for StealChallengerFailed {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}
impl CoupTransition for StealInvitesBlock {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}
impl CoupTransition for StealBlockInvitesChallenge {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}
impl CoupTransition for StealBlockChallenged {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}
impl CoupTransition for StealBlockChallengerFailed {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}