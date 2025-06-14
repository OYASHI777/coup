use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
use super::game_state::GameData;
pub struct AssassinateInvitesChallenge {
}
pub struct AssassinateInvitesBlock {
    player_blocking: usize,
}
pub struct AssassinateBlockInvitesChallenge {
    player_blocking: usize,
}
pub struct AssassinateBlockChallenged {
    player_blocking: usize,
    player_challenger: usize,
}
pub struct AssassinateBlockChallengerFailed {
    player_challenger: usize,
}
pub struct AssassinateSucceeded {
    player_blocking: usize,
}
pub struct AssassinateChallenged {
    player_challenger: usize,
}
pub struct AssassinateChallengerFailed {
    player_challenger: usize,
}

impl CoupTransition for AssassinateInvitesChallenge {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateInvitesBlock {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateBlockInvitesChallenge {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateBlockChallenged {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateBlockChallengerFailed {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateSucceeded {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateChallenged {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for AssassinateChallengerFailed {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}