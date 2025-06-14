use crate::prob_manager::engine::models::turn_start::TurnStart;
use crate::prob_manager::engine::models::{engine_state::CoupTransition, game_state::GameState};
use crate::history_public::ActionObservation;
use super::engine_state::{EngineState, EngineStateName};
use super::game_state::GameData;
#[derive(Copy, Clone)]
pub struct CoupHit {
    pub player_hit: usize,
}

impl CoupTransition for CoupHit {
    fn state_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
                match *player_id == self.player_hit {
                    true => {
                        EngineState::TurnStart(TurnStart {  })
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

    fn reverse_state_update(&self, action: &ActionObservation, game_data: &mut GameData) {
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