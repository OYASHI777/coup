use rand::Rng;
use rand::thread_rng;
use ahash::AHashMap;

use crate::history_public::ActionObservation;
use super::keys::{BRKey, MSKey, Infostate, MAX_NUM_BRKEY};

pub struct HeuristicMixedStrategyPolicy {
    policies: AHashMap<MSKey, AHashMap<BRKey, Vec<f32>>>,
    // action_map is for possible moves made at MSKey path location | not included in path but are future moves
    action_map: AHashMap<MSKey, Vec<ActionObservation>>,
}
pub trait MSInterface {
    fn reset(&mut self);
    fn update(&mut self, key: &MSKey, possible_moves: &Vec<ActionObservation>, update_values: &AHashMap<BRKey, Vec<f32>>);
    fn is_key_in_action_map(&self, key: &MSKey) -> bool;
    fn update_action_map(&mut self, key: &MSKey, possible_moves: &Vec<ActionObservation>);
    fn add_value(&mut self, key_ms: &MSKey, player_id: usize, infostate: &Infostate, index: usize, value: f32);
    fn policies_contains_key(&self, key: &MSKey) -> bool;
    fn action_map_contains_key(&self, key: &MSKey) -> bool;
    fn policy_insert(&mut self, key: MSKey, value: AHashMap<BRKey, Vec<f32>>);
    fn action_map_insert(&mut self, key: MSKey, value: Vec<ActionObservation>);
    fn policy_get_mut(&mut self, key: &MSKey) -> Option<&mut AHashMap<BRKey, Vec<f32>>>;
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
    fn add_value(&mut self, key_ms: &MSKey, player_id: usize, infostate: &Infostate, index: usize, value: f32) {
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
    fn policy_get_mut(&mut self, key: &MSKey) -> Option<&mut AHashMap<BRKey, Vec<f32>>> {
        self.policies.get_mut(key)
    }
    fn get_best_response_index(&self, key: &MSKey, infostate: &Infostate) -> Option<usize> {
        // Returns index of highest value
        if let Some(policy) = self.policies.get(key) {
            let player_id = key.player_id();
            let key: BRKey = BRKey::new(player_id, infostate);
            if let Some(infostate_policy) = policy.get(&key) {
                infostate_policy.iter().enumerate().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).map(|(index, _)| index)
            } else {
                None
            }
        } else {
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
