use crate::gamestate::GameState;
use crate::models::ActionPacket;

// turn?
pub struct Replay {
    action_replay: Vec<ActionPacket>,
    starting_deck: String,
    bool_turn_move: Vec<bool>,
    turn_player: usize,
}

// I need to know if its a turn_start move or a response move
// Last turn_move
// turn_player

impl Replay {
    pub fn new(court_deck: &str) -> Self {
        Replay {
            action_replay: Vec::new(),
            starting_deck: court_deck.to_string(),
        }
    }
    pub fn add_action(&mut self, action_packet: ActionPacket) {
        // Stores state after Action is performed
        let action_data: ActionPacket = ActionData::new(player_id, action, new_state.clone());
    }
    pub fn legal_move_sets(&self) {
        // Things needed to know
        // Players who can move or respond next
        // For each player
        // [Assassin P2, P2 Challenge Assassin, P0 Revealed Assassin] 
    }
}
