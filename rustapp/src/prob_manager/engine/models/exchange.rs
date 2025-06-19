use super::end::End;
use super::turn_start::TurnStart;
use super::engine_state::CoupTransition;
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
use super::game_state::GameData;
#[derive(Copy, Clone)]
pub struct ExchangeInvitesChallenge {
    pub player_turn: usize,
}
#[derive(Copy, Clone)]
pub struct ExchangeDrawing {
    pub player_turn: usize,
}
#[derive(Copy, Clone)]
pub struct ExchangeDrawn {
    pub player_turn: usize,
}
#[derive(Copy, Clone)]
pub struct ExchangeChallenged {
    pub player_turn: usize,
    pub player_challenger: usize,
}
#[derive(Copy, Clone)]
pub struct ExchangeChallengerFailed {
    pub player_turn: usize,
    pub player_challenger: usize,
}

impl CoupTransition for ExchangeInvitesChallenge {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, _game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::CollectiveChallenge { opposing_player_id, final_actioner, .. } => {
                match opposing_player_id == final_actioner {
                    true => {
                        // nobody challenged
                        EngineState::ExchangeDrawing(
                            ExchangeDrawing {  
                                player_turn: self.player_turn,
                            }
                        )
                    },
                    false => {
                        EngineState::ExchangeChallenged(
                            ExchangeChallenged { 
                                player_turn: self.player_turn,
                                player_challenger: *final_actioner 
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

    fn state_leave_reverse(&self, action: &ActionObservation, _game_data: &mut GameData) {
        debug_assert!(
            match action {
                ActionObservation::CollectiveChallenge { .. } => true,
                _ => false,
            },
            "Illegal Move!"
        )
    }
}
impl CoupTransition for ExchangeDrawing {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, _game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::ExchangeDraw { .. } => {
                EngineState::ExchangeDrawn(
                    ExchangeDrawn {  
                        player_turn: self.player_turn,
                    }
                )
            },
            _ => panic!("Illegal Move"),
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, _game_data: &mut GameData) {
        debug_assert!(
            match action {
                ActionObservation::ExchangeDraw { .. } => true,
                _ => false,
            },
            "Illegal Move!"
        )
    }
}
impl CoupTransition for ExchangeDrawn {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, _game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::ExchangeChoice { .. } => {
                EngineState::TurnStart(
                    TurnStart {  
                        player_turn: self.player_turn,
                    }
                )
            },
            _ => panic!("Illegal Move"),
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, _game_data: &mut GameData) {
        debug_assert!(
            match action {
                ActionObservation::ExchangeChoice { .. } => true,
                _ => false,
            },
            "Illegal Move!"
        )
    }
}
impl CoupTransition for ExchangeChallenged {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::RevealRedraw { .. } => {
                EngineState::ExchangeChallengerFailed(
                    ExchangeChallengerFailed { 
                        player_turn: self.player_turn,
                        player_challenger: self.player_challenger 
                    }
                )
            },
            ActionObservation::Discard { player_id, no_cards, .. } => {
                match game_data.game_will_be_won(*player_id, *no_cards as u8) {
                    true => {
                        EngineState::End(End { })
                    },
                    false => {
                        EngineState::TurnStart(
                            TurnStart { 
                                player_turn: self.player_turn,
                            }
                        )
                    },
                }
            },
            _ => panic!("Illegal Move"),
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, _game_data: &mut GameData) {
        debug_assert!(
            match action {
                ActionObservation::RevealRedraw { .. } 
                | ActionObservation::Discard { .. } => true,
                _ => false,
            },
            "Illegal Move!"
        )
    }
}
impl CoupTransition for ExchangeChallengerFailed {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, action: &ActionObservation, _game_data: &mut GameData) -> EngineState {
        match action {
            ActionObservation::Discard { .. } => {
                EngineState::ExchangeDrawing(
                    ExchangeDrawing {  
                        player_turn: self.player_turn,
                    }
                )
            },
            _ => panic!("Illegal Move"),
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