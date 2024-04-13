use crate::prob_manager::permutation_generator::{gen_bag_combinations, gen_table_combinations};
use crate::string_utils::{remove_chars, sort_str};

// Preloading Ambassador maps
use super::coup_const::{TOKENS, BAG_SIZES};
use lazy_static::lazy_static;
use std::collections::{HashSet, HashMap};


const DIM1: usize = 6; // Players
const DIM2: usize = 3; // Ways to draw 2 cards from 4
const DIM3: usize = 6; // Max ways to choose 2 cards from 4 (will be less if there are repeated cards)

// Scrapping this for now
pub fn preload_transition_amb() -> HashMap<String, [[HashSet<String>;DIM2];DIM1]> {
    let vec_states: Vec<String> = gen_table_combinations(TOKENS, &BAG_SIZES);

    let mut output: HashMap<String, [[HashSet<String>;DIM2];DIM1]> = HashMap::new();
    let mut current_hand: String;
    let mut player_index_lower: usize;
    let mut player_index_upper: usize;
    let amb_index: usize = 6 * 2;
    let mut choice_pool: String;
    let mut potential_choices: Vec<String>;
    let bag_size: usize = 2;
    let mut remainder: String;
    // let mut new_state: String;
    let mut counter: usize = 0;
    for current_state in vec_states.iter(){
        let mut amb_map: [[HashSet<String>; DIM2]; DIM1] = Default::default();
        for i in 0..DIM1{
            for j in 0..DIM2{
                amb_map[i][j] = HashSet::with_capacity(DIM3);
            }
        }
        
        if counter % 10000 == 0 {
            println!("{counter}");
        }
        counter += 1;
        for player_no in (0 as usize)..(6 as usize) {
            player_index_lower = player_no * 2;
            player_index_upper = player_no * 2 + 2;
            // Case 1
            // println!("current_state {current_state} player {player_no}");

            current_hand = current_state[player_index_lower..player_index_upper].to_string();
            // println!("current_hand {current_hand} case 0");
            choice_pool = current_hand.clone() + &current_state[amb_index..(amb_index + 2)];
            // println!("choice_pool {choice_pool}");
            
            potential_choices = gen_bag_combinations(&choice_pool, &bag_size);
            // dbg!(potential_choices.clone());
            for choice in &potential_choices {
                remainder = remove_chars(&choice_pool, choice).to_string() + &current_state[(amb_index + 2)..(amb_index + 3)];
                remainder = sort_str(&remainder);
  
                let new_state: String = current_state[0..player_index_lower].to_string() + &choice + &current_state[(player_index_lower + 2)..amb_index] + &remainder;
                // println!("remainder {remainder} new_state {new_state}");
                amb_map[player_no][0].insert(new_state);
            }
            // Case 2
            // println!("current_hand {current_hand} case 1");
            choice_pool = current_hand.clone() + &current_state[(amb_index + 1)..((amb_index) + 3)];
            // println!("choice_pool {choice_pool}");
            potential_choices = gen_bag_combinations(&choice_pool, &bag_size);
            // dbg!(potential_choices.clone());
            for choice in &potential_choices {
                remainder = remove_chars(&choice_pool, choice).to_string() + &current_state[(amb_index)..(amb_index + 1)];
                remainder = sort_str(&remainder);
                
                let new_state: String = current_state[0..player_index_lower].to_string() + &choice + &current_state[(player_index_lower + 2)..amb_index] + &remainder;
                // println!("remainder {remainder} new_state {new_state}");
                amb_map[player_no][1].insert(new_state);
            }
            
            // Case 3
            
            // println!("current_hand {current_hand} case 0");
            choice_pool = current_hand.clone() + &current_state[amb_index..(amb_index + 1)] + &current_state[(amb_index + 2)..(amb_index + 3)];
            // println!("choice_pool {choice_pool}");
            potential_choices = gen_bag_combinations(&choice_pool, &bag_size);
            // dbg!(potential_choices.clone());
            for choice in &potential_choices {
                remainder = remove_chars(&choice_pool, choice).to_string() + &current_state[(amb_index + 1)..(amb_index + 2)];
                remainder = sort_str(&remainder);
                
                let new_state: String = current_state[0..player_index_lower].to_string() + &choice + &current_state[(player_index_lower + 2)..amb_index] + &remainder;
                // println!("remainder {remainder} new_state {new_state}");
                amb_map[player_no][2].insert(new_state);
            }
        }
        // println!("current_state {current_state}");
        // dbg!(amb_map.clone());
        output.insert(current_state.to_string(), amb_map.clone());
    }
    output
}

// USAGE
// state: &str -> ( Case 1 | Case 2 | Case 3 ) X ( Players )
//          Case 1 [&str outcomes (can repeat)]
// &str : arr[arr[HashSet]:(3 cases)]:(6 players)