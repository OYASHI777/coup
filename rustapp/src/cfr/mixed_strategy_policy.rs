use std::collections::HashMap;

use rustapp::history_public::ActionObservation;

use super::action_serialiser;
use super::keys::MSKey;

struct HeuristicMixedStrategyPolicy {
    policies: HashMap<MSKey, Vec<f32>>,
    action_map: HashMap<MSKey, Vec<ActionObservation>>,
}
trait MSInterface {
    fn new() -> Self;
    fn update(&mut self, key: &MSKey, possible_moves: &Vec<ActionObservation>, values: &Vec<f32>);
    fn get_best_response_index(&self, key: &MSKey) -> Option<usize>;
    fn get_best_response(&self, key: &MSKey) -> Option<ActionObservation>;
}

impl HeuristicMixedStrategyPolicy {
    pub fn is_key_in_action_map(&self, key: &MSKey) -> bool {
        if let Some(_) = self.action_map.get(key) {
            true
        } else {
            false
        }
    }
    pub fn update_action_map(&mut self, key: &MSKey, possible_moves: &Vec<ActionObservation>) {
        if !self.is_key_in_action_map(key) {
            self.action_map.insert(key.clone(), possible_moves.to_vec());
        }
    }
}

impl MSInterface for HeuristicMixedStrategyPolicy {
    fn new() -> Self {
        HeuristicMixedStrategyPolicy {
            policies: HashMap::new(),
            action_map: HashMap::new(),
        }
    }
    fn update(&mut self, key: &MSKey, possible_moves: &Vec<ActionObservation>, values: &Vec<f32>) {
        self.update_action_map(key, possible_moves);
        if let Some(policy) = self.policies.get_mut(key) {
            assert!(values.len() == policy.len(), "policy must match shape of values!");
            let mut i: usize = 0;
            while i < values.len() {
                // Updating by summing, without averaging
                // In our case, the only actual mix strat we care about is the root node for self play
                // so we do not average this!
                policy[i] += values[i];
            }
        } else {
            self.policies.insert(key.clone(), vec![0.0; values.len()]);
        }
    }
    fn get_best_response_index(&self, key: &MSKey) -> Option<usize> {
        // Returns index of highest value
        if let Some(policy) = self.policies.get(key) {
            policy.iter().enumerate().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).map(|(index, _)| index)
        } else {
            None
        }
    }
    fn get_best_response(&self, key: &MSKey) -> Option<ActionObservation> {
        if let Some(action_vec) = self.action_map.get(key){
            let mut action_index: Option<usize> = self.get_best_response_index(key);
            if let Some(index) = action_index {
                Some(action_vec[index])
            } else {
                None
            }
        } else {
            None
        }
    }
}