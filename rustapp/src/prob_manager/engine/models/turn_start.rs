use std::hint::unreachable_unchecked;

use crate::{prob_manager::engine::{fsm_engine::Node, models::engine_state::{CoupTransition, EngineState}}, traits::prob_manager::coup_analysis::CoupTraversal};
use super::game_state::GameState;
use crate::history_public::ActionObservation;
use crate::prob_manager::engine::constants::INCOME_GAIN;
pub struct TurnStart {
    pub game_state: GameState,
}

impl TurnStart {
    pub fn new() -> Self {
        TurnStart { 
            game_state: GameState::new(),
        }
    }
}

impl CoupTransition for TurnStart {
    fn next(self, action: &ActionObservation) -> EngineState {
        match action {
            ActionObservation::Income { player_id } => {
                let mut game_state = self.game_state;
                game_state.coins_add(*player_id, INCOME_GAIN);
                EngineState::TurnStart(TurnStart { game_state, })
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

    fn prev(self, action: &ActionObservation) -> EngineState {
        match action {
            ActionObservation::EmptyAO => todo!(),
            ActionObservation::ChallengeAccept => todo!(),
            ActionObservation::ChallengeDeny => todo!(),
            ActionObservation::Income { player_id } => todo!(),
            ActionObservation::ForeignAid { player_id } => todo!(),
            ActionObservation::Tax { player_id } => todo!(),
            ActionObservation::Steal { player_id, opposing_player_id, amount } => todo!(),
            ActionObservation::Assassinate { player_id, opposing_player_id } => todo!(),
            ActionObservation::Coup { player_id, opposing_player_id } => todo!(),
            ActionObservation::CollectiveChallenge { participants, opposing_player_id, final_actioner } => todo!(),
            ActionObservation::CollectiveBlock { participants, opposing_player_id, final_actioner } => todo!(),
            ActionObservation::BlockSteal { player_id, opposing_player_id, card } => todo!(),
            ActionObservation::BlockAssassinate { player_id, opposing_player_id } => todo!(),
            ActionObservation::Discard { player_id, card, no_cards } => todo!(),
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => todo!(),
            ActionObservation::Exchange { player_id } => todo!(),
            ActionObservation::ExchangeDraw { player_id, card } => todo!(),
            ActionObservation::ExchangeChoice { player_id, relinquish } => todo!(),
        }
    }
}

impl Node for TurnStart {
    fn dispatch(&self) -> bool {
        false
    }
}