use std::sync::Mutex;
use ahash::AHashMap;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use rand::prelude::SliceRandom;
use super::constraint::{GroupConstraint, CollectiveConstraint};
use crate::{cfr::keys::Infostate, history_public::Card};
use dashmap::DashMap;
use crate::cfr::keys::INFOSTATES;

#[derive(Debug)]
pub struct NaiveSampler<'a> {
    // struct that randomly samples an action
    store: HashMap<&'a str, Vec<&'a str>>,
}

impl<'a> NaiveSampler<'a> { 
    pub fn new() -> Self {
        let mut store: HashMap<&str, Vec<&str>> = HashMap::new();

        let a = vec!["AA", "AB", "AC", "AD", "AE"];
        let b = vec!["AB", "BB", "BC", "BD", "BE"];
        let c = vec!["AC", "BC", "CC", "CD", "CE"];
        let d = vec!["AD", "BD", "CD", "DD", "DE"];
        let e = vec!["AE", "BE", "CE", "DE", "EE"];
        let blank = vec!["AA", "AB", "AC", "AD", "AE", "BB", "BC", "BD", "BE", "CC", "CD", "CE", "DD", "DE", "EE"];

        store.insert("A", a);
        store.insert("B", b);
        store.insert("C", c);
        store.insert("D", d);
        store.insert("E", e);
        store.insert("_", blank);
        store.insert("AA", vec!["AA"]);
        store.insert("AB", vec!["AB"]);
        store.insert("AC", vec!["AC"]);
        store.insert("AD", vec!["AD"]);
        store.insert("AE", vec!["AE"]);
        store.insert("BB", vec!["BB"]);
        store.insert("BC", vec!["BC"]);
        store.insert("BD", vec!["BD"]);
        store.insert("BE", vec!["BE"]);
        store.insert("CC", vec!["CC"]);
        store.insert("CD", vec!["CD"]);
        store.insert("CE", vec!["CE"]);
        store.insert("DD", vec!["DD"]);
        store.insert("DE", vec!["DE"]);
        store.insert("EE", vec!["EE"]);
        NaiveSampler{
            store,
        }
    }
    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        for key in ["A", "B", "C", "D", "E", "_"].iter() {
            if let Some(list) = self.store.get_mut(*key) {
                list.shuffle(&mut rng);
            }
        }
    }
    pub fn par_constructor(&mut self, constraint: &CollectiveConstraint) -> Option<String> {
        // Returns None if impossible state, and a random String if its possible 
        // mostly microseconds 3ms max
        // let start_time = Instant::now();

        self.shuffle();
        // create 6 pointers 1 for each player, to point to the vector the player references
        let mut pointers: Vec<&Vec<&str>> = vec![&self.store["_"]; 6]; 

        // Run checks for each player_id to assign the right vector of Strings to them
        for player_id in 0..6 {
            if let Some(card) = constraint.get_pc_hm().get(&player_id){
                // Assign the player's pointer to the relevant vector
                // This should be the key to find the vector in store
                pointers[player_id] = self.store.get(card.card_to_str()).unwrap_or(&self.store["_"]);
            } else if let Some(card_vec) = constraint.get_jc_hm().get(&player_id){
                //convert card_vec to a Vec of 1 string using card_to_char()
                // Assign a pointer to this
                let mut key_vec: Vec<char> = card_vec.iter().map(Card::card_to_char).collect();
                key_vec.sort_unstable();
                let key: String = key_vec.iter().collect();
                pointers[player_id] = self.store.get(key.as_str()).unwrap_or(&self.store["_"]);

            }
        }

        // Nested for loops
        // Higher values makes longer tasks shorter but shorter tasks take more time!
        let num_splits: usize;
        let max_splits = 2;
        if pointers[0].len() < max_splits {
            num_splits = pointers[0].len();
        } else {
            num_splits = max_splits;
        }
        let chunk_size: usize = pointers[0].len() / num_splits;
        let result: Mutex<Option<String>> = Mutex::new(None);
        rayon::scope(|s| {
            for i in 0..num_splits {
                let start: usize = i * chunk_size;
                let end: usize = if i == num_splits - 1 { pointers[0].len() } else { start + chunk_size };
                let slice: &[&str] = &pointers[0][start..end];
        
                s.spawn(|_| {
                    let res: Option<String> = self.search_in_half(slice, &constraint, &pointers);
                    let mut result_lock = result.lock().unwrap();
                    if result_lock.is_none() && res.is_some() {
                        *result_lock = res;
                    }
                });
            }
        });

        let final_result: Option<String> = result.into_inner().unwrap();
        // let duration = start_time.elapsed();
        // println!("Duration = {:?}", duration);
        final_result
    }

    pub fn hm_constructor(&mut self, constraint: &CollectiveConstraint, player: u8, infostates: &Vec<Infostate>) -> AHashMap<String, String> {
        // returns a map of player infostate to 2 randomly drawn card from pile (last 3 cards)
        // TODO: Optimize, this shit increases avg time per node by 20+ microseconds
        // TODO: Change, needs to manually do...
        self.shuffle();
        let mut result: AHashMap<String, String> = AHashMap::with_capacity(INFOSTATES.len());
        let player_id: usize = player as usize;
        for infostate in infostates {
            let card_vec: Vec<Card> = infostate.to_vec_card();
            // DO NOT MOVE TEMP_CONSTRAINT OUT OF LOOP AS THAT WILL MESS THINGS UP
            let mut temp_constraint: CollectiveConstraint = constraint.clone();
            temp_constraint.remove_constraints(player_id);
            temp_constraint.add_joint_constraint(player_id, &card_vec);
            if let Some(card_str) = self.par_constructor(&mut temp_constraint) {
                let mut rng = rand::thread_rng();
                let random_number: f64 = rng.gen();
                let mut chosen_cards: Vec<char> = if random_number < 0.33333333333 {
                    vec![card_str.chars().nth(13).unwrap(), card_str.chars().nth(14).unwrap()]
                } else if random_number < 0.6666666666 {
                    vec![card_str.chars().nth(12).unwrap(), card_str.chars().nth(14).unwrap()]
                } else {
                    vec![card_str.chars().nth(12).unwrap(), card_str.chars().nth(13).unwrap()]
                };
                chosen_cards.sort_unstable();
                let chosen_cards_str: String = chosen_cards.into_iter().collect();
                result.insert(infostate.to_str().to_string(), chosen_cards_str);
            } else {
                // println!("Generation failed for player: {:?}", player);
                // temp_constraint.print();
                // debug_assert!(false, "Generation failed");
            }
        }
        result
    }

    fn search_in_half(&self, half: &[&str], constraint: &CollectiveConstraint, pointers: &Vec<&Vec<&str>>) -> Option<String> {
        let mut counter_hm: HashMap<&str, usize> = HashMap::new();
        counter_hm.insert("A", 0);
        counter_hm.insert("B", 0);
        counter_hm.insert("C", 0);
        counter_hm.insert("D", 0);
        counter_hm.insert("E", 0);
        for &card0 in half {
            // increment counter_hm based on &card0 cards
            if self.increment_continue(&card0, &mut counter_hm) {
                continue;
            }
            for &card1 in pointers[1] {
                // Check if current string when incremented into counter_hm would make counter > 3
                // if larger than 3 continue;
                // else increment
                if self.increment_continue(&card1, &mut counter_hm) {
                    continue;
                }
                for &card2 in pointers[2] {
                    // Check if current string when incremented into counter_hm would make counter > 3
                    // if larger than 3 continue;
                    // else increment
                    if self.increment_continue(&card2, &mut counter_hm) {
                        continue;
                    }
                    for &card3 in pointers[3] {
                        // Check if current string when incremented into counter_hm would make counter > 3
                        // if larger than 3 continue;
                        // else increment
                        if self.increment_continue(&card3, &mut counter_hm) {
                            continue;
                        }              
                        for &card4 in pointers[4] {
                            // Check if current string when incremented into counter_hm would make counter > 3
                            // if larger than 3 continue;
                            // else increment            
                            if self.increment_continue(&card4, &mut counter_hm) {
                                continue;
                            }            
                            for &card5 in pointers[5] {
                                // Check if current string when incremented into counter_hm would make counter > 3
                                // if larger than 3 continue;
                                // else increment
                                if self.increment_continue(&card5, &mut counter_hm) {
                                    continue;
                                }
                                let mut card6 = String::new();
                                for (&card_type, &count) in counter_hm.iter() {
                                    let remaining = 3 - count; // Calculate how many more of this card type are needed
                                    for _ in 0..remaining {
                                        card6.push_str(card_type); // Append the card type as many times as needed
                                    }
                                }
                                let mut chars: Vec<char> = card6.chars().collect(); // Convert string to vector of characters
                                chars.sort(); // Sort the vector in ascending order
                                card6 = chars.into_iter().collect();
                                //infer &card6 to point to a 3 digit string
                                // this is inferred by the remaining counts in counter_hm
                                // Since all of counter_hm must be 3, the remaining strings are just the keys from counter_hm 
                                // such that they all get incremented to 3

                                // submit &card0 &card1 ... &card6 to a function that determines if they are legal
                                // this function returns an Option<String>
                                // return value is not none, return the Option<String>
                                                        // Check if the entire combination is legal, including card6
                                if let Some(result) = self.check_if_legal(&[card0, card1, card2, card3, card4, card5, &card6], constraint) {
                                    return Some(result);
                                }
                                self.decrement(&card5, &mut counter_hm);
                                // decrement counter_hm based on &card5 cards
                            }
                            self.decrement(&card4, &mut counter_hm);
                            // decrement counter_hm based on &card4 cards
                        }
                        self.decrement(&card3, &mut counter_hm);
                        // decrement counter_hm based on &card3 cards
                    }
                    self.decrement(&card2, &mut counter_hm);
                    // decrement counter_hm based on &card2 cards
                }
                self.decrement(&card1, &mut counter_hm);
                // decrement counter_hm based on &card1 cards
            }
            self.decrement(&card0, &mut counter_hm);
            // decrement counter_hm based on &card0 cards
        }
        None
    }
    fn check_if_legal(&self, cards: &[&str], constraint: &CollectiveConstraint) -> Option<String>{
        // Checks of string cards produced are fulfil the constraints
        // Check only gc_vec constraints as pc_hm and jc_hm are fulfilled in construction
        for gc in constraint.get_gc_vec().iter() {
            let participation_list: &[u8; 7] = gc.get_list();  // Assume this returns a &[u8; 7]
            let card_char: char = gc.card().card_to_char();  // Get the card character for this constraint
            let required_count: usize = gc.count();  // Required count of this card character
            let mut total_count: usize = 0;

            // Loop through the participation list
            for (i, &participation) in participation_list.iter().enumerate() {
                if participation == 1 { 
                    total_count += cards[i].matches(card_char).count();
                }
            }
            // Check if the total count meets or exceeds the required count
            if total_count < required_count {
                return None;  // If any constraint is not met, return None
            }
        }
        Some(cards.join(""))
    }
    fn increment_continue(&self, str_ref: &str, counter_hm: &mut HashMap<&str, usize> ) -> bool {
        // Takes a hashmap and increments according to str_ref value
        // If an impossible str_ref value is given, true is returned and nothing is incremented
        if str_ref == "AA" {
            if counter_hm["A"] > 1{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("A"){
                    *value += 2;
                }
            }
        } else if str_ref == "AB" {
            if counter_hm["A"] > 2 || counter_hm["B"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("A"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("B"){
                    *value += 1;
                }
            }
        } else if str_ref == "AC" {
            if counter_hm["A"] > 2 || counter_hm["C"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("A"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("C"){
                    *value += 1;
                }
            }
        } else if str_ref == "AD" {
            if counter_hm["A"] > 2 || counter_hm["D"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("A"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("D"){
                    *value += 1;
                }
            }
        } else if str_ref == "AE" {
            if counter_hm["A"] > 2 || counter_hm["E"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("A"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("E"){
                    *value += 1;
                }
            }
        } else if str_ref == "BB" { 
            if counter_hm["B"] > 1{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("B"){
                    *value += 2;
                }
            }
        } else if str_ref == "BC" { 
            if counter_hm["B"] > 2 || counter_hm["C"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("B"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("C"){
                    *value += 1;
                }
            }
        } else if str_ref == "BD" { 
            if counter_hm["B"] > 2 || counter_hm["D"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("B"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("D"){
                    *value += 1;
                }
            }
        } else if str_ref == "BE" { 
            if counter_hm["B"] > 2 || counter_hm["E"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("B"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("E"){
                    *value += 1;
                }
            }
        } else if str_ref == "CC" {
            if counter_hm["C"] > 1{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("C"){
                    *value += 2;
                }
            }
        } else if str_ref == "CD" {
            if counter_hm["C"] > 2 || counter_hm["D"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("C"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("D"){
                    *value += 1;
                }
            }
        } else if str_ref == "CE" {
            if counter_hm["C"] > 2 || counter_hm["E"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("C"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("E"){
                    *value += 1;
                }
            }
        } else if str_ref == "DD" {
            if counter_hm["D"] > 1{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("D"){
                    *value += 2;
                }
            }
        } else if str_ref == "DE" {
            if counter_hm["D"] > 2 || counter_hm["E"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("D"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("E"){
                    *value += 1;
                }
            }
        } else if str_ref == "EE" {
            if counter_hm["E"] > 1{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("E"){
                    *value += 2;
                }
            }
        }
        false
    }
    fn decrement(&self, str_ref: &str, counter_hm: &mut HashMap<&str, usize> ) -> bool {
        // Takes a HashMap and decrements according to str_ref value
        if str_ref == "AA" {
            if let Some(value) = counter_hm.get_mut("A"){
                *value -= 2;
            }
        } else if str_ref == "AB" {
            if let Some(value) = counter_hm.get_mut("A"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("B"){
                *value -= 1;
            }
        } else if str_ref == "AC" {
            if let Some(value) = counter_hm.get_mut("A"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("C"){
                *value -= 1;
            }
        } else if str_ref == "AD" {
            if let Some(value) = counter_hm.get_mut("A"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("D"){
                *value -= 1;
            }
        } else if str_ref == "AE" {
            if let Some(value) = counter_hm.get_mut("A"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("E"){
                *value -= 1;
            }
        } else if str_ref == "BB" { 
            if let Some(value) = counter_hm.get_mut("B"){
                *value -= 2;
            }
        } else if str_ref == "BC" { 
            if let Some(value) = counter_hm.get_mut("B"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("C"){
                *value -= 1;
            }
        } else if str_ref == "BD" { 
            if let Some(value) = counter_hm.get_mut("B"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("D"){
                *value -= 1;
            }
        } else if str_ref == "BE" { 
            if let Some(value) = counter_hm.get_mut("B"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("E"){
                *value -= 1;
            }
        } else if str_ref == "CC" {
            if let Some(value) = counter_hm.get_mut("C"){
                *value -= 2;
            }
        } else if str_ref == "CD" {
            if let Some(value) = counter_hm.get_mut("C"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("D"){
                *value -= 1;
            }
        } else if str_ref == "CE" {
            if let Some(value) = counter_hm.get_mut("C"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("E"){
                *value -= 1;
            }
        } else if str_ref == "DD" {
            if let Some(value) = counter_hm.get_mut("D"){
                *value -= 2;
            }
        } else if str_ref == "DE" {
            if let Some(value) = counter_hm.get_mut("D"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("E"){
                *value -= 1;
            }
        } else if str_ref == "EE" {
            if let Some(value) = counter_hm.get_mut("E"){
                *value -= 2;
            }
        }
        false
    }
}
