use std::collections::HashMap;
use super::keys::{BRKey, MAX_NUM_BRKEY};
use crate::prob_manager::constraint::CollectiveConstraint;
pub struct BestResponseIndVec {
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
pub const INFOSTATES: [&str; 15] = ["AA", "AB", "AC", "AD", "AE", "BB", "BC", "BD", "BE", "CC", "CD", "CE", "DD", "DE", "EE"];
impl BestResponseIndVec {
    pub fn create_root(constraint: &CollectiveConstraint, max_size: usize) -> Self {
        // [Placeholder]
        // check all infostate to see if player can have those cards
        let mut root_hm: HashMap<BRKey, bool> = HashMap::with_capacity(MAX_NUM_BRKEY);
        for player_id in 0..6 {
            for infostate in INFOSTATES {
                if constraint.player_can_have_active_cards_str(player_id, infostate) {
                    let key: BRKey = BRKey::new(player_id, infostate.to_string());
                    root_hm.insert(key, true);
                }
            }
        }
        let mut policies = Vec::with_capacity(max_size);
        policies.push(root_hm);
        BestResponseIndVec { policies }
    }
    pub fn infostates_arr() -> [&'static str; 15] {
        INFOSTATES
    }
    pub fn push(&mut self, hm: HashMap<BRKey, bool>) {

    }
    pub fn pop(&mut self) {
        self.policies.pop();
    }
    pub fn len(&self) -> usize{
        self.policies.len()
    }
    pub fn get_last(&self) -> Option<&HashMap<BRKey, bool>>{
        if self.len() == 0 {
            // But it should always be at least 1 because the root node is the first!
            None
        } else {
            Some(&self.policies[self.len() - 1])
        }
    }
    pub fn with_capacity(max_size: usize) -> Self {
        let mut policies = Vec::with_capacity(max_size);
        // for _ in 0..max_size {
        //     // 90 because 15 states * 6 players = 90 required.
        //     policies.push(HashMap::with_capacity(MAX_NUM_BRKEY));
        // }
        BestResponseIndVec { policies }
    }
    pub fn get_grid(&self, game_turn_no: usize) -> Option<&HashMap<BRKey, bool>> {
        self.policies.get(game_turn_no)
    }
    pub fn get_indicator(&self, game_turn_no: usize, key: &BRKey) -> Option<bool> {
        self.policies.get(game_turn_no)?.get(key).copied()
    }
    pub fn set_grid(&mut self, game_turn_no: usize, grid: HashMap<BRKey, bool>) {
        if let Some(policy_map) = self.policies.get_mut(game_turn_no) {
            *policy_map = grid; 
        }
    }
    pub fn set_indicator(&mut self, game_turn_no: usize, key: BRKey, indicator: bool) {
        if let Some(policy_map) = self.policies.get_mut(game_turn_no) {
            policy_map.insert(key, indicator);
        }
    }
    pub fn clear_indicator(&mut self, game_turn_no: usize, key: BRKey) {
        if let Some(policy_map) = self.policies.get_mut(game_turn_no) {
            policy_map.remove(&key);
        }
    }
    pub fn clear_grid(&mut self, game_turn_no: usize) {
        if let Some(policy_map) = self.policies.get_mut(game_turn_no) {
            policy_map.clear();
        }
    }
    pub fn load_from_mixed_strat_policy(&self){
        // load t + 1 grid from grid(t) and mixed_strat(t->t+1)
        // Take note of impossible to reach => None
        // And possible to reach but players wont play towards in under a BR policy => 0 (false)
    }
}
