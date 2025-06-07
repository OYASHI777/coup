// Journey here
// Tried to iteratively find naive probability by filtering
// Concurrent and normal iteration times are around 0.1 s calculation of belief is around 0.1 seconds
// This is too long
// Tried instead to save into hashmap and store in bson

use std::hash::Hash;
use std::fmt::{Debug, Display};
use std::hint::unreachable_unchecked;
use crate::history_public::{Card, AOName, ActionObservation};
use crate::traits::prob_manager::coup_analysis::{CoupPossibilityAnalysis, CoupTraversal};
use crate::traits::prob_manager::card_state::CardPermState;
use crate::prob_manager::coup_const::MAX_PERM_STATES;
use ahash::AHashSet;
/// Struct that card count manually, by simulating movement of cards (as chars) for all possible permutations

pub struct BruteCardCountManagerGeneric<T: CardPermState> 
where
    T: CardPermState + Hash + Eq + Copy + Clone + Display + Debug,
{
    private_player: Option<usize>,
    history: Vec<ActionObservation>,
    all_states: Vec<T>,
    calculated_states: Vec<T>, // All the states that fulfil current constraints
    public_constraints: Vec<Vec<Card>>,
    inferred_constraints: Vec<Vec<Card>>,
    impossible_constraints: [[bool; 5]; 7],
    impossible_constraints_2: [[[bool; 5]; 5]; 7],
    impossible_constraints_3: [[[bool; 5]; 5]; 5],
    auto_calculate_impossible_constraints_2: bool,
    auto_calculate_impossible_constraints_3: bool,
    impossible_constraints_2_is_updated: bool,
    impossible_constraints_3_is_updated: bool,
}
impl<T> BruteCardCountManagerGeneric<T> 
where
    T: CardPermState + Hash + Eq + Copy + Clone + Display + Debug,
{
    /// Constructor
    pub fn new(auto_calc_2: bool, auto_calc_3: bool) -> Self {
        let history = Vec::with_capacity(60);
        let all_states: Vec<T> = T::gen_table_combinations();
        let calculated_states: Vec<T> = all_states.clone().into_iter().collect();
        let mut public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2); 6];
        public_constraints.push(Vec::with_capacity(3));
        let mut inferred_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2); 6];
        inferred_constraints.push(Vec::with_capacity(3));
        let impossible_constraints = [[false; 5]; 7];
        let impossible_constraints_2 = [[[false; 5]; 5]; 7];
        let impossible_constraints_3 = [[[false; 5]; 5]; 5];
        Self {
            history,
            private_player: None,
            all_states,
            calculated_states,
            public_constraints,
            inferred_constraints,
            impossible_constraints,
            impossible_constraints_2,
            impossible_constraints_3,
            auto_calculate_impossible_constraints_2: auto_calc_2,
            auto_calculate_impossible_constraints_3: auto_calc_3,
            impossible_constraints_2_is_updated: auto_calc_2,
            impossible_constraints_3_is_updated: auto_calc_3,
        }
    }
    /// total number of possible states
    pub fn len(&self) -> usize {
        self.calculated_states.len()
    }
    /// adds public constraint
    pub fn add_public_constraint(&mut self, player_id: usize, card: Card) {
        self.public_constraints[player_id].push(card);
    }
    /// Use Rayon to parallelize the process of running `mix_one_char` on
    /// each state in `self.calculated_states`, collecting all results
    /// into a new AHashSet.
    pub fn reveal_redraw(
        &mut self,
        player_reveal: usize,
        card_i: Card,
    ) {
        let mut current_dead_cards: Vec<Card> = self.public_constraints[player_reveal].clone();
        current_dead_cards.push(card_i);
        self.restrict(player_reveal, &current_dead_cards);
        let mut temp_set = AHashSet::with_capacity(MAX_PERM_STATES);
        let initial_len = self.calculated_states.len();
        let mut i: usize = 0;
        while i < initial_len {
            let t = self.calculated_states[i];
            for new_state in t.mix_one_card(player_reveal, 6, card_i).iter() {
                if !temp_set.contains(new_state) {
                    temp_set.insert(*new_state);
                    self.calculated_states.push(*new_state);
                }
            }
            i += 1;
        }
        self.print_legal_states();
    }
    /// Use Rayon to parallelize the process of running `mix_one_char` on
    /// each state in `self.calculated_states`, collecting all results
    /// into a new AHashSet.
    pub fn ambassador(
        &mut self,
        player_reveal: usize,
    ) {
        let initial_len: usize = self.calculated_states.len();
        let mut temp_set = AHashSet::with_capacity(MAX_PERM_STATES - initial_len);
        let mut i: usize = 0;
        while i < initial_len {
            let t = self.calculated_states[i];
            for new_state in t.mix_multiple_chars_with_player6(player_reveal, &self.public_constraints[player_reveal]).iter() {
                if !temp_set.contains(new_state) {
                    temp_set.insert(*new_state);
                    self.calculated_states.push(*new_state);
                }
            }
            i += 1;
        }
    }
    pub fn redraw_swap(&mut self, player_reveal: usize, card_reveal: Card, card_redraw: Card) {
        debug_assert!(card_reveal != card_redraw, "Ensure redraw_swap used only when redraw and reveal are different!");
        let temp_states = self.calculated_states.clone();
        let mut temp_set = AHashSet::with_capacity(self.calculated_states.len());
        self.calculated_states.clear();
        let mut player_cards = self.public_constraints[player_reveal].clone();
        player_cards.push(Card::try_from(card_reveal as u8).unwrap());
        for state in temp_states.iter() {
            if state.player_has_cards(player_reveal, &player_cards) {
                if let Some(new_state) = state.player_swap_cards(player_reveal, 6, card_reveal, card_redraw) {
                    if !temp_set.contains(&new_state) {
                        temp_set.insert(new_state);
                        self.calculated_states.push(new_state);
                    }
                }
            }
        }
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
        self.print_legal_states();
    }
    /// Emulates the ExchangeChoice swap
    pub fn exchange_choice_swap(&mut self, player_exchange: usize, draw: &[Card], relinquish: &[Card]) {
        log::info!("Brute Prob: exchange_choice_swap Ran: player: {}, draw: {:?}, relinquish: {:?}", player_exchange, draw, relinquish);
        self.print_legal_states();
        let temp_states = self.calculated_states.clone();
        let mut temp_set = AHashSet::with_capacity(self.calculated_states.len());
        self.calculated_states.clear();
        for state in temp_states.iter() {
            if let Some(new_state) = state.player_swap_cards_draw_relinquish(player_exchange, 6, &self.public_constraints[player_exchange], &vec![], draw, relinquish) {
                if !temp_set.contains(&new_state) {
                    temp_set.insert(new_state);
                    self.calculated_states.push(new_state);
                }
            }
        }

    }
    /// This function returns true if a player can have a particular card
    pub fn player_can_have_card(&self, player_id: usize, card: Card) -> bool {
        self.calculated_states.iter().
        any(|state| state.player_has_cards(player_id, &[card]))
    }
    /// This function returns true if a player can have all of these cards
    /// Does not care about alive or dead status
    pub fn player_can_have_cards(&self, player_id: usize, cards: &[Card]) -> bool {
        // Check in paralle if any state satisfies the requirement
        // let mut deduplicated = cards.to_vec();
        // deduplicated.sort_unstable();
        // deduplicated.dedup();
        // for card in deduplicated {
        //     if !self.player_can_have_card_alive(player_id, card) {
        //         return false;
        //     }
        // }
        self.calculated_states.iter().any(|state| state.player_has_cards(player_id, cards))
    }
    /// Checks if player can have cards if they also draw a set of cards
    pub fn player_can_have_cards_after_draw(&self, player_id: usize, dead_cards: &[Card], cards: &[Card], draw: &[Card]) -> bool {
        let mut cards_count = [0usize; 5];
        cards.iter().for_each(|c| cards_count[*c as usize] += 1);
        draw.iter().for_each(|c| if cards_count[*c as usize] > 0 {cards_count[*c as usize] -= 1});
        dead_cards.iter().for_each(|c| cards_count[*c as usize] += 1);
        let mut check_cards = Vec::with_capacity(2);
        cards_count.iter().enumerate().for_each(|(card_num, count)| check_cards.extend(std::iter::repeat(Card::try_from(card_num as u8).unwrap()).take(*count)));
        if check_cards.is_empty() {
            true
        } else {
            self.player_can_have_cards(player_id, &check_cards)
        }
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
    pub fn set_impossible_constraints_2(&mut self) {
        // Early return if we have no states; then every card is impossible in all states
        // or every card is possible—depending on your game logic. Usually, with zero states,
        // "cannot have" is trivially true for all. But check game logic as needed.
        if self.calculated_states.is_empty() {
            self.impossible_constraints_2 = [[[true; 5]; 5]; 7];
            return
        } else {
            self.impossible_constraints_2 = [[[false; 5]; 5]; 7];
        }

        // For each player
        for player_id in 0..7 {
            // For each card variant (assuming your Card enum maps 1:1 to these indices)
            // e.g., 0 = Duke, 1 = Assassin, 2 = Captain, 3 = Ambassador, 4 = Contessa
            if self.public_constraints[player_id].len() > 0 {
                self.impossible_constraints_2[player_id] = [[true; 5]; 5];
                continue;
            }
            for card_idx_i in 0..5 as usize {
                for card_idx_j in card_idx_i..5 as usize {
                    // Convert card_idx -> Card -> char
                    let mut card_counts: [u8; 5] = [0; 5];
                    card_counts[card_idx_i] += 1;
                    card_counts[card_idx_j] += 1;
    
                    // We want to know if there's ANY state in which the player's substring
                    // includes `card_char`. If there is, then `cannot_have` is false.
                    // If we can't find it in ANY state, `cannot_have` is true.
                    let found_in_any_state = self.calculated_states.iter().any(|state| {
                        let actual_count = state.player_card_counts(player_id);
                        for card_num in 0..5 {
                            if actual_count[card_num] < card_counts[card_num] {
                                return false;
                            }
                        }
                        true
                    });
    
                    // If found_in_any_state == false, that means:
                    // "There is NO state in which the player has this card alive"
                    // So the player "cannot have" it => result = true
                    self.impossible_constraints_2[player_id][card_idx_i][card_idx_j] = !found_in_any_state;
                    self.impossible_constraints_2[player_id][card_idx_j][card_idx_i] = !found_in_any_state;
                }
            }
        }
        self.impossible_constraints_2_is_updated = true;
    }
    pub fn set_impossible_constraints_3(&mut self) {

        // Early return if we have no states; then every card is impossible in all states
        // or every card is possible—depending on your game logic. Usually, with zero states,
        // "cannot have" is trivially true for all. But check game logic as needed.
        if self.calculated_states.is_empty() {
            self.impossible_constraints_3 = [[[true; 5]; 5]; 5];
            return
        } else {
            self.impossible_constraints_3 = [[[false; 5]; 5]; 5];
        }

        // For each player
        for card_idx_i in 0..5 as usize {
            for card_idx_j in card_idx_i..5 as usize {
                for card_idx_k in card_idx_j..5 as usize {

                    // Convert card_idx -> Card -> char
                    let mut card_counts: [u8; 5] = [0; 5];
                    card_counts[card_idx_i] += 1;
                    card_counts[card_idx_j] += 1;
                    card_counts[card_idx_k] += 1;
    
                    // We want to know if there's ANY state in which the player's substring
                    // includes `card_char`. If there is, then `cannot_have` is false.
                    // If we can't find it in ANY state, `cannot_have` is true.
                    let found_in_any_state = self.calculated_states.iter().any(|state| {
                        let actual_count = state.player_card_counts(6);
                        for card_num in 0..5 {
                            if actual_count[card_num] < card_counts[card_num] {
                                return false;
                            }
                        }
                        true
                    });
    
                    // If found_in_any_state == false, that means:
                    // "There is NO state in which the player has this card alive"
                    // So the player "cannot have" it => result = true
                    self.impossible_constraints_3[card_idx_i][card_idx_j][card_idx_k] = !found_in_any_state;
                    self.impossible_constraints_3[card_idx_i][card_idx_k][card_idx_j] = !found_in_any_state;
                    self.impossible_constraints_3[card_idx_j][card_idx_i][card_idx_k] = !found_in_any_state;
                    self.impossible_constraints_3[card_idx_j][card_idx_k][card_idx_i] = !found_in_any_state;
                    self.impossible_constraints_3[card_idx_k][card_idx_i][card_idx_j] = !found_in_any_state;
                    self.impossible_constraints_3[card_idx_k][card_idx_j][card_idx_i] = !found_in_any_state;
                }
            }
        }
        self.impossible_constraints_3_is_updated = true;
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
    /// Returns a 7x5x5 boolean array `[[[bool; 5]; 5]; 7]`.
    ///
    /// - Outer index = player (0..6)
    /// - Inner index = card as usize (0..4 or 0..5, depending on how you define Card).
    ///
    /// `result[player_id][card_index_i][card_index_j]` will be `true` if, **in every** state within
    /// `self.calculated_states`, that `player_id` does **not** have those cards.
    ///
    /// Returns an array that is true if a player does cannot have that card alive
    pub fn validated_impossible_constraints_2(&self) -> [[[bool; 5]; 5]; 7] {
        self.impossible_constraints_2.clone()
    }
    /// Returns a 7x5x5 boolean array `[[[bool; 5]; 5]; 5]`.
    ///
    /// - Outer index = player (0..6)
    /// - Inner index = card as usize (0..4 or 0..5, depending on how you define Card).
    ///
    /// `result[card_index_i][card_index_j][card_index_k]` will be `true` if, **in every** state within
    /// `self.calculated_states`, that `player_id` does **not** have those cards.
    ///
    /// Returns an array that is true if a player does cannot have that card alive
    pub fn validated_impossible_constraints_3(&self) -> [[[bool; 5]; 5]; 5] {
        self.impossible_constraints_3.clone()
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
        // Setting to false if no need for auto calculation
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
        self.impossible_constraints_2_is_updated = self.auto_calculate_impossible_constraints_2;
        self.impossible_constraints_3_is_updated = self.auto_calculate_impossible_constraints_3;
        if self.auto_calculate_impossible_constraints_2 {
            self.set_impossible_constraints_2();
        }
        if self.auto_calculate_impossible_constraints_3 {
            self.set_impossible_constraints_3();
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
        log::info!("Brute impossible constraints: {:?}", self.validated_impossible_constraints());
        log::info!("Brute impossible constraints 2: {:?}", self.validated_impossible_constraints_2());
        log::info!("Brute impossible constraints 3: {:?}", self.validated_impossible_constraints_3());
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

impl<T> CoupTraversal for BruteCardCountManagerGeneric<T>
where
    T: CardPermState + Hash + Eq + Copy + Clone + Display + Debug,
{
    fn start_public(&mut self) {
        // Do nothing
    }

    fn start_private(&mut self, player: usize, cards: &[Card; 2]) {
        self.private_player = Some(player);
        self.restrict(player, cards);
        // self.inferred_constraints[player].push(cards[0]);
        // self.inferred_constraints[player].push(cards[1]);
        // self.inferred_constraints[player].sort_unstable();
        self.update_constraints();
        self.set_impossible_constraints_2();
        self.set_impossible_constraints_3();
    }

    fn push_ao_public(&mut self, action: &ActionObservation) {
        // TODO: Match on action instead?
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
                match no_cards {
                    1 => {
                        self.public_constraints[*player_id].push(card[0]);
                        if self.public_constraints[*player_id].len() == 2 {
                            // Handles the case where the players' dead cards are both the same, without this, restrict won't ensure player has both cards
                            self.restrict(*player_id, &self.public_constraints[*player_id].clone());
                        } else {
                            self.restrict(*player_id, &[card[0]]);
                        }
                    },
                    2 => {
                        self.public_constraints[*player_id].push(card[0]);
                        self.public_constraints[*player_id].push(card[1]);
                        self.restrict(*player_id, card);
                    }
                    _ => {
                        unsafe {
                            unreachable_unchecked()
                        }
                    }
                }
                self.update_constraints();
            },
            ActionObservation::RevealRedraw { player_id, reveal, .. } => {
                self.reveal_redraw(*player_id, *reveal);
                self.update_constraints();
            },
            ActionObservation::ExchangeChoice { player_id, .. } => {
                self.ambassador(*player_id);
                self.update_constraints();
            },
            _ => {},
        }
        self.history.push(action.clone());
    }

    fn push_ao_public_lazy(&mut self, action: &ActionObservation) {
        self.push_ao_public(action);
    }

    fn push_ao_private(&mut self, action: &ActionObservation) {
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
                let mut card_restrictions = Vec::with_capacity(2);
                for i in 0..*no_cards {
                    card_restrictions.push(card[i]);
                    self.public_constraints[*player_id].push(card[i]);
                }
                self.restrict(*player_id, &card_restrictions);
                self.update_constraints();
            },
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
                let mut current_dead_cards: Vec<Card> = self.public_constraints[*player_id].clone();
                current_dead_cards.push(*reveal);
                self.restrict(*player_id, &current_dead_cards);
                if *reveal != *redraw {
                    self.restrict(6, &[*redraw]);
                    self.redraw_swap(*player_id, *reveal, *redraw);
                }
                self.update_constraints();
            },
            ActionObservation::ExchangeDraw { card, .. } => {
                self.restrict(6, card);
                self.update_constraints();
            },
            ActionObservation::ExchangeChoice { player_id, relinquish } => {
                if let Some(ActionObservation::ExchangeDraw { card: draw, .. }) = self.history.last().cloned() {
                    self.exchange_choice_swap(*player_id, &draw, relinquish);
                } else {
                    debug_assert!(false, "Some shit is wrong man");
                }
                self.update_constraints();
            },
            _ => {}
        }
        self.history.push(action.clone());
    }

    fn push_ao_private_lazy(&mut self, action: &ActionObservation) {
        self.push_ao_private(action);
    }

    /// Unsupported, as information is loss after pushing and cannot be reverted
    fn pop(&mut self) {
        unimplemented!("brute prob does not support pop as it is irreversible")
    }

    fn reset(&mut self) {
        self.history.clear();
        self.private_player = None;
        self.public_constraints = vec![Vec::with_capacity(2); 6];
        self.public_constraints.push(Vec::with_capacity(3));
        self.inferred_constraints = vec![Vec::with_capacity(2); 6];
        self.inferred_constraints.push(Vec::with_capacity(3));
        self.impossible_constraints = [[false; 5]; 7];
        self.impossible_constraints_2 = [[[false; 5]; 5]; 7];
        self.impossible_constraints_3 = [[[false; 5]; 5]; 5];
        self.calculated_states = self.all_states.clone().into_iter().collect();
    }
}

impl<T> CoupPossibilityAnalysis for BruteCardCountManagerGeneric<T> 
where
    T: CardPermState + Hash + Eq + Copy + Clone + Display + Debug,
{
    /// Returns all the dead cards for each player that we are certain they have
    /// Assumes calculates states align with latest constraints
    fn public_constraints(&mut self) -> &Vec<Vec<Card>> {
        &self.public_constraints
    }

    fn sorted_public_constraints(&mut self) -> &Vec<Vec<Card>> {
        // Sorted in update_constraints
        &self.public_constraints
    }
    /// Returns all the dead cards for each player that we are certain they have
    /// Assumes calculates states align with latest constraints
    fn inferred_constraints(&mut self) -> &Vec<Vec<Card>> {
        &self.inferred_constraints
    }

    fn sorted_inferred_constraints(&mut self) -> &Vec<Vec<Card>> {
        // Sorted in update_constraints
        &self.inferred_constraints
    }

    fn player_impossible_constraints(&mut self) -> &[[bool; 5]; 7] {
        &self.impossible_constraints
    }

    fn player_impossible_constraints_paired(&mut self) -> &[[[bool; 5]; 5]; 7] {
        if !self.impossible_constraints_2_is_updated {
            self.set_impossible_constraints_2();
        }
        &self.impossible_constraints_2
    }
    
    fn player_impossible_constraints_triple(&mut self) -> &[[[bool; 5]; 5]; 5] {
        if !self.impossible_constraints_3_is_updated {
            self.set_impossible_constraints_3();
        }
        &self.impossible_constraints_3
    }

    fn player_can_have_card_alive(&mut self, player: usize, card: Card) -> bool {
        !self.impossible_constraints[player][card as usize]
    }

    fn player_can_have_card_alive_lazy(&mut self, player: usize, card: Card) -> bool {
        !self.impossible_constraints[player][card as usize]
    }

    fn player_can_have_cards_alive(&mut self, player: usize, cards: &[Card]) -> bool {
        if player < 6 {
            if cards.len() == 2 {
                if !self.impossible_constraints_2_is_updated {
                    self.set_impossible_constraints_2();
                }
                return !self.impossible_constraints_2[player][cards[0] as usize][cards[1] as usize]
            } else if cards.len() == 1 {
                return self.player_can_have_card_alive(player, cards[0])
            }
        } else if player == 6 {
            if cards.len() == 1 {
                return self.player_can_have_card_alive(player, cards[0])
            } else if cards.len() == 2 {
                if !self.impossible_constraints_2_is_updated {
                    self.set_impossible_constraints_2();
                }
                return !self.impossible_constraints_2[player][cards[0] as usize][cards[1] as usize]
            } else if cards.len() == 3 {
                if !self.impossible_constraints_3_is_updated {
                    self.set_impossible_constraints_3();
                }
                return !self.impossible_constraints_3[cards[0] as usize][cards[1] as usize][cards[2] as usize]
            }
        }
        false
    }

    fn player_can_have_cards_alive_lazy(&mut self, player: usize, cards: &[Card]) -> bool {
        if player < 6 {
            if cards.len() == 2 {
                if !self.impossible_constraints_2_is_updated {
                    self.set_impossible_constraints_2();
                }
                return !self.impossible_constraints_2[player][cards[0] as usize][cards[1] as usize]
            } else if cards.len() == 1 {
                return self.player_can_have_card_alive(player, cards[0])
            }
        } else if player == 6 {
            if cards.len() == 1 {
                return self.player_can_have_card_alive(player, cards[0])
            } else if cards.len() == 2 {
                if !self.impossible_constraints_2_is_updated {
                    self.set_impossible_constraints_2();
                }
                return !self.impossible_constraints_2[player][cards[0] as usize][cards[1] as usize]
            } else if cards.len() == 3 {
                if !self.impossible_constraints_3_is_updated {
                    self.set_impossible_constraints_3();
                }
                return !self.impossible_constraints_3[cards[0] as usize][cards[1] as usize][cards[2] as usize]
            }
        }
        false
    }

    fn is_legal_move_public(&mut self, action_observation: &ActionObservation) -> bool {
        match action_observation {
            ActionObservation::Discard { player_id, card, no_cards } => {
                if *no_cards == 1 {
                    self.player_can_have_card_alive_lazy(*player_id, card[0])
                } else {
                    self.player_can_have_cards_alive_lazy(*player_id, card)
                }
            },
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
                self.player_can_have_card_alive_lazy(*player_id, *reveal)
            },
            _ => true,
        }
    }

    fn is_legal_move_private(&mut self, action_observation: &ActionObservation) -> bool {
        match action_observation {
            ActionObservation::Discard { player_id, card, no_cards } => {
                if *no_cards == 1 {
                    self.player_can_have_card_alive_lazy(*player_id, card[0])
                } else {
                    self.player_can_have_cards_alive_lazy(*player_id, card)
                }
            },
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
                if *reveal == *redraw {
                    true
                } else {
                    self.player_can_have_card_alive_lazy(*player_id, *reveal)
                    && self.player_can_have_card_alive_lazy(6, *redraw)
                }
            },
            ActionObservation::ExchangeDraw { card, .. } => {
                self.player_can_have_cards_alive_lazy(6, card)
            },
            ActionObservation::ExchangeChoice { player_id, relinquish } =>  {
                let player_dead = self.public_constraints()[*player_id].len() as u8;
                let mut required = [0u8; 5];
                relinquish.iter().for_each(|c| required[*c as usize] += 1); 
                if let ActionObservation::ExchangeDraw { card: draw, .. } = self.history[self.history.len() - 1] {
                    draw.iter().for_each(|c| if required[*c as usize] > 0 { required[*c as usize] -= 1 } );
                }
                let total_cards = required.iter().sum::<u8>();
                if total_cards == 0 {
                    true
                } else if total_cards + player_dead > 2 {
                    false
                } else {
                    // if updated {..} just check the state
                    let mut cards = Vec::with_capacity(2);
                    for c in 0..5 {
                        for _ in 0..required[c] {
                            cards.push(Card::try_from(c as u8).unwrap());
                        }
                    }
                    self.player_can_have_cards_alive_lazy(*player_id, &cards)
                }
                // if let ActionObservation::ExchangeDraw { card: draw, .. } = &self.history[self.history.len() - 1] {
                //     return self.player_can_have_cards_after_draw(*player_id, &self.public_constraints[*player_id], relinquish, draw)
                // }
                // false
            },
            _ => true,
        }
    }
}