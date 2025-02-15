// Journey here
// Tried to iteratively find naive probability by filtering
// Concurrent and normal iteration times are around 0.1 s calculation of belief is around 0.1 seconds
// This is too long
// Tried instead to save into hashmap and store in bson

use std::usize;
use std::hash::Hash;
use crate::history_public::{Card, AOName, ActionObservation};
use super::permutation_generator::{gen_table_combinations};
use super::coup_const::{BAG_SIZES, TOKENS};
use super::card_state::CardPermState;
// use core::hash::Hasher;
use ahash::{AHashMap, AHashSet};
use rayon::prelude::*;
/// Struct that card count manually, by simulating movement of cards (as chars) for all possible permutations

pub struct BruteCardCountManagerGeneric<T: CardPermState> 
where
    T: CardPermState + Hash + Eq + Clone + std::fmt::Display + std::fmt::Debug,
{
    all_states: Vec<T>,
    calculated_states: AHashSet<T>, // All the states that fulfil current constraints
    public_constraints: Vec<Vec<Card>>,
    inferred_constraints: Vec<Vec<Card>>,
    impossible_constraints: [[bool; 5]; 7],
}
impl<T> BruteCardCountManagerGeneric<T> 
where
    T: CardPermState + Hash + Eq + Clone + std::fmt::Display + std::fmt::Debug,
{
    /// Constructor
    pub fn new() -> Self {
        let all_states: Vec<T> = T::gen_table_combinations();
        let calculated_states: AHashSet<T> = all_states.clone().into_iter().collect();
        let mut public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2); 6];
        public_constraints.push(Vec::with_capacity(3));
        let mut inferred_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2); 6];
        inferred_constraints.push(Vec::with_capacity(3));
        let impossible_constraints = [[false; 5]; 7];
        Self {
            all_states,
            calculated_states,
            public_constraints,
            inferred_constraints,
            impossible_constraints,
        }
    }
    /// Resets
    pub fn reset(&mut self) {
        self.public_constraints = vec![Vec::with_capacity(2); 6];
        self.public_constraints.push(Vec::with_capacity(3));
        self.inferred_constraints = vec![Vec::with_capacity(2); 6];
        self.inferred_constraints.push(Vec::with_capacity(3));
        self.impossible_constraints = [[false; 5]; 7];
        self.calculated_states = self.all_states.clone().into_iter().collect();
    }
    /// adds public constraint
    pub fn add_public_constraint(&mut self, player_id: usize, card: Card) {
        self.public_constraints[player_id].push(card);
    }
    /// Modifies internal state based on latest action done by player
    pub fn push_ao(&mut self, ao: &ActionObservation, bool_know_priv_info: bool) {
        debug_assert!(!bool_know_priv_info, "Not yet supported!");
        match ao.name() {
            AOName::Discard => {
                match ao.no_cards() {
                    1 => {
                        self.public_constraints[ao.player_id()].push(ao.cards()[0]);
                        if self.public_constraints[ao.player_id()].len() == 2 {
                            // Handles the case where the players' dead cards are both the same, without this, restrict won't ensure player has both cards
                            self.restrict(ao.player_id(), &self.public_constraints[ao.player_id()].clone());
                        } else {
                            self.restrict(ao.player_id(), &[ao.cards()[0]]);
                        }
                    }
                    2 => {
                        self.public_constraints[ao.player_id()].push(ao.cards()[0]);
                        self.public_constraints[ao.player_id()].push(ao.cards()[1]);
                        self.restrict(ao.player_id(), ao.cards());
                    }
                    _ => {
                        debug_assert!(false, "woops");
                    }
                }
                self.update_constraints();
            },
            AOName::RevealRedraw => {
                self.reveal_redraw(ao.player_id(), ao.card());
                self.update_constraints();
            },
            AOName::ExchangeDraw => {
                // TODO: public private split
                self.ambassador(ao.player_id());
                self.update_constraints();
            },
            AOName::ExchangeChoice => {
            },
            _ => {

            },
        };
    }
    /// Unsupported, as information is loss after pushing and cannot be reverted
    pub fn pop(&mut self) {
        panic!("No pops! This goes in one direction only!")
    }
    /// Use Rayon to parallelize the process of running `mix_one_char` on
    /// each state in `self.calculated_states`, collecting all results
    /// into a new AHashSet.
    pub fn reveal_redraw(
        &mut self,
        player_reveal: usize,
        card_i: Card,
    ) {
        // Step 1: In parallel, call `mix_one_char` for each state, flatten the results
        // into a single Vec<String>.
        let mut current_dead_cards: Vec<Card> = self.public_constraints[player_reveal].clone();
        current_dead_cards.push(card_i);
        self.restrict(player_reveal, &current_dead_cards);
        let new_states_vec: Vec<T> = self
        .calculated_states
        .iter()  // parallel iteration over our existing states
        .flat_map(|state| state.mix_one_card(player_reveal, 6, card_i))
        .collect();
    
        // Step 2: Convert that Vec<String> into a new AHashSet to remove duplicates.
        let new_states: AHashSet<T> = new_states_vec.into_iter().collect();
        
        // Finally, assign this new set back to `self.calculated_states`.
        self.calculated_states = new_states;
        log::info!("Brute Prob Redraw:");
        self.print_legal_states();
    }
    /// Use Rayon to parallelize the process of running `mix_one_char` on
    /// each state in `self.calculated_states`, collecting all results
    /// into a new AHashSet.
    pub fn ambassador(
        &mut self,
        player_reveal: usize,
    ) {
        // Step 1: In parallel, call `mix_one_char` for each state, flatten the results
        // into a single Vec<String>.
        let new_states_vec: Vec<T> = self
            .calculated_states
            .iter()  // parallel iteration over our existing states
            .flat_map(|state| state.mix_multiple_chars_with_player6(player_reveal, &self.public_constraints[player_reveal]))
            .collect();

        // Step 2: Convert that Vec<String> into a new AHashSet to remove duplicates.
        let new_states: AHashSet<T> = new_states_vec.into_iter().collect();

        // Finally, assign this new set back to `self.calculated_states`.
        self.calculated_states = new_states;
    }
    /// This function filters out `self.calculated_states` such that only
    /// states where `player_reveal` possesses *all* cards in `card_chars` remain.
    pub fn restrict(&mut self, player_reveal: usize, cards: &[Card]) {
        log::info!("Brute Prob: Restrict Ran: player: {}, cards: {:?}", player_reveal, cards);
        log::info!("Before Restrict");
        self.print_legal_states();
        self.calculated_states.retain(|state| {
            state.player_has_cards(
                player_reveal,
                cards,
            )
        });
        log::info!("After Restrict");
        self.print_legal_states();
        // log::info!("legal states after Restrict: {:?}", self.calculated_states);
    }
    /// This function returns true if a player can have a particular card
    pub fn player_can_have_card(&self, player_id: usize, card: Card) -> bool {
        self.calculated_states.iter().
        any(|state| state.player_has_cards(player_id, &[card]))
    }
    /// This function returns true if a player can have a particular card
    /// Does care about alive or dead status
    pub fn player_can_have_card_alive(&self, player_id: usize, card: Card) -> bool {
        // if self.public_constraints[player_id].len() == 1 {
        //     let mut card_vec: Vec<Card> = self.public_constraints[player_id].clone();
        //     card_vec.push(card);
        //     self.player_can_have_cards(player_id, &card_vec[0..2])
        // } else {
        //     self.calculated_states.iter().
        //     any(|state| state.player_has_cards(player_id, &[card]))
        // }
        !self.impossible_constraints[player_id][card as usize]
    }
    /// This function returns true if a player can have all of these cards
    /// Does not care about alive or dead status
    pub fn player_can_have_cards(&self, player_id: usize, cards: &[Card]) -> bool {
        // Check in paralle if any state satisfies the requirement
        let mut deduplicated = cards.to_vec();
        deduplicated.sort_unstable();
        deduplicated.dedup();
        for card in deduplicated {
            if !self.player_can_have_card_alive(player_id, card) {
                return false;
            }
            if self.public_constraints.iter().map(|v| v.iter().filter(|c| **c == cards[0]).count() as u8).sum::<u8>() +
            self.inferred_constraints.iter().map(|v| v.iter().filter(|c| **c == cards[0]).count() as u8).sum::<u8>() +
            cards.iter().filter(|c| **c == card).count() as u8
            > 3 {
                return false;
            }
        }
        self.calculated_states.iter().any(|state| state.player_has_cards(player_id, cards))
    }
    /// For each player (0..6), determine which cards they **must** have in *every* possible state.
    /// Returns a `Vec<Vec<char>>` of length 7, where `result[player_id]` is a sorted list
    /// (with multiplicities) of all cards that player *always* holds in all current `calculated_states`.
    ///
    /// For example, if player 0 *always* has at least one 'A' and one 'B' across all states,
    /// then `result[0]` will include 'A' and 'B'. If they sometimes have 'C' and sometimes not,
    /// 'C' won't appear in `result[0]`. If they always have two 'A's (i.e., every state has "AA"),
    /// then `result[0]` will contain `['A','A']`.
    pub fn must_have_cards(&self) -> Vec<Vec<Card>> {
        
        let mut result: Vec<Vec<Card>> = vec![Vec::with_capacity(2); 6];
        result.push(Vec::with_capacity(3));
        // If there are no states at all, every player's "must have" set is empty
        if self.calculated_states.is_empty() {
            return result;
        }

        // For each of the 7 players, compute the "intersection frequency map"
        // across all `calculated_states`.
        for player_id in 0..7 {

            // Start by taking the frequency map from the first state
            let mut iter = self.calculated_states.iter();
            let first_state = iter.next().unwrap();
            let mut common_freq = first_state.player_card_counts(player_id);

            // Intersect with the frequency maps of all subsequent states
            for state in iter {
                let freq = state.player_card_counts(player_id);

                // For each character currently in common_freq, lower it to the
                // min frequency if this new state has fewer of that character.
                for (card_id, common_count) in common_freq.clone().iter().enumerate() {
                    let freq_count = freq[card_id];
                    let new_count = *common_count.min(&freq_count);
                    common_freq[card_id] = new_count;
                }
                if common_freq.iter().all(|count| *count == 0) {
                    break
                }
            }

            // Now `common_freq` holds the minimum number of each card that appears
            // in **every** state for this player. Convert that map to a Vec<char>.
            let mut must_have_for_player = Vec::with_capacity(3);
            for (card_id, count) in common_freq.iter().enumerate() {
                // Repeat each character `count` times
                for _ in 0..*count {
                    must_have_for_player.push(Card::try_from(card_id as u8).unwrap());
                }
            }

            result[player_id] = must_have_for_player;
        }

        result
    }
    /// Returns a 7x5 boolean array `[ [bool; 5]; 7 ]`.
    ///
    /// - Outer index = player (0..6)
    /// - Inner index = card as usize (0..4 or 0..5, depending on how you define Card).
    ///
    /// `result[player_id][card_index]` will be `true` if, **in every** state within
    /// `self.calculated_states`, that `player_id` does **not** have that card.
    ///
    /// Returns an array that is true if a player does cannot have that card alive
    pub fn set_impossible_constraints(&mut self) {
        let mut result = [[false; 5]; 7];

        // Early return if we have no states; then every card is impossible in all states
        // or every card is possible—depending on your game logic. Usually, with zero states,
        // "cannot have" is trivially true for all. But check game logic as needed.
        if self.calculated_states.is_empty() {
            self.impossible_constraints = [[true; 5]; 7];
        }

        // For each player
        for player_id in 0..7 {
            // For each card variant (assuming your Card enum maps 1:1 to these indices)
            // e.g., 0 = Duke, 1 = Assassin, 2 = Captain, 3 = Ambassador, 4 = Contessa
            for card_idx in 0..5 as usize {
                // Convert card_idx -> Card -> char
                let card_enum = Card::try_from(card_idx as u8).unwrap();

                // We want to know if there's ANY state in which the player's substring
                // includes `card_char`. If there is, then `cannot_have` is false.
                // If we can't find it in ANY state, `cannot_have` is true.
                let found_in_any_state = self.calculated_states.iter().any(|state| {
                    let actual_count = state.player_card_count(player_id, card_enum);
                    let reference_count = self.public_constraints[player_id].iter().filter(|c| **c == card_enum).count() as u8;
                    actual_count > reference_count
                });

                // If found_in_any_state == false, that means:
                // "There is NO state in which the player has this card alive"
                // So the player "cannot have" it => result = true
                result[player_id][card_idx] = !found_in_any_state;
            }
        }

        self.impossible_constraints = result;
    }
    /// Returns a 7x5 boolean array `[ [bool; 5]; 7 ]`.
    ///
    /// - Outer index = player (0..6)
    /// - Inner index = card as usize (0..4 or 0..5, depending on how you define Card).
    ///
    /// `result[player_id][card_index]` will be `true` if, **in every** state within
    /// `self.calculated_states`, that `player_id` does **not** have that card.
    ///
    /// Returns an array that is true if a player does cannot have that card alive
    pub fn validated_impossible_constraints(&self) -> [[bool; 5]; 7] {
        self.impossible_constraints.clone()
    }
    /// Returns a 7x5 boolean array `[ [bool; 5]; 7 ]`.
    ///
    /// - Outer index = player (0..6)
    /// - Inner index = card as usize (0..4 or 0..5, depending on how you define Card).
    ///
    /// `result[player_id][card_index]` will be `true` if, **in every** state within
    /// `self.calculated_states`, that `player_id` does **not** have that card.
    ///
    /// Returns an array that is true if a player does cannot have that card alive or dead
    pub fn validated_impossible_constraints_include_dead(&self) -> [[bool; 5]; 7] {
        let mut result = [[false; 5]; 7];

        // Early return if we have no states; then every card is impossible in all states
        // or every card is possible—depending on your game logic. Usually, with zero states,
        // "cannot have" is trivially true for all. But check game logic as needed.
        if self.calculated_states.is_empty() {
            return [[true; 5]; 7];
        }

        // For each player
        for player_id in 0..7 {

            // For each card variant (assuming your Card enum maps 1:1 to these indices)
            // e.g., 0 = Duke, 1 = Assassin, 2 = Captain, 3 = Ambassador, 4 = Contessa
            for card_idx in 0..5 as usize {
                // Convert card_idx -> Card -> char
                let card_enum = Card::try_from(card_idx as u8).unwrap();

                // We want to know if there's ANY state in which the player's substring
                // includes `card_char`. If there is, then `cannot_have` is false.
                // If we can't find it in ANY state, `cannot_have` is true.
                let found_in_any_state = self.calculated_states.iter().any(|state| {
                    state.player_has_cards(player_id, &[card_enum])
                });

                // If found_in_any_state == false, that means:
                // "There is NO state in which the player has this card."
                // So the player "cannot have" it => result = true
                result[player_id][card_idx] = !found_in_any_state;
            }
        }

        result
    }
    /// Returns all the dead cards for each player that we are certain they have
    /// Assumes calculates states align with latest constraints
    pub fn update_constraints(&mut self) {
        self.inferred_constraints = self.must_have_cards();
        self.set_impossible_constraints();
        for (player, cards) in self.public_constraints.iter().enumerate() {
            for card in cards.iter() {
                if let Some(pos) = self.inferred_constraints[player].iter().position(|c| *c == *card ){
                    self.inferred_constraints[player].swap_remove(pos);
                }
            }
        }
        for vec in self.public_constraints.iter_mut() {
            vec.sort_unstable();
        }
        for vec in self.inferred_constraints.iter_mut() {
            vec.sort_unstable();
        }

    }
    /// Returns all the dead cards for each player that we are certain they have
    /// Assumes calculates states align with latest constraints
    pub fn validated_public_constraints(&self) -> Vec<Vec<Card>> {
        self.public_constraints.clone()
    }
    /// Returns all the dead cards for each player that we are certain they have
    /// Assumes calculates states align with latest constraints
    pub fn validated_inferred_constraints(&self) -> Vec<Vec<Card>> {
        self.inferred_constraints.clone()
    }
    /// Print Calculated States => All current possible legal states
    pub fn print_legal_states(&self) {
        log::info!("Brute Prob legal_states.len: {:?}", self.calculated_states);
    }
    /// Prints useful shit
    pub fn printlog(&self) {
        log::info!("calculated_states.len: {}", self.calculated_states.len());
        log::info!("Brute public constraints: {:?}", self.validated_public_constraints());
        log::info!("Brute inferred constraints: {:?}", self.validated_inferred_constraints());
        log::info!("Brute impossible cards: {:?}", self.validated_impossible_constraints());
    }
    /// Checks if calculated_states fulfils all self.public_constraints
    pub fn validate(&self) -> bool {
        let mut output = true;
        for (player_id, card_vec) in self.public_constraints.iter().enumerate() {
            output = self.calculated_states.iter().all(|state| state.player_has_cards(player_id, card_vec)) && output;
            if !output {
                return output
            }
        }
        output
    }
}
