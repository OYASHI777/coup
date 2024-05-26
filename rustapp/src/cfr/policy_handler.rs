use crate::history_public::{AOName, History, ActionObservation};
use super::best_response_policy::{BestResponseIndVec, INFOSTATES};
use super::mixed_strategy_policy::{self, MSInterface};
use super::keys::{BRKey, MSKey, MAX_NUM_BRKEY};
use std::collections::HashMap;
use std::time::Instant;

pub struct PolicyHandler;
// Updates policies for forward and backward passes

#[derive(PartialEq, Eq)]
pub enum ForwardPassType{
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
            _ => {todo!()},
        }
    }
}

pub trait PolicyUpdate {
    // fn backward_pass_best_response_policy(&self, bri_vec: &mut BestResponseIndVec);
    // fn forward_pass_best_response_policy(&self, path_time_t: &str, influence_time_t: &[u8; 6],history_time_t_1: &History, bri_vec: &mut BestResponseIndVec, mix_strat_policy_time_t: Box<dyn MSInterface>);
    // fn backward_pass_mixed_strategy_policy(&self, mixed_strategy_policy: &mut Box<dyn MSInterface>, key: &MSKey, possible_moves: &Vec<ActionObservation>, reward: HashMap<BRKey, f32>);
    // fn forward_pass_mixed_strategy_policy(&self);
}

impl PolicyUpdate for PolicyHandler {
    // fn backward_pass_best_response_policy(&self, bri_vec: &mut BestResponseIndVec) {
    //     bri_vec.pop();
    // }
    // fn forward_pass_best_response_policy(&self, path_time_t: &str, influence_time_t: &[u8; 6], history_time_t_1: &History, bri_vec: &mut BestResponseIndVec, mix_strat_policy_time_t: Box<dyn MSInterface>) {
    //     // time t is the node before the latest move is chosen
    //     // History should have been updated with new move
    //     // Then we forward pass from time t to t+1
    //     let chosen_move: &ActionObservation = history_time_t_1.latest_move();
    //     let chosen_name: AOName = chosen_move.name();
    //     let move_player: usize = chosen_move.player_id() as usize;
    //     // Rules for updating next set are
    //     // Interactive Cases: CollectiveChallenge, CollectiveBlock
    //     // Chance Cases: Need to manoever infostates around
    //     // Normal Cases: Need to determine next player 
    //     //      Find best_response of mix_strat
    //     //      clone it
    //     //      if best_response for infostate, keep at 1 else make 0
    //     // TODO: Only legit infostates based on constraints_ t + 1 are passed 
        
    //     // Make a copy
    //     let mut bri_indicator: HashMap<BRKey, bool> = bri_vec.get_last().unwrap().clone(); 
    //     if self.case_type(chosen_name) == ForwardPassType::Normal {
    //         for infostate in BestResponseIndVec::infostates_arr() {
    //             let key_ms: MSKey = MSKey::new(move_player, path_time_t);
    //             let key_bri: BRKey = BRKey::new(move_player, infostate);
    //             let infostate_best_response: ActionObservation = mix_strat_policy_time_t.get_best_response(&key_ms, infostate);
    //             if infostate_best_response == *chosen_move {
    //                 // maintain 1
    //                 bri_indicator.insert(key_bri, true);
    //             } else {
    //                 // change to 0
    //                 bri_indicator.insert(key_bri, false);
    //             }
    //         }
    //     } else {
    //         // [Placeholder]
    //         // default is to clone
    //     }
    //     bri_vec.push(bri_indicator);
    // }
    // fn backward_pass_mixed_strategy_policy(&self, mixed_strategy_policy: &mut Box<dyn MSInterface>, key: &MSKey, possible_moves: &Vec<ActionObservation>, reward: HashMap<BRKey, f32>) {
    //     // updates mixed_strategy_policy aka Q values based on rewards
    //     mixed_strategy_policy.update(key, possible_moves, &reward);
    // }
    // fn forward_pass_mixed_strategy_policy(&self) {

    // }
}

pub trait Pruner {
    fn should_prune_current(&self, indicator: &HashMap<BRKey, bool>) -> bool;
    // fn player_0_others_1(&self, indicator: HashMap<BRKey, bool>) -> bool;
    fn player_all_0(&self, indicator: HashMap<BRKey, bool>, player_id: usize) -> bool;
}

impl Pruner for PolicyHandler {
    fn should_prune_current(&self, indicator: &HashMap<BRKey, bool>) -> bool {
        let start_time = Instant::now();
        // If some combination that is a valid gamestate exists, that sums to more than 0
        // return false
        // else return true
        let mut counter_hm: HashMap<char, usize> = HashMap::with_capacity(5);
        counter_hm.insert('A', 0);
        counter_hm.insert('B', 0);
        counter_hm.insert('C', 0);
        counter_hm.insert('D', 0);
        counter_hm.insert('E', 0);

        let mut counter_ind: usize = 0;

        let mut key: BRKey = BRKey::new(0, "");
        for infostate0 in INFOSTATES {
            // player 0
            key.set_player_id(0);
            key.set_infostate(infostate0);
            if let Some(ind_value) = indicator.get(&key) {
                if *ind_value {
                    let duration = start_time.elapsed();
                    println!("Time for prune check False: {:?}", duration);
                    return false;
                }
            } else {
                continue;
            }
            if infostate0.chars().nth(0).unwrap() == infostate0.chars().nth(1).unwrap() {
                if counter_hm[&infostate0.chars().nth(0).unwrap()] > 1 {
                    continue;
                } else {
                    counter_hm.insert(infostate0.chars().nth(0).unwrap(), counter_hm[&infostate0.chars().nth(0).unwrap()] + 2);
                }
            } else {
                if counter_hm[&infostate0.chars().nth(0).unwrap()] > 2 || counter_hm[&infostate0.chars().nth(1).unwrap()] > 2 {
                    continue;
                } else {
                    counter_hm.insert(infostate0.chars().nth(0).unwrap(), counter_hm[&infostate0.chars().nth(0).unwrap()] +1);
                    counter_hm.insert(infostate0.chars().nth(1).unwrap(), counter_hm[&infostate0.chars().nth(1).unwrap()] + 1);
                }
            }
            
            // Check if adding to counter_hm exceeds count, break if true
            for infostate1 in INFOSTATES {
                key.set_player_id(1);
                key.set_infostate(infostate1);
                if let Some(ind_value) = indicator.get(&key) {
                    if *ind_value {
                        let duration = start_time.elapsed();
                        println!("Time for prune check False: {:?}", duration);
                        return false;
                    }
                } else {
                    continue;
                }
                if infostate1.chars().nth(0).unwrap() == infostate1.chars().nth(1).unwrap() {
                    if counter_hm[&infostate1.chars().nth(0).unwrap()] > 1 {
                        continue;
                    } else {
                        counter_hm.insert(infostate1.chars().nth(0).unwrap(), counter_hm[&infostate1.chars().nth(0).unwrap()] + 2);
                    }
                } else {
                    if counter_hm[&infostate1.chars().nth(0).unwrap()] > 2 || counter_hm[&infostate1.chars().nth(1).unwrap()] > 2 {
                        continue;
                    } else {
                        counter_hm.insert(infostate1.chars().nth(0).unwrap(), counter_hm[&infostate1.chars().nth(0).unwrap()] +1);
                        counter_hm.insert(infostate1.chars().nth(1).unwrap(), counter_hm[&infostate1.chars().nth(1).unwrap()] + 1);
                    }
                }
                for infostate2 in INFOSTATES {
                    key.set_player_id(2);
                    key.set_infostate(infostate2);
                    if let Some(ind_value) = indicator.get(&key) {
                        if *ind_value {
                            let duration = start_time.elapsed();
                            println!("Time for prune check False: {:?}", duration);
                            return false;
                        }
                    } else {
                        continue;
                    }
                    if infostate2.chars().nth(0).unwrap() == infostate2.chars().nth(1).unwrap() {
                        if counter_hm[&infostate2.chars().nth(0).unwrap()] > 1 {
                            continue;
                        } else {
                            counter_hm.insert(infostate2.chars().nth(0).unwrap(), counter_hm[&infostate2.chars().nth(0).unwrap()] + 2);
                        }
                    } else {
                        if counter_hm[&infostate2.chars().nth(0).unwrap()] > 2 || counter_hm[&infostate2.chars().nth(1).unwrap()] > 2 {
                            continue;
                        } else {
                            counter_hm.insert(infostate2.chars().nth(0).unwrap(), counter_hm[&infostate2.chars().nth(0).unwrap()] +1);
                            counter_hm.insert(infostate2.chars().nth(1).unwrap(), counter_hm[&infostate2.chars().nth(1).unwrap()] + 1);
                        }
                    }
                    for infostate3 in INFOSTATES {
                        key.set_player_id(3);
                        key.set_infostate(infostate3);
                        if let Some(ind_value) = indicator.get(&key) {
                            if *ind_value {
                                let duration = start_time.elapsed();
                                println!("Time for prune check False: {:?}", duration);
                                return false;
                            }
                        } else {
                            continue;
                        }
                        if infostate3.chars().nth(0).unwrap() == infostate3.chars().nth(1).unwrap() {
                            if counter_hm[&infostate3.chars().nth(0).unwrap()] > 1 {
                                continue;
                            } else {
                                counter_hm.insert(infostate3.chars().nth(0).unwrap(), counter_hm[&infostate3.chars().nth(0).unwrap()] + 2);
                            }
                        } else {
                            if counter_hm[&infostate3.chars().nth(0).unwrap()] > 2 || counter_hm[&infostate3.chars().nth(1).unwrap()] > 2 {
                                continue;
                            } else {
                                counter_hm.insert(infostate3.chars().nth(0).unwrap(), counter_hm[&infostate3.chars().nth(0).unwrap()] +1);
                                counter_hm.insert(infostate3.chars().nth(1).unwrap(), counter_hm[&infostate3.chars().nth(1).unwrap()] + 1);
                            }
                        }
                        for infostate4 in INFOSTATES {
                            key.set_player_id(4);
                            key.set_infostate(infostate4);
                            if let Some(ind_value) = indicator.get(&key) {
                                if *ind_value {
                                    let duration = start_time.elapsed();
                                    println!("Time for prune check False: {:?}", duration);
                                    return false;
                                }
                            } else {
                                continue;
                            }
                            if infostate4.chars().nth(0).unwrap() == infostate4.chars().nth(1).unwrap() {
                                if counter_hm[&infostate4.chars().nth(0).unwrap()] > 1 {
                                    continue;
                                } else {
                                    counter_hm.insert(infostate4.chars().nth(0).unwrap(), counter_hm[&infostate4.chars().nth(0).unwrap()] + 2);
                                }
                            } else {
                                if counter_hm[&infostate4.chars().nth(0).unwrap()] > 2 || counter_hm[&infostate4.chars().nth(1).unwrap()] > 2 {
                                    continue;
                                } else {
                                    counter_hm.insert(infostate4.chars().nth(0).unwrap(), counter_hm[&infostate4.chars().nth(0).unwrap()] +1);
                                    counter_hm.insert(infostate4.chars().nth(1).unwrap(), counter_hm[&infostate4.chars().nth(1).unwrap()] + 1);
                                }
                            }
                            for infostate5 in INFOSTATES {
                                key.set_player_id(5);
                                key.set_infostate(infostate5);
                                if let Some(ind_value) = indicator.get(&key) {
                                    if *ind_value {
                                        let duration = start_time.elapsed();
                                        println!("Time for prune check False: {:?}", duration);
                                        return false;
                                    }
                                } else {
                                    continue;
                                }
                                if infostate5.chars().nth(0).unwrap() == infostate5.chars().nth(1).unwrap() {
                                    if counter_hm[&infostate5.chars().nth(0).unwrap()] > 1 {
                                        continue;
                                    } else {
                                        counter_hm.insert(infostate5.chars().nth(0).unwrap(), counter_hm[&infostate5.chars().nth(0).unwrap()] + 2);
                                    }
                                } else {
                                    if counter_hm[&infostate5.chars().nth(0).unwrap()] > 2 || counter_hm[&infostate5.chars().nth(1).unwrap()] > 2 {
                                        continue;
                                    } else {
                                        counter_hm.insert(infostate5.chars().nth(0).unwrap(), counter_hm[&infostate5.chars().nth(0).unwrap()] +1);
                                        counter_hm.insert(infostate5.chars().nth(1).unwrap(), counter_hm[&infostate5.chars().nth(1).unwrap()] + 1);
                                    }
                                }


                                // Player 5 exit
                                if infostate5.chars().nth(0).unwrap() == infostate5.chars().nth(1).unwrap() {
                                    counter_hm.insert(infostate5.chars().nth(0).unwrap(), counter_hm[&infostate5.chars().nth(0).unwrap()] - 2);
                                } else {
                                    counter_hm.insert(infostate5.chars().nth(0).unwrap(), counter_hm[&infostate5.chars().nth(0).unwrap()] - 1);
                                    counter_hm.insert(infostate5.chars().nth(1).unwrap(), counter_hm[&infostate5.chars().nth(1).unwrap()] - 1);
                                }
                            }
                            
                            // Player 4 exit
                            if infostate4.chars().nth(0).unwrap() == infostate4.chars().nth(1).unwrap() {
                                counter_hm.insert(infostate4.chars().nth(0).unwrap(), counter_hm[&infostate4.chars().nth(0).unwrap()] - 2);
                            } else {
                                counter_hm.insert(infostate4.chars().nth(0).unwrap(), counter_hm[&infostate4.chars().nth(0).unwrap()] - 1);
                                counter_hm.insert(infostate4.chars().nth(1).unwrap(), counter_hm[&infostate4.chars().nth(1).unwrap()] - 1);
                            }
                        }
                        // Player 3 exit
                        if infostate3.chars().nth(0).unwrap() == infostate3.chars().nth(1).unwrap() {
                            counter_hm.insert(infostate3.chars().nth(0).unwrap(), counter_hm[&infostate3.chars().nth(0).unwrap()] - 2);
                        } else {
                            counter_hm.insert(infostate3.chars().nth(0).unwrap(), counter_hm[&infostate3.chars().nth(0).unwrap()] - 1);
                            counter_hm.insert(infostate3.chars().nth(1).unwrap(), counter_hm[&infostate3.chars().nth(1).unwrap()] - 1);
                        }
                        
                    }
                    // Player 2 exit
                    if infostate2.chars().nth(0).unwrap() == infostate2.chars().nth(1).unwrap() {
                        counter_hm.insert(infostate2.chars().nth(0).unwrap(), counter_hm[&infostate2.chars().nth(0).unwrap()] - 2);
                    } else {
                        counter_hm.insert(infostate2.chars().nth(0).unwrap(), counter_hm[&infostate2.chars().nth(0).unwrap()] - 1);
                        counter_hm.insert(infostate2.chars().nth(1).unwrap(), counter_hm[&infostate2.chars().nth(1).unwrap()] - 1);
                    }
                    
                }
                // Player 1 exit
                if infostate1.chars().nth(0).unwrap() == infostate1.chars().nth(1).unwrap() {
                    counter_hm.insert(infostate1.chars().nth(0).unwrap(), counter_hm[&infostate1.chars().nth(0).unwrap()] - 2);
                } else {
                    counter_hm.insert(infostate1.chars().nth(0).unwrap(), counter_hm[&infostate1.chars().nth(0).unwrap()] - 1);
                    counter_hm.insert(infostate1.chars().nth(1).unwrap(), counter_hm[&infostate1.chars().nth(1).unwrap()] - 1);
                }
                
            }
            // Player 0 exit
            if infostate0.chars().nth(0).unwrap() == infostate0.chars().nth(1).unwrap() {
                counter_hm.insert(infostate0.chars().nth(0).unwrap(), counter_hm[&infostate0.chars().nth(0).unwrap()] - 2);
            } else {
                counter_hm.insert(infostate0.chars().nth(0).unwrap(), counter_hm[&infostate0.chars().nth(0).unwrap()] - 1);
                counter_hm.insert(infostate0.chars().nth(1).unwrap(), counter_hm[&infostate0.chars().nth(1).unwrap()] - 1);
            }
        }
        let duration = start_time.elapsed();
        println!("Time for prune check: {:?}", duration);
        true
    }
    fn player_all_0(&self, indicator: HashMap<BRKey, bool>, player_id: usize) -> bool {
        for infostate in INFOSTATES {
            let key_br: BRKey = BRKey::new(player_id, infostate);
            if let Some(value) = indicator.get(&key_br) {
                if *value {
                    return false;
                }
            }
        }
        true
    }
}