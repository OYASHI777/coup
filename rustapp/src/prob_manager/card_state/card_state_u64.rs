use std::{hash::Hasher, process::Output};

use crate::history_public::Card;
use crate::prob_manager::permutation_generator::gen_table_combinations;

use super::CardPermState;
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CardStateu64(u64);
impl std::hash::Hash for CardStateu64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl std::fmt::Display for CardStateu64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in 0..7 as usize {
            let mut cards = self.get_player_cards_unsorted(i);
            cards.sort_unstable();
            for card in cards.iter() {
                write!(f, "{}", card.card_to_str())?
            }
        }
        Ok(())
    }
}
impl std::fmt::Debug for CardStateu64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in 0..7 as usize {
            let mut cards = self.get_player_cards_unsorted(i);
            cards.sort_unstable();
            for card in cards.iter() {
                write!(f, "{}", card.card_to_str())?
            }
        }
        Ok(())
    }
}
use crate::prob_manager::coup_const::{BAG_SIZES, TOKENS};
impl CardStateu64 {
    /// Default creation for the uninitiated
    pub fn new() -> Self {
        Self(0)
    }
    /// Returns the cards a player has (unsorted)
    pub fn get_player_cards_unsorted(&self, player_id: usize) -> Vec<Card> {
        debug_assert!(player_id < 7, "Invalid player ID");
        let shift = player_id * 6;

        let mut player_cards = Vec::with_capacity(4); // 4 reqquired for the ExchangeChoice
        if player_id < 6 {
            let mask = 0b111111;
            let player_bits = (self.0 >> shift) & mask;
            for i in 0..2 {
                let card_value = (player_bits >> (i * 3)) & 0b111;
                player_cards.push(Card::try_from(card_value as u8).unwrap());
            }
        } else {
            let mask = 0b111111111;
            let player_bits = (self.0 >> shift) & mask;
            for i in 0..3 {
                let card_value = (player_bits >> (i * 3)) & 0b111;
                player_cards.push(Card::try_from(card_value as u8).unwrap());
            }
        }
        player_cards
    }
    /// Sets the cards for a particular player
    pub fn set_player_cards(&mut self, player_id: usize, cards: &[Card]) {
        debug_assert!(player_id < 7, "Invalid player ID");
        let shift = player_id * 6;
        let mask = match player_id {
            0 => 0b111111,
            1 => 0b111111,
            2 => 0b111111,
            3 => 0b111111,
            4 => 0b111111,
            5 => 0b111111,
            6 => 0b111111111, 
            _ => unreachable!(),
        };
        let mut encoded_cards: u64 = 0;
        for (i, card) in cards.iter().enumerate() {
            encoded_cards |= (*card as u64) << (i*3); 
        }
        self.0 &= !(mask << shift);
        self.0 |= (encoded_cards as u64) << shift;
    }
    // A small helper to generate all combinations of `k` elements from a slice.
    pub fn combinations<T: Clone>(arr: &[T], k: usize) -> Vec<Vec<T>> {
        let mut result = Vec::with_capacity(10); // Its always < 10
    
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
        // push_combination(&indices, arr, &mut result);
    
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
}

impl CardPermState for CardStateu64 {
    /// Creates CardPermState from str reference
    fn from_str(str: &str) -> Self {
        let cards: Vec<Card> = str.chars().map(|c| Card::char_to_card(c)).collect();
        let mut state = Self::new();
        state.set_player_cards(0, &cards[0..2]);
        state.set_player_cards(1, &cards[2..4]);
        state.set_player_cards(2, &cards[4..6]);
        state.set_player_cards(3, &cards[6..8]);
        state.set_player_cards(4, &cards[8..10]);
        state.set_player_cards(5, &cards[10..12]);
        state.set_player_cards(6, &cards[12..15]);
        state
    }

    fn gen_table_combinations() -> Vec<Self> {
        // This is a lazy unoptimized implementation as it's only intended to be called once at start of program
        let all_states: Vec<String> = gen_table_combinations(TOKENS, &BAG_SIZES);
        let output = all_states.iter().map(|state| Self::from_str(state)).collect();
        log::info!("all_states: {:?}", output);
        output
    }

    fn player_has_cards(&self, player_id: usize, cards: &[Card]) -> bool {
        let mut player_cards: Vec<Card> = self.get_player_cards_unsorted(player_id);
        // Check if all needed cards exist in extracted_cards
        for needed in cards {
            if let Some(pos) = player_cards.iter().position(|&c| c == *needed) {
                player_cards.remove(pos);
            } else {
                return false;
            }
        }
        true
    }
    fn player_has_cards_after_draw(&self, player_id: usize, cards: &[Card], draw: &[Card]) -> bool {
        let mut player_cards: Vec<Card> = self.get_player_cards_unsorted(player_id);
        player_cards.extend_from_slice(draw);
        // Check if all needed cards exist in extracted_cards
        for needed in cards {
            if let Some(pos) = player_cards.iter().position(|&c| c == *needed) {
                player_cards.remove(pos);
            } else {
                return false;
            }
        }
        true
    }
    fn player_card_count(&self, player_id: usize, card: Card) -> u8 {
        let cards = self.get_player_cards_unsorted(player_id);
        let mut counter = 0;
        for card_player in cards.iter() {
            if card == *card_player {
                counter += 1;
            } 
        }
        counter
    }
    fn player_card_counts(&self, player_id: usize) -> [u8; 5] {
        let cards = self.get_player_cards_unsorted(player_id);
        let mut output = [0; 5];
        for card_player in cards.iter() {
            output[*card_player as usize] += 1
        }
        output
    }
    fn mix_one_card(&self, player_id: usize, player_other: usize, card: Card) -> Vec<Self> {
        debug_assert!(player_id < 7 && player_other < 7, "Invalid player IDs");
        // Get player cards
        let mut player_cards = self.get_player_cards_unsorted(player_id);
        let other_cards = self.get_player_cards_unsorted(player_other);
        
        // If `player_id` doesn't have `card`, return empty result
        if !player_cards.contains(&card) {
            return vec![];
        }

        let mut result = Vec::with_capacity(3);

        // Remove only 1 `card` from `player_id`
        if let Some(pos) = player_cards.iter().position(|&c| c == card) {
            player_cards.swap_remove(pos);
        }

        // First case: the same card is received back, meaning no change
        // result.push(*self);

        // Generate all swaps with `player_other`'s cards
        for (i, &other_card) in other_cards.iter().enumerate() {
            let mut new_player_cards = player_cards.clone();
            let mut new_other_cards = other_cards.clone();

            // Remove the `other_card` from `player_other`
            new_other_cards.swap_remove(i);

            // Swap them
            new_player_cards.push(other_card);
            new_other_cards.push(card);

            // Create new state
            let mut new_state = *self;
            new_state.set_player_cards(player_id, &new_player_cards);
            new_state.set_player_cards(player_other, &new_other_cards);

            // Add new state to the result
            result.push(new_state);
        }

        result
    }

    fn mix_multiple_chars_with_player6(&self, player_id: usize, player_dead_cards: &[Card]) -> Vec<Self> {
        debug_assert!(player_id < 6, "Only players 0-5 can swap with player 6");

        // Get player cards
        let mut player_cards = self.get_player_cards_unsorted(player_id);
        let player6_cards = self.get_player_cards_unsorted(6);

        // Remove "dead" cards from player
        for dead_card in player_dead_cards {
            if let Some(pos) = player_cards.iter().position(|&c| c == *dead_card) {
                player_cards.swap_remove(pos);
            }
        }

        // Combine remaining cards from both players
        let mut combined_cards = player_cards.clone();
        combined_cards.extend_from_slice(&player6_cards);

        let player_count = player_cards.len(); // Typically 2 cards if player_id < 6

        // Generate all ways to reassign the total cards
        let combinations = Self::combinations(&combined_cards, player_count);
        let mut results = Vec::with_capacity(combinations.len());
        for combo_for_player in combinations {
            // Sort the chosen cards for player_id
            let mut new_player_cards = player_dead_cards.to_vec();
            new_player_cards.extend_from_slice(&combo_for_player);
            new_player_cards.sort_unstable();

            // The leftover cards go to player 6
            let mut new_player6_cards = combined_cards.clone();
            for &card in &combo_for_player {
                if let Some(pos) = new_player6_cards.iter().position(|&x| x == card) {
                    new_player6_cards.swap_remove(pos);
                }
            }
            new_player6_cards.sort_unstable();

            // Create a new state with the redistributed cards
            let mut new_state = *self;
            new_state.set_player_cards(player_id, &new_player_cards);
            new_state.set_player_cards(6, &new_player6_cards);

            results.push(new_state);
        }

        results
    }
    
    fn player_swap_cards(&self, player_i: usize, player_j: usize, card_i: Card, card_j: Card) -> Option<Self> {
        let mut new_state = self.clone();
        let mut player_i_cards = self.get_player_cards_unsorted(player_i);
        let mut player_j_cards = self.get_player_cards_unsorted(player_j);
        if let Some(pos_i) = player_i_cards.iter().rposition(|c| *c == card_i) {
            if let Some(pos_j) = player_j_cards.iter().rposition(|c| *c == card_j) {
                player_i_cards[pos_i] = card_j;
                player_j_cards[pos_j] = card_i;
                new_state.set_player_cards(player_i, &player_i_cards);
                new_state.set_player_cards(player_j, &player_j_cards);
                return Some(new_state);
            }
        }
        None
    }
    fn player_swap_cards_draw_relinquish(&self, player_drawing: usize, player_drawn: usize, draw: &[Card], relinquish: &[Card]) -> Option<Self> {
        let mut new_state = self.clone();
        let mut player_drawing_cards = self.get_player_cards_unsorted(player_drawing);
        let mut player_drawn_cards = self.get_player_cards_unsorted(player_drawn);
        if let Some(pos) = player_drawn_cards.iter().position(|c| *c == relinquish[0]) {
            player_drawn_cards.swap_remove(pos);
        }
        player_drawing_cards.push(relinquish[0]);
        if let Some(pos) = player_drawn_cards.iter().position(|c| *c == relinquish[1]) {
            player_drawn_cards.swap_remove(pos);
        }
        player_drawing_cards.push(relinquish[1]);
        if let Some(pos) = player_drawing_cards.iter().position(|c| *c == draw[0]) {
            player_drawing_cards.swap_remove(pos);
        }
        player_drawn_cards.push(draw[0]);
        if let Some(pos) = player_drawing_cards.iter().position(|c| *c == draw[1]) {
            player_drawing_cards.swap_remove(pos);
        }
        player_drawn_cards.push(draw[1]);
        // Remove this to check if able to add illegal moves! for simulation
        if player_drawing_cards.len() < 3 && player_drawn_cards.len() < 4 {
            player_drawing_cards.sort_unstable();
            player_drawn_cards.sort_unstable();
            new_state.set_player_cards(player_drawing, &player_drawing_cards);
            new_state.set_player_cards(player_drawn, &player_drawn_cards);
            return Some(new_state);
        } 
        None
    }
}
