use std::collections::HashMap;
use std::collections::HashSet;
use super::permutation_generator::{gen_table_combinations, gen_bag_combinations};
use super::coup_const::{BAG_SIZES, MAP_CONDHAND_INDEX, TOKENS, MAX_PERM_STATES};
use crate::string_utils::sort_str;
use crate::string_utils::{contains_all_chars, remove_chars};
use ndarray::{ArrayView1, ArrayView2};
use rayon::prelude::*;

#[derive(Debug)]
pub struct Constraints {
    // String not &str as it will not be known at compile time
    p0: Option<String>,
    p1: Option<String>,
    p2: Option<String>,
    p3: Option<String>,
    p4: Option<String>,
    p5: Option<String>,
}

impl Constraints {
    pub fn new() -> Self {
        Constraints{
            p0: None,
            p1: None,
            p2: None,
            p3: None,
            p4: None,
            p5: None,
        }
    }
    pub fn add(&mut self, id: usize ,constrain: &str){
        debug_assert!(id <= 5, "Please provide a proper id");
        let target = match id {
            0 => &mut self.p0,
            1 => &mut self.p1,
            2 => &mut self.p2,
            3 => &mut self.p3,
            4 => &mut self.p4,
            5 => &mut self.p5,
            _ => panic!("Invalid ID"), // Handled by the debug_assert! above.
        };
        // Use take() to replace target with None temporarily while working on it
        *target = match target.take(){
            None => Some(constrain.to_string()),
            Some(x) => {
                let mut chars: Vec<char> = x.chars().chain(constrain.chars()).collect();
                chars.sort_unstable();
                Some(chars.iter().collect())
            },
        };
        // TODO: Add a debug_assert! if constrain bursts
    }
    pub fn remove(&mut self, id: usize, constrain: &str){
        debug_assert!(id <= 5, "Invalid ID used in remove");
        
        let target = match id {
            0 => &mut self.p0,
            1 => &mut self.p1,
            2 => &mut self.p2,
            3 => &mut self.p3,
            4 => &mut self.p4,
            5 => &mut self.p5,
            _ => panic!("Invalid ID"), // Handled by the debug_assert! above.
        };

        if let Some(x) = target {
            if x == constrain {
                *target = None;
            } else if constrain.len() < x.len(){
                let mut result = x.clone();
                for c in constrain.chars() {
                    // Gets first occurance and removes it
                    if let Some(pos) = result.find(c) {
                        result.remove(pos);
                    }
                }
                *target = Some(result);
                
            } else {
                debug_assert!(false, "input constraint len too long! Longer than stored constraint!");
            }
        }
    }
    pub fn get_constraint(&self, id: usize) -> Option<&String>{
        debug_assert!(id <= 5, "Invalid ID used in get_constraint");
        match id {
            0 => self.p0.as_ref(),
            1 => self.p1.as_ref(),
            2 => self.p2.as_ref(),
            3 => self.p3.as_ref(),
            4 => self.p4.as_ref(),
            5 => self.p5.as_ref(),
            _ => None,
        }
    }
    pub fn satisfied(&self, deck_string: &str) -> bool {
        // Checks if deck_string satisfies constrains

        let constraints = [&self.p0, &self.p1, &self.p2, &self.p3, &self.p4, &self.p5];
        // Checks constraints for all players
        for (player_id, target) in constraints.iter().enumerate() {
            let index_start: usize = player_id * 2;
            let index_end: usize = index_start + 2;

            if let Some(constraint) = target {
                // Ensure index_end does not exceed deck_string length
                debug_assert!(index_end <= deck_string.len(), "Deck_string too small! or index_end too large!");

                let slice = &deck_string[index_start..index_end];

                // Check if the constraint is not satisfied
                if constraint.len() == 2 && slice != constraint {
                    return false;
                } else if constraint.len() == 1 && !slice.contains(constraint) {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Debug)]
pub struct ProbState {
    //TODO: Change to indexmap
    pub index_combi: HashMap<usize, String>,
    pub combi_index: HashMap<String, usize>,
    pub prob_store0: Vec<f64>,
    pub prob_store1: Vec<f64>,
    pub prob_store2: Vec<f64>,
    pub prob_store3: Vec<f64>,
    pub prob_store4: Vec<f64>,
    pub prob_store5: Vec<f64>,
    pub constraints: Constraints,
    pub save_state: bool,

}

// Should initialise with all 1.5 million states for 6 players
    // 6 seperate prob stores
        // Initialise prob should be 1/1.5mil
    // 1 bimap
// Constraints should be a symbolic representation not a list
    // So when 1/2/3 cards become public information computation needed can reduce by around 15x (if 1 player's full hand is shown there are only 100k possible permutations)
    // None "A" "AB"
    // Combinable
    // "A" + "B" => "AB" (ordered)
    // Constraints limit what can be created next
        // Make first node a special case of constraint?
    // So [Node 0] -> (Action 0) -> [Node 1]
        // [Node 1 should contain constraints for output of Action 0]
        // if Node 0 is the starting, it contains constraints for its own generation
// Save_State should be true / false
    // have a delete function
// Replay to be stored in manager not in probstate
    // Manager will track all macro features
// Transition functionality required | We check the probability based on the conditional probabilities
    // 1 to 1 Non Poll moves that do not shuffle cards => Income/Foreign Aid/Coup/Exchange/Steal/Assassinate/
    // Swap moves => ExchangeChoice
        // Own player ExchangeChoice is simple, its just a constraint, cos ur prob of doing action is 1, so 
        // Tocheck => if player0's size needs to be 1.5 million long or the constrained one based on private information
            // player0 size can be constrained to private information as p(h | s_{i}, beta_{-i}) includes infostate that has private information
    // Random Swap moves => RevealRedraw
        // This needs to be constrained further behind, then shuffle
        // The shuffle will be handled here
        // Constrained behind will be recalculated by manager
    // 1 to 1 Collective Poll moves => CollectiveChallenge CollectiveBlock
        // 6 probability choices but still 1 to 1
// Transition functions to borrow another ProbState and modify it
impl ProbState{
    pub fn new() -> Self {
        let mut index_combi = HashMap::new();
        let mut combi_index = HashMap::new();

        // Reserve space
        index_combi.reserve(MAX_PERM_STATES);
        combi_index.reserve(MAX_PERM_STATES);

        ProbState{
            index_combi,
            combi_index,
            prob_store0: Vec::with_capacity(MAX_PERM_STATES),
            prob_store1: Vec::with_capacity(MAX_PERM_STATES),
            prob_store2: Vec::with_capacity(MAX_PERM_STATES),
            prob_store3: Vec::with_capacity(MAX_PERM_STATES),
            prob_store4: Vec::with_capacity(MAX_PERM_STATES),
            prob_store5: Vec::with_capacity(MAX_PERM_STATES),
            // valid_next_states to reflect public information, include constraints on player life None, "A", "AB"
            constraints: Constraints::new(),
            // !!! Record death states
            // Something to record constraints
            save_state: false,
        }
    }
    pub fn set_save_state(&mut self, new_state: bool){
        self.save_state = new_state;
    }
    pub fn game_start(&mut self) {
        // TODO: Split into seperate function for 1-5 and one for p0
        // TODO LOW PRIO improve gen_table_combinations for when there are constraints 
        // game_start              time:   [2.1389 s 2.1478 s 2.1572 s]
        let hand_vec = gen_table_combinations(TOKENS, &BAG_SIZES);

        let prob: f64 = 1.0 / hand_vec.len() as f64;
        let mut count: usize = 0;
        for combi in hand_vec.iter(){
            // TODO: Concurrency
            // Creating Bimap
            if self.constraints.satisfied(combi){
                self.index_combi.insert(count, combi.clone());
                self.combi_index.insert(combi.clone(), count);
                // Inserting into prob_store
                self.prob_store0.push(prob);
                self.prob_store1.push(prob);
                self.prob_store2.push(prob);
                self.prob_store3.push(prob);
                self.prob_store4.push(prob);
                self.prob_store5.push(prob);
            }
            count += 1;
        }
    }
    pub fn game_start_same(&mut self) {
        //game_start_same         time:   [2.2315 s 2.2755 s 2.3221 s]
        let hand_vec = gen_table_combinations(TOKENS, &BAG_SIZES);

        // Sequentially populate index_combi and combi_index
        for combi in hand_vec.iter() {
            if self.constraints.satisfied(combi) {
                let index = self.index_combi.len(); // This ensures unique index based on current size
                self.index_combi.insert(index, combi.clone());
                self.combi_index.insert(combi.clone(), index);
            }
        }

        // The denom needs to be adjusted if the count of combinations is less than initially calculated
        let adjusted_denom = self.index_combi.len() as f64;

        // Prepare the probability value based on the actual count of combinations that satisfy constraints
        let prob = 1.0 / adjusted_denom;

        // Hopefully faster than iterating?
        let new_size = self.index_combi.len();
        self.prob_store0.resize(new_size, prob);
        self.prob_store1.resize(new_size, prob);
        self.prob_store2.resize(new_size, prob);
        self.prob_store3.resize(new_size, prob);
        self.prob_store4.resize(new_size, prob);
        self.prob_store5.resize(new_size, prob);
    }
    pub fn game_start_rayon(&mut self) {
        //game_start_rayon        time:   [2.1587 s 2.1675 s 2.1769 s]
        let hand_vec = gen_table_combinations(TOKENS, &BAG_SIZES);

        // Sequentially populate index_combi and combi_index
        for combi in hand_vec.iter() {
            if self.constraints.satisfied(combi) {
                let index = self.index_combi.len(); // This ensures unique index based on current size
                self.index_combi.insert(index, combi.clone());
                self.combi_index.insert(combi.clone(), index);
            }
        }

        // The denom needs to be adjusted if the count of combinations is less than initially calculated
        let adjusted_denom = self.index_combi.len() as f64;

        // Prepare the probability value based on the actual count of combinations that satisfy constraints
        let prob = 1.0 / adjusted_denom;

        // Step 2: Update probability stores - this can be parallelized
        self.push_prob_stores(prob);
    }
    fn push_prob_stores(&mut self, prob: f64) {
        // pushes in parrallel
        let prob_stores = [&mut self.prob_store0, &mut self.prob_store1, &mut self.prob_store2,
                           &mut self.prob_store3, &mut self.prob_store4, &mut self.prob_store5];

        prob_stores.into_par_iter()
                   .for_each(|store| {
                        for _ in 0..self.index_combi.len() {
                            store.push(prob);
                        }
                   });
    }
    // pub fn clone(&self) -> Self {
    //     Self {
    //         index_combi: self.index_combi.clone(),
    //         combi_index: self.combi_index.clone(),
    //         prob_store: self.prob_store.clone(),
    //         valid_next_states: self.valid_next_states.clone(),
    //     }
    // }

    // fn check_valid_next_state(&self, state: &String) -> bool{
    //     // Returns true of String is a valid next state to traverse to
        
    //     match &self.valid_next_states {
    //         Some(hashset) => {
    //             if hashset.contains(state) {
    //                 true
    //             } else {
    //                 false
    //             }
    //         },
    //         // None is an uninitialise hashset representing all states are possible
    //         // It does not mean that there are no possible future states (That is theoretically impossible)
    //         None => true,
    //     }
    // }

    // fn add_to_state(&mut self, probability: f64, state: &String) {
    //     // Adds to state if it does not yet exist
    //     // Also updates the relevant maps as required

    //     if self.combi_index.contains_key(state) {
    //         self.prob_store[*self.combi_index.get(state).unwrap()] += probability;
    //     } else {
    //         let new_index: usize = self.prob_store.len();
    //         self.combi_index.insert(state.clone(), new_index);
    //         self.index_combi.insert(new_index, state.clone());
    //         self.prob_store.push(probability);
    //     }
    // }

    // pub fn assign_constraint(&mut self, new_valid_next_states: Option<HashSet<String>>) {
    //     // Transmits possible state information backwards in time
    //     // Leaves calculations of how to modify the constraints for the outer interface
    //     self.valid_next_states = new_valid_next_states.clone();
    // }
    
    // pub fn assign_unconstrained(&mut self, prob_state: &ProbState) {
    //     // Make a copy
    //     // Modifies current state except for valid_next_states
    //     self.index_combi = prob_state.index_combi.clone();
    //     self.combi_index = prob_state.combi_index.clone();
    //     self.prob_store = prob_state.prob_store.clone();
    // }

    // pub fn restrict_constraint(&mut self, player_no: &usize, contains_cards: &String) {
    //     // This function helps us to register new information gained that limits the possible states at a current ProbState
    //     // e.g. if someone dies, and their cards are revealed.

    //     // Reduces the current valid_next_states set to include only states where player_no's bag contains contains_cards
    //     debug_assert!(*player_no <= 6 as usize && *player_no >= 0 as usize);
    //     debug_assert!(contains_cards.len() > 0 as usize && contains_cards.len() <= 2 as usize);

    //     let player_index_lower: usize = *player_no * 2 as usize;
    //     let player_index_upper: usize = *player_no * 2 as usize + if *player_no < (6 as usize) {2 as usize} else {3 as usize};

    //     // If the self.valid_next_states is None => the valid states is just the starting hand
    //     let valid_next_states: HashSet<String> = match &self.valid_next_states {
    //         Some(hashset) => hashset.clone(),
    //         None => HashSet::from(self.combi_index.keys().cloned().collect::<HashSet<String>>()),
    //     };

    //     // Keeping only those who fulfil the constraint
    //     self.valid_next_states = Some(valid_next_states.iter().filter(|s| {
    //         let bag:String = s[player_index_lower..player_index_upper].to_string();
    //         contains_all_chars(&bag, contains_cards)
    //     }).cloned().collect());
        
    // }

    // pub fn standard_move(&self, player_no: &usize, policy_cond_hand: ArrayView1<f64>) -> Self {
    //     // Takes policy conditioned on the hand of the player
    //     // Modifies the current State and produces a new one based on that
        
    //     let mut output = self.clone();
    //     let mut prob_multiply: f64;
    //     let mut cond_index: &usize;
    //     let mut player_hand: String;
    //     let mut state: &String;
    //     // player_no : [0, 5] inclusive
    //     let player_index: usize = player_no * 2;
    //     // Loop through prob_store
    //     for (index, prob_value) in output.prob_store.iter_mut().enumerate() {
    //         state = self.index_combi.get(&index).expect("State index does not exist");

    //         // Skip loop if there are constraints on what the output state can be
    //         match &self.valid_next_states {
    //             Some(valid_states) if !valid_states.contains(state) => continue,
    //             _ => {}
    //         }

    //         player_hand = match state.get(player_index..(player_index + 2)) {
    //             Some(hand) => hand.to_string(),
    //             None => panic!("Player Index out of bounds in State")
    //         };
    //         cond_index = MAP_CONDHAND_INDEX.get(player_hand.as_str()).expect("player_hand not in map");
    //         prob_multiply = policy_cond_hand[*cond_index];

    //         *prob_value *= prob_multiply;

    //     }
    //         // For each item, find the probability to multiply
    //         // Multiply current with new and multiply prob into the old figure
        
    //     output
    // }

    // // to proofread and add debug_assert
    // pub fn shuffle_choose(&self, player_no: &usize, policy_cond_hand: ArrayView2<f64>) -> Self {
    //     // This is a more specific function, because the general function would be too clunky
    //     // For now this will refer to other players i.e. not player 0

    //     // Don't need to propagate constraint's
    //     // In Coup Other player's moves do not swap your cards out anyways, so future state's will not enter impossible territory
    //     // One only needs to constraint the original move that yielded a certain outcome (As known via revealed information)

    //     // Policy_cond_hand should be a Vec<Vec<64>> Policy_cond_hand[The current hand of the player][Policy of which hand to choose]
    //     // P(Choose AB | hand == "CC") = policy_cond_hand[AB_index][CC_index]

    //     let mut output: ProbState = ProbState::new();
    //     let player_index_lower: usize = player_no * 2;
    //     let player_index_upper: usize = player_no * 2 + 2;
    //     let amb_index: usize = 6 * 2;
    //     let mut current_state: String;
    //     let mut current_hand: String;
    //     let mut potential_choices: Vec<String>;
    //     let mut choice_pool: String;
    //     let mut remainder: String;
    //     let mut new_state: String;
    //     let mut probability: f64;
    //     let mut denom_normalise: f64 = 0.0;
    //     let mut cond_hand_index0: usize;
    //     let mut cond_hand_index1: usize;

    //     // For each state in the ProbState
    //     for (index, value) in self.prob_store.iter().enumerate(){

    //         current_state = self.index_combi.get(&index).expect("prob_state::shuffle_choose index is not found in index_combi key").clone();
    //         // Consider 3 difference cases of 2 drawn cards
    //         current_hand = current_state[player_index_lower..player_index_upper].to_string();
    //         cond_hand_index0 = *MAP_CONDHAND_INDEX.get(current_hand.as_str()).unwrap();

    //         // Case 1 Amb player 6 | X X O | chosen
    //             // break down all possible combinations
    //             // Create new state String for each 
    //             // Assign Probability
                
    //         choice_pool = current_hand.clone() + &current_state[amb_index..(amb_index + 2)];
    //         potential_choices = gen_bag_combinations(&choice_pool, &2);
    //         debug_assert!(!potential_choices.is_empty());

    //         for choice in &potential_choices {
    //             // probability = policy_cond_hand[*MAP_CONDHAND_INDEX.get(&current_hand).unwrap()][*MAP_CONDHAND_INDEX.get(choice).unwrap()];
    //             cond_hand_index1 = *MAP_CONDHAND_INDEX.get(choice.as_str()).unwrap();
    //             probability = *policy_cond_hand.get((cond_hand_index0, cond_hand_index1)).expect("indexes out of range in ArrayView2");
    //             denom_normalise += probability;
    //         }
    //         for choice in &potential_choices {
    //             // Choice is naturally sorted in increasing order
    //             // Getting and sorting the ambassador hand after choice is made
    //             remainder = remove_chars(&choice_pool, choice).to_string() + &current_state[(amb_index + 2)..(amb_index + 3)];
    //             remainder = sort_str(&remainder);
                
    //             new_state = current_state[0..player_index_lower].to_string() + &choice + &current_state[(player_index_lower + 2)..amb_index] + &remainder;
    //             if self.check_valid_next_state(&new_state){
    //                 // probability = policy_cond_hand[*MAP_CONDHAND_INDEX.get(&current_hand).unwrap()][*MAP_CONDHAND_INDEX.get(choice).unwrap()];
    //                 cond_hand_index1 = *MAP_CONDHAND_INDEX.get(choice.as_str()).unwrap();
    //                 probability = *policy_cond_hand.get((cond_hand_index0, cond_hand_index1)).expect("indexes out of range in ArrayView2");
                    
    //                 output.add_to_state( value * probability / denom_normalise / 3.0, &new_state);
    //             }

    //         }
    //         // Case 2 Amb player 6 | O X X | chosen
    //             // break down all possible combinations
    //             // Create new state String for each 
    //             // Assign Probability
    //         denom_normalise = 0.0;
    //         choice_pool = current_hand.clone() + &current_state[(amb_index + 1)..(amb_index + 3)];
    //         potential_choices = gen_bag_combinations(&choice_pool, &2);
            
    //         for choice in &potential_choices {
    //             // probability = policy_cond_hand[*MAP_CONDHAND_INDEX.get(&current_hand).unwrap()][*MAP_CONDHAND_INDEX.get(choice).unwrap()];
    //             cond_hand_index1 = *MAP_CONDHAND_INDEX.get(choice.as_str()).unwrap();
    //             probability = *policy_cond_hand.get((cond_hand_index0, cond_hand_index1)).expect("indexes out of range in ArrayView2");
    //             denom_normalise += probability;
    //         }
    //         for choice in &potential_choices {
    //             // Choice is naturally sorted in increasing order
    //             // Getting and sorting the ambassador hand after choice is made
    //             remainder = remove_chars(&choice_pool, choice).to_string() + &current_state[amb_index..(amb_index + 1)];
    //             remainder = sort_str(&remainder);

    //             new_state = current_state[0..player_index_lower].to_string() + &choice + &current_state[(player_index_lower + 2)..amb_index] + &remainder;
    //             if self.check_valid_next_state(&new_state){
    //                 // probability = policy_cond_hand[*MAP_CONDHAND_INDEX.get(&current_hand).unwrap()][*MAP_CONDHAND_INDEX.get(choice).unwrap()];
    //                 cond_hand_index1 = *MAP_CONDHAND_INDEX.get(choice.as_str()).unwrap();
    //                 probability = *policy_cond_hand.get((cond_hand_index0, cond_hand_index1)).expect("indexes out of range in ArrayView2");
                
    //                 output.add_to_state( value * probability / denom_normalise / 3.0, &new_state);
    //             }
    //         }

    //         // Case 3 Amb player 6 | X O X | chosen
    //             // break down all possible combinations
    //             // Create new state String for each 
    //             // Assign Probability
    //         denom_normalise = 0.0;
    //         choice_pool = current_hand.clone() + &current_state[amb_index..(amb_index + 1)] + &current_state[(amb_index + 2)..(amb_index + 3)];
    //         potential_choices = gen_bag_combinations(&choice_pool, &2);
                
    //         for choice in &potential_choices {
    //             // probability = policy_cond_hand[*MAP_CONDHAND_INDEX.get(&current_hand).unwrap()][*MAP_CONDHAND_INDEX.get(choice).unwrap()];
    //             cond_hand_index1 = *MAP_CONDHAND_INDEX.get(choice.as_str()).unwrap();
    //             probability = *policy_cond_hand.get((cond_hand_index0, cond_hand_index1)).expect("indexes out of range in ArrayView2");
    //             denom_normalise += probability;
    //         }
    //         for choice in &potential_choices {
    //             // Choice is naturally sorted in increasing order
    //             // Getting and sorting the ambassador hand after choice is made
    //             remainder = remove_chars(&choice_pool, choice).to_string() + &current_state[(amb_index + 1)..(amb_index + 2)];
    //             remainder = sort_str(&remainder);

    //             new_state = current_state[0..player_index_lower].to_string() + &choice + &current_state[(player_index_lower + 2)..amb_index] + &remainder;
    //             if self.check_valid_next_state(&new_state){
    //                 // probability = policy_cond_hand[*MAP_CONDHAND_INDEX.get(&current_hand).unwrap()][*MAP_CONDHAND_INDEX.get(choice).unwrap()];
    //                 cond_hand_index1 = *MAP_CONDHAND_INDEX.get(choice.as_str()).unwrap();
    //                 probability = *policy_cond_hand.get((cond_hand_index0, cond_hand_index1)).expect("indexes out of range in ArrayView2");
                    
    //                 output.add_to_state( value * probability / denom_normalise / 3.0, &new_state);
    //             }
    //         }
    //     }
    //     output
    // }

    // // To proof read
    // pub fn shuffle_card(&self, player_no: &usize, card: &String) -> Self {
    //     // Swaps a player's card into the deck and replaces it with a random one
    //     // This assumes that player has the card
    //     // Constraint must be properly changed before this!!! So current states are legit
        
    //     // Prob can just be 1 cos it doesnt matter or maybe not...
    //     // Put error for needing to constraint before shuffling so the card exists
    //     debug_assert!(self.combi_index.keys().all(|key| {
    //         let c0 = key.chars().nth(player_no * 2 as usize).unwrap();
    //         let c1 = key.chars().nth(player_no * 2 as usize + 1 as usize).unwrap();
    //         c0 == card.chars().nth(0).unwrap() || c1 == card.chars().nth(0).unwrap()
    //     }));
    //     let mut output: ProbState = ProbState::new();
    //     let player_index_lower: usize = player_no * 2 as usize;
    //     let player_index_upper: usize = player_no * 2 + 2 as usize;
    //     let amb_index_lower: usize = (6 * 2) as usize;
    //     let amb_index_upper: usize = (6 * 2 + 3) as usize;
    //     let mut current_hand: String;
    //     let mut new_hand: String;
    //     let mut new_amb: String;
    //     let mut card_pool: String;
    //     let mut state: String;
    //     let mut new_state: String;

    //     for (index, old_probability) in self.prob_store.iter().enumerate() {
    //         state = self.index_combi.get(&index).unwrap().clone();
    //         current_hand = state[player_index_lower..player_index_upper].to_string();
    //         card_pool = card.clone() + &state[amb_index_lower..amb_index_upper];
    //         new_hand = remove_chars(&current_hand, &card.to_string());
    //         for (ipool, vpool) in card_pool.char_indices(){
    //             new_hand += &vpool.to_string();
    //             new_hand = sort_str(&new_hand);
    //             debug_assert!(new_hand.len() == 2);

    //             new_amb = card_pool[0..ipool].to_string() + &card_pool[(ipool + 1 as usize)..];
    //             new_amb = sort_str(&new_amb);
    //             debug_assert!(new_amb.len() == 3);

    //             new_state = state[0..player_index_lower].to_string() + &new_hand + &state[player_index_upper..amb_index_lower] + &new_amb;
    //             debug_assert!(new_state.len() == 15);

    //             if self.check_valid_next_state(&new_state){
    //                 output.add_to_state(((1.0 / 4.0)  as f64) * old_probability, &new_state);
    //             }
    //         }
    //     }
    //     output
    // }
}