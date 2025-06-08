use crate::prob_manager::engine::models::engine_state::CoupTransition;

use super::game_state::GameState;
use super::engine_state::EngineState;
pub struct ForeignAidInvitesBlock {
    game_state: GameState,
}
pub struct ForeignAidBlockInvitesChallenge {
    game_state: GameState,
    player_blocking: usize,
}
pub struct ForeignAidBlockChallenged {
    game_state: GameState,
    player_blocking: usize,
    player_challenger: usize,
}
pub struct ForeignAidBlockChallengerFailed {
    game_state: GameState,
    player_challenger: usize,
}

impl CoupTransition for ForeignAidInvitesBlock {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ForeignAidBlockInvitesChallenge {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ForeignAidBlockChallenged {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ForeignAidBlockChallengerFailed {
    fn next(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }

    fn prev(self, action: &crate::history_public::ActionObservation) -> EngineState {
        todo!()
    }
}