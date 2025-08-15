use crate::prob_manager::engine::constants::GAIN_FOREIGNAID;
use crate::history_public::ActionObservation;
use super::end::End;
use super::engine_state::CoupTransition;
use super::engine_state::EngineState;
use super::game_state::GameData;
use super::turn_start::TurnStart;
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ForeignAidInvitesBlock {
    pub player_turn: usize,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ForeignAidBlockInvitesChallenge {
    pub player_turn: usize,
    pub player_blocking: usize,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ForeignAidBlockChallenged {
    pub player_turn: usize,
    pub player_blocking: usize,
    pub player_challenger: usize,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ForeignAidBlockChallengerFailed {
    pub player_turn: usize,
    pub player_challenger: usize,
}

impl CoupTransition for ForeignAidInvitesBlock {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::CollectiveBlock { opposing_player_id, final_actioner , .. } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody blocked
                        // game_data.coins[self.player_turn] += GAIN_FOREIGNAID;
                        game_data.add_coins(self.player_turn, GAIN_FOREIGNAID);
                        EngineState::TurnStart(
                            TurnStart {  
                                player_turn: self.player_turn,
                            }
                        )
                    },
                    false => {
                        // final_actioner blocked
                        EngineState::ForeignAidBlockInvitesChallenge(
                            ForeignAidBlockInvitesChallenge { 
                                player_turn: self.player_turn,
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
                        // game_data.coins[self.player_turn] -= GAIN_FOREIGNAID;
                        game_data.sub_coins(self.player_turn, GAIN_FOREIGNAID);
                    },
                    false => {
                        // final_actioner blocked
                    },
                }
            },
            _ => {
                debug_assert!(false, "illegal move!")
            },
        }
    }
}
impl CoupTransition for ForeignAidBlockInvitesChallenge {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, _game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::CollectiveChallenge { opposing_player_id, final_actioner, .. } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody challenged
                        EngineState::TurnStart( 
                            TurnStart {  
                                player_turn: self.player_turn,
                            }
                        )
                    },
                    false => {
                        // final_actioner challenged
                        EngineState::ForeignAidBlockChallenged(
                            ForeignAidBlockChallenged { 
                                player_turn: self.player_turn,
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

    fn state_leave_reverse(&self, action: &ActionObservation, _game_data: &mut GameData) {
        debug_assert!(
            match action {
                ActionObservation::CollectiveChallenge { .. } => true,
                _ => false,
            },
            "Illegal Move!"
        )
    }
}
impl CoupTransition for ForeignAidBlockChallenged {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Discard { player_id, no_cards, .. } => {
                debug_assert!(*no_cards == 1, "no_cards: {no_cards} should be 1");
                // game_data.coins[self.player_turn] += GAIN_FOREIGNAID;
                game_data.add_coins(self.player_turn, GAIN_FOREIGNAID);
                match game_data.game_will_be_won(*player_id, *no_cards as u8) {
                    true => {
                        EngineState::End(End { })
                    },
                    false => {
                        EngineState::TurnStart(
                            TurnStart { 
                                player_turn: self.player_turn,
                            }
                        )
                    },
                }
                // TODO: Chain blocks!
                // EngineState::ForeignAidInvitesBlock(
                //     ForeignAidInvitesBlock {  }
                // )
            }
            ActionObservation::RevealRedraw { .. } => {
                EngineState::ForeignAidBlockChallengerFailed(
                    ForeignAidBlockChallengerFailed { 
                        player_turn: self.player_turn,
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
            ActionObservation::Discard { .. } => {
                // game_data.coins[self.player_turn] -= GAIN_FOREIGNAID;
                game_data.sub_coins(self.player_turn, GAIN_FOREIGNAID);
            },
            ActionObservation::RevealRedraw { .. } => {
            },
            _ => {
                debug_assert!(false, "Illegal move!")
            }
        }
    }
}
impl CoupTransition for ForeignAidBlockChallengerFailed {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Discard { player_id, no_cards, .. } => {
                match game_data.game_will_be_won(*player_id, *no_cards as u8) {
                    true => {
                        EngineState::End(End { })
                    },
                    false => {
                        EngineState::TurnStart(
                            TurnStart { 
                                player_turn: self.player_turn,
                            }
                        )
                    },
                }
            },
            _ => {
                panic!("Illegal move!")
            }
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, _game_data: &mut GameData) {
        debug_assert!(
            match action {
                ActionObservation::Discard { .. } => true,
                _ => false,
            },
            "Illegal Move!"
        )
    }
}