use crate::prob_manager::engine::models::engine_state::CoupTransition;
use crate::history_public::ActionObservation;
use crate::prob_manager::engine::models::turn_start::TurnStart;
use super::game_state::{GameState, GameData};
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
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::CollectiveBlock { opposing_player_id, final_actioner , .. } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody blocked
                        game_data.next_player();
                        EngineState::TurnStart(TurnStart {  })
                    },
                    false => {
                        // final_actioner blocked
                        EngineState::ForeignAidBlockInvitesChallenge(ForeignAidBlockInvitesChallenge { player_blocking: *final_actioner })
                    },
                }
            },
            _ => {
                panic!("illegal move!")
            },
        }
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        EngineState::TurnStart(TurnStart {  })
    }
}
impl CoupTransition for ForeignAidBlockInvitesChallenge {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ForeignAidBlockChallenged {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}
impl CoupTransition for ForeignAidBlockChallengerFailed {
    fn next(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn prev(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }
}