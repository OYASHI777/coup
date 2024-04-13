use std::collections::{HashMap, HashSet};
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

// Just update the value

// Create functions that generate next reach probability
// And a transition back
// Look into how to store BR
pub struct ReachProbability {
    // Either one of these
    // Use 1 if we do not care between Not possible and not reached states (according to current BR profile)

    player_reach: Vec<HashSet<String>>,
    // Use 2 if we need to use the Not possible states and Not reached (according to current BR profile) states separately
    // player_reach2: Vec<HashMap<String, bool>>,
    // Will need to store transition i guess?
    // Should make a store for all the policies
    // Using a vec is hardly faster because the sizes are small <5% speedup
    player_reach_vec: Vec<Vec<String>>,
    // Choose either, or choose later when decided... I kinda like the HashSet more, as I can add and remove easily
}

impl ReachProbability {
    pub fn initialise() -> Self {
        let mut player_reach: Vec<HashSet<String>> = Vec::with_capacity(6);
        let mut player_reach_vec: Vec<Vec<String>> = Vec::with_capacity(6);
        let card_arr: [&str; 15] = ["AA", "AB", "AC", "AD", "AE", "BB", "BC", "BD", "BE", "CC", "CD", "CE", "DD", "DE", "EE"];
        for _ in 0..6 {
            let mut player_set: HashSet<String> = HashSet::with_capacity(15); // Change later
            for card_str in card_arr.iter(){
                player_set.insert(card_str.to_string());
            }
            player_reach.push(player_set);
            let player_vec: Vec<String> = card_arr.iter().map(|&card_str| card_str.to_string()).collect();
            player_reach_vec.push(player_vec);
        }
        Self {
            player_reach,
            player_reach_vec,
        }
    }
    pub fn modify_player_set(&mut self, player_id: usize, new_set: HashSet<String>) {
        if player_id < self.player_reach.len() {
            self.player_reach[player_id] = new_set;
        } else {
            panic!("Player ID out of range");
        }
    }
    pub fn modify_player_vec(&mut self, player_id: usize, new_vec: Vec<String>) {
        if player_id < self.player_reach.len() {
            self.player_reach_vec[player_id] = new_vec;
        } else {
            panic!("Player ID out of range");
        }
    }

    pub fn add_to_player_set(&mut self, player_id: usize, hand: String) {
        if player_id < self.player_reach.len() {
            self.player_reach[player_id].insert(hand);
        } else {
            panic!("Player ID out of range");
        }
    }

    pub fn remove_from_player_set(&mut self, player_id: usize, hand: &str) {
        if player_id < self.player_reach.len() {
            self.player_reach[player_id].remove(hand);
        } else {
            panic!("Player ID out of range");
        }
    }

    pub fn print_log(&self) {
        println!("Reach Probability:");
        for (idx, set) in self.player_reach.iter().enumerate() {
            println!("Player {}: {:?}", idx, set);
        }
    }
    pub fn wrong_naive_prune(&self ) -> bool {
        // This is wrong because it prunes even when a player has 1. We still want to search if someone has 1.
        // Naive_prune checks if pi_-i (s_i) = 0
        // This is the probability that under the BR policy, any players will play to the current state
            // assuming your actions play towards the current state
        // This is important if for some state, a player has reach prob of 0, and pi_-i(s_i) == 0, we prune the node!
        // This is the reach probability of being in the current state for each player
        //      Player
        //     0  1  2  3  4  5
        // AA [0, 1, 1, 0, 0, 1]
        // AB [1, 0, 1, 0, 1, 1]
        // AC [0, 1, 1, 0, 0, 0]
        // BB [1, 1, 0, 0, 1, 1]
        // BC [0, 0, 1, 0, 0, 0]
        // CC [1, 0, 1, 1, 0, 1]
        // naive prune returns true if there is some combination of valid hands eg. "AAABBBCCCDDDEEE" that is valid, 
            // and all players in that have a reach probability of 1
            // In a best response policy, players play their best moves only so they are either 1 or 0 as in above
            // The reachprob is thus stored as a HashSet of hands that have 1 as value for some player.
        // A combination of hands is valid if they satisfy the constraints of the game:
            // For Coup: this just means there cannot be more than 3 of each card

        // This takes max Time: of 200ms and min time of 6µs (1st item is true)
            // This is actually alright as in the case of 200ms we save more future time of check leaf nodes
            // This is also rarely the case as it would involve 15 states for each player leading to unreachable states, which is unrealistic
        // The extreme case almost never happens as impossible states are slowly removed due to public info like discard etc.
        // Ive only seen 16ms at worst
        // IF everyone has roughly 9 hands its still 3.2µs
        
        // In general this will take a long time if the product of the number of states for all player is high, 
        // and the node will not be reached (and so we have to prune) 
        // But in practice to construct such a thing is hard
            // Having more states => more likely that a reach exists => low run time
        // Also in practice the number of states wont be high
            // At some turn, for the player choosing the next action
            // for each state, they choose a Best Response (BR) which is just one action a
            // If that action is chosen, the next history's reach probability for that state is 1 else 0 or and Impossible case
            // So for some reach probability, the states in the HashSet will only be those whose best response in the previous round is to choose that action
            // Given many actions, it is unlikely that there are many that are 1 in the first place
            // Might also need to store a HashMap<> with the zeroes for seperability but the prune remains the same (i think)

        // Normally you would check if they would play to the next state, 
        // but You can just create the next reach_prob by transitioning it
        // and run this function So its the same

        //TODO: store this in self dynamically, and reset when finished calculating!
        let mut card_count: HashMap<String, usize> = HashMap::new();
        card_count.insert("A".to_string(), 0);
        card_count.insert("B".to_string(), 0);
        card_count.insert("C".to_string(), 0);
        card_count.insert("D".to_string(), 0);
        card_count.insert("E".to_string(), 0);
        return self.recur_check(&mut card_count, 0);

    }
    pub fn recur_check(&self, card_count: &mut HashMap<String, usize>, player_id: usize) -> bool {
        println!("card_count in node {} {:?}", player_id , card_count);
        if player_id == 5 {
            'hand: for hand in &self.player_reach[player_id]{
                let mut temp_hm: HashMap<String, usize> = HashMap::new();
                for letter in hand.chars(){
                    *temp_hm.entry(letter.to_string()).or_insert(0) += 1;
                }
                for letter in hand.chars() {
                    // Adding to it makes an impossible case
                    if (card_count[&letter.to_string()] + temp_hm[&letter.to_string()]) > 3 {
                        continue 'hand;
                    }
                }
                // Testing how long it takes to run please return true for actual
                return true;
            }
            return false;
        }
        'hand: for hand in &self.player_reach[player_id]{
            let mut temp_hm: HashMap<String, usize> = HashMap::new();
            for letter in hand.chars(){
                *temp_hm.entry(letter.to_string()).or_insert(0) += 1;
            }
            for letter in hand.chars() {
                // Adding to it makes an impossible case
                if (card_count[&letter.to_string()] + temp_hm[&letter.to_string()]) > 3 {
                    continue 'hand;
                }
            }
            // Updating letters if ok
            for letter in hand.chars(){
                if let Some(count) = card_count.get_mut(&letter.to_string()){
                    *count += 1;
                } else {
                    panic!("card not found!");
                }
            }
            // Recurse
            if self.recur_check(card_count, player_id + 1) {
                return true;
            }
            // Reset and try next
            for letter in hand.chars(){
                if let Some(count) = card_count.get_mut(&letter.to_string()){
                    *count -= 1
                } else {
                    panic!("card not found!");
                }
            } 
        }
        false
    }
    pub fn info_state_recur_check(&self, card_count: &mut HashMap<String, usize>, id_vec: &Vec<usize>) -> bool {
        println!("card_count in node {:?}", card_count);
        let player_id = id_vec[0];
        if id_vec.len() == 1 {
            'hand: for hand in &self.player_reach[player_id]{
                let mut temp_hm: HashMap<String, usize> = HashMap::new();
                for letter in hand.chars(){
                    *temp_hm.entry(letter.to_string()).or_insert(0) += 1;
                }
                for letter in hand.chars() {
                    // Adding to it makes an impossible case
                    if (card_count[&letter.to_string()] + temp_hm[&letter.to_string()]) > 3 {
                        continue 'hand;
                    }
                }
                // Only returns if possible case is found
                return true;
            }
            return false;
        }
        // Remove once only at the start
        let mut next_vec: Vec<usize> = id_vec.clone();
        next_vec.swap_remove(0);
        'hand: for hand in &self.player_reach[player_id]{
            let mut temp_hm: HashMap<String, usize> = HashMap::new();
            for letter in hand.chars(){
                *temp_hm.entry(letter.to_string()).or_insert(0) += 1;
            }
            for letter in hand.chars() {
                // Adding to it makes an impossible case
                if (card_count[&letter.to_string()] + temp_hm[&letter.to_string()]) > 3 {
                    continue 'hand;
                }
            }
            // Updating letters if ok
            for letter in hand.chars(){
                if let Some(count) = card_count.get_mut(&letter.to_string()){
                    *count += 1;
                } else {
                    panic!("card not found!");
                }
            }
            // Recurse
            if self.info_state_recur_check(card_count, &next_vec) {
                return true;
            }
            // Reset and try next
            for letter in hand.chars(){
                if let Some(count) = card_count.get_mut(&letter.to_string()){
                    *count -= 1
                } else {
                    panic!("card not found!");
                }
            } 
        }
        false
    }
    pub fn info_state_prune(&self, player_id: usize, hand: String) -> bool {
        // We prune a particular info_state (so we do not keep updating it but leave its propogated value to be 0)
        // If the player's infostate reach prob = 0, we check if pi_-i (s_i) = 0
        // If it is, we no longer update the value for that infostate
        // See wrong_naive_prune for an illustration
        let mut other_player_vec: Vec<usize> = Vec::new();
        for i in 0..6 {
            if i != player_id {
                other_player_vec.push(i);
            }
        }
        let mut card_count: HashMap<String, usize> = HashMap::new();
        card_count.insert("A".to_string(), 0);
        card_count.insert("B".to_string(), 0);
        card_count.insert("C".to_string(), 0);
        card_count.insert("D".to_string(), 0);
        card_count.insert("E".to_string(), 0);
        for letter in hand.chars(){
            *card_count.entry(letter.to_string()).or_insert(0) += 1;
        }
        return self.info_state_recur_check(&mut card_count, &other_player_vec);
    }
    pub fn node_prune(&self) -> bool {
        // Checks if all states have reach prob of 0 or are not possible to reach
        // This is basically that all states do not have a prob of one, stored in player_reach
        // This is pruned because no player, in any infostate, in the Best Response Policy would ever play to this state
        for hashset in self.player_reach.iter(){
            if !hashset.is_empty(){
                return false;
            }
        }
        true
    }
    pub fn info_state_recur_check_vec(&self, card_count: &mut HashMap<String, usize>, id_vec: &Vec<usize>) -> bool {
        println!("card_count in node {:?}", card_count);
        let player_id = id_vec[0];
        if id_vec.len() == 1 {
            'hand: for hand in &self.player_reach_vec[player_id]{
                let mut temp_hm: HashMap<String, usize> = HashMap::new();
                for letter in hand.chars(){
                    *temp_hm.entry(letter.to_string()).or_insert(0) += 1;
                }
                for letter in hand.chars() {
                    if (card_count[&letter.to_string()] + temp_hm[&letter.to_string()]) > 3 {
                        continue 'hand;
                    }
                }
                return true;
            }
            return false;
        }
        let mut next_vec: Vec<usize> = id_vec.clone();
        next_vec.swap_remove(0);
        'hand: for hand in &self.player_reach_vec[player_id]{
            let mut temp_hm: HashMap<String, usize> = HashMap::new();
            for letter in hand.chars(){
                *temp_hm.entry(letter.to_string()).or_insert(0) += 1;
            }
            for letter in hand.chars() {
                if (card_count[&letter.to_string()] + temp_hm[&letter.to_string()]) > 3 {
                    continue 'hand;
                }
            }
            for letter in hand.chars(){
                if let Some(count) = card_count.get_mut(&letter.to_string()){
                    *count += 1;
                } else {
                    panic!("card not found!");
                }
            }
            if self.info_state_recur_check(card_count, &next_vec) {
                return true;
            }
            for letter in hand.chars(){
                if let Some(count) = card_count.get_mut(&letter.to_string()){
                    *count -= 1
                } else {
                    panic!("card not found!");
                }
            } 
        }
        false
    }

    pub fn info_state_prune_vec(&self, player_id: usize, hand: String) -> bool {
        let mut other_player_vec: Vec<usize> = Vec::new();
        for i in 0..6 {
            if i != player_id {
                other_player_vec.push(i);
            }
        }
        let mut card_count: HashMap<String, usize> = HashMap::new();
        for letter in "ABCDE".chars(){
            card_count.insert(letter.to_string(), 0);
        }
        for letter in hand.chars(){
            *card_count.entry(letter.to_string()).or_insert(0) += 1;
        }
        return self.info_state_recur_check(&mut card_count, &other_player_vec);
    }
}


// struct PCMCCFR {
//     state_reach_probability: Vec<HashMap<String, >>
// }
