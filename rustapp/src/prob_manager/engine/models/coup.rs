use crate::prob_manager::engine::models::end::End;
use crate::prob_manager::engine::models::turn_start::TurnStart;
use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
use super::game_state::GameData;
#[derive(Copy, Clone)]
pub struct CoupHit {
    pub player_hit: usize,
}

impl CoupTransition for CoupHit {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
                match *player_id == self.player_hit {
                    true => {
                        match game_data.game_will_be_won(*player_id, *no_cards as u8) {
                            true => {
                                EngineState::End(End {  })
                            },
                            false => {
                                EngineState::TurnStart(TurnStart {  })
                            },
                        }
                    },
                    false => {
                        panic!("Illegal Move");
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
            ActionObservation::Discard { player_id, card, no_cards } => {
                match *player_id == self.player_hit {
                    true => {},
                    false => {
                        panic!("Illegal Move");
                    },
                }
            },
            _ => {
                panic!("Illegal Move");
            }
        }
    }
}