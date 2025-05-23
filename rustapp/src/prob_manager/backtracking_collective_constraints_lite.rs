use crate::history_public::{AOName, ActionObservation, Card};
use super::{backtracking_prob::{CoupConstraint, CoupConstraintAnalysis}, collective_constraint::CompressedCollectiveConstraint, compressed_group_constraint::CompressedGroupConstraint};
use super::backtracking_collective_constraints::{BacktrackMetaData, ActionInfo, ActionInfoName};
use ahash::AHashSet;
use crossbeam::channel::after;
use std::{marker::Copy, path::Path};

// TODO: REFACTOR ActionInfo and ActionInfoName to BacktrackManager or its own file
#[derive(Clone, Debug)]
pub struct SignificantAction {
    move_no: usize, // move_no is seperate from the game move number
    player: u8,
    action_info: ActionInfo,
    meta_data: BacktrackMetaData,
}

impl SignificantAction {
    pub fn initial(move_no: usize, player: u8, action_info: ActionInfo) -> Self {
        let pd_metadata = BacktrackMetaData::initial();
        Self {
            move_no,
            player,
            action_info,
            meta_data: pd_metadata,
        }
    }
    pub fn start() -> Self {
        let meta_data: BacktrackMetaData = BacktrackMetaData::start();
        Self {
            move_no: 0,
            player: 77,
            action_info: ActionInfo::Start,
            meta_data,
        }
    }
    pub fn start_inferred() -> Self {
        let meta_data: BacktrackMetaData = BacktrackMetaData::start();
        Self {
            move_no: 0,
            player: 77,
            action_info: ActionInfo::StartInferred,
            meta_data,
        }
    }
    pub fn name(&self) -> ActionInfoName {
        self.action_info.name()
    }
    pub fn move_no(&self) -> usize {
        self.move_no
    }
    pub fn player(&self) -> u8 {
        self.player
    }
    pub fn action_info(&self) -> &ActionInfo {
        &self.action_info
    }
    pub fn action_info_mut(&mut self) -> &mut ActionInfo {
        &mut self.action_info
    }
    pub fn meta_data(&self) -> &BacktrackMetaData {
        &self.meta_data
    }
    pub fn public_constraints(&self) -> &Vec<Vec<Card>> {
        self.meta_data.public_constraints()
    }
    pub fn inferred_constraints(&self) -> &Vec<Vec<Card>> {
        self.meta_data.inferred_constraints()
    }
    pub fn set_inferred_constraints(&mut self, inferred_constraints: &Vec<Vec<Card>>) {
        self.meta_data.set_inferred_constraints(inferred_constraints)
    }
    pub fn impossible_constraints(&self) -> &[[bool; 5]; 7] {
        self.meta_data.impossible_constraints()
    }
    pub fn impossible_constraints_2(&self) -> &[[[bool; 5]; 5]; 7] {
        self.meta_data.impossible_constraints_2()
    }
    pub fn impossible_constraints_3(&self) -> &[[[bool; 5]; 5]; 5] {
        self.meta_data.impossible_constraints_3()
    }
    pub fn set_impossible_constraints(&mut self, impossible_constraints: &[[bool; 5]; 7]) {
        self.meta_data.set_impossible_constraints(impossible_constraints);
    }
    pub fn add_inferred_constraints(&mut self, player_id: usize, card: Card)  {
        self.meta_data.inferred_constraints[player_id].push(card);
        debug_assert!(player_id < 6 
            && self.meta_data.inferred_constraints[player_id].len() < 3 
            || player_id == 6 
            && self.meta_data.inferred_constraints[player_id].len() < 4, 
            "bad push");
    }
    pub fn check_add_inferred_constraints(&mut self, player_id: usize, card: Card) -> bool {
        if !self.meta_data.inferred_constraints[player_id].contains(&card) {
            self.meta_data.inferred_constraints[player_id].push(card);
            debug_assert!(player_id < 6 
                && self.meta_data.inferred_constraints[player_id].len() < 3 
                || player_id == 6 
                && self.meta_data.inferred_constraints[player_id].len() < 4, 
                "bad push");
            return true;
        }
        false
    }
    pub fn player_cards_known<T>(&self, player_id: T) -> usize 
    where
        T: Into<usize> + Copy,
    {
        self.meta_data.player_cards_known(player_id)
    }
    pub fn player_has_public_constraint<T>(&self, player_id: T, card: Card) -> bool 
    where
        T: Into<usize> + Copy,
    {   
        self.meta_data.player_has_public_constraint(player_id, card)
    }
    pub fn player_has_inferred_constraint<T>(&self, player_id: T, card: Card) -> bool 
    where
        T: Into<usize> + Copy,
        {   
            self.meta_data.player_has_inferred_constraint(player_id, card)
        }
    pub fn player_constraints_all_full<T>(&self, player_id: T, card: Card) -> bool 
    where
        T: Into<usize> + Copy,
    {
        self.meta_data.player_constraints_all_full(player_id, card)
    }
    pub fn known_card_count(&self, card: Card) -> u8 {
        self.meta_data.inferred_constraints().iter().map(|v| v.iter().filter(|c| **c == card).count() as u8).sum::<u8>()
        + self.meta_data.public_constraints().iter().map(|v| v.iter().filter(|c| **c == card).count() as u8).sum::<u8>()
    }
    pub fn action_info_str(&self) -> String {
        format!("Player: {} {:?} public_constraints: {:?}, inferred_constraints: {:?}, impossible_constraints: {:?}", self.player, self.action_info, self.public_constraints(), self.inferred_constraints(), self.impossible_constraints())
    }
}   

// 1: Add recursion on finding inferred constraint
//      - Can possibly store a boolean that determines if any empty redraw is before a number, so no need to lookback for that
// 2: Optimize to consider where we might not need to recurse (non recursive method can get 1/250 games wrong)
//      - Consider memoizing by storing a card, that a revealredraw might need to redraw
//          - during forward pass, if its impossible for player to have that card, then redraw it
#[derive(Clone, Debug)]
/// A struct that helps in card counting. Stores all information known about cards by a particular player.
/// Lighter version to provide only functionality for latest move inference
///     - Is memoized and not lazy evaluated
///     - Does not regenerate path so inference of past states is impossible
pub struct BackTrackCollectiveConstraintLite {
    public_constraints: Vec<Vec<Card>>, // Stores all the dead cards of dead players, None are all behind
    inferred_constraints: Vec<Vec<Card>>, // Stores all the inferred cards of alive players 
    impossible_constraints: [[bool; 5]; 7], // For each player store an array of bool where each index is a Card, this represents whether a player cannot have a card true => cannot
    impossible_constraints_2: [[[bool; 5]; 5]; 7], // [Card Smaller][Card Bigger]
    impossible_constraints_3: [[[bool; 5]; 5]; 5], // [Card small] [Card med][Card big]
    move_no: usize, // turn number (0 being the start) (post increment this so assign then increment)
    history: Vec<SignificantAction>, // Stores 
}

impl BackTrackCollectiveConstraintLite {
    /// TODO: change gamestart for different inferred starting hands
    /// Recreates start state based on Start ActionInfo which may include inferred cards
    fn regenerate_game_start(&mut self) {
        self.public_constraints.iter_mut().for_each(|v| v.clear());
        self.inferred_constraints = self.history.first().unwrap().inferred_constraints().clone();
        self.impossible_constraints = [[false; 5]; 7];
        self.impossible_constraints_2 = [[[false; 5]; 5]; 7]; 
        self.impossible_constraints_3 = [[[false; 5]; 5]; 5];
        // TODO: Make this nicer
        // !! Not gonna reset move_no
        // Not adding inferred information as sometimes a discard could try to insert
        // info that has been inferred from add_inferred_information
        // TODO: TEST or I could just not save the meta_data but let it infer anyways
        // self.add_inferred_information();
        log::trace!("regenerate_game_start");
        self.printlog();
    }
    pub fn to_meta_data(&mut self) -> BacktrackMetaData {
        BacktrackMetaData { 
            public_constraints: self.public_constraints.clone(), 
            inferred_constraints: self.inferred_constraints.clone(), 
            impossible_constraints: self.impossible_constraints.clone(),
            impossible_constraints_2: self.impossible_constraints_2.clone(),
            impossible_constraints_3: self.impossible_constraints_3.clone(),
        }
    }
    /// Calculate for the latest addition
    fn calculate_stored_move_initial(&mut self) {
        // let action: &SignificantAction = &self.history[history_index];
        let history_index = self.history.len() - 1;
        let (player_id, action_info) = {
            let action = &self.history[history_index];
            (action.player() as usize, action.action_info().clone())
        };
        log::trace!("calculate_stored_move_initial: {:?}", action_info);
        match action_info {
            ActionInfo::Start => {
                // self.regenerate_game_start();
                debug_assert!(false, "You should not be here!");
            },
            ActionInfo::StartInferred => {
                // self.regenerate_game_start();
                debug_assert!(false, "You should not be here!");
            },
            ActionInfo::Discard{ discard} => {
                self.death(player_id, discard);
            },
            ActionInfo::RevealRedraw{ .. } => {},
            ActionInfo::ExchangeDrawChoice{ .. } => {},
            ActionInfo::ExchangeDraw { draw } => {
                if draw.is_empty() {
                    // early exit on public info ExchangeDraw
                    self.history[history_index].meta_data = self.to_meta_data();
                    log::info!("recalculated_stored_move_initial: {} {:?}", history_index, self.history[history_index].action_info());
                    return;
                }
            },
            ActionInfo::ExchangeChoice { .. } => {},
        }
        self.generate_impossible_constraints(self.history.len() - 1);
        self.generate_inferred_constraints();
        self.history[history_index].meta_data = self.to_meta_data();
        log::info!("recalculated_stored_move_initial: {} {:?}", history_index, self.history[history_index].action_info());
        self.printlog();
    }   
    // Add other normal methods for inference
}

impl BackTrackCollectiveConstraintLite {
    // pub fn sorted_public_constraints(&self) -> Vec<Vec<Card>> {
    //     let mut output = self.public_constraints.clone();
    //     for card_vec in output.iter_mut() {
    //         card_vec.sort_unstable()
    //     }
    //     output
    // }
    // pub fn sorted_inferred_constraints(&self) -> Vec<Vec<Card>> {
    //     let mut output = self.inferred_constraints.clone();
    //     output.iter_mut().for_each(|v| v.sort_unstable());
    //     output
    // }
    #[inline]
    pub fn player_is_alive(&self, player_id: usize) -> bool {
        self.public_constraints[player_id].len() < 2
    }
    pub fn player_constraints_all_full<T>(&self, player_id: T, card: Card) -> bool 
    where
        T: Into<usize> + Copy,
    {   
        self.player_cards_known(player_id.into()) == 2 &&
        self.inferred_constraints[player_id.into()].iter().all(|&c| c == card) &&
        self.public_constraints[player_id.into()].iter().all(|&c| c == card)
    }
    /// Gets the inferred constraint counts of each card for a player
    pub fn get_inferred_card_counts(&self, player_id: usize) -> [u8; 5] {
        debug_assert!(player_id < 7, "Player ID to big! player_id: {}", player_id);
        let mut counts: [u8; 5] = [0; 5];
        for card in self.inferred_constraints[player_id].iter() {
            counts[*card as usize] += 1;
        }
        counts
    }
    /// Gets the public constraint catd count of each card for a player
    pub fn get_public_card_counts(&self, player_id: usize) -> [u8; 5] {
        debug_assert!(player_id < 7, "Player ID to big! player_id: {}", player_id);
        let mut counts: [u8; 5] = [0; 5];
        if player_id < 6 {
            for card in self.public_constraints[player_id].iter() {
                counts[*card as usize] += 1;
            }
        }
        counts
    }
    /// Gets the number of dead cards a player has for a particular card
    /// NOTE:
    /// - Not actually used except for debugging
    pub fn player_dead_card_count(&self, player_id: usize, card: Card) -> u8 {
        let mut output: u8 = 0;
        // [COMBINE SJ]
        for dead_card in self.public_constraints[player_id].iter() {
            if *dead_card == card {
                output += 1;
            } 
        }
        output
    }
    /// Gets the number of dead cards a player has for a particular card
    /// NOTE:
    /// - Not actually used except for debugging
    pub fn player_lives(&self, player_id: usize) -> u8 {
        // [COMBINE SJ]
        2 - self.public_constraints[player_id].len() as u8   
    }
    /// Gets the number of dead cards a player has for a each card
    /// NOTE:
    /// - Not actually used except for debugging
    pub fn player_dead_card_counts(&self, player_id: usize) -> [u8; 5] {
        let mut output: [u8; 5] = [0; 5];
        // [COMBINE SJ]
        for card in self.public_constraints[player_id].iter() {
            output[*card as usize] += 1;
        }
        output
    }
    /// Gets the number of known alive cards a player has for a particular card
    /// NOTE:
    /// - Not actually used except for debugging
    pub fn player_inferred_card_count(&self, player_id: usize, card: Card) -> u8 {
        let mut output: u8 = 0;
        // [COMBINE SJ]
        for item in self.inferred_constraints[player_id].iter() {
            if *item == card {
                output += 1;
            }
        }
        output
    }
    /// Gets array of counts of known alive cards a player has for each card
    /// NOTE:
    /// - Not actually used except for debugging
    pub fn player_inferred_card_counts(&self, player_id: usize) -> [u8; 5] {
        let mut output: [u8; 5] = [0; 5];
        // [COMBINE SJ]
        for card in self.inferred_constraints[player_id].iter() {
            output[*card as usize] += 1;
        }
        output
    }
    pub fn dead_card_count(&self) -> [u8; 5] {
        let mut dead_card_counts: [u8; 5] = [0; 5];
        for dead_cards in self.public_constraints.iter() {
            for card in dead_cards.iter() {
                dead_card_counts[*card as usize] += 1;
            }
        }
        dead_card_counts
    }
    // TODO: make in to generic
    /// Returns number of a player's cards are known, i.e. Dead or inferred
    pub fn player_cards_known(&self, player_id: usize) -> u8 {
        (self.public_constraints[player_id].len() + self.inferred_constraints[player_id].len()) as u8
    }
    /// Adds to tracked inferred constraints
    pub fn add_inferred_player_constraint(&mut self, player_id: usize, card: Card) {
        debug_assert!(player_id < 6, "Use proper player_id thats not pile");
        debug_assert!(self.inferred_constraints[player_id].len() < 2, "Adding inferred knowledge to fully known player!");
        // PD Obsolete
        // self.inferred_card_count[card as usize] += 1;
        self.inferred_constraints[player_id].push(card);
    }
    #[inline]
    /// Return true if inferred player constraint contains a particular card 
    pub fn inferred_player_constraint_contains(&self, player_id: usize, card: Card) -> bool {
        // [COMBINE SJ] rewrite
        self.inferred_constraints[player_id].contains(&card)
    }
    /// Used for when move is added to history and not for recalculation
    pub fn death_initial(&mut self, player_id: usize, card: Card) {
        self.add_dead_card(player_id, card);
        // Check before recursing
        // self.update_past_move_hidden_info();
    }
    /// Used for recalculation
    pub fn death(&mut self, player_id: usize, card: Card) {
        self.add_dead_card(player_id, card);
    }
    pub fn add_dead_card(&mut self, player_id: usize, card: Card) {
        self.public_constraints[player_id].push(card);
        
        // If the dead card was already inferred, remove it
        if let Some(pos) = self.inferred_constraints[player_id].iter().position(|&c| c == card) {
            self.inferred_constraints[player_id].swap_remove(pos);
        }
    }   
    /// Function to call for move RevealRedraw
    pub fn reveal_redraw(&mut self, history_index: usize, player_id: usize, card: Card) {
        // TODO: Custom impossible swaps and generation
    }
    /// Used for when move is added to history and not for recalculation
    pub fn reveal_redraw_initial(&mut self, player_id: usize, card: Card) {
        // Consider moving the impossible states around?
        // self.update_past_move_hidden_info();
        // TODO: Custom impossible swaps and generation
    }
    /// Function to call when the card revealed and the redrawn card is the same card
    pub fn reveal_redraw_same(&mut self, player_id: usize, card: Card) {
        if !self.inferred_constraints[player_id].contains(&card) {
            log::trace!("reveal_redraw_same player_id: {}, card: {:?}", player_id, card);
            self.inferred_constraints[player_id].push(card);
        }
        // TODO: Custom impossible swaps and generation
    }
    /// Function to call when both the card revealed and redrawn are known and not the same
    pub fn reveal_redraw_diff(&mut self, player_id: usize, reveal: Card, redraw: Card) {
        if let Some(pos) = self.inferred_constraints[player_id].iter().position(|c| *c == reveal) {
            self.inferred_constraints[player_id].swap_remove(pos);
        }
        if let Some(pos) = self.inferred_constraints[6].iter().position(|c| *c == redraw) {
            self.inferred_constraints[6].swap_remove(pos);
        }
        self.inferred_constraints[player_id].push(redraw);
        self.inferred_constraints[6].push(reveal);
        // TODO: Custom impossible swaps and generation
    }
    /// Function to call when both the card revealed is left in the pile
    /// Assumes redraw is None
    /// Assumes what is revealed is relinquish and passed to the pile
    pub fn reveal_redraw_relinquish(&mut self, player_id: usize, relinquish: Card) {
        if let Some(pos) = self.inferred_constraints[player_id].iter().position(|c| *c == relinquish) {
            self.inferred_constraints[player_id].swap_remove(pos);
        }
        self.inferred_constraints[6].push(relinquish);
    }
    /// Function to call for move Ambassador, without considering private information seen by the player who used Ambassador
    pub fn ambassador_public(&mut self, player_id: usize) {

    }
    /// Function to call for move Ambassador, when considering private information seen by the player who used Ambassador
    pub fn ambassador_private(&mut self, player_id: usize) {
        todo!()
    }
    /// Does Backtracking to determine if at a particular point that particular player could not have had some set of cards at start of turn
    /// Assuming we won't be using this for ambassador?
    pub fn impossible_to_have_cards_general(&self, index_lookback: usize, player_of_interest: usize, cards: &[u8; 5]) -> bool {
        log::trace!("impossible_to_have_cards player_of_interest: {}, cards: {:?}", player_of_interest, cards);
        debug_assert!(player_of_interest != 6 && cards.iter().sum::<u8>() <= 2 || player_of_interest == 6 && cards.iter().sum::<u8>() <= 3, "cards too long!");
        let mut public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(3), Vec::with_capacity(3), Vec::with_capacity(3), Vec::with_capacity(3), Vec::with_capacity(3), Vec::with_capacity(3), Vec::with_capacity(4), ];
        let mut inferred_constraints: Vec<Vec<Card>> = public_constraints.clone();
        let latest_move = self.history.last().unwrap(); 
        // match latest_move.action_info() {
        //     ActionInfo::Discard { discard } => {
        //         inferred_constraints[latest_move.player() as usize].push(*discard);
        //     },
        //     ActionInfo::RevealRedraw { reveal, redraw, .. } => {
        //         inferred_constraints[latest_move.player() as usize].push(*reveal);
        //         redraw.map(|pile_original_card| inferred_constraints[6].push(pile_original_card));
        //     },
        //     ActionInfo::ExchangeDrawChoice { draw, relinquish } => {
        //         inferred_constraints[latest_move.player() as usize].extend(relinquish.iter().copied());
        //         inferred_constraints[6].extend(draw.iter().copied());
        //     },
        //     ActionInfo::Start
        //     | ActionInfo::StartInferred => {},
        // }
        for (card_num, card_count) in cards.iter().enumerate() {
            for _ in 0..*card_count {
                inferred_constraints[player_of_interest].push(Card::try_from(card_num as u8).unwrap());
            }
        }
        // Do an unwrap_or false
        !self.possible_to_have_cards_recurse(index_lookback, &mut public_constraints, &mut inferred_constraints, &[0; 5])
        // !self.possible_to_have_cards_recurse(index_lookback - 1, index, player_of_interest, &mut public_constraints, &mut inferred_constraints, cards)
    }
    /// returns false if possible
    /// TODO: Consider passing in array to count inferred_so_far
    /// Traces the game tree in reverse (from latest move to earliest move) by backtracking
    /// Tracks possible paths known cards could have come from in the past
    /// If a state is found to satisfy cards at the index_of_interest return Some(true)
    /// If no state is every found return Some(false) or None
    /// Assume cards should be sorted before use
    pub fn possible_to_have_cards_recurse(&self, index_loop: usize, public_constraints: &mut Vec<Vec<Card>>, inferred_constraints: &mut Vec<Vec<Card>>, cards: &[u8; 5]) -> bool {
        // Will temporarily not use memo and generate all group_constraints from start
        // Needed for checks
        log::trace!("After");
        log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
        log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.history[index_loop].public_constraints(), inferred_constraints);
        if !self.is_valid_combination(index_loop, inferred_constraints) {
            // early exit before terminal node
            log::trace!("is_valid_combination evaluated to false");
            return false
        }
        log::trace!("is_valid_combination evaluated to true");
        let player_loop = self.history[index_loop].player() as usize;
        let mut response = false;
        match self.history[index_loop].action_info() {
            ActionInfo::Discard { discard } => {
                log::trace!("Before Discard");
                log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.history[index_loop].public_constraints(), inferred_constraints);
                let mut removed_discard = false;
                if let Some(pos) = public_constraints[player_loop].iter().rposition(|c| *c == *discard) {
                    public_constraints.swap_remove(pos);
                    removed_discard = true;
                }
                inferred_constraints[player_loop].push(*discard);
                // recurse
                response = self.possible_to_have_cards_recurse(index_loop - 1, public_constraints, inferred_constraints, cards);
                if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *discard) {
                    inferred_constraints[player_loop].swap_remove(pos);
                }
                if removed_discard {
                    public_constraints[player_loop].push(*discard);
                }
                return response;
            },
            ActionInfo::RevealRedraw { reveal, redraw, relinquish } => {
                // Check if will burst before pushing
                match redraw {
                    Some(redraw_i) => {
                        log::trace!("Before Reveal Redraw");
                        log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                        log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.history[index_loop].public_constraints(), inferred_constraints);
                
                        let (mut removed_redraw, mut removed_reveal) = (false, false);
                        if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *redraw_i) {
                            inferred_constraints[player_loop].swap_remove(pos);
                            removed_redraw = true;
                        } 
                        inferred_constraints[6].push(*redraw_i);
                        if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *reveal) {
                            inferred_constraints[6].swap_remove(pos);
                            removed_reveal = true;
                        } 
                        inferred_constraints[player_loop].push(*reveal);
                        response = self.possible_to_have_cards_recurse(index_loop - 1, public_constraints, inferred_constraints, cards);
                        if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *reveal) {
                            inferred_constraints[player_loop].swap_remove(pos);
                        }
                        if removed_reveal {
                            inferred_constraints[6].push(*reveal);
                        }
                        if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *redraw_i) {
                            inferred_constraints[6].swap_remove(pos);
                        }
                        if removed_redraw {
                            inferred_constraints[player_loop].push(*redraw_i);
                        }
                    },
                    None => {
                        match relinquish {
                            Some(relinquish_i) => {
                                // swap cards around sir
                                // relinquish_i == *reveal always
                                // Case 0: player redrew card != reveal
                                // Case 1: player redrew card == reveal (reveal from pile)
                                if inferred_constraints[6].len() == 3
                                && !inferred_constraints[6].contains(&reveal) {
                                    // This state cannot be arrive after the reveal_redraw
                                    return false;
                                }
                                log::trace!("Before Reveal Relinquish");
                                log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.history[index_loop].public_constraints(), inferred_constraints);
                
                                if inferred_constraints[player_loop].is_empty() {
                                    log::trace!("inferred_constraints[player_loop].is_empty(): {:?}", inferred_constraints[player_loop]);
                                    // let mut bool_move_from_pile_to_player = false;
                                    let mut removed_reveal = false;
                                    if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *reveal) {
                                        inferred_constraints[6].swap_remove(pos);
                                        removed_reveal = true;
                                    }
                                    inferred_constraints[player_loop].push(*reveal);
                                    if inferred_constraints[player_loop].len() < 3
                                    && inferred_constraints.iter().map(|v| v.iter().filter(|c| **c == *reveal).count() as u8).sum::<u8>() < 4{
                                        response = self.possible_to_have_cards_recurse(index_loop - 1, public_constraints, inferred_constraints, cards);
                                    }
                                    if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *reveal) {
                                        inferred_constraints[player_loop].swap_remove(pos);
                                    }
                                    if removed_reveal {
                                        inferred_constraints[6].push(*reveal);
                                    }
                                    return response;
                                }
                                let mut iter_cards = inferred_constraints[player_loop].clone();
                                iter_cards.sort_unstable();
                                iter_cards.dedup();
                                for (i, card_player) in iter_cards.iter().enumerate() {
                                    // Card Source was not from Pile
                                    log::trace!("Before Reveal Relinquish B");
                                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                    log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.history[index_loop].public_constraints(), inferred_constraints);
                
                                    if inferred_constraints[player_loop].len() < 2 {
                                        let mut bool_move_from_pile_to_player = false;
                                        if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *reveal) {
                                            inferred_constraints[6].swap_remove(pos);
                                            bool_move_from_pile_to_player = true;
                                        }
                                        inferred_constraints[player_loop].push(*reveal);
                                        
                                        if inferred_constraints.iter().map(|v| v.iter().filter(|c| **c == *reveal).count() as u8).sum::<u8>() < 4{
                                            response = self.possible_to_have_cards_recurse(index_loop - 1, public_constraints, inferred_constraints, cards);
                                        }
                                        
                                        if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *reveal) {
                                            inferred_constraints[player_loop].swap_remove(pos);
                                        }
                                        if bool_move_from_pile_to_player {
                                            inferred_constraints[6].push(*reveal);
                                        }
                                        if response {
                                            return true;
                                        }
                                    }
                                    // Card Source was from Pile
                                    if *card_player != *reveal {
                                        log::trace!("Before Reveal Relinquish C");
                                        log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                        log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.history[index_loop].public_constraints(), inferred_constraints);
                
                                        let mut bool_move_from_pile_to_player = false;
                                        let mut bool_move_from_player_to_pile = false;
                                        if let Some(pos) = inferred_constraints[player_loop].iter().position(|c| *c == *card_player) {
                                            inferred_constraints[player_loop].swap_remove(pos);
                                            bool_move_from_player_to_pile = true;
                                        }
                                        inferred_constraints[6].push(*card_player);
                                        if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *reveal) {
                                            inferred_constraints[6].swap_remove(pos);
                                            bool_move_from_pile_to_player = true;
                                        }
                                        inferred_constraints[player_loop].push(*reveal);
                                        // TODO: Recurse here in other version
                                        response = self.possible_to_have_cards_recurse(index_loop - 1, public_constraints, inferred_constraints, cards);

                                        if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *reveal) {
                                            inferred_constraints[player_loop].swap_remove(pos);
                                        }
                                        if bool_move_from_pile_to_player {
                                            inferred_constraints[6].push(*reveal);
                                        }
                                        if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *card_player) {
                                            inferred_constraints[6].swap_remove(pos);
                                        }
                                        if bool_move_from_player_to_pile {
                                            inferred_constraints[player_loop].push(*card_player);
                                        }
                                        if response {
                                            return true;
                                        }
                                    }
                                }
                            },
                            None => {
                                if inferred_constraints[player_loop].len() + inferred_constraints[6].len() == 5 
                                && !inferred_constraints[player_loop].contains(&reveal)
                                && !inferred_constraints[6].contains(&reveal) {
                                    // This state cannot be arrive after the reveal_redraw
                                    return false;
                                }
 
                                if inferred_constraints[player_loop].is_empty() {
                                    log::trace!("Before Reveal Redraw None");
                                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                    log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.history[index_loop].public_constraints(), inferred_constraints);
                
                                    let mut bool_move_from_pile_to_player = false;
                                    if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *reveal) {
                                        inferred_constraints[6].swap_remove(pos);
                                        bool_move_from_pile_to_player = true;
                                    }
                                    inferred_constraints[player_loop].push(*reveal);

                                    if inferred_constraints[player_loop].len() < 3
                                    && inferred_constraints.iter().map(|v| v.iter().filter(|c| **c == *reveal).count() as u8).sum::<u8>() < 4{
                                        response = self.possible_to_have_cards_recurse(index_loop - 1, public_constraints, inferred_constraints, cards);
                                    }
                                    if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *reveal) {
                                        inferred_constraints[player_loop].swap_remove(pos);
                                    }
                                    if bool_move_from_pile_to_player {
                                        inferred_constraints[6].push(*reveal);
                                    }
                                    return response;
                                }
                                let mut iter_cards = inferred_constraints[player_loop].clone();
                                iter_cards.sort_unstable();
                                iter_cards.dedup();
                                // Doesnt handle empty case
                                for (_, card_player) in iter_cards.iter().enumerate() {
                                    log::trace!("Before Reveal Redraw None B");
                                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                    log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.history[index_loop].public_constraints(), inferred_constraints);
                
                                    // Card Source was not from Pile
                                    let mut bool_move_from_pile_to_player = false;
                                    if *card_player != *reveal || inferred_constraints[6].contains(&reveal) {
                                        if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *reveal) {
                                            inferred_constraints[6].swap_remove(pos);
                                            bool_move_from_pile_to_player = true;
                                        }
                                        inferred_constraints[player_loop].push(*reveal);
                            
                                        // Probably need push only if certain conditions met
                                        
                                        if inferred_constraints[player_loop].len() < 3  
                                        && inferred_constraints[6].len() < 4 
                                        && inferred_constraints.iter().map(|v| v.iter().filter(|c| **c == *reveal).count() as u8).sum::<u8>() < 4{
                                            // TODO: Recurse here in other version
                                            // variants.push(temp);
                                            response = self.possible_to_have_cards_recurse(index_loop - 1, public_constraints, inferred_constraints, cards);
                                        }

                                        if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *reveal) {
                                            inferred_constraints[player_loop].swap_remove(pos);
                                        }
                                        if bool_move_from_pile_to_player {
                                            inferred_constraints[6].push(*reveal);
                                        }
                                        if response {
                                            return true;
                                        }
                                    }
                                    log::trace!("Before Reveal Redraw None C");
                                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                    log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.history[index_loop].public_constraints(), inferred_constraints);
                
                                    // Card Source was from Pile
                                    let mut bool_move_from_pile_to_player_2 = false;
                                    let mut bool_move_from_player_to_pile = false;
                                    if let Some(pos) = inferred_constraints[player_loop].iter().position(|c| *c == *card_player) {
                                        inferred_constraints[player_loop].swap_remove(pos);
                                        bool_move_from_player_to_pile = true;
                                    }
                                    inferred_constraints[6].push(*card_player);
                                    if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *reveal) {
                                        inferred_constraints[6].swap_remove(pos);
                                        bool_move_from_pile_to_player_2 = true;
                                    }
                                    inferred_constraints[player_loop].push(*reveal);

                                    // Probably need push only if certain conditions met
                                    if inferred_constraints[player_loop].len() < 3  
                                    && inferred_constraints[6].len() < 4 
                                    && inferred_constraints.iter().map(|v| v.iter().filter(|c| **c == *reveal).count() as u8).sum::<u8>() < 4{
                                        // TODO: Recurse here in other version
                                        // variants.push(temp);
                                        response = self.possible_to_have_cards_recurse(index_loop - 1, public_constraints, inferred_constraints, cards);
                                    }

                                    if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *reveal) {
                                        inferred_constraints[player_loop].swap_remove(pos);
                                    }
                                    if bool_move_from_pile_to_player_2 {
                                        inferred_constraints[6].push(*reveal);
                                    }
                                    if let Some(pos) = inferred_constraints[6].iter().rposition(|c| c == card_player) {
                                        inferred_constraints[6].swap_remove(pos);
                                    }
                                    if bool_move_from_player_to_pile {
                                        inferred_constraints[player_loop].push(*card_player);
                                    }
                                    if response {
                                        return true
                                    }
                                }
                            },
                        }
                    },
                }
            },
            ActionInfo::ExchangeDrawChoice { .. } => {
                unimplemented!("Deprecated!");
                response = self.recurse_variants_exchange_public(index_loop, public_constraints, inferred_constraints, player_loop, cards);
            },
            ActionInfo::ExchangeDraw { draw } => {
                if !draw.is_empty() {
                    // Assumes draw is always 2 cards
                    let current_count_0 = inferred_constraints[6].iter().filter(|c| **c == draw[0]).count();
                    if draw[0] == draw[1] {
                        match current_count_0 {
                            0 => {
                                inferred_constraints[6].push(draw[0]);
                                inferred_constraints[6].push(draw[0]);
                                response = self.possible_to_have_cards_recurse(index_loop - 1, public_constraints, inferred_constraints, cards);
                                if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == draw[0]) {
                                    inferred_constraints.swap_remove(pos);
                                }
                                if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == draw[0]) {
                                    inferred_constraints.swap_remove(pos);
                                }
                                if response {
                                    return true
                                }
                            },
                            1 => {
                                inferred_constraints[6].push(draw[0]);
                                response = self.possible_to_have_cards_recurse(index_loop - 1, public_constraints, inferred_constraints, cards);
                                if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == draw[0]) {
                                    inferred_constraints.swap_remove(pos);
                                }
                                if response {
                                    return true
                                }
                            },
                            2 => {
                                response = self.possible_to_have_cards_recurse(index_loop - 1, public_constraints, inferred_constraints, cards);
                                if response {
                                    return true
                                }
                            },
                            _ => debug_assert!(false, "you should not be here!"),
                        }
                    } else {
                        let current_count_1 = inferred_constraints[6].iter().filter(|c| **c == draw[0]).count();
                        if current_count_0 < 1 {
                            inferred_constraints[6].push(draw[0]);
                        }
                        if current_count_1 < 1 {
                            inferred_constraints[6].push(draw[1]);
                        }
                        response = self.possible_to_have_cards_recurse(index_loop - 1, public_constraints, inferred_constraints, cards);
                        if current_count_0 < 1 {
                            if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == draw[0]) {
                                inferred_constraints.swap_remove(pos);
                            }
                        }
                        if current_count_1 < 1 {
                            if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == draw[1]) {
                                inferred_constraints.swap_remove(pos);
                            }
                        }
                        if response {
                            return true
                        }
                    }
                } else {
                    debug_assert!(false, "New API should not have reached here, but should have been skipped!");
                }
            },
            ActionInfo::ExchangeChoice { .. } => {
                if let ActionInfo::ExchangeDraw { draw } = self.history[index_loop - 1].action_info() {
                    if draw.is_empty() {
                        response = self.recurse_variants_exchange_public(index_loop, public_constraints, inferred_constraints, player_loop, cards);
                    } else {
                        // Assumes both relinquish cards are known
                        // Assumes hand cards are known (they are alive cards)
                        // Pool to choose from is hand + draw
                        todo!()
                    }
                }
            },
            ActionInfo::Start
            | ActionInfo::StartInferred => {
                // Managed to reach base
                log::trace!("possible_to_have_cards_recurse found true at index: {}", index_loop);
                response = true;
            },
        }
        response
    }
    /// Return true if hypothesised card permutations cannot be shown to be impossible
    pub fn is_valid_combination(&self, index_loop: usize , inferred_constraints: &Vec<Vec<Card>>) -> bool {
        let public_constraints = self.history[index_loop].public_constraints();
        // Check actual constraints at leaf node
        // All public_constraints inside actual
        log::trace!("is_valid_combination for: index {}, considering public_constraints: {:?}, inferred_constraints: {:?}", index_loop, public_constraints, inferred_constraints);
        for card in [Card::Ambassador, Card::Assassin, Card::Captain, Card::Duke, Card::Contessa].iter() {
            if public_constraints.iter().map(|v| v.iter().filter(|c| **c == *card).count() as u8).sum::<u8>() 
            + inferred_constraints.iter().map(|v| v.iter().filter(|c| **c == *card).count() as u8).sum::<u8>() > 3 {
                log::trace!("is_valid_combination constraints has too many {:?}", card);
                return false
            }
        }
        for player in 0..6 {
            if public_constraints[player].len() + inferred_constraints[player].len() > 2 {
                log::trace!("is_valid_combination player {} has too many cards", player);
                return false
            }
        }
        if public_constraints[6].len() + inferred_constraints[6].len() > 3 {
            log::trace!("is_valid_combination pile has too many cards");
            return false
        }
        for player in 0..7 {
            if inferred_constraints[player].len() == 1 && self.history[index_loop].impossible_constraints()[player][inferred_constraints[player][0] as usize]{
                return false
            }
            if inferred_constraints[player].len() == 2 && self.history[index_loop].impossible_constraints_2()[player][inferred_constraints[player][0] as usize][inferred_constraints[player][1] as usize]{
                return false
            }
        }
        if inferred_constraints[6].len() == 3 && self.history[index_loop].impossible_constraints_3()[inferred_constraints[6][0] as usize][inferred_constraints[6][1] as usize][inferred_constraints[6][2] as usize]{
            return false
        }
        // =========================================
        for player in 0..7 {
            let mut current_card_counts: [u8; 5] = [0; 5];
            inferred_constraints[player].iter().for_each(|c| current_card_counts[*c as usize] += 1);
            
            let mut required_card_counts: [u8; 5] = [0; 5];
            self.history[index_loop].inferred_constraints()[player].iter().for_each(|c| required_card_counts[*c as usize] += 1);
            self.history[index_loop].public_constraints()[player].iter().for_each(|c| required_card_counts[*c as usize] += 1);

            let mut total_count : u8 = 0;
            current_card_counts.iter().zip(required_card_counts.iter()).for_each(|(cur, req)| total_count += *cur.max(req));
            let fulfilled = if player == 6 {
                total_count <= 3
            } else {
                total_count <= 2
            };
            if !fulfilled {
                log::trace!("is_valid_combination player {} failed to fulfil previous state!", player);
                return false
            }
        }
        true
    }
    pub fn recurse_variants_exchange_public(&self, index_loop: usize, public_constraints: &mut Vec<Vec<Card>>, inferred_constraints: &mut Vec<Vec<Card>>, player_loop: usize, cards: &[u8; 5]) -> bool {
        let player_lives = 2 - self.history[index_loop].public_constraints()[player_loop].len() as u8;
        let mut iter_cards_player = inferred_constraints[player_loop].clone();
        iter_cards_player.sort_unstable();
        iter_cards_player.dedup();
        let mut iter_cards_pile = inferred_constraints[6].clone();
        iter_cards_pile.sort_unstable();
        iter_cards_pile.dedup();
        let mut player_count = [0u8; 5];
        let mut pile_count = [0u8; 5];
        inferred_constraints[player_loop].iter().for_each(|c| player_count[*c as usize] += 1);
        inferred_constraints[6].iter().for_each(|c| pile_count[*c as usize] += 1);
        // let mut redraw_count = [0u8; 5];
        // let mut relinquish_count = [0u8; 5];
        // redraw.iter().for_each(|c| redraw_count[*c as usize] += 1);
        // relinquish.iter().for_each(|c| relinquish_count[*c as usize] += 1);

        
        // Can maybe consider all possible unique moves characterized by player_to_pile and pile_to_player
        // redraw_count and relinquish_count define the degree of freedom for both of those
        // the possible choices that can be player_to_pile depend on whats in player_hand
        // 2 AMB -> up to 2 AMB from player_to_pile
        // 1 AMB -> 1 AMB from player_to_pile
        // So I guess can loop through all possible version of player_to_pile and pile_to_player
        // player_count
        // 0 moves, 1 moves, 2 moves
        // NOTE AMB player to pile and AMB pile to player cancel out so no intersection of player and pile ish

        // 0 player_to_pile move, 0 pile_to_player move
        log::trace!("Before Exchange Same");
        log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
        log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
                
        if self.possible_to_have_cards_recurse(index_loop - 2, public_constraints, inferred_constraints, cards) {
            return true;
        }
        // 1 player_to_pile move, 0 pile_to_player move
        if inferred_constraints[6].len() < 3 && inferred_constraints[player_loop].len() > 0{
            for card_player in iter_cards_player.iter() {
            // move to pile
                if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *card_player) {
                    log::trace!("Before Exchange 1 player_to_pile 0 pile_to_player");
                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                    log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                    inferred_constraints[player_loop].swap_remove(pos);
                    inferred_constraints[6].push(*card_player);
                    if self.possible_to_have_cards_recurse(index_loop - 2, public_constraints, inferred_constraints, cards) {
                        return true;
                    }
                    inferred_constraints[player_loop].push(*card_player);
                    if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *card_player) {
                        inferred_constraints[6].swap_remove(pos);
                    }
                }
            }
        }
        // 0 player_to_pile move, 1 pile_to_player move
        if inferred_constraints[player_loop].len() < 2 && inferred_constraints[6].len() > 0{
            for card_pile in iter_cards_pile.iter() {
            // move to player
                if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *card_pile) {
                    log::trace!("Before Exchange 0 player_to_pile 1 pile_to_player");
                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                    log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                    inferred_constraints[6].swap_remove(pos);
                    inferred_constraints[player_loop].push(*card_pile);
                    if self.possible_to_have_cards_recurse(index_loop - 2, public_constraints, inferred_constraints, cards) {
                        return true;
                    }
                    inferred_constraints[6].push(*card_pile);
                    if let Some(pos) = inferred_constraints[player_loop].iter().position(|c| *c == *card_pile) {
                        inferred_constraints[player_loop].swap_remove(pos);
                    }
                }
            }
        }
        // 1 player_to_pile move, 1 pile_to_player move
        if inferred_constraints[player_loop].len() > 0 && inferred_constraints[6].len() > 0 {
            for card_player in iter_cards_player.iter() {
                for card_pile in iter_cards_pile.iter() {
                    if card_player == card_pile {
                        continue;
                    }
                    log::trace!("Before Exchange 1 player_to_pile 1 pile_to_player");
                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                    log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                    let (mut bool_pile_removed, mut bool_player_removed) = (false, false);
                    if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *card_pile) {
                        inferred_constraints[6].swap_remove(pos);
                        bool_pile_removed = true;
                    }
                    if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *card_player) {
                        inferred_constraints[player_loop].swap_remove(pos);
                        bool_player_removed = true;
                    }
                    inferred_constraints[6].push(*card_player);
                    inferred_constraints[player_loop].push(*card_pile);
                    if self.possible_to_have_cards_recurse(index_loop - 2, public_constraints, inferred_constraints, cards) {
                        return true;
                    }
                    if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *card_pile) {
                        inferred_constraints[player_loop].swap_remove(pos);
                    }
                    if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *card_player) {
                        inferred_constraints[6].swap_remove(pos);
                    }
                    if bool_player_removed {
                        inferred_constraints[player_loop].push(*card_player);
                    }
                    if bool_pile_removed {
                        inferred_constraints[6].push(*card_pile);
                    }
                }
            }
        }
        if player_lives > 1 {
            // 2 player_to_pile move, 0 pile_to_player move
            if inferred_constraints[player_loop].len() == 2 && inferred_constraints[6].len() < 2 {
                log::trace!("Before Exchange 2 player_to_pile 0 pile_to_player");
                log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                let card_0 = inferred_constraints[player_loop][0];
                let card_1 = inferred_constraints[player_loop][1];
                inferred_constraints[player_loop].clear();
                inferred_constraints[6].push(card_0);
                inferred_constraints[6].push(card_1);
                if self.possible_to_have_cards_recurse(index_loop - 2, public_constraints, inferred_constraints, cards) {
                    return true;
                }
                if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == card_0) {
                    inferred_constraints[6].swap_remove(pos);
                }
                if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == card_1) {
                    inferred_constraints[6].swap_remove(pos);
                }
                inferred_constraints[player_loop].push(card_0);
                inferred_constraints[player_loop].push(card_1);
            }
            // 0 player_to_pile move, 2 pile_to_player move
            if inferred_constraints[player_loop].len() == 0 && inferred_constraints[6].len() > 1 {
                for index_pile_to_player_0 in 0..iter_cards_pile.len() {
                    for index_pile_to_player_1 in index_pile_to_player_0..iter_cards_pile.len() {
                        if index_pile_to_player_0 == index_pile_to_player_1 && pile_count[iter_cards_pile[index_pile_to_player_0] as usize] < 2 {
                            continue; // Ensure enough cards to move
                        }
                        log::trace!("Before Exchange 0 player_to_pile 2 pile_to_player");
                        log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                        log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                        let (mut bool_pile_removed_0, mut bool_pile_removed_1) = (false, false);
                        if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == iter_cards_pile[index_pile_to_player_0]) {
                            inferred_constraints[6].swap_remove(pos);
                            bool_pile_removed_0 = true;
                        }
                        if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == iter_cards_pile[index_pile_to_player_1]) {
                            inferred_constraints[6].swap_remove(pos);
                            bool_pile_removed_1 = true;
                        }
                        inferred_constraints[player_loop].push(iter_cards_pile[index_pile_to_player_0]);
                        inferred_constraints[player_loop].push(iter_cards_pile[index_pile_to_player_1]);
                        if self.possible_to_have_cards_recurse(index_loop - 2, public_constraints, inferred_constraints, cards) {
                            return true;
                        }
                        if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == iter_cards_pile[index_pile_to_player_1]) {
                            inferred_constraints[player_loop].swap_remove(pos);
                        }
                        if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == iter_cards_pile[index_pile_to_player_0]) {
                            inferred_constraints[player_loop].swap_remove(pos);
                        }
                        if bool_pile_removed_1 {
                            inferred_constraints[6].push(iter_cards_pile[index_pile_to_player_1]);
                        }
                        if bool_pile_removed_0 {
                            inferred_constraints[6].push(iter_cards_pile[index_pile_to_player_0]);
                        }
                    }
                }
            }
            // 2 player_to_pile move, 1 pile_to_player move
            if inferred_constraints[6].len() > 0 && inferred_constraints[6].len() < 3 && inferred_constraints[player_loop].len() > 1 {
                for card_pile in iter_cards_pile.iter() {
                    for index_player_to_pile_0 in 0..iter_cards_player.len() {
                        // TODO: Shift index_player_to_pile == case shift here
                        if iter_cards_player[index_player_to_pile_0] == *card_pile {
                            continue; // Avoid duplicates
                        }
                        for index_player_to_pile_1 in index_player_to_pile_0..iter_cards_player.len() {
                            // Check DF
                            if iter_cards_player[index_player_to_pile_1] == *card_pile {
                                continue; // Avoid duplicates
                            }
                            if index_player_to_pile_0 == index_player_to_pile_1 && player_count[iter_cards_player[index_player_to_pile_0] as usize] < 2 {
                                // Checks that player has enough cards to move out
                                // TODO: OPTIMIZE Can shift this out of for loop actually
                                continue // Ensure enough cards to move
                            }
                            log::trace!("Before Exchange 2 player_to_pile 1 pile_to_player");
                            log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                            log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                            let (mut bool_player_removed_0, mut bool_player_removed_1, mut bool_pile_removed_0) = (false, false, false);
                            if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == iter_cards_player[index_player_to_pile_0]) {
                                inferred_constraints[player_loop].swap_remove(pos);
                                bool_player_removed_0 = true;
                            }
                            if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == iter_cards_player[index_player_to_pile_1]) {
                                inferred_constraints[player_loop].swap_remove(pos);
                                bool_player_removed_1 = true;
                            }
                            if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *card_pile) {
                                inferred_constraints[6].swap_remove(pos);
                                bool_pile_removed_0 = true;
                            }
                            inferred_constraints[6].push(iter_cards_player[index_player_to_pile_0]);
                            inferred_constraints[6].push(iter_cards_player[index_player_to_pile_1]);
                            inferred_constraints[player_loop].push(*card_pile);
                            if self.possible_to_have_cards_recurse(index_loop - 2, public_constraints, inferred_constraints, cards) {
                                return true;
                            }
                            if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *card_pile) {
                                inferred_constraints[player_loop].swap_remove(pos);
                            }
                            if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == iter_cards_player[index_player_to_pile_1]) {
                                inferred_constraints[6].swap_remove(pos);
                            }
                            if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == iter_cards_player[index_player_to_pile_0]) {
                                inferred_constraints[6].swap_remove(pos);
                            }
                            if bool_pile_removed_0 {
                                inferred_constraints[6].push(*card_pile);
                            }
                            if bool_player_removed_1 {
                                inferred_constraints[player_loop].push(iter_cards_player[index_player_to_pile_1]);
                            }
                            if bool_player_removed_0 {
                                inferred_constraints[player_loop].push(iter_cards_player[index_player_to_pile_0]);
                            }
                        }
                    }
                }
            }
            // 1 player_to_pile move, 2 pile_to_player move
            if inferred_constraints[player_loop].len() == 1 && inferred_constraints[6].len() > 1{
                for card_player in iter_cards_player.iter() {
                    for index_pile_to_player_0 in 0..iter_cards_pile.len() {
                        if iter_cards_pile[index_pile_to_player_0] == *card_player {
                            continue; // Avoid Duplicates
                        }
                        for index_pile_to_player_1 in index_pile_to_player_0..iter_cards_pile.len() {
                            // Check DF
                            if iter_cards_pile[index_pile_to_player_1] == *card_player {
                                continue; // Avoid Duplicates
                            }
                            if index_pile_to_player_0 == index_pile_to_player_1 && (pile_count[iter_cards_pile[index_pile_to_player_0] as usize] < 2) {
                                // Checks that player has enough cards to move out
                                // TODO: OPTIMIZE Can shift this out of for loop actually
                                continue // Ensure enough cards to move
                            }
                            log::trace!("Before Exchange 1 player_to_pile 2 pile_to_player");
                            log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                            log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                            let (mut bool_pile_removed_0, mut bool_pile_removed_1, mut bool_player_removed_0) = (false, false, false);
                            if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == iter_cards_pile[index_pile_to_player_0]) {
                                inferred_constraints[6].swap_remove(pos);
                                bool_pile_removed_0 = true;
                            }
                            if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == iter_cards_pile[index_pile_to_player_1]) {
                                inferred_constraints[6].swap_remove(pos);
                                bool_pile_removed_1 = true;
                            }
                            if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *card_player) {
                                inferred_constraints[player_loop].swap_remove(pos);
                                bool_player_removed_0 = true;
                            }
                            inferred_constraints[player_loop].push(iter_cards_pile[index_pile_to_player_0]);
                            inferred_constraints[player_loop].push(iter_cards_pile[index_pile_to_player_1]);
                            inferred_constraints[6].push(*card_player);
                            if self.possible_to_have_cards_recurse(index_loop - 2, public_constraints, inferred_constraints, cards) {
                                return true;
                            }
                            if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *card_player) {
                                inferred_constraints[6].swap_remove(pos);
                            }
                            if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == iter_cards_pile[index_pile_to_player_1]) {
                                inferred_constraints[player_loop].swap_remove(pos);
                            }
                            if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == iter_cards_pile[index_pile_to_player_0]) {
                                inferred_constraints[player_loop].swap_remove(pos);
                            }
                            if bool_player_removed_0 {
                                inferred_constraints[player_loop].push(*card_player);
                            }
                            if bool_pile_removed_1 {
                                inferred_constraints[6].push(iter_cards_pile[index_pile_to_player_1]);
                            }
                            if bool_pile_removed_0 {
                                inferred_constraints[6].push(iter_cards_pile[index_pile_to_player_0]);
                            }
                        }
                    }
                }
            }
            // 2 player_to_pile move, 2 pile_to_player move
            if inferred_constraints[player_loop].len() > 1 && inferred_constraints[6].len() > 1 {
                for index_player_to_pile_0 in 0..iter_cards_player.len() {
                    for index_player_to_pile_1 in index_player_to_pile_0..iter_cards_player.len() {
                        if index_player_to_pile_0 == index_player_to_pile_1 && player_count[iter_cards_player[index_player_to_pile_0] as usize] < 2 {
                            // Checks that player has enough cards to move out
                            // TODO: OPTIMIZE Can shift this out of for loop actually
                            continue // Ensure enough cards to move
                        }
                        // Check DF
                        for index_pile_to_player_0 in 0..iter_cards_pile.len() {
                            if iter_cards_pile[index_pile_to_player_0] == iter_cards_player[index_player_to_pile_0] || iter_cards_pile[index_pile_to_player_0] == iter_cards_player[index_player_to_pile_1] {
                                continue; // Avoid Duplicates
                            }
                            for index_pile_to_player_1 in index_pile_to_player_0..iter_cards_pile.len() {
                                if iter_cards_pile[index_pile_to_player_1] == iter_cards_player[index_player_to_pile_0] || iter_cards_pile[index_pile_to_player_1] == iter_cards_player[index_player_to_pile_1] {
                                    continue; // Avoid Duplicates
                                }
                                if index_pile_to_player_0 == index_pile_to_player_1 && (pile_count[iter_cards_pile[index_pile_to_player_0] as usize] < 2) {
                                    // Checks that player has enough cards to move out
                                    // TODO: OPTIMIZE Can shift this out of for loop actually
                                    continue // Ensure enough cards to move
                                }
                                log::trace!("Before Exchange 2 player_to_pile 2 pile_to_player");
                                log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                                let (mut bool_pile_removed_0, mut bool_pile_removed_1, mut bool_player_removed_0, mut bool_player_removed_1) = (false, false, false, false);
                                if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == iter_cards_pile[index_pile_to_player_0]) {
                                    inferred_constraints[6].swap_remove(pos);
                                    bool_pile_removed_0 = true;
                                }
                                if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == iter_cards_pile[index_pile_to_player_1]) {
                                    inferred_constraints[6].swap_remove(pos);
                                    bool_pile_removed_1 = true;
                                }
                                if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == iter_cards_player[index_player_to_pile_0]) {
                                    inferred_constraints[player_loop].swap_remove(pos);
                                    bool_player_removed_0 = true;
                                }
                                if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == iter_cards_player[index_player_to_pile_1]) {
                                    inferred_constraints[player_loop].swap_remove(pos);
                                    bool_player_removed_1 = true;
                                }
                                inferred_constraints[player_loop].push(iter_cards_pile[index_pile_to_player_0]);
                                inferred_constraints[player_loop].push(iter_cards_pile[index_pile_to_player_1]);
                                inferred_constraints[6].push(iter_cards_player[index_player_to_pile_0]);
                                inferred_constraints[6].push(iter_cards_player[index_player_to_pile_1]);
                                if self.possible_to_have_cards_recurse(index_loop - 2, public_constraints, inferred_constraints, cards) {
                                    return true;
                                }
                                if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == iter_cards_player[index_player_to_pile_1]) {
                                    inferred_constraints[6].swap_remove(pos);
                                }
                                if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == iter_cards_player[index_player_to_pile_0]) {
                                    inferred_constraints[6].swap_remove(pos);
                                }
                                if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == iter_cards_pile[index_pile_to_player_1]) {
                                    inferred_constraints[player_loop].swap_remove(pos);
                                }
                                if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == iter_cards_pile[index_pile_to_player_0]) {
                                    inferred_constraints[player_loop].swap_remove(pos);
                                }
                                if bool_player_removed_1 {
                                    inferred_constraints[player_loop].push(iter_cards_player[index_player_to_pile_1]);
                                }
                                if bool_player_removed_0 {
                                    inferred_constraints[player_loop].push(iter_cards_player[index_player_to_pile_0]);
                                }
                                if bool_pile_removed_1 {
                                    inferred_constraints[6].push(iter_cards_pile[index_pile_to_player_1]);
                                }
                                if bool_pile_removed_0 {
                                    inferred_constraints[6].push(iter_cards_pile[index_pile_to_player_0]);
                                }
                            } 
                        }
                    }
                }
            }
        }
        false
    }
    /// Brute force generates everything
    fn generate_impossible_constraints(&mut self, history_index: usize) {
        // TODO: [OPTIMIZE] consider total dead cards inferred etc...
        self.impossible_constraints = [[false; 5]; 7];
        self.impossible_constraints_2 = [[[false; 5]; 5]; 7];
        self.impossible_constraints_3 = [[[false; 5]; 5]; 5];
        let mut cards: [u8; 5] = [0; 5];
        for player_of_interest in 0..7 {
            if self.public_constraints[player_of_interest].len() == 2 {
                self.impossible_constraints[player_of_interest] = [true; 5];
                continue;
            }
            for card in 0..5 {
                cards[card] = 1;
                log::trace!("generate_impossible_constraints 1 card : {:?}", card);
                self.impossible_constraints[player_of_interest][card] = self.impossible_to_have_cards_general(history_index, player_of_interest, &cards);
                cards[card] = 0;
            }
        }
        for player_of_interest in 0..7 {
            if self.public_constraints[player_of_interest].len() > 0 {
                self.impossible_constraints_2[player_of_interest] = [[true; 5]; 5];
                continue;
            }
            for card_a in 0..5 {
                for card_b in card_a..5 {
                    cards[card_a] += 1;
                    cards[card_b] += 1;
                    log::trace!("generate_impossible_constraints 2 card : {:?}, {:?}", card_a, card_b);
                    let output = self.impossible_to_have_cards_general(history_index, player_of_interest, &cards);
                    // OPTIMIZE lmao...
                    self.impossible_constraints_2[player_of_interest][card_a][card_b] = output;
                    self.impossible_constraints_2[player_of_interest][card_b][card_a] = output;
                    cards[card_a] -= 1;
                    cards[card_b] -= 1;
                }
            }
        }
        for card_a in 0..5 {
            for card_b in card_a..5 {
                for card_c in card_b..5 {
                    cards[card_a] += 1;
                    cards[card_b] += 1;
                    cards[card_c] += 1;
                    log::trace!("generate_impossible_constraints 3 card : {:?}, {:?}, {:?}", card_a, card_b, card_c);
                    let output = self.impossible_to_have_cards_general(history_index, 6, &cards);
                    // OPTIMIZE lmao...
                    self.impossible_constraints_3[card_a][card_b][card_c] = output;
                    self.impossible_constraints_3[card_a][card_c][card_b] = output;
                    self.impossible_constraints_3[card_b][card_a][card_c] = output;
                    self.impossible_constraints_3[card_b][card_c][card_a] = output;
                    self.impossible_constraints_3[card_c][card_a][card_b] = output;
                    self.impossible_constraints_3[card_c][card_b][card_a] = output;
                    cards[card_a] -= 1;
                    cards[card_b] -= 1;
                    cards[card_c] -= 1;
                }
            }
        }
    }
    /// Generates based on impossible_constraints
    fn generate_inferred_constraints(&mut self) {
        self.inferred_constraints.iter_mut().for_each(|v| v.clear());
        for player in 0..6 {
            if self.public_constraints[player].len() == 0 {
                if self.impossible_constraints[player].iter().map(|b| !*b as u8).sum::<u8>() == 1 {
                    if let Some(card_num) = self.impossible_constraints[player].iter().position(|b| !*b) {
                        self.inferred_constraints[player].push(Card::try_from(card_num as u8).unwrap());
                        self.inferred_constraints[player].push(Card::try_from(card_num as u8).unwrap());
                        continue;
                    }
                }
                // if 1 card not impossible and all the rest impossible
                let mut must_have_card: [u8; 5] = [3; 5];
                'outer: for card_num_a in 0..5 {
                    for card_num_b in card_num_a..5 {
                        // AA AB BB
                        // means nothing, I need to check if all have A or all have B
                        // need count lol
                        if self.impossible_constraints_2[player][card_num_a][card_num_b] {
                            continue;
                        }
                        let mut next = [0u8; 5];
                        next[card_num_a] += 1;
                        next[card_num_b] += 1;
                        must_have_card.iter_mut().zip(next.iter()).for_each(|(m, n)| *m = (*m).min(*n));
                        if must_have_card == [0; 5] {
                            break 'outer;
                        }
                    }
                }
                for (card_num, card_count) in must_have_card.iter().enumerate() {
                    for _ in 0..*card_count {
                        self.inferred_constraints[player].push(Card::try_from(card_num as u8).unwrap());
                    }
                } 
            } else if self.public_constraints[player].len() == 1 {
                if self.impossible_constraints[player].iter().map(|b| !*b as u8).sum::<u8>() == 1 {
                    if let Some(card_num) = self.impossible_constraints[player].iter().position(|b| !*b) {
                        self.inferred_constraints[player].push(Card::try_from(card_num as u8).unwrap());
                        continue;
                    }
                }
            }
        }
        let mut must_have_card: [u8; 5] = [3; 5];
        'outer: for card_num_a in 0..5 {
            for card_num_b in card_num_a..5 {
                for card_num_c in card_num_b..5 {
                    // AA AB BB
                    // means nothing, I need to check if all have A or all have B
                    // need count lol
                    if self.impossible_constraints_3[card_num_a][card_num_b][card_num_c] {
                        continue;
                    }
                    let mut next = [0u8; 5];
                    next[card_num_a] += 1;
                    next[card_num_b] += 1;
                    next[card_num_c] += 1;
                    must_have_card.iter_mut().zip(next.iter()).for_each(|(m, n)| *m = (*m).min(*n));
                    if must_have_card == [0; 5] {
                        return
                        // break 'outer;
                    }
                }
            }
        }
        for (card_num, card_count) in must_have_card.iter().enumerate() {
            for _ in 0..*card_count {
                log::trace!("generate_inferred_constraints must_have_card: {:?}", must_have_card);
                log::trace!("generate_inferred_constraints pushing: {:?}", Card::try_from(card_num as u8).unwrap());
                self.inferred_constraints[6].push(Card::try_from(card_num as u8).unwrap());
            }
        } 
    }
    pub fn impossible_constraints(&self) -> &[[bool; 5]; 7] {
        &self.impossible_constraints
    }
    pub fn impossible_constraints_2(&self) -> &[[[bool; 5]; 5]; 7] {
        &self.impossible_constraints_2
    }
    /// Returns an array of [player][card] that returns true if a player cannot have a particular card alive
    pub fn generate_one_card_impossibilities_player_card_indexing(&self) -> [[bool; 5]; 7] {
        self.impossible_constraints.clone()
    }
    /// Returns an array of [player][card] that returns true if a player cannot have a particular card alive
    pub fn generate_two_card_impossibilities_player_card_indexing(&self) -> [[[bool; 5]; 5]; 7] {
        self.impossible_constraints_2.clone()
    }
    pub fn generate_three_card_impossibilities_player_card_indexing(&self) -> [[[bool; 5]; 5]; 5] {
        self.impossible_constraints_3.clone()
    }
}

impl CoupConstraint for BackTrackCollectiveConstraintLite {
    // TODO: OPTIMIZE, i guess you don't really need history inside here...
    fn game_start_public() -> Self {
        let public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::new()]; 
        let inferred_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(3)]; 
        // let revealed_status = vec![Vec::with_capacity(5); 7];
        // TODO: Add inferred_card_count
        let mut history: Vec<SignificantAction> = Vec::with_capacity(50);
        // Start takes the inferred information discovered via a pathdependent lookback
        history.push(SignificantAction::start());
        // StartInferred takes the inferred information from start, and runs add_inferred_information
        // This seperation prevents handling cases where you add discovered information that is already inside due to add_inferred_information
        history.push(SignificantAction::start_inferred());
        Self {
            public_constraints,
            inferred_constraints,
            impossible_constraints: [[false; 5]; 7],
            impossible_constraints_2: [[[false; 5]; 5]; 7], 
            impossible_constraints_3: [[[false; 5]; 5]; 5], 
            move_no: 1,
            history,
        }
    }
    // TODO: OPTIMIZE, i guess you don't really need history inside here...
    fn game_start_private(player: usize, cards: &[Card; 2]) -> Self {
        let public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::new()]; 
        let inferred_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(3)]; 
        // let revealed_status = vec![Vec::with_capacity(5); 7];
        // TODO: Add inferred_card_count
        let mut history: Vec<SignificantAction> = Vec::with_capacity(50);
        // Start takes the inferred information discovered via a pathdependent lookback
        let mut start = SignificantAction::start(); 
        start.add_inferred_constraints(player, cards[0]);
        start.add_inferred_constraints(player, cards[1]);
        start.meta_data.impossible_constraints[player] = [true; 5];
        start.meta_data.impossible_constraints[player][cards[0] as usize] = false;
        start.meta_data.impossible_constraints[player][cards[1] as usize] = false;
        start.meta_data.impossible_constraints_2[player] = [[true; 5]; 5];
        start.meta_data.impossible_constraints_2[player][cards[0] as usize][cards[1] as usize] = false;
        start.meta_data.impossible_constraints_2[player][cards[1] as usize][cards[0] as usize] = false;
        history.push(start);
        // StartInferred takes the inferred information from start, and runs add_inferred_information
        // This seperation prevents handling cases where you add discovered information that is already inside due to add_inferred_information
        let mut start_inferred = SignificantAction::start_inferred(); 
        start_inferred.add_inferred_constraints(player, cards[0]);
        start_inferred.add_inferred_constraints(player, cards[1]);
        start_inferred.meta_data.impossible_constraints[player] = [true; 5];
        start_inferred.meta_data.impossible_constraints[player][cards[0] as usize] = false;
        start_inferred.meta_data.impossible_constraints[player][cards[1] as usize] = false;
        start_inferred.meta_data.impossible_constraints_2[player] = [[true; 5]; 5];
        start_inferred.meta_data.impossible_constraints_2[player][cards[0] as usize][cards[1] as usize] = false;
        start_inferred.meta_data.impossible_constraints_2[player][cards[1] as usize][cards[0] as usize] = false;
        history.push(start_inferred);
        Self {
            public_constraints,
            inferred_constraints,
            impossible_constraints: [[false; 5]; 7],
            impossible_constraints_2: [[[false; 5]; 5]; 7], 
            impossible_constraints_3: [[[false; 5]; 5]; 5], 
            move_no: 1,
            history,
        }
    }
    /// Adds move that may contain private or public info
    fn add_move(&mut self, player_id: u8, action: ActionInfo) {
        debug_assert!(action != ActionInfo::Start && action != ActionInfo::StartInferred, "should not be pushing this!");
        // match action {
        //     ActionInfo::Discard { .. } => {
        //         let significant_action = SignificantAction::initial(self.move_no, player_id, action);
        //         // Pushing before due to recursion mechanic
        //         self.history.push(significant_action);
        //         self.calculate_stored_move_initial();
        //         // Handle inference
        //     },
        //     ActionInfo::RevealRedraw { .. } => {
        //         let significant_action = SignificantAction::initial(self.move_no, player_id, action);
        //         self.history.push(significant_action);
        //         self.calculate_stored_move_initial();
        //         // Handle inference
        //     },
        //     ActionInfo::ExchangeDrawChoice { .. } => {
        //         let significant_action = SignificantAction::initial(self.move_no, player_id, action);
        //         // Handle inference
        //         // TODO: This is temporary, unsure how might split between public and private
        //         // It is possible that we can just use private, since public is just private with empty vec?
        //         self.history.push(significant_action);
        //         self.calculate_stored_move_initial();
        //     },
        //     ActionInfo::ExchangeChoice { hand, relinquish } => {
        //         let significant_action = SignificantAction::initial(self.move_no, player_id, action);
        //         self.history.push(significant_action);
        //         self.calculate_stored_move_initial();
        //     },
        //     ActionInfo::ExchangeDraw { draw } => {
        //         let significant_action = SignificantAction::initial(self.move_no, player_id, action);
        //         self.history.push(significant_action);
        //         self.calculate_stored_move_initial();
        //     }
        //     ActionInfo::Start 
        //     | ActionInfo::StartInferred => {
        //         // TODO: Consider removing Start, so we can eliminate this branch entirely
        //         debug_assert!(false, "should not be pushing this!");
        //     },
        // }
        let significant_action = SignificantAction::initial(self.move_no, player_id, action);
        self.history.push(significant_action);
        self.calculate_stored_move_initial();
        // post increment
        self.move_no += 1;
    }

    fn printlog(&self) {
        log::info!("{}", format!("Public Constraints: {:?}", self.public_constraints));
        log::info!("{}", format!("Inferred Constraints: {:?}", self.inferred_constraints));
        // let info_strings = self.history.iter().map(|s| s.action_info_str()).collect::<Vec<String>>();
        log::info!("{}", format!("History: {:?}", self.history.iter().map(|s| s.action_info_str()).collect::<Vec<String>>()));
    }
}
impl CoupConstraintAnalysis for BackTrackCollectiveConstraintLite {
    fn public_constraints(&self) -> &Vec<Vec<Card>> {
        &self.public_constraints
    }

    fn sorted_public_constraints(&mut self) -> &Vec<Vec<Card>> {
        self.public_constraints.iter_mut().for_each(|v| v.sort_unstable());
        &self.public_constraints
    }
    
    fn inferred_constraints(&mut self) -> &Vec<Vec<Card>> {
        // self.generate_inferred_constraints();
        &self.inferred_constraints
    }
    
    fn sorted_inferred_constraints(&mut self) -> &Vec<Vec<Card>> {
        // self.generate_inferred_constraints();
        self.inferred_constraints.iter_mut().for_each(|v| v.sort_unstable());
        &self.inferred_constraints
    }

    fn player_impossible_constraints(&mut self) -> &[[bool; 5]; 7] {
        &self.impossible_constraints
    }

    fn player_impossible_constraints_paired(&mut self) -> &[[[bool; 5]; 5]; 7] {
        &self.impossible_constraints_2
    }

    fn player_impossible_constraints_triple(&mut self) -> &[[[bool; 5]; 5]; 5] {
        &self.impossible_constraints_3
    }
    
    fn player_can_have_card_alive(&self, player: u8, card: Card) -> bool{
        !self.impossible_constraints[player as usize][card as usize]
    }
    
    fn player_can_have_cards_alive(&self, player: u8, cards: &Vec<Card>) -> bool{
        if player < 6 {
            if cards.len() == 2 {
                return !self.impossible_constraints_2[player as usize][cards[0] as usize][cards[1] as usize]
            } else if cards.len() == 1 {
                return self.player_can_have_card_alive(player, cards[0])
            }
        } else if player == 6 {
            if cards.len() == 1 {
                return self.player_can_have_card_alive(player, cards[0])
            } else if cards.len() == 2 {
                return !self.impossible_constraints_2[player as usize][cards[0] as usize][cards[1] as usize]
            } else if cards.len() == 3 {
                return !self.impossible_constraints_3[cards[0] as usize][cards[1] as usize][cards[2] as usize]
            }
        }
        false
    }
}