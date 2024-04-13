use crate::models::Card;
use rand::{thread_rng, Rng};
use rand::prelude::SliceRandom;
#[derive(Clone)]


pub struct GameState {
    turn_no: usize,
    court_deck: String,
    court_status: String,
    court_cards: Vec<Vec<Card>>,
    court_sizes: [usize; 7],
    influence: [u8; 6],
    coins: [u8; 6],
}

// Make like converter

impl GameState {
    pub fn default()-> Self{
        GameState{
            turn_no: 17,
            court_deck: "AAABBBCCCDDDEEE".to_string(),
            court_status: "111111111111111".to_string(),
            court_cards: Vec::new(),
            court_sizes: [2, 2, 2, 2, 2, 2, 3],
            influence: [2, 2, 2, 2, 2, 2],
            coins: [2, 2, 2, 2, 2, 2],
        }
    }
    pub fn distribute_cards(&mut self){
        self.deck_to_cards(true);
    }
    pub fn logcc(&self){
        log::info!("{}", format!("Court Card: {:?}", self.court_cards));
        log::info!("{}", format!("Influence: {:?}", self.influence));
        log::info!("{}", format!("Coins    : {:?}", self.coins));
        log::info!("{}", format!("Deck     : {:?}", self.court_deck));
        log::info!("{}", format!("Status   : {:?}", self.court_status));
    }
    pub fn reset(&mut self, bool_shuffle: bool) {
        self.court_deck = "AAABBBCCCDDDEEE".to_string();
        self.court_status = "111111111111111".to_string();
        self.influence = [2, 2, 2, 2, 2, 2];
        self.coins = [2, 2, 2, 2, 2, 2];
        self.deck_to_cards(bool_shuffle);
    }
    // ===== Getters, Setters, and Modifiers===== 
    pub fn get_turn_no(&self) -> usize {
        self.turn_no
    }
    pub fn inc_turn_no(&mut self) {
        self.turn_no += 1;
    }
    pub fn get_deck(&self) -> String {
        self.court_deck.clone()
    }
    pub fn get_cards(&self) -> Vec<Vec<Card>> {
        self.court_cards.clone()
    }
    pub fn get_coins(&self, player_id: usize) -> u8 {
        self.coins[player_id]
    }
    pub fn set_coins(&mut self, player_id: usize, coins: u8){
        self.coins[player_id] = coins;
    }
    pub fn add_coins(&mut self, player_id: usize, coins: u8){
        self.coins[player_id] += coins;
    }
    pub fn sub_coins(&mut self, player_id: usize, coins: u8){
        self.coins[player_id] -= coins;
    }
    pub fn get_influence(&self, player_id: usize) -> u8 {
        self.influence[player_id]
    }
    pub fn set_influence(&mut self, player_id: usize, influence: u8){
        self.influence[player_id] = influence;
    }
    pub fn add_influence(&mut self, player_id: usize, influence: u8){
        self.influence[player_id] += influence;
    }
    pub fn sub_influence(&mut self, player_id: usize, influence: u8){
        self.influence[player_id] -= influence;
    }
    pub fn get_player_hand(&self, player_id: usize) -> &Vec<Card> {
        assert!(player_id < 6, "Player_Id must be less than 6");
        &self.court_cards[player_id]
    }
    // ===== END Getters, Setters, and Modifiers ===== 
    // ===== Checkers =====
    pub fn player_has_card(&self, player_id: usize, card: &Card) -> bool {
        self.court_cards.get(player_id).map_or(false, |cards| cards.contains(&card))
    }
    pub fn is_alive(&self, player_id: usize) -> bool{
        let hp: u8 = self.influence[player_id];
        if hp > 2 {
            panic!("Influence is impossible Value")
        } else if hp > 0 {
            true
        } else {
            false
        }
    }
    // ===== END Checkers =====
    pub fn deck_to_cards(&mut self, bool_shuffle: bool){
        // sets self.court_cards based on court_deck
        // Only shuffles is all status is one
        // This shuffle is the start of the game card giving out

        let mut cards: Vec<Card> = self.court_deck.chars().map(|c| self.char_to_card(c)).collect();
        let status_vec: Vec<char> = self.court_status.chars().collect();
    
        // Check if all statuses are '1'
        let all_active = status_vec.iter().all(|&status| status == '1');
    
        // Shuffle only if bool_shuffle is true and all statuses are '1'
        if bool_shuffle && all_active {
            cards.shuffle(&mut thread_rng());
            // Since all statuses are '1', no need to shuffle statuses
            // Rebuild court_deck from shuffled cards
            self.court_deck = cards.iter().map(|card| self.card_to_char(card)).collect();
            self.sort_deck();
        }
    
        // Reset court_cards
        self.court_cards.clear();
        
        let mut start = 0;
        for &size in self.court_sizes.iter() {
            let end = start + size;
            if end > cards.len() {
                break;
            }
    
            // Filter cards for each group based on their status being '1'
            let current_group: Vec<Card> = cards[start..end]
                .iter()
                .enumerate()
                .filter(|(i, _)| status_vec[start + i] == '1')
                .map(|(_, &card)| card)
                .collect();
            
            self.court_cards.push(current_group);
            start = end;
        }

        // Log the cards distributed
    }
    fn sort_deck(&mut self) {
        let mut deck: Vec<char> = self.court_deck.chars().collect();
        let mut status: Vec<char> = self.court_status.chars().collect();

        let mut start: usize = 0;
        for &size in &self.court_sizes {
            let end = start + size;
            if end > self.court_deck.len() || end > self.court_status.len(){
                break;
            }
            let mut combined: Vec<(char, char)> = deck[start..end]
            .iter()
            .zip(status[start..end].iter())
            .map(|(&c, &s)| (c, s))
            .collect();
            combined.sort_unstable_by_key(|&(c, _)| c);
            let (sorted_deck, sorted_status): (Vec<_>, Vec<_>) = combined.iter().cloned().unzip();

            deck.splice(start..end, sorted_deck.iter().cloned());
            status.splice(start..end, sorted_status.iter().cloned());
            start = end;
        }
        self.court_deck = deck.into_iter().collect();
        self.court_status = status.into_iter().collect();
    }
    pub fn change_deck(&mut self, deck: &str, status: &str){
        self.court_deck = deck.to_string();
        self.court_status = status.to_string();
    }
    pub fn card_to_char(&self, card: &Card) -> char {
        // Example implementation, adjust according to your Card enum and logic
        match card {
            Card::Ambassador => 'A',
            Card::Assassin => 'B',
            Card::Captain => 'C',
            Card::Duke => 'D',
            Card::Contessa => 'E',
        }
    }
    pub fn char_to_card(&self, card_char: char) -> Card {
        match card_char {
            'A' => Card::Ambassador,
            'B' => Card::Assassin,
            'C' => Card::Captain,
            'D' => Card::Duke,
            'E' => Card::Contessa,
            // Add cases for all Card variants as strings
            _ => panic!("Unknown card string: {}", card_char), // Or handle the error as appropriate for your application
        }
    }

    pub fn player_discard(&mut self, player_id: usize, card: &Card) -> bool{
        // Check for the card input should be done elsewhere
        log::trace!("In player_discard");
        if let Some(index) = self.court_cards[player_id].iter().position(|x| *x == *card){
            log::trace!("In player_discard IF");
            
            self.court_cards[player_id].swap_remove(index);
            let start_index: usize = self.court_sizes[..player_id].iter().sum::<usize>();
            let end_index = start_index + self.court_sizes[player_id];
            let card_char: char = self.card_to_char(card);
            
            
            let mut chars: Vec<char> = self.court_status.chars().collect();
            
            
            log::trace!("In player_discard beFOR");
            for i in start_index..end_index {
                log::trace!("In for {}", format!("{}", i));
                log::trace!("In for {}", format!("{:?}", self.court_deck));
                if self.court_deck.chars().nth(i).unwrap_or_default() == card_char && chars[i] == '1'{
                    chars[i] = '0';
                    self.court_status = chars.into_iter().collect();
                    self.influence[player_id] -= 1;
                    if self.influence[player_id] == 0 as u8 {
                        self.coins[player_id] = 0;
                        log::info!("{}", format!("Player {} has lost all their influence and are EXILED!", player_id));
                    }
                    log::info!("{}",format!("Player {}'s Ending cards: ", player_id));
                    self.logcc();
                    return true;
                }
            }
            // Did not change anything
            return false;
        } else {
            //  Card not found
            return false;
        }
    }

    fn card_live_index(&self, player_id: usize, card: &Card) -> Option<usize> {
        // returns index of a card a player has that is alive
        let start: usize = self.court_sizes[0..player_id].iter().sum();
        let end: usize = start + self.court_sizes[player_id];
        let char_card: char = self.card_to_char(card);
        let char_vec: Vec<char> = self.court_deck.chars().collect();
        let status_vec: Vec<char> = self.court_status.chars().collect();
        
        for i in start..end {
            if char_vec[i] == char_card && status_vec[i] == '1'{
                return Some(i);
            }
        }
        None
    }
    fn swap_deck(&mut self, index_0: usize, index_1: usize, bool_sort: bool){
        // General swap, including life status
        assert!(index_0 < self.court_sizes.iter().sum(), "index_0 should be less than 15");
        assert!(index_1 < self.court_sizes.iter().sum(), "index_1 should be less than 15");
        let mut deck: Vec<char> = self.court_deck.chars().collect();
        let mut status: Vec<char> = self.court_status.chars().collect();
        deck.swap(index_0, index_1);
        status.swap(index_0, index_1);
        
        self.court_deck = deck.into_iter().collect();
        self.court_status = status.into_iter().collect();
        if bool_sort{
            self.sort_deck();
        }
    }
    pub fn swap_cards(&mut self, player_id0: usize, player_id1: usize, card0: &Card, card1: &Card) {
        // Swaps cards of the players use for the random shuffle if card revealed
        // Modifies both court_deck and court_status and court_cards
        assert!(player_id0 != player_id1, "player_id should be different");
        assert!(self.court_cards[player_id0].len() > 0, "Player_id0 should be alive");
        assert!(self.court_cards[player_id1].len() > 0, "Player_id1 should be alive");
        assert!(self.court_cards[player_id0].contains(card0), "Player_id0 should have card0");
        assert!(self.court_cards[player_id1].contains(card1), "Player_id0 should have card1");
        if let Some(card0_index) = self.court_cards[player_id0].iter().position(|c| c == card0) {
            if let Some(card1_index) = self.court_cards[player_id1].iter().position(|c| c == card1) {
                let p0_card: Card = self.court_cards[player_id0].remove(card0_index);
                let p1_card: Card = self.court_cards[player_id1].remove(card1_index);
                self.court_cards[player_id0].push(p1_card);
                self.court_cards[player_id1].push(p0_card);
            } else {
                panic!("card0_index not found");
            }
        } else {
            panic!("card1_index not found");
        }
        if let Some(index0) = self.card_live_index(player_id0, card0){
            if let Some(index1) = self.card_live_index(player_id1, card1){
                self.swap_deck(index0, index1, true);
            } else {
                panic!("Card1 not found by card_live_index");
            }
        } else {
            panic!("Card0 not found by card_live_index");
        }
    }
    pub fn swap_deck_rnd(&mut self, player_id: usize, card: &Card){
        // randomly swaps card with a card in the pile
        // player's card is not included in the pile before redraw
        
        let deck_id: usize = 6;
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..=2);
        let deck_card = &self.court_cards[deck_id][random_index].clone();

        self.swap_cards(player_id, deck_id, card, deck_card);
    }
    
    pub fn swap_pooled(&mut self, player_id: usize, chosen_cards: &Vec<Card>){
        // this is the ambassador swap
        let mut chosen_chars: Vec<char> = chosen_cards.iter().map(|card| self.card_to_char(card)).collect();
        let mut deck_char: Vec<char> = self.court_deck.chars().collect();
        let mut status_char: Vec<char> = self.court_status.chars().collect();
        let player_start: usize = self.court_sizes[0..player_id].iter().sum();
        let player_end: usize = player_start + self.court_sizes[player_id];
        let pile_start: usize = self.court_sizes[0..6].iter().sum();
        let pile_end: usize = pile_start + self.court_sizes[6];

        while !chosen_chars.is_empty(){
            for i in 0..self.court_sizes[player_id]{
                // You never need to swap if the card is like dead
                // You only ever might swap if the card is alive
                if chosen_chars.is_empty(){
                    break;
                }
                if status_char[player_start + i] != '0'{
                    for j in 0..self.court_sizes[6]{
                        // If need to swap, swap
                        if deck_char[player_start + i] == chosen_chars[0]{
                            // Do not need to swap if you already have the card
                            chosen_chars.remove(0);
                            //next i
                            break;
                        } else if deck_char[pile_start + j] == chosen_chars[0]{
                            deck_char.swap(player_start + i, pile_start + j);
                            status_char.swap(player_start + i, pile_start + j);
                            chosen_chars.remove(0);
                            //next i
                            break;
                        }
                    }
                }
            }
        }
        self.court_deck = deck_char.into_iter().collect();
        self.court_status = status_char.into_iter().collect();
        self.sort_deck();
        self.deck_to_cards(false);
    }
}