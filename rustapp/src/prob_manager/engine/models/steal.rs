use super::game_state::GameData;
use super::engine_state::EngineState;
use crate::{prob_manager::engine::models::engine_state::CoupTransition};
use crate::history_public::{ActionObservation, Card};
#[derive(Copy, Clone)]
pub struct StealInvitesChallenge {
    pub player_turn: usize,
}
#[derive(Copy, Clone)]
pub struct StealChallenged {
    pub player_turn: usize,
    player_blocking: usize,
    player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct StealChallengerFailed {
    pub player_turn: usize,
    player_blocking: usize,
    player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct StealInvitesBlock {
    pub player_turn: usize,
    player_blocking: u8,
}
#[derive(Copy, Clone)]
pub struct StealBlockInvitesChallenge {
    pub player_turn: usize,
    player_blocking: u8,
    card_blocker: Card,
}
#[derive(Copy, Clone)]
pub struct StealBlockChallenged {
    pub player_turn: usize,
    player_blocking: u8,
    player_challenger: u8,
    card_blocker: Card,
}
#[derive(Copy, Clone)]
pub struct StealBlockChallengerFailed {
    pub player_turn: usize,
    player_challenger: u8,
}

impl CoupTransition for StealInvitesChallenge {
    fn state_enter_update(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for StealChallenged {
    fn state_enter_update(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for StealChallengerFailed {
    fn state_enter_update(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for StealInvitesBlock {
    fn state_enter_update(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for StealBlockInvitesChallenge {
    fn state_enter_update(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for StealBlockChallenged {
    fn state_enter_update(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for StealBlockChallengerFailed {
    fn state_enter_update(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}