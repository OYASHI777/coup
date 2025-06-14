use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
use super::game_state::GameData;
#[derive(Copy, Clone)]
pub struct AssassinateInvitesChallenge {
}
#[derive(Copy, Clone)]
pub struct AssassinateInvitesBlock {
    pub player_blocking: usize,
}
#[derive(Copy, Clone)]
pub struct AssassinateBlockInvitesChallenge {
    pub player_blocking: usize,
}
#[derive(Copy, Clone)]
pub struct AssassinateBlockChallenged {
    pub player_blocking: usize,
    pub player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct AssassinateBlockChallengerFailed {
    pub player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct AssassinateSucceeded {
    pub player_blocking: usize,
}
#[derive(Copy, Clone)]
pub struct AssassinateChallenged {
    pub player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct AssassinateChallengerFailed {
    pub player_challenger: usize,
}

impl CoupTransition for AssassinateInvitesChallenge {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateInvitesBlock {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateBlockInvitesChallenge {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateBlockChallenged {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateBlockChallengerFailed {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateSucceeded {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateChallenged {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateChallengerFailed {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}