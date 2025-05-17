pub mod card_state_u64;
use crate::history_public::Card;
pub trait CardPermState: Sized {
    fn from_str(str: &str) -> Self;
    fn gen_table_combinations() -> Vec<Self>;
    /// Returns true if a player has all of the cards
    fn player_has_cards(&self, player_id: usize, cards: &[Card]) -> bool;
    // Returns the total number of particular card a player has
    fn player_card_count(&self, player_id: usize, card: Card) -> u8;
    // Returns the total number all cards a player has
    fn player_card_counts(&self, player_id: usize) -> [u8; 5];
    /// RevealRedraw: Mixes a single known card of a player and another player  
    /// Returns all the possible outcome combinations except self!
    fn mix_one_card(&self, player_id: usize, player_other: usize, card: Card) -> Vec<Self>;
    /// Ambassador: Mixes all the player's cards with player 6 (pile)  
    /// Returns all possible outcome combinations except self!
    fn mix_multiple_chars_with_player6(&self, player_id: usize, player_dead_cards: &[Card]) -> Vec<Self>;
    /// RevealRedraw: Swaps reveal card with redraw card
    /// Returns a new state if possible
    fn player_swap_cards(&self, player_i: usize, player_j: usize, card_i: Card, card_j: Card) -> Option<Self>;
}