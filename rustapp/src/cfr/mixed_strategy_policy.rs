use std::collections::HashMap;

use crate::history_public::ActionObservation;
use super::keys::{MSKey, MAX_NUM_BRKEY};

struct HeuristicMixedStrategyPolicy {
    policies: HashMap<MSKey, HashMap<String, Vec<f32>>>,
    action_map: HashMap<MSKey, Vec<ActionObservation>>,
}
pub trait MSInterface {
    fn update(&mut self, key: &MSKey, possible_moves: &Vec<ActionObservation>, update_values: &HashMap<String, Vec<f32>>);
    fn get_best_response_index(&self, key: &MSKey, infostate: &str) -> Option<usize>;
    fn get_best_response(&self, key: &MSKey, infostate: &str) -> ActionObservation;
}

impl HeuristicMixedStrategyPolicy {
    fn new() -> Self {
        HeuristicMixedStrategyPolicy {
            policies: HashMap::new(),
            action_map: HashMap::new(),
        }
    }
    pub fn is_key_in_action_map(&self, key: &MSKey) -> bool {
        if let Some(_) = self.action_map.get(key) {
            true
        } else {
            false
        }
    }
    pub fn update_action_map(&mut self, key: &MSKey, possible_moves: &Vec<ActionObservation>) {
        if !self.is_key_in_action_map(key) {
            self.action_map.insert(key.clone(), possible_moves.clone());
        }
    }
}

impl MSInterface for HeuristicMixedStrategyPolicy {
    fn update(&mut self, key: &MSKey, possible_moves: &Vec<ActionObservation>, update_values: &HashMap<String, Vec<f32>>) {
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
            let mut policy: HashMap<String, Vec<f32>> = HashMap::with_capacity(MAX_NUM_BRKEY);
            for (infostate,  value) in update_values.iter() {
                policy.insert(infostate.clone(), vec![0.0; possible_moves.len()]);
            }
            self.policies.insert(key.clone(), policy);
        }
    }
    fn get_best_response_index(&self, key: &MSKey, infostate: &str) -> Option<usize> {
        // Returns index of highest value
        if let Some(policy) = self.policies.get(key) {
            if let Some(infostate_policy) = policy.get(infostate) {
                infostate_policy.iter().enumerate().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).map(|(index, _)| index)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn get_best_response(&self, key: &MSKey, infostate: &str) -> ActionObservation {
        let action_index: Option<usize> = self.get_best_response_index(key, infostate);
        if let Some(index) = action_index {
            if let Some(action_vec) = self.action_map.get(key){
                action_vec[index].clone()
            } else {
                ActionObservation::EmptyAO
            }
        } else {
            ActionObservation::EmptyAO
        }
    }
}