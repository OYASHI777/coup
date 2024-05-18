#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct BRKey {
    player_id: usize,
    state_index: usize,
}
pub const MAX_NUM_BRKEY: usize = 6 * 15; 
impl BRKey {
    pub fn new(player_id: usize, state_index: usize) -> Self {
        debug_assert!(player_id > 0, "Invalid player_id");
        debug_assert!(player_id < 6, "Invalid player_id");
        BRKey {
            player_id,
            state_index,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MSKey {
    player_id: usize,
    path: String,
}

impl MSKey {
    pub fn new(player_id: usize, path: &str) -> Self {
        debug_assert!(player_id > 0, "Invalid player_id");
        debug_assert!(player_id < 6, "Invalid player_id");
        MSKey {
            player_id,
            path: path.to_string(),
        }
    }
}