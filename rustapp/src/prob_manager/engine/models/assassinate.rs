use super::end::End;
use super::turn_start::TurnStart;
use super::engine_state::CoupTransition;
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
use super::game_state::GameData;
// TODO: [NOTE] Discard provided in legal moves should consider how many cards a player has alive
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AssassinateInvitesChallenge {
    pub player_turn: usize,
    pub player_blocking: usize,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AssassinateInvitesBlock {
    pub player_turn: usize,
    pub player_blocking: usize,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AssassinateBlockInvitesChallenge {
    pub player_turn: usize,
    pub player_blocking: usize,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AssassinateBlockChallenged {
    pub player_turn: usize,
    pub player_blocking: usize,
    pub player_challenger: usize,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AssassinateBlockChallengerFailed {
    pub player_turn: usize,
    pub player_challenger: usize,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AssassinateSucceeded {
    pub player_turn: usize,
    pub player_blocking: usize,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AssassinateChallenged {
    pub player_turn: usize,
    pub player_blocking: usize,
    pub player_challenger: usize,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AssassinateChallengerFailed {
    pub player_turn: usize,
    pub player_blocking: usize,
    pub player_challenger: usize,
}

impl CoupTransition for AssassinateInvitesChallenge {
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
                        EngineState::AssassinateInvitesBlock(
                            AssassinateInvitesBlock { 
                                player_turn: self.player_turn, 
                                player_blocking: self.player_blocking,
                            }
                        )
                    },
                    false => {
                        EngineState::AssassinateChallenged(
                            AssassinateChallenged { 
                                player_turn: self.player_turn,
                                player_blocking: self.player_blocking,
                                player_challenger: *final_actioner,
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
                ActionObservation::CollectiveChallenge { .. } => true,
                _ => false,
            }
        )
    }
}
impl CoupTransition for AssassinateInvitesBlock {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match (game_data.influence()[self.player_blocking], action) {
            (0, _) => {
                EngineState::TurnStart(
                    TurnStart { 
                        player_turn: self.player_turn,
                    }
                )
            },
            (1, ActionObservation::BlockAssassinate { player_id, opposing_player_id }) 
            | (2, ActionObservation::BlockAssassinate { player_id, opposing_player_id }) => {
                match player_id == opposing_player_id {
                    true => {
                        // No Block
                        EngineState::AssassinateSucceeded(
                            AssassinateSucceeded { 
                                player_turn: self.player_turn, 
                                player_blocking: self.player_blocking,
                            }
                        )
                    },
                    false => {
                        // player_id is the defender/blocking-player
                        // Opposing_player_id is the attacker
                        EngineState::AssassinateBlockInvitesChallenge(
                            AssassinateBlockInvitesChallenge { 
                                player_turn: self.player_turn, 
                                player_blocking: self.player_blocking, 
                            }
                        )
                    },
                }
            },
            _ => {

                panic!("Illegal! player lives: {}, for player: {}", game_data.influence()[self.player_blocking], self.player_blocking);
            },
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, _game_data: &mut GameData) {
        debug_assert!(
            match action {
                ActionObservation::BlockAssassinate { .. } => true,
                _ => false,
            },
            "Illegal Move!"
        )
    }
}
impl CoupTransition for AssassinateBlockInvitesChallenge {
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
                        EngineState::AssassinateBlockChallenged(
                            AssassinateBlockChallenged { 
                                player_turn: self.player_turn, 
                                player_blocking: self.player_blocking, 
                                player_challenger: *final_actioner, 
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
                ActionObservation::CollectiveChallenge { .. } => true,
                _ => false,
            },
            "Illegal Move!"
        )
    }
}
impl CoupTransition for AssassinateBlockChallenged {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::RevealRedraw { .. } => {
                EngineState::AssassinateBlockChallengerFailed(
                    AssassinateBlockChallengerFailed { 
                        player_turn: self.player_turn, 
                        player_challenger: self.player_challenger,
                    }
                )
            },
            ActionObservation::Discard { player_id, no_cards, .. } => {
                // NOTE: Cards cannot be Contessa!
                // NOTE: This would be discard the WHOLE HAND
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
impl CoupTransition for AssassinateBlockChallengerFailed {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        // TODO: [OPTIMIZE] You can technically just return TurnStart if you have a check wrapper outside this!
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
impl CoupTransition for AssassinateSucceeded {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            // TODO: [OPTIMIZE] You can technically just return TurnStart if you have a check wrapper outside this!
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
                panic!("Illegal Move!")
            },
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, _game_data: &mut GameData) {
        debug_assert!(
            match action {
                // TODO: [OPTIMIZE] You can technically just return TurnStart if you have a check wrapper outside this!
                ActionObservation::Discard { .. } => true,
                _ => false,
            },
            "Illegal Move!"
        )
    }
}
impl CoupTransition for AssassinateChallenged {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::RevealRedraw { .. } => {
                EngineState::AssassinateChallengerFailed(
                    AssassinateChallengerFailed { 
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
impl CoupTransition for AssassinateChallengerFailed {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            // TODO: [OPTIMIZE] You can technically just return TurnStart if you have a check wrapper outside this!
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
                            EngineState::AssassinateInvitesBlock(
                                AssassinateInvitesBlock { 
                                    player_turn: self.player_turn,
                                    player_blocking: self.player_blocking,
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
                            EngineState::AssassinateInvitesBlock(
                                AssassinateInvitesBlock { 
                                    player_turn: self.player_turn,
                                    player_blocking: self.player_blocking,
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