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
    pub player_turn: usize,
}

impl TurnStart {
    pub fn new(player_turn: usize) -> Self {
        TurnStart { 
            player_turn,
        }
    }
    pub fn next_player(&mut self, influence: &[u8; 6]) {
        let mut current_turn: usize = (self.player_turn + 1) % 6;
        while influence[current_turn] == 0 {
            current_turn = (current_turn + 1) % 6;
        }
        self.player_turn = current_turn;
    }
}

impl CoupTransition for TurnStart {
    fn state_enter_update(&mut self, game_data: &mut GameData) {
        self.next_player(&game_data.influence);
    }
    fn state_enter_reverse(&mut self, game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Income { player_id } => {
                EngineState::TurnStart(
                    TurnStart { 
                        player_turn: self.player_turn,
                    }
                )
            },
            ActionObservation::Coup { player_id, opposing_player_id } => {
                EngineState::CoupHit(
                    CoupHit { 
                        player_turn: self.player_turn,
                        player_hit: *opposing_player_id,
                    }
                )
            }
            ActionObservation::ForeignAid { player_id } => {
                EngineState::ForeignAidInvitesBlock(
                    ForeignAidInvitesBlock {  
                        player_turn: self.player_turn,
                    }
                )
            },
            ActionObservation::Tax { player_id } => {
                EngineState::TaxInvitesChallenge(
                    TaxInvitesChallenge {  
                        player_turn: self.player_turn,
                    }
                )
            },
            ActionObservation::Steal { player_id, opposing_player_id, amount } => {
                todo!()
            },
            ActionObservation::Assassinate { player_id, opposing_player_id } => {
                todo!()
            },
            ActionObservation::Exchange { player_id } => {
                EngineState::ExchangeInvitesChallenge(
                    ExchangeInvitesChallenge {  
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
        // nothing
    }
}

impl Node for TurnStart {
    fn dispatch(&self) -> bool {
        false
    }
}