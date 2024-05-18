use std::collections::HashMap;
use super::keys::{BRKey, MAX_NUM_BRKEY, INFOSTATES};
struct HeuristicValueFunction;

trait ValueEvaluation<T> {
    fn predict_value(&self, input: &Vec<T>) -> HashMap<BRKey, f32>;
    fn predict_value_batch(&self);
}

impl ValueEvaluation<u8> for HeuristicValueFunction {
    fn predict_value(&self, input: &Vec<u8>) -> HashMap<BRKey, f32>{
        // take some input
        let mut output: HashMap<BRKey, f32> = HashMap::with_capacity(MAX_NUM_BRKEY);
        // For Heuristic, if a player is dead 0, else 1 is split among remaining players
        // input: influence vector
        let no_alive: usize = input.iter().filter(|&x| *x > 0 as u8).count();
        let value_survival: f32 = 1.0 / no_alive as f32;
        let mut player_value: f32;
        for player_id in 0..6{
            if input[player_id] == 0 {
                player_value = 0.0;
            } else {
                player_value = value_survival;
            }
            for infostate in INFOSTATES {
                output.insert(BRKey::new(player_id, infostate.to_string()), player_value);
            }
        }
        return output
    } 
    fn predict_value_batch(&self) {
        
    }
}