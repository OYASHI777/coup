use crate::history_public::{AOName, History, ActionObservation};
use super::best_response_policy::BestResponseIndVec;
use super::mixed_strategy_policy::MSInterface;
use super::keys::{BRKey, MSKey, MAX_NUM_BRKEY};
use std::collections::HashMap;

pub struct PolicyHandler;
// Updates policies for forward and backward passes

#[derive(PartialEq, Eq)]
enum ForwardPassType{
    Normal,
    Interactive,
    Chance,
    Shuffle,
    Reveal,
    Removal,
    Null,
}

impl PolicyHandler {
    // new
    // function to determine if normal case or what case for a node
    pub fn new() -> Self {
        PolicyHandler{}
    }    
    pub fn case_type(&self, latest_action_name: AOName) -> ForwardPassType {
        match latest_action_name {
            AOName::EmptyAO => ForwardPassType::Null,
            AOName::Income
            | AOName::Assassinate
            | AOName::BlockAssassinate
            | AOName::Steal
            | AOName::BlockSteal
            | AOName::Tax
            | AOName::Exchange
            | AOName::ForeignAid
            | AOName::Coup => ForwardPassType::Normal,
            AOName::CollectiveChallenge
            | AOName::CollectiveBlock => ForwardPassType::Interactive,
            AOName::ExchangeDraw => ForwardPassType::Chance, // This is where more infostates transition
            // To continue thinking if I should add and remove non dead infostates [what values would they initialise to]
            // transition preserves continuity more
            AOName::ExchangeChoice => ForwardPassType::Shuffle,
            AOName::Discard => ForwardPassType::Removal,
            AOName::RevealRedraw => ForwardPassType::Reveal,
        }
    }
}

pub trait PolicyUpdate {
    fn backward_pass_best_response_policy(&self, bri_vec: &mut BestResponseIndVec);
    fn forward_pass_best_response_policy(&self, path_time_t: &str, influence_time_t: &[u8; 6],history_time_t_1: History, bri_vec: &mut BestResponseIndVec, mix_strat_policy_time_t: Box<dyn MSInterface>);
    fn backward_pass_mixed_strategy_policy(&self);
    fn forward_pass_mixed_strategy_policy(&self);
}

impl PolicyUpdate for PolicyHandler {
    fn backward_pass_best_response_policy(&self, bri_vec: &mut BestResponseIndVec) {
        bri_vec.pop();
    }
    fn forward_pass_best_response_policy(&self, path_time_t: &str, influence_time_t: &[u8; 6],history_time_t_1: History, bri_vec: &mut BestResponseIndVec, mix_strat_policy_time_t: Box<dyn MSInterface>) {
        // time t is the node before the latest move is chosen
        // History should have been updated with new move
        // Then we forward pass from time t to t+1
        let chosen_move: &ActionObservation = history_time_t_1.latest_move();
        let chosen_name: AOName = chosen_move.name();
        // Rules for updating next set are
        // Interactive Cases: CollectiveChallenge, CollectiveBlock
        // Chance Cases: Need to manoever infostates around
        // Normal Cases: Need to determine next player 
        //      Find best_response of mix_strat
        //      clone it
        //      if best_response for infostate, keep at 1 else make 0
        let mut bri_indicator: HashMap<BRKey, bool> = bri_vec.get_last().unwrap().clone(); 
        if self.case_type(chosen_name) == ForwardPassType::Normal {
            let move_player: usize = chosen_move.player_id() as usize;
            for infostate in BestResponseIndVec::infostates_arr() {
                let key_ms: MSKey = MSKey::new(move_player, path_time_t);
                let key_bri: BRKey = BRKey::new(move_player, infostate.to_string());
                let infostate_best_response: ActionObservation = mix_strat_policy_time_t.get_best_response(&key_ms, infostate);
                if infostate_best_response == *chosen_move {
                    // maintain 1
                    bri_indicator.insert(key_bri, true);
                } else {
                    // change to 0
                    bri_indicator.insert(key_bri, false);
                }
            }
        } else {
            // [Placeholder]
            // default is to clone
        }
        bri_vec.push(bri_indicator);
    }
    fn backward_pass_mixed_strategy_policy(&self) {

    }
    fn forward_pass_mixed_strategy_policy(&self) {

    }
}