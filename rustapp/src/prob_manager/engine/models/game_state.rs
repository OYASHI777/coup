use crate::prob_manager::engine::constants::{STARTING_COINS, STARTING_INFLUENCE};
use super::engine_state::EngineState;
use super::turn_start::TurnStart;

#[derive(Clone)]
pub struct GameData {
    influence: [u8; 6],
    pub coins: [u8; 6],
}
impl GameData {
    pub fn new() -> Self {
        GameData { 
            influence: STARTING_INFLUENCE, 
            coins: STARTING_COINS, 
        }
    }
    pub fn influence(&self) -> &[u8; 6] {
        &self.influence
    }
    pub fn add_influence(&mut self, player: usize, amount: u8) {
        self.influence[player] += amount;
    }
    pub fn sub_influence(&mut self, player: usize, amount: u8) {
        self.influence[player] -= amount;
    }
    pub fn coins(&self) -> &[u8; 6] {
        &self.coins
    }
    pub fn add_coins(&mut self, player: usize, amount: u8) {
        todo!();
    }
    pub fn sub_coins(&mut self, player: usize, amount: u8) {
        todo!();
    }
}

impl GameData {
    /// Checks if game will be won after a player loses no_cards
    pub fn game_will_be_won(&self, player: usize, no_cards: u8) -> bool {
        self.influence.iter().enumerate().filter(
            |(p, l)| {
                if *p != player {
                    **l > 0
                } else {
                    **l > no_cards
                }
            }
        ).count() == 1
    }
    /// Returns Coin amount zeroized for dead players
    pub fn coins_display(&self) -> [u8; 6] {
        let mut output = self.coins.clone();
        self.influence.iter().enumerate().for_each(
            |(p, l)| {
                output[p] = output[p] * (*l > 0) as u8
            }
        );
        output
    }
}

#[derive(Clone)]
pub struct GameState {
    pub game_data: GameData,
    pub engine_state: EngineState,
}

impl GameState {
    pub fn new(player_turn: usize) -> Self {
        GameState { 
            game_data: GameData::new(),
            engine_state: EngineState::TurnStart(
                TurnStart{ 
                    player_turn,
                }
            ),
        }
    }
    pub fn start(player_turn: usize) -> Self {
        GameState { 
            game_data: GameData::new(),
            engine_state: EngineState::TurnStart(
                TurnStart{ 
                    player_turn,
                }
            ),
        }
    }
    pub fn influence(&self) -> &[u8; 6] {
        &self.game_data.influence()
    }
    pub fn coins(&self) -> &[u8; 6] {
        &self.game_data.coins
    }
}
