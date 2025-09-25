use super::engine_state::EngineState;
use super::turn_start::TurnStart;
use crate::prob_manager::engine::constants::{STARTING_COINS, STARTING_INFLUENCE};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GameData {
    pub influence: [u8; 6],
    pub coins: [u8; 6],
    pub players_alive: u8,
}

impl Default for GameData {
    fn default() -> Self {
        Self::new()
    }
}

impl GameData {
    pub fn new() -> Self {
        GameData {
            influence: STARTING_INFLUENCE,
            coins: STARTING_COINS,
            players_alive: 6,
        }
    }
    pub fn influence(&self) -> &[u8; 6] {
        &self.influence
    }
    pub fn add_influence(&mut self, player: usize, amount: u8) {
        self.players_alive += (self.influence[player] == 0) as u8;
        self.influence[player] += amount;
    }
    pub fn sub_influence(&mut self, player: usize, amount: u8) {
        self.influence[player] -= amount;
        self.players_alive -= (self.influence[player] == 0) as u8;
    }
    pub fn coins(&self) -> &[u8; 6] {
        &self.coins
    }
    pub fn add_coins(&mut self, player: usize, amount: u8) {
        self.coins[player] += amount;
    }
    pub fn sub_coins(&mut self, player: usize, amount: u8) {
        self.coins[player] -= amount;
    }
    /// Returns an iterator over all players alive
    pub fn players_alive(&self) -> impl Iterator<Item = usize> + '_ {
        self.influence
            .iter()
            .enumerate()
            .filter_map(|(player, influence)| (*influence > 0).then_some(player))
    }
    /// Returns an iterator over all players alive and not player
    pub fn player_targets_alive(&self, player: usize) -> impl Iterator<Item = usize> + '_ {
        self.influence
            .iter()
            .enumerate()
            .filter_map(move |(opposing_player, influence)| {
                (*influence > 0 && opposing_player != player).then_some(opposing_player)
            })
    }
    /// Returns an iterator over all players alive and not player that have coins
    pub fn player_targets_steal(&self, player: usize) -> impl Iterator<Item = usize> + '_ {
        self.influence
            .iter()
            .enumerate()
            .zip(self.coins.iter())
            .filter_map(move |((opposing_player, influence), coins)| {
                (*influence > 0 && *coins > 0 && opposing_player != player)
                    .then_some(opposing_player)
            })
    }
    /// Checks if game will be won after a player loses no_cards
    pub fn game_will_be_won(&self, player: usize, no_cards: u8) -> bool {
        // self.influence.iter().enumerate().filter(
        //     |(p, l)| {
        //         if *p != player {
        //             **l > 0
        //         } else {
        //             **l > no_cards
        //         }
        //     }
        // ).count() == 1
        debug_assert!(self.influence()[player] != 0, "We assume player is alive");
        self.players_alive == 2 && self.influence()[player] <= no_cards
    }
    /// Returns Coin amount zeroized for dead players
    pub fn coins_display(&self) -> [u8; 6] {
        let mut output = self.coins;
        self.influence
            .iter()
            .enumerate()
            .for_each(|(p, l)| output[p] *= (*l > 0) as u8);
        output
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GameState {
    pub game_data: GameData,
    pub engine_state: EngineState,
}

impl GameState {
    pub fn new(player_turn: usize) -> Self {
        GameState {
            game_data: GameData::new(),
            engine_state: EngineState::TurnStart(TurnStart { player_turn }),
        }
    }
    pub fn start(player_turn: usize) -> Self {
        GameState {
            game_data: GameData::new(),
            engine_state: EngineState::TurnStart(TurnStart { player_turn }),
        }
    }
    pub fn influence(&self) -> &[u8; 6] {
        self.game_data.influence()
    }
    pub fn coins(&self) -> &[u8; 6] {
        &self.game_data.coins
    }
}
