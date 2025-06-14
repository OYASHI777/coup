use crate::prob_manager::engine::models::engine_state::CoupTransition;
use crate::history_public::ActionObservation;
use super::game_state::GameState;
use super::engine_state::EngineState;
pub struct ForeignAidInvitesBlock {
}
pub struct ForeignAidBlockInvitesChallenge {
    player_blocking: usize,
}
pub struct ForeignAidBlockChallenged {
    player_blocking: usize,
    player_challenger: usize,
}
pub struct ForeignAidBlockChallengerFailed {
    player_challenger: usize,
}

impl CoupTransition for ForeignAidInvitesBlock {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        match action {
            ActionObservation::ChallengeAccept { player_id, opposing_player_id } => todo!(),
            ActionObservation::ChallengeDeny => todo!(),
            _ => {
                panic!("illegal move!")
            },
        }
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ForeignAidBlockInvitesChallenge {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ForeignAidBlockChallenged {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ForeignAidBlockChallengerFailed {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        todo!()
    }
}