use crate::prob_manager::engine::constants::GAIN_DUKE;
use crate::prob_manager::engine::models::turn_start::TurnStart;
use crate::prob_manager::engine::models::{engine_state::CoupTransition};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
use super::game_state::GameData;
#[derive(Copy, Clone)]
pub struct TaxInvitesChallenge {
    pub player_turn: usize,
}
#[derive(Copy, Clone)]
pub struct TaxChallenged {
    pub player_turn: usize,
    pub player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct TaxChallengerFailed {
    pub player_turn: usize,
    pub player_challenger: usize,
}

impl CoupTransition for TaxInvitesChallenge {
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
                        game_data.coins[self.player_turn] += GAIN_DUKE;
                        EngineState::TurnStart(
                            TurnStart {  
                                player_turn: self.player_turn,
                            }
                        )
                    },
                    false => {
                        EngineState::TaxChallenged(
                            TaxChallenged { 
                                player_turn: self.player_turn,
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
    
    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::CollectiveChallenge { opposing_player_id, final_actioner, .. } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody blocked
                        game_data.coins[self.player_turn] -= GAIN_DUKE;
                    },
                    false => {
                        
                    },
                }
            },
            _ => {
                debug_assert!(false, "Illegal Move!")
            }
        }
    }
}
impl CoupTransition for TaxChallenged {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::RevealRedraw { .. } => {
                game_data.coins[self.player_turn] += GAIN_DUKE;
                EngineState::TaxChallengerFailed(
                    TaxChallengerFailed { 
                        player_turn: self.player_turn,
                        player_challenger: self.player_challenger, 
                    }
                )
            },
            ActionObservation::Discard { .. } => {
                EngineState::TurnStart(
                    TurnStart {  
                        player_turn: self.player_turn,
                    }
                )
            },
            _ => {
                panic!("Illegal Move");
            }
        }
    }
    
    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::RevealRedraw { .. } => {
                game_data.coins[self.player_turn] -= GAIN_DUKE;
            },
            ActionObservation::Discard { .. } => {
            },
            _ => {
                debug_assert!(false, "Illegal Move");
            }
        }
    }
}
impl CoupTransition for TaxChallengerFailed {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, _game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Discard { .. } => {
                EngineState::TurnStart(
                    TurnStart {  
                        player_turn: self.player_turn,
                    }
                )
            },
            _ => {
                panic!("Illegal Move");
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