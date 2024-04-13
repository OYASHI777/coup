use crate::prob_manager::permutation_generator::gen_bag_combinations;
use crate::prob_manager::permutation_generator::gen_starting_hand;
use crate::prob_manager::permutation_generator::gen_table_combinations;


pub fn test1() {
    println!("RUNNING TEST: Hand Generation");
    let tokens = "AAABBBCCCDDDEEE";
    let bag_sizes: Vec<usize> = vec![2, 2, 2, 2, 2, 2, 3];
    for player_no in 0..6 {
        println!("Player: {player_no}");
        for ihand in gen_bag_combinations("AABBCCDDEE", &2){
            let output: Vec<String> = gen_starting_hand(tokens, &bag_sizes, &player_no, &ihand);
            let total: usize = output.len();
            println!("{ihand}: {total}");
        }
    }
}

pub fn test2() {
    println!("RUNNING TEST: Hand Generation");
    let  tokens = "AAABBBCCCDDDEEE";
    let bag_sizes: Vec<usize> = vec![2, 2, 2, 2, 2, 2, 3];
    let output: Vec<String> = gen_table_combinations(tokens, &bag_sizes);
    // println!("{output:?}");
    let total: usize = output.len();
    println!("\tTotal Produced: {total}");
    if total == 1469700 {
        println!("===== PASSED 1 / 1 =====");
    } else {
        println!("===== FAILED 0 / 1 =====");
    }
}

pub fn test3() {
    println!("RUNNING TEST: Bag Generation");
    let tokens = "ABC";
    let output4: Vec<String> = gen_bag_combinations(tokens, &2);
    println!("{output4:?}");
}

pub fn test5() {
    println!("RUNNING TEST: string_utils::remove_chars");
    let tokens: &str = "ABCDE";
    let output: String = crate::string_utils::remove_chars(tokens, &"AB".to_string());
    println!("{output}")
}

pub fn test_prob_state1() {
    use crate::prob_manager::prob_state::ProbState;
    let player_no: usize = 0;
    let hand: &str = "AB";
    let mut test_object = ProbState::new_hand(&player_no, &hand.to_string());
    dbg!(test_object.index_combi.len());
    dbg!(test_object.combi_index.len());
    dbg!(test_object.prob_store.len());
    // println!("{test_object:?}");
}

pub fn test_string_utils1(){
    use crate::string_utils::contains_all_chars;
    let mut counter: usize = 0;
    let mut total: usize = 0;

    let mut s1: String = String::from("ABCDE");
    let mut s2: String = String::from("DE");
    if contains_all_chars(&s1, &s2) == true {
        counter += 1;
    }
    total += 1;
    s1 = String::from("ABCDE");
    s2 = String::from("ED");
    if contains_all_chars(&s1, &s2) == true {
        counter += 1;
    }
    total += 1;
    
    s1 = String::from("ABCDE");
    s2 = String::from("JZ");
    if contains_all_chars(&s1, &s2) == false {
        counter += 1;
    }
    total += 1;
    
    s1 = String::from("ABBCDE");
    s2 = String::from("BJ");
    if contains_all_chars(&s1, &s2) == false {
        counter += 1;
    }
    total += 1;
    
    s1 = String::from("ABBCDE");
    s2 = String::from("EE");
    if contains_all_chars(&s1, &s2) == false {
        counter += 1;
    }
    total += 1;
    if counter == total {
        println!{"===== PASSED {counter} / {total}====="};
    } else {
        println!{"===== FAILED {counter} / {total}====="};

    }
}

// pub fn test_trans_map(){
//     use std::collections::{HashMap, HashSet};
//     use crate::prob_manager::coup_transition_maps::preload_transition_amb;
//     println!("start amb_map");
//     let amb_map: HashMap<String, [[HashSet<String>;3];6]> = preload_transition_amb();
//     println!("end amb_map");
//     let mut counter: usize = 0;
//     for (key, state_map) in amb_map.iter(){
//         if counter < 1400000 {
//             continue
//         } else if counter > 1400010 {
//             break
//         }
//         println!("STATE: {key}");
//         for (player_no, cases) in state_map.iter().enumerate(){
//             print!("P{player_no}: ");
//             for (case, hashset) in cases.iter().enumerate(){
//                 print!("C{case}: ");
//                 for state in hashset.iter(){
//                     print!("{state} | ");
//                 }
//             }
//         }
//         counter += 1;
//     }
// }