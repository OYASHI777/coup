use super::best_response_policy;
use super::mixed_strategy_policy;

struct PolicyHandler;
// Updates policies for forward and backward passes

impl PolicyHandler {

}

trait PolicyUpdate {
    fn backward_pass_best_response_policy(&self);
    fn forward_pass_best_response_policy(&self);
    fn backward_pass_mixed_strategy_policy(&self);
    fn forward_pass_mixed_strategy_policy(&self);
}