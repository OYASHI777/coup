use std::collections::HashMap;
use super::keys::{BRKey, MAX_NUM_BRKEY};

struct BestResponseIndVec {
    // === BRKey ===
    //          Player
    //     0  1  2  3  4  5
    // AA [0, 1, 1, 0, 0, 1]
    // AB [1, 0, 1, 0, 1, 1]
    // AC [0, 1, 1, 0, 0, 0]
    // BB [1, 1, 0, 0, 1, 1]
    // BC [0, 0, 1, 0, 0, 0]
    // CC [1, 0, 1, 1, 0, 1]
    policies: Vec<HashMap<BRKey, bool>>
}

impl BestResponseIndVec {
    fn with_capacity(max_size: usize) -> Self {
        let mut policies = Vec::with_capacity(max_size);
        for _ in 0..max_size {
            // 90 because 15 states * 6 players = 90 required.
            policies.push(HashMap::with_capacity(MAX_NUM_BRKEY));
        }
        BestResponseIndVec { policies }
    }
    fn get_grid(&self, game_turn_no: usize) -> Option<&HashMap<BRKey, bool>> {
        self.policies.get(game_turn_no)
    }
    fn get_indicator(&self, game_turn_no: usize, key: &BRKey) -> Option<bool> {
        self.policies.get(game_turn_no)?.get(key).copied()
    }
    fn set_grid(&mut self, game_turn_no: usize, grid: HashMap<BRKey, bool>) {
        if let Some(policy_map) = self.policies.get_mut(game_turn_no) {
            *policy_map = grid; 
        }
    }
    fn set_indicator(&mut self, game_turn_no: usize, key: BRKey, indicator: bool) {
        if let Some(policy_map) = self.policies.get_mut(game_turn_no) {
            policy_map.insert(key, indicator);
        }
    }
    fn clear_indicator(&mut self, game_turn_no: usize, key: BRKey) {
        if let Some(policy_map) = self.policies.get_mut(game_turn_no) {
            policy_map.remove(&key);
        }
    }
    fn clear_grid(&mut self, game_turn_no: usize) {
        if let Some(policy_map) = self.policies.get_mut(game_turn_no) {
            policy_map.clear();
        }
    }
    fn load_from_mixed_strat_policy(&self){
        // load t + 1 grid from grid(t) and mixed_strat(t->t+1)
        // Take note of impossible to reach => None
        // And possible to reach but players wont play towards in under a BR policy => 0 (false)
    }
}
