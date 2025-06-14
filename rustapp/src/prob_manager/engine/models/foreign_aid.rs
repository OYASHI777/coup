use crate::prob_manager::engine::constants::GAIN_FOREIGNAID;
use crate::prob_manager::engine::models::engine_state::CoupTransition;
use crate::history_public::ActionObservation;
use crate::prob_manager::engine::models::turn_start::TurnStart;
use super::game_state::GameData;
use super::engine_state::EngineState;
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
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::CollectiveBlock { opposing_player_id, final_actioner , .. } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody blocked
                        game_data.coins[game_data.player_turn] += GAIN_FOREIGNAID;
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

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::CollectiveBlock { opposing_player_id, final_actioner , .. } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody blocked
                        game_data.prev_player();
                        game_data.coins[game_data.player_turn] -= GAIN_FOREIGNAID;
                    },
                    false => {
                        // final_actioner blocked
                    },
                }
            },
            _ => {
                panic!("illegal move!")
            },
        }
    }
}
impl CoupTransition for ForeignAidBlockInvitesChallenge {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
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

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::CollectiveChallenge { opposing_player_id, final_actioner, .. } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody challenged
                        game_data.prev_player();
                    },
                    false => {
                        // final_actioner challenged
                    },
                }
            }
            _ => {
                panic!("Illegal move!")
            }
        }
    }
}
impl CoupTransition for ForeignAidBlockChallenged {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
                debug_assert!(*no_cards == 1, "no_cards: {no_cards} should be 1");
                // TODO: Chain blocks!
                // game_data.coins[game_data.player_turn] += GAIN_FOREIGNAID;
                // game_data.next_player();
                // EngineState::TurnStart(
                //     TurnStart {  }
                // )
                EngineState::ForeignAidInvitesBlock(
                    ForeignAidInvitesBlock {  }
                )
            }
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
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

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
                debug_assert!(*no_cards == 1, "no_cards: {no_cards} should be 1");
                // game_data.prev_player();
                // game_data.coins[game_data.player_turn] -= GAIN_FOREIGNAID;
            }
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
            },
            _ => {
                panic!("Illegal move!")
            }
        }
    }
}
impl CoupTransition for ForeignAidBlockChallengerFailed {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
                if *player_id != self.player_challenger {
                    panic!("Illegal Move");
                }
                game_data.next_player();
                EngineState::TurnStart(
                    TurnStart {  }
                )
            },
            _ => {
                panic!("Illegal move!")
            }
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
                if *player_id != self.player_challenger {
                    panic!("Illegal Move");
                }
                game_data.prev_player();
            },
            _ => {
                panic!("Illegal move!")
            }
        }
    }
}