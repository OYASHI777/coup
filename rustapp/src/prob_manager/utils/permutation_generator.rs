use crate::string_utils::remove_chars;
use crate::string_utils::sort_str;
use crate::string_utils::unique_sorted_string;
use crate::string_utils::filter_string_within;
use crate::prob_manager::constants::{MAX_HAND_STATES, MAX_PERM_STATES};

// This is unnecessary but used in case I ever want to port this code into a non-coup game
const LOCAL_MAX_HAND_CAPACITY: usize = MAX_HAND_STATES;
const LOCAL_MAX_PERM_CAPACITY: usize = MAX_PERM_STATES;

pub fn gen_starting_hand(tokens: &str, bag_sizes: &Vec<usize>, player_no: &usize, hand: &String) -> Vec<String> {
    // Takes the starting hand, and returns all possible ways cards can be distributed given the starting hand of the player

    let total: usize = bag_sizes.iter().sum();
    assert!(tokens.len() == total, "INPUT ISSUE in gen_starting_hand: tokens.len() == bag_sizes.iter().sum()");
    let full_tokens: String = tokens.to_string(); 
    let mut tokens: String = remove_chars(tokens, hand); 
    tokens = sort_str(tokens.as_str());
    assert!(bag_sizes[*player_no] as usize == hand.len(), "INPUT ISSUE in gen_starting_hand: bag_size and hand size should be equal");


    let mut old_storage: Vec<String> = Vec::with_capacity(LOCAL_MAX_HAND_CAPACITY);
    for (index, bag_size) in bag_sizes.iter().enumerate() {

        if index == 0 {
            let combinations: Vec<String> = if index != *player_no {
                gen_bag_combinations(tokens.as_str(), bag_size)
            } else {
                vec![hand.to_string()]
            };

            old_storage = combinations;
        } else {
            let mut temp_storage: Vec<String> = Vec::with_capacity(LOCAL_MAX_HAND_CAPACITY);

            for old_string in old_storage.iter(){
                // Make tokens


                let remaining_tokens: String = if index <= *player_no {
                    remove_chars(tokens.as_str(), old_string)
                } else {
                    remove_chars(full_tokens.as_str(), old_string)
                };

                let combinations: Vec<String> = if index != *player_no {
                    gen_bag_combinations(remaining_tokens.as_str(), bag_size)
                } else {
                    vec![hand.to_string()]
                };
                for combi in combinations.iter(){
                    temp_storage.push(old_string.to_string() + combi);
                }
            }
            old_storage = temp_storage;
            // Address remaining tokens
        }
    }
    return old_storage;
}

pub fn gen_table_combinations(tokens: &str, bag_sizes: &Vec<usize>) -> Vec<String>{
    // Generates all possible ways cards can be distributed
    let total: usize = bag_sizes.iter().sum();
    assert!(tokens.len() == total as usize, "INPUT ISSUE in gen_table_combinations: tokens.len() == bag_sizes.iter().sum()");
    let tokens: String = sort_str(tokens);


    let mut old_storage: Vec<String> = Vec::with_capacity(LOCAL_MAX_PERM_CAPACITY);
    for (index, bag_size) in bag_sizes.iter().enumerate() {

        if index == 0 {
            let combinations: Vec<String> = gen_bag_combinations(tokens.as_str(), bag_size);
            old_storage = combinations;
        } else {
            let mut temp_storage: Vec<String> = Vec::with_capacity(LOCAL_MAX_PERM_CAPACITY);
            for old_string in old_storage.iter(){
                // Make tokens
                let remaining_tokens: String = remove_chars(tokens.as_str(), old_string);

                let combinations: Vec<String> = gen_bag_combinations(remaining_tokens.as_str(), bag_size);
                for combi in combinations.iter(){

                    temp_storage.push(old_string.to_string() + combi);
                }
            }
            old_storage = temp_storage;
            // Address remaining tokens
        }
    }
    return old_storage;
}

pub fn gen_bag_combinations(tokens: &str, bag_size: &usize) -> Vec::<String>{
    // Takes a reference to a string, and a size of the bag, and returns every possible combination of the string that could be in the bag
    // A bag implies order does not matter
    // String is arranged in ascending order
    // bag_size has to be <= tokens length
    let token_len = tokens.len();
    assert!(*bag_size as usize <= token_len, "INPUT ISSUE in gen_bag_combinations: bag_size ({bag_size}) must be <= token_len ({token_len})");
    assert!(*bag_size > 0, "INPUT ISSUE in gen_bag_combinations: bag_size must be > 0");
    if *bag_size as usize == token_len {
        return vec![tokens.to_string()];
    }

    let unique_sorted_tokens: Vec<char> = unique_sorted_string(tokens);
    let sorted_tokens: String = sort_str(tokens);
    let tokens: &str = sorted_tokens.as_str();

    // Last character that the combination can start with (inclusive)
    let mut largest_char_min: char;
    let mut smallest_char_max: char;

    let mut old_storage: Vec<String> = Vec::new();

    let mut current_index: usize = 0;
    while current_index < *bag_size as usize {
        // The appropriate ending char => largest char that the current index can take
        largest_char_min = match tokens.chars().nth(token_len - (*bag_size as usize - current_index)) {
            Some(ch) => ch,
            None => {panic!("Index out of bounds");},
        };
        
        // The appropriate starting char => smallest char that the current index can take
        smallest_char_max = match tokens.chars().nth(current_index) {
            Some(ch) => ch,
            None => {panic!("Index out of bounds");},
        };

        // List of possible cards to choose from will be the unique values captured within smallest_char_max to largest_char_min (inclusive)
        
        // Get all possible unique values for this current index
        let unique_possible_tokens = filter_string_within(&unique_sorted_tokens, &smallest_char_max, &largest_char_min);
        
        if old_storage.len() == 0{
            let new_storage: Vec<String> = unique_possible_tokens.iter().map(|&c| c.to_string()).collect();
            old_storage = new_storage;
        } else {
            let mut temp_storage: Vec<String> = Vec::new();
            for old_string in old_storage.iter() {
                let remaining_tokens: String = remove_chars(tokens, old_string);
                // new possible values are all unqiue values >= old_string
                let new_storage: Vec<String> = unique_sorted_string(remaining_tokens.as_str()).iter().map(|&c| c.to_string()).collect();
                for string_new in new_storage.iter() {
                    if old_string[(old_string.len()-1)..] <= **string_new {
                        temp_storage.push(old_string.to_string() + string_new);
                    }
                }
            }
            old_storage = temp_storage;
        }
        current_index += 1;
    }

    return old_storage;

}
