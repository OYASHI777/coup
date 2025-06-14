use crate::prob_manager::engine::models::turn_start::TurnStart;
use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
use super::game_state::GameData;
#[derive(Copy, Clone)]
pub struct ExchangeInvitesChallenge {
}
#[derive(Copy, Clone)]
pub struct ExchangeDrawing {
}
#[derive(Copy, Clone)]
pub struct ExchangeDrawn {
}
#[derive(Copy, Clone)]
pub struct ExchangeChallenged {
    pub player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct ExchangeChallengerFailed {
    pub player_challenger: usize,
}

impl CoupTransition for ExchangeInvitesChallenge {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::CollectiveChallenge { participants, opposing_player_id, final_actioner } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody challenged
                        EngineState::ExchangeDrawing(ExchangeDrawing {  })
                    },
                    false => {
                        EngineState::ExchangeChallenged(ExchangeChallenged { player_challenger: *final_actioner })
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
            },
            _ => {
                panic!("Illegal Move");
            }
        }
    }
}
impl CoupTransition for ExchangeDrawing {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::ExchangeDraw { player_id, card } => {
                EngineState::ExchangeDrawn(ExchangeDrawn {  })
            },
            _ => panic!("Illegal Move"),
        }
    }

    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::ExchangeDraw { player_id, card } => {
            },
            _ => panic!("Illegal Move"),
        }
    }
}
impl CoupTransition for ExchangeDrawn {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::ExchangeChoice { player_id, relinquish } => {
                game_data.next_player();
                EngineState::TurnStart(TurnStart {  })
            },
            _ => panic!("Illegal Move"),
        }
    }

    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::ExchangeChoice { player_id, relinquish } => {
                game_data.prev_player();
            },
            _ => panic!("Illegal Move"),
        }
    }
}
impl CoupTransition for ExchangeChallenged {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
                EngineState::ExchangeChallengerFailed(ExchangeChallengerFailed { player_challenger: self.player_challenger })
            },
            ActionObservation::Discard { player_id, card, no_cards } => {
                game_data.next_player();
                EngineState::TurnStart(TurnStart {  })
            },
            _ => panic!("Illegal Move"),
        }
    }

    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
            },
            ActionObservation::Discard { player_id, card, no_cards } => {
                game_data.prev_player();
            },
            _ => panic!("Illegal Move"),
        }
    }
}
impl CoupTransition for ExchangeChallengerFailed {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
                EngineState::ExchangeDrawing(ExchangeDrawing {  })
            },
            _ => panic!("Illegal Move"),
        }
    }

    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
            },
            _ => panic!("Illegal Move"),
        }
    }
}