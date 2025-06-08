use super::game_state::GameState;
use super::engine_state::EngineState;
use crate::{history_public::Card, prob_manager::engine::models::engine_state::CoupTransition};

pub struct StealInvitesChallenge {
    game_state: GameState,
}
pub struct StealChallenged {
    game_state: GameState,
    player_blocking: usize,
    player_challenger: usize,
}
pub struct StealChallengerFailed {
    game_state: GameState,
    player_blocking: usize,
    player_challenger: usize,
}
pub struct StealInvitesBlock {
    game_state: GameState,
    player_blocking: u8,
}
pub struct StealBlockInvitesChallenge {
    game_state: GameState,
    player_blocking: u8,
    card_blocker: Card,
}
pub struct StealBlockChallenged {
    game_state: GameState,
    player_blocking: u8,
    player_challenger: u8,
    card_blocker: Card,
}
pub struct StealBlockChallengerFailed {
    game_state: GameState,
    player_challenger: u8,
}

impl CoupTransition for StealInvitesChallenge {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for StealChallenged {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for StealChallengerFailed {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for StealInvitesBlock {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for StealBlockInvitesChallenge {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for StealBlockChallenged {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for StealBlockChallengerFailed {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}