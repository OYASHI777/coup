use std::collections::HashMap;

use super::action_serialiser;
use super::keys::MSKey;

struct MixedStrategyPolicy {
    policies: HashMap<MSKey, Vec<i64>>,
}

impl MixedStrategyPolicy {
    pub fn new() -> Self {
        MixedStrategyPolicy {
            policies: HashMap::new(),
        }
    }
    pub fn get_best_response_index(&self, key: &MSKey) -> Option<usize> {
        // Returns index of highest value
        if let Some(policy) = self.policies.get(key) {
            policy.iter().enumerate().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).map(|(index, _)| index)
        } else {
            None
        }
    }
    pub fn update_policy(&mut self, key: &MSKey) {
        // updates based on some value function
    }
}