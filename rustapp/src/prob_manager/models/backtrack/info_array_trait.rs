use crate::{history_public::Card, prob_manager::engine::constants::MAX_HAND_SIZE_PLAYER};

/// Trait abstracting the functionality needed for info arrays in backtracking
pub trait InfoArrayTrait: Clone + std::fmt::Debug {
    /// Create a new info array for public game start
    fn start_public() -> Self;

    /// Create a new info array for private game start with player cards
    fn start_private(player: usize, cards: &[Card; MAX_HAND_SIZE_PLAYER]) -> Self;

    /// Clone with only public data
    fn clone_public(&self) -> Self;

    /// Access to public constraints
    fn public_constraints(&self) -> &Vec<Vec<Card>>;
    fn public_constraints_mut(&mut self) -> &mut Vec<Vec<Card>>;
    fn sort_public_constraints(&mut self);

    /// Access to inferred constraints
    fn inferred_constraints(&self) -> &Vec<Vec<Card>>;
    fn inferred_constraints_mut(&mut self) -> &mut Vec<Vec<Card>>;
    fn sort_inferred_constraints(&mut self);
    fn set_inferred_constraints(&mut self, inferred_constraints: &Vec<Vec<Card>>);

    /// Access to impossible constraints (single cards) - these need to provide array-like access
    fn get_impossible_constraint(&self, player: usize, card: usize) -> bool;
    fn set_impossible_constraint(&mut self, player: usize, card: usize, value: bool);
    fn set_all_impossible_constraints(&mut self, player: usize, value: bool);

    /// Access to impossible constraints (paired cards)
    fn get_impossible_constraint_2(&self, player: usize, card1: usize, card2: usize) -> bool;
    fn set_impossible_constraint_2(
        &mut self,
        player: usize,
        card1: usize,
        card2: usize,
        value: bool,
    );
    fn set_all_impossible_constraints_2(&mut self, player: usize, value: bool);

    /// Access to impossible constraints (triple cards)
    fn get_impossible_constraint_3(&self, card1: usize, card2: usize, card3: usize) -> bool;
    fn set_impossible_constraint_3(
        &mut self,
        card1: usize,
        card2: usize,
        card3: usize,
        value: bool,
    );

    /// Direct array access methods (for compatibility with existing code)
    fn impossible_constraints(&self) -> [[bool; 5]; 7];
    fn impossible_constraints_paired(&self) -> [[[bool; 5]; 5]; 7];
    fn impossible_constraints_triple(&self) -> [[[bool; 5]; 5]; 5];

    /// Player utility methods
    fn player_cards_known<T>(&self, player_id: T) -> usize
    where
        T: Into<usize> + Copy;

    fn player_has_public_constraint<T>(&self, player_id: T, card: Card) -> bool
    where
        T: Into<usize> + Copy;

    fn player_has_inferred_constraint<T>(&self, player_id: T, card: Card) -> bool
    where
        T: Into<usize> + Copy;

    fn player_constraints_all_full<T>(&self, player_id: T, card: Card) -> bool
    where
        T: Into<usize> + Copy;

    /// Debug methods for logging/printing impossible constraints
    fn format_impossible_constraints(&self) -> String;
    fn format_impossible_constraints_2(&self) -> String;
    fn format_impossible_constraints_3(&self) -> String;

    /// Helper methods for complex constraint analysis
    fn count_possible_single_constraints(&self, player: usize) -> u8;
    fn find_only_possible_single_constraint(&self, player: usize) -> Option<usize>;

    /// Check if all cards of a specific type are dead (in public constraints)
    fn all_cards_dead(&self, card: Card) -> bool;
}
