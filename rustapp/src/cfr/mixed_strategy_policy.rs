use rand::Rng;
use rand::thread_rng;
use ahash::AHashMap;

use crate::history_public::ActionObservation;
use super::keys::{BRKey, MSKey, Infostate, MAX_NUM_BRKEY};
use std::collections::hash_map::Keys;
use rand::seq::SliceRandom;

pub struct HeuristicMixedStrategyPolicy {
    policies: AHashMap<MSKey, AHashMap<BRKey, Vec<f32>>>, //TODO: Change AHashMap with BRKey to IntMap somehow
    // action_map is for possible moves made at MSKey path location | not included in path but are future moves
    action_map: AHashMap<MSKey, Vec<ActionObservation>>,
}
pub trait MSInterface {
    fn reset(&mut self);
    fn update(&mut self, key: &MSKey, possible_moves: &Vec<ActionObservation>, update_values: &AHashMap<BRKey, Vec<f32>>);
    fn is_key_in_action_map(&self, key: &MSKey) -> bool;
    fn update_action_map(&mut self, key: &MSKey, possible_moves: &Vec<ActionObservation>);
    fn add_value(&mut self, key_ms: &MSKey, player_id: u8, infostate: &Infostate, index: usize, value: f32);
    fn policies_contains_key(&self, key: &MSKey) -> bool;
    fn action_map_get(&self, key: &MSKey) -> Option<&Vec<ActionObservation>>;
    fn action_map_contains_key(&self, key: &MSKey) -> bool;
    fn policy_insert(&mut self, key: MSKey, value: AHashMap<BRKey, Vec<f32>>);
    fn action_map_insert(&mut self, key: MSKey, value: Vec<ActionObservation>);
    fn insert_default_if_empty_by_keys(&mut self, player_id: u8, path: &str, actions: &Vec<ActionObservation>, infostates: Keys<'_, Infostate, bool>);
    fn policy_get_mut(&mut self, key: &MSKey) -> Option<&mut AHashMap<BRKey, Vec<f32>>>;
    fn policy_get(&self, key: &MSKey) -> Option<& AHashMap<BRKey, Vec<f32>>>;
    fn get_best_response_index(&self, key: &MSKey, infostate: &Infostate) -> Option<usize>;
    fn get_best_response(&self, key: &MSKey, infostate: &Infostate) -> ActionObservation;
}

impl HeuristicMixedStrategyPolicy {
    
}

impl HeuristicMixedStrategyPolicy {
    pub fn new() -> Self {
        HeuristicMixedStrategyPolicy {
            policies: AHashMap::new(),
            action_map: AHashMap::new(),
        }
    }

}

impl MSInterface for HeuristicMixedStrategyPolicy {
    fn reset(&mut self) {
        self.policies.clear();
        self.action_map.clear();
    }
    fn update(&mut self, key: &MSKey, possible_moves: &Vec<ActionObservation>, update_values: &AHashMap<BRKey, Vec<f32>>) {
        self.update_action_map(key, possible_moves);
        if let Some(policy) = self.policies.get_mut(key) {
            for (infostate, old_values_vec) in policy.iter_mut() {
                let mut i: usize = 0;
                while i < old_values_vec.len() {
                    old_values_vec[i] += update_values[infostate][i];
                    i += 1;
                }
            }
        } else {
            // TODO: Need a function to create new policies based on 
            let mut policy: AHashMap<BRKey, Vec<f32>> = AHashMap::with_capacity(MAX_NUM_BRKEY);
            for (infostate,  value) in update_values.iter() {
                policy.insert(infostate.clone(), vec![0.0; possible_moves.len()]);
            }
            self.policies.insert(key.clone(), policy);
        }
    }
    fn is_key_in_action_map(&self, key: &MSKey) -> bool {
        if let Some(_) = self.action_map.get(key) {
            true
        } else {
            false
        }
    }
    fn update_action_map(&mut self, key: &MSKey, possible_moves: &Vec<ActionObservation>) {
        if !self.is_key_in_action_map(key) {
            self.action_map.insert(key.clone(), possible_moves.clone());
        }
    }
    fn add_value(&mut self, key_ms: &MSKey, player_id: u8, infostate: &Infostate, index: usize, value: f32) {
        let key_br: BRKey = BRKey::new(player_id, infostate);
        if let Some(policy) = self.policies.get_mut(key_ms) {
            if let Some(infostate_policy) = policy.get_mut(&key_br) {
                if let Some(policy_value) = infostate_policy.get_mut(index) {
                    *policy_value += value;
                } else {
                    panic!("Index not in policy vector");
                }
            } else {
                panic!("BRKey not in policy hashmap!");
            }
        } else {
            panic!("MSKey not in policies hashmap")
        }
    }
    fn policies_contains_key(&self, key: &MSKey) -> bool {
        self.policies.contains_key(key)
    }
    fn action_map_contains_key(&self, key: &MSKey) -> bool {
        self.action_map.contains_key(key)
    }
    fn policy_insert(&mut self, key: MSKey, value: AHashMap<BRKey, Vec<f32>>) {
        self.policies.insert(key, value);
    }
    fn action_map_insert(&mut self, key: MSKey, value: Vec<ActionObservation>) {
        self.action_map.insert(key, value);
    }
    fn insert_default_if_empty_by_keys(&mut self, player_id: u8, path: &str, actions: &Vec<ActionObservation>, infostates: Keys<'_, Infostate, bool>) {
        let key_ms: MSKey = MSKey::new(player_id, path); 
        let mut key_br: BRKey = BRKey::new(player_id, &Infostate::AA);
        let mut new_policy: AHashMap<BRKey, Vec<f32>> = AHashMap::with_capacity(MAX_NUM_BRKEY);
        for infostate in infostates {
            key_br.set_infostate(infostate);
            new_policy.insert(key_br, vec![0.0; actions.len()]);
        }
        if !self.action_map_contains_key(&key_ms) {
            self.action_map_insert(key_ms.clone(), actions.clone());
            if self.action_map_contains_key(&key_ms){
            }
        }
        if !self.policies_contains_key(&key_ms) {
            self.policy_insert(key_ms.clone(), new_policy.clone());
            if self.policies_contains_key(&key_ms){
            }
            
        }
    }
    fn action_map_get(&self, key: &MSKey) -> Option<&Vec<ActionObservation>> {
        self.action_map.get(key)
    }
    fn policy_get_mut(&mut self, key: &MSKey) -> Option<&mut AHashMap<BRKey, Vec<f32>>> {
        self.policies.get_mut(key)
    }
    fn policy_get(&self, key: &MSKey) -> Option<&AHashMap<BRKey, Vec<f32>>> {
        self.policies.get(key)
    }
    fn get_best_response_index(&self, key: &MSKey, infostate: &Infostate) -> Option<usize> {
        // Returns index of highest value
        // returns random index if all the same
        if let Some(policy) = self.policies.get(key) {
            let player_id = key.player_id();
            let key: BRKey = BRKey::new(player_id, infostate);
            if let Some(infostate_policy) = policy.get(&key) {
                // infostate_policy.iter().enumerate().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).map(|(index, _)| index)
                // Find the maximum value in the infostate_policy
                if let Some(&max_value) = infostate_policy.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
                    // Collect all indices that have the maximum value
                    let max_indices: Vec<usize> = infostate_policy.iter()
                        .enumerate()
                        .filter_map(|(index, &value)| if value == max_value { Some(index) } else { None })
                        .collect();
                    
                    // Select a random index from the max_indices if more than 1
                    if max_indices.len() == 1 {
                        Some(max_indices[0])
                    } else {
                        let mut rng = thread_rng();
                        max_indices.choose(&mut rng).cloned()
                    }
                } else {
                    println!("max_value failed! for {:?}", infostate_policy);
                    panic!("max_value failed!");
                }
            } else {
                None
            }
        } else {
            println!("MSKey of {:?} not found! in get_best_response_index!", key);
            None
        }
    }
    fn get_best_response(&self, key: &MSKey, infostate: &Infostate) -> ActionObservation {
        let action_index: Option<usize> = self.get_best_response_index(key, infostate);
        if let Some(index) = action_index {
            if let Some(action_vec) = self.action_map.get(key){
                action_vec[index].clone()
            } else {
                panic!("action_map does not contain key!");
            }
        } else {
            //TODO: Change and check for random value logic
            let mut rng = thread_rng();
            if let Some(action_vec) = self.action_map.get(key){
                let random_index: usize = rng.gen_range(0..action_vec.len());
                action_vec[random_index].clone() 
            } else {
                panic!("action_map does not contain key!");
            }
        }
    }
}
