use rand::prelude::SliceRandom;
use rand::thread_rng;
use crate::prob_manager::naive_prob::NaiveProb;
use crate::prob_manager::naive_sampler::NaiveSampler;
use crate::history_public::{AOName, ActionObservation, Card, History};
use crate::prob_manager::constraint::CollectiveConstraint;
use super::action_serialiser::{DefaultEmbedding, ActionEmbedding};
// use super::best_response_policy::BRKey;
use std::collections::HashMap;
struct Explorer<'a> {
    // This is a struct used to conduct Pure Monte Carlo CounterFactual Regret Minimization (PMCCFR)
    path: String,
    action_embedder: Box<dyn ActionEmbedding>,
    history: History,
    prob: NaiveProb<'a>,
    chance_sampler: NaiveSampler<'a>,
    max_depth: usize,
    bool_monte_carlo: bool,
    node_counter: u128,
    visit_counter: u128,
    // TO Document best_response_policy
    // best_response_policy_vec: Vec<HashMap<BRKey, Option<Vec<bool>>>>,
}

impl <'a> Explorer<'a> {
    pub fn new(max_depth: usize) -> Self {
        // make this into config HashMap when done
        Explorer{
            // temporarily the starting_player is 0
            path: "root".to_string(),
            action_embedder: Box::new(DefaultEmbedding),
            history: History::new(0),
            prob: NaiveProb::new(),
            chance_sampler: NaiveSampler::new(),
            max_depth,
            bool_monte_carlo: true,
            node_counter: 0,
            visit_counter: 0,
            // best_response_policy: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.path = "root".to_string();
        self.history.reset();
        self.prob.reset();
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

    use std::time::Instant;
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

    use std::time::Instant;
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

    use std::time::Instant;
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

    use std::time::Instant;
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