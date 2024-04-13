use crate::models::{CounterResult, Card, Action, DefaultBehaviour, ActionPacket, ActionName};
use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;
use crate::models::ActionFactory::ActionFactory;
use crate::player::{Player, PlayerInterface};
use crate::gamestate::GameState;
// use crate::replay::{ActionData, Replay};
const PILE_PLAYER_ID: usize = 6;
use log;

// This is the struct that runs the game
pub struct Game {
    players: Vec<Player>,
    court_deck: String,
    court_status: String,
    court_cards: Vec<Vec<Card>>,
    game_state: GameState,
    turn_no: usize,
    current_turn: usize,
    game_won: bool,
}

impl Game {
    pub fn new() -> Self{
        let players = (0..6).map(Player::new).collect();
        let mut rng = rand::thread_rng();
        Game {
            players,
            court_deck: "AAABBBCCCDDDEEE".to_string(),
            court_status: "111111111111111".to_string(),
            court_cards: Vec::new(),
            game_state: GameState::default(),
            // id of the current player's turn
            current_turn: rng.gen_range(0..=5),
            // number of turns
            turn_no: 0,
            game_won: false,
        }
    }
    pub fn run_game(&mut self){
        // run this to start the game!
        self.game_state.deck_to_cards(true);
        while !self.game_won{
            log::info!("turn_no: {}", self.turn_no);

            let mut start_move: Box<dyn DefaultBehaviour> = ActionFactory::create_action(&Action::TurnStart, self.current_turn, 17);
            start_move.execute(self);
            self.next_turn();
            log::info!("");
        }
        self.log_report();
    }
    fn winner(&mut self, player_id: usize) {
        // runs when game is won!
        self.game_won = true;
        log::info!("{}", format!("GAME ENDS!! Player {} WINS!!", player_id));
    }
    fn next_turn(&mut self) {
        // if no winner, increment relevant variables for next turn
        let winner_id = self.find_winner();
        match winner_id {
            Some(id) => {
                self.winner(id);
            },
            None => {},
        }
        // finds next person alive to determine current-turn
        self.current_turn = (self.current_turn + 1) % 6;
        while !self.is_alive(self.current_turn){
            self.current_turn = (self.current_turn + 1) % 6;
        }
        // increments to next turn
        self.turn_no += 1;
    }

    fn next_player_alive(&self, player_id: usize) -> usize {
        // finds next player alive
        let mut output: usize = (player_id + 1) % 6;
        while !self.is_alive(output){
            output = (output + 1) % 6;
            if output == player_id {
                panic!("next_player_alive went full cycle");
            }
        }
        output
    }
    pub fn is_alive(&self, player_id: usize) -> bool{
        self.game_state.is_alive(player_id)
    }
    fn find_winner(&self) -> Option<usize> {
        // there is a winner if only one player is alive
        let mut count: u8 = 0;
        let mut output: usize = 0;
        for i in 0..6 {
            if self.is_alive(i){
                count += 1;
                output = i;
            }
        }
        if count == 1 {
            return Some(output);
        } else {
            return None;
        }
    }
    pub fn logcc(&self){
        // this just logs the game state
        self.game_state.logcc();
    }
    pub fn log_report(&self){
        // this is the ending report
        log::info!("");
        log::info!("=== GAME REPORT ===");
        log::info!("{}", format!("Total turns player: {}", self.turn_no));
    }
    fn char_to_card(&self, c: char) -> Card {
        // the mapping to convert char to card
        self.game_state.char_to_card(c)
    }

    pub fn player_has_card(&self, player_id: usize, card: &Card) -> bool {
        // Returns false if player does not exist
        // Returns T/F based on whether the player has the card
        // self.court_playing_cards.get(player_id).map_or(false, |cards| cards.contains(&card))
        self.game_state.player_has_card(player_id, card)
    }
    pub fn player_get_influence(&self, player_id: usize) -> u8 {
        // gets the influence of a player from the gamestate
        self.game_state.get_influence(player_id)
    }
    pub fn player_remove_card(&mut self, player_id: usize, card: &Card) -> CounterResult{
        // Removes the card if player has the card
        // player_id usage checks done on parsing
        if self.game_state.player_discard(player_id, card) {
            return CounterResult::Success
        } else {
            return CounterResult::Failure
        }
    }

    pub fn player_get_coins(&self, player_id: usize) -> u8 {
        // Checks for this will be done in parsing
        self.game_state.get_coins(player_id)
    }
    
    pub fn player_add_coins(&mut self, player_id: usize, amount: u8){
        // Checks for this will be done in parsing player_id should be [0,5]
        self.game_state.add_coins(player_id, amount);
        // Add Msg Passing
    }
    pub fn player_minus_coins(&mut self, player_id: usize, amount: u8){
        // Checks for this will be done in parsing
        self.game_state.sub_coins(player_id, amount);
        // Add Msg Passing
    }
    pub fn get_turn_no(&self) -> usize {
        self.turn_no
    }
    pub fn get_cards(&self) -> Vec<Vec<Card>> {
        self.game_state.get_cards()
    }
    pub fn player_get_hand(&self, player_id: usize) -> Vec<Card> {
        self.game_state.get_player_hand(player_id).clone()
    }
    // pub fn round_robin_poll(&self, exclude_player_id: usize, actions: &[Action]) -> Option<Box<dyn DefaultBehaviour>> {
    //     // asks each player alive, starting from the next player if they wish to act on a challenge, or to block
    //     let mut i: usize = self.next_player_alive(exclude_player_id);
    //     // TOFIX actions is wrong type
    //     log::info!("Offering other players a chance for counterplay!");
    //     while i != exclude_player_id {
    //         //Insert <Get legal policy actions>
    //         //This should process challenge only for now
    //         let mut action_pack_vec = ActionFactory::create_action_packet_vec_blind(actions, i, exclude_player_id, self.get_turn_no());
    //         self.ensure_legal(&mut action_pack_vec);

                       
    //         let action: ActionPacket = self.players[i].choose_actions(action_pack_vec);
            
    //         if action.get_action_name() != ActionName::Pass {
    //             return Some(ActionFactory::create_action_from_pkt(&action));
    //         }
    //         i = self.next_player_alive(i);
    //     }
    //     None
    // }
    pub fn round_robin_poll_counterplay(&mut self, exclude_player_id: usize, actions: &[Action]) -> CounterResult {
        // asks each player alive, starting from the next player if they wish to act on a challenge, or to block
        let mut i: usize = self.next_player_alive(exclude_player_id);
        // TOFIX actions is wrong type
        log::info!("Offering other players a chance for counterplay!");
        while i != exclude_player_id {
            //Insert <Get legal policy actions>
            //This should process challenge only for now
            let mut action_pack_vec = ActionFactory::create_action_packet_vec_blind(actions, i, exclude_player_id, self.get_turn_no());
            self.ensure_legal(&mut action_pack_vec);

            let action: ActionPacket = self.players[i].choose_actions(action_pack_vec);
            let mut box_default_behaviour: Box<dyn DefaultBehaviour> = ActionFactory::create_action_from_pkt(&action);
            
            box_default_behaviour.execute(self);
            if action.get_action_name() != ActionName::Pass{
                return box_default_behaviour.get_result();
            }
            
            i = self.next_player_alive(i);
        }
        return CounterResult::Failure;
    }

    // pub fn poll_simult(&self, exclude_player_id: usize ,actions: &[Action]) -> Box<dyn DefaultBehaviour>{
    //     // For each player, generate legal moves
    // }

    pub fn present_legal_moves(&self, player_id: usize, opposing_player_id: usize, actions: &[Action]) -> Box<dyn DefaultBehaviour>{
        // Tells player to make decision
        // None if pass is given
        // Should send player ActionPackets to choose from
        // Present legal moves to player_id
        // add card_data to input!!!!
        // Make vector of action packets from it
        let mut action_pack_vec = ActionFactory::create_action_packet_vec_blind(actions, player_id, opposing_player_id, self.get_turn_no());

        self.ensure_legal(&mut action_pack_vec);

        self.send_move_pkts(player_id, action_pack_vec)
    }
    pub fn send_move_pkts(&self, player_id: usize, action_pack_vec: Vec<ActionPacket>)-> Box<dyn DefaultBehaviour> {
        // presents legal moves to player, but this is in the form of actionpackets
        
        
        log::info!("Sending pkts {}", format!("{:?}", action_pack_vec));
        if let Some(player) = self.players.get(player_id){
            let chosen_action: ActionPacket = player.choose_actions(action_pack_vec);
            return ActionFactory::create_action_from_pkt(&chosen_action);        
        } else {
            panic!("Player does not exist");
        }
    }
    fn ensure_legal(&self, blind_move_set: &mut Vec<ActionPacket>){
        // makes sure actionpackets are legal moves
        blind_move_set.retain(|pkt| {self.is_legal(pkt)});
    }
    fn is_legal(&self, pkt: &ActionPacket) -> bool{
        // check for if an actionpacket is legal
        // Both Players have to be alive
        if !self.is_alive(pkt.get_player_id()) {
            return false;
        }
        if pkt.get_opposing_player_id() != 17 && !self.is_alive(pkt.get_opposing_player_id()) {
            return false;
        }
        // Can only Coup if coins >= 7
        if pkt.get_action_name() == ActionName::Coup && self.player_get_coins(pkt.get_player_id()) < 7 {
            return false;
        }
        // Can only Assassinate if coins >= 3
        if pkt.get_action_name() == ActionName::Assassinate && self.player_get_coins(pkt.get_player_id()) < 3 {
            return false;
        }
        if pkt.get_action_name() == ActionName::Steal && self.player_get_coins(pkt.get_opposing_player_id()) == 0 {
            return false;
        }
        let turn_start_set: [ActionName;6]= [ActionName::Income, ActionName::ForeignAid, ActionName::Tax, ActionName::Assassinate, ActionName::Exchange, ActionName::Steal];
        if self.player_get_coins(pkt.get_player_id()) > 9 && turn_start_set.iter().any(|a| *a == pkt.get_action_name()){
            return false;
        }
        // Can only discard card that a player has
        if pkt.get_action_name() == ActionName::Discard{
            // If player does not have the card
            if !self.player_has_card(pkt.get_player_id(), &pkt.get_card(0 as usize)){
                return false;
            } 
        }
        // Check receive only if card is in both pile and player hand
        true
    }

    pub fn shuffle_redraw(&mut self, player_id: usize, card: &Card){
        // take card from player and randomly shuffle from Pile
        // adds it to the pile and randomly shuffles and redraws
        let mut rng = thread_rng();
        let num: i32 = rng.gen_range(0..4);

        if self.game_state.player_has_card(player_id, card){
            if num != 0 {
                self.game_state.swap_deck_rnd(player_id, card);
            }
            // else do not swap, 25% chance of receive the same cards!s
            log::info!("{}", format!("Player {}'s cards after shuffling: ", player_id));
            self.logcc();
        } else {
            panic!("Tried to remove card from player who does not have the card!");
        }
    }
    pub fn choose_random_cards(&self, player_id: usize, n: usize) -> Vec<Card>{
        // I have a Vec<Vec<Card>> in self.court_cards
        // I wish to choose n random Cards in self.court_cards[player_id] and return a Vec<Card> containing them

        let player_cards = &self.get_cards()[player_id];

        let mut rng = thread_rng();
        player_cards
        .choose_multiple(&mut rng, n)
        .cloned()
        .collect()

    }
    pub fn swap_pooled(&mut self, player_id: usize, card_data: &Vec<Card>){
        self.game_state.swap_pooled(player_id, card_data);
    }

}
