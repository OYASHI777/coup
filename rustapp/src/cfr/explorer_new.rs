use rand::prelude::SliceRandom;
use rand::thread_rng;
use crate::prob_manager::naive_prob::NaiveProb;
use crate::prob_manager::naive_sampler::NaiveSampler;
use crate::history_public::{AOName, ActionObservation, Card, History};
use crate::prob_manager::constraint::CollectiveConstraint;
use super::action_serialiser::{DefaultEmbedding, ActionEmbedding};
use super::best_response_policy::BestResponseIndVec;
use super::mixed_strategy_policy::{HeuristicMixedStrategyPolicy, MSInterface, ValueStorage};
use super::policy_handler::{PolicyHandler, PolicyUpdate};
use super::value_function::{HeuristicValueFunction, ValueEvaluation}
use super::keys::{BRKey, MSKey, INFOSTATES, MAX_NUM_BRKEY};
use super::rewards_store::RewardsStore;
// use super::best_response_policy::BRKey;

use std::collections::HashMap;
use std::usize::MAX;
struct Explorer<'a> {
    // This is a struct used to conduct Pure Monte Carlo CounterFactual Regret Minimization (PMCCFR)
    path: String,
    action_embedder: Box<dyn ActionEmbedding>,
    history: History,
    best_response_policy_vec: BestResponseIndVec,
    mixed_strategy_policy_vec: Box<dyn MSInterface>,
    q_values: Box<dyn MSInterface>,
    rewards_store: RewardsStore, // See mixed_strategy_policy
    rewards_evaluator: Box<dyn ValueEvaluation<u8>>,
    prob: NaiveProb<'a>,
    chance_sampler: NaiveSampler<'a>,
    policy_handler: PolicyHandler,

    max_depth: usize,
    bool_monte_carlo: bool,
    node_counter: u128,
    visit_counter: u128,
    // TO Document best_response_policy
}

impl <'a> Explorer<'a> {
    pub fn new(max_depth: usize) -> Self {
        // make this into config HashMap when done
        Explorer{
            // temporarily the starting_player is 0
            path: "root".to_string(),
            action_embedder: Box::new(DefaultEmbedding),
            history: History::new(0),
            best_response_policy_vec: BestResponseIndVec::new(max_depth + 1),
            mixed_strategy_policy_vec: Box::new(HeuristicMixedStrategyPolicy::new()),
            q_values: Box::new(HeuristicMixedStrategyPolicy::new()),
            rewards_store: RewardsStore::new(),
            rewards_evaluator: Box::new(HeuristicValueFunction),
            prob: NaiveProb::new(),
            chance_sampler: NaiveSampler::new(),
            policy_handler: PolicyHandler::new(),
            max_depth,
            bool_monte_carlo: true,
            node_counter: 0,
            visit_counter: 0,
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
        self.rewards_store.reset();
        self.node_counter = 0;
        self.visit_counter = 0;
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

    pub fn pmccfr(&mut self, depth_counter: usize, path: &str, history: History, prob: NaiveProb, reach_prob: HashMap<BRKey, bool>, back_transition: Option<HashMap<BRKey, Vec<BRKey>>>) -> HashMap<BRKey, f32> {
        // TODO: move history out of self and into an input
        // TODO: Reorganise using backtransition for return as backtransition depends on action
        // Prunes not for next actions but for current action
        // Add forward pass and prune
        // TODO: Do one that prunes next actions on blue node
        self.visit_counter += 1;
        
        // INITIALISE REWARDS
        let mut rewards: HashMap<BRKey, f32> = HashMap::with_capacity(MAX_NUM_BRKEY);

        if depth_counter >= self.max_depth || history.game_won(){
            // CASE WHEN LEAF NODE REACHED
            self.node_counter += 1;
            self.update_depth();
            let latest_influence_time_t: &[u8; 6] = history.latest_influence();
            let move_player: usize = history.latest_move().player_id();
            rewards = self.rewards_evaluator.predict_value(&latest_influence_time_t.to_vec());
            // if let Some(None) = back_transition {
            //     // TODO: Modify rewards with back_transition
            //     rewards = rewards;
            // }
        } else {
                   
            // Always false for simulations
            let bool_know_priv_info: bool = false; 
            let possible_outcomes: Vec<ActionObservation> = history.generate_legal_moves();
            let path_t: String = self.path;
            // TOCHECK: In cases of collective action next, this makes no sense
            if self.is_chance_node() {
                // FOR SPECIAL CASES => {Collective Challenge, CollectiveBlock}
                // TODO: Free for all CollectiveChallenge and CollectiveBlock
                let latest_influence_time_t: &[u8; 6] = history.latest_influence();
                for player_id in 0..6 as usize {
                    if latest_influence_time_t[player_id] > 0 {
                        let key_ms_t: MSKey = MSKey::new(player_id, &self.path);
                        // TODO: check if q_values has key_ms_t
                        if !self.q_values.action_map_contains_key(&key_ms_t) {
                            self.q_values.update_action_map(&key_ms_t, &vec![ActionObservation::ChallengeAccept, ActionObservation::ChallengeDeny]);
                        }
                    }
                }
                // TODO: Initialise mixed strategy policy
            } else {
                // FOR NORMAL MOVES
                let player_id: usize = possible_outcomes[0].player_id();
                let key_ms_t: MSKey = MSKey::new(player_id, &self.path);
                // INITIALISING Q VALUES
                if !self.q_values.action_map_contains_key(&key_ms_t) {
                    self.q_values.update_action_map(&key_ms_t, &possible_outcomes);
                    let new_policy: HashMap<BRKey, f32> = HashMap::with_capacity(MAX_NUM_BRKEY);
                    for key in reach_prob.keys() {
                        new_policy.insert(key.clone(), vec![0.0; possible_outcomes.len()]);
                    }
                    self.q_values.policy_insert(key_ms_t.clone(), new_policy);
                }
                // TODO: check if q_values has key_ms_t
                // INITIALISING MIXED STRATEGY POLICY
                if !self.mixed_strategy_policy_vec.action_map_contains_key(&key_ms_t) {
                    self.mixed_strategy_policy_vec.action_map_insert(key_ms_t.clone(), possible_outcomes.clone());
                }
                if !self.mixed_strategy_policy_vec.policies_contains_key(&key_ms_t) {
                    let new_policy: HashMap<BRKey, f32> = HashMap::with_capacity(MAX_NUM_BRKEY);
                    for key in reach_prob.keys() {
                        new_policy.insert(key.clone(), vec![0.0; possible_outcomes.len()]);
                    }
                    self.mixed_strategy_policy_vec.policy_insert(key_ms_t.clone(), new_policy)
                }
                
            }
            if self.is_chance_node() {
                // TODO: SHIFT OUT AND USE ACTIONS
                // CollectiveChallenge, CollectiveBlock
                if let Some(action) = possible_outcomes.choose(&mut thread_rng()).cloned() {
                    let latest_influence_time_t: &[u8; 6] = history.latest_influence();
                    self.add_node(action, bool_know_priv_info);
                    // TODO: forward pass
                    // fwd pass needs mixed strategy policy working
                    // mix_strat_policy_time_t is a variable
                    self.policy_handler.forward_pass_best_response_policy(&self.path, latest_influence_time_t, &history, &mut self.best_response_policy_vec, mix_strat_policy_time_t);
                    self.pmccfr(depth_counter + 1);
                    // TODO: backpropogate
                    self.drop_node();
                }
            } else {
                // Initialise rewards to 0 for relevant infostates
                for key_br in reach_prob.keys() {
                    rewards.insert(key_br.clone(), 0.0);
                }

                let mut action_index: usize = 0;
                // LOOP THROUGH ALL ACTIONS AND RECURSE
                for action in possible_outcomes {

                    if action.name() == AOName::ExchangeDraw {
                        // Chance Node
                        if let Some(sampled_action) = self.naive_sample_exchange_draw(action.player_id()) {
                            self.add_node(sampled_action, bool_know_priv_info);
                            let next_reach_prob: HashMap<BRKey, bool> = HashMap::with_capacity(MAX_NUM_BRKEY);
                            // TODO: Fill reach prob based on input reachprob and action for pmccfr function
                            let transition_map: HashMap<String, Vec<String>> = HashMap::with_capacity(MAX_NUM_BRKEY);
                            // TODO: Fill transition hashmap for pmccfr function
                            let temp_reward: HashMap<BRKey, f32> = self.pmccfr(depth_counter + 1, next_reach_prob, Some(transition_map));
                            // TODO: backpropogate
                            self.drop_node();
                        } else {
                            self.node_counter += 1;
                        }
                    } else if action.name() == AOName::ExchangeChoice {
                        // Shuffle reach prob
                        let mut next_reach_prob: HashMap<BRKey, bool> = reach_prob.clone();
                        // Need to feed proper transition map
                        let mut valid_infostate = action.

                    } else if !self.naive_prune(&action) {
                        // NORMAL MOVE CASE
                        // all actions will be sampled

                        // How to update depends on the best_response_indicator stored in best_response_policy
                        // has 2 cases: updatespolicy_indicator for current player is 1 or 0

                        let latest_influence_time_t: &[u8; 6] = history.latest_influence();
                        // Add History and stuff to sample a possible move
                        let current_player: usize = history.latest_move().player_id();
                        next_history = history.clone().push_ao(action);
                        next_prob = prob.clone().push_ao(action);
                        self.add_node(action, bool_know_priv_info);
                        
                        // INITIALISING next_reach_prob based on action sampled to be passed into recursion
                        // TODO: Abstract this to a function
                        let mut next_reach_prob: HashMap<BRKey, bool> = reach_prob.clone();
                        // Filling next reach prob based on input reachprob and action for pmccfr function
                        //      If indicator == 1 and action is best response => 1
                        //      If indicator == 1 and action is not best response => 0
                        //      If indicator == 0 => 0
                        // TOCHECK: next player?
                        let next_player_id: usize = action.player_id();
                        for infostate in INFOSTATES {
                            let key_br: BRKey = BRKey::new(next_player_id, infostate);
                            if let Some(old_indicator) = reach_prob.get(&key_br){
                                if old_indicator == 1 {
                                    // Get best action or random if non yet!
                                    // TODO: Double check if mixed_strategy policy_ vec is correct, reference policy handler
                                    let infostate_best_response: ActionObservation = self.mixed_strategy_policy_vec.get_best_response(&key_ms_t, infostate);
                                    if !infostate_best_response == action {
                                        // change 1 to 0
                                        next_reach_prob.insert(key_br, false);
                                    } 
                                }
                            }
                        }
                        
                        // Filling transition hashmap for pmccfr function 1 to 1
                        // TODO: Abstract this to a function
                        // let mut transition_map: HashMap<String, Vec<String>> = HashMap::with_capacity(MAX_NUM_BRKEY);
                        // for key_br in next_reach_prob.keys() {
                        //     // 1 to 1 no change case
                        //     transition_map.insert(key_br.clone(), vec![key_br.clone()]);
                        // }

                        // Recurse and continue on an action
                        let temp_reward: HashMap<BRKey, f32> = self.pmccfr(depth_counter + 1, next_reach_prob, None);
                        
                        let move_player: usize = action.player_id();
                        let key_ms: MSKey = MSKey::new(move_player, &path_t);
                        
                        // UPDATE Q_VALUES BASED ON RETURNED REWARDS
                        // TODO: Abstract this to a function
                        if let Some(q_value) = self.q_values.policy_get_mut(&key_ms_t) {
                            for infostate in INFOSTATES {
                                let key_br: BRKey = BRKey::new(next_player_id, infostate);
                                // TODO: update q_value 
                                // TOCHECK: value updating
                                if let Some(values_vec) = q_value.get_mut(&key_br) {
                                    values_vec[action_index] += temp_reward[key_br];
                                }
                            }
                        } else {
                            panic!("q_values should have been initialised earlier!");
                        }
                        // UPDATE REWARDS for now if reach_prob = 1 we update
                            // Ignore the distinction for when current player is playing BR cos its complicated for infostates
                        // TODO: Abstract this to a function
                        for (key_br, value) in reach_prob {
                            // check fi all current_player == 0
                            if let Some(bool_val) = reach_prob.get(&key_br){
                                if bool_val {
                                    if let Some(temp_reward_val) = temp_reward.get(&key_br) {
                                        let increment_val: f32 = temp_reward_val;
                                    } else {
                                        let increment_val: f32 = 0.0;
                                    }
                                    if let Some(reward_val) = rewards.get_mut(&key_br){
                                        *reward_val += increment_val
                                    } else {
                                        rewards.insert(key_br.clone(), increment_val);
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
                        self.node_counter += 1;
                    }
                    action_index += 1;
                }
                // UPDATED MIXED STRATEGY POLICY based on action with highest Q value
                // Abstract to a function
                if [AOName::ForeignAid, AOName::Exchange, AOName::Assassinate, AOName::Steal, AOName::Tax, AOName::BlockAssassinate, AOName::BlockSteal].contains(&self.history.latest_move().name()) {
                    // CASE WHEN action == CollectiveChallenge or CollectiveBlock
                } else {
                    // MIXED STRATEGY POLICY should have been initialised earlier
                    let mskey_t: MSKey = MSKey::new(possible_outcomes[0].player_id(), &path_t);
                    for infostate in INFOSTATES {
                        if let Some(best_move_index) = self.q_values.get_best_response_index(&mskey_t, infostate) {
                            self.mixed_strategy_policy_vec.add_value(&mskey_t, infostate, best_move_index, 1.0);
                        } else {
                            panic!("q_values should be initialised properly!");
                        }
                    }
                }
            }
            // TODO: Backtransition here
        }
        if let Some(transition_map) = back_transition {
            let rewards_clone: HashMap<BRKey, f32> = HashMap::with_capacity(MAX_NUM_BRKEY);
            //  Temp
            for key_br in rewards.keys() {
                rewards_clone.insert(key_br.clone(), 0.0);
            }
            for (key_br, key_br_vec) in transition_map {
                for new_key_br in key_br_vec {
                    if let Some(value) = rewards_clone.get_mut(&new_key_br) {
                        *value += rewards[key_br];
                    }
                }
            }
        } else {
            rewards
        }
    }
}