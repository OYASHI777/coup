use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::{EngineState, EngineStateName};
use super::game_state::GameData;
#[derive(Copy, Clone)]
pub struct AssassinateInvitesChallenge {
}
#[derive(Copy, Clone)]
pub struct AssassinateInvitesBlock {
    player_blocking: usize,
}
#[derive(Copy, Clone)]
pub struct AssassinateBlockInvitesChallenge {
    player_blocking: usize,
}
#[derive(Copy, Clone)]
pub struct AssassinateBlockChallenged {
    player_blocking: usize,
    player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct AssassinateBlockChallengerFailed {
    player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct AssassinateSucceeded {
    player_blocking: usize,
}
#[derive(Copy, Clone)]
pub struct AssassinateChallenged {
    player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct AssassinateChallengerFailed {
    player_challenger: usize,
}

impl CoupTransition for AssassinateInvitesChallenge {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateInvitesBlock {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateBlockInvitesChallenge {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateBlockChallenged {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateBlockChallengerFailed {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateSucceeded {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateChallenged {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for AssassinateChallengerFailed {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}