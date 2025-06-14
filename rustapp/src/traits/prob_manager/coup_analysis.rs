use crate::history_public::{ActionObservation, Card};

pub trait CoupTraversal {
    // TODO: Consider taking starting_player: Option<usize>
    /// Starts a game with public information
    fn start_public(&mut self);
    /// Starts a game with private information
    fn start_private(&mut self, player: usize, cards: &[Card; 2]);
    /// Adds an Action with only public information
    fn push_ao_public(&mut self, action: &ActionObservation);
    /// Adds an Action with only public information using lazy calculation methodology
    fn push_ao_public_lazy(&mut self, action: &ActionObservation);
    /// Adds an Action with public and private information
    fn push_ao_private(&mut self, action: &ActionObservation);
    /// Adds an Action with public and private information using lazy calculation methodology
    fn push_ao_private_lazy(&mut self, action: &ActionObservation);
    /// Pops latest move
    fn pop(&mut self);
    /// Resets game state. Use start_public or start_private to start a new game
    fn reset(&mut self);
}

pub trait CoupPossibilityAnalysis {
    /// Returns reference to latest Public Constraints
    fn public_constraints(&mut self) -> &Vec<Vec<Card>>;
    /// Returns reference to latest sorted Public Constraints
    fn sorted_public_constraints(&mut self) -> &Vec<Vec<Card>>;
    /// Returns reference to latest Inferred Constraints
    fn inferred_constraints(&mut self) -> &Vec<Vec<Card>>;
    /// Returns reference to latest sorted Inferred Constraints
    fn sorted_inferred_constraints(&mut self) -> &Vec<Vec<Card>>;
    /// Returns reference to array[player][card] storing whether a player can have a card alive
    fn player_impossible_constraints(&mut self) -> &[[bool; 5]; 7];
    /// Returns reference to array[player][card_i][card_j] storing whether a player can have a card_i and card_j alive
    fn player_impossible_constraints_paired(&mut self) -> &[[[bool; 5]; 5]; 7];
    /// Returns reference to array[card_i][card_j][card_k] storing whether pile can have card_i, card_j, and card_k
    fn player_impossible_constraints_triple(&mut self) -> &[[[bool; 5]; 5]; 5];
    /// Returns true if player can have a particular card alive
    fn player_can_have_card_alive(&mut self, player: usize, card: Card) -> bool;
    /// Returns true if player can have a particular card alive | evaluates lazily
    fn player_can_have_card_alive_lazy(&mut self, player: usize, card: Card) -> bool;
    /// Returns true if player can have a collection of cards alive
    fn player_can_have_cards_alive(&mut self, player: usize, cards: &[Card]) -> bool;
    /// Returns true if player can have a collection of cards alive | evaluates lazily
    fn player_can_have_cards_alive_lazy(&mut self, player: usize, cards: &[Card]) -> bool;
    /// Returns true if move is legal considering only public information
    /// Assumes the player can make a turn and does not check if it is the player's turn
    fn is_legal_move_public(&mut self, action_observation: &ActionObservation) -> bool;
    /// Returns true if move is legal considering public and private information
    /// Assumes the player can make a turn and does not check if it is the player's turn
    fn is_legal_move_private(&mut self, action_observation: &ActionObservation) -> bool;
} 