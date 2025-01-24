// Journey here
// Tried to iteratively find naive probability by filtering
// Concurrent and normal iteration times are around 0.1 s calculation of belief is around 0.1 seconds
// This is too long
// Tried instead to save into hashmap and store in bson

use crate::history_public::{Card, AOName, ActionObservation};
use super::permutation_generator::{gen_table_combinations, gen_bag_combinations};
use super::coup_const::{BAG_SIZES, TOKENS, MAX_PERM_STATES};
// use core::hash::Hasher;
use std::usize;
use ahash::{AHashMap, AHashSet};
use rayon::prelude::*;
/// Struct that card count manually, by simulating movement of cards (as chars) for all possible permutations
pub struct BruteCardCountManager {
    certain_cards: Vec<Vec<Card>>,
    public_constraints: Vec<Vec<Card>>,
    inferred_constraints: Vec<Vec<Card>>,
    all_states: Vec<String>,
    calculated_states: AHashSet<String>, // All the states that fulfil current constraints
    index_start_arr: [usize; 7],
    index_end_arr: [usize; 7],
}
impl BruteCardCountManager {
    /// Constructor
    pub fn new() -> Self {
        let mut certain_cards: Vec<Vec<Card>> = vec![Vec::with_capacity(2); 6];
        certain_cards.push(Vec::with_capacity(3));
        let mut public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2); 6];
        public_constraints.push(Vec::with_capacity(3));
        let mut inferred_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2); 6];
        inferred_constraints.push(Vec::with_capacity(3));
        let all_states: Vec<String> = gen_table_combinations(TOKENS, &BAG_SIZES);
        let calculated_states: AHashSet<String> = all_states.clone().into_iter().collect();
        Self {
            certain_cards,
            public_constraints,
            inferred_constraints,
            all_states,
            calculated_states,
            index_start_arr: [0, 2, 4, 6, 8, 10, 12],
            index_end_arr: [2, 4, 6, 8, 10, 12, 15],
        }
    }
    /// Resets
    pub fn reset(&mut self) {
        self.certain_cards = vec![Vec::with_capacity(2); 6];
        self.certain_cards.push(Vec::with_capacity(3));
        self.public_constraints = vec![Vec::with_capacity(2); 6];
        self.public_constraints.push(Vec::with_capacity(3));
        self.inferred_constraints = vec![Vec::with_capacity(2); 6];
        self.inferred_constraints.push(Vec::with_capacity(3));
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
                            self.restrict(ao.player_id(), self.public_constraints[ao.player_id()].clone().iter().map(|c| c.card_to_char()).collect());
                        } else {
                            self.restrict(ao.player_id(), vec![ao.cards()[0].card_to_char()]);
                        }
                    }
                    2 => {
                        self.public_constraints[ao.player_id()].push(ao.cards()[0]);
                        self.public_constraints[ao.player_id()].push(ao.cards()[1]);
                        self.restrict(ao.player_id(), ao.cards().clone().into_iter().map(|c| c.card_to_char()).collect());
                    }
                    _ => {
                        debug_assert!(false, "woops");
                    }
                }
                self.update_constraints();
            },
            AOName::RevealRedraw => {
                self.reveal_redraw(ao.player_id(), ao.card().card_to_char());
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
    /// Returns bounds of indices for a particular player
    const fn player_slice_bounds(player_id: usize) -> (usize, usize) {
        match player_id {
            0 => (0, 2),
            1 => (2, 4),
            2 => (4, 6),
            3 => (6, 8),
            4 => (8, 10),
            5 => (10, 12),
            6 => (12, 15),
            _ => panic!("Invalid player_id"),
        }
    }
    /// Takes an &str as input and returns true if a player has a particular card
    pub fn player_has_cards(
        state: &str,
        player_id: usize,
        needed_cards: &[char], // Can be 1 or 2 cards, or even more.
        index_start_arr: &[usize; 7],
        index_end_arr: &[usize; 7],
    ) -> bool {
        // Get the player's substring from the main 15-char string
        let start = index_start_arr[player_id];
        let end = index_end_arr[player_id];
        let player_slice = &state[start..end];
    
        // Convert the player's slice into a vector so we can track removals
        let mut slice_chars: Vec<char> = player_slice.chars().collect();
    
        // For each needed card, try to remove one occurrence from slice_chars
        // If at any point a needed card isn't found, we return false
        for &needed in needed_cards {
            if let Some(pos) = slice_chars.iter().position(|&c| c == needed) {
                slice_chars.remove(pos);
            } else {
                return false;
            }
        }
    
        // If all needed cards were found (and removed), return true
        true
    }
    
    // A small helper to generate all combinations of `k` elements from a slice.
    pub fn combinations<T: Clone>(arr: &[T], k: usize) -> Vec<Vec<T>> {
        let mut result = Vec::new();
    
        // A helper function that takes indices and builds the corresponding combination
        fn push_combination<T: Clone>(indices: &[usize], arr: &[T], result: &mut Vec<Vec<T>>) {
            let combo = indices.iter().map(|&i| arr[i].clone()).collect();
            result.push(combo);
        }
    
        if k > arr.len() {
            return result; // no combinations possible if k > arr.len()
        }
    
        // We'll store the "current combination" indices in `indices`
        let mut indices: Vec<usize> = (0..k).collect(); // initial combination: [0, 1, ..., k-1]
    
        // Push the first combination
        push_combination(&indices, arr, &mut result);
    
        // Generate subsequent combinations in lexicographic order
        loop {
            // We'll try to increment from the rightmost position
            let mut i = k - 1;
    
            // Move `i` leftward until we find a position we can increment
            while indices[i] == i + arr.len() - k {
                if i == 0 {
                    // If we're at the leftmost position and can't increment, we're done
                    return result;
                }
                i -= 1;
            }
    
            // Increment the current position
            indices[i] += 1;
    
            // Then reset all subsequent positions
            for j in i + 1..k {
                indices[j] = indices[j - 1] + 1;
            }
    
            // Push the updated combination
            push_combination(&indices, arr, &mut result);
        }
    }
    
    /// RevealRedraw: Mixes a single known card of a player and another player
    /// Returns all the possible outcome combinations
    pub fn mix_one_char(
        &self,
        original_str: &str,
        player_reveal: usize,
        player_other: usize,
        card_i: char,
    ) -> Vec<String> {
    
        // Extract the slices from the original string
        let (start_reveal, end_reveal) = Self::player_slice_bounds(player_reveal);
        let (start_other, end_other) = Self::player_slice_bounds(player_other);
    
        // Convert slice into a vector of characters for easier manipulation
        let player_reveal_cards: Vec<char> = original_str[start_reveal..end_reveal].chars().collect();
        let player_other_cards: Vec<char> = original_str[start_other..end_other].chars().collect();
    
        // If player i doesn't have the card, return empty (or you could skip this check if guaranteed)
        if !player_reveal_cards.contains(&card_i) {
            return vec![];
        }
    
        let mut result = Vec::new();
    
        let mut revealed_player_cards = player_reveal_cards.clone();
        // Remove the card_i from i and card_j from j
        // (We remove only one occurrence if duplicates are present)
        if let Some(pos_i) = revealed_player_cards.iter().position(|&c| c == card_i) {
            revealed_player_cards.swap_remove(pos_i);
        }
        // Case when same card received back
        result.push(original_str.to_string());

        // For each card in player_j, swap with card_i
        for (pos_other, &card_j) in player_other_cards.iter().enumerate() {
            // Work on local copies
            let mut new_revealed_player_cards = revealed_player_cards.clone();
            let mut new_other_player_cards = player_other_cards.clone();
    
            new_other_player_cards.swap_remove(pos_other);
    
            // Swap them
            new_revealed_player_cards.push(card_j);
            new_other_player_cards.push(card_i);
    
            // Sort each player's cards to maintain ascending order in the final string
            new_revealed_player_cards.sort_unstable();
            new_other_player_cards.sort_unstable();
    
            // Rebuild the final string
            let mut new_string = String::new();
            // For each of the 7 players, we either insert the new_i_cards, new_j_cards, or original
            for player_id in 0..7 {
                let (start, end) = Self::player_slice_bounds(player_id);
                if player_id == player_reveal {
                    new_string.push_str(&new_revealed_player_cards.iter().collect::<String>());
                } else if player_id == player_other {
                    new_string.push_str(&new_other_player_cards.iter().collect::<String>());
                } else {
                    // remain as in the original string
                    new_string.push_str(&original_str[start..end]);
                }
            }
    
            // Collect
            result.push(new_string);
        }
    
        result
    }
    /// Ambassador: Mixes all the player's cards with player 6 (pile)
    /// Returns all possible outcome combinations
    pub fn mix_multiple_chars_with_player6(
        &self,
        original_str: &str,
        player_id_mix: usize,
    ) -> Vec<String> {
       
        // We'll fix the "other" player to always be player_id = 6
        // (If player_id_i == 6, then you'd be mixing with yourself, which might not make sense,
        //  but we'll keep it consistent with the request.)
        let player_id_j = 6;
    
        let (start_i, end_i) = Self::player_slice_bounds(player_id_mix);
        let (start_j, end_j) = Self::player_slice_bounds(player_id_j);
    
        // Extract both players' cards from the original string
        let i_cards: Vec<char> = original_str[start_i..end_i].chars().collect();
        let player_j_cards: Vec<char> = original_str[start_j..end_j].chars().collect();
    
        // Combine all cards (player i's plus player 6's)
        let mut combined_cards = i_cards.clone();
        combined_cards.extend_from_slice(&player_j_cards);
    
        // The number of cards in i_cards and j_cards
        let i_count = i_cards.len();        // typically 2 if i < 6, or 3 if i == 6
        let j_count = player_j_cards.len(); // should be 3 for player 6
    
        // Generate all possible ways to re-distribute these total cards
        // back into i_count for Player i and j_count for Player 6
        // i.e., we choose any subset of `i_count` from the combined list for Player i
        // and the remaining `j_count` go to Player 6.
        let mut results = Vec::new();
        for combo_for_i in Self::combinations(&combined_cards, i_count) {
            // Sort the chosen cards for i
            let mut new_i_cards = combo_for_i.clone();
            new_i_cards.sort_unstable();
    
            // The leftover cards go to j
            let mut new_j_cards = combined_cards.clone();
            // Remove exactly one occurrence of each chosen card from new_j_cards
            for c in &combo_for_i {
                if let Some(pos) = new_j_cards.iter().position(|&x| x == *c) {
                    new_j_cards.swap_remove(pos);
                }
            }
            new_j_cards.sort_unstable();
    
            // Rebuild the final string
            let mut new_string = String::with_capacity(15);
            for pid in 0..7 {
                let (st, en) = Self::player_slice_bounds(pid);
                if pid == player_id_mix {
                    new_string.push_str(&new_i_cards.iter().collect::<String>());
                } else if pid == player_id_j {
                    new_string.push_str(&new_j_cards.iter().collect::<String>());
                } else {
                    // Remain unchanged
                    new_string.push_str(&original_str[st..en]);
                }
            }
    
            results.push(new_string);
        }
    
        results
    }
    
    /// Use Rayon to parallelize the process of running `mix_one_char` on
    /// each state in `self.calculated_states`, collecting all results
    /// into a new AHashSet.
    pub fn reveal_redraw(
        &mut self,
        player_reveal: usize,
        card_i: char,
    ) {
        // Step 1: In parallel, call `mix_one_char` for each state, flatten the results
        // into a single Vec<String>.
        self.restrict(player_reveal, vec![card_i]);
        let new_states_vec: Vec<String> = self
            .calculated_states
            .par_iter()  // parallel iteration over our existing states
            .flat_map(|state| self.mix_one_char(state, player_reveal, 6, card_i))
            .collect();

        // Step 2: Convert that Vec<String> into a new AHashSet to remove duplicates.
        let new_states: AHashSet<String> = new_states_vec.into_iter().collect();

        // Finally, assign this new set back to `self.calculated_states`.
        self.calculated_states = new_states;
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
        let new_states_vec: Vec<String> = self
            .calculated_states
            .par_iter()  // parallel iteration over our existing states
            .flat_map(|state| self.mix_multiple_chars_with_player6(state, player_reveal))
            .collect();

        // Step 2: Convert that Vec<String> into a new AHashSet to remove duplicates.
        let new_states: AHashSet<String> = new_states_vec.into_iter().collect();

        // Finally, assign this new set back to `self.calculated_states`.
        self.calculated_states = new_states;
    }
    /// This function filters out `self.calculated_states` such that only
    /// states where `player_reveal` possesses *all* cards in `card_chars` remain.
    pub fn restrict(&mut self, player_reveal: usize, card_chars: Vec<char>) {
        log::info!("Brute Prob: Restrict Ran: player: {}, cards: {:?}", player_reveal, card_chars);
        self.calculated_states.retain(|state| {
            Self::player_has_cards(
                state,
                player_reveal,
                &card_chars,
                &self.index_start_arr,
                &self.index_end_arr,
            )
        });
        // log::info!("legal states after Restrict: {:?}", self.calculated_states);
    }
    /// This function returns true if a player can have a particular card
    pub fn player_can_have_card(&self, player_id: usize, card: Card) -> bool {
        let states: Vec<&String> = self.calculated_states.iter().collect();

        let start = self.index_start_arr[player_id];
        let end = self.index_end_arr[player_id];

        let card_char = card.card_to_char();
        self.calculated_states.par_iter().
        any(|state| state[self.index_start_arr[player_id]..self.index_end_arr[player_id]].contains(card_char))
    }
    /// This function returns true if a player can have all of these cards
    pub fn player_can_have_cards(&self, player_id: usize, cards: &[Card]) -> bool {
        let states: Vec<&String> = self.calculated_states.iter().collect();

        let start = self.index_start_arr[player_id];
        let end = self.index_end_arr[player_id];

        // Check in parallel if any state satisfies the requirement
        states.par_iter().any(|state| {
            let player_slice = &state[start..end];
            let mut chars_in_hand: Vec<char> = player_slice.chars().collect();

            cards.iter().all(|card| {
                let needed_char = card.card_to_char();
                if let Some(pos) = chars_in_hand.iter().position(|&c| c == needed_char) {
                    chars_in_hand.remove(pos);
                    true
                } else {
                    false
                }
            })
        })
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
        let mut result: Vec<Vec<Card>> = Vec::with_capacity(7);

        // If there are no states at all, every player's "must have" set is empty
        if self.calculated_states.is_empty() {
            result.resize_with(7, Vec::new);
            return result;
        }

        // Helper to build a frequency map of the substring belonging to one player in a given state
        fn freq_of_slice(state: &str, start: usize, end: usize) -> AHashMap<char, usize> {
            let mut map = AHashMap::new();
            for c in state[start..end].chars() {
                *map.entry(c).or_insert(0) += 1;
            }
            map
        }

        // For each of the 7 players, compute the "intersection frequency map"
        // across all `calculated_states`.
        for player_id in 0..7 {
            let start = self.index_start_arr[player_id];
            let end = self.index_end_arr[player_id];

            // Start by taking the frequency map from the first state
            let mut iter = self.calculated_states.iter();
            let first_state = iter.next().unwrap();
            let mut common_freq = freq_of_slice(first_state, start, end);

            // Intersect with the frequency maps of all subsequent states
            for state in iter {
                let freq = freq_of_slice(state, start, end);

                // For each character currently in common_freq, lower it to the
                // min frequency if this new state has fewer of that character.
                for (&ch, &common_count) in common_freq.clone().iter() {
                    let freq_count = freq.get(&ch).copied().unwrap_or(0);
                    let new_count = common_count.min(freq_count);
                    if new_count == 0 {
                        // If this state has none, then the intersection can't have it
                        common_freq.remove(&ch);
                    } else {
                        // Update the count in common_freq to the new minimum
                        *common_freq.get_mut(&ch).unwrap() = new_count;
                    }
                }
            }

            // Now `common_freq` holds the minimum number of each card that appears
            // in **every** state for this player. Convert that map to a Vec<char>.
            let mut must_have_for_player = Vec::new();
            for (ch, count) in common_freq {
                // Repeat each character `count` times
                for _ in 0..count {
                    must_have_for_player.push(Card::char_to_card(ch));
                }
            }
            // Sort for a consistent order (e.g., "AAB" not "BAA")
            must_have_for_player.sort_unstable();

            result.push(must_have_for_player);
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
    /// In other words, for all states, the substring of `player_id` does not
    /// contain the corresponding card character. Hence, that player **cannot** have that card.
    pub fn validated_impossible_constraints(&self) -> [[bool; 5]; 7] {
        let mut result = [[false; 5]; 7];

        // Early return if we have no states; then every card is impossible in all states
        // or every card is possible—depending on your game logic. Usually, with zero states,
        // "cannot have" is trivially true for all. But check game logic as needed.
        if self.calculated_states.is_empty() {
            return [[true; 5]; 7];
        }

        // For each player
        for player_id in 0..7 {
            let start = self.index_start_arr[player_id];
            let end = self.index_end_arr[player_id];

            // For each card variant (assuming your Card enum maps 1:1 to these indices)
            // e.g., 0 = Duke, 1 = Assassin, 2 = Captain, 3 = Ambassador, 4 = Contessa
            for card_idx in 0..5 as usize {
                // Convert card_idx -> Card -> char
                let card_enum = Card::try_from(card_idx as u8).unwrap();
                let card_char = card_enum.card_to_char();

                // We want to know if there's ANY state in which the player's substring
                // includes `card_char`. If there is, then `cannot_have` is false.
                // If we can't find it in ANY state, `cannot_have` is true.
                let found_in_any_state = self.calculated_states.par_iter().any(|state| {
                    let player_slice = &state[start..end];
                    player_slice.contains(card_char)
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
        self.certain_cards = self.must_have_cards();
        self.inferred_constraints = self.certain_cards.clone();
        for (player, cards) in self.public_constraints.iter().enumerate() {
            for card in cards.iter() {
                if let Some(pos) = self.inferred_constraints[player].iter().position(|c| *c == *card ){
                    self.inferred_constraints[player].swap_remove(pos);
                }
            }
        }
        for vec in self.certain_cards.iter_mut() {
            vec.sort_unstable();
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
        log::info!("Brute certain cards: {:?}", self.certain_cards);
        log::info!("Brute public constraints: {:?}", self.validated_public_constraints());
        log::info!("Brute inferred constraints: {:?}", self.validated_inferred_constraints());
        log::info!("Brute impossible cards: {:?}", self.validated_impossible_constraints());
    }
    /// Checks if calculated_states fulfils all self.public_constraints
    pub fn validate(&self) -> bool {
        let mut output = true;
        for (player_id, card_vec) in self.public_constraints.iter().enumerate() {
            let card_chars = card_vec.iter().map(|c| c.card_to_char()).collect();
            output = self.calculated_states.iter().all(|state| Self::validate_str(state, player_id, &card_chars)) && output;
            if !output {
                return output
            }
        }
        output
    }
    /// Assumes card_chars is sorted, for the love of God sort it first
    fn validate_str(str: &str, player_id: usize, card_chars: &Vec<char>) -> bool {
        let (index_start, index_end) = Self::player_slice_bounds(player_id);
        if card_chars.len() == 2 {
            let card_str: String = card_chars.iter().collect();
            return str[index_start..index_end] == card_str;
        } else if card_chars.len() == 1 {
            return str[index_start..index_end].chars().nth(0) == Some(card_chars[0]) || str[index_start..index_end].chars().nth(1) == Some(card_chars[0])
        }
        true
    }
}
