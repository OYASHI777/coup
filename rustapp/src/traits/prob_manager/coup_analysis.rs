use crate::history_public::{ActionObservation, Card};
use crate::prob_manager::engine::constants::MAX_HAND_SIZE_PLAYER;
use crate::prob_manager::engine::models::game_state::GameData;
use crate::prob_manager::engine::models_prelude::*;

pub trait CoupTraversal {
    // TODO: Consider taking starting_player: Option<usize>
    /// Starts a game with public information
    fn start_public(&mut self, player: usize);
    /// Starts a game with private information
    fn start_private(&mut self, player: usize, cards: &[Card; MAX_HAND_SIZE_PLAYER]);
    /// Starts a game with all cards known
    fn start_known(&mut self, cards: &Vec<Vec<Card>>);
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
}
/// Visitor Pattern
/// Supplies *one method per `EngineState` variant*.
/// Each method receives the full `GameState` plus the variant-specific payload and
/// returns the list of legal `ActionObservation`s for that state.
pub trait CoupGeneration {
    //  Turn control
    fn on_turn_start(&self, state: &TurnStart, data: &GameData) -> Vec<ActionObservation>;
    fn on_end(&self, state: &End, data: &GameData) -> Vec<ActionObservation>;

    //  Coup
    fn on_coup_hit(&self, state: &CoupHit, data: &GameData) -> Vec<ActionObservation>;

    //  Foreign Aid
    fn on_foreign_aid_invites_block(
        &self,
        state: &ForeignAidInvitesBlock,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_foreign_aid_block_invites_challenge(
        &self,
        state: &ForeignAidBlockInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_foreign_aid_block_challenged(
        &self,
        state: &ForeignAidBlockChallenged,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_foreign_aid_block_challenger_failed(
        &self,
        state: &ForeignAidBlockChallengerFailed,
        data: &GameData,
    ) -> Vec<ActionObservation>;

    //  Tax
    fn on_tax_invites_challenge(
        &self,
        state: &TaxInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_tax_challenged(&self, state: &TaxChallenged, data: &GameData) -> Vec<ActionObservation>;
    fn on_tax_challenger_failed(
        &self,
        state: &TaxChallengerFailed,
        data: &GameData,
    ) -> Vec<ActionObservation>;

    //  Steal
    fn on_steal_invites_challenge(
        &self,
        state: &StealInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_steal_challenged(
        &self,
        state: &StealChallenged,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_steal_challenger_failed(
        &self,
        state: &StealChallengerFailed,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_steal_invites_block(
        &self,
        state: &StealInvitesBlock,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_steal_block_invites_challenge(
        &self,
        state: &StealBlockInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_steal_block_challenged(
        &self,
        state: &StealBlockChallenged,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_steal_block_challenger_failed(
        &self,
        state: &StealBlockChallengerFailed,
        data: &GameData,
    ) -> Vec<ActionObservation>;

    //  Exchange
    fn on_exchange_invites_challenge(
        &self,
        state: &ExchangeInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_exchange_drawing(
        &self,
        state: &ExchangeDrawing,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_exchange_drawn(&self, state: &ExchangeDrawn, data: &GameData) -> Vec<ActionObservation>;
    fn on_exchange_challenged(
        &self,
        state: &ExchangeChallenged,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_exchange_challenger_failed(
        &self,
        state: &ExchangeChallengerFailed,
        data: &GameData,
    ) -> Vec<ActionObservation>;

    //  Assassinate
    fn on_assassinate_invites_challenge(
        &self,
        state: &AssassinateInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_assassinate_invites_block(
        &self,
        state: &AssassinateInvitesBlock,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_assassinate_block_invites_challenge(
        &self,
        state: &AssassinateBlockInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_assassinate_block_challenged(
        &self,
        state: &AssassinateBlockChallenged,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_assassinate_block_challenger_failed(
        &self,
        state: &AssassinateBlockChallengerFailed,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_assassinate_succeeded(
        &self,
        state: &AssassinateSucceeded,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_assassinate_challenged(
        &self,
        state: &AssassinateChallenged,
        data: &GameData,
    ) -> Vec<ActionObservation>;
    fn on_assassinate_challenger_failed(
        &self,
        state: &AssassinateChallengerFailed,
        data: &GameData,
    ) -> Vec<ActionObservation>;
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
    /// Returns array[player][card] storing whether a player can have a card alive
    fn player_impossible_constraints(&mut self) -> [[bool; 5]; 7];
    /// Returns array[player][card_i][card_j] storing whether a player can have a card_i and card_j alive
    fn player_impossible_constraints_paired(&mut self) -> [[[bool; 5]; 5]; 7];
    /// Returns array[card_i][card_j][card_k] storing whether pile can have card_i, card_j, and card_k
    fn player_impossible_constraints_triple(&mut self) -> [[[bool; 5]; 5]; 5];
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
