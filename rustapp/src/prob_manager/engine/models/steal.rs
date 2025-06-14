use super::game_state::GameData;
use super::engine_state::{EngineState, EngineStateName};
use crate::{prob_manager::engine::models::engine_state::CoupTransition};
use crate::history_public::{ActionObservation, Card};
#[derive(Copy, Clone)]
pub struct StealInvitesChallenge {
}
#[derive(Copy, Clone)]
pub struct StealChallenged {
    player_blocking: usize,
    player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct StealChallengerFailed {
    player_blocking: usize,
    player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct StealInvitesBlock {
    player_blocking: u8,
}
#[derive(Copy, Clone)]
pub struct StealBlockInvitesChallenge {
    player_blocking: u8,
    card_blocker: Card,
}
#[derive(Copy, Clone)]
pub struct StealBlockChallenged {
    player_blocking: u8,
    player_challenger: u8,
    card_blocker: Card,
}
#[derive(Copy, Clone)]
pub struct StealBlockChallengerFailed {
    player_challenger: u8,
}

impl CoupTransition for StealInvitesChallenge {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for StealChallenged {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for StealChallengerFailed {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for StealInvitesBlock {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for StealBlockInvitesChallenge {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for StealBlockChallenged {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for StealBlockChallengerFailed {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}