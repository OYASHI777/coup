use std::hash::Hasher;
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct BRKey {
    player_id: usize,
    infostate: String,
}
pub const MAX_NUM_BRKEY: usize = 6 * 15; 
pub const INFOSTATES: [&str; 15] = ["AA", "AB", "AC", "AD", "AE", "BB", "BC", "BD", "BE", "CC", "CD", "CE", "DD", "DE", "EE"];
impl BRKey {
    pub fn new(player_id: usize, infostate: &str) -> Self {
        debug_assert!(player_id > 0, "Invalid player_id");
        debug_assert!(player_id < 6, "Invalid player_id");
        BRKey {
            player_id,
            infostate: infostate.to_string(),
        }
    }
    pub fn set_infostate(&mut self, new_infostate: &str) {
        self.infostate = new_infostate.to_string()
    }
    pub fn set_player_id(&mut self, new_player_id: usize) {
        self.player_id = new_player_id;
    }
    pub fn player_id(&self) -> usize {
        self.player_id
    }
    pub fn infostate(&self) -> String {
        self.infostate.clone()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn player_id(&self) -> usize {
        self.player_id
    }
}

impl Hash for MSKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.player_id.hash(state);
        self.path.hash(state);
    }
}