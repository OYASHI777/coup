
use super::permutation_generator::{gen_table_combinations, gen_bag_combinations};
use super::coup_const::{BAG_SIZES, MAX_HAND_STATES, TOKENS, MAX_PERM_STATES};
use crate::string_utils::sort_str;
use crate::string_utils::{contains_all_chars, remove_chars};
use ndarray::{ArrayView1, ArrayView2};
use rayon::prelude::*;
use indexmap::IndexMap;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap};
use std::cell::RefCell;
#[derive(Debug)]
pub struct Constraints {
    // String not &str as it will not be known at compile time
    p0: Option<String>,
    p1: Option<String>,
    p2: Option<String>,
    p3: Option<String>,
    p4: Option<String>,
    p5: Option<String>,
}

impl Constraints {
    pub fn new() -> Self {
        Constraints{
            p0: None,
            p1: None,
            p2: None,
            p3: None,
            p4: None,
            p5: None,
        }
    }
    pub fn add(&mut self, id: usize ,constrain: &str){
        debug_assert!(id <= 5, "Please provide a proper id");
        let target = match id {
            0 => &mut self.p0,
            1 => &mut self.p1,
            2 => &mut self.p2,
            3 => &mut self.p3,
            4 => &mut self.p4,
            5 => &mut self.p5,
            _ => panic!("Invalid ID"), // Handled by the debug_assert! above.
        };
        // Use take() to replace target with None temporarily while working on it
        *target = match target.take(){
            None => Some(constrain.to_string()),
            Some(x) => {
                let mut chars: Vec<char> = x.chars().chain(constrain.chars()).collect();
                chars.sort_unstable();
                Some(chars.iter().collect())
            },
        };
        // TODO: Add a debug_assert! if constrain bursts
    }
    pub fn remove(&mut self, id: usize, constrain: &str){
        debug_assert!(id <= 5, "Invalid ID used in remove");
        
        let target = match id {
            0 => &mut self.p0,
            1 => &mut self.p1,
            2 => &mut self.p2,
            3 => &mut self.p3,
            4 => &mut self.p4,
            5 => &mut self.p5,
            _ => panic!("Invalid ID"), // Handled by the debug_assert! above.
        };

        if let Some(x) = target {
            if x == constrain {
                *target = None;
            } else if constrain.len() < x.len(){
                let mut result = x.clone();
                for c in constrain.chars() {
                    // Gets first occurance and removes it
                    if let Some(pos) = result.find(c) {
                        result.remove(pos);
                    }
                }
                *target = Some(result);
                
            } else {
                debug_assert!(false, "input constraint len too long! Longer than stored constraint!");
            }
        }
    }
    pub fn get_constraint(&self, id: usize) -> Option<&String>{
        debug_assert!(id <= 5, "Invalid ID used in get_constraint");
        match id {
            0 => self.p0.as_ref(),
            1 => self.p1.as_ref(),
            2 => self.p2.as_ref(),
            3 => self.p3.as_ref(),
            4 => self.p4.as_ref(),
            5 => self.p5.as_ref(),
            _ => None,
        }
    }
    pub fn satisfied(&self, deck_string: &str) -> bool {
        // Checks if deck_string satisfies constrains

        let constraints = [&self.p0, &self.p1, &self.p2, &self.p3, &self.p4, &self.p5];
        // Checks constraints for all players
        for (player_id, target) in constraints.iter().enumerate() {
            let index_start: usize = player_id * 2;
            let index_end: usize = index_start + 2;

            if let Some(constraint) = target {
                // Ensure index_end does not exceed deck_string length
                debug_assert!(index_end <= deck_string.len(), "Deck_string too small! or index_end too large!");

                let slice = &deck_string[index_start..index_end];

                // Check if the constraint is not satisfied
                if constraint.len() == 2 && slice != constraint {
                    return false;
                } else if constraint.len() == 1 && !slice.contains(constraint) {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Debug)]
pub struct ProbState {
    //TODO: Change to indexmap
    // Consider f32 precision
    pub prob_stores: [IndexMap<String, f64>; 6],
    pub constraints: Constraints,
    pub save_state: bool,
}

// Should initialise with all 1.5 million states for 6 players
    // 6 seperate prob stores
        // Initialise prob should be 1/1.5mil
    // 1 bimap
// Constraints should be a symbolic representation not a list
    // So when 1/2/3 cards become public information computation needed can reduce by around 15x (if 1 player's full hand is shown there are only 100k possible permutations)
    // None "A" "AB"
    // Combinable
    // "A" + "B" => "AB" (ordered)
    // Constraints limit what can be created next
        // Make first node a special case of constraint?
    // So [Node 0] -> (Action 0) -> [Node 1]
        // [Node 1 should contain constraints for output of Action 0]
        // if Node 0 is the starting, it contains constraints for its own generation
// Save_State should be true / false
    // have a delete function
// Replay to be stored in manager not in probstate
    // Manager will track all macro features
// Transition functionality required | We check the probability based on the conditional probabilities
    // 1 to 1 Non Poll moves that do not shuffle cards => Income/Foreign Aid/Coup/Exchange/Steal/Assassinate/
    // Swap moves => ExchangeChoice
        // Own player ExchangeChoice is simple, its just a constraint, cos ur prob of doing action is 1, so 
        // Tocheck => if player0's size needs to be 1.5 million long or the constrained one based on private information
            // player0 size can be constrained to private information as p(h | s_{i}, beta_{-i}) includes infostate that has private information
    // Random Swap moves => RevealRedraw
        // This needs to be constrained further behind, then shuffle
        // The shuffle will be handled here
        // Constrained behind will be recalculated by manager
    // 1 to 1 Collective Poll moves => CollectiveChallenge CollectiveBlock
        // 6 probability choices but still 1 to 1
// Transition functions to borrow another ProbState and modify it
// THOUGHT: I think when looking ahead, a reveal card does not need to backtrack?
// THOUGHT: Actually do u even need to backtrack in general, if you just constraint on reveal doesnt that cover the prob?
    // IF you dont need to backtrack in general you can just normalise at the root
    // Dont think need to backtrack in general, it is impossible for you to jump from an impossible state to your constraint state between card reveal and last card swap
// Remember root node probability to normalise
// THOUGHT: Instead of storing a player's belief states as 1.5 millions stuff, can u just decompose it to 6x15 = 90 + the amb side
    // Might not even need 6 belief states if this is ok
    // Will not work

// THOUGHT: Why do u even need a probability Engine?
    // All transition probs are from CFR, prob of reaching that history though needs it
// PMCCFR only needs chance probabilities
    // All strategy probabilities seem to be not conditional?
    
impl ProbState{
    pub fn new() -> Self {
        ProbState{
            prob_stores: [
                IndexMap::with_capacity(MAX_HAND_STATES), // Intended Contains private information
                IndexMap::with_capacity(MAX_PERM_STATES),
                IndexMap::with_capacity(MAX_PERM_STATES),
                IndexMap::with_capacity(MAX_PERM_STATES),
                IndexMap::with_capacity(MAX_PERM_STATES),
                IndexMap::with_capacity(MAX_PERM_STATES),
            ],
            constraints: Constraints::new(),
            save_state: false,
        }
    }
    pub fn set_save_state(&mut self, new_state: bool){
        self.save_state = new_state;
    }
    pub fn game_start(&mut self, hand: &str) {
        //game_start              time:   [2.2416 s 2.2688 s 2.2974 s]
        // Not much speed up as operation is very simple
        // TODO: Split into seperate function for 1-5 and one for p0
        // game_start              time:   [2.1389 s 2.1478 s 2.1572 s]
        // TODO LOW PRIO improve gen_table_combinations for when there are constraints 
        //                  Important for testing later on
        // Can make faster by only creating those fulfilling the constraints
        let hand_vec = gen_table_combinations(TOKENS, &BAG_SIZES);
        let hand_vec_filtered: Vec<String> = hand_vec.iter().filter(|x| self.constraints.satisfied(x)).cloned().collect();
        let prob: f64 = 1.0 / hand_vec.len() as f64;
        self.prob_stores.par_iter_mut().enumerate().for_each(|(store_index, prob_store)|{
            match store_index {
                0 => parallel_insertion_cond(prob_store, &hand_vec_filtered, hand, prob),
                _ => parallel_insertion(prob_store, &hand_vec_filtered, prob),
            }
        });
    }
    pub fn standard_move_hp(&mut self, probabilities: &HashMap<String, f64>, player_id: usize){
        let prob_stores: [Arc<Mutex<IndexMap<String, f64>>>; 6] = [
            Arc::new(Mutex::new(IndexMap::with_capacity(MAX_HAND_STATES))), // Intended Contains private information
            Arc::new(Mutex::new(IndexMap::with_capacity(MAX_PERM_STATES))),
            Arc::new(Mutex::new(IndexMap::with_capacity(MAX_PERM_STATES))),
            Arc::new(Mutex::new(IndexMap::with_capacity(MAX_PERM_STATES))),
            Arc::new(Mutex::new(IndexMap::with_capacity(MAX_PERM_STATES))),
            Arc::new(Mutex::new(IndexMap::with_capacity(MAX_PERM_STATES))),
        ];
        // Loop through prob_stores
        let index_start = player_id * 2;
        let index_end = player_id * 2 + 2;
        // For each key in indexmap find the probabilities in HashMap[key[index_start:index_end]] and multiply it by the value in the self.probstores indexmap and save it to prob_stores
        // if player_id == prob_stores index, multiply by 1 (or really just clone it whichever is faster)
        // Loop through prob_stores in parallel
        // Loop through prob_stores in parallel
        self.prob_stores.par_iter().enumerate().for_each(|(i, store)| {
            // If the current prob_stores index matches the player_id
            if i == player_id {
                // Copy the values from self.prob_stores to prob_stores
                let mut prob_stores_lock = prob_stores[i].lock().unwrap();
                *prob_stores_lock = store.clone();
            } else {
                let mut new_store = prob_stores[i].lock().unwrap();
                // Iterate over keys in the store
                for (key, value) in store {
                    // Extract the substring based on the index range
                    let sub_key = &key[index_start..index_end];

                    // Retrieve the probability from the HashMap
                    if let Some(prob) = probabilities.get(sub_key) {
                        // Multiply the probability with the value in prob_stores and update the value
                        new_store.insert(key.clone(), *value * prob);
                    }
                }
            }
        });
    }   
    pub fn standard_move(&mut self, probabilities: &HashMap<String, f64>, player_id: usize) {
        let mut prob_stores: [IndexMap<String, f64>; 6] = [
            IndexMap::with_capacity(MAX_HAND_STATES),
            IndexMap::with_capacity(MAX_PERM_STATES),
            IndexMap::with_capacity(MAX_PERM_STATES),
            IndexMap::with_capacity(MAX_PERM_STATES),
            IndexMap::with_capacity(MAX_PERM_STATES),
            IndexMap::with_capacity(MAX_PERM_STATES),
        ];
    
        let index_start = player_id * 2;
        let index_end = player_id * 2 + 2;
    
        for (i, store) in self.prob_stores.iter().enumerate() {
            if i == player_id {
                // Copy the values from self.prob_stores to prob_stores
                prob_stores[i] = store.clone();
            } else {
                // Iterate over keys in the store
                for (key, value) in store {
                    // Extract the substring based on the index range
                    let sub_key = &key[index_start..index_end];
    
                    // Retrieve the probability from the HashMap
                    if let Some(prob) = probabilities.get(sub_key) {
                        // Multiply the probability with the value in prob_stores and update the value
                        prob_stores[i].insert(key.clone(), *value * prob);
                    }
                }
            }
        }
    }
}
pub fn parallel_insertion_cond(prob_store: &mut IndexMap<String, f64>, keys: &Vec<String>, condition: &str, prob: f64) {
    // Wrap the IndexMap in a mutex for concurrent access
    let prob_store_mutex = Arc::new(Mutex::new(prob_store));

    // Iterate over keys in parallel and insert them into the prob_store if the condition is met
    keys.par_iter().for_each(|key| {
        // Check the condition for the first two characters of the key
        if key.starts_with(condition) {
            // Lock the mutex to access the prob_store
            let mut prob_store = prob_store_mutex.lock().unwrap();
            // Insert the key-value pair into the prob_store
            prob_store.insert(key.clone(), prob);
        }
    });
}
pub fn parallel_insertion(prob_store: &mut IndexMap<String, f64>, keys: &Vec<String>, prob: f64) {
    // Wrap the IndexMap in a mutex for concurrent access
    let prob_store_mutex = Arc::new(Mutex::new(prob_store));

    // Iterate over keys in parallel and insert them into the prob_store
    keys.par_iter().for_each(|key| {
        // Lock the mutex to access the prob_store
        let mut prob_store = prob_store_mutex.lock().unwrap();
        // Insert the key-value pair into the prob_store
        prob_store.insert(key.clone(), prob);
    });
}