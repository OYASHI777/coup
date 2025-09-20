use super::engine_state::{CoupTransition, EngineState};
use super::game_state::GameData;
use crate::history_public::ActionObservation;
use crate::prob_manager::engine::fsm_engine::Node;
use crate::prob_manager::engine::models_prelude::*;
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TurnStart {
    pub player_turn: usize,
}

impl TurnStart {
    pub fn new(player_turn: usize) -> Self {
        TurnStart { player_turn }
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
        self.next_player(&game_data.influence());
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(
        &self,
        action: &ActionObservation,
        _game_data: &mut GameData,
    ) -> EngineState {
        match action {
            ActionObservation::Income { .. } => EngineState::TurnStart(TurnStart {
                player_turn: self.player_turn,
            }),
            ActionObservation::Coup {
                opposing_player_id, ..
            } => EngineState::CoupHit(CoupHit {
                player_turn: self.player_turn,
                player_hit: *opposing_player_id,
            }),
            ActionObservation::ForeignAid { .. } => {
                EngineState::ForeignAidInvitesBlock(ForeignAidInvitesBlock {
                    player_turn: self.player_turn,
                })
            }
            ActionObservation::Tax { .. } => {
                EngineState::TaxInvitesChallenge(TaxInvitesChallenge {
                    player_turn: self.player_turn,
                })
            }
            ActionObservation::Steal {
                opposing_player_id, ..
            } => EngineState::StealInvitesChallenge(StealInvitesChallenge {
                player_turn: self.player_turn,
                player_blocking: *opposing_player_id,
            }),
            ActionObservation::Assassinate {
                opposing_player_id, ..
            } => EngineState::AssassinateInvitesChallenge(AssassinateInvitesChallenge {
                player_turn: self.player_turn,
                player_blocking: *opposing_player_id,
            }),
            ActionObservation::Exchange { .. } => {
                EngineState::ExchangeInvitesChallenge(ExchangeInvitesChallenge {
                    player_turn: self.player_turn,
                })
            }
            _ => {
                panic!("Illegal Move");
            }
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, _game_data: &mut GameData) {
        // nothing
        debug_assert!(
            match action {
                ActionObservation::Income { .. }
                | ActionObservation::Coup { .. }
                | ActionObservation::ForeignAid { .. }
                | ActionObservation::Tax { .. }
                | ActionObservation::Steal { .. }
                | ActionObservation::Assassinate { .. }
                | ActionObservation::Exchange { .. } => true,
                _ => false,
            },
            "Illegal Move!"
        )
    }
}

impl Node for TurnStart {
    fn dispatch(&self) -> bool {
        false
    }
}
