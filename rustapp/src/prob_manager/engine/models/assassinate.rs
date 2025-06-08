use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
pub struct AssassinateInvitesChallenge {
    game_state: GameState,
}
pub struct AssassinateInvitesBlock {
    game_state: GameState,
    player_blocking: usize,
}
pub struct AssassinateBlockInvitesChallenge {
    game_state: GameState,
    player_blocking: usize,
}
pub struct AssassinateBlockChallenged {
    game_state: GameState,
    player_blocking: usize,
    player_challenger: usize,
}
pub struct AssassinateBlockChallengerFailed {
    game_state: GameState,
    player_challenger: usize,
}
pub struct AssassinateSucceeded {
    game_state: GameState,
    player_blocking: usize,
}
pub struct AssassinateChallenged {
    game_state: GameState,
    player_challenger: usize,
}
pub struct AssassinateChallengerFailed {
    game_state: GameState,
    player_challenger: usize,
}

impl CoupTransition for AssassinateInvitesChallenge {
    fn next(self, action: &ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateInvitesBlock {
    fn next(self, action: &ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateBlockInvitesChallenge {
    fn next(self, action: &ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateBlockChallenged {
    fn next(self, action: &ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateBlockChallengerFailed {
    fn next(self, action: &ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateSucceeded {
    fn next(self, action: &ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateChallenged {
    fn next(self, action: &ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateChallengerFailed {
    fn next(self, action: &ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation) -> EngineState {
        todo!()
    }
}