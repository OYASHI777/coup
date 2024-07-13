use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use tokio::time::error::Elapsed;
use crate::prob_manager::naive_prob::NaiveProb;
use crate::prob_manager::naive_sampler::NaiveSampler;
use crate::history_public::{AOName, ActionObservation, Card, History};
use crate::prob_manager::constraint::CollectiveConstraint;
use super::action_serialiser::{DefaultEmbedding, ActionEmbedding};
use super::best_response_policy::BestResponseIndVec;
use super::mixed_strategy_policy::{HeuristicMixedStrategyPolicy, MSInterface};
use super::policy_handler::{PolicyHandler, Pruner};
use super::value_function::{HeuristicValueFunction, ValueEvaluation};
use super::keys::{BRKey, MSKey, INFOSTATES, MAX_NUM_BRKEY, Infostate};
use super::reach_prob::ReachProb;
// use std::collections::hash_map::RandomState;
use std::time::Instant;
use ahash::AHashMap;
// use super::best_response_policy::BRKey;

use std::collections::HashMap;
struct Explorer<'a> {
    // This is a struct used to conduct Pure Monte Carlo CounterFactual Regret Minimization (PMCCFR)
    start_player: usize,
    path: String,
    action_embedder: Box<dyn ActionEmbedding>,
    history: History,
    best_response_policy_vec: BestResponseIndVec, //Effectively obsolete
    mixed_strategy_policy_vec: Box<dyn MSInterface>,
    q_values: Box<dyn MSInterface>,
    rewards_evaluator: Box<dyn ValueEvaluation<u8>>,
    prob: NaiveProb<'a>,
    chance_sampler: NaiveSampler<'a>,
    policy_handler: PolicyHandler,
    max_depth: usize,
    bool_monte_carlo: bool,
    node_counter: u128,
    visit_counter: u128,
    prune_counter: u128,
    won_counter: u128,
    // TO Document best_response_policy
}

impl <'a> Explorer<'a> {
    pub fn new(max_depth: usize) -> Self {
        // make this into config HashMap when done
        Explorer{
            // temporarily the starting_player is 0
            start_player: 0,
            path: "root".to_string(),
            action_embedder: Box::new(DefaultEmbedding),
            history: History::new(0),
            best_response_policy_vec: BestResponseIndVec::new(max_depth + 1),
            mixed_strategy_policy_vec: Box::new(HeuristicMixedStrategyPolicy::new()),
            q_values: Box::new(HeuristicMixedStrategyPolicy::new()),
            rewards_evaluator: Box::new(HeuristicValueFunction),
            prob: NaiveProb::new(),
            chance_sampler: NaiveSampler::new(),
            policy_handler: PolicyHandler::new(),
            max_depth,
            bool_monte_carlo: true,
            node_counter: 0,
            visit_counter: 0,
            prune_counter: 0,
            won_counter: 0,
            // best_response_policy: HashMap::new(),
        }
    }

    pub fn load(&mut self) {
        // loads in history
        // Initialises proper root for best_response_policy_vec
    }

    pub fn reset(&mut self) {
        self.path = "root".to_string();
        self.history.reset();
        self.best_response_policy_vec.reset();
        self.prob.reset();
        self.q_values.reset();
        self.mixed_strategy_policy_vec.reset();
        self.node_counter = 0;
        self.visit_counter = 0;
        self.prune_counter = 0;
        self.won_counter = 0;
    }

    pub fn get_root_policy(&self) -> Option<& AHashMap<BRKey, Vec<f32>>>{
        let key: MSKey = MSKey::new(self.start_player as u8, "root");
        self.mixed_strategy_policy_vec.policy_get(&key)
    }
    
    pub fn get_root_action_map(&self) -> Option<&Vec<ActionObservation>>{
        let key: MSKey = MSKey::new(self.start_player as u8, "root");
        self.mixed_strategy_policy_vec.action_map_get(&key)
    }

    pub fn reset_counters(&mut self) {
        // Temp
        self.path = "root".to_string();
        self.history.reset();
        self.prob.reset();
        self.node_counter = 0;
        self.visit_counter = 0;
        self.prune_counter = 0;
        self.won_counter = 0;
    }

    pub fn print(&self) {
        self.history.print_history();
    }

    pub fn set_max_depth(&mut self, max_depth: usize) {
        self.max_depth = max_depth;
    }

    pub fn add_node(&mut self, action: ActionObservation, bool_know_priv_info: bool) {
        // false if adding to past history
        // true if simulating future history
        self.history.push_ao(action);
        self.prob.push_ao(&action, bool_know_priv_info);
        self.path = self.action_embedder.add_action(&self.path, &action);
    }

    pub fn drop_node(&mut self) {
        self.history.remove_ao();
        self.prob.pop();
        self.path = self.action_embedder.remove_action(&self.path);
        // TODO: add dropping of latest rewards in the store
    }

    pub fn is_chance_node(&self) -> bool {
        // Next move is CollectiveChallenge, CollectiveBlock or ExchangeDraw
        // Refer to history_public generate_legal_moves
        let action: &ActionObservation = self.history.latest_move();
        match action.name() {
            AOName::EmptyAO 
            | AOName::Income 
            | AOName::Coup 
            | AOName::ExchangeDraw
            | AOName::ExchangeChoice 
            | AOName::RevealRedraw => false,
            AOName::ForeignAid
            | AOName::Tax
            | AOName::Steal
            | AOName::Exchange
            | AOName::Assassinate => {
                return true;
            },
            AOName::BlockSteal
            | AOName::BlockAssassinate => {
                if self.history.store_at(self.history.store_len() - 1).player_id() == self.history.store_at(self.history.store_len() - 1).opposing_player_id() {
                    false
                } else {
                    true
                }
            },
            AOName::CollectiveChallenge => {
                if self.history.store_at(self.history.store_len() - self.history.dist_from_turn()).name() == AOName::Exchange {
                    // ExchangeDraw
                    true
                } else {
                    false
                }
            },
            AOName::Discard => {
                if self.history.store_at(self.history.store_len() - self.history.dist_from_turn()).name() == AOName::Exchange &&
                self.history.store_at(self.history.store_len() - 2).name() == AOName::RevealRedraw {
                    // ExchangeDraw
                    true
                } else {
                    false
                }
            },
            AOName::CollectiveBlock => {
                if self.history.store_at(self.history.store_len() - 1).final_actioner() == self.history.store_at(self.history.store_len() - 1).opposing_player_id() {
                    false
                } else {
                    true
                }
            },
            _ => {panic!("not implemented")},
        }
    }
    pub fn is_exchangedraw_node(&self) -> bool {
        // Next Move is ExchangeDraw
        // Refer to history_public generate_legal_moves
        let action: &ActionObservation = self.history.latest_move();
        match action.name() {
            AOName::EmptyAO 
            | AOName::Income 
            | AOName::Coup 
            | AOName::ExchangeDraw
            | AOName::ExchangeChoice 
            | AOName::RevealRedraw => false,
            AOName::ForeignAid
            | AOName::Tax
            | AOName::Steal
            | AOName::Exchange
            | AOName::Assassinate => {
                return false;
            },
            AOName::BlockSteal
            | AOName::BlockAssassinate => {
                false
            },
            AOName::CollectiveChallenge => {
                if self.history.store_at(self.history.store_len() - self.history.dist_from_turn()).name() == AOName::Exchange {
                    // ExchangeDraw
                    true
                } else {
                    false
                }
            },
            AOName::Discard => {
                if self.history.store_at(self.history.store_len() - self.history.dist_from_turn()).name() == AOName::Exchange &&
                self.history.store_at(self.history.store_len() - 2).name() == AOName::RevealRedraw {
                    // ExchangeDraw
                    true
                } else {
                    false
                }
            },
            AOName::CollectiveBlock => {
                false
            },
            _ => {panic!("not implemented")},
        }
    }
    pub fn is_collective_node(&self) -> bool {
        // Next move is CollectiveChallenge or CollectiveBlock
        // Refer to history_public generate_legal_moves
        let action: &ActionObservation = self.history.latest_move();
        match action.name() {
            AOName::EmptyAO 
            | AOName::Income 
            | AOName::Coup 
            | AOName::ExchangeDraw
            | AOName::ExchangeChoice 
            | AOName::RevealRedraw => false,
            AOName::ForeignAid
            | AOName::Tax
            | AOName::Steal
            | AOName::Exchange
            | AOName::Assassinate => {
                return true;
            },
            AOName::BlockSteal
            | AOName::BlockAssassinate => {
                if self.history.store_at(self.history.store_len() - 1).player_id() == self.history.store_at(self.history.store_len() - 1).opposing_player_id() {
                    false
                } else {
                    true
                }
            },
            AOName::CollectiveChallenge => {
                false
            },
            AOName::Discard => {
                false
            },
            AOName::CollectiveBlock => {
                if self.history.store_at(self.history.store_len() - 1).final_actioner() == self.history.store_at(self.history.store_len() - 1).opposing_player_id() {
                    false
                } else {
                    true
                }
            },
            _ => {panic!("not implemented")},
        }
    }

    pub fn chance_node_sample_size(&self) -> usize {
        debug_assert!(self.is_chance_node(), "Not a Chance Node!");
        // TODO: Make more advanced based on players remaining
        1
    }

    pub fn generate_chance_outcome(&self, possible_outcomes: &Vec<ActionObservation>) -> ActionObservation {
        // TEMP: Not Policy dependent for now
        self.history.print_history();
        println!("Generated moves: {:?}", possible_outcomes);
        if let Some(sample_outcome) = possible_outcomes.choose(& mut thread_rng()) {
            return *sample_outcome;
        } else {
            panic!("No random move can be generate if no moves are provided!");
        }
    }

    pub fn set_depth(&mut self, new_depth: usize){
        self.max_depth = new_depth;
    }

    pub fn explore(&mut self) {
        if self.bool_monte_carlo{
            self.explore_recurse(0);
        } else {
            self.explore_recurse_mc(0);
        }
    }

    pub fn update_depth(&self) {

    }
    pub fn nodes_reached(&self) -> u128 {
        self.node_counter
    }
    pub fn nodes_traversed(&self) -> u128 {
        self.visit_counter
    }
    pub fn nodes_pruned(&self) -> u128 {
        self.prune_counter
    }
    pub fn nodes_won(&self) -> u128 {
        self.won_counter
    }
    pub fn naive_prune(&self, action: &ActionObservation) -> bool {
        // can player play this card?
        let latest_constraint: CollectiveConstraint = self.prob.latest_constraint();
        if action.name() == AOName::Discard {
            if action.no_cards() == 1 {
                if latest_constraint.player_can_have_active_card(action.player_id(), &action.cards()[0]) {
                    false
                } else {
                    true
                }
            } else if action.no_cards() == 2 {
                if latest_constraint.player_can_have_active_cards(action.player_id(), &action.cards()) {
                    false
                } else {
                    true
                }
            } else {
                false
            }
        } else if action.name() == AOName::RevealRedraw {
            if latest_constraint.player_can_have_active_card(action.player_id(), &action.card()) {
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn naive_sample_exchange_draw(&mut self, player_id: usize) -> Option<ActionObservation> {
        // Randomly generates a string to produce a sampled exchangedraw
        let latest_constraint: CollectiveConstraint = self.prob.latest_constraint();
        if let Some(state_string) = self.chance_sampler.par_constructor(&latest_constraint){
            let mut rng = thread_rng();
            let pile_indices:[usize; 3] = [12, 13, 14];
            let sampled_indices: Vec<usize> = pile_indices.choose_multiple(&mut rng, 2).cloned().collect();
            assert_eq!(sampled_indices.len(), 2, "Sampled indices should be exactly 2");
            let sampled_char: [char; 2] = [state_string.chars().nth(sampled_indices[0]).unwrap(), state_string.chars().nth(sampled_indices[1]).unwrap()];
            let cards: [Card; 2] = [Card::char_to_card(sampled_char[0]),Card::char_to_card(sampled_char[1])];
            let sampled_action: ActionObservation = ActionObservation::ExchangeDraw { player_id: player_id, card: cards };
            Some(sampled_action)
        } else {
            None
        }
    }

    pub fn explore_recurse(&mut self, depth_counter: usize) {
        self.visit_counter += 1;
        if depth_counter >= self.max_depth {
            self.node_counter += 1;
            self.update_depth();
        } else if self.history.game_won(){
            // game won
            self.node_counter += 1;
            self.update_depth();
        } else {
            // Always false for simulations
            let bool_know_priv_info: bool = false; 
            
            let possible_outcomes: Vec<ActionObservation> = self.history.generate_legal_moves();
            for action in possible_outcomes {
                self.add_node(action, bool_know_priv_info);
                self.explore_recurse(depth_counter + 1);
                // TODO: backpropogate
                self.drop_node();
            }
        }
    }

    pub fn explore_recurse_naive_prune(&mut self, depth_counter: usize) {
        self.visit_counter += 1;
        if depth_counter >= self.max_depth {
            self.node_counter += 1;
            self.update_depth();
        } else if self.history.game_won(){
            // game won
            self.node_counter += 1;
            self.update_depth();
        } else {
            // Always false for simulations
            let bool_know_priv_info: bool = false; 
            
            let possible_outcomes: Vec<ActionObservation> = self.history.generate_legal_moves();
            for action in possible_outcomes {
                if action.name() == AOName::ExchangeDraw {
                    if self.prob.player_can_have_cards_constructor(action.player_id(), action.cards()) {
                        self.add_node(action, bool_know_priv_info);
                        self.explore_recurse_naive_prune(depth_counter + 1);
                        // TODO: backpropogate
                        self.drop_node();
                    } else {
                        continue;
                    }
                } else if !self.naive_prune(&action) {
                    self.add_node(action, bool_know_priv_info);
                    self.explore_recurse_naive_prune(depth_counter + 1);
                    // TODO: backpropogate
                    self.drop_node();
                } else {
                    self.node_counter += 1;
                }
            }
        }
    }
    pub fn explore_recurse_mc(&mut self, depth_counter: usize) {
   
        self.visit_counter += 1;
        if depth_counter >= self.max_depth {
            self.node_counter += 1;
            self.update_depth();
        } else if self.history.game_won(){
            // game won
            self.node_counter += 1;
            self.update_depth();
        } else {
            // Always false for simulations
            let bool_know_priv_info: bool = false; 
            let possible_outcomes: Vec<ActionObservation> = self.history.generate_legal_moves();
            if self.is_chance_node() {
                if let Some(action) = possible_outcomes.choose(&mut thread_rng()).cloned() {
                    self.add_node(action, bool_know_priv_info);
                    self.explore_recurse_mc(depth_counter + 1);
                    // TODO: backpropogate
                    self.drop_node();
                }
            } else {
                // Continue as per without mc
                for action in possible_outcomes {
                    self.add_node(action, bool_know_priv_info);
                    self.explore_recurse_mc(depth_counter + 1);
                    // TODO: backpropogate
                    self.drop_node();
                }
            }
        }
    }

    pub fn explore_recurse_mc_naive_prune(&mut self, depth_counter: usize) {
        self.visit_counter += 1;
        if depth_counter >= self.max_depth {
            self.node_counter += 1;
            self.update_depth();
        } else if self.history.game_won(){
            // game won
            self.node_counter += 1;
            self.update_depth();
        } else {
            // Always false for simulations
            let bool_know_priv_info: bool = false; 
            let possible_outcomes: Vec<ActionObservation> = self.history.generate_legal_moves();
            if self.is_chance_node() {
                if let Some(action) = possible_outcomes.choose(&mut thread_rng()).cloned() {
                    self.add_node(action, bool_know_priv_info);
                    // TODO: forward pass
                    self.explore_recurse_mc_naive_prune(depth_counter + 1);
                    // TODO: backpropogate
                    self.drop_node();
                }
            } else {
                // Continue as per without mc
                for action in possible_outcomes {
                    if action.name() == AOName::ExchangeDraw {
                        if let Some(sampled_action) = self.naive_sample_exchange_draw(action.player_id()) {
                            self.add_node(sampled_action, bool_know_priv_info);
                            // TODO: forward pass
                            self.explore_recurse_mc_naive_prune(depth_counter + 1);
                            // TODO: backpropogate
                            self.drop_node();
                        } else {
                            self.node_counter += 1;
                        }
                    } else if !self.naive_prune(&action) {
                        self.add_node(action, bool_know_priv_info);
                        // TODO: forward pass
                        self.explore_recurse_mc_naive_prune(depth_counter + 1);
                        // TODO: backpropogate
                        self.drop_node();
                    } else {
                        self.node_counter += 1;
                    }
                }
            }
        }
    }
    // pub fn explore_recurse_pmc_naive_prune_green(&mut self, depth_counter: usize) {
    //     // Prunes not for next actions but for current action
    //     // Add forward pass and prune
    //     // TODO: Do one that prunes next actions on blue node
    //     self.visit_counter += 1;
    //     if depth_counter >= self.max_depth || self.history.game_won(){
    //         // Leaf Node reached!
    //         self.node_counter += 1;
    //         self.update_depth();
    //         let latest_influence_time_t: &[u8; 6] = self.history.latest_influence();
    //         let move_player: usize = self.history.latest_move().player_id();
    //         let rewards = self.rewards_evaluator.predict_value(&latest_influence_time_t.to_vec());
    //         let key: MSKey = MSKey::new(move_player, &self.path);
    //         self.rewards_store.insert_policy(key, rewards);
    //     } else {

    //         // if bri_self = 1 and all opponents = 0
    //             // pre prune case, only select BR Strats (Not implemented here)
    //         // else if bri_self = 0 and some opponents = 1
    //             // for all actions
    //                 // update value function storage | Q values
    //                 // if action is best response
    //                     // update current reward r  as the reward r after new_action
    //             // Reassign BR Strat based on new q value table
    //         // else: (1, 1) case
    //             // for all actions:
    //                 // recurse but with bri_ind updated based on whether move made was best response
                    

    //         // Always false for simulations
    //         let bool_know_priv_info: bool = false; 
    //         let possible_outcomes: Vec<ActionObservation> = self.history.generate_legal_moves();
    //         // 3 cases
    //         // chance node
    //         // all a is searched 
    //         // not all a is searched (for now we skip this and just prune)

    //         if self.is_chance_node() {
    //             if let Some(action) = possible_outcomes.choose(&mut thread_rng()).cloned() {
    //                 let latest_influence_time_t: &[u8; 6] = self.history.latest_influence();
    //                 self.add_node(action, bool_know_priv_info);
    //                 // TODO: forward pass
    //                 // fwd pass needs mixed strategy policy working
    //                 // mix_strat_policy_time_t is a variable
    //                 self.policy_handler.forward_pass_best_response_policy(&self.path, latest_influence_time_t, &self.history, &mut self.best_response_policy_vec, mix_strat_policy_time_t);
    //                 self.explore_recurse_mc_naive_prune(depth_counter + 1);
    //                 // TODO: backpropogate
    //                 self.drop_node();
    //             }
    //         } else {
    //             // Continue as per without mc
    //             for action in possible_outcomes {
    //                 if action.name() == AOName::ExchangeDraw {
    //                     if let Some(sampled_action) = self.naive_sample_exchange_draw(action.player_id()) {
    //                         self.add_node(sampled_action, bool_know_priv_info);
    //                         // TODO: forward pass
    //                         self.explore_recurse_mc_naive_prune(depth_counter + 1);
    //                         // TODO: backpropogate
    //                         self.drop_node();
    //                     } else {
    //                         self.node_counter += 1;
    //                     }
    //                 } else if !self.naive_prune(&action) {
    //                     // The most normal case
    //                     // all actions will be sampled

    //                     // How to update depends on the best_response_indicator stored in best_response_policy
    //                     // has 2 cases: updatespolicy_indicator for current player is 1 or 0

    //                     let latest_influence_time_t: &[u8; 6] = self.history.latest_influence();
    //                     // Add History and stuff
    //                     // useful for later
    //                     let current_player: usize = self.history.latest_move().player_id();
    //                     self.add_node(action, bool_know_priv_info);
    //                     // Forward pass
    //                     self.policy_handler.forward_pass_best_response_policy(&self.path, latest_influence_time_t, &self.history, &mut self.best_response_policy_vec, mix_strat_policy_time_t);
    //                     // Recurse
    //                     self.explore_recurse_mc_naive_prune(depth_counter + 1);
    //                     // TODO: Update Q values based on reward AFTER action is played | put in policy handler?
    //                     let move_player: usize = self.history.latest_move().player_id();
    //                     let key: MSKey = MSKey::new(move_player, &self.path);
    //                     if let Some(reward_action) = self.rewards_store.get(&key){
    //                         self.policy_handler.backward_pass_mixed_strategy_policy(&mut self.q_values, &key, &possible_outcomes, reward_action);
    //                         // Updating Q values over ~~
    //                         // Update reward for this node based on next reward node a few cases... prob have to do it on infostate by infostate level
    //                         //                       path.previous          path.current
    //                         // We can update since we are not in a pruned node
    //                         // [Update Rules]
    //                         // For each infostate, 2 cases, indicator for player at infostate is 1, and infostate is 0
    //                         // if ind is 1 :
    //                         //      r(p0) = r(p0) + r'(p0)
    //                         //      r(ps) = r(ps) + r'(ps) * sig^{ps}(I, a)
    //                         // if ind is 0 and there exists some possible gamestate where another player's indicator = 1
    //                         //      if a = BR^{ps}(I): (this if should prob be outside this tbh)
    //                         //          r(p0) = r'(p0)
    //                         // if ind is 0 and there doesnt exist some possible gamestate where another player's indicator = 1
    //                         //      just set rewards for this player to zero as this is not possible
    //                         //      r(ps) = 0
    //                         // TODO: Better to consolidate all action updates into reward_action
    //                         let path_current: String = self.action_embedder.remove_action(&self.path);
    //                         // Create BRKey
    //                         // Update by infostate if exists
    //                         for infostate in INFOSTATES {
    //                             for player_id in 0..6 as usize {
    //                                 let key_t: BRKey = BRKey::new(player_id, &infostate);
    //                                 if player_id == current_player {
    //                                     // Some update rules
    //                                 } else {
    //                                     // Some update rules
    //                                 }
    //                             }

    //                         }


    //                     } else {
    //                         panic!("reward does not exist at MSKey");
    //                     }

    //                     // Update other stuff
    //                     // Drop Stuff including rewards from action played
    //                     self.drop_node();
    //                     // Drop policy_indicator and value function                        
                        
                        
    //                     // TODO: backpropogate to update mixed strategy policy
    //                     // drop other stuff
    //                 } else {
    //                     // Pruned, do set some dummy values for now
    //                     self.node_counter += 1;
    //                 }
    //                 // Update Mixed Strategy Policy based on Q values
    //             }
    //         }
    //     }
    // }

    pub fn pmccfr(&mut self, depth_counter: usize, reach_prob: &ReachProb, back_transition: Option<HashMap<BRKey, Vec<BRKey>>>) -> AHashMap<BRKey, f32> {
        // TODO: move history out of self and into an input
        // TODO: Reorganise using backtransition for return as backtransition depends on action
        // Prunes not for next actions but for current action
        // Add forward pass and prune
        // TODO: Do one that prunes next actions on blue node
        // TODO: Change reach_prob to struct with 6 players and each a hm of infostate: bool
        // TODO: rayon the should_prune_current
        self.visit_counter += 1;
        
        // INITIALISE REWARDS
        // TODO: abstract to a function
        let mut rewards: AHashMap<BRKey, f32> = AHashMap::with_capacity(MAX_NUM_BRKEY);
        if depth_counter >= self.max_depth || self.history.game_won(){
            // CASE WHEN LEAF NODE REACHED
            if self.history.game_won() {
                self.won_counter += 1;
            }
            self.node_counter += 1;
            let latest_influence_time_t: &[u8; 6] = self.history.latest_influence();
            let move_player: usize = self.history.latest_move().player_id();
            rewards = self.rewards_evaluator.predict_value(&latest_influence_time_t.to_vec());
            // if let Some(None) = back_transition {
            //     // TODO: Modify rewards with back_transition
            //     rewards = rewards;
            // }
        // } else if self.policy_handler.should_prune_current(reach_prob){
        } else if reach_prob.should_prune(){
            self.prune_counter += 1;
            let mut key_br: BRKey = BRKey::new(0, &Infostate::AA);
            for player_id in 0..6 as u8 {
                key_br.set_player_id(player_id);
                for infostate in reach_prob.player_infostate_keys(player_id) {
                    key_br.set_infostate(infostate);
                    rewards.insert(key_br, 0.0);
                }
            }
        } else {
            // Always false for simulations
            let bool_know_priv_info: bool = false; 
            let possible_outcomes: Vec<ActionObservation> = self.history.generate_legal_moves();
            let path_t: String = self.path.clone();
            // TOCHECK: In cases of collective action next, this makes no sense
            // INITIALISATIONS
            if self.is_exchangedraw_node() {
                // Dont need to initialise
            } else if self.is_collective_node() {
                // FOR SPECIAL CASES => {Collective Challenge, CollectiveBlock}
                // based on key_ms_t
                // TODO: fix for ExchangeDraw case
                // TODO: Free for all CollectiveChallenge and CollectiveBlock
                let latest_influence_time_t: &[u8; 6] = self.history.latest_influence();
                let possible_challenge_moves: Vec<ActionObservation> = vec![ActionObservation::ChallengeAccept, ActionObservation::ChallengeDeny];
                for player_id in 0..6 as u8 {
                    if latest_influence_time_t[player_id as usize] > 0 {
                        self.mixed_strategy_policy_vec.insert_default_if_empty_by_keys(player_id, &self.path, &possible_challenge_moves, reach_prob.player_infostate_keys(player_id));
                        self.q_values.insert_default_if_empty_by_keys(player_id, &self.path, &possible_challenge_moves, reach_prob.player_infostate_keys(player_id));   
                    }
                }
            } else {
                // FOR NORMAL MOVES
                // based on key_ms_t
                let player_id: u8 = possible_outcomes[0].player_id() as u8;
                self.mixed_strategy_policy_vec.insert_default_if_empty_by_keys(player_id, &self.path, &possible_outcomes, reach_prob.player_infostate_keys(player_id));
                self.q_values.insert_default_if_empty_by_keys(player_id, &self.path, &possible_outcomes, reach_prob.player_infostate_keys(player_id));
            }
            // RECURSIONS
            // Test if depth < 6 whether can random sample
            if self.is_collective_node() {
                // TODO: SHIFT OUT AND USE ACTIONS
                // CollectiveChallenge
                // CollectiveBlock
                // TODO: sample a random action and sample if nobody challenges (which is move 0)
                // Should sample outcomes equally so we converge on a more reliable outcome
                // Dont need to sample action by action, can sample whole, but must update the mixed strategy reach prob and rewards accurately 
                // Or maybe its ok to sample because sometimes players choose no and end up with no risk anyways, just randomly sample the move...
                if let Some(action) = possible_outcomes.choose(&mut thread_rng()).cloned() {
                    self.add_node(action, bool_know_priv_info);
                    // TODO: forward pass
                    // fwd pass needs mixed strategy policy working
                    // mix_strat_policy_time_t is a variable
                    // self.policy_handler.forward_pass_best_response_policy(&self.path, latest_influence_time_t, &self.history, &mut self.best_response_policy_vec, mix_strat_policy_time_t);
                    let mut next_reach_prob: ReachProb = reach_prob.clone();

                    // TODO: Link mixed strategy
                    // This is a temp to see how many nodes will be visited
                    let mut rng = thread_rng();
                    let opposing_player_id: u8 = action.opposing_player_id() as u8;
                    for player_id in 0..6 as u8 {
                        if player_id == opposing_player_id {
                            continue;
                        }
                        let infostates: Vec<Infostate> = next_reach_prob.player_infostate_keys(player_id).cloned().collect();
                        for infostate in infostates {
                            if let Some(indicator) = next_reach_prob.get_mut_status(player_id, &infostate) {
                                if *indicator {
                                    if rng.gen_range(0.0..1.0) > 0.5 {
                                        *indicator = false;
                                        // next_reach_prob.set_status(player_id, &infostate, false);
                                    }
                                }
                            }
                        }
                    }
                    
                    rewards = self.pmccfr(depth_counter + 1, &next_reach_prob, None);
                    // rewards = self.pmccfr(depth_counter + 1, &reach_prob, None);
                    // TODO: backpropogate
                    self.drop_node();
                }
            // Add ExchangeDraw here
            } else if self.is_exchangedraw_node() {
                if let Some(action) = possible_outcomes.choose(&mut thread_rng()).cloned() {
                    self.add_node(action, bool_know_priv_info);
                    // Placeholder
                    rewards = self.pmccfr(depth_counter + 1, reach_prob, None);
                    // TODO: backpropogate
                    self.drop_node();
                }
            } else {
                // Initialise rewards to 0 for relevant infostates
                let mut key_br: BRKey = BRKey::new(0, &Infostate::AA);
                for player_id in 0..6 as u8 {
                    key_br.set_player_id(player_id);
                    for infostate in reach_prob.player_infostate_keys(player_id) {
                        key_br.set_infostate(infostate);
                        rewards.insert(key_br, 0.0);
                    }
                }

                let mut action_index: usize = 0;
                // LOOP THROUGH ALL ACTIONS AND RECURSE
                for action in possible_outcomes.iter() {
                    // TODO: Include naive pruning for impossible actions!
                    if self.naive_prune(action){
                        continue;
                    } if action.name() == AOName::ExchangeDraw {
                        // Chance Node
                        if let Some(sampled_action) = self.naive_sample_exchange_draw(action.player_id()) {
                            self.add_node(sampled_action, bool_know_priv_info);
                            // let next_reach_prob: HashMap<BRKey, bool> = HashMap::with_capacity(MAX_NUM_BRKEY);
                            // TODO: Fill reach prob based on input reachprob and action for pmccfr function
                            // let transition_map: HashMap<String, Vec<String>> = HashMap::with_capacity(MAX_NUM_BRKEY);
                            // TODO: Fill transition hashmap for pmccfr function
                            rewards = self.pmccfr(depth_counter + 1, reach_prob, None);
                            // TODO: backpropogate
                            self.drop_node();
                            // TODO: UPDATE REWARDS?
                        } 
                    } else if action.name() == AOName::ExchangeChoice {
                        // TODO: Change to the actual code randomly sample from each player's mixed_strategy
                        // Shuffle reach prob
                        // let mut next_reach_prob: HashMap<BRKey, bool> = reach_prob.clone();
                        self.add_node(*action, bool_know_priv_info);
                        // Recurse and continue on an action
                        // rewards should be temp-rewards
                        rewards = self.pmccfr(depth_counter + 1, reach_prob, None);
                        self.drop_node();
                        // TODO: UPDATE REWARDS?
                        // Player chooses "AB"
                        // Need to know exchangeDraw
                        // reward should update to all states that can choose AB
                    } else if ![AOName::Discard, AOName::RevealRedraw].contains(&action.name()) {
                        // NORMAL MOVE CASE
                        // all actions will be sampled

                        // How to update depends on the best_response_indicator stored in best_response_policy
                        // has 2 cases: updatespolicy_indicator for current player is 1 or 0

                        // Add History and stuff to sample a possible move
                        
                        let next_player_id: u8 = action.player_id() as u8;
                        let key_ms_t = MSKey::new(next_player_id, &path_t);
                        
                        
                        // INITIALISING next_reach_prob based on action sampled to be passed into recursion
                        // TODO: Abstract this to a function
                        let mut next_reach_prob: ReachProb = reach_prob.clone();
                        // Filling next reach prob based on input reachprob and action for pmccfr function
                        //      If indicator == 1 and action is best response => 1
                        //      If indicator == 1 and action is not best response => 0
                        //      If indicator == 0 => 0
                        // TOCHECK: next player?
                        for infostate in reach_prob.player_infostate_keys(next_player_id) {
                            if let Some(old_indicator) = reach_prob.get_status(next_player_id, infostate) {
                                if *old_indicator {
                                    let infostate_best_response: ActionObservation = self.mixed_strategy_policy_vec.get_best_response(&key_ms_t, infostate);
                                    if !(infostate_best_response == *action) {
                                        // change 1 to 0
                                        next_reach_prob.set_status(next_player_id, infostate, false);
                                    }
                                }
                            }
                        }
                        // Filling transition hashmap for pmccfr function 1 to 1
                        // TODO: Abstract this to a function
                        // let mut transition_map: HashMap<String, Vec<String>> = HashMap::with_capacity(MAX_NUM_BRKEY);
                        // for key_br in next_reach_prob.keys() {
                        //     // 1 to 1 no change case
                        //     transition_map.insert(key_br, vec![key_br]);
                        // }
                        self.add_node(*action, bool_know_priv_info);
                        // Recurse and continue on an action
                        let temp_reward: AHashMap<BRKey, f32> = self.pmccfr(depth_counter + 1, &next_reach_prob, None);
                        
                        // TODO: Change, move_player is same as next_player_id
                        let move_player: u8 = action.player_id() as u8;
                        
                        // UPDATE Q_VALUES BASED ON RETURNED REWARDS
                        // TODO: Abstract this to a function
                        if let Some(q_value) = self.q_values.policy_get_mut(&key_ms_t) {
                            let mut key_br: BRKey = BRKey::new(next_player_id, &Infostate::AA);
                            for infostate in INFOSTATES {
                                key_br.set_infostate(infostate);
                                // TODO: update q_value 
                                // TOCHECK: value updating
                                if let Some(values_vec) = q_value.get_mut(&key_br) {
                                    if let Some(reward_val) = temp_reward.get(&key_br){
                                        values_vec[action_index] += *reward_val;
                                    }
                                }
                            }
                        } else {
                            panic!("q_values should have been initialised earlier!");
                        }
                        // UPDATE REWARDS for now if reach_prob = 1 we update
                            // Ignore the distinction for when current player is playing BR cos its complicated for infostates
                        // TODO: Abstract this to a function
                        let mut key_br: BRKey = BRKey::new(0, &Infostate::AA);
                        for player_id in 0..6 as u8 {
                            key_br.set_player_id(player_id);
                            for infostate in reach_prob.player_infostate_keys(player_id) {
                                if let Some(indicator) = reach_prob.get_status(player_id, infostate) {
                                    if *indicator {
                                        let mut increment_val: f32 = 0.0;
                                        key_br.set_infostate(infostate);
                                        // 0-100ns for increment but usually 100ns
                                        if let Some(temp_reward_val) = temp_reward.get(&key_br) {
                                            increment_val = *temp_reward_val;
                                        }
                                        // 0-100ns usually 100 ns
                                        if let Some(reward_val) = rewards.get_mut(&key_br){
                                            *reward_val += increment_val;
                                        } else {
                                            rewards.insert(key_br, increment_val);
                                        }
                                    }
                                }
                            }
                        }
                        // IMPROVE: Do like some rayon shit
                        // Drop node we added before recursing
                        self.drop_node();                      
                        
                        
                    } else {
                        // CASE DISCARD, REVEALREDRAW and player cannot have the cards to do the current action
                        // Do not do anything
                        let move_player: u8 = action.player_id() as u8;
                        let key_ms_t: MSKey = MSKey::new(move_player, &path_t);
                        
                        let mut next_reach_prob: ReachProb = ReachProb::new_empty();
                        // let mut next_reach_prob: HashMap<BRKey, bool> = HashMap::with_capacity(reach_prob.len());
                        // Filling next reach prob based on input reachprob and action for pmccfr function
                        // If player is move player:
                        //      If indicator == 1 and action is best response => 1
                        //      If indicator == 1 and action is not best response => 0
                        //      If indicator == 0 => 0
                        // Abstract this to a function
                        if action.name() == AOName::Discard {
                            if action.no_cards() == 1 {
                                // INITIALISING REACH_PROB AFTER THIS MOVE
                                let card_str: &str = action.cards()[0].card_to_str();
                                for player_id in 0..6 as u8 {
                                    for infostate in reach_prob.player_infostate_keys(player_id) {
                                        if let Some(old_indicator) = reach_prob.get_status(player_id, infostate) {
                                            if player_id == action.player_id() as u8 {
                                                if infostate.contains(card_str) {
                                                    if *old_indicator {
                                                        let infostate_best_response = self.mixed_strategy_policy_vec.get_best_response(&key_ms_t, &infostate);
                                                        if *action == infostate_best_response {
                                                            next_reach_prob.set_status(player_id, infostate, true);
                                                        } else {
                                                            next_reach_prob.set_status(player_id, infostate, false);
                                                        }
                                                    } else {
                                                        next_reach_prob.set_status(player_id, infostate, *old_indicator);
                                                    }
                                                }
                                            } else {
                                                // Populate as per before
                                                next_reach_prob.set_status(player_id, infostate, *old_indicator);
                                            }
                                        } 
                                    }
                                }
                            } else {
                                let cards_action: Infostate = Infostate::cards_to_enum(&action.cards()[0], &action.cards()[1]);
                                for player_id in 0..6 as u8 {
                                    for infostate in reach_prob.player_infostate_keys(player_id) {
                                        if let Some(old_indicator) = reach_prob.get_status(player_id, infostate) {
                                            if player_id == action.player_id() as u8 {
                                                if *infostate == cards_action {
                                                    // Only insert for infostates that can possibly have discarded the card
                                                    // TODO: Change
                                                    // DIscard 2 cards only ever has one choice
                                                    // Then player dies
                                                    // Should change indicator to 0 because player is never alive
                                                    // This makes it so that when pruning, this player is effectively ignored as though the game were
                                                    // only for the alive players
                                                    next_reach_prob.set_status(player_id, infostate, false);
                                                }
                                            } else {
                                                // Populate as per before
                                                next_reach_prob.set_status(player_id, infostate, *old_indicator);
                                            }
                                        }
                                    }
                                }
                            }
                        } else if action.name() == AOName::RevealRedraw {
                            // INITIALISING REACH_PROB AFTER THIS MOVE
                            let card_str: &str = action.card().card_to_str();
                            for player_id in 0..6 {
                                for infostate in reach_prob.player_infostate_keys(player_id) {
                                    if let Some(old_indicator) = reach_prob.get_status(player_id, infostate) {
                                        if player_id == action.player_id() as u8 {
                                            if infostate.contains(card_str) {
                                                let infostate_best_response = self.mixed_strategy_policy_vec.get_best_response(&key_ms_t, &infostate);
                                                if *old_indicator {
                                                    if *action == infostate_best_response {
                                                        next_reach_prob.set_status(player_id, infostate, true);
                                                    } else {
                                                        next_reach_prob.set_status(player_id, infostate, false);
                                                    }
                                                } else {
                                                    next_reach_prob.set_status(player_id, infostate, false);
                                                }
                                            }
                                        } else {
                                            next_reach_prob.set_status(player_id, infostate,*old_indicator);
                                        }
                                    }
                                }
                            }
                        }
                        self.add_node(*action, bool_know_priv_info);
                        // RECURSING
                        let temp_reward: AHashMap<BRKey, f32> = self.pmccfr(depth_counter + 1, &next_reach_prob, None);
                        // Update the rewards SAME AS NORMAL MOVES
                        let move_player: u8 = action.player_id() as u8;
                        
                        // UPDATE Q_VALUES BASED ON RETURNED REWARDS
                        // TODO: Abstract this to a function
                        let mut key_br: BRKey = BRKey::new(move_player, &Infostate::AA);
                        if let Some(q_value) = self.q_values.policy_get_mut(&key_ms_t) {
                            for infostate in INFOSTATES {
                                key_br.set_infostate(infostate);
                                // TODO: update q_value 
                                // TOCHECK: value updating
                                if let Some(values_vec) = q_value.get_mut(&key_br) {
                                    if let Some(reward_val) = temp_reward.get(&key_br) {
                                        values_vec[action_index] += *reward_val;
                                    }
                                }
                            }
                        } else {
                            panic!("q_values should have been initialised earlier!");
                        }
                        // UPDATE REWARDS for now if reach_prob = 1 we update
                            // Ignore the distinction for when current player is playing BR cos its complicated for infostates
                        // TODO: Abstract this to a function
                        let mut key_br: BRKey = BRKey::new(0, &Infostate::AA);
                        for player_id in 0..6 as u8 {
                            key_br.set_player_id(player_id);
                            for infostate in reach_prob.player_infostate_keys(player_id) {
                                if let Some(indicator) = reach_prob.get_status(player_id, infostate) {
                                    if *indicator {
                                        let mut increment_val: f32 = 0.0;
                                        key_br.set_infostate(infostate);
                                        if let Some(temp_reward_val) = temp_reward.get(&key_br) {
                                            increment_val = *temp_reward_val;
                                        } 
                                        if let Some(reward_val) = rewards.get_mut(&key_br){
                                            *reward_val += increment_val
                                        } else {
                                            rewards.insert(key_br, increment_val);
                                        }
                                    }
                                }
                            }
                        }
                        self.drop_node();
                    }
                    action_index += 1;
                }
                // UPDATED MIXED STRATEGY POLICY based on action with highest Q value
                // Abstract to a 
                if self.is_collective_node() {
                    // CASE WHEN action == CollectiveChallenge or CollectiveBlock
                    // TODO: Check
                    let mut mskey_t: MSKey = MSKey::new(0, &path_t);
                    for player_id in 0..6 {
                        mskey_t.set_player_id(player_id);
                        for infostate in INFOSTATES {
                            if let Some(best_move_index) = self.q_values.get_best_response_index(&mskey_t, infostate) {
                                self.mixed_strategy_policy_vec.add_value(&mskey_t, player_id, infostate, best_move_index, 1.0);
                            }
                        }
                    }
                }
                else if self.is_chance_node() {
                    // CASE WHEN action == ExchangeDraw
                } else {
                    // MIXED STRATEGY POLICY should have been initialised earlier
                    let current_player: u8 = possible_outcomes[0].player_id() as u8;
                    let mskey_t: MSKey = MSKey::new(current_player, &path_t);
                    for infostate in INFOSTATES {
                        if let Some(best_move_index) = self.q_values.get_best_response_index(&mskey_t, infostate) {
                            self.mixed_strategy_policy_vec.add_value(&mskey_t, current_player, infostate, best_move_index, 1.0);
                        } 
                    }
                }
            }
        }
        // TODO: Backtransition here
        if let Some(transition_map) = back_transition {
            let mut rewards_clone: AHashMap<BRKey, f32> = AHashMap::with_capacity(MAX_NUM_BRKEY);
            //  Temp
            for key_br in rewards.keys() {
                rewards_clone.insert(*key_br, 0.0);
            }
            for (key_br, key_br_vec) in transition_map {
                for new_key_br in key_br_vec {
                    if let Some(value) = rewards_clone.get_mut(&new_key_br) {
                        *value += rewards[&key_br];
                    }
                }
            }
            rewards_clone
        } else {
            rewards
        }
    }
}

pub fn cfr_test(){
    // max_depth | total end nodes | total nodes visited
    // 1 | 9 | 10 | 23.9µs
    // 2 | 657 | 667 | 67.3µs
    // 3 | 9985 | 10652 | 1.0512ms
    // 4 | 67849 | 78501 | 10.8536ms
    // 5 | 1921289 | 1999790 | 111.1832ms
    // 6 | 45024622 | 47024512 | 3.2281328s
    // 7 | 506043405 | 553012287 | 64.434922s
    // 8 | 7299339741 | 7854551863 | 716.8951678s

    let max_test_depth: usize = 13;
    let mut pmccfr: Explorer = Explorer::new(0);
    for max_depth in 1..max_test_depth {
        pmccfr.set_depth(max_depth);
        let start_time = Instant::now();
        pmccfr.explore();
        let elapsed_time = start_time.elapsed();
        let nodes_traversed: u128 = pmccfr.nodes_traversed();
        let nodes_reached: u128 = pmccfr.nodes_reached();
        // println!("Max Depth: {}", max_depth);
        // println!("Nodes Reached: {}", nodes_reached);
        // println!("Nodes Traversed: {}", nodes_traversed);
        println!("{} | {} | {} | {:?}", max_depth, nodes_reached, nodes_traversed, elapsed_time);
        // println!("Total Time Taken: {:?}", elapsed_time);
        pmccfr.reset();
    }
}
pub fn cfr_prune_test(){
    // max_depth | total_end_nodes | total_nodes_visited
    // 1 | 9 | 18 | 25.3µs
    // 2 | 657 | 1323 | 67.9µs
    // 3 | 7266 | 15198 | 420.3µs
    // 4 | 10155 | 28146 | 1.3121ms
    // 5 | 80495 | 178885 | 5.0897ms
    // 6 | 722158 | 1542610 | 41.5231ms
    // 7 | 1832426 | 4485213 | 157.1395ms
    // 8 | 10319640 | 23298496 | 724.5508ms
    // 9 | 76888710 | 166771235 | 4.4578065s
    // 10 | 263800653 | 617546480 | 21.5476017s
    //11 | 1327829521 | 3009749018 | 89.5332159s

    let max_test_depth: usize = 20;
    for max_depth in 10..max_test_depth {
        let mut pmccfr: Explorer = Explorer::new(max_depth);
        let start_time = Instant::now();
        pmccfr.explore_recurse_naive_prune(0);
        let elapsed_time = start_time.elapsed();
        let nodes_traversed: u128 = pmccfr.nodes_traversed();
        let nodes_reached: u128 = pmccfr.nodes_reached();
        println!("Max Depth: {}", max_depth);
        println!("Nodes Reached: {}", nodes_reached);
        println!("Nodes Traversed: {}", nodes_traversed);
        println!("{} | {} | {} | {:?}", max_depth, nodes_reached, nodes_traversed, elapsed_time);
        println!("Total Time Taken: {:?}", elapsed_time);
    }
}
pub fn mccfr_test(){
    // max_depth | total_end_nodes | total_nodes_visited
    // 1 | 9 | 18
    // 2 | 17 | 43
    // 3 | 57 | 140
    // 4 | 321 | 725
    // 5 | 907 | 2210
    // 6 | 2858 | 7019
    // 7 | 12265 | 28666
    // 8 | 44107 | 104975  
    // 9 | 150169 | 361840 
    // 10 | 552635 | 1315771
    // 11 | 2060243 | 4889974
    // 12 | 7317020 | 17459595

    let max_test_depth: usize = 200;
    let max_iterations: usize = 1000;
    let mut pmccfr: Explorer = Explorer::new(0);
    for max_depth in 1..max_test_depth {
        let mut max_nodes_traversed: u128 = 0;
        let mut max_nodes_reached: u128 = 0;
        pmccfr.set_depth(max_depth);
        let mut i: usize = 0;
        while i < max_iterations {
            pmccfr.explore_recurse_mc(0);
            let nodes_traversed: u128 = pmccfr.nodes_traversed();
            let nodes_reached: u128 = pmccfr.nodes_reached();
            // println!("Max Depth: {}", max_depth);
            // println!("Nodes Reached: {}", nodes_reached);
            // println!("Nodes Traversed: {}", nodes_traversed);
            // println!("Total Time Taken: {:?}", elapsed_time);
            if nodes_traversed > max_nodes_traversed {
                max_nodes_traversed = nodes_traversed;
            }
            if nodes_reached > max_nodes_reached {
                max_nodes_reached = nodes_reached;
            }
            i += 1;
            pmccfr.reset();
        }
        println!("{} | {} | {}", max_depth, max_nodes_reached, max_nodes_traversed);
    }
}
pub fn mccfr_prune_test(){
    // max_depth | total_end_nodes | total_nodes_visited
    // 1 | 9 | 10
    // 2 | 17 | 27
    // 3 | 57 | 84
    // 4 | 321 | 405
    // 5 | 907 | 1312
    // 6 | 2860 | 4168
    // 7 | 12248 | 16400
    // 8 | 44193 | 60965
    // 9 | 151621 | 213850
    // 10 | 548181 | 757979
    // 11 | 2052800 | 2819914
    // 12 | 7353081 | 10183281

    let max_test_depth: usize = 200;
    let max_iterations: usize = 1000;
    let mut pmccfr: Explorer = Explorer::new(0);
    for max_depth in 10..max_test_depth {
        let mut max_nodes_traversed: u128 = 0;
        let mut max_nodes_reached: u128 = 0;
        pmccfr.set_depth(max_depth);
        let mut i: usize = 0;
        while i < max_iterations {
            pmccfr.explore_recurse_mc_naive_prune(0);
            let nodes_traversed: u128 = pmccfr.nodes_traversed();
            let nodes_reached: u128 = pmccfr.nodes_reached();
            // println!("Max Depth: {}", max_depth);
            // println!("Nodes Reached: {}", nodes_reached);
            // println!("Nodes Traversed: {}", nodes_traversed);
            // println!("Total Time Taken: {:?}", elapsed_time);
            if nodes_traversed > max_nodes_traversed {
                max_nodes_traversed = nodes_traversed;
            }
            if nodes_reached > max_nodes_reached {
                max_nodes_reached = nodes_reached;
            }
            i += 1;
            pmccfr.reset();
        }
        println!("{} | {} | {}", max_depth, max_nodes_reached, max_nodes_traversed);
    }
}
pub fn pmccfr_test(start_test_depth: usize, max_test_depth: usize, max_iterations: usize) {
    // max_depth | total_end_nodes | total_nodes_visited | Pure Pruned
    // 1 | 9 | 10 | 0
    // 2 | 17 | 27 | 0
    // 3 | 49 | 76 | 0
    // 4 | 309 | 385 | 0
    // 5 | 875 | 1256 | 0
    // 6 | 2671 | 3976 | 0
    // 7 | 11510 | 15430 | 8
    // 8 | 41740 | 57343 | 24
    // 9 | 139419 | 198332 | 300
    // 10 | 509256 | 707819 | 990
    // 11 | 1872992 | 2576667 | 5728
    
    // random challenge
    // 1 | 9 | 10 | 0
    // 2 | 17 | 27 | 0
    // 3 | 49 | 76 | 0
    // 4 | 305 | 381 | 0
    // 5 | 895 | 1280 | 0
    // 6 | 2673 | 3958 | 0
    // 7 | 11564 | 15488 | 8
    // 8 | 42642 | 58496 | 16
    // 9 | 132878 | 187450 | 294
    // 10 | 497707 | 694303 | 1535
    // 11 | 1942557 | 2689918 | 11672

    // Fast max 1000 iter
    // 1 | 0 | 9 | 10 | 0 | 1000
    // 2 | 0 | 17 | 27 | 0 | 1000
    // 3 | 0 | 61 | 88 | 0 | 1000
    // 4 | 0 | 91 | 169 | 20 | 1000
    // 5 | 0 | 173 | 345 | 50 | 1000
    // 6 | 0 | 258 | 586 | 90 | 1000
    // 7 | 0 | 387 | 1019 | 138 | 1000
    // 8 | 0 | 486 | 1560 | 178 | 1000
    // 9 | 0 | 543 | 1868 | 213 | 999
    // 10 | 0 | 690 | 2539 | 245 | 1000
    // 11 | 0 | 742 | 3865 | 221 | 997
    // 12 | 0 | 754 | 4500 | 272 | 999
    // 13 | 0 | 735 | 4438 | 221 | 979
    // 14 | 0 | 671 | 5461 | 269 | 979
    // 15 | 0 | 869 | 5666 | 221 | 855
    // 16 | 0 | 733 | 6583 | 269 | 813
    // 17 | 0 | 958 | 6707 | 269 | 546
    // 18 | 0 | 859 | 6756 | 277 | 498
    // 19 | 0 | 953 | 9362 | 221 | 439
    // 20 | 0 | 754 | 8109 | 221 | 439
    // 21 | 0 | 808 | 9469 | 269 | 404
    // 22 | 0 | 525 | 9631 | 221 | 419
    // 23 | 0 | 432 | 10724 | 277 | 376
    // 24 | 0 | 444 | 10359 | 261 | 346
    // 25 | 0 | 303 | 11002 | 221 | 274
    // 26 | 0 | 305 | 10842 | 231 | 246
    // 27 | 0 | 286 | 10822 | 269 | 155
    // 28 | 0 | 305 | 7511 | 265 | 137
    // 29 | 0 | 301 | 15766 | 257 | 70
    // 30 | 0 | 337 | 11209 | 269 | 42
    // 31 | 0 | 108 | 11627 | 257 | 17
    // 32 | 0 | 274 | 11359 | 221 | 17
    // 33 | 0 | 219 | 10021 | 277 | 15
    // 34 | 0 | 119 | 9541 | 273 | 6
    // 35 | 0 | 79 | 10885 | 269 | 4
    // 36 | 0 | 1 | 8889 | 221 | 2
    // 37 | 0 | 82 | 11660 | 273 | 3
    // 38 | 0 | 46 | 11848 | 221 | 3
    // let start_test_depth: usize = 10;
    // let max_test_depth: usize = 11;
    // let max_iterations: usize = 1;
    let mut pmccfr: Explorer = Explorer::new(0);
    for max_depth in start_test_depth..max_test_depth {
        let mut i: usize = 0;
        let mut max_nodes_traversed: u128 = 0;
        let mut max_nodes_reached: u128 = 0;
        let mut max_nodes_won: u128 = 0;
        let mut min_nodes_pruned: u128 = 1000000000000000000000000000;
        let mut total_nodes_traversed: u128 = 0;
        let mut total_nodes_reached: u128 = 0;
        let mut total_nodes_pruned: u128 = 0;
        let mut total_nodes_won: u128 = 0;
        let mut no_depth_pos: u128 = 0;
        let mut total_time: f64 = 0.0;
        let mut iteration_time: f64 = 0.0;
        while i < max_iterations {
            pmccfr.set_depth(max_depth);
            let reach_prob: ReachProb = ReachProb::new();
            // for player_id in 0..6 as usize {
            //     for infostate in INFOSTATES {
            //         let brkey: BRKey = BRKey::new(player_id, infostate);
            //         reach_prob.insert(brkey, true);
            //     }
            // }
            let start_time = Instant::now();
            pmccfr.pmccfr(0, &reach_prob, None);
            let elapsed_time = start_time.elapsed();
            total_time += elapsed_time.as_secs_f64();
            iteration_time = elapsed_time.as_secs_f64();
            // println!("Iteration time: {:?}", elapsed_time);
            let nodes_traversed: u128 = pmccfr.nodes_traversed();
            let nodes_reached: u128 = pmccfr.nodes_reached();
            let nodes_pruned: u128 = pmccfr.nodes_pruned();
            let nodes_won: u128 = pmccfr.nodes_won();
            total_nodes_pruned += nodes_pruned;
            total_nodes_reached += nodes_reached;
            total_nodes_traversed += nodes_traversed;
            total_nodes_won += nodes_won;
            if max_nodes_traversed < nodes_traversed {
                max_nodes_traversed = nodes_traversed;
            }
            if max_nodes_reached < nodes_reached {
                max_nodes_reached = nodes_reached;
            }
            if max_nodes_won < nodes_won {
                max_nodes_won = nodes_won;
            }
            if min_nodes_pruned > nodes_pruned {
                min_nodes_pruned = nodes_pruned;
            }
            if nodes_reached > 0 {
                no_depth_pos += 1;
            }
            pmccfr.reset_counters();
            i += 1;
            let avg_time_current_node: f64 = iteration_time / nodes_traversed as f64;
            // println!("avg time for iteration of depth {}: {:.9} secs iteration: {:.9} secs", i, avg_time_current_node, iteration_time);
        }
        let avg_time: f64 = total_time / max_iterations as f64;
        let avg_time_per_node: f64 = total_time / total_nodes_traversed as f64;

        println!("avg_time for depth {} : {:.9} secs", max_depth, avg_time);
        println!("avg_time_per_node for depth {} : {:.9} secs", max_depth, avg_time_per_node);
        let avg_nodes_pruned: f32 = total_nodes_pruned as f32 / max_iterations as f32;
        let avg_nodes_reached: f32 = total_nodes_reached as f32 / max_iterations as f32;
        let avg_nodes_traversed: f32 = total_nodes_traversed as f32 / max_iterations as f32;
        // println!("{} | {} | {} | {} | {} | {}", max_depth, max_nodes_won, max_nodes_reached, max_nodes_traversed, min_nodes_pruned, no_depth_pos);
        println!("{} | {} | {:.0} | {:.0} | {:.0} | {}", max_depth, total_nodes_won, avg_nodes_reached, avg_nodes_traversed, avg_nodes_pruned, no_depth_pos);
        // if let Some(policy) = pmccfr.get_root_policy() {
        //     println!("policy: {:?}", policy);
        // } else {
        //     println!("policy returned None");
        // }
        // if let Some(action_map) = pmccfr.get_root_action_map() {
        //     println!("action_map: {:?}", action_map);
        // } else {
        //     println!("action_map returned None");
        // }
        pmccfr.reset();
    }
}