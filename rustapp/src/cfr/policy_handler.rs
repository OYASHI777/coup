use super::best_response_policy;
use super::mixed_strategy_policy;
use super::keys::{BRKey, MSKey};
use std::collections::HashMap;

pub struct PolicyHandler;
// Updates policies for forward and backward passes

pub trait PolicyUpdate {
    fn backward_pass_best_response_policy(&self, old_best_response_indicators: &HashMap<BRKey, bool>, mixed_strategy_policy: &HashMap<MSKey, Vec<i64>>);
    fn forward_pass_best_response_policy(&self);
    fn backward_pass_mixed_strategy_policy(&self);
    fn forward_pass_mixed_strategy_policy(&self);
}

impl PolicyUpdate for PolicyHandler {
    fn backward_pass_best_response_policy(&self, old_best_response_indicators: &HashMap<BRKey, bool>, mixed_strategy_policy: &HashMap<MSKey, Vec<i64>>) {
        
    }
    fn forward_pass_best_response_policy(&self) {

    }
    fn backward_pass_mixed_strategy_policy(&self) {

    }
    fn forward_pass_mixed_strategy_policy(&self) {

    }
}