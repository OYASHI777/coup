use std::hint::unreachable_unchecked;

use crate::{prob_manager::engine::{fsm_engine::Node, models::engine_state::{CoupTransition, EngineState}}, traits::prob_manager::coup_analysis::CoupTraversal};
use super::game_state::GameState;
use crate::history_public::ActionObservation;
use crate::prob_manager::engine::constants::INCOME_GAIN;
use super::coup::*;
use super::end::*;
use super::exchange::*;
use super::foreign_aid::*;
use super::steal::*;
use super::tax::*;
use super::assassinate::*;
pub struct TurnStart {
}

impl TurnStart {
    pub fn new() -> Self {
        TurnStart { 
        }
    }
}

impl CoupTransition for TurnStart {
    fn next(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        match action {
            ActionObservation::Income { player_id } => {
                coins[*player_id] += INCOME_GAIN;
                GameState::next_player(influence, player_turn);
                EngineState::TurnStart(self)
            },
            ActionObservation::ForeignAid { player_id } => {
                todo!()
            },
            ActionObservation::Tax { player_id } => {
                todo!()
            },
            ActionObservation::Steal { player_id, opposing_player_id, amount } => {
                todo!()
            },
            ActionObservation::Assassinate { player_id, opposing_player_id } => {
                todo!()
            },
            ActionObservation::Exchange { player_id } => {
                todo!()
            },
            _ => {
                unsafe {
                    unreachable_unchecked()
                }
            }
        }
    }

    fn prev(self, action: &ActionObservation, influence: &mut [u8; 6], coins: &mut [u8; 6], player_turn: &mut usize) -> EngineState {
        match action {
            ActionObservation::Income { player_id } => {
                coins[*player_id] -= INCOME_GAIN;
                GameState::prev_player(influence, player_turn);
                EngineState::TurnStart(self)
            },
            ActionObservation::ForeignAid { player_id } => {
                EngineState::ForeignAidInvitesBlock(ForeignAidInvitesBlock {  })
            },
            ActionObservation::Tax { player_id } => {
                todo!()
            },
            ActionObservation::Steal { player_id, opposing_player_id, amount } => {
                todo!()
            },
            ActionObservation::Assassinate { player_id, opposing_player_id } => {
                todo!()
            },
            ActionObservation::Exchange { player_id } => {
                todo!()
            },
            _ => {
                unsafe {
                    unreachable_unchecked()
                }
            }
        }
    }
}

impl Node for TurnStart {
    fn dispatch(&self) -> bool {
        false
    }
}