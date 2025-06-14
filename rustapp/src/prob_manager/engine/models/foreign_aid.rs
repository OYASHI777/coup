use crate::prob_manager::engine::constants::GAIN_FOREIGNAID;
use crate::prob_manager::engine::models::engine_state::CoupTransition;
use crate::history_public::ActionObservation;
use crate::prob_manager::engine::models::turn_start::TurnStart;
use super::game_state::GameData;
use super::engine_state::{EngineState, EngineStateName};
#[derive(Copy, Clone)]
pub struct ForeignAidInvitesBlock {
}
#[derive(Copy, Clone)]
pub struct ForeignAidBlockInvitesChallenge {
    pub player_blocking: usize,
}
#[derive(Copy, Clone)]
pub struct ForeignAidBlockChallenged {
    pub player_blocking: usize,
    pub player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct ForeignAidBlockChallengerFailed {
    pub player_challenger: usize,
}

impl CoupTransition for ForeignAidInvitesBlock {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::CollectiveBlock { opposing_player_id, final_actioner , .. } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody blocked
                        game_data.next_player();
                        EngineState::TurnStart(
                            TurnStart {  }
                        )
                    },
                    false => {
                        // final_actioner blocked
                        EngineState::ForeignAidBlockInvitesChallenge(
                            ForeignAidBlockInvitesChallenge { 
                                player_blocking: *final_actioner 
                            }
                        )
                    },
                }
            },
            _ => {
                panic!("illegal move!")
            },
        }
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for ForeignAidBlockInvitesChallenge {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::CollectiveChallenge { opposing_player_id, final_actioner, .. } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody challenged
                        game_data.next_player();
                        EngineState::TurnStart( 
                            TurnStart {  }
                        )
                    },
                    false => {
                        // final_actioner challenged
                        EngineState::ForeignAidBlockChallenged(
                            ForeignAidBlockChallenged { 
                                player_blocking: self.player_blocking, 
                                player_challenger: *final_actioner, 
                            }
                        )
                    },
                }
            }
            _ => {
                panic!("Illegal move!")
            }
        }
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for ForeignAidBlockChallenged {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
                debug_assert!(*no_cards == 1, "no_cards: {no_cards} should be 1");
                game_data.influence[*player_id] -= *no_cards as u8;
                game_data.next_player();
                EngineState::TurnStart(
                    TurnStart {  }
                )
            }
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
                game_data.coins[*player_id] += GAIN_FOREIGNAID;
                EngineState::ForeignAidBlockChallengerFailed(
                    ForeignAidBlockChallengerFailed { 
                        player_challenger: self.player_challenger, 
                    }
                )
            },
            _ => {
                panic!("Illegal move!")
            }
        }
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}
impl CoupTransition for ForeignAidBlockChallengerFailed {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        todo!()
    }

    fn reverse_state_update(&self, game_data: &mut GameData) {
        todo!()
    }
}