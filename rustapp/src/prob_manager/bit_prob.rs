// Journey here
// Tried to iteratively find naive probability by filtering
// Concurrent and normal iteration times are around 0.1 s calculation of belief is around 0.1 seconds
// This is too long
// Tried instead to save into hashmap and store in bson

use crate::history_public::{Card, AOName, ActionObservation};
use super::bitconstraint_iterative::CompressedCollectiveConstraint;
use std::usize;

pub struct BitCardCountManager {
    // a vec of constraints to push and pop
    // dead cards to push or pop
    // Will not locally store game history, jsut the constraint history
    constraint_history: Vec<Option<CompressedCollectiveConstraint>>, // I think None is stored if there are no changes
    // distfromlast tells u how far away to fetch the last constraint_history
    dist_from_last: Vec<usize>,
}
impl BitCardCountManager {
    /// Constructor
    pub fn new() -> Self {
        BitCardCountManager{
            constraint_history: Vec::with_capacity(240),
            dist_from_last:Vec::with_capacity(240),
        }
    }
    /// Returns everything to original state
    pub fn reset(&mut self) {
        self.constraint_history = Vec::with_capacity(240);
        self.dist_from_last = Vec::with_capacity(240);
    }
    /// Returns how far away to look to find the last constraint_history
    pub fn prev_index(&self) -> usize {
        self.dist_from_last[self.dist_from_last.len() - 1]
    }
    /// Checks if the latest constraint is completely empty
    pub fn latest_constraint_is_empty(&self) -> bool {
        if self.constraint_history.len() == 0 {
            true
        } else {
            let latest_constraint: CompressedCollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
            latest_constraint.is_empty()
        }
    }
    /// Logs the constraint's log
    pub fn printlog(&self) {
        log::trace!("{}", format!("Constraint History Len{}", self.constraint_history.len()));
        if self.constraint_history.len() == 0 {

        } else {
            let latest_constraint: CompressedCollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
            latest_constraint.printlog();
        }
        // log::info!("{}", format!("Set Size: {}", self.calculated_states.len()));
    }
    /// Gets the Latest Constraint
    pub fn latest_constraint(&self) -> CompressedCollectiveConstraint {
        if self.constraint_history.len() == 0 {
            return CompressedCollectiveConstraint::game_start()
        } else {
            self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap()
        }
    }
    /// Entrypoint for any action done, updates history accordingly
    pub fn push_ao(&mut self, ao: &ActionObservation, bool_know_priv_info: bool){
        // log::trace!("{}", format!("Before Pushing AO {:?}", ao));
        // self.printlog();
        if ao.name() == AOName::Discard {
            match ao.no_cards() {
                1 => {
                    if let Some(temp_card) = ao.cards().first(){
                        self.constraint_history.push(self.constraint_history[self.constraint_history.len() - self.prev_index()].clone());
                        if let Some(last_constraint) = self.constraint_history.last_mut().and_then(|opt| opt.as_mut()) {
                            last_constraint.death(ao.player_id(), *temp_card);
                            last_constraint.sort_unstable();
                        } else {
                            debug_assert!(false, "constraint not stored at prev_index!");
                        }
                    } else {
                        debug_assert!(false, "Card does not exist!!");
                    }
                },
                2 => {
                    let temp_cards = ao.cards();
                    self.constraint_history.push(self.constraint_history[self.constraint_history.len() - self.prev_index()].clone());
                    if let Some(last_constraint) = self.constraint_history.last_mut().and_then(|opt| opt.as_mut()) {
                        last_constraint.death(ao.player_id(), temp_cards[0]);
                        last_constraint.death(ao.player_id(), temp_cards[1]);
                        last_constraint.sort_unstable();
                    } else {
                        debug_assert!(false, "Card does not exist!!");
                    }
                },
                _ => {
                    debug_assert!(false,"Unexpected no_cards case");
                }
            }
            self.dist_from_last.push(1);
        } else if ao.name() == AOName::RevealRedraw{
            self.constraint_history.push(self.constraint_history[self.constraint_history.len() - self.prev_index()].clone());
            if let Some(last_constraint) = self.constraint_history.last_mut().and_then(|opt| opt.as_mut()) {
                last_constraint.reveal_redraw(ao.player_id(), ao.card());
                last_constraint.sort_unstable();
            } else {
                // Handle the case where the last element is None or the vector is empty
                debug_assert!(false, "constraint not stored at prev_index!");
            }
            self.dist_from_last.push(1);
        } else if ao.name() == AOName::ExchangeDraw {
            self.constraint_history.push(self.constraint_history[self.constraint_history.len() - self.prev_index()].clone());
            if let Some(last_constraint) = self.constraint_history.last_mut().and_then(|opt| opt.as_mut()) {
                // Player gets info that the pile has 2 cards, which prunes other groups
                // if ao.cards()[0] == ao.cards()[1]{
                //     last_constraint.group_initial_prune(6, &ao.cards()[0], 2, false);
                // } else {
                //     last_constraint.group_initial_prune(6, &ao.cards()[0], 1, false);
                //     last_constraint.group_initial_prune(6, &ao.cards()[1], 1, false);
                // }
                if bool_know_priv_info {
                    // Case where we want to store knowledge of cards drawn into constraints
                    // if ao.cards()[0] == ao.cards()[1] {
                    //     last_constraint.add_group_constraint_exchange(ao.player_id(), &ao.cards()[0], 2);
                    // } else {
                    //     last_constraint.add_group_constraint_exchange(ao.player_id(), &ao.cards()[0], 1);
                    //     last_constraint.add_group_constraint_exchange(ao.player_id(), &ao.cards()[1], 1);
                    // }
                    // TODO: Actually store the private knowledge
                    last_constraint.ambassador_private(ao.player_id());
                    last_constraint.sort_unstable();
                } else {
                    // Case where adding to past history and one does not know what card is drawn in exchange draw
                    last_constraint.ambassador_public(ao.player_id());
                    last_constraint.sort_unstable();
                }
            } else {
                debug_assert!(false, "constraint not stored at prev_index!");
            }
            self.dist_from_last.push(1);
        } else if ao.name() == AOName::ExchangeChoice {
            if self.dist_from_last.len() != 0 {
                self.constraint_history.push(self.constraint_history[self.constraint_history.len() - self.prev_index()].clone());
            } else {
                let game_start_constraint: CompressedCollectiveConstraint = CompressedCollectiveConstraint::game_start();
                self.constraint_history.push(Some(game_start_constraint));
            }
            self.dist_from_last.push(1);
        } else {
            // Add new case
            if self.dist_from_last.len() != 0{
                self.constraint_history.push(None);
                self.dist_from_last.push(self.dist_from_last[self.dist_from_last.len() - 1] + 1);
            } else {
                let game_start_constraint: CompressedCollectiveConstraint = CompressedCollectiveConstraint::game_start();
                self.constraint_history.push(Some(game_start_constraint));
                self.dist_from_last.push(1);
            }
        } 
    }
    /// pop latest move
    pub fn pop(&mut self) {
        self.dist_from_last.pop();
        self.constraint_history.pop();
    }
    /// Converts card to a char
    pub fn card_to_char(&self, card: Card) -> char {
        // Ambassador => A
        // Assassin => B
        // Captain => C
        // Duke => D
        // Contessa => E
        (b'A' + card as u8) as char
    }
    /// Converts char to a card
    pub fn char_to_card(&self, card_char: char) -> Card {
        // Ambassador <= A
        // Assassin <= B
        // Captain <= C
        // Duke <= D
        // Contessa <= E
        Card::try_from(card_char as u8 - b'A').unwrap()
    }
    // pub fn filter_state_simple(&mut self){
    //     let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     self.calculated_states = self.all_states.par_iter()
    //         .filter(|state| self.state_satisfies_constraints(state, &latest_constraint))
    //         .cloned()
    //         .collect();
    // }
    // pub fn filter_state_simple_test(&mut self, constraint: &CollectiveConstraint){
    //     let latest_constraint: CollectiveConstraint = constraint.clone();
    //     self.calculated_states = self.all_states.par_iter()
    //         .filter(|state| self.state_satisfies_constraints(state, &latest_constraint))
    //         .cloned()
    //         .collect();
    // }
    
    // // Helper method to determine if a state satisfies all constraints
    // // TODO: Change back to private
    // pub fn state_satisfies_constraints(&self, state: &str, latest_constraint: &CollectiveConstraint) -> bool {
    //     // println!("Check");
    //     // Check jc_hm constraints
    //     for i in 0..6 {
    //         if let Some(cards) = latest_constraint.get_jc_hm().get(&i) {
    //             let card_char_vec: Vec<char> = cards.iter().map(|c| self.card_to_char(c)).collect();
    //             let index_start: usize = 2 * i;
    //             let index_end: usize;

    //             index_end = index_start + 2;
 
    
    //             if state.len() < index_end {
    //                 return false;
    //             }
    
    //             let state_chars: Vec<char> = state[index_start..index_end].chars().collect();
    //             if state_chars != card_char_vec {
    //                 return false; // The state does not satisfy this jc_hm constraint
    //             }
    //         }
    //     }
    
    //     // Check pc_hm constraints
    //     for i in 0..6 {
    //         if let Some(card) = latest_constraint.get_pc_hm().get(&i) {
    //             let card_char: char = self.card_to_char(card);
    //             let index_start: usize = 2 * i;
    //             let index_end: usize;

    //             index_end = index_start + 2;

    
    //             if state.len() < index_end || !state[index_start..index_end].contains(card_char) {
    //                 return false; // The state does not satisfy this pc_hm constraint
    //             }
    //         }
    //     }
    
    //     // This should check that there are gc_hm_count of the card.
    //     // Check gc_hm constraints
    //     // for card in &self.card_list {
    //     //     if let Some(participation_list) = latest_constraint.gc_hm.get(&card) {
    //     //         let card_char: char = self.card_to_char(&card);
    
    //     //         let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //     //             .filter_map(|(player_id, &participation)| {
    //     //                 if participation == 1 {
    //     //                     Some((self.index_start_arr[player_id], self.index_end_arr[player_id]))
    //     //                 } else {
    //     //                     None
    //     //                 }
    //     //             }).collect();
    
    //     //         let satisfies_gc_hm = participating_indices.iter().any(|&(start, end)| {
    //     //             state.len() >= end && state[start..end].contains(card_char)
    //     //         });
    
    //     //         if !satisfies_gc_hm {
    //     //             return false; // The state does not satisfy this gc_hm constraint
    //     //         }
    //     //     }
    //     // }
        
    //     // Check gc_vec constraints
    //     let mut index: usize = 0;
    //     // println!("Before While");
    //     while index < latest_constraint.get_gc_vec().len(){
    //         let participation_list: &[u8; 7] = latest_constraint.get_gc_vec()[index].get_list();
    //         let card_char: char = latest_constraint.get_gc_vec()[index].card().card_to_char();

    //         let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //                 .filter_map(|(player_id, &participation)| {
    //                     if participation == 1 {
    //                         Some((self.index_start_arr[player_id], self.index_end_arr[player_id]))
    //                     } else {
    //                         None
    //                     }
    //                 }).collect();
    //         let mut total_count = 0;
    //         let required_count = latest_constraint.get_gc_vec()[index].count();
    //         let mut satisfies_gc_vec: bool = false;
    //         // println!("Required Count: {}", required_count);
    //         // println!("Participation List: {:?}", participation_list);
    //         // println!("Participation Indices: {:?}", participating_indices);
    //         for &(start, end) in participating_indices.iter() {
    //             // println!("Start: {}", start);
    //             // println!("End: {}", end);
    //             // println!("State len: {}", state.len());
    //             if state.len() >= end {
    //                 total_count += state[start..end].matches(card_char).count();
    //                 // println!("Total Count: {}", total_count);
    //                 if total_count >= required_count {
    //                     satisfies_gc_vec = true;
    //                     break;
    //                 }
    //             }
    //         }
    //         if !satisfies_gc_vec {
    //             return false; // The state does not satisfy this gc_vec constraint
    //         }
    //         index += 1;
    //     }

    //     true // The state satisfies all constraints
    // }

    // pub fn player_can_have_card(&self, player_id: usize, card: &Card) -> bool {
    //     // This is the ideal set theory version
    //     // ~20µs
    //     // never more than 50µs
    //     let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     latest_constraint.player_can_have_active_card(player_id, card)
    // }
    // pub fn player_can_have_card_constructor(&mut self, player_id: usize, card: &Card) -> bool {
    //     // This is the ideal constructed version
    //     // > 20µs up to 500µs
    //     let mut latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     latest_constraint.add_raw_public_constraint(player_id, *card);
    //     if self.naive_sampler.par_constructor(&latest_constraint).is_none(){
    //         return false
    //     } else {
    //         return true
    //     }
    // }
    // pub fn player_can_have_cards(&self, player_id: usize, cards: &[Card; 2]) -> bool {
    //     // This is the ideal set theory version
    //     // Does not work for player_id == 6
    //     // MEDIAN: 22µs
    //     // MEAN: 22µs
    //     // MAX: 65.8µs (600 cases)
    //     let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     latest_constraint.player_can_have_active_cards(player_id, cards)
    // }
    pub fn debug_panicker(&self) {
        self.latest_constraint().debug_panicker();
    }
}
