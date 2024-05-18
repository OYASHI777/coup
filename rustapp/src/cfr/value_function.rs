use std::collections::HashMap;
use super::keys::{BRKey, MAX_NUM_BRKEY};
struct HeuristicValueFunction;

trait ValueEvaluation {
    fn predict_value(&self) -> HashMap<BRKey, f32>;
    fn predict_value_batch(&self);
}

impl ValueEvaluation for HeuristicValueFunction {
    fn predict_value(&self) -> HashMap<BRKey, f32>{
        let output: HashMap<BRKey, f32> = HashMap::with_capacity(MAX_NUM_BRKEY);
        return output
    } 
    fn predict_value_batch(&self) {
        
    }
}