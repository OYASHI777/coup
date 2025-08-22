use crate::prob_manager::engine::constants::GAIN_STEAL;
use crate::history_public::{ActionObservation, Card};
use super::end::End;
use super::game_state::GameData;
use super::engine_state::EngineState;
use super::turn_start::TurnStart;
use super::engine_state::CoupTransition;
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct StealInvitesChallenge {
    pub player_turn: usize,
    pub player_blocking: usize,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct StealChallenged {
    pub player_turn: usize,
    pub player_blocking: usize,
    pub player_challenger: usize,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct StealChallengerFailed {
    pub player_turn: usize,
    pub player_blocking: usize,
    pub player_challenger: usize,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct StealInvitesBlock {
    pub player_turn: usize,
    pub player_blocking: usize,
    pub coins_stolen: u8,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct StealBlockInvitesChallenge {
    pub player_turn: usize,
    pub player_blocking: usize,
    pub card_blocker: Card,
    pub coins_stolen: u8,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct StealBlockChallenged {
    pub player_turn: usize,
    pub player_blocking: usize,
    pub player_challenger: usize,
    pub card_blocker: Card,
    pub coins_stolen: u8,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct StealBlockChallengerFailed {
    pub player_turn: usize,
    pub player_challenger: usize,
    pub coins_stolen: u8,
}

impl CoupTransition for StealInvitesChallenge {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::CollectiveChallenge { opposing_player_id, final_actioner, .. } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody challenges
                        EngineState::StealInvitesBlock(
                            StealInvitesBlock { 
                                player_turn: self.player_turn, 
                                player_blocking: self.player_blocking, 
                                coins_stolen: game_data.coins()[self.player_blocking].min(GAIN_STEAL),
                            }
                        )
                    },
                    false => {
                        EngineState::StealChallenged(
                            StealChallenged { 
                                player_turn: self.player_turn, 
                                player_blocking: self.player_blocking, 
                                player_challenger: *final_actioner,
                            }
                        )
                    },
                }
            },
            _ => {
                panic!("Illegal Move");
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
impl CoupTransition for StealChallenged {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::RevealRedraw { .. } => {
                EngineState::StealChallengerFailed(
                    StealChallengerFailed { 
                        player_turn: self.player_turn, 
                        player_blocking: self.player_blocking, 
                        player_challenger: self.player_challenger,
                    }
                )
            },
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
                panic!("Illegal Move!");
            }
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, _game_data: &mut GameData) {
        debug_assert!(
            match action {
                ActionObservation::RevealRedraw { .. } 
                | ActionObservation::Discard { .. } => true,
                _ => false,
            },
            "Illegal Move!"
        )
    }
}
impl CoupTransition for StealChallengerFailed {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Discard { player_id, no_cards, .. } => {
                if *player_id == self.player_blocking {
                    match game_data.influence()[self.player_blocking] < *no_cards as u8 {
                        true => {
                            // Player is dead already and cannot block
                            EngineState::TurnStart(
                                TurnStart { 
                                    player_turn: self.player_turn,
                                }
                            )
                        },
                        false => {
                            // Player is alive to block
                            EngineState::StealInvitesBlock(
                                StealInvitesBlock { 
                                    player_turn: self.player_turn, 
                                    player_blocking: self.player_blocking, 
                                    coins_stolen: game_data.coins()[self.player_blocking].min(GAIN_STEAL),
                                }
                            )
                        }
                    }
                } else {
                    match game_data.game_will_be_won(*player_id, *no_cards as u8) {
                        true => {
                            EngineState::End(End { })
                        },
                        false => {
                            EngineState::StealInvitesBlock(
                                StealInvitesBlock { 
                                    player_turn: self.player_turn, 
                                    player_blocking: self.player_blocking, 
                                    coins_stolen: game_data.coins()[self.player_blocking].min(GAIN_STEAL),
                                }
                            )
                        },
                    }
                }
            },
            _ => {
                panic!("Illegal Move!");
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
impl CoupTransition for StealInvitesBlock {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::BlockSteal { player_id, opposing_player_id, card } => {
                match player_id == opposing_player_id {
                    true => {
                        // pass on block
                        debug_assert!(*opposing_player_id != self.player_turn, "Illegal Move!");
                        game_data.sub_coins(self.player_blocking, self.coins_stolen);
                        game_data.add_coins(self.player_turn, self.coins_stolen);
                        EngineState::TurnStart(
                            TurnStart { 
                                player_turn: self.player_turn,
                            }
                        )
                    },
                    false => {
                        // player_id blocked
                        match card {
                            Card::Ambassador 
                            | Card::Captain => {
                                EngineState::StealBlockInvitesChallenge(
                                    StealBlockInvitesChallenge { 
                                        player_turn: self.player_turn, 
                                        player_blocking: self.player_blocking, 
                                        card_blocker: *card, 
                                        coins_stolen: self.coins_stolen,
                                    }
                                )
                            },
                            _ => {
                                panic!("Illegal Move!");
                            }
                        }
                    },
                }
            },
            _ => {
                panic!("Illegal Move!");
            }
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::BlockSteal { player_id, opposing_player_id, card } => {
                match player_id == opposing_player_id {
                    true => {
                        // pass on block
                        debug_assert!(*opposing_player_id == self.player_turn, "Illegal Move!");
                        game_data.add_coins(self.player_blocking, self.coins_stolen);
                        game_data.sub_coins(self.player_turn, self.coins_stolen);
                    },
                    false => {
                        // player_id blocked
                        match card {
                            Card::Ambassador 
                            | Card::Captain => {
                            },
                            _ => {
                                debug_assert!(false, "Illegal Move!");
                            }
                        }
                    },
                }
            },
            _ => {
                debug_assert!(false, "Illegal Move!");
            }
        }
    }
}
impl CoupTransition for StealBlockInvitesChallenge {
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
                        // No Challenger
                        EngineState::TurnStart(
                            TurnStart { 
                                player_turn: self.player_turn, 
                            }
                        )
                    },
                    false => {
                        EngineState::StealBlockChallenged(
                            StealBlockChallenged { 
                                player_turn: self.player_turn, 
                                player_blocking: self.player_blocking, 
                                player_challenger: *final_actioner, 
                                card_blocker: self.card_blocker, 
                                coins_stolen: self.coins_stolen,
                            }
                        )
                    },
                }
            },
            _ => {
                panic!("Illegal Move");
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
impl CoupTransition for StealBlockChallenged {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::RevealRedraw { .. } => {
                EngineState::StealBlockChallengerFailed(
                    StealBlockChallengerFailed { 
                        player_turn: self.player_turn, 
                        player_challenger: self.player_challenger, 
                        coins_stolen: self.coins_stolen, 
                    }
                )
            },
            ActionObservation::Discard { player_id, no_cards, .. } => {
                game_data.sub_coins(self.player_blocking, self.coins_stolen);
                game_data.add_coins(self.player_turn, self.coins_stolen);
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
                panic!("Illegal Move!");
            }
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::RevealRedraw { .. } => {
            },
            ActionObservation::Discard { .. } => {
                game_data.add_coins(self.player_blocking, self.coins_stolen);
                game_data.sub_coins(self.player_turn, self.coins_stolen);
            },
            _ => {
                debug_assert!(false, "Illegal Move!");
            }
        }
    }
}
impl CoupTransition for StealBlockChallengerFailed {
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
                panic!("Illegal Move!");
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