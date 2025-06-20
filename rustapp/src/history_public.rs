use serde::{Serialize, Deserialize};
use std::convert::TryFrom;
#[derive(Debug, PartialEq, Eq, Copy, Clone, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Card {
    Ambassador = 0, // 0
    Assassin = 1, // 1
    Captain = 2, // 2
    Duke = 3, // 3
    Contessa = 4, // 4
}
impl Card {
    pub fn card_to_string(&self) -> String {
        match self {
            Card::Ambassador => "A".to_string(),
            Card::Assassin => "B".to_string(),
            Card::Captain => "C".to_string(),
            Card::Duke => "D".to_string(),
            Card::Contessa => "E".to_string(),
        }
    }
    pub fn card_to_str(&self) -> &'static str {
        match self {
            Card::Ambassador => "A",
            Card::Assassin => "B",
            Card::Captain => "C",
            Card::Duke => "D",
            Card::Contessa => "E",
        }
    }
    pub fn card_to_char(&self) -> char {
        // Ambassador => A
        // Assassin => B
        // Captain => C
        // Duke => D
        // Contessa => E
        (b'A' + *self as u8) as char
    }
    pub fn char_to_card(card_char: char) -> Card {
        // Ambassador <= A
        // Assassin <= B
        // Captain <= C
        // Duke <= D
        // Contessa <= E
        Card::try_from(card_char as u8 - b'A').unwrap()
    }
    pub fn str_to_card(card_str: &str) -> Card {
        match card_str {
            "A" => Card::Ambassador,
            "B" => Card::Assassin,
            "C" => Card::Captain,
            "D" => Card::Duke,
            "E" => Card::Contessa,
            _ => panic!("Invalid input provided!"),
        }
    }
}
impl From<Card> for u8 {
    fn from(card: Card) -> u8 {
        card as u8
    }
}
impl TryFrom<u8> for Card {
    type Error = ();
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Card::Ambassador),
            1 => Ok(Card::Assassin),
            2 => Ok(Card::Captain),
            3 => Ok(Card::Duke),
            4 => Ok(Card::Contessa),
            _ => Err(())
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AOName {
    EmptyAO,
    ChallengeAccept,
    ChallengeDeny,
    Income,
    ForeignAid,
    Tax,
    Steal,
    Assassinate,
    Coup,
    CollectiveChallenge,
    CollectiveBlock,
    BlockSteal,
    BlockAssassinate,
    Discard,
    RevealRedraw,
    Exchange,
    ExchangeDraw,
    ExchangeChoice,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AOResult {
    Success,
    Failure,
    Pass,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ActionObservation {
    EmptyAO,
    // Challenge Status before collation
    ChallengeAccept {
        player_id: usize,
        opposing_player_id: usize,
    },
    ChallengeDeny,
    Income {
        player_id: usize,
    },
    ForeignAid{
        player_id: usize,
    },
    Tax {
        player_id: usize,
    },
    Steal {
        player_id: usize,
        opposing_player_id: usize,
        amount: u8,
    },
    Assassinate{
        player_id: usize,
        opposing_player_id: usize,
    },
    Coup {
        player_id: usize,
        opposing_player_id: usize,
    },
    CollectiveChallenge {
        participants: [bool; 6],
        opposing_player_id: usize,
        final_actioner: usize, // == opposing_player_id then no one blocks
    },
    CollectiveBlock{
        participants: [bool; 6],
        opposing_player_id: usize,
        final_actioner: usize, // == opposing player (the player being blocked) if no one blocks
        // Does not need an Eval
    },
    BlockSteal {
        player_id: usize,
        opposing_player_id: usize, // if == player_id, then the player passes on the block
        card: Card,
        // The card used to block with
    },
    BlockAssassinate {
        player_id: usize,
        opposing_player_id: usize, // if == player_id, then the player passes on the block
    },
    Discard {
        player_id: usize,
        card: [Card; 2],
        no_cards: usize,
    },
    RevealRedraw {
        player_id: usize,
        reveal: Card, //Card Revealed
        redraw: Card,
    },
    Exchange {
        player_id: usize,
    },
    ExchangeDraw {
        player_id: usize,
        card: [Card; 2],
    },
    ExchangeChoice {
        // cards represents their choice of hand
        // For no_cards == 1, both cards in cards should be the same
        player_id: usize,
        relinquish: [Card; 2], // Assume either all is known or none are known
    },
    // TODO: Add ExchangeChoice Private, and generate possible moves based on history and exchangedraw choice
}

impl Default for ActionObservation{
    fn default() -> Self{
        ActionObservation::EmptyAO
    }
}
// Need split result
// Collective tells us who challenges or blocks
// Evalresult should tell us status of whether it succeeded or failed
impl ActionObservation {
    pub fn name(&self) -> AOName {
        use ActionObservation::*;
        match self {
            EmptyAO => AOName::EmptyAO,
            ChallengeAccept { .. } => AOName::ChallengeAccept,
            ChallengeDeny => AOName::ChallengeDeny,
            Income { .. } => AOName::Income,
            ForeignAid { .. } => AOName::ForeignAid,
            Tax { .. } => AOName::Tax,
            Steal { .. } => AOName::Steal,
            Assassinate { .. } => AOName::Assassinate,
            Coup { .. } => AOName::Coup,
            CollectiveChallenge { .. } => AOName::CollectiveChallenge,
            CollectiveBlock { .. } => AOName::CollectiveBlock,
            BlockSteal { .. } => AOName::BlockSteal,
            BlockAssassinate { .. } => AOName::BlockAssassinate,
            Discard { .. } => AOName::Discard,
            RevealRedraw { .. } => AOName::RevealRedraw,
            Exchange { .. } => AOName::Exchange,
            ExchangeDraw { .. } => AOName::ExchangeDraw,
            ExchangeChoice { .. } => AOName::ExchangeChoice,
        }
    }
    pub fn player_id(&self) -> usize {
        match self {
            ActionObservation::Income { player_id }
            | ActionObservation::ForeignAid { player_id }
            | ActionObservation::Tax { player_id }
            | ActionObservation::Steal { player_id, .. }
            | ActionObservation::Assassinate { player_id, .. }
            | ActionObservation::Coup { player_id, .. }
            | ActionObservation::CollectiveChallenge { final_actioner: player_id, .. }
            | ActionObservation::CollectiveBlock { final_actioner: player_id, .. }
            | ActionObservation::BlockSteal { player_id, .. }
            | ActionObservation::BlockAssassinate { player_id, .. }
            | ActionObservation::Discard { player_id, .. }
            | ActionObservation::RevealRedraw { player_id, .. }
            | ActionObservation::Exchange { player_id, .. }
            | ActionObservation::ExchangeDraw { player_id, .. }
            | ActionObservation::ExchangeChoice { player_id, .. } => *player_id,
            // No player_id available in these variants, so we panic
            ActionObservation::ChallengeAccept { .. } => {
                panic!("ChallengeAccept  does not contain a player_id");
            },
            ActionObservation::ChallengeDeny => {
                panic!("ChallengeDeny  does not contain a player_id");
            },
            ActionObservation::EmptyAO => {
                panic!("EmptyAO does not contain a player_id");
            },
        }
    }
    pub fn opposing_player_id(&self) -> usize {
        match self {
            ActionObservation::Steal { opposing_player_id, .. }
            | ActionObservation::Assassinate { opposing_player_id, .. }
            | ActionObservation::Coup { opposing_player_id, .. }
            | ActionObservation::CollectiveChallenge { opposing_player_id, .. }
            | ActionObservation::CollectiveBlock { opposing_player_id, .. }
            | ActionObservation::BlockSteal { opposing_player_id, .. }
            | ActionObservation::BlockAssassinate { opposing_player_id, .. } => *opposing_player_id,
            // Variants without an opposing_player_id field
            _ => panic!("This ActionObservation variant does not contain an opposing_player_id"),
        }
    }
    pub fn card(&self) -> Card {
        match self {
            ActionObservation::BlockSteal { card, .. } => *card, 
            ActionObservation::RevealRedraw { reveal: card, .. } => *card, 
            // Include other cases if there are more variants holding a result field
            _ => panic!("This ActionObservation variant does not contain a result"),
        }
    }
    pub fn cards(&self) -> &[Card; 2] {
        match self {
            ActionObservation::Discard { card, .. } => card, 
            ActionObservation::ExchangeDraw { card, .. } => card, 
            ActionObservation::ExchangeChoice { relinquish, .. } => relinquish, 
            // Include other cases if there are more variants holding a result field
            _ => panic!("This ActionObservation variant does not contain a result"),
        }
    }
    pub fn no_cards(&self) -> usize {
        match self {
            ActionObservation::Discard { no_cards, .. } => *no_cards, 
            // ActionObservation::ExchangeDraw { no_cards, .. } => *no_cards, 
            // ActionObservation::ExchangeChoice { no_cards, .. } => *no_cards, 
            _ => panic!("This ActionObservation variant does not contain an no_discard"),
        }
    }
    pub fn amount(&self) -> u8 {
        match self {
            ActionObservation::Steal { amount, .. } => *amount,
            _ => panic!("This ActionObservation variant does not contain amount!"),
        }
    }
    pub fn final_actioner(&self) -> usize {
        match self {
            ActionObservation::CollectiveBlock { final_actioner , .. } => *final_actioner,
            ActionObservation::CollectiveChallenge { final_actioner , .. } => *final_actioner,
            _ => panic!("This ActionObservation variant does not contain amount!"),
        }
    }

}
#[derive(Copy, Clone)]
struct Gamestate {
    influence: [u8; 6],
    coins: [u8; 6],
}
impl Gamestate {
    pub fn new() -> Self {
        Gamestate{
            influence: [2, 2, 2, 2, 2, 2],
            coins: [2, 2, 2, 2, 2, 2],
        }
    }
    pub fn empty() -> Self {
        Gamestate{
            influence: [0, 0, 0, 0, 0, 0],
            coins: [0, 0, 0, 0, 0, 0],
        }
    }
    pub fn influence(&self) -> &[u8; 6] {
        &self.influence
    }
    pub fn coins(&self) -> &[u8; 6] {
        &self.coins
    }
    pub fn add_influence(&mut self, player_id: usize, amount: u8) {
        self.influence[player_id] += amount;
    }
    pub fn add_coins(&mut self, player_id: usize, amount: u8) {
        self.coins[player_id] += amount;
    }
    pub fn set_influence(&mut self, player_id: usize, amount: u8) {
        self.influence[player_id] = amount;
    }
    pub fn set_coins(&mut self, player_id: usize, amount: u8) {
        self.coins[player_id] = amount;
    }
    pub fn subtract_influence(&mut self, player_id: usize, amount: u8) {
        self.influence[player_id] -= amount;
    }
    pub fn subtract_coins(&mut self, player_id: usize, amount: u8) {
        self.coins[player_id] -= amount;
    }
    pub fn modify(&mut self, new_gamestate: Gamestate){
        for i in 0..6{
            self.influence[i] = new_gamestate.influence[i];
            self.coins[i] = new_gamestate.coins[i];
        }
    }
    pub fn reset(&mut self){
        for i in 0..6{
            self.influence[i] = 0;
            self.coins[i] = 0;
        }
    }
    pub fn reset_start(&mut self){
        for i in 0..6{
            self.influence[i] = 2;
            self.coins[i] = 2;
        }
    }
}
const MAX_HISTORY_LEN: usize = 200;
pub struct History {
    store: [ActionObservation; MAX_HISTORY_LEN],
    store_len: usize,
    dist_from_turn: [usize ; MAX_HISTORY_LEN],
    // influence: [u8; 6],
    // coins: [u8; 6],
    gamestate: [Gamestate; MAX_HISTORY_LEN],
    current_player_turn: usize, 
    // public_card_count: HashMap<Card, u8>, // TODO: Change this to array
    public_card_count: [u8; 5],
}

impl History {
    pub fn new(starting_player: usize) -> Self {
        let mut temp: [Gamestate; MAX_HISTORY_LEN] = [Gamestate::empty(); MAX_HISTORY_LEN];
        temp[0] = Gamestate::new();
        // let mut public_card_count: HashMap<Card, u8> = HashMap::new();
        // public_card_count.insert(Card::Ambassador, 0);
        // public_card_count.insert(Card::Assassin, 0);
        // public_card_count.insert(Card::Captain, 0);
        // public_card_count.insert(Card::Duke, 0);
        // public_card_count.insert(Card::Contessa, 0);
        let public_card_count = [0; 5];
        History{
            store: [ActionObservation::EmptyAO; MAX_HISTORY_LEN],
            store_len: 0,
            dist_from_turn: [1; MAX_HISTORY_LEN],
            gamestate: temp,
            // influence: [2, 2, 2, 2, 2, 2],
            // coins: [2, 2, 2, 2, 2, 2],
            current_player_turn: starting_player,
            public_card_count,
        }
    }
    pub fn reset(&mut self) {
        while self.store_len() > 0 {
            self.remove_ao();
        }
    }
    pub fn push(&mut self, ao: ActionObservation, bool_new_turn: bool){
        // Increase store_len
        // Creates new gamestate 
        self.store_len += 1;
        self.store[self.store_len - 1] = ao;
        if bool_new_turn {
            self.dist_from_turn[self.store_len - 1] = 1;
            // self.current_player_turn = self.next_player(self.current_player_turn);
            self.current_player_turn = ao.player_id();
            debug_assert!(self.current_player_turn == ao.player_id(), "Player turn not the same as ActionObservation");
        } else {
            self.dist_from_turn[self.store_len - 1] = self.dist_from_turn[self.store_len - 2] + 1;
        }
        // Making new starting gamestate same as the old ending gamestate for adjustments to take place
        if self.store_len > 1 {
            self.gamestate[self.store_len - 1].modify(self.gamestate[self.store_len - 2]);
        }
        // in the 1 case it is the first move of the game and should be initialise already and not required to copy
    }
    pub fn log_state(&self){
        log::info!("Current move: {}", self.store_len());
        log::info!("{}", format!("Current_player_turn: {:?}", self.current_player_turn));
        log::info!("{}", format!("Influence: {:?}", self.latest_influence()));
        log::info!("{}", format!("Coins: {:?}", self.latest_coins()));
        // log::info!("{}", format!("Store_len: {:?}", self.store_len));
        log::info!("{}", format!("Dead Cards: {:?}", self.public_card_count));
    }
    pub fn print_history(&self) {
        if self.store_len == 0 {
            println!("No History yet");
        } else {
            println!("History: {:?}", self.store[self.store_len - 1]);
        }
    }
    pub fn print_replay_history_braindead(&self) {
        log::warn!("vec!{:?};", self.get_history(self.store_len));
    }
    pub fn get_replay_history_braindead(&self) -> String {
        format!("vec!{:?};", self.get_history(self.store_len)).to_string()
    }
    pub fn log_history(&self){
        log::info!("{}", format!("Current_player_turn: {:?}", self.get_history(self.store_len)));
        println!("{}", format!("Current_player_turn: {:?}", self.get_history(self.store_len)));
    }
    pub fn pop(&mut self){
        if self.store_len == 0 {
            panic!("Why pop from empty array?");
        }
        if self.dist_from_turn[self.store_len - 1] == 0 {
            // self.current_player_turn = self.prev_player(self.current_player_turn);
            // Get the player of the originating move from the move before the latest move that is about to be removed
            self.current_player_turn = self.store[self.store_len - 1 - self.dist_from_turn[self.store_len - 2]].player_id();
        }
        self.store[self.store_len - 1] = ActionObservation::EmptyAO;
        self.gamestate[self.store_len - 1].reset();
        self.dist_from_turn[self.store_len - 1] = 0;
        self.store_len -= 1;
        if self.store_len == 0 {
            // Make default starting at 1
            self.gamestate[0].reset_start();
        }
    }
    pub fn add_coins(&mut self, player_id: usize, amount: u8){
        self.gamestate[self.store_len - 1].add_coins(player_id, amount);
    }
    pub fn subtract_coins(&mut self, player_id: usize, amount: u8){
        self.gamestate[self.store_len - 1].subtract_coins(player_id, amount);
    }
    pub fn add_influence(&mut self, player_id: usize, amount: u8){
        self.gamestate[self.store_len - 1].add_influence(player_id, amount);
    }
    pub fn subtract_influence(&mut self, player_id: usize, amount: u8){
        self.gamestate[self.store_len - 1].subtract_influence(player_id, amount);
        if self.latest_influence()[player_id] == 0 {
            self.gamestate[self.store_len - 1].set_coins(player_id, 0);
        }
    }
    pub fn add_public_card_count(&mut self, card: &Card) {
        // *self.public_card_count.entry(*card).or_insert(0) += 1;
        self.public_card_count[*card as usize] += 1;
    }
    pub fn subtract_public_card_count(&mut self, card: &Card) {
        debug_assert!(self.public_card_count[*card as usize] > 0, "Card already at zero! Cannot decrease further!");
        // *self.public_card_count.entry(*card).or_insert(0) -= 1;
        self.public_card_count[*card as usize] -= 1;
    }
    pub fn latest_influence(&self) -> &[u8; 6]{
        if self.store_len == 0 {
            // Case when game is just started and in default state
            self.gamestate[0].influence()
        } else {
            self.gamestate[self.store_len - 1].influence()
        }
    }
    pub fn latest_coins(&self) -> &[u8; 6]{
        if self.store_len == 0 {
            // Case when game is just started and in default state
            self.gamestate[0].coins()
        } else {
            self.gamestate[self.store_len - 1].coins()
        }
    }
    pub fn latest_move(&self) -> &ActionObservation {
        if self.store_len == 0 {
            &ActionObservation::EmptyAO
        } else {
            &self.store[self.store_len - 1]
        }
    }
    pub fn game_won(&self) -> bool {
        let mut count: usize = 0;
        for i in 0..6{
            if self.latest_influence()[i] == 0{
                count += 1;
            }
        }
        if count == 5 {
            true
        } else {
            false
        }
    }
    pub fn get_public_card_count(&self, card: &Card) -> u8 {
        self.public_card_count[*card as usize]
    } 
    pub fn get_history(&self, len: usize) -> Vec<ActionObservation> {
        debug_assert!(len <= MAX_HISTORY_LEN, "Use a proper len in get_history!");
        self.store[..len].to_vec()
    }
    pub fn store_len(&self) -> usize {
        self.store_len
    }
    pub fn dist_from_turn(&self) -> usize {
        if self.store_len() == 0 {
            0
        } else {
            self.dist_from_turn[self.store_len() - 1]
        }
    }
    pub fn store_at(&self, index: usize) -> &ActionObservation {
        &self.store[index]
    }
    pub fn get_dist_from_turn(&self, len: usize) -> Vec<usize> {
        debug_assert!(len <= MAX_HISTORY_LEN, "Use a proper len in get_dist_from_turn!");
        self.dist_from_turn[..len].to_vec()
    }
    pub fn next_player(&self, player_id: usize) -> usize {
        let mut current_turn: usize = (player_id + 1) % 6;
        while self.latest_influence()[current_turn] == 0 {
            current_turn = (current_turn + 1) % 6;
        }
        current_turn
    }
    pub fn push_ao(&mut self, ao: ActionObservation) {
        // Adds ActionObservation to History and updates relevant gamestate
        // Here gamestate, influence and coin values may be updated
        let ao_name: AOName = ao.name();
        if ao_name == AOName::Income {
            // Case I1
            self.push(ao, true);
            self.add_coins(ao.player_id(), 1);
        } else if ao_name == AOName::Coup {
            // Case COUP1
            self.push(ao, true);
            self.subtract_coins(ao.player_id(), 7);
        } else if ao_name == AOName::Assassinate{
            // Case B1-B12
            self.push(ao, true);
            self.subtract_coins(ao.player_id(), 3);
        } else if [AOName::Steal, AOName::Exchange, AOName::ForeignAid, AOName::Tax].contains(&ao_name){
            // Case Start of Turn C1-C9 A1-A3 FA1-FA4 D1-D3
            self.push(ao, true);
        } else if ao.name() == AOName::RevealRedraw {
            self.push(ao, false);
            if self.store_len >= 3 && self.store[self.store_len - 3].name() == AOName::Tax {
                let tax_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].player_id();
                self.add_coins(tax_id, 3);
            }
        } else if ao.name() == AOName::Discard {
            // Always pushes
            self.push(ao, false);
            // Always subtracts influence
            if ao.no_cards() == 1 {
                self.subtract_influence(ao.player_id(), 1);
                let card: Card = ao.cards()[0];
                self.add_public_card_count(&card);
                
            } else if ao.no_cards() == 2 {
                self.subtract_influence(ao.player_id(), 2);
                for card in ao.cards().iter(){
                    self.add_public_card_count(card);
                }
            } else {
                panic!("Discarding too many cards");
            }
            // Case Checked
            // Case FA4 C3 C8 D3
            if self.store_len >= 4{
                if self.store[self.store_len - 3].name() == AOName::BlockSteal && self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].name() == AOName::Steal{
                    // Case C3 C8
                    // Coup Discard, can lookback to previous move -5 as BlockSteal so we include second condition
                    let stealer_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].player_id();
                    let stolen_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].opposing_player_id();
                    let amount: u8 = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].amount();
                    if self.latest_influence()[stealer_id] != 0 {
                        self.add_coins(stealer_id, amount);
                    }
                    if self.latest_influence()[stolen_id] != 0 {
                        self.subtract_coins(stolen_id, amount);
                    }
                } else if self.store[self.store_len - 4].name() == AOName::ForeignAid && self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].name() == AOName::ForeignAid{
                    let fa_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].player_id();
                    self.add_coins(fa_id, 2);
                } else if self.store[self.store_len - 4].name() == AOName::Steal{
                    // if Blocker dead 
                    // Case C6 C7 C8 C9
                    if self.latest_influence()[self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].opposing_player_id()] == 0 {
                        let stealer_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].player_id();
                        let stolen_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].opposing_player_id();
                        let amount: u8 = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].amount();
                        if self.latest_influence()[stealer_id] != 0 {
                            self.add_coins(stealer_id, amount);
                        }
                        // Because if person is dead, coins will reset to 0 in logic and we subtract influence at start
                        // Which we have to because of the if blocker dead part
                        if self.latest_influence()[stolen_id] != 0 {
                            self.subtract_coins(stolen_id, amount);
                        }
                    }
                }
            } else if self.store_len == 3 {
                if self.store[self.store_len - 3].name() == AOName::BlockSteal && self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].name() == AOName::Steal{
                    // Case C3 C8
                    // Coup Discard, can lookback to previous move -5 as BlockSteal so we include second condition
                    let stealer_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].player_id();
                    let stolen_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].opposing_player_id();
                    let amount: u8 = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].amount();
                    if self.latest_influence()[stealer_id] != 0 {
                        self.add_coins(stealer_id, amount);
                    }
                    if self.latest_influence()[stolen_id] != 0 {
                        self.subtract_coins(stolen_id, amount);
                    }
                }
            }
        
        } else if ao.name() == AOName::CollectiveBlock && ao.final_actioner() == ao.opposing_player_id() {
            // Case when nobody blocks
            // Case FA1
            // Player being challenged is the FA player_id
            self.push(ao, false);
            let fa_id:usize = ao.opposing_player_id();
            self.add_coins(fa_id, 2);
        } else if ao.name() == AOName::CollectiveChallenge {
            self.push(ao, false);
            if self.store[self.store_len - 2].name() == AOName::Tax {
                // Case D1
                if ao.opposing_player_id() == ao.final_actioner() {
                    // Case D1 Nobody Challenges
                    let tax_id:usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].player_id();
                    self.add_coins(tax_id, 3);
                }
            } 
        } else if ao.name() == AOName::BlockSteal {
            self.push(ao, false);
            if ao.player_id() == ao.opposing_player_id() {
                // In the case where player passes on BlockSteal
                // Cases C1 C6 update coins
                let stealer_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].player_id();
                let stolen_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].opposing_player_id();
                let amount: u8 = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].amount();
                self.add_coins(stealer_id, amount);
                self.subtract_coins(stolen_id, amount);
            }
        } else {
            // All other cases
            self.push(ao, false);
        }
    }

    pub fn remove_ao(&mut self){
        // Always pop after reversing the statuses
        let last_name: AOName = self.store[self.store_len - 1].name();
        let ao: ActionObservation = self.store[self.store_len - 1];
        // Always Pop
        // public_card_count is not a history so we have to update it
        if last_name == AOName::Discard{
            if ao.no_cards() == 1 {
                let card: Card = ao.cards()[0];
                self.subtract_public_card_count(&card);
            } else if ao.no_cards() == 2 {
                for card in ao.cards().iter(){
                    self.subtract_public_card_count(card);
                }
            } else {
                debug_assert!(false, "no_cards is impossible amount!");
            }
        }
        self.pop();
    }
    pub fn add_moves(&self, changed_vec: &mut Vec<ActionObservation>, player_id: usize, move_name: AOName, private_player: Option<usize>) {
        // Adds possible move for a particular AOName that are allowed by influence and coins
        let bool_know_priv_info = if let Some(player) = private_player { player == player_id} else {false};
        let mut id: usize = (player_id + 1) % 6;
        match move_name {
            AOName::Income => {
                changed_vec.push(ActionObservation::Income{player_id: player_id});
            },
            AOName::ForeignAid => {
                changed_vec.push(ActionObservation::ForeignAid {player_id: player_id});
            },
            AOName::Tax => {
                changed_vec.push(ActionObservation::Tax {player_id: player_id});
            },
            AOName::Steal => {
                while id != player_id {
                    if self.latest_influence()[id] > 0 {
                        if self.latest_coins()[id] > 1 {
                            changed_vec.push(ActionObservation::Steal { player_id: player_id, opposing_player_id: id, amount: 2 });
                        } else {
                            changed_vec.push(ActionObservation::Steal { player_id: player_id, opposing_player_id: id, amount: self.latest_coins()[id] });
                        }
                    }
                    id = (id + 1) % 6;
                }
            },
            AOName::Assassinate => {
                if self.latest_coins()[player_id] >= 3{
                    while id != player_id {
                        if self.latest_influence()[id] > 0 {
                            changed_vec.push(ActionObservation::Assassinate { player_id: player_id, opposing_player_id: id });
                        }
                        id = (id + 1) % 6;
                    }
                }
            },
            AOName::Coup => {
                if self.latest_coins()[player_id] >= 7 {
                    while id != player_id {
                        if self.latest_influence()[id] > 0 {
                            changed_vec.push(ActionObservation::Coup { player_id: player_id, opposing_player_id: id });
                        }
                        id = (id + 1) % 6;
                    }
                }
            },
            AOName::CollectiveChallenge => {
                let opposing_player_id = player_id; // Assuming player_id is the target of the challenge
                let base_participants = [false; 6];
                // Doing it Bitwise from 0..64
                // Checks every combination by representing in 000000 or 010010
                
                //Adding case where nobody challenges make finalchallenger the person being challenged (an impossible result)
                changed_vec.push(ActionObservation::CollectiveChallenge { 
                    participants: [false, false, false, false, false, false], 
                    opposing_player_id: opposing_player_id, final_actioner: opposing_player_id 
                });

                for comb in 1..(1 << 6) {
                    let mut participants = base_participants;
                    let mut any_true = false;
                    //skip case where targetted player is true, it should always be false
                    if (comb & (1 << opposing_player_id)) != 0 {
                        continue;
                    }
                    // Modifies the participants based on combination and them being alive
                    for i in 0..6 {
                        // Skip opposing player or players with no influence
                        if i == opposing_player_id || self.latest_influence()[i] <= 0 {
                            continue;
                        }
                        // 01 << 1 => 10 which is like 2
                        // If comb player position has a 1
                        if (comb & (1 << i)) != 0 {
                            participants[i] = true;
                            any_true = true;
                        }
                    }
    
                    // Only add to changed_vec if there is at least one challenger
                    // Can be false because player influence may all be 0
                    if any_true {
                        for i in 0..6 {
                            if participants[i] {
                                let final_actioner = i;
                                changed_vec.push(ActionObservation::CollectiveChallenge {
                                    participants,
                                    opposing_player_id,
                                    final_actioner,
                                });
                            }
                        }
                    }
                }
            },
            AOName::CollectiveBlock => {
                //player_id taken by add_move here will be the player being targetted!
                let opposing_player_id = player_id; // Assuming player_id is the target of the challenge
                let base_participants = [false; 6];
                // Doing it Bitwise from 0..64
                // Checks every combination by representing in 000000 or 010010
                changed_vec.push(ActionObservation::CollectiveBlock { 
                    participants: [false, false, false, false, false, false], 
                    opposing_player_id: opposing_player_id, final_actioner: opposing_player_id 
                });
                for comb in 1..(1 << 6) {
                    let mut participants = base_participants;
                    let mut any_true = false;
                    //skip case where targetted player is true, it should always be false
                    if (comb & (1 << opposing_player_id)) != 0 {
                        continue;
                    }
                    // Modifies the participants based on combination and them being alive
                    for i in 0..6 {
                        // Skip opposing player or players with no influence
                        if i == opposing_player_id || self.latest_influence()[i] <= 0 {
                            continue;
                        }
                        // 01 << 1 => 10 which is like 2
                        // If comb player position has a 1
                        if (comb & (1 << i)) != 0 {
                            participants[i] = true;
                            any_true = true;
                        }
                    }
                    
                    // Only add to changed_vec if there is at least one challenger
                    if any_true {
                        for i in 0..6 {
                            if participants[i] {
                                let final_actioner = i;
                                changed_vec.push(ActionObservation::CollectiveBlock {
                                    participants,
                                    opposing_player_id,
                                    final_actioner,
                                });
                            }
                        }
                    }
                }
            },
            AOName::BlockSteal => {
                let stealer_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].player_id();
                changed_vec.push(ActionObservation::BlockSteal { player_id: player_id, opposing_player_id: stealer_id, card: Card::Ambassador});
                changed_vec.push(ActionObservation::BlockSteal { player_id: player_id, opposing_player_id: stealer_id, card: Card::Captain});
                // This is a move the represents a Pass
                changed_vec.push(ActionObservation::BlockSteal { player_id: player_id, opposing_player_id: player_id, card: Card::Captain});
            },
            AOName::BlockAssassinate => {
                // BlockAssassinate is a legal ActionObservation to be proposed
                // Finding victim id from Assassinate
                let attacker_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].player_id();
                changed_vec.push(ActionObservation::BlockAssassinate { player_id: player_id, opposing_player_id: attacker_id});
                // This is a move that represents a Pass
                changed_vec.push(ActionObservation::BlockAssassinate { player_id: player_id, opposing_player_id: player_id});

            },
            
            AOName::Discard => {
                let num_dead_amb: u8 =  self.get_public_card_count(&Card::Ambassador);
                let num_dead_ass: u8 =  self.get_public_card_count(&Card::Assassin);
                let num_dead_cpt: u8 =  self.get_public_card_count(&Card::Captain);
                let num_dead_duk: u8 =  self.get_public_card_count(&Card::Duke);
                let num_dead_con: u8 =  self.get_public_card_count(&Card::Contessa);

                if num_dead_amb < 3 {
                    changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Ambassador, Card::Ambassador], no_cards: 1});
                }
                if num_dead_ass < 3 {
                    changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Assassin, Card::Assassin], no_cards: 1});
                }
                if num_dead_cpt < 3 {
                    changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Captain, Card::Captain], no_cards: 1});
                }
                if num_dead_duk < 3 {
                    changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Duke, Card::Duke], no_cards: 1});
                }
                if num_dead_con < 3 {
                    changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Contessa, Card::Contessa], no_cards: 1});
                }
            },
            AOName::RevealRedraw => {
                //TOChange
                // Always after RevealRedraw
                // CollectiveChallenge => EvalResult (Failure) => RevealRedraw
                let num_dead_amb: u8 =  self.get_public_card_count(&Card::Ambassador);
                let num_dead_ass: u8 =  self.get_public_card_count(&Card::Assassin);
                let num_dead_cpt: u8 =  self.get_public_card_count(&Card::Captain);
                let num_dead_duk: u8 =  self.get_public_card_count(&Card::Duke);
                let num_dead_con: u8 =  self.get_public_card_count(&Card::Contessa);
                if self.store_len >= 2 {
                    if self.store[self.store_len - 2].name() == AOName::Assassinate {
                        // B5 - B11
                        // Its nested cause if you dont, the debug_assert will trigger, and i want the check to work like that
                        // Spaghetti code i know
                        if num_dead_ass < 3 {
                            changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Assassin , redraw: Card::Assassin});
                            if bool_know_priv_info {
                                if num_dead_amb < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Assassin , redraw: Card::Ambassador});
                                }
                                if num_dead_cpt < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Assassin , redraw: Card::Captain});
                                }
                                if num_dead_duk < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Assassin , redraw: Card::Duke});
                                }
                                if num_dead_con < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Assassin , redraw: Card::Contessa});
                                }
                            }
                        }
                        if num_dead_amb < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Ambassador, Card::Ambassador], no_cards: 1});
                        }
                        if num_dead_cpt < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Captain, Card::Captain], no_cards: 1});
                        }
                        if num_dead_duk < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Duke, Card::Duke], no_cards: 1});
                        }
                        if num_dead_con < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Contessa, Card::Contessa], no_cards: 1});
                        }
                    } else if self.store[self.store_len - 2].name() == AOName::Steal {
                        // C8 - C12
                        if num_dead_cpt < 3 {
                            changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Captain, redraw: Card::Captain});
                            if bool_know_priv_info {
                                if num_dead_amb < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Captain, redraw: Card::Ambassador});
                                }
                                if num_dead_ass < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Captain, redraw: Card::Assassin});
                                }
                                if num_dead_duk < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Captain, redraw: Card::Duke});
                                }
                                if num_dead_con < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Captain, redraw: Card::Contessa});
                                }
                            }
                        }
                        if num_dead_amb < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Ambassador, Card::Ambassador], no_cards: 1});
                        }
                        if num_dead_ass < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Assassin, Card::Assassin], no_cards: 1});
                        }
                        if num_dead_duk < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Duke, Card::Duke], no_cards: 1});
                        }
                        if num_dead_con < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Contessa, Card::Contessa], no_cards: 1});
                        }
                    } else if self.store[self.store_len - 2].name() == AOName::Tax {
                        // D3
                        if num_dead_duk < 3 {
                            changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Duke });
                            if bool_know_priv_info {
                                if num_dead_amb < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Ambassador });
                                }
                                if num_dead_ass < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Assassin });
                                }
                                if num_dead_cpt < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Captain });
                                }
                                if num_dead_con < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Contessa });
                                }
                            }
                        }
                        if num_dead_amb < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Ambassador, Card::Ambassador], no_cards: 1});
                        }
                        if num_dead_ass < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Assassin, Card::Assassin], no_cards: 1});
                        }
                        if num_dead_cpt < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Captain, Card::Captain], no_cards: 1});
                        }
                        if num_dead_con < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Contessa, Card::Contessa], no_cards: 1});
                        }
                    } else if self.store[self.store_len - 2].name() == AOName::Exchange {
                        // A3
                        if num_dead_amb < 3 {
                            changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Ambassador, redraw: Card::Ambassador });
                            if bool_know_priv_info {
                                if num_dead_ass < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Ambassador, redraw: Card::Assassin });
                                }
                                if num_dead_cpt < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Ambassador, redraw: Card::Captain });
                                }
                                if num_dead_duk < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Ambassador, redraw: Card::Duke });
                                }
                                if num_dead_con < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Ambassador, redraw: Card::Contessa });
                                }
                            }
                        }
                        if num_dead_ass < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Assassin, Card::Assassin], no_cards: 1});
                        }
                        if num_dead_cpt < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Captain, Card::Captain], no_cards: 1});
                        }
                        if num_dead_duk < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Duke, Card::Duke], no_cards: 1});
                        }
                        if num_dead_con < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Contessa, Card::Contessa], no_cards: 1});
                        }
                    } else if self.store[self.store_len - 2].name() == AOName::BlockAssassinate {
                        if num_dead_con < 3 {
                            // B4 B11
                            // Special Assassinate Case
                            changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Contessa, redraw: Card::Contessa });
                            if bool_know_priv_info {
                                if num_dead_amb < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Contessa, redraw: Card::Ambassador });
                                }
                                if num_dead_ass < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Contessa, redraw: Card::Assassin });
                                }
                                if num_dead_cpt < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Contessa, redraw: Card::Captain });
                                }
                                if num_dead_duk < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Contessa, redraw: Card::Duke });
                                }
                            }
                        }
                        if self.latest_influence()[player_id] == 1 {
                            if num_dead_amb < 3 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Ambassador, Card::Ambassador], no_cards: 1});
                            }
                            if num_dead_ass < 3 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Assassin, Card::Assassin], no_cards: 1});
                            }
                            if num_dead_cpt < 3 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Captain, Card::Captain], no_cards: 1});
                            }
                            if num_dead_duk < 3 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Duke, Card::Duke], no_cards: 1});
                            }
                        } else {
                            debug_assert!(self.latest_influence()[player_id] == 2 , "Improper Influence reached in add_moves discard");
                            if num_dead_amb < 2 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Ambassador, Card::Ambassador], no_cards: 2});
                            }
                            if num_dead_ass < 2 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Assassin, Card::Assassin], no_cards: 2});
                            }
                            if num_dead_cpt < 2 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Captain, Card::Captain], no_cards: 2});
                            }
                            if num_dead_duk < 2 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Duke, Card::Duke], no_cards: 2});
                            }
                            if num_dead_amb < 3 {
                                if num_dead_ass < 3 {
                                    changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Ambassador, Card::Assassin], no_cards: 2});
                                }
                                if num_dead_cpt < 3 {
                                    changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Ambassador, Card::Captain], no_cards: 2});
                                }
                                if num_dead_duk < 3 {
                                    changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Ambassador, Card::Duke], no_cards: 2});
                                }
                            }
                            if num_dead_ass < 3 {
                                if num_dead_cpt < 3 {
                                    changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Assassin, Card::Captain], no_cards: 2});
                                }
                                if num_dead_duk < 3 {
                                    changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Assassin, Card::Duke], no_cards: 2});
                                }
                            }
                            if num_dead_cpt < 3 {
                                if num_dead_duk < 3 {
                                    changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Captain, Card::Duke], no_cards: 2});
                                }
                            }
                        }
                    } else if self.store[self.store_len - 2].name() == AOName::BlockSteal {
                        // 
                        if self.store[self.store_len - 2].card() == Card::Ambassador {
                            if num_dead_amb < 3 {
                                changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Ambassador, redraw: Card::Ambassador });
                                if bool_know_priv_info {
                                    if num_dead_ass < 3 {
                                        changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Ambassador, redraw: Card::Assassin });
                                    }
                                    if num_dead_cpt < 3 {
                                        changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Ambassador, redraw: Card::Captain });
                                    }
                                    if num_dead_duk < 3 {
                                        changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Ambassador, redraw: Card::Duke });
                                    }
                                    if num_dead_con < 3 {
                                        changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Ambassador, redraw: Card::Contessa });
                                    }
                                }
                            }
                            if num_dead_ass < 3 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Assassin, Card::Assassin], no_cards: 1});
                            }
                            if num_dead_cpt < 3 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Captain, Card::Captain], no_cards: 1});
                            }
                            if num_dead_duk < 3 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Duke, Card::Duke], no_cards: 1});
                            }
                            if num_dead_con < 3 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Contessa, Card::Contessa], no_cards: 1});
                            }
                        } else if self.store[self.store_len - 2].card() == Card::Captain {
                            if num_dead_cpt < 3 {
                                changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Captain, redraw: Card::Captain});
                                if bool_know_priv_info {
                                    if num_dead_amb < 3 {
                                        changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Captain, redraw: Card::Ambassador});
                                    }
                                    if num_dead_ass < 3 {
                                        changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Captain, redraw: Card::Assassin});
                                    }
                                    if num_dead_duk < 3 {
                                        changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Captain, redraw: Card::Duke});
                                    }
                                    if num_dead_con < 3 {
                                        changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Captain, redraw: Card::Contessa});
                                    }
                                }
                            }
                            if num_dead_amb < 3 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Ambassador, Card::Ambassador], no_cards: 1});
                            }
                            if num_dead_ass < 3 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Assassin, Card::Assassin], no_cards: 1});
                            }
                            if num_dead_duk < 3 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Duke, Card::Duke], no_cards: 1});
                            }
                            if num_dead_con < 3 {
                                changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Contessa, Card::Contessa], no_cards: 1});
                            }
                        } else  {
                            debug_assert!(false, "Card in BlockSteal Seems to be wrong, so RevealRedraw add_move does not work well");
                        }
                    } else if self.store_len >= 3 && self.store[self.store_len - 3].name() == AOName::ForeignAid {
                        // FA3
                        if num_dead_duk < 3 {
                            changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Duke });
                            if bool_know_priv_info {
                                if num_dead_amb < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Ambassador });
                                }
                                if num_dead_ass < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Assassin });
                                }
                                if num_dead_cpt < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Captain });
                                }
                                if num_dead_con < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Contessa });
                                }
                            }
                        }
                        if num_dead_amb < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Ambassador, Card::Ambassador], no_cards: 1});
                        }
                        if num_dead_ass < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Assassin, Card::Assassin], no_cards: 1});
                        }
                        if num_dead_cpt < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Captain, Card::Captain], no_cards: 1});
                        }
                        if num_dead_con < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Contessa, Card::Contessa], no_cards: 1});
                        }
                    } else {
                        debug_assert!(false, "unintended state reached in add_moves revealredraw");
                    }
                } else if self.store_len >= 3 {
                    if self.store[self.store_len - 3].name() == AOName::ForeignAid {
                        // FA3
                        if num_dead_duk < 3 {
                            changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Duke });
                            if bool_know_priv_info {
                                if num_dead_amb < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Ambassador });
                                }
                                if num_dead_ass < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Assassin });
                                }
                                if num_dead_cpt < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Captain });
                                }
                                if num_dead_con < 3 {
                                    changed_vec.push(ActionObservation::RevealRedraw { player_id: player_id, reveal: Card::Duke, redraw: Card::Contessa });
                                }
                            }
                        }
                        if num_dead_amb < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Ambassador, Card::Ambassador], no_cards: 1});
                        }
                        if num_dead_ass < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Assassin, Card::Assassin], no_cards: 1});
                        }
                        if num_dead_cpt < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Captain, Card::Captain], no_cards: 1});
                        }
                        if num_dead_con < 3 {
                            changed_vec.push(ActionObservation::Discard { player_id: player_id, card: [Card::Contessa, Card::Contessa], no_cards: 1});
                        }
                    } else {
                        debug_assert!(false, "unintended state reached in add_moves revealredraw");
                    }
                } else {
                    debug_assert!(false, "unintended state reached in add_moves revealredraw");
                } 
            },
            AOName::Exchange => {
                changed_vec.push(ActionObservation::Exchange{ player_id: player_id});  
            },
            AOName::ExchangeChoice => {
                let num_dead_amb: u8 =  self.get_public_card_count(&Card::Ambassador);
                let num_dead_ass: u8 =  self.get_public_card_count(&Card::Assassin);
                let num_dead_cpt: u8 =  self.get_public_card_count(&Card::Captain);
                let num_dead_duk: u8 =  self.get_public_card_count(&Card::Duke);
                let num_dead_con: u8 =  self.get_public_card_count(&Card::Contessa);
                let count_card_arr = [num_dead_amb, num_dead_ass, num_dead_cpt, num_dead_duk, num_dead_con];
                if bool_know_priv_info {
                    // Let hand be determined externally so hand here is a default value
                    let mut total_counts: [u8; 5] = [0; 5];
                    for i in 0..5 {
                        total_counts[i] += 1;
                        if total_counts[i] + count_card_arr[i] < 4 {
                            for j in i..5 {
                                total_counts[j] += 1;
                                if total_counts[j] + count_card_arr[j] < 4 {
                                    changed_vec.push(ActionObservation::ExchangeChoice { player_id: player_id, relinquish: [Card::try_from(i as u8).unwrap(), Card::try_from(j as u8).unwrap()]});
                                }
                                total_counts[j] -= 1;
                            }
                        }
                        total_counts[i] -= 1;
                    }
                    // if self.latest_influence()[player_id] == 2 {
                    //     for i_p in 0..5 {
                    //         let mut total_counts: [u8; 5] = [0; 5];
                    //         total_counts[i_p] += 1;
                    //         if total_counts[i_p] + count_card_arr[i_p] < 3 {
                    //             for j_p in i_p..5 {
                    //                 total_counts[j_p] += 1;
                    //                 if total_counts[j_p] + count_card_arr[j_p] < 3 {
                    //                     for i in 0..5 {
                    //                         total_counts[i] += 1;
                    //                         if total_counts[i] + count_card_arr[i] < 3 {
                    //                             for j in i..5 {
                    //                                 total_counts[j] += 1;
                    //                                 if total_counts[j] + count_card_arr[j] < 3 {
                    //                                     changed_vec.push(ActionObservation::ExchangeChoice { player_id: player_id, no_cards: 2, hand: [Card::try_from(i_p as u8).unwrap(), Card::try_from(j_p as u8).unwrap()], relinquish: [Card::try_from(i as u8).unwrap(), Card::try_from(j as u8).unwrap()]});
                    //                                 }
                    //                                 total_counts[j] -= 1;
                    //                             }
                    //                         }
                    //                         total_counts[i] -= 1;
                    //                     }
                    //                 }
                    //                 total_counts[j_p] -= 1;
                    //             }
                    //         }
                    //     }
                    // } else {
                    //     debug_assert!(self.latest_influence()[player_id] == 1, "influence should be either 1 or 2");
                    //     for i_p in 0..5 {
                    //         let mut total_counts: [u8; 5] = [0; 5];
                    //         total_counts[i_p] += 1;
                    //         if total_counts[i_p] + count_card_arr[i_p] < 3 {
                    //             for i in 0..5 {
                    //                 total_counts[i] += 1;
                    //                 if total_counts[i] + count_card_arr[i] < 3 {
                    //                     for j in i..5 {
                    //                         total_counts[j] += 1;
                    //                         if total_counts[j] + count_card_arr[j] < 3 {
                    //                             changed_vec.push(ActionObservation::ExchangeChoice { player_id: player_id, no_cards: 1, hand: [Card::try_from(i_p as u8).unwrap(), Card::try_from(i_p as u8).unwrap()], relinquish: [Card::try_from(i as u8).unwrap(), Card::try_from(j as u8).unwrap()]});
                    //                         }
                    //                         total_counts[j] -= 1;
                    //                     }
                    //                 }
                    //                 total_counts[i] -= 1;
                    //             }
                    //         }
                    //     }

                    // }
                } else {
                    // for i in 0..5 {
                    //     if count_card_arr[i] < 2 {
                    //         changed_vec.push(ActionObservation::ExchangeChoice { player_id: player_id, no_cards: self.latest_influence()[player_id], hand: [Card::Ambassador, Card::Ambassador], relinquish: [Card::try_from(i as u8).unwrap(), Card::try_from(i as u8).unwrap()]});
                    //     }
                    //     for j in (i + 1)..5 {
                    //         if count_card_arr[i] < 3 && count_card_arr[j] < 3 {
                    //             changed_vec.push(ActionObservation::ExchangeChoice { player_id: player_id, no_cards: self.latest_influence()[player_id], hand: [Card::Ambassador, Card::Ambassador], relinquish: [Card::try_from(i as u8).unwrap(), Card::try_from(j as u8).unwrap()]});
                    //         }
                    //     }
                    // }
                    // Assumes hand and relinquish are unknown, so placeholder Ambassadors are left there
                    changed_vec.push(ActionObservation::ExchangeChoice { player_id: player_id, relinquish: [Card::Ambassador, Card::Ambassador]});
                }
            },
            AOName::ExchangeDraw => {
                // No Pruning of impossible moves as I am going to do it in the naive_prob check anyways
                if bool_know_priv_info {
                    let num_dead_amb: u8 =  self.get_public_card_count(&Card::Ambassador);
                    let num_dead_ass: u8 =  self.get_public_card_count(&Card::Assassin);
                    let num_dead_cpt: u8 =  self.get_public_card_count(&Card::Captain);
                    let num_dead_duk: u8 =  self.get_public_card_count(&Card::Duke);
                    let num_dead_con: u8 =  self.get_public_card_count(&Card::Contessa);
                    if num_dead_amb < 2 {
                        changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Ambassador, Card::Ambassador] });
                    }
                    if num_dead_amb < 3 {
                        if num_dead_ass < 3 {
                            changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Ambassador, Card::Assassin] });
                        }
                        if num_dead_cpt < 3 {
                            changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Ambassador, Card::Captain] });
                        }
                        if num_dead_duk < 3 {
                            changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Ambassador, Card::Duke] });
                        }
                        if num_dead_con < 3 {
                            changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Ambassador, Card::Contessa] });
                        }
                    }
                    if num_dead_ass < 2 {
                        changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Assassin, Card::Assassin] });
                    }
                    if num_dead_ass < 3 {
                        if num_dead_cpt < 3 {
                            changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Assassin, Card::Captain] });
                        }
                        if num_dead_duk < 3 {
                            changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Assassin, Card::Duke] });
                        }
                        if num_dead_con < 3 {
                            changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Assassin, Card::Contessa] });
                        }
                    }
                    if num_dead_cpt < 2 {
                        changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Captain, Card::Captain] });
                    }
                    if num_dead_cpt < 3 {
                        if num_dead_duk < 3 {
                            changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Captain, Card::Duke] });
                        }
                        if num_dead_con < 3 {
                            changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Captain, Card::Contessa] });
                        }
                    }
                    if num_dead_duk < 2 {
                        changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Duke, Card::Duke] });
                    }
                    if num_dead_duk < 3 {
                        if num_dead_con < 3 {
                            changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Duke, Card::Contessa] });
                        }
                    }
                    if num_dead_con < 2 {
                        changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Contessa, Card::Contessa] });
                    }
                } else {
                    // Assumes card info is unknown and placeholders of Ambassadors are used
                    changed_vec.push(ActionObservation::ExchangeDraw { player_id: player_id, card: [Card::Ambassador, Card::Ambassador] });
                }
            }
            _ => panic!("{}", format!("add_move for AOName {:?} not implemented", move_name)),
        }
    }
    /// This technically does not consider what cards the player can or should have!
    /// Generates legal moves for the purpose of search
    pub fn generate_legal_moves(&self, private_player: Option<usize>) -> Vec<ActionObservation>{
        // Refer to paths.txt for different cases
        let mut output: Vec<ActionObservation> = Vec::new();
        if self.store_len == 0 {
            self.add_moves(&mut output, self.current_player_turn, AOName::Income, private_player);
            self.add_moves(&mut output, self.current_player_turn, AOName::ForeignAid, private_player);
            self.add_moves(&mut output, self.current_player_turn, AOName::Tax, private_player);
            self.add_moves(&mut output, self.current_player_turn, AOName::Exchange, private_player);
            self.add_moves(&mut output, self.current_player_turn, AOName::Steal, private_player);
            self.add_moves(&mut output, self.current_player_turn, AOName::Assassinate, private_player);
            return output;
        }
        
        match self.store[self.store_len - 1].name() {
            AOName::EmptyAO => {
                // Case Checked
                // This should only happen at the start of the game
                debug_assert!(false, "Generating Legal Moves for EmptyAO not at the start of the game!");
            }
            AOName::Income => {
                // Case Checked
                let next_player_id: usize = self.next_player(self.current_player_turn);
                if self.latest_coins()[next_player_id] >= 10 {
                    self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                } else {
                    self.add_moves(&mut output, next_player_id, AOName::Income, private_player);
                    self.add_moves(&mut output, next_player_id, AOName::ForeignAid, private_player);
                    self.add_moves(&mut output, next_player_id, AOName::Tax, private_player);
                    self.add_moves(&mut output, next_player_id, AOName::Exchange, private_player);
                    self.add_moves(&mut output, next_player_id, AOName::Steal, private_player);
                    self.add_moves(&mut output, next_player_id, AOName::Assassinate, private_player);
                    self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                }
            },
            AOName::ForeignAid => {
                // Case Checked
                let last_player: usize = self.store[self.store_len - 1].player_id();
                self.add_moves(&mut output, last_player, AOName::CollectiveBlock, private_player);
            },
            AOName::Tax
            | AOName::Steal
            | AOName::Exchange 
            | AOName::Assassinate => {
                // Case Checked
                let last_player: usize = self.store[self.store_len - 1].player_id();
                self.add_moves(&mut output, last_player, AOName::CollectiveChallenge, private_player);
            },
            AOName::BlockSteal => {
                if self.store[self.store_len - 1].player_id() == self.store[self.store_len - 1].opposing_player_id() {
                    // If player passed on the block
                    // Case C6 & C1
                    let next_player_id: usize = self.next_player(self.current_player_turn);
                    if self.latest_coins()[next_player_id] >= 10 {
                        self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                    } else {
                        self.add_moves(&mut output, next_player_id, AOName::Income, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::ForeignAid, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Tax, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Exchange, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Steal, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Assassinate, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                    }
                } else {
                    // If player did not pass on the block
                    // C2 C3 C4 C7 C8 C9 => CollectiveChallenge
                    let victim_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].opposing_player_id();
                    self.add_moves(&mut output, victim_id, AOName::CollectiveChallenge, private_player);
                }
            },
            AOName::BlockAssassinate => {
                //Case Checked
                let victim_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].opposing_player_id();
                if self.store[self.store_len - 1].player_id() == self.store[self.store_len - 1].opposing_player_id() {
                    // If player passed on the block
                    // Case B1 B8
                    self.add_moves(&mut output, victim_id, AOName::Discard, private_player);
                } else {
                    // If player blocks
                    // B2 B3 B4 B5 B9 B10 B11 B12 => CollectiveChallenge
                    self.add_moves(&mut output, victim_id, AOName::CollectiveChallenge, private_player);
                }
            },
            AOName::Coup => {
                //Case Checked
                let opposing_player_id: usize = self.store[self.store_len - 1].opposing_player_id(); 
                self.add_moves(&mut output, opposing_player_id, AOName::Discard, private_player);
            },
            AOName::CollectiveChallenge => {
                // Case Checked
                // last_player here is a dummy an unused inside
                if self.store[self.store_len - 1].opposing_player_id() == self.store[self.store_len - 1].final_actioner() {
                    // Pass
                    // FA2
                    if self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].name() == AOName::ForeignAid ||
                    // B2 B9 
                    self.store_len >= 2 && self.store[self.store_len - 2].name() == AOName::BlockAssassinate ||
                    // C2 C7
                    self.store_len >= 2 && self.store[self.store_len - 2].name() == AOName::BlockSteal ||
                    // D1
                    self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].name() == AOName::Tax {
                        let next_player_id: usize = self.next_player(self.current_player_turn);
                        if self.latest_coins()[next_player_id] >= 10 {
                            self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                        } else {
                            self.add_moves(&mut output, next_player_id, AOName::Income, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::ForeignAid, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Tax, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Exchange, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Steal, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Assassinate, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                        }
                    } else if self.store_len >= 2 && self.store[self.store_len - 2].name() == AOName::Steal {
                        // In Cases: C1 C2 C3 C4 => BlockSteal
                        let victim_id: usize = self.store[self.store_len - 2].opposing_player_id();
                        self.add_moves(&mut output, victim_id, AOName::BlockSteal, private_player);
                    } else if self.store_len >= 2 && self.store[self.store_len - 2].name() == AOName::Assassinate {
                        // In Cases: B1 B2 B3 B4 B5 => BlockAssassinate
                        let victim_id: usize = self.store[self.store_len - 2].opposing_player_id();
                        self.add_moves(&mut output, victim_id, AOName::BlockAssassinate, private_player);
                    } else if self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].name() == AOName::Exchange {
                        // In Cases: A1 => ExchangeDraw
                        let exchanger_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].player_id();
                        self.add_moves(&mut output, exchanger_id, AOName::ExchangeDraw, private_player);
                    }
                } else {
                    // Success Case
                    let challenged_id: usize = self.store[self.store_len - 1].opposing_player_id();
                    self.add_moves(&mut output, challenged_id, AOName::RevealRedraw, private_player);
                }
            },
            AOName::CollectiveBlock => {
                // Case Checked
                debug_assert!(self.store[self.store_len - 2].name() == AOName::ForeignAid, "Collective Block Should only be after ForeignAid!");
                let blocker_id: usize = self.store[self.store_len - 1].final_actioner();
                if blocker_id == self.store[self.store_len - 1].opposing_player_id() {
                    // Case FA1 => Next Turn
                    // Nobody Blocks
                    let next_player_id: usize = self.next_player(self.current_player_turn);
                    if self.latest_coins()[next_player_id] >= 10 {
                        self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                    } else {
                        self.add_moves(&mut output, next_player_id, AOName::Income, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::ForeignAid, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Tax, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Exchange, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Steal, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Assassinate, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                    }
                } else {
                    // Case FA2 FA3 FA4 Challenge the duke
                    self.add_moves(&mut output, blocker_id, AOName::CollectiveChallenge, private_player);
                }
            },
            AOName::Discard => {
                //TODO: store_len check
                // In Cases: FA3 FA4 COUP1 A2 B1 B3 B4 B5 B6 B6 B7 B8 B10 B11 B12 C3 C4 C5 C8 C9 D2 D3| => Next Turn
                // In Cases: A3| => ExchangeChoice
                // In Cases: B8 B9 B9 B10 B11 B12 | => BlockAssassinate
                // In Cases: C6 C7 C8 C9 | => BlockSteal

                if self.store[self.store_len - 2].name() == AOName::Coup ||
                self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].name() == AOName::ForeignAid ||
                self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].name() == AOName::Tax{
                    // In Case COUP1 => Next Turn
                    // In Cases FA3 FA4 => Next Turn
                    // In Cases D2 D3 => Next Turn
                    let next_player_id: usize = self.next_player(self.current_player_turn);
                    if self.latest_coins()[next_player_id] >= 10 {
                        self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                    } else {
                        self.add_moves(&mut output, next_player_id, AOName::Income, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::ForeignAid, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Tax, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Exchange, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Steal, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Assassinate, private_player);
                        self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                    }
                } else if self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].name() == AOName::Exchange {
                    if self.store[self.store_len - 2].name() == AOName::RevealRedraw {
                        // A3 => Exchange Choice
                        let exchange_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].player_id();
                        self.add_moves(&mut output, exchange_id, AOName::ExchangeDraw, private_player);
                    } else if self.store[self.store_len - 2].name() == AOName::CollectiveChallenge {
                        // A2 => Next Turn
                        debug_assert!(self.store[self.store_len - 2].opposing_player_id() != self.store[self.store_len - 2].final_actioner(), "Bad AOResult in generate_legal_moves Discard Ambassador");
                        let next_player_id: usize = self.next_player(self.current_player_turn);
                        if self.latest_coins()[next_player_id] >= 10 {
                            self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                        } else {
                            self.add_moves(&mut output, next_player_id, AOName::Income, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::ForeignAid, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Tax, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Exchange, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Steal, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Assassinate, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                        }
                    } else {
                        debug_assert!(false, "unintended state reached in generate_legal_moves Discard Ambassador");
                    }
                } else if self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].name() == AOName::Assassinate {
                    if self.store[self.store_len - 4].name() == AOName::Assassinate{
                        // B8 B9 B9 B10 B11 B12 => Block Assassinate
                        let blocker_id: usize = self.store[self.store_len - 4].opposing_player_id();
                        if self.latest_influence()[blocker_id] > 0 {
                            self.add_moves(&mut output, blocker_id, AOName::BlockAssassinate, private_player);
                        } else {
                            let next_player_id: usize = self.next_player(self.current_player_turn);
                            if self.latest_coins()[next_player_id] >= 10 {
                                self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                            } else {
                                self.add_moves(&mut output, next_player_id, AOName::Income, private_player);
                                self.add_moves(&mut output, next_player_id, AOName::ForeignAid, private_player);
                                self.add_moves(&mut output, next_player_id, AOName::Tax, private_player);
                                self.add_moves(&mut output, next_player_id, AOName::Exchange, private_player);
                                self.add_moves(&mut output, next_player_id, AOName::Steal, private_player);
                                self.add_moves(&mut output, next_player_id, AOName::Assassinate, private_player);
                                self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                            }
                        }
                    } else {
                        // Every other Assassinate option => Next Turn
                        let next_player_id: usize = self.next_player(self.current_player_turn);
                        if self.latest_coins()[next_player_id] >= 10 {
                            self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                        } else {
                            self.add_moves(&mut output, next_player_id, AOName::Income, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::ForeignAid, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Tax, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Exchange, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Steal, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Assassinate, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                        }
                    }
                } else if self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].name() == AOName::Steal {
                    if self.store_len >= 4 && self.store[self.store_len - 4].name() == AOName::Steal{
                       // In Cases: C6 C7 C8 C9 | => BlockSteal
                        let blocker_id: usize = self.store[self.store_len - self.dist_from_turn[self.store_len - 1]].opposing_player_id();
                        if self.latest_influence()[blocker_id] > 0 {
                            self.add_moves(&mut output, blocker_id, AOName::BlockSteal, private_player);
                        } else {
                            // blocker is dead
                            let next_player_id: usize = self.next_player(self.current_player_turn);
                            if self.latest_coins()[next_player_id] >= 10 {
                                self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                            } else {
                                self.add_moves(&mut output, next_player_id, AOName::Income, private_player);
                                self.add_moves(&mut output, next_player_id, AOName::ForeignAid, private_player);
                                self.add_moves(&mut output, next_player_id, AOName::Tax, private_player);
                                self.add_moves(&mut output, next_player_id, AOName::Exchange, private_player);
                                self.add_moves(&mut output, next_player_id, AOName::Steal, private_player);
                                self.add_moves(&mut output, next_player_id, AOName::Assassinate, private_player);
                                self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                            }
                        }
                    } else {
                        // Every other Steal option => Next Turn
                        let next_player_id: usize = self.next_player(self.current_player_turn);
                        if self.latest_coins()[next_player_id] >= 10 {
                            self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                        } else {
                            self.add_moves(&mut output, next_player_id, AOName::Income, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::ForeignAid, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Tax, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Exchange, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Steal, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Assassinate, private_player);
                            self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                        }
                    }
                } else {
                    debug_assert!(false, "unintended state reached in generate_legal_moves Discard");
                }
                
            },
            AOName::RevealRedraw => {
                // Case Checked
                // In all cases: -3 is a challenge -2 is AOResult::Failure and the next move is discard
                // Challenge failed, initiator of challenge loses and discards
                //temp eventually all -2
                let discard_id: usize = self.store[self.store_len - 2].player_id();
                self.add_moves(&mut output, discard_id, AOName::Discard, private_player);
            },
            AOName::ExchangeDraw => {
                // ID from ExchangeDraw
                let exchange_id: usize = self.store[self.store_len - 1].player_id();
                self.add_moves(&mut output, exchange_id, AOName::ExchangeChoice, private_player);
            }
            AOName::ExchangeChoice => {
                // Case Checked
                let next_player_id: usize = self.next_player(self.current_player_turn);
                if self.latest_coins()[next_player_id] >= 10 {
                    self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                } else {
                    self.add_moves(&mut output, next_player_id, AOName::Income, private_player);
                    self.add_moves(&mut output, next_player_id, AOName::ForeignAid, private_player);
                    self.add_moves(&mut output, next_player_id, AOName::Tax, private_player);
                    self.add_moves(&mut output, next_player_id, AOName::Exchange, private_player);
                    self.add_moves(&mut output, next_player_id, AOName::Steal, private_player);
                    self.add_moves(&mut output, next_player_id, AOName::Assassinate, private_player);
                    self.add_moves(&mut output, next_player_id, AOName::Coup, private_player);
                }
            },
            _ => {}
        }
        output
    }
}