// Journey here
// Tried to iteratively find naive probability by filtering
// Concurrent and normal iteration times are around 0.1 s calculation of belief is around 0.1 seconds
// This is too long
// Tried instead to save into hashmap and store in bson

use crate::history_public::{Card, AOName, ActionObservation};
use super::permutation_generator::{gen_table_combinations, gen_bag_combinations};
use super::coup_const::{BAG_SIZES, TOKENS, MAX_PERM_STATES};
use super::constraint::{GroupConstraint, CollectiveConstraint};
use super::naive_sampler::{self, NaiveSampler};
use std::collections::{HashMap, HashSet};
// use core::hash::Hasher;
use std::hash::{Hash, Hasher};
use std::usize;
use rand::Rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use serde::{Serialize, Deserialize};
use serde_json;
use super::loader::{load_initial_hashmap, save_bson_hashmap};
use rand::prelude::SliceRandom;
use std::sync::Mutex;
use core::sync::atomic::AtomicBool;

pub struct NaiveProb<'a> {
    // a vec of constraints to push and pop
    // dead cards to push or pop
    // Will not locally store game history, jsut the constraint history
    constraint_history: Vec<Option<CollectiveConstraint>>,
    all_states: Vec<String>,
    // distfromlast tells u how far away to fetch the last constraint_history
    dist_from_last: Vec<usize>,
    calculated_states: Vec<String>, // All the states that fulfil current constraints
    index_start_arr: [usize; 7],
    index_end_arr: [usize; 7],
    card_list: [Card; 5],
    set_store: Vec<HashMap<String, HashSet<String>>>,
    belief_hm: HashMap<String, Vec<f64>>,
    unique_2p_hands: Vec<String>,
    unique_3p_hands: Vec<String>,
    naive_sampler: NaiveSampler<'a>,
}
impl<'a> NaiveProb<'a> {
    pub fn new() -> Self {
        let unique_2p_hands: Vec<String> = gen_bag_combinations(TOKENS, &2);
        let unique_3p_hands: Vec<String> = gen_bag_combinations(TOKENS, &3);
        let mut all_states: Vec<String> = gen_table_combinations(TOKENS, &BAG_SIZES);
        let mut rng = rand::thread_rng();
        all_states.shuffle(&mut rng); // Shuffle in place
        let naive_sampler: NaiveSampler = NaiveSampler::new();
        NaiveProb{
            constraint_history: Vec::with_capacity(1500),
            all_states,
            dist_from_last:Vec::with_capacity(1500),
            calculated_states: Vec::with_capacity(MAX_PERM_STATES),
            index_start_arr: [0, 2, 4, 6, 8, 10, 12],
            index_end_arr: [2, 4, 6, 8, 10, 12, 15],
            card_list: [Card::Ambassador, Card::Assassin, Card::Captain, Card::Duke, Card::Contessa],
            set_store: vec![HashMap::new(); 6],
            belief_hm: HashMap::new(),
            unique_2p_hands,
            unique_3p_hands,
            naive_sampler,
        }
    }
    pub fn reset(&mut self) {
        self.constraint_history = Vec::with_capacity(1500);
        self.dist_from_last = Vec::with_capacity(1500);
        self.calculated_states = Vec::with_capacity(MAX_PERM_STATES)
    }
    pub fn set_generation(&mut self) {
        let mut pc_hm: HashMap<usize, char> = HashMap::new();
        let mut set_store: Vec<HashMap<String, HashSet<String>>> = vec![HashMap::new(); 6];
        for i in 0..6 {
            pc_hm.insert(i, 'A');
            pc_hm.insert(i, 'B');
            pc_hm.insert(i, 'C');
            pc_hm.insert(i, 'D');
            pc_hm.insert(i, 'E');
        }
        let char_list: [char; 5] = ['A', 'B', 'C', 'D', 'E'];
        let mut index_start: usize;
        let mut index_end: usize;
        // Initialising pc sets
        for i in 0..6 {
            index_start = self.index_start_arr[i];
            index_end = self.index_end_arr[i];
            for card_char in char_list.iter(){
                let filtered: Vec<String> = self.all_states.par_iter()
                .filter(|state| state[index_start..index_end].contains(*card_char))
                .cloned().collect();
            let hash_set: HashSet<String> = filtered.into_iter().collect();
            set_store[i].insert(card_char.to_string(), hash_set);
        }
        let str_list: [&str; 15] = ["AA", "AB", "AC", "AD", "AE", "BB", "BC", "BD", "BE", "CC", "CD", "CE", "DD", "DE", "EE"];

        for i in 0..6 {
            index_start = self.index_start_arr[i];
            index_end = self.index_end_arr[i];
            for s in str_list.iter(){
                let card_char_vec: Vec<char> = s.chars().collect();
                let filtered: Vec<String> = self.all_states.par_iter()
                .filter(|state| {
                    let state_chars: Vec<char> = state[index_start..index_end].chars().collect();
                    state.len() >= index_end && state_chars == *card_char_vec
                })
                .cloned().collect();
                let hash_set: HashSet<String> = filtered.into_iter().collect();
                set_store[i].insert(s.to_string(), hash_set);
            }
        }
    }
    // Initialising jc sets

    }
    pub fn prev_index(&self) -> usize {
        self.dist_from_last[self.dist_from_last.len() - 1]
    }
    // pub fn sort_and_serialize_hashmap<K, V>(&self, hashmap: &HashMap<K, V>) -> String
    // where
    //     K: Serialize + Ord + std::hash::Hash,
    //     V: Serialize,
    // {   // Serializes Hashmap
    //     let mut sorted_map: HashMap<&K, &V> = hashmap.iter().collect();
    //     let mut sorted_pairs: Vec<(&K, &V)> = sorted_map.drain().collect();
    //     sorted_pairs.sort_by_key(|&(k, _)| k);
    //     serde_json::to_string(&sorted_pairs).unwrap()
    // }
    // pub fn make_key_belief(&self) -> String {
    //     // Makes key to store the belief probabilities based on constraints
    //     let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     let public_constraint_key: String = self.sort_and_serialize_hashmap(&latest_constraint.pc_hm);
    //     let joint_constraint_key: String = self.sort_and_serialize_hashmap(&latest_constraint.jc_hm);
    //     let group_constraint_key: String = self.sort_and_serialize_hashmap(&latest_constraint.gc_hm);
    //     let big_key: String = format!("{}_{}_{}", public_constraint_key, joint_constraint_key, group_constraint_key);
    //     big_key
    // }
    // pub fn key_in_bson_hashmap(&mut self, key: String) -> bool {
    //     if self.belief_hm.contains_key(&key){
    //         true
    //     } else {
    //         false
    //     }
    // }
    // pub fn add_to_hashmap(&mut self, key: String, belief_vec: Vec<f64>){
    //     self.belief_hm.entry(key).or_insert(belief_vec);
    // }
    // pub fn load_bson_hashmap(&mut self){
    //     self.belief_hm = load_initial_hashmap("naive_belief_prob_hashmap.bson");
    //     log::trace!("{}", format!("Loaded up with len: {:?}", self.belief_hm.len()));
    // }
    // pub fn save_bson_hashmap(&mut self){
    //     match save_bson_hashmap(&self.belief_hm, "naive_belief_prob_hashmap.bson"){
    //         Ok(_) => log::info!("Saved HashMap of len: {}", self.belief_hm.len()),
    //         Err(_) => log::info!("Failed to save HashMap"),
    //     }
    // }
    pub fn latest_constraint_is_empty(&self) -> bool {
        if self.constraint_history.len() == 0 {
            true
        } else {
            let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
            latest_constraint.is_empty()
        }
    }
    pub fn print_belief_hm_len(&self){
        println!("HashMap now of len: {}", self.belief_hm.len());
    }
    pub fn printlog(&self) {
        log::trace!("{}", format!("Constraint History Len{}", self.constraint_history.len()));
        if self.constraint_history.len() == 0 {

        } else {
            let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
            latest_constraint.printlog();
        }
        // log::info!("{}", format!("Set Size: {}", self.calculated_states.len()));
    }
    pub fn calc_state_len(&self) -> usize {
        self.calculated_states.len()
    }
    pub fn log_calc_state_len(&self){
        log::trace!("{}", format!("Calculated_State Length: {}", self.calculated_states.len()));
    }
    pub fn log_calc_state(&self){
        log::info!("{}", format!("Calculated_State: {:?}", self.calculated_states));
    }
    // Push()
    pub fn latest_constraint(&self) -> CollectiveConstraint {
        if self.constraint_history.len() == 0 {
            return CollectiveConstraint::new()
        } else {
            self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap()
        }
    }
    pub fn push_ao(&mut self, ao: &ActionObservation, bool_know_priv_info: bool){
        // log::trace!("{}", format!("Before Pushing AO {:?}", ao));
        // self.printlog();
        if ao.name() == AOName::Discard {
            match ao.no_cards() {
                1 => {
                    if let Some(temp_card) = ao.cards().first(){
                        self.constraint_history.push(self.constraint_history[self.constraint_history.len() - self.prev_index()].clone());
                        if let Some(last_constraint) = self.constraint_history.last_mut().and_then(|opt| opt.as_mut()) {
                            last_constraint.group_initial_prune(ao.player_id(), temp_card, 1, true);
                            last_constraint.add_public_constraint(ao.player_id(), *temp_card);
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
                        if temp_cards[0] == temp_cards[1]{
                            last_constraint.group_initial_prune(ao.player_id(), &temp_cards[0], 2, true);
                        } else {
                            last_constraint.group_initial_prune(ao.player_id(), &temp_cards[0], 1, true);
                            last_constraint.group_initial_prune(ao.player_id(), &temp_cards[1], 1, true);
                        }
                        last_constraint.add_joint_constraint(ao.player_id(), &temp_cards.to_vec());
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
                // We update when cards a mixed with pile by player_id but no card is revealed
                // last_constraint.update_group_constraint_hm(ao.player_id());
                // last_constraint.add_group_constraint_hm(ao.player_id(), ao.card());
                last_constraint.group_initial_prune(ao.player_id(), &ao.card(), 1, false);
                // last_constraint.update_group_constraint(ao.player_id());
                last_constraint.add_group_constraint(ao.player_id(), &ao.card(), 1);

                // last_constraint.update_group_constraint(ao.player_id());
                // We add because a card is revealed before mixing | We add after updating for minor efficiency
                // TODEBUG: Should only add if not already inside!
                // last_constraint.add_group_constraint(ao.player_id(), ao.card());
            } else {
                // Handle the case where the last element is None or the vector is empty
                debug_assert!(false, "constraint not stored at prev_index!");
            }
            self.dist_from_last.push(1);
        } else if ao.name() == AOName::ExchangeDraw {
            self.constraint_history.push(self.constraint_history[self.constraint_history.len() - self.prev_index()].clone());
            if let Some(last_constraint) = self.constraint_history.last_mut().and_then(|opt| opt.as_mut()) {
                // Player gets info that the pile has 2 cards, which prunes other groups
                if ao.cards()[0] == ao.cards()[1]{
                    last_constraint.group_initial_prune(6, &ao.cards()[0], 2, false);
                } else {
                    last_constraint.group_initial_prune(6, &ao.cards()[0], 1, false);
                    last_constraint.group_initial_prune(6, &ao.cards()[1], 1, false);
                }
                if bool_know_priv_info {
                    // Case where we want to store knowledge of cards drawn into constraints
                    if ao.cards()[0] == ao.cards()[1] {
                        last_constraint.add_group_constraint_exchange(ao.player_id(), &ao.cards()[0], 2);
                    } else {
                        last_constraint.add_group_constraint_exchange(ao.player_id(), &ao.cards()[0], 1);
                        last_constraint.add_group_constraint_exchange(ao.player_id(), &ao.cards()[1], 1);
                    }
                } else {
                    // Case where adding to past history and one does not know what card is drawn in exchange draw
                    last_constraint.update_group_constraint(ao.player_id());
                }
            } else {
                debug_assert!(false, "constraint not stored at prev_index!");
            }
            self.dist_from_last.push(1);
        } else if ao.name() == AOName::ExchangeChoice {
            if self.dist_from_last.len() != 0 {
                self.constraint_history.push(self.constraint_history[self.constraint_history.len() - self.prev_index()].clone());
            } else {
                let empty_collective_constraint: CollectiveConstraint = CollectiveConstraint::new();
                self.constraint_history.push(Some(empty_collective_constraint));
            }
            self.dist_from_last.push(1);
        } else {
            // Add new case
            if self.dist_from_last.len() != 0{
                self.constraint_history.push(None);
                self.dist_from_last.push(self.dist_from_last[self.dist_from_last.len() - 1] + 1);
            } else {
                let empty_collective_constraint: CollectiveConstraint = CollectiveConstraint::new();
                self.constraint_history.push(Some(empty_collective_constraint));
                self.dist_from_last.push(1);
            }
        } 
    }
    pub fn pop(&mut self) {
        self.dist_from_last.pop();
        self.constraint_history.pop();
    }
    //TODO: Add union constraints
    //TODO: Calculate Beliefs
    // pub fn filter_state(&mut self) {
    //     // Test for 
    //     // RELEASE MODE TIME           : 0.10~0.14 s for just the public constraints both single and joint
    //     // RELEASE MODE TIME concurrent: 0.13~0.23 s for just the public constraints both single and joint
    //     // RELEASE MODE TIME           : 0.15~0.18 s with single, joint and group
    //     // RELEASE MODE TIME concurrent: 0.10~0.11 s for just the public constraints both single and joint
    //     // if let Some(latest_constraint) = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone();
    //     let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     self.calculated_states = self.all_states.clone();
    //     let mut start_time = Instant::now();
    //     for i in 0..6{
    //         if let Some(cards) = latest_constraint.jc_hm.get(&i){
    //             // cards is a Vec<Card> of size 2
    //             // Convert to chars like "AB"
    //             // check that at the current_states state[index_start..index_end] == "AB"
    //             let card_char_vec: Vec<char> = cards.iter().map(|c| self.card_to_char(c)).collect();
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             self.calculated_states.retain(
    //                 |state| {
    //                     let state_chars: Vec<char> = state[index_start..index_end].chars().collect();
    //                     state.len() >= index_end && state_chars == *card_char_vec
    //                 }
    //             );
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for JC Filter: {:?}", elapsed_time);
    //     start_time = Instant::now();
    //     for i in 0..6 {
    //         if let Some(card) = latest_constraint.pc_hm.get(&i){
    //             let card_char: char = self.card_to_char(&card);
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             // filter the string where [index_start: index_end] contains card_char
    //             self.calculated_states.retain(|state| state[index_start..index_end].contains(card_char));
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for PC Filter: {:?}", elapsed_time);
    //     //TODO: move these items to self
    //     let index_start_arr: [usize; 7] = [0, 2, 4, 6, 8, 10, 12];
    //     let index_end_arr: [usize; 7] = [2, 4, 6, 8, 10, 12, 15];
    //     let card_list: [Card; 5] = [Card::Ambassador, Card::Assassin, Card::Captain, Card::Duke, Card::Contessa];
    //     for card in card_list {
    //         if let Some(participation_list) = latest_constraint.gc_hm.get(&card){
    //             let card_char: char = self.card_to_char(&card);

    //             let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //             .filter_map(|(player_id, &participation)| {
    //                 if participation == 1{
    //                     Some((index_start_arr[player_id], index_end_arr[player_id]))
    //                 } else {
    //                     None
    //                 }
    //             }).collect();
    //             self.calculated_states.retain(|state| {
    //                 participating_indices.iter().any(|&(start, end)|{
    //                     state.len() >= end && state[start..end].contains(card_char)
    //                 })
    //             });
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for GC Filter: {:?}", elapsed_time);
    //     // for group_constraint in latest_constraint.gc_vec.iter(){
    //     //     // group_constraint.participation_list: [u8; 7]
    //     //     let participation_list: &[u8] = group_constraint.get_list();
    //     //     let card: &Card = group_constraint.card();
    //     //     let card_char: char = self.card_to_char(&card);

    //     //     let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //     //         .filter_map(|(player_id, &participation)| {
    //     //             if participation == 1{
    //     //                 Some((index_start_arr[player_id], index_end_arr[player_id]))
    //     //             } else {
    //     //                 None
    //     //             }
    //     //         }).collect();
    //     //     self.calculated_states.retain(|state| {
    //     //         participating_indices.iter().any(|&(start, end)|{
    //     //             state.len() >= end && state[start..end].contains(card_char)
    //     //         })
    //     //     });
    //     // }
    // }
    // pub fn filter_state_concurrent(&mut self) {
    //     let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     self.calculated_states = self.all_states.clone();

    //     // USE NORMAL VERSION FOR THIS, NON CONCURRENT ALWAYS WINS IN TIME WHEN THERE IS NO JOINT
    //     let mut start_time = Instant::now();
    //     for i in 0..6{
    //         if let Some(cards) = latest_constraint.jc_hm.get(&i){
    //             // cards is a Vec<Card> of size 2
    //             // Convert to chars like "AB"
    //             // check that at the current_states state[index_start..index_end] == "AB"
    //             let card_char_vec: Vec<char> = cards.iter().map(|c| self.card_to_char(c)).collect();
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             let filtered: Vec<String> = self.calculated_states.par_iter()
    //                 .filter(|state| {
    //                     let state_chars: Vec<char> = state[index_start..index_end].chars().collect();
    //                     state.len() >= index_end && state_chars == *card_char_vec
    //                 })
    //                 .cloned().collect();
    //             self.calculated_states = filtered;
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Concurrent JC Filter: {:?}", elapsed_time);
    //     start_time = Instant::now();
    //     for i in 0..6 {
    //         if let Some(card) = latest_constraint.pc_hm.get(&i){
    //             let card_char: char = self.card_to_char(&card);
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             // filter the string where [index_start: index_end] contains card_char
    //             let filtered: Vec<String> = self.calculated_states.par_iter()
    //                 .filter(|state| state[index_start..index_end].contains(card_char))
    //                 .cloned().collect();
    //             self.calculated_states = filtered;
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Concurrent PC Filter: {:?}", elapsed_time);
    //     // Add concurrent version
    //     let index_start_arr: [usize; 7] = [0, 2, 4, 6, 8, 10, 12];
    //     let index_end_arr: [usize; 7] = [2, 4, 6, 8, 10, 12, 15];

    //     let index_start_arr: [usize; 7] = [0, 2, 4, 6, 8, 10, 12];
    //     let index_end_arr: [usize; 7] = [2, 4, 6, 8, 10, 12, 15];
    //     let card_list: [Card; 5] = [Card::Ambassador, Card::Assassin, Card::Captain, Card::Duke, Card::Contessa];
    //     start_time = Instant::now();
    //     for card in card_list {
    //         if let Some(participation_list) = latest_constraint.gc_hm.get(&card){
    //             let card_char: char = self.card_to_char(&card);

    //             let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //             .filter_map(|(player_id, &participation)| {
    //                 if participation == 1{
    //                     Some((index_start_arr[player_id], index_end_arr[player_id]))
    //                 } else {
    //                     None
    //                 }
    //             }).collect();
    //             let filtered: Vec<String> = self.calculated_states.par_iter()
    //                 .filter(|state| {
    //                     participating_indices.iter().any(|&(start, end)| {
    //                         state.len() >= end && state[start..end].contains(card_char)
    //                     })
    //                 })
    //                 .cloned().collect();

    //             // Update calculated_states with filtered results
    //             self.calculated_states = filtered;
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Concurrent GC Filter: {:?}", elapsed_time);
    //     // Assuming participation_list and card_char are prepared as before
    //     // for group_constraint in latest_constraint.gc_vec.iter() {
    //     //     // Assuming get_list() and card() methods, similar preparation as in sequential version
    //     //     let participation_list = group_constraint.get_list(); // Example method call, adjust as needed
    //     //     let card_char = self.card_to_char(&group_constraint.card());

    //     //     let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //     //         .filter_map(|(player_id, &participation)| {
    //     //             if participation == 1 {
    //     //                 Some((index_start_arr[player_id], index_end_arr[player_id]))
    //     //             } else {
    //     //                 None
    //     //             }
    //     //         })
    //     //         .collect();

    //     //     // Concurrently filter states based on group constraints
    //     //     let filtered: Vec<String> = self.calculated_states.par_iter()
    //     //         .filter(|state| {
    //     //             participating_indices.iter().any(|&(start, end)| {
    //     //                 state.len() >= end && state[start..end].contains(card_char)
    //     //             })
    //     //         })
    //     //         .cloned().collect();

    //     //     // Update calculated_states with filtered results
    //     //     self.calculated_states = filtered;
    //     // }
    // }
    // pub fn filter_state_set(&mut self) {
    //     let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     let mut state_set: HashSet<String> = HashSet::new();
    //     let mut temp_set: HashSet<String> = HashSet::new();
    //     let mut bool_first: bool = true;
    //     let mut key: String;
    //     // JC Filter
    //     let start_time = Instant::now();
    //     for i in 0..6 {
    //         if let Some(cards) = latest_constraint.jc_hm.get(&i) {
    //             key = cards.iter().map(|c| self.card_to_char(c)).collect::<String>();
    //             println!("key, {}", key);
    //             if bool_first {
    //                 state_set = self.set_store[i][&key].clone();
    //                 bool_first = false;
    //             } else {
    //                 temp_set = self.set_store[i][&key].clone();
    //                 state_set = state_set.intersection(&temp_set).cloned().collect::<HashSet<_>>();
    //             }
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Set JC Filter: {:?}", elapsed_time);
    //     // PC Filter
    //     let start_time = Instant::now();
    //     for i in 0..6 {
    //         if let Some(card) = latest_constraint.pc_hm.get(&i){
    //             key = self.card_to_char(&card).to_string();
    //             if bool_first {
    //                 state_set = self.set_store[i][&key].clone();
    //                 bool_first = false;
    //             } else {
    //                 temp_set = self.set_store[i][&key].clone();
    //                 state_set = state_set.intersection(&temp_set).cloned().collect::<HashSet<_>>();
    //             }
    //         }
    //     }
    //     // Do a check to convert to vec at the end! store in self.calculated_states
    //     // Before GC
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Set CC Filter: {:?}", elapsed_time);
        
    //     self.calculated_states = state_set.into_iter().collect();
    // }
    pub fn card_to_char(&self, card: &Card) -> char {
        // Example implementation, adjust according to your Card enum and logic
        match card {
            Card::Ambassador => 'A',
            Card::Assassin => 'B',
            Card::Captain => 'C',
            Card::Duke => 'D',
            Card::Contessa => 'E',
        }
    }
    pub fn char_to_card(&self, card_char: char) -> Card {
        match card_char {
            'A' => Card::Ambassador,
            'B' => Card::Assassin,
            'C' => Card::Captain,
            'D' => Card::Duke,
            'E' => Card::Contessa,
            _ => panic!("Bad char used!"),
        }
    }
    // pub fn get_latest_beliefs(&mut self) -> Vec<f64>{
    //     // This is microsecond for small calculated states
    //     // Time 0.17 ~ 0.20 s for full state
    //     // I wish to iterate through Vec<String>
    //     // For each
    //     //TODO: Also store total times each move has been played 
    //      // Can make it output the beliefs
    //     // your trainer takes history and naive_prob, uses PMCCFR to search
    //      // When it reaches a node it needs to collect their states to modify a beliefstate

    //     let mut hand_to_index_offset_map: HashMap<String, usize> = HashMap::new();
    //     let mut count: usize = 0;
    //     let mut max_count: usize = 0;
    //     for hand in &self.unique_2p_hands {
    //         hand_to_index_offset_map.insert(hand.clone(), count);
    //         count += 1;
    //     }

    //     max_count = 6 * count;
    //     count = 0;
    //     // Doing the same but for player 7, the pile which has 3 cards
    //     for hand in &self.unique_3p_hands {
    //         hand_to_index_offset_map.insert(hand.clone(), count);
    //         count += 1;
    //     }
    //     // 35 combinations
    //     // How do you speed the bottom up
    //     max_count += count;
    //     let mut card_freq: Vec<f64> = vec![0.0; max_count];
    //     let mut total_sum: u64 = 0;
    //     for state in &self.calculated_states {
    //         for player_index in 0..7 {
    //             let start_index = self.index_start_arr[player_index];
    //             let end_index = self.index_end_arr[player_index];
    //             let player_hand = &state[start_index..end_index];
                
    //             let card_index = match hand_to_index_offset_map.get(player_hand) {
    //                 Some(index) => *index,
    //                 None => panic!("Invalid card combination"),
    //             };
    //             card_freq[player_index * 15 + card_index] += 1.0; 
    //         }
    //         total_sum += 1;
    //     }
    //     card_freq.iter_mut().for_each(|f| *f /= total_sum as f64);
    //     // Up till here
    //     let key: String = self.make_key_belief();
    //     self.add_to_hashmap(key, card_freq.clone());
    //     card_freq
    // }
    // pub fn get_latest_beliefs_concurrent(&mut self) -> Vec<f64>{
    //     //TIME Significantly reduced for large state sets
    //     // E.g 1.5million full set from 173 ms => 51ms
    //     // Using this and filter_optimal, longest time all in is around 150ms tops down from 300-400ms
    //     // OK what if we dont use filter_optimal first, but we just directly iter through the whole thing from here
    //     // I wish to iterate through Vec<String>
    //     // For each
    //     //TODO: Also store total times each move has been played 
    //      // Can make it output the beliefs
    //     // your trainer takes history and naive_prob, uses PMCCFR to search
    //      // When it reaches a node it needs to collect their states to modify a beliefstate

    //     // Move hand_to_index_map out
    //     let mut hand_to_index_offset_map: HashMap<String, usize> = HashMap::new();
    //     let mut count: usize = 0;
    //     let mut max_count: usize = 0;
    //     for hand in &self.unique_2p_hands {
    //         hand_to_index_offset_map.insert(hand.clone(), count);
    //         count += 1;
    //     }
    //     max_count = 6 * count;
    //     count = 0;
    //     // Doing the same but for player 7, the pile which has 3 cards
    //     for hand in &self.unique_3p_hands {
    //         hand_to_index_offset_map.insert(hand.clone(), count);
    //         count += 1;
    //     }
    //     max_count += count;
    //     let card_freq: Arc<Vec<AtomicUsize>> = Arc::new((0..max_count).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>());
    //     let total_sum = Arc::new(AtomicUsize::new(0));
    //     let total_sum = self.calculated_states.len() as u64;

    //     self.calculated_states.par_iter().for_each(|state| {
    //         let mut local_counts = vec![0usize; max_count];
    
    //         for player_index in 0..7 {
    //             let start_index = self.index_start_arr[player_index];
    //             let end_index = self.index_end_arr[player_index];
    //             let player_hand = &state[start_index..end_index];
    
    //             if let Some(&card_index) = hand_to_index_offset_map.get(player_hand) {
    //                 local_counts[player_index * 15 + card_index] += 1;
    //             } else {
    //                 panic!("Invalid card combination");
    //             }
    //         }
    
    //         for (card_index, &count) in local_counts.iter().enumerate() {
    //             if count > 0 {
    //                 card_freq[card_index].fetch_add(count, Ordering::SeqCst);
    //             }
    //         }
    //     });
    
    //     let beliefs: Vec<f64> = card_freq.iter()
    //         .map(|freq| freq.load(Ordering::SeqCst) as f64 / total_sum as f64)
    //         .collect();

    //     beliefs
    // }
    pub fn filter_state_simple(&mut self){
        let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        self.calculated_states = self.all_states.par_iter()
            .filter(|state| self.state_satisfies_constraints(state, &latest_constraint))
            .cloned()
            .collect();
    }
    pub fn filter_state_simple_test(&mut self, constraint: &CollectiveConstraint){
        let latest_constraint: CollectiveConstraint = constraint.clone();
        self.calculated_states = self.all_states.par_iter()
            .filter(|state| self.state_satisfies_constraints(state, &latest_constraint))
            .cloned()
            .collect();
    }
    // pub fn filter_state_optimal(&mut self){
    //     let mut start_time = Instant::now();
    //     let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Getting latest_constraint: {:?}", elapsed_time);
    //     start_time = Instant::now();
    //     self.calculated_states = self.all_states.clone();
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Cloning all states: {:?}", elapsed_time);


    //     // let mut start_time = Instant::now();
    //     for i in 0..6{
    //         if let Some(cards) = latest_constraint.jc_hm.get(&i){
    //             // cards is a Vec<Card> of size 2
    //             // Convert to chars like "AB"
    //             // check that at the current_states state[index_start..index_end] == "AB"
    //             let card_char_vec: Vec<char> = cards.iter().map(|c| self.card_to_char(c)).collect();
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             let filtered: Vec<String> = self.calculated_states.par_iter()
    //                 .filter(|state| {
    //                     let state_chars: Vec<char> = state[index_start..index_end].chars().collect();
    //                     state.len() >= index_end && state_chars == *card_char_vec
    //                 })
    //                 .cloned().collect();
    //             self.calculated_states = filtered;
    //         }
    //     }
    //     // let elapsed_time = start_time.elapsed();
    //     // log::info!("Time taken for Optimal JC Filter: {:?}", elapsed_time);
    //     // start_time = Instant::now();
    //     for i in 0..6 {
    //         if let Some(card) = latest_constraint.pc_hm.get(&i){
    //             let card_char: char = self.card_to_char(&card);
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             // filter the string where [index_start: index_end] contains card_char
    //             self.calculated_states.retain(|state| state[index_start..index_end].contains(card_char));
    //         }
    //     }
    //     // let elapsed_time = start_time.elapsed();
    //     // log::info!("Time taken for Optimal PC Filter: {:?}", elapsed_time);
    //     // Add concurrent version
    //     // start_time = Instant::now();

    //     for card in self.card_list {
    //         if let Some(participation_list) = latest_constraint.gc_hm.get(&card){
    //             let card_char: char = self.card_to_char(&card);

    //             let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //             .filter_map(|(player_id, &participation)| {
    //                 if participation == 1{
    //                     Some((self.index_start_arr[player_id], self.index_end_arr[player_id]))
    //                 } else {
    //                     None
    //                 }
    //             }).collect();
    //             let filtered: Vec<String> = self.calculated_states.par_iter()
    //                 .filter(|state| {
    //                     participating_indices.iter().any(|&(start, end)| {
    //                         state.len() >= end && state[start..end].contains(card_char)
    //                     })
    //                 })
    //                 .cloned().collect();

    //             // Update calculated_states with filtered results
    //             self.calculated_states = filtered;
    //         }
    //     }
    //     // let elapsed_time = start_time.elapsed();
    //     // log::info!("Time taken for Optimal GC Filter: {:?}", elapsed_time);
    // }
    // pub fn filter_state_optimal2(&mut self){
    //     // This is usually worse than filter_state_optimal
    //     let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();

    //     self.calculated_states = self.all_states.clone();

    //     let mut first_filter: bool = true;

    //     // let mut start_time = Instant::now();
    //     for i in 0..6{
    //         if let Some(cards) = latest_constraint.jc_hm.get(&i){
    //             // cards is a Vec<Card> of size 2
    //             // Convert to chars like "AB"
    //             // check that at the current_states state[index_start..index_end] == "AB"
    //             let card_char_vec: Vec<char> = cards.iter().map(|c| self.card_to_char(c)).collect();
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             let states_source = if first_filter {
    //                 &self.all_states
    //             } else {
    //                 &self.calculated_states
    //             };
    //             let filtered: Vec<String> = states_source.par_iter()
    //             .filter(|state| {
    //                 let state_chars: Vec<char> = state[index_start..index_end].chars().collect();
    //                 state.len() >= index_end && state_chars == *card_char_vec
    //             })
    //             .cloned().collect();
    //             self.calculated_states = filtered;
    //             first_filter = false;
    //         }
    //     }
    //     // let elapsed_time = start_time.elapsed();
    //     // log::info!("Time taken for Optimal JC Filter: {:?}", elapsed_time);
    //     // start_time = Instant::now();
    //     for i in 0..6 {
    //         if let Some(card) = latest_constraint.pc_hm.get(&i){
    //             let card_char: char = self.card_to_char(&card);
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             // filter the string where [index_start: index_end] contains card_char
    //             if first_filter {
    //                 self.calculated_states = self.all_states.par_iter()
    //                 .filter(|state| state[index_start..index_end].contains(card_char))
    //                 .cloned().collect();
    //                 first_filter = false;
    //             } else {

    //                 self.calculated_states.retain(|state| state[index_start..index_end].contains(card_char));
    //             }
    //         }
    //     }
    //     // let elapsed_time = start_time.elapsed();
    //     // log::info!("Time taken for Optimal PC Filter: {:?}", elapsed_time);
    //     // Add concurrent version
    //     // start_time = Instant::now();
    //     for card in self.card_list {
    //         if let Some(participation_list) = latest_constraint.gc_hm.get(&card){
    //             let card_char: char = self.card_to_char(&card);

    //             let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //             .filter_map(|(player_id, &participation)| {
    //                 if participation == 1{
    //                     Some((self.index_start_arr[player_id], self.index_end_arr[player_id]))
    //                 } else {
    //                     None
    //                 }
    //             }).collect();
    //             if first_filter {

    //                 self.calculated_states = self.all_states.par_iter()
    //                     .filter(|state| {
    //                         participating_indices.iter().any(|&(start, end)| {
    //                             state.len() >= end && state[start..end].contains(card_char)
    //                         })
    //                     })
    //                     .cloned().collect();
    //             } else {

    //                 self.calculated_states = self.calculated_states.par_iter()
    //                     .filter(|state| {
    //                         participating_indices.iter().any(|&(start, end)| {
    //                             state.len() >= end && state[start..end].contains(card_char)
    //                         })
    //                     })
    //                     .cloned().collect();
    //             }

    //             // Eventually change this so if the constraints are empty and no filter is required, to just get belief states to use all_states
    //             if first_filter {
    //                 self.calculated_states = self.all_states.clone();
    //             }
    //         }
    //     }
    //     // let elapsed_time = start_time.elapsed();
    //     // log::info!("Time taken for Optimal GC Filter: {:?}", elapsed_time);
    // }
    pub fn compute_beliefs_direct(&mut self) -> Vec<f64> {
        // Very fast but values slightly different 20~30ms
        // The other versions actually produce wrong values when gc_hm is the only criterion... I dont know why but ill just use this

        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    
        let mut hand_to_index_offset_map: HashMap<String, usize> = HashMap::new();
        let mut count: usize = 0;
        let mut max_count: usize;
        for hand in &self.unique_2p_hands {
            hand_to_index_offset_map.insert(hand.clone(), count);
            count += 1;
        }
        max_count = 6 * count;
        count = 0;
        // Doing the same but for player 7, the pile which has 3 cards
        for hand in &self.unique_3p_hands {
            hand_to_index_offset_map.insert(hand.clone(), count);
            count += 1;
        }
        max_count += count;
    
        let card_freq: Arc<Vec<AtomicUsize>> = Arc::new((0..max_count).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>());
        let total_sum = Arc::new(AtomicUsize::new(0));
    
        self.all_states.par_iter().for_each(|state| {
            if !self.state_satisfies_constraints(state, &latest_constraint) {
                return; // Skip states that do not satisfy the constraints
            }
            total_sum.fetch_add(1, Ordering::SeqCst);
            // Update card frequencies for states that satisfy the constraints
            let mut local_counts = vec![0usize; max_count];

            for player_index in 0..7 {
                let start_index = self.index_start_arr[player_index];
                let end_index = self.index_end_arr[player_index];
                let player_hand = &state[start_index..end_index];
    
                if let Some(&card_index) = hand_to_index_offset_map.get(player_hand) {
                    local_counts[player_index * 15 + card_index] += 1;
                } else {
                    panic!("Invalid card combination");
                }
            }
    
            for (card_index, &count) in local_counts.iter().enumerate() {
                if count > 0 {
                    card_freq[card_index].fetch_add(count, Ordering::SeqCst);
                }
            }
        });
    
        // let total_valid_states = card_freq.iter().map(|freq| freq.load(Ordering::SeqCst)).sum::<usize>();
        let total_valid_states = total_sum.load(Ordering::SeqCst);
        log::info!("Total Valid States {}", total_valid_states);
        let beliefs: Vec<f64> = card_freq.iter()
            .map(|freq| freq.load(Ordering::SeqCst) as f64 / total_valid_states as f64)
            .collect();
    
        beliefs
    }
    
    // Helper method to determine if a state satisfies all constraints
    // TODO: Change back to private
    pub fn state_satisfies_constraints(&self, state: &str, latest_constraint: &CollectiveConstraint) -> bool {
        // println!("Check");
        // Check jc_hm constraints
        for i in 0..6 {
            if let Some(cards) = latest_constraint.get_jc_hm().get(&i) {
                let card_char_vec: Vec<char> = cards.iter().map(|c| self.card_to_char(c)).collect();
                let index_start: usize = 2 * i;
                let index_end: usize;

                index_end = index_start + 2;
 
    
                if state.len() < index_end {
                    return false;
                }
    
                let state_chars: Vec<char> = state[index_start..index_end].chars().collect();
                if state_chars != card_char_vec {
                    return false; // The state does not satisfy this jc_hm constraint
                }
            }
        }
    
        // Check pc_hm constraints
        for i in 0..6 {
            if let Some(card) = latest_constraint.get_pc_hm().get(&i) {
                let card_char: char = self.card_to_char(&card);
                let index_start: usize = 2 * i;
                let index_end: usize;

                index_end = index_start + 2;

    
                if state.len() < index_end || !state[index_start..index_end].contains(card_char) {
                    return false; // The state does not satisfy this pc_hm constraint
                }
            }
        }
    
        // This should check that there are gc_hm_count of the card.
        // Check gc_hm constraints
        // for card in &self.card_list {
        //     if let Some(participation_list) = latest_constraint.gc_hm.get(&card) {
        //         let card_char: char = self.card_to_char(&card);
    
        //         let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
        //             .filter_map(|(player_id, &participation)| {
        //                 if participation == 1 {
        //                     Some((self.index_start_arr[player_id], self.index_end_arr[player_id]))
        //                 } else {
        //                     None
        //                 }
        //             }).collect();
    
        //         let satisfies_gc_hm = participating_indices.iter().any(|&(start, end)| {
        //             state.len() >= end && state[start..end].contains(card_char)
        //         });
    
        //         if !satisfies_gc_hm {
        //             return false; // The state does not satisfy this gc_hm constraint
        //         }
        //     }
        // }
        
        // Check gc_vec constraints
        let mut index: usize = 0;
        // println!("Before While");
        while index < latest_constraint.get_gc_vec().len(){
            let participation_list: &[u8; 7] = latest_constraint.get_gc_vec()[index].get_list();
            let card_char: char = latest_constraint.get_gc_vec()[index].card().card_to_char();

            let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
                    .filter_map(|(player_id, &participation)| {
                        if participation == 1 {
                            Some((self.index_start_arr[player_id], self.index_end_arr[player_id]))
                        } else {
                            None
                        }
                    }).collect();
            let mut total_count = 0;
            let required_count = latest_constraint.get_gc_vec()[index].count();
            let mut satisfies_gc_vec: bool = false;
            // println!("Required Count: {}", required_count);
            // println!("Participation List: {:?}", participation_list);
            // println!("Participation Indices: {:?}", participating_indices);
            for &(start, end) in participating_indices.iter() {
                // println!("Start: {}", start);
                // println!("End: {}", end);
                // println!("State len: {}", state.len());
                if state.len() >= end {
                    total_count += state[start..end].matches(card_char).count();
                    // println!("Total Count: {}", total_count);
                    if total_count >= required_count {
                        satisfies_gc_vec = true;
                        break;
                    }
                }
            }
            if !satisfies_gc_vec {
                return false; // The state does not satisfy this gc_vec constraint
            }
            index += 1;
        }

        true // The state satisfies all constraints
    }
    // pub fn get_leaf_belief(&mut self) -> Vec<f64>{
    //     self.filter_state_optimal();
    //     self.get_latest_beliefs_concurrent()
    // }
    // pub fn gen_and_save_belief(&mut self) {
    //     // self.filter_state_optimal();
    //     let beliefs: Vec<f64> = self.compute_beliefs_direct();
    //     let key: String = self.make_key_belief();
    //     self.add_to_hashmap(key, beliefs.clone());
    // }
    pub fn chance_reveal_redraw(&mut self, player_id: usize, temp_vec: Vec<String>) -> HashMap<String, String>{
        // Hand list is a list of possible hands with valid reach probability we wish to search for
        // "AA" "AB" etc. 
        // Return HashMap of transition hand probabilities 
            // "AA" -> "AA" "AB" "AC" "AD" "AE" use card
            // "AB" -> "AB" "BB" "BC" "BD" "BE" (Revealed A)
            // So its Player hand == "AB" A|XXX 
            // First X is card revealed
            // XXX is pile
            // Condition: Hand is "AB" => ["AB" += 1] and if X is "B" "BB" += 1 etc.
        // Need original reach probabilities too?
        // 2 Cases 2 alive cards and 1 alive card -> Check constraint
        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        let start_index: usize = player_id * 2;
        let end_index: usize = start_index + 2;
        let mut rng = rand::thread_rng();
        self.all_states.shuffle(&mut rng); // Shuffle in place

        let results = Arc::new(Mutex::new(HashMap::new()));

        let start_time = Instant::now();

        let temp_vec_set: HashSet<String> = temp_vec.into_iter().collect();
        // 22ms
        self.all_states.par_iter().for_each_with(HashSet::new(), |local_set, state| {
            if start_index < state.len() && end_index <= state.len() {
                let state_substring = &state[start_index..end_index];

                // Early local check to minimize lock contention
                if temp_vec_set.contains(state_substring) && !local_set.contains(state_substring) {
                    if self.state_satisfies_constraints(state, &latest_constraint) {
                        local_set.insert(state_substring.to_string());
                        let mut results_lock = results.lock().unwrap();
                        results_lock.entry(state_substring.to_string()).or_insert_with(|| state.clone());
                    }
                }
            }
        });
        let elapsed_time = start_time.elapsed();
        log::info!("Time Taken to check: {:?}", elapsed_time);
        
        Arc::try_unwrap(results).expect("Failed to unwrap Arc").into_inner().expect("Failed to unlock Mutex")
    }
    pub fn chance_reveal_redraw_exit(&mut self, player_id: usize, temp_vec: Vec<String>) -> HashMap<String, String> {
        // Fastest, use this one
        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        let start_index: usize = player_id * 2;
        let end_index: usize = start_index + 2;
        let mut rng = rand::thread_rng();
        self.all_states.shuffle(&mut rng); // Shuffle in place

        let results = Arc::new(Mutex::new(HashMap::new()));
        let found_count = Arc::new(AtomicUsize::new(0));
        let should_exit = Arc::new(AtomicBool::new(false));
        let temp_vec_set: HashSet<String> = temp_vec.into_iter().collect();

        let start_time = Instant::now();

        self.all_states.par_iter().for_each_with((HashSet::new(), Arc::clone(&should_exit)), |(local_set, should_exit), state| {
            if should_exit.load(Ordering::SeqCst) {
                // Early exit if all necessary states have been found.
                return;
            }
            
            if start_index < state.len() && end_index <= state.len() {
                let state_substring = &state[start_index..end_index];

                if temp_vec_set.contains(state_substring) && !local_set.contains(state_substring) {
                    if self.state_satisfies_constraints(state, &latest_constraint) {
                        local_set.insert(state_substring.to_string());
                        let mut results_lock = results.lock().unwrap();
                        if !results_lock.contains_key(state_substring) {
                            results_lock.insert(state_substring.to_string(), state.clone());
                            let count = found_count.fetch_add(1, Ordering::SeqCst) + 1;
                            if count >= temp_vec_set.len() {
                                // Signal to other threads to stop processing.
                                should_exit.store(true, Ordering::SeqCst);
                            }
                        }
                    }
                }
            }
        });

        let elapsed_time = start_time.elapsed();
        log::info!("Time Taken to check: {:?}", elapsed_time);

        Arc::try_unwrap(results).expect("Failed to unwrap Arc").into_inner().expect("Failed to unlock Mutex")
    }
    pub fn chance_reveal_redraw_norm(&mut self, player_id: usize, temp_vec: Vec<String>) -> HashMap<String, String> {
        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        let start_index: usize = player_id * 2;
        let end_index: usize = start_index + 2;
        let mut rng = rand::thread_rng();
        self.all_states.shuffle(&mut rng); // Shuffle in place

        let mut results: HashMap<String, String> = HashMap::new();
        let temp_vec_set: HashSet<String> = temp_vec.into_iter().collect();
        let mut found_count = 0;

        let start_time = Instant::now();

        for state in &self.all_states {
            if found_count >= temp_vec_set.len() {
                // Exit early if we've found matches for all items in temp_vec_set
                break;
            }
            
            if start_index < state.len() && end_index <= state.len() {
                let state_substring = &state[start_index..end_index];

                if temp_vec_set.contains(state_substring) && !results.contains_key(state_substring) {
                    if self.state_satisfies_constraints(state, &latest_constraint) {
                        results.insert(state_substring.to_string(), state.clone());
                        found_count += 1;
                    }
                }
            }
        }

        let elapsed_time = start_time.elapsed();
        log::info!("Time Taken to check: {:?}", elapsed_time);

        results
    }
    pub fn chance_sample_exit(&mut self) -> Option<String> {
        // Returns None if no String could be found
        // Returns Some(String) if a string that satisfies the constraints could be found
        // Randomly Finds the first string that fulfils the criterion
        // Fastest, use this one
        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        // let start_time = Instant::now();
        // This takes around 20 ms instead of the 50 ms normally taken to shuffle
        let mut rng = rand::thread_rng();

        self.all_states.shuffle(&mut rng); // Shuffle in place
        // let elapsed_time = start_time.elapsed();
        // println!("Shuffle Time: {:?}", elapsed_time);

        let result = Arc::new(Mutex::new(None));
        let should_exit = Arc::new(AtomicBool::new(false));

        self.all_states.par_iter().for_each_with(Arc::clone(&should_exit), |should_exit, state| {
            if should_exit.load(Ordering::SeqCst) {
                // Early exit if a string has been found.
                return;
            }
            
            if self.state_satisfies_constraints(state, &latest_constraint) {
                let mut result_lock = result.lock().unwrap();
                if result_lock.is_none() {
                    *result_lock = Some(state.to_string());
                    should_exit.store(true, Ordering::SeqCst); // Signal to other threads to stop processing
                    return; // Exit this thread's processing early
                }
            }
        });

        let result_lock = result.lock().unwrap();
        result_lock.clone() // Return the first result if available
    }
    pub fn chance_sample_exit_test(&mut self, constraint: &CollectiveConstraint) -> Option<String> {
        // Returns None if no String could be found
        // Returns Some(String) if a string that satisfies the constraints could be found
        // Randomly Finds the first string that fulfils the criterion
        // Fastest, use this one
        // let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        // let mut rng = rand::thread_rng();
        // self.all_states.shuffle(&mut rng); // Shuffle in place

        let result = Arc::new(Mutex::new(None));
        let should_exit = Arc::new(AtomicBool::new(false));

        self.all_states.par_iter().for_each_with(Arc::clone(&should_exit), |should_exit, state| {
            if should_exit.load(Ordering::SeqCst) {
                // Early exit if a string has been found.
                return;
            }
            
            if self.state_satisfies_constraints(state, constraint) {
                let mut result_lock = result.lock().unwrap();
                if result_lock.is_none() {
                    *result_lock = Some(state.to_string());
                    should_exit.store(true, Ordering::SeqCst); // Signal to other threads to stop processing
                    return; // Exit this thread's processing early
                }
            }
        });

        let result_lock = result.lock().unwrap();
        result_lock.clone() // Return the first result if available
    }
    pub fn can_player_have_card(&mut self, player_id: usize, card: &Card) -> Option<String> {
        // Returns None if no String could be found
        // Returns Some(String) if a string that satisfies the constraints could be found
        // Randomly Finds the first string that fulfils the criterion
        // Fastest, use this one
        let mut latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        let dead_cards: u8 = latest_constraint.dead_card_count()[card];
        if dead_cards == 3 {
            return None;
        }
        if player_id != 6 {
            latest_constraint.add_raw_public_constraint(player_id, *card);
        } else {
            // Cannot treat the card as dead!
            latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], *card, 0, 1));
        }
        let mut rng = rand::thread_rng();
        self.all_states.shuffle(&mut rng); // Shuffle in place

        let result = Arc::new(Mutex::new(None));
        let should_exit = Arc::new(AtomicBool::new(false));

        self.all_states.par_iter().for_each_with(Arc::clone(&should_exit), |should_exit, state| {
            if should_exit.load(Ordering::SeqCst) {
                // Early exit if a string has been found.
                return;
            }
            
            if self.state_satisfies_constraints(state, &latest_constraint) {
                let mut result_lock = result.lock().unwrap();
                if result_lock.is_none() {
                    *result_lock = Some(state.to_string());
                    should_exit.store(true, Ordering::SeqCst); // Signal to other threads to stop processing
                    return; // Exit this thread's processing early
                }
            }
        });

        let result_lock = result.lock().unwrap();
        result_lock.clone() // Return the first result if available
    }
    pub fn can_player_have_card_test(&mut self, constraint: &CollectiveConstraint, player_id: usize, card: &Card) -> Option<String> {
        // Returns None if no String could be found
        // Returns Some(String) if a string that satisfies the constraints could be found
        // Randomly Finds the first string that fulfils the criterion
        // Fastest, use this one
        // let mut latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        let mut latest_constraint = constraint.clone();
        let dead_cards: u8 = latest_constraint.dead_card_count()[card];
        if dead_cards == 3 {
            return None;
        }
        if player_id != 6 {
            latest_constraint.add_raw_public_constraint(player_id, *card);
        } else {
            // Cannot treat the card as dead!
            latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], *card, 0, 1));
        }
        let mut rng = rand::thread_rng();
        self.all_states.shuffle(&mut rng); // Shuffle in place

        let result = Arc::new(Mutex::new(None));
        let should_exit = Arc::new(AtomicBool::new(false));

        self.all_states.par_iter().for_each_with(Arc::clone(&should_exit), |should_exit, state| {
            if should_exit.load(Ordering::SeqCst) {
                // Early exit if a string has been found.
                return;
            }
            
            if self.state_satisfies_constraints(state, &latest_constraint) {
                let mut result_lock = result.lock().unwrap();
                if result_lock.is_none() {
                    *result_lock = Some(state.to_string());
                    should_exit.store(true, Ordering::SeqCst); // Signal to other threads to stop processing
                    return; // Exit this thread's processing early
                }
            }
        });

        let result_lock = result.lock().unwrap();
        result_lock.clone() // Return the first result if available
    }
    pub fn can_player_have_cards(&mut self, player_id: usize, cards: &[Card; 2]) -> Option<String> {
        // Returns None if no String could be found
        // Returns Some(String) if a string that satisfies the constraints could be found
        // Randomly Finds the first string that fulfils the criterion
        // Fastest, use this one
        let mut latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        if player_id == 6 {
            if cards[0] == cards[1] {
                latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[0], 0, 2));
            } else {
                latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[0], 0,1));
                latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[1], 0, 1));
            }
        } else {
            // Now test with both constraints
            latest_constraint.add_raw_public_constraint(player_id, cards[0]);
            latest_constraint.add_raw_public_constraint(player_id, cards[1]);
        }
        let result = Arc::new(Mutex::new(None));
        let should_exit = Arc::new(AtomicBool::new(false));

        self.all_states.par_iter().for_each_with(Arc::clone(&should_exit), |should_exit, state| {
            if should_exit.load(Ordering::SeqCst) {
                // Early exit if a string has been found.
                return;
            }
            
            if self.state_satisfies_constraints(state, &latest_constraint) {
                let mut result_lock = result.lock().unwrap();
                if result_lock.is_none() {
                    *result_lock = Some(state.to_string());
                    should_exit.store(true, Ordering::SeqCst); // Signal to other threads to stop processing
                    return; // Exit this thread's processing early
                }
            }
        });

        let result_lock = result.lock().unwrap();
        result_lock.clone() // Return the first result if available
    }

    pub fn player_can_have_card(&self, player_id: usize, card: &Card) -> bool {
        // This is the ideal set theory version
        // ~20µs
        // never more than 50µs
        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        latest_constraint.player_can_have_active_card(player_id, card)
    }
    pub fn player_can_have_card_constructor(&mut self, player_id: usize, card: &Card) -> bool {
        // This is the ideal constructed version
        // > 20µs up to 500µs
        let mut latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        latest_constraint.add_raw_public_constraint(player_id, *card);
        let output = self.naive_sampler.par_constructor(&latest_constraint);
        if output.is_none() {
            return false
        } else {
            log::info!("Possible Combo: {}", output.unwrap());
            return true
        }
    }
    pub fn sample_random_state(&mut self) -> Option<String> {
        // This is the ideal constructed Sampler
        // MEDIAN: 200µs
        // MEAN: 1.5 ms
        // Much 100-500x faster than Brute
        self.naive_sampler.par_constructor(&self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap())
    }
    pub fn player_can_have_cards(&self, player_id: usize, cards: &[Card; 2]) -> bool {
        // This is the ideal set theory version
        // Does not work for player_id == 6
        // MEDIAN: 22µs
        // MEAN: 22µs
        // MAX: 65.8µs (600 cases)
        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        latest_constraint.player_can_have_active_cards(player_id, cards)
    }
    pub fn player_can_have_cards_constructor(&mut self, player_id: usize, cards: &[Card; 2]) -> bool {
        // This is the ideal constructed version
        // MEDIAN: 25µs
        // MEAN: 32µs
        // MAX: 519.8µs (600 cases)
        let mut latest_constraint: CollectiveConstraint = self.latest_constraint();
        if player_id == 6 {
            if cards[0] == cards[1] {
                latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[0], 0, 2));
            } else {
                latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[0], 0, 1));
                latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[1], 0, 1));
            }
        } else {
            latest_constraint.add_raw_public_constraint(player_id, cards[0]);
            latest_constraint.add_raw_public_constraint(player_id, cards[1]);
        }
        if self.naive_sampler.par_constructor(&latest_constraint).is_none(){
            return false
        } else {
            return true
        }
    }
    pub fn player_can_have_cards_constructor_state(&mut self, player_id: usize, cards: &[Card; 2]) -> Option<String> {
        // This is the ideal constructed version
        // MEDIAN: 25µs
        // MEAN: 32µs
        // MAX: 519.8µs (600 cases)
        let mut latest_constraint: CollectiveConstraint = self.latest_constraint();
        if player_id == 6 {
            if cards[0] == cards[1] {
                latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[0], 0, 2));
            } else {
                latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[0], 0, 1));
                latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[1], 0, 1));
            }
        } else {
            latest_constraint.add_raw_public_constraint(player_id, cards[0]);
            latest_constraint.add_raw_public_constraint(player_id, cards[1]);
        }
        self.naive_sampler.par_constructor(&latest_constraint)
    }
    pub fn filter_player_can_have_card(&mut self, player_id: usize, card: &Card){
        let mut latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        // Use raw because we are wondering if player can have card and do not know if they actually have the card
        latest_constraint.add_raw_public_constraint(player_id, *card);
        self.calculated_states = self.all_states.par_iter()
            .filter(|state| self.state_satisfies_constraints(state, &latest_constraint))
            .cloned()
            .collect();
    }
    pub fn filter_player_can_have_cards(&mut self, player_id: usize, cards: &[Card; 2]){
        self.filter_player_can_have_card(player_id, &cards[0]);
        if self.calc_state_len() != 0 {

            let mut latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
            // Use raw because we are wondering if player can have card and do not know if they actually have the card
            // Now test with both constraints
            latest_constraint.add_raw_public_constraint(player_id, cards[0]);
            latest_constraint.add_raw_public_constraint(player_id, cards[1]);
            self.calculated_states = self.all_states.par_iter()
                .filter(|state| self.state_satisfies_constraints(state, &latest_constraint))
                .cloned()
                .collect();
        }
    }
    // Simply updates calculated states to align with the latest constraints
    pub fn update_calculated_states(&mut self) {
        let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        self.calculated_states = self.all_states.par_iter()
        .filter(|state| self.state_satisfies_constraints(state, &latest_constraint))
        .cloned()
        .collect();
    }
    /// Returns all the cards for each player that we are certain they have
    /// Assumes calculates states align with latest constraints
    pub fn validated_inferred_constraints(&self) -> Vec<Vec<Card>> {
        let mut result = [[0; 5]; 7];
    
        let player_indices = vec![
            vec![0, 1],      // Player 0
            vec![2, 3],      // Player 1
            vec![4, 5],      // Player 2
            vec![6, 7],      // Player 3
            vec![8, 9],      // Player 4
            vec![10, 11],    // Player 5
            vec![12, 13, 14] // Player 6
        ];
        
        // For each player, we'll go through all possible cards (0..4 if 5 card variants),
        // and compute how many times that card appears in the player's bag for each state.
        // The smallest count across *all* states is the guaranteed number of that card.
        for (player_id, indices) in player_indices.iter().enumerate() {
            for card_val in 0..5 {
                // We start with a very large minimum. We'll keep track of
                // the least number of occurrences across all states.
                let mut min_count = u8::MAX;
                
                for state in &self.calculated_states {
                    // Count how many times this card appears in the indices for the current state
                    let count_in_bag = indices.iter()
                        .filter(|&&idx| {
                            let c = state.chars().nth(idx).unwrap();
                            Card::char_to_card(c) as usize == card_val
                        })
                        .count() as u8;
                    
                    // Update the minimum
                    min_count = min_count.min(count_in_bag);
                    if min_count == 0 {
                        break;
                    }
                }
                
                // The smallest count across all states is how many times
                // we know for sure the player has this card.
                result[player_id][card_val] = min_count;
            }
        }
        let mut dead_result: [[u8; 5]; 7] = [[0; 5]; 7];
        let latest_constraint = self.latest_constraint();
        let public_constraint = latest_constraint.get_pc_hm();
        let joint_constraint = latest_constraint.get_jc_hm();
        for player_id in 0..6 as usize {
            if let Some(card) = public_constraint.get(&player_id) {
                dead_result[player_id][*card as usize] += 1;
            }
            if let Some(thing) = joint_constraint.get(&player_id) {
                for card in thing {
                    dead_result[player_id][*card as usize] += 1;
                }
            }
        }
        let mut alive_result: [[u8; 5]; 7] = [[0; 5]; 7];
        for (player_id, (res, dead_res)) in result.iter().zip(dead_result.iter()).enumerate() {
            for card_idx in 0..5 {
                alive_result[player_id][card_idx] = res[card_idx] - dead_res[card_idx];
            }
        }
        let mut output: Vec<Vec<Card>> = vec![Vec::with_capacity(2); 7];
        for (player_id, counts) in alive_result.iter().enumerate() {
            for (card_num, amount) in counts.iter().enumerate() {
                for _ in 0..*amount {
                    output[player_id].push(Card::try_from(card_num as u8).unwrap());
                }
            }
        }
        for card_vec in output.iter_mut() {
            card_vec.sort_unstable();
        }
        output
    }
    /// Returns all the cards for each player that we are certain they have
    /// Assumes calculates states align with latest constraints
    pub fn validated_impossible_constraints(&self) -> [[bool; 5]; 7] {
        let mut result = [[true; 5]; 7];
        for permutation in self.calculated_states.iter() {
            let card_idx = Card::char_to_card(permutation.chars().nth(0).unwrap()) as usize;
            result[0][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(1).unwrap()) as usize;
            result[0][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(2).unwrap()) as usize;
            result[1][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(3).unwrap()) as usize;
            result[1][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(4).unwrap()) as usize;
            result[2][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(5).unwrap()) as usize;
            result[2][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(6).unwrap()) as usize;
            result[3][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(7).unwrap()) as usize;
            result[3][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(8).unwrap()) as usize;
            result[4][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(9).unwrap()) as usize;
            result[4][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(10).unwrap()) as usize;
            result[5][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(11).unwrap()) as usize;
            result[5][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(12).unwrap()) as usize;
            result[6][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(13).unwrap()) as usize;
            result[6][card_idx] = false;
            let card_idx = Card::char_to_card(permutation.chars().nth(14).unwrap()) as usize;
            result[6][card_idx] = false;
        }
        result
    }
    /// Returns all the dead cards for each player that we are certain they have
    /// Assumes calculates states align with latest constraints
    pub fn validated_public_constraints(&self) -> Vec<Vec<Card>> {
        let mut output: Vec<Vec<Card>> = vec![Vec::with_capacity(2); 7];
        let latest_constraint = self.latest_constraint();
        let public_constraint = latest_constraint.get_pc_hm();
        let joint_constraint = latest_constraint.get_jc_hm();
        for player_id in 0..6 as usize {
            if let Some(card) = public_constraint.get(&player_id) {
                output[player_id].push(*card);
            }
            if let Some(thing) = joint_constraint.get(&player_id) {
                for card in thing {
                    output[player_id].push(*card);
                }
            }
        }
        for card_vec in output.iter_mut() {
            card_vec.sort_unstable();
        }
        output
    }
}
