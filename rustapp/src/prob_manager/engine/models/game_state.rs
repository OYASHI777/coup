use crate::prob_manager::engine::models::{engine_state::EngineState, turn_start::TurnStart};

pub struct GameState {
    influence: [u8; 6],
    coins: [u8; 6],
    player_turn: usize,
    state: EngineState,
}

impl GameState {
    pub fn new() -> Self {
        GameState { 
            influence: [0; 6], 
            coins: [0; 6], 
            player_turn: 0,
            state: EngineState::TurnStart(TurnStart{ }),
        }
    }
    pub fn start(player_turn: usize) -> Self {
        GameState { 
            influence: [0; 6], 
            coins: [0; 6], 
            player_turn: player_turn,
            state: EngineState::TurnStart(TurnStart{ }),
        }
    }
    pub fn influence(&self) -> &[u8; 6] {
        &self.influence
    }
    pub fn influence_add(&mut self, player: usize, amount: u8) {
        self.influence[player] += amount;
    }
    pub fn influence_sub(&mut self, player: usize, amount: u8) {
        self.influence[player] -= amount;
    }
    pub fn coins(&self) -> &[u8; 6] {
        &self.coins
    }
    pub fn coins_add(&mut self, player: usize, amount: u8) {
        self.coins[player] += amount;
    }
    pub fn coins_sub(&mut self, player: usize, amount: u8) {
        self.coins[player] += amount;
    }
    pub fn next_player(influence: &mut [u8; 6], player_turn: &mut usize) {
        let mut current_turn: usize = (*player_turn + 1) % 6;
        while influence[current_turn] == 0 {
            current_turn = (current_turn + 1) % 6;
        }
        *player_turn = current_turn;
    }
    pub fn prev_player(influence: &mut [u8; 6], player_turn: &mut usize) {
        let mut current_turn: usize = (*player_turn + 5) % 6;
        while influence[current_turn] == 0 {
            current_turn = (current_turn + 5) % 6;
        }
        *player_turn = current_turn;
    }
}