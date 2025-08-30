use std::collections::HashMap;
use lazy_static::lazy_static; //This allows us to declare "constant" dynamic variables allowing them to run just once in runtime

// A list of constants
pub const TOKENS: &str = "AAABBBCCCDDDEEE";

// MAX CAPACITY is the maximum possible states that cards can be distributed given the player knows 2 cards!
// max(78060, 107940) => see test1 in unittests
pub const MAX_HAND_STATES: usize = 107940;
pub const MAX_PERM_STATES: usize = 1469700; // See test 
pub const MAX_GAME_LENGTH: usize = 120;
pub static ARR_POSSIBLE_HANDS: [&'static str; 15] = ["AA", "AB", "AC", "AD", "AE", "BB", "BC", "BD", "BE", "CC", "CD", "CE", "DD", "DE", "EE"]; 
lazy_static! {
    pub static ref BAG_SIZES: Vec<usize> = vec![2, 2, 2, 2, 2, 2, 3];
    //consider making the indices based on BAG_SIZES

    // pub static ref VEC_POSSIBLE_HANDS: Vec<String> = super::permutation_generator::gen_bag_combinations(TOKENS, &PLAYER_HAND_SIZE); 

    // These variables represent a mapping for which index represents which hand one might have e.g. "AB" or "AA"
    // To be used to index into the conditional policy
    // pub static ref MAP_CONDHAND_INDEX: HashMap<String, usize> = VEC_POSSIBLE_HANDS.iter().enumerate().map(|(index, key)| (key.clone(), index)).collect();
    // pub static ref MAP_INDEX_CONDHAND: HashMap<usize, String> = VEC_POSSIBLE_HANDS.iter().enumerate().map(|(index, key)| (index, key.clone())).collect();

    pub static ref MAP_CONDHAND_INDEX: HashMap<&'static str, usize> = {
        let mut m = HashMap::new();
        for (index, &hand) in ARR_POSSIBLE_HANDS.iter().enumerate(){
            m.insert(hand, index);
        }
        m
    };
    pub static ref MAP_INDEX_CONDHAND: HashMap<usize, &'static str> = {
        let mut m = HashMap::new();
        for (index, &hand) in ARR_POSSIBLE_HANDS.iter().enumerate(){
            m.insert(index, hand);
        }
        m
    };
}


