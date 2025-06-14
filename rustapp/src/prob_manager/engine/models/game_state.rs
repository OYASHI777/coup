use crate::prob_manager::engine::{constants::{STARTING_COINS, STARTING_INFLUENCE}};
use crate::traits::prob_manager::coup_analysis::CoupTraversal;
use super::engine_state::{CoupTransition, EngineState, EngineStateName};
use super::turn_start::TurnStart;
use crate::history_public::ActionObservation;

pub struct GameData {
    pub influence: [u8; 6],
    pub coins: [u8; 6],
    pub player_turn: usize,
}
impl GameData {
    pub fn new(player_turn: usize) -> Self {
        GameData { 
            influence: STARTING_INFLUENCE, 
            coins: STARTING_COINS, 
            player_turn,
        }
    }
}

impl GameData {
    pub fn next_player(&mut self) {
        let mut current_turn: usize = (self.player_turn + 1) % 6;
        while self.influence[current_turn] == 0 {
            current_turn = (current_turn + 1) % 6;
        }
        self.player_turn = current_turn;
    }
    /// Assumes previously deducted influence has already been readded in
    pub fn prev_player(&mut self) {
        let mut current_turn: usize = (self.player_turn + 5) % 6;
        while self.influence[current_turn] == 0 {
            current_turn = (current_turn + 5) % 6;
        }
        self.player_turn = current_turn;
    }
}

pub struct GameState {
    pub game_data: GameData,
    pub engine_state: EngineState,
}

impl GameState {
    pub fn new() -> Self {
        GameState { 
            game_data: GameData::new(0),
            engine_state: EngineState::TurnStart(TurnStart{ }),
        }
    }
    pub fn start(player_turn: usize) -> Self {
        GameState { 
            game_data: GameData::new(player_turn),
            engine_state: EngineState::TurnStart(TurnStart{ }),
        }
    }
    pub fn influence(&self) -> &[u8; 6] {
        &self.game_data.influence
    }
    pub fn coins(&self) -> &[u8; 6] {
        &self.game_data.coins
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
