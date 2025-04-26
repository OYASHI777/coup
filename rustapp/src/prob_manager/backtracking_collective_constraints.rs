use crate::history_public::{AOName, ActionObservation, Card};
use super::{collective_constraint::CompressedCollectiveConstraint, compressed_group_constraint::CompressedGroupConstraint};
use ahash::AHashSet;
use crossbeam::channel::after;
use std::{marker::Copy, path::Path};

#[derive(Clone, Debug)]
pub enum ActionInfo {
    Start,
    StartInferred,
    Discard {discard: Card}, // Player | Card
    // reveal: card publically shown
    // redraw: Card taken from pile
    // relinquish: Card left in pile
    RevealRedraw {reveal: Card, redraw: Option<Card>, relinquish: Option<Card>}, // Player | Reveal Card | Redraw Option<Card>
    ExchangeDrawChoice {draw: Vec<Card>, relinquish: Vec<Card>}, // Player | Draw Vec<Card> | Return Vec<Card>
}

impl ActionInfo {
    pub fn public(&self) -> Self {
        match self {
            ActionInfo::Start => {
                self.clone()
            },
            ActionInfo::StartInferred => {
                self.clone()
            },
            ActionInfo::Discard { discard } => {
                self.clone()
            },
            ActionInfo::RevealRedraw { reveal: revealed, .. } => {
                ActionInfo::RevealRedraw { reveal: *revealed, redraw: None, relinquish: None }
            },
            ActionInfo::ExchangeDrawChoice { draw, .. } => {
                ActionInfo::ExchangeDrawChoice { draw: Vec::with_capacity(2), relinquish: Vec::with_capacity(2) }
            },
        }
    }
    /// All private information is known
    pub fn fully_known(&self) -> bool {
        match self {
            Self::Start => {
                true
            }
            Self::StartInferred => {
                true
            }
            Self::Discard{ .. } => {
                true
            },
            Self::RevealRedraw{ redraw, .. } => {
                redraw.is_some()
            },
            Self::ExchangeDrawChoice {draw, relinquish} => {
                draw.len() == 2 && relinquish.len() == 2
            }
        }
    }
    /// At least some private information known, or no private information to know
    pub fn partially_known(&self) -> bool {
        match self {
            Self::Start => {
                true
            }
            Self::StartInferred => {
                true
            }
            Self::Discard{ .. } => {
                true
            },
            Self::RevealRedraw{ redraw, .. } => {
                redraw.is_some()
            },
            Self::ExchangeDrawChoice {draw, relinquish} => {
                draw.len() > 0 || relinquish.len() > 0
            }
        }
    }
    /// No private information is known
    pub fn fully_unknown(&self) -> bool {
        match self {
            Self::Start => {
                false
            }
            Self::StartInferred => {
                false
            }
            Self::Discard{ .. } => {
                false
            },
            Self::RevealRedraw{ redraw, .. } => {
                redraw.is_none()
            },
            Self::ExchangeDrawChoice {draw, relinquish} => {
                draw.len() == 0 && relinquish.len() == 0
            }
        }
    }
    pub fn name(&self) -> ActionInfoName {
        match self {
            ActionInfo::Start => {
                ActionInfoName::Start
            },
            ActionInfo::StartInferred => {
                ActionInfoName::StartInferred
            },
            ActionInfo::Discard { .. } => {
                ActionInfoName::Discard
            },
            ActionInfo::RevealRedraw { .. } => {
                ActionInfoName::RevealRedraw
            },
            ActionInfo::ExchangeDrawChoice { .. } => {
                ActionInfoName::ExchangeDrawChoice
            },
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionInfoName {
    Start,
    StartInferred,
    Discard, // Player | Card
    RevealRedraw, // Player | Reveal Card | Redraw Option<Card>
    ExchangeDrawChoice, // Player | Draw Vec<Card> | Return Vec<Card>
}

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
// TODO: change gamestart for different inferred starting hands
// TODO: DO NOT calculate if all ambassadors before are unknown, and current is ambassador
// TODO: Implement move counter properly
// TODO: implement Into<BacktrackMetaData> for BackTrackCollectiveConstraint
// TODO: Store SignificantAction
// TODO: Fix up add_inferred_cards API to not take vec_changes
// TOD: Fix up reveal_redraw API to make inline with add_inferred_card, even though it also adds a group
// TODO: [OPTIMIZE] impossible_constraints can be stored as a u64 / 7 u8s (56 bits)
#[derive(Clone, Debug)]
pub struct BacktrackMetaData {
    public_constraints: Vec<Vec<Card>>,
    inferred_constraints: Vec<Vec<Card>>,
    impossible_constraints: [[bool; 5]; 7], // For each player store an array of bool where each index is a Card, this represents whether a player cannot have a card true => cannot
    impossible_constraints_2: [[[bool; 5]; 5]; 7],
    impossible_constraints_3: [[[bool; 5]; 5]; 5],
}

impl BacktrackMetaData {
    pub fn initial() -> Self {
        Self::start()
    }
    pub fn start() -> Self {
        let public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::new()]; 
        let inferred_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(3)]; 
        let impossible_constraints: [[bool; 5]; 7] = [[false; 5]; 7];
        let impossible_constraints_2: [[[bool; 5]; 5]; 7] = [[[false; 5]; 5]; 7];
        let impossible_constraints_3: [[[bool; 5]; 5]; 5] = [[[false; 5]; 5]; 5];
        Self {
            public_constraints,
            inferred_constraints,
            impossible_constraints,
            impossible_constraints_2,
            impossible_constraints_3,
        }
    }
    pub fn public_constraints(&self) -> &Vec<Vec<Card>> {
        &self.public_constraints
    }
    pub fn inferred_constraints(&self) -> &Vec<Vec<Card>> {
        &self.inferred_constraints
    }   
    pub fn set_inferred_constraints(&mut self, inferred_constraints: &Vec<Vec<Card>>) {
        self.inferred_constraints = inferred_constraints.clone();
    }   
    pub fn impossible_constraints(&self) -> &[[bool; 5]; 7] {
        &self.impossible_constraints
    }   
    /// Changes stored impossible_constraints
    pub fn set_impossible_constraints(&mut self, impossible_constraints: &[[bool; 5]; 7]) {
        self.impossible_constraints = impossible_constraints.clone();
    }
    pub fn impossible_constraints_2(&self) -> &[[[bool; 5]; 5]; 7] {
        &self.impossible_constraints_2
    }   
    pub fn impossible_constraints_3(&self) -> &[[[bool; 5]; 5]; 5] {
        &self.impossible_constraints_3
    }   
    pub fn player_cards_known<T>(&self, player_id: T) -> usize 
    where
        T: Into<usize> + Copy,
    {
        self.public_constraints[player_id.into()].len() + self.inferred_constraints[player_id.into()].len()
    }
    pub fn player_has_public_constraint<T>(&self, player_id: T, card: Card) -> bool 
    where
        T: Into<usize> + Copy,
    {   
        self.public_constraints[player_id.into()].contains(&card)
    }
    pub fn player_has_inferred_constraint<T>(&self, player_id: T, card: Card) -> bool 
    where
        T: Into<usize> + Copy,
    {   
        self.inferred_constraints[player_id.into()].contains(&card)
    }
    pub fn player_constraints_all_full<T>(&self, player_id: T, card: Card) -> bool 
    where
        T: Into<usize> + Copy,
    {   
        self.player_cards_known(player_id) == 2 &&
        self.inferred_constraints[player_id.into()].iter().all(|&c| c == card) &&
        self.public_constraints[player_id.into()].iter().all(|&c| c == card)
    }
}   

// 1: Add recursion on finding inferred constraint
//      - Can possibly store a boolean that determines if any empty redraw is before a number, so no need to lookback for that
// 2: Optimize to consider where we might not need to recurse (non recursive method can get 1/250 games wrong)
//      - Consider memoizing by storing a card, that a revealredraw might need to redraw
//          - during forward pass, if its impossible for player to have that card, then redraw it
#[derive(Clone, Debug)]
/// A struct that helps in card counting. Stores all information known about cards by a particular player.
pub struct BackTrackCollectiveConstraint {
    public_constraints: Vec<Vec<Card>>, // Stores all the dead cards of dead players, None are all behind
    inferred_constraints: Vec<Vec<Card>>, // Stores all the inferred cards of alive players 
    impossible_constraints: [[bool; 5]; 7], // For each player store an array of bool where each index is a Card, this represents whether a player cannot have a card true => cannot
    impossible_constraints_2: [[[bool; 5]; 5]; 7], // [Card Smaller][Card Bigger]
    impossible_constraints_3: [[[bool; 5]; 5]; 5], // [Card small] [Card med][Card big]
    move_no: usize, // turn number (0 being the start) (post increment this so assign then increment)
    history: Vec<SignificantAction>, // Stores 
}

impl BackTrackCollectiveConstraint {
    pub fn game_start() -> Self {
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
    /// TODO: change gamestart for different inferred starting hands
    /// Recreates start state based on Start ActionInfo which may include inferred cards
    fn regenerate_game_start(&mut self) {
        self.public_constraints.iter_mut().for_each(|v| v.clear());
        self.inferred_constraints = self.history.first().unwrap().inferred_constraints().clone();
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
    /// Recalculate current Constraint from scratch using history
    /// Can recursively call itself
    fn regenerate_path(&mut self) {
        log::info!("Regenerating Path");
        // self.regenerate_game_start();
        // TODO: Implement skipping starting empty ambassadors
        let mut skip_starting_empty_ambassador: bool = true;
        for index in 1..self.history.len() {
            // run the update for that action
            // if action is an starting empty ambassador, continue
            // Should just run 2 loops so you skip the branch really
            self.calculate_stored_move(index);
        }
        log::trace!("End Regenerate End");
    }
    fn calculate_stored_move(&mut self, history_index: usize) {
        // let action: &SignificantAction = &self.history[history_index];
        log::trace!("calculate_stored_move history_index: {history_index}, player: {} {:?}", self.history[history_index].player(), self.history[history_index].action_info());
        let (player_id, action_info) = {
            let action = &self.history[history_index];
            (action.player() as usize, action.action_info().clone())
        };
    
        match action_info {
            ActionInfo::Start 
            |ActionInfo::StartInferred => {
                return
            },
            ActionInfo::Discard{ discard} => {
                self.death(player_id as usize, discard);
            },
            ActionInfo::RevealRedraw{ reveal, redraw, relinquish  } => {
                // TODO: reveal_relinquish?
                // TODO: handle the known card case obviously lol
                match redraw {
                    None => {
                        // TODO: [OPTIMIZE] If lookback_1 checks based on reveal
                        //  Then whenever u regenerate, you don't really have to lookback as there is nothing new to check
                        match relinquish {
                            Some(relinquish_card) => {
                                debug_assert!(relinquish_card == reveal, "We normally assume reveal and relinquish are the same else, one should use redraw");
                                self.reveal_redraw_relinquish(player_id, relinquish_card);
                                // self.reveal_redraw(history_index, player_id, reveal);
                            },
                            None => {
                                self.reveal_redraw(history_index, player_id, reveal);
                            },
                        }
                    },
                    Some(drawn) => {
                        if drawn == reveal {
                            // TODO: Probably check if inferred_card already there
                            // if so add it in.
                            self.reveal_redraw_same(player_id, reveal);
                            log::trace!("After reveal_redraw_same");
                            self.printlog();
                        } else {
                            // TODO: Redraw can give you back info about the previous ambassador perhaps?
                            // TODO: Swap would be custom and required
                            // TODO: Refactor for neatness
                            self.reveal_redraw_diff(player_id, reveal, drawn);
                            log::trace!("After reveal_redraw_diff");
                            self.printlog();
                        }
                    },
                }
            },
            ActionInfo::ExchangeDrawChoice{ draw, relinquish } => {
                // TODO: handle the known card case obviously lol
                self.ambassador_public(player_id);
            },
        }
        // TODO: generate impossible_constraints can differ by move.
        log::trace!("calculate_stored_move generate_impossible_constraints history_index: {history_index}");
        self.generate_impossible_constraints();
        self.history[history_index].meta_data = self.to_meta_data();
        log::info!("recalculated_stored_move end: player: {player_id} {} {:?}", history_index, self.history[history_index].action_info());
        self.printlog();
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
                self.death_initial(player_id as usize, discard);
            },
            ActionInfo::RevealRedraw{ reveal, redraw, .. } => {
                // TODO: handle the known card case obviously lol
                match redraw {
                    None => {},
                    Some(drawn) => {
                        if drawn == reveal {
                            // TODO: Probably check if inferred_card already there
                            // if so add it in.
                            // self.reveal_redraw_same(player_id, reveal);
                            unimplemented!("Not handling private information cases yet")
                        } else {
                            // TODO: Redraw can give you back info about the previous ambassador perhaps?
                            // TODO: Swap would be custom and required
                            // TODO: Refactor for neatness
                            // self.reveal_redraw_diff(player_id, reveal, drawn);
                            unimplemented!("Not handling private information cases yet")
                        }
                    },
                }
            },
            ActionInfo::ExchangeDrawChoice{ draw, relinquish } => {
                // TODO: handle the known card case obviously lol
                self.ambassador_public(player_id);
            },
        }
        self.generate_impossible_constraints();
        self.history[history_index].meta_data = self.to_meta_data();
        log::info!("recalculated_stored_move_initial: {} {:?}", history_index, self.history[history_index].action_info());
        self.printlog();
    }   
    // TODO: [OPTIMIZE] this to make it more ergonomic 
    /// handles pushing of LATEST moves only
    /// Does not care for public/private state of action_info
    /// NOTE: move_no in collective_constraint is different from move_no in game
    pub fn add_move(&mut self, player_id: u8, action_info: ActionInfo) {
        match action_info {
            ActionInfo::Discard { .. } => {
                let significant_action = SignificantAction::initial(self.move_no, player_id, action_info);
                // Pushing before due to recursion mechanic
                self.history.push(significant_action);
                self.calculate_stored_move_initial();
                // Handle inference
            },
            ActionInfo::RevealRedraw { .. } => {
                let significant_action = SignificantAction::initial(self.move_no, player_id, action_info);
                self.history.push(significant_action);
                self.calculate_stored_move_initial();
                // Handle inference
            },
            ActionInfo::ExchangeDrawChoice { .. } => {
                let significant_action = SignificantAction::initial(self.move_no, player_id, action_info);
                // Handle inference
                // TODO: This is temporary, unsure how might split between public and private
                // It is possible that we can just use private, since public is just private with empty vec?
                self.history.push(significant_action);
                self.calculate_stored_move_initial();
            },
            ActionInfo::Start 
            | ActionInfo::StartInferred => {
                // TODO: Consider removing Start, so we can eliminate this branch entirely
                debug_assert!(false, "should not be pushing this!");
            },
        }
        // post increment
        self.move_no += 1;
    }
    // Add other normal methods for inference
}

impl BackTrackCollectiveConstraint {
    pub fn sorted_public_constraints(&self) -> Vec<Vec<Card>> {
        let mut output = self.public_constraints.clone();
        for card_vec in output.iter_mut() {
            card_vec.sort_unstable()
        }
        output
    }
    pub fn sorted_inferred_constraints(&self) -> Vec<Vec<Card>> {
        let mut output = self.inferred_constraints.clone();
        for card_vec in output.iter_mut() {
            card_vec.sort_unstable()
        }
        output
    }
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
    /// Logs the state
    pub fn printlog(&self) {
        log::info!("{}", format!("Public Constraints: {:?}", self.public_constraints));
        log::info!("{}", format!("Inferred Constraints: {:?}", self.inferred_constraints));
        let info_strings = self.history.iter().map(|s| s.action_info_str()).collect::<Vec<String>>();
        log::info!("{}", format!("History: {:?}", info_strings));
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
        self.lookback_initial();
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
        self.lookback_initial();
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
    pub fn lookback_initial(&mut self) -> bool {
        todo!()
    }
    /// Assumes a maximally informative group is present
    fn generate_impossible_constraints(&mut self) {
        todo!()
    }
    /// Returns an array of [player][card] that returns true if a player cannot have a particular card alive
    pub fn generate_one_card_impossibilities_player_card_indexing(&self) -> [[bool; 5]; 7] {
        self.impossible_constraints.clone()
    }
}