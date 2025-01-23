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
use ahash::AHashSet;
use rayon::prelude::*;
pub struct BruteCardCountManager {
    all_states: Vec<String>,
    calculated_states: AHashSet<String>, // All the states that fulfil current constraints
    index_start_arr: [usize; 7],
    index_end_arr: [usize; 7],
}
impl BruteCardCountManager {
    pub fn new() -> Self {
        let all_states: Vec<String> = gen_table_combinations(TOKENS, &BAG_SIZES);
        let calculated_states: AHashSet<String> = all_states.clone().into_iter().collect();
        Self {
            all_states,
            calculated_states,
            index_start_arr: [0, 2, 4, 6, 8, 10, 12],
            index_end_arr: [2, 4, 6, 8, 10, 12, 15],
        }
    }
    pub fn reset(&mut self) {
        self.calculated_states = self.all_states.clone().into_iter().collect();
    }
    pub fn push_ao(&mut self, ao: &ActionObservation, bool_know_priv_info: bool) {
        debug_assert!(!bool_know_priv_info, "Not yet supported!");
        match ao.name() {
            AOName::Discard => {
                self.restrict(ao.player_id(), vec![ao.card().card_to_char()]);
            },
            AOName::RevealRedraw => {
                self.reveal_redraw(ao.player_id(), ao.card().card_to_char());
            },
            AOName::ExchangeDraw => {
                
            },
            AOName::ExchangeChoice => {
                self.ambassador(ao.player_id());
            },
            _ => {

            },
        };
    }
    pub fn pop(&mut self) {
        panic!("No pops! This goes in one direction only!")
    }
    fn player_slice_bounds(player_id: usize) -> (usize, usize) {
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
    
    // TODO: Test
    /// RevealRedraw: Mixes a single known card of a player and another player
    /// Returns all the possible outcome combinations
    pub fn mix_one_char(
        &self,
        original_str: &str,
        player_reveal: usize,
        player_other: usize,
        card_i: char,
    ) -> Vec<String> {
        // Helper to determine the slice boundaries for each player's cards
        fn player_slice_bounds(player_id: usize) -> (usize, usize) {
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
    
        // For each card in player_j, swap with card_i
        for &card_j in &player_other_cards {
            // Work on local copies
            let mut new_i_cards = player_reveal_cards.clone();
            let mut new_j_cards = player_other_cards.clone();
    
            // Remove the card_i from i and card_j from j
            // (We remove only one occurrence if duplicates are present)
            if let Some(pos_i) = new_i_cards.iter().position(|&c| c == card_i) {
                new_i_cards.remove(pos_i);
            }
            if let Some(pos_j) = new_j_cards.iter().position(|&c| c == card_j) {
                new_j_cards.remove(pos_j);
            }
    
            // Swap them
            new_i_cards.push(card_j);
            new_j_cards.push(card_i);
    
            // Sort each player's cards to maintain ascending order in the final string
            new_i_cards.sort_unstable();
            new_j_cards.sort_unstable();
    
            // Rebuild the final string
            let mut new_string = String::new();
            // For each of the 7 players, we either insert the new_i_cards, new_j_cards, or original
            for player_id in 0..7 {
                let (start, end) = player_slice_bounds(player_id);
                if player_id == player_reveal {
                    new_string.push_str(&new_i_cards.iter().collect::<String>());
                } else if player_id == player_other {
                    new_string.push_str(&new_j_cards.iter().collect::<String>());
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
        let total = i_count + j_count;
    
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
                    new_j_cards.remove(pos);
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
        self.calculated_states.retain(|state| {
            Self::player_has_cards(
                state,
                player_reveal,
                &card_chars,
                &self.index_start_arr,
                &self.index_end_arr,
            )
        });
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
    pub fn printlog(&self) {
        log::info!("calculated_states.len: {}", self.calculated_states.len());
        log::info!("{:?}", self.calculated_states);
    }
}
