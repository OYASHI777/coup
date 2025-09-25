// A list of constants
pub const TOKENS: &str = "AAABBBCCCDDDEEE";

// MAX CAPACITY is the maximum possible states that cards can be distributed given the player knows 2 cards!
// max(78060, 107940) => see test1 in unittests
pub const MAX_HAND_STATES: usize = 107940;
pub const MAX_PERM_STATES: usize = 1469700; // See test
pub const MAX_GAME_LENGTH: usize = 120;
pub const BAG_SIZES: [usize; 7] = [2, 2, 2, 2, 2, 2, 3];
