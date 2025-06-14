use crate::prob_manager::engine::constants::GAIN_DUKE;
use crate::prob_manager::engine::models::turn_start::TurnStart;
use crate::prob_manager::engine::models::{engine_state::CoupTransition};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
use super::game_state::GameData;
#[derive(Copy, Clone)]
pub struct TaxInvitesChallenge {
}
#[derive(Copy, Clone)]
pub struct TaxChallenged {
    pub player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct TaxChallengerFailed {
    pub player_challenger: usize,
}

impl CoupTransition for TaxInvitesChallenge {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::CollectiveChallenge { participants, opposing_player_id, final_actioner } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody challenges
                        game_data.coins[game_data.player_turn] += GAIN_DUKE;
                        game_data.next_player();
                        EngineState::TurnStart(TurnStart {  })
                    },
                    false => {
                        EngineState::TaxChallenged(
                            TaxChallenged { 
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
    
    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::CollectiveChallenge { participants, opposing_player_id, final_actioner } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody blocked
                        game_data.prev_player();
                        game_data.coins[game_data.player_turn] -= GAIN_DUKE;
                    },
                    false => {
                        
                    },
                }
            },
            _ => {
                panic!("Illegal Move");
            }
        }
    }
}
impl CoupTransition for TaxChallenged {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
                game_data.coins[game_data.player_turn] += GAIN_DUKE;
                EngineState::TaxChallengerFailed(
                    TaxChallengerFailed { 
                        player_challenger: self.player_challenger, 
                    }
                )
            },
            ActionObservation::Discard { player_id, card, no_cards } => {
                game_data.next_player();
                EngineState::TurnStart(TurnStart {  })
            },
            _ => {
                panic!("Illegal Move");
            }
        }
    }
    
    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
                game_data.coins[game_data.player_turn] -= GAIN_DUKE;
            },
            ActionObservation::Discard { player_id, card, no_cards } => {
                game_data.prev_player();
            },
            _ => {
                panic!("Illegal Move");
            }
        }
    }
}
impl CoupTransition for TaxChallengerFailed {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
                game_data.next_player();
                EngineState::TurnStart(TurnStart {  })
            },
            _ => {
                panic!("Illegal Move");
            }
        }
    }
    
    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
                game_data.prev_player();
            },
            _ => {
                panic!("Illegal Move");
            }
        }
    }
}