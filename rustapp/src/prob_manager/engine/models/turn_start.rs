use crate::{prob_manager::engine::{fsm_engine::Node, models::engine_state::{CoupTransition, EngineState}}, traits::prob_manager::coup_analysis::CoupTraversal};
use super::game_state::GameState;
use crate::history_public::ActionObservation;
use super::game_state::GameData;
use super::coup::*;
use super::end::*;
use super::exchange::*;
use super::foreign_aid::*;
use super::steal::*;
use super::tax::*;
use super::assassinate::*;
#[derive(Copy, Clone)]
pub struct TurnStart {
}

impl TurnStart {
    pub fn new() -> Self {
        TurnStart { 
        }
    }
}

impl CoupTransition for TurnStart {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Income { player_id } => {
                EngineState::TurnStart(TurnStart { })
            },
            ActionObservation::Coup { player_id, opposing_player_id } => {
                EngineState::CoupHit(
                    CoupHit { 
                        player_hit: *opposing_player_id,
                    }
                )
            }
            ActionObservation::ForeignAid { player_id } => {
                EngineState::ForeignAidInvitesBlock(ForeignAidInvitesBlock {  })
            },
            ActionObservation::Tax { player_id } => {
                EngineState::TaxInvitesChallenge(TaxInvitesChallenge {  })
            },
            ActionObservation::Steal { player_id, opposing_player_id, amount } => {
                todo!()
            },
            ActionObservation::Assassinate { player_id, opposing_player_id } => {
                todo!()
            },
            ActionObservation::Exchange { player_id } => {
                EngineState::ExchangeInvitesChallenge(ExchangeInvitesChallenge {  })
            },
            _ => {
                panic!("Illegal Move");
            }
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        // nothing
    }
}

impl Node for TurnStart {
    fn dispatch(&self) -> bool {
        false
    }
}