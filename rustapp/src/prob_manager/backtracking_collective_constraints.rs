use crate::history_public::{AOName, ActionObservation, Card};
use crate::traits::prob_manager::coup_analysis::CoupConstraintAnalysis;
use super::{backtracking_prob::CoupConstraint, collective_constraint::CompressedCollectiveConstraint, compressed_group_constraint::CompressedGroupConstraint};
use ahash::AHashSet;
use crossbeam::channel::after;
use std::{marker::Copy, path::Path};


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionInfo {
    Start,
    StartInferred,
    Discard {discard: Card}, // Player | Card
    RevealRedraw {reveal: Card, redraw: Option<Card>, relinquish: Option<Card>}, // Player | Reveal Card | Redraw Option<Card>
    ExchangeDraw {draw: Vec<Card>}, // Player | Draw Vec<Card> | Return Vec<Card>
    ExchangeChoice {relinquish: Vec<Card>}, // Player | Draw Vec<Card> | Return Vec<Card>
    ExchangeDrawChoice {draw: Vec<Card>, relinquish: Vec<Card>}, // Player | Draw Vec<Card> | Return Vec<Card>
}

impl ActionInfo {
    pub fn public(&self) -> Self {
        match self {
            Self::Start => {
                self.clone()
            },
            Self::StartInferred => {
                self.clone()
            },
            Self::Discard { discard } => {
                self.clone()
            },
            Self::RevealRedraw { reveal: revealed, .. } => {
                Self::RevealRedraw { reveal: *revealed, redraw: None, relinquish: None }
            },
            Self::ExchangeDrawChoice { draw, .. } => {
                Self::ExchangeDrawChoice { draw: Vec::with_capacity(2), relinquish: Vec::with_capacity(2) }
            },
            Self::ExchangeDraw { .. } => {
                Self::ExchangeDraw { draw: Vec::with_capacity(2) }
            },
            Self::ExchangeChoice { .. } => {
                Self::ExchangeChoice { relinquish: Vec::with_capacity(2) }
            }
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
            },
            Self::ExchangeDraw { draw } => {
                draw.len() == 2
            },
            Self::ExchangeChoice { relinquish } => {
                unimplemented!("This is indeterminate as we need to know if player has 1 life or 2")
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
            Self::ExchangeDraw { draw } => {
                !draw.is_empty()
            }
            Self::ExchangeChoice { relinquish } => {
                !relinquish.is_empty()
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
            Self::ExchangeDraw { draw } => {
                draw.is_empty()
            }
            Self::ExchangeChoice { relinquish } => {
                relinquish.is_empty()
            }
        }
    }
    pub fn name(&self) -> ActionInfoName {
        match self {
            Self::Start => {
                ActionInfoName::Start
            },
            Self::StartInferred => {
                ActionInfoName::StartInferred
            },
            Self::Discard { .. } => {
                ActionInfoName::Discard
            },
            Self::RevealRedraw { .. } => {
                ActionInfoName::RevealRedraw
            },
            Self::ExchangeDrawChoice { .. } => {
                ActionInfoName::ExchangeDrawChoice
            },
            Self::ExchangeDraw { .. } => {
                ActionInfoName::ExchangeDraw
            },
            Self::ExchangeChoice { .. } => {
                ActionInfoName::ExchangeChoice
            }
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionInfoName {
    Start,
    StartInferred,
    Discard, // Player | Card
    RevealRedraw, // Player | Reveal Card | Redraw Option<Card>
    ExchangeDraw, 
    ExchangeChoice, 
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
        let meta_data: BacktrackMetaData = BacktrackMetaData::start_public();
        Self {
            move_no: 0,
            player: 77,
            action_info: ActionInfo::Start,
            meta_data,
        }
    }
    pub fn start_inferred() -> Self {
        let meta_data: BacktrackMetaData = BacktrackMetaData::start_public();
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
    pub public_constraints: Vec<Vec<Card>>,
    pub inferred_constraints: Vec<Vec<Card>>,
    pub impossible_constraints: [[bool; 5]; 7], // For each player store an array of bool where each index is a Card, this represents whether a player cannot have a card true => cannot
    pub impossible_constraints_2: [[[bool; 5]; 5]; 7],
    pub impossible_constraints_3: [[[bool; 5]; 5]; 5],
}

impl BacktrackMetaData {
    pub fn initial() -> Self {
        Self::start_public()
    }
    pub fn start_public() -> Self {
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
    pub fn start_private(player: usize, cards: &[Card; 2]) -> Self {
        debug_assert!(cards.len() < 3, "player has too many cards!");
        let public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::new()]; 
        let mut inferred_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(3)]; 
        inferred_constraints[player].push(cards[0]);
        inferred_constraints[player].push(cards[1]);
        // Start takes the inferred information discovered via a pathdependent lookback
        let mut impossible_constraints = [[false; 5]; 7];
        impossible_constraints[player] = [true; 5];
        impossible_constraints[player][cards[0] as usize] = false;
        impossible_constraints[player][cards[1] as usize] = false;
        let mut impossible_constraints_2 = [[[false; 5]; 5]; 7];
        impossible_constraints_2[player] = [[true; 5]; 5];
        let mut impossible_constraints_3 = [[[false; 5]; 5]; 5];
        impossible_constraints_3[cards[0] as usize][cards[0] as usize][cards[0] as usize] = true;
        impossible_constraints_3[cards[1] as usize][cards[1] as usize][cards[1] as usize] = true;
        if cards[0] == cards[1] {
            // update impossible_2
            for p in 0..7 {
                impossible_constraints_2[p][cards[0] as usize][cards[0] as usize] = true;
            }
            // update impossible_3 where more than 2
            for c in 0..5 {
                impossible_constraints_3[cards[0] as usize][cards[0] as usize][c] = true;
                impossible_constraints_3[cards[0] as usize][c][cards[0] as usize] = true;
                impossible_constraints_3[c][cards[0] as usize][cards[0] as usize] = true;
            }
        }
        impossible_constraints_2[player][cards[0] as usize][cards[1] as usize] = false;
        impossible_constraints_2[player][cards[1] as usize][cards[0] as usize] = false;
        // StartInferred takes the inferred information from start, and runs add_inferred_information
        // This seperation prevents handling cases where you add discovered information that is already inside due to add_inferred_information
        Self {
            public_constraints,
            inferred_constraints,
            impossible_constraints,
            impossible_constraints_2, 
            impossible_constraints_3, 
        }
    }
    /// Clones meta_data but only with public data copied
    pub fn clone_public(&self) -> Self {
        let public_constraints: Vec<Vec<Card>> = self.public_constraints.clone(); 
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
    pub fn public_constraints_mut(&mut self) -> &mut Vec<Vec<Card>> {
        &mut self.public_constraints
    }
    pub fn sort_public_constraints(&mut self) {
        self.public_constraints.iter_mut().for_each(|v| v.sort_unstable());
    }
    pub fn inferred_constraints(&self) -> &Vec<Vec<Card>> {
        &self.inferred_constraints
    }   
    pub fn inferred_constraints_mut(&mut self) -> &mut Vec<Vec<Card>> {
        &mut self.inferred_constraints
    }   
    pub fn sort_inferred_constraints(&mut self) {
        self.inferred_constraints.iter_mut().for_each(|v| v.sort_unstable());
    }
    pub fn set_inferred_constraints(&mut self, inferred_constraints: &Vec<Vec<Card>>) {
        self.inferred_constraints = inferred_constraints.clone();
    }   
    pub fn impossible_constraints(&self) -> &[[bool; 5]; 7] {
        &self.impossible_constraints
    }   
    pub fn impossible_constraints_mut(&mut self) -> &mut [[bool; 5]; 7] {
        &mut self.impossible_constraints
    }   
    pub fn impossible_constraints_2(&self) -> &[[[bool; 5]; 5]; 7] {
        &self.impossible_constraints_2
    }   
    pub fn impossible_constraints_2_mut(&mut self) -> &mut[[[bool; 5]; 5]; 7] {
        &mut self.impossible_constraints_2
    }   
    pub fn impossible_constraints_3(&self) -> &[[[bool; 5]; 5]; 5] {
        &self.impossible_constraints_3
    }   
    pub fn impossible_constraints_3_mut(&mut self) -> &mut [[[bool; 5]; 5]; 5] {
        &mut self.impossible_constraints_3
    }   
    /// Changes stored impossible_constraints
    pub fn set_impossible_constraints(&mut self, impossible_constraints: &[[bool; 5]; 7]) {
        self.impossible_constraints = impossible_constraints.clone();
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
    /// Recalculate current Constraint from scratch using history
    /// Can recursively call itself
    fn regenerate_path(&mut self) {
        log::info!("Regenerating Path");
        self.regenerate_game_start();
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
            ActionInfo::ExchangeDraw { draw } => todo!(),
            ActionInfo::ExchangeChoice { relinquish } => todo!(),
            ActionInfo::ExchangeDrawChoice{ draw, relinquish } => {
                // TODO: handle the known card case obviously lol
                self.ambassador_public(player_id);
            },
        }
        // TODO: generate impossible_constraints can differ by move.
        log::trace!("calculate_stored_move generate_impossible_constraints history_index: {history_index}");
        self.generate_impossible_constraints(history_index);
        self.generate_inferred_constraints();
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
                    None => {
                        self.reveal_redraw_initial(player_id, reveal);
                    },
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
            ActionInfo::ExchangeDraw { draw } => todo!(),
            ActionInfo::ExchangeChoice { relinquish } => todo!(),
            ActionInfo::ExchangeDrawChoice{ draw, relinquish } => {
                // TODO: handle the known card case obviously lol
                self.ambassador_public(player_id);
            },
        }
        self.generate_impossible_constraints(self.history.len() - 1);
        self.generate_inferred_constraints();
        self.history[history_index].meta_data = self.to_meta_data();
        log::info!("recalculated_stored_move_initial: {} {:?}", history_index, self.history[history_index].action_info());
        self.printlog();
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
        output.iter_mut().for_each(|v| v.sort_unstable());
        // for card_vec in output.iter_mut() {
        //     card_vec.sort_unstable()
        // }
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
        self.update_past_move_hidden_info();
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
        self.update_past_move_hidden_info();
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
    pub fn update_past_move_hidden_info(&mut self) -> bool {
        log::trace!("In update_past_move_hidden_info");
        let index = self.history.len() - 1;
        let action_info = self.history[index].action_info().clone();
        let player_index = self.history[index].player();
        let mut bool_changes = false;
        let mut bool_skip_start_update = false;
        match action_info {
            ActionInfo::RevealRedraw { reveal: reveal_considered, redraw: redraw_considered, relinquish } => {
                match redraw_considered {
                    Some(redraw_card) => {
                        unimplemented!("adding of private info not supported yet");
                    },
                    None => {
                        let mut card_assured_players: Vec<u8> = Vec::with_capacity(3);
                        let mut card_assured_players_flags = [false; 6];
                        let mut reveal_players = [false; 6];
                        let mut illegal_to_change = [false; 6];
                        card_assured_players.push(self.history[index].player());
                        card_assured_players_flags[self.history[index].player() as usize] = true;
                        // 2 because first 2 moves are Start and StartInferred
                        for i in (2..index).rev() {
                            log::trace!("RR iter saw: {:?}", action_info);
                            let action_data = &self.history[i];
                            let action_player = self.history[i].player();
                            let action_name = action_data.name();
                            match action_name {
                                ActionInfoName::Discard => {
                                    if let ActionInfo::Discard { discard } = action_data.action_info() {
                                        log::trace!("saw action_player: {action_player} discard: {:?} in iter", discard);
                                        if *discard == reveal_considered {
                                            log::trace!("Pushing action_player: {action_player} into discard_considered");
                                            card_assured_players.push(action_player);
                                            card_assured_players_flags[action_player as usize] = true;
                                        }
                                    }
                                },
                                ActionInfoName::RevealRedraw => {
                                    log::trace!("lookback_initial RevealRedraw checking past RevealRedraw");
                                    log::trace!("lookback_initial original player: {} index: {} RevealRedraw: {:?}", self.history.len() - 1, self.history.len(), action_info);
                                    log::trace!("lookback_initial checked player: {} index: {} RevealRedraw: {:?}", action_data.player(), i, action_data.action_info());

                                    let need_redraw_update = self.need_redraw_update_2(i, reveal_considered, &illegal_to_change);
                                    if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                        if redraw_i.is_none() && *reveal_i != reveal_considered && 
                                        (card_assured_players_flags[action_player as usize] || reveal_players[action_player as usize]){
                                            // Exclude the revealredraw that was just played as its effectively considered a 
                                            // inferred_constraint
                                            illegal_to_change[action_player as usize] = true;
                                            card_assured_players.retain(|p| *p != action_player);
                                            card_assured_players_flags[action_player as usize] = false;
                                            log::trace!("card_assured_players removed: retained not player: {action_player}");
                                            log::trace!("card_assured_players: {:?}",card_assured_players);
                                        }
                                        // Testing
                                        if *redraw_i == Some(reveal_considered) && 
                                        (card_assured_players_flags[action_player as usize] || reveal_players[action_player as usize]){
                                            illegal_to_change[action_player as usize] = true;
                                            card_assured_players.retain(|p| *p != action_player);
                                            card_assured_players_flags[action_player as usize] = false;
                                            log::trace!("card_assured_players removed: retained not player: {action_player}");
                                            log::trace!("card_assured_players: {:?}",card_assured_players);
                                        }
                                        if *reveal_i == reveal_considered && 
                                        *redraw_i != Some(*reveal_i) 
                                        && !card_assured_players_flags[action_player as usize] {
                                            // reveal_players.push(action_player);
                                            reveal_players[action_player as usize] = true;
                                        }
                                    }
                                    log::trace!("need_redraw_update evaluated to {need_redraw_update}");
                                    if need_redraw_update {
                                        log::trace!("lookback_initial original index: {} RevealRedraw: {:?}", self.history.len() - 1, action_info);
                                        log::trace!("lookback_initial considering: index: {} player: {} {:?}", i, action_data.player(), action_data.action_info());
                                        if let ActionInfo::RevealRedraw { reveal, redraw, .. } = self.history[i].action_info_mut() {
                                            log::trace!("lookback_initial setting redraw to: {:?}", reveal_considered);
                                            *redraw = Some(reveal_considered);
                                            // Continuing only if the card had to have come from the pile
                                            bool_changes = true;
                                            if action_player == player_index {
                                                if Some(*reveal) != *redraw {
                                                    self.lookback_1_continual(i);
                                                }
                                                bool_skip_start_update = true;
                                            }

                                        }
                                    } else if action_player == player_index {
                                        let mut need_relinquish_update = false;
                                        if let ActionInfo::RevealRedraw { redraw, ..} = self.history[i].action_info() {
                                            if redraw.is_none() {
                                                need_relinquish_update = self.need_relinquish_update(i, reveal_considered);
                                            }
                                        }
                                        log::trace!("need_relinquish_update evaluated to {need_relinquish_update}");
                                        log::trace!("lookback_initial original index: {} RevealRedraw: {:?}", self.history.len() - 1, action_info);
                                        log::trace!("lookback_initial considering: index: {} player: {} {:?}", i, action_data.player(), action_data.action_info());
                                        if let ActionInfo::RevealRedraw { reveal, redraw, relinquish} = self.history[i].action_info_mut() {
                                            if need_relinquish_update {
                                                *relinquish = Some(*reveal);
                                                bool_changes = true;
                                            }
                                            // Avoid updating start when loop ends
                                            bool_skip_start_update = true;

                                        }
                                    }
                                },
                                ActionInfoName::ExchangeDrawChoice => {
                                },
                                _ => {}
                            }
                        }
                        debug_assert!(self.history[0].name() == ActionInfoName::Start, "wrong Significant Action at index 0!");
                        if !bool_skip_start_update {
                            log::trace!("lookback_initial index: {:?}", index);
                            log::trace!("lookback_initial processed item: {:?}", self.history.last().unwrap().action_info());
                            log::trace!("lookback_initial adding inferred_constraint to start: (player: {}, card: {:?})", player_index, reveal_considered);
                            log::trace!("lookback_initial start before: {:?}", self.history.first().unwrap().meta_data());
                            self.history[0].add_inferred_constraints(player_index as usize, reveal_considered);
                            log::trace!("lookback_initial start after: {:?}", self.history.first().unwrap().meta_data());
                            bool_changes = true;
                        }
                    },
                }
            },
            ActionInfo::Discard { discard: discard_considered } => {
                let mut card_assured_players: Vec<u8> = Vec::with_capacity(3);
                let mut card_assured_players_flags = [false; 6];
                card_assured_players_flags[self.history[index].player() as usize] = true;
                // TEST UNSURE how this interacts with AMB
                let mut reveal_players = [false; 6];
                // Add here as we don't loop over current index
                // May expand to all cards known later
                let mut illegal_to_change = [false; 6];
                card_assured_players.push(self.history[index].player());
                for i in (2..index).rev() {
                    let action_data = &self.history[i];
                    log::trace!("iter saw action_data: {:?}", action_data);
                    let action_player = action_data.player();
                    let action_name = action_data.name();
                    match action_name {
                        ActionInfoName::Discard => {
                            // Collecting same card discards found along the way
                            // Discard is same for all player index and for initial card added
                            if let ActionInfo::Discard { discard } = action_data.action_info() {
                                log::trace!("saw action_player: {action_player} discard: {:?} in iter", discard);
                                if *discard == discard_considered {
                                    log::trace!("Pushing action_player: {action_player} into discard_considered");
                                    card_assured_players.push(action_player);
                                    card_assured_players_flags[action_player as usize] = true;
                                }
                            }
                        },
                        ActionInfoName::RevealRedraw => {
                            log::trace!("lookback_initial RevealRedraw checking past RevealRedraw");
                            log::trace!("lookback_initial original player: {} index: {} RevealRedraw: {:?}", self.history.len() - 1, self.history.len(), action_info);
                            log::trace!("lookback_initial checked player: {} index: {} RevealRedraw: {:?}", action_data.player(), i, action_data.action_info());

                            let need_redraw_update = self.need_redraw_update_2(i, discard_considered, &illegal_to_change);
                            if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                if redraw_i.is_none() && *reveal_i != discard_considered && 
                                (card_assured_players_flags[action_player as usize] || reveal_players[action_player as usize]){
                                    // Exclude the revealredraw that was just played as its effectively considered a 
                                    illegal_to_change[action_player as usize] = true;
                                    card_assured_players.retain(|p| *p != action_player);
                                    card_assured_players_flags[action_player as usize] = false;
                                    log::trace!("card_assured_players removed: retained not player: {action_player}");
                                    log::trace!("card_assured_players: {:?}",card_assured_players);
                                }
                                // Testing
                                if *redraw_i == Some(discard_considered) && 
                                (card_assured_players_flags[action_player as usize] || reveal_players[action_player as usize]){
                                    illegal_to_change[action_player as usize] = true;
                                    card_assured_players.retain(|p| *p != action_player);
                                    card_assured_players_flags[action_player as usize] = false;
                                    log::trace!("card_assured_players removed: retained not player: {action_player}");
                                    log::trace!("card_assured_players: {:?}",card_assured_players);
                                }
                                if *reveal_i == discard_considered && 
                                *redraw_i != Some(*reveal_i) 
                                && !card_assured_players_flags[action_player as usize] {
                                    reveal_players[action_player as usize] = true;
                                }
                            }
                            log::trace!("need_redraw_update evaluated to {need_redraw_update}");
                            if need_redraw_update {
                                log::trace!("lookback_initial original index: {} RevealRedraw: {:?}", self.history.len() - 1, action_info);
                                log::trace!("lookback_initial considering: index: {} player: {} {:?}", i, action_data.player(), action_data.action_info());
                                if let ActionInfo::RevealRedraw { reveal, redraw, .. } = self.history[i].action_info_mut() {
                                    log::trace!("lookback_initial setting redraw to: {:?}", discard_considered);
                                    *redraw = Some(discard_considered);
                                    // Continuing only if the card had to have come from the pile
                                    bool_changes = true;
                                    if action_player == player_index {
                                        if Some(*reveal) != *redraw {
                                            self.lookback_1_continual(i);
                                        }
                                        bool_skip_start_update = true;
                                    }

                                }
                            } else if action_player == player_index {
                                let mut need_relinquish_update = false;
                                if let ActionInfo::RevealRedraw { redraw, ..} = self.history[i].action_info() {
                                    if redraw.is_none() {
                                        need_relinquish_update = self.need_relinquish_update(i, discard_considered);
                                    }
                                }
                                log::trace!("need_relinquish_update evaluated to {need_relinquish_update}");
                                log::trace!("lookback_initial original index: {} RevealRedraw: {:?}", self.history.len() - 1, action_info);
                                log::trace!("lookback_initial considering: index: {} player: {} {:?}", i, action_data.player(), action_data.action_info());
                                if let ActionInfo::RevealRedraw { reveal, redraw, relinquish} = self.history[i].action_info_mut() {
                                    if need_relinquish_update {
                                        *relinquish = Some(*reveal);
                                        bool_changes = true;
                                    }
                                    // Avoid updating start when loop ends
                                    bool_skip_start_update = true;
                                }
                            }
                        },
                        ActionInfoName::ExchangeDrawChoice => {
                        },
                        _ => {},
                    }
                }
                debug_assert!(self.history[0].name() == ActionInfoName::Start, "wrong Significant Action at index 0!");
                if !bool_skip_start_update {
                    log::trace!("lookback_initial processed item: {:?}", self.history.last().unwrap().action_info());
                    log::trace!("lookback_initial adding inferred_constraint to start: (player: {}, card: {:?})", player_index, discard_considered);
                    log::trace!("lookback_initial start before: {:?}", self.history.first().unwrap().meta_data());
                    self.history[0].add_inferred_constraints(player_index as usize, discard_considered);
                    log::trace!("lookback_initial start after: {:?}", self.history.first().unwrap().meta_data());
                    bool_changes = true;
                }
            },
            _ => {}
        }
        if bool_changes {
            self.regenerate_path();
            log::trace!("End Regenerate Path lookback_initial D");
            self.printlog();
        }
        bool_changes
    }
    /// lookback for when we discover a previous item and want to continue for that
    /// This happens e.g. when you discard => infer the redraw of a previous RevealRedraw
    /// and you need to update those before the redraw
    ///     - Start => when we know their starting inferred_constraint
    ///     - RevealRedraw => when we know what the left in the pile
    /// so you call this for the previous RevealRedraw
    /// Assumes some inferred information has already been learnt before this.
    /// => Assumes that regenerate will be called outside of this regardless
    pub fn lookback_1_continual(&mut self, index: usize) -> bool {
        // index is the index for history
        log::trace!("In lookback_1_continual index: {index}");
        log::trace!("index: {:?}", self.history[index].action_info());
        self.printlog();
        log::trace!("Start: {:?}", self.history[0].meta_data());
        let action_info = self.history[index].action_info().clone();
        match action_info {
            ActionInfo::RevealRedraw{reveal: reveal_considered, redraw: redraw_considered, ..} => {
                // TODO: [OPTIMIZE] Consider just 1 check instead of this merge
                match redraw_considered {
                    Some(redraw_card) => {
                        // === CASE UPDATE START ===
                        // reveal_considered != redraw_considered
                        //  - Redraw card implies that the pile had the card before it was given to the player
                        //  - Update the starting node if we can tell the pile had the card at start
                        //      - Don't need to update the inferred of a none starting node as
                        //        we do not need any intermediate accuracy, and we know that it will be reflected
                        //        properly in the latest inferred constraints anyways
                        //  - Conditions where pile card was in starting node
                        //      - All Previous RevealRedraw, reveal != redraw_considered
                        //          - reveal == redraw_considered => would mean the maybe the player put it in
                        //      - All Previous RevealRedraw redraw card does not matter
                        //          - reveal == redraw => redraw_considered would still be in pile
                        //          - reveal != redraw => card was redrawn to player, but redraw_considered would still be in pile
                        //      - No Previous Ambassador where no cards known
                        //          - unable to infer further as reveal_considered could have been transferred from player to pile
                        //      - More Ambassador
                        //          - [TODO] unhandled
                        // reveal_considered == redraw_considered
                        //  - Redraw card may come from pile or may have been same card player put in
                        for i in (2..index).rev() {
                            match self.history[i].name() {
                                ActionInfoName::RevealRedraw => {
                                    if let ActionInfo::RevealRedraw { reveal, redraw, relinquish } = self.history[i].action_info() {
                                        // CASE player put the considered card into the pile
                                        // If all cards for redraw are known
                                        // If current player has RevealRedrawn before they must have withdrawn the Discarded card
                                        // 
                                        // CASE Player RevealRedraw same card
                                        // - did not change the pile state => no early exit
                                        if Some(*reveal) == *redraw {
                                            continue;
                                        } else if Some(*reveal) == redraw_considered {
                                            // Testing this
                                            // CASE when a player RevealRedraw the card and it was not redrawn
                                            // Could be in pile or player
                                            // TODO: Maybe need to handle relinquish case
                                            return false;
                                        }
                                    }
                                    let bool_change_relinquish = self.history[i - 1].known_card_count(redraw_card) == 2 
                                    && !self.history[i - 1].inferred_constraints()[self.history[i].player() as usize].contains(&redraw_card);
                                    log::trace!("lookback_1_continual bool_change_relinquish: {bool_change_relinquish}");
                                    log::trace!("self.history[i - 1].inferred_constraints: {:?}", self.history[i - 1].inferred_constraints());
                                    log::trace!("self.history[i - 1].known_card_count(redraw_card) == 2: {:?}", self.history[i - 1].known_card_count(redraw_card) == 2);
                                    log::trace!("!self.history[i - 1].inferred_constraints()[self.history[i].player() as usize].contains(&redraw_card): {:?}", !self.history[i - 1].inferred_constraints()[self.history[i].player() as usize].contains(&redraw_card));
                                    if bool_change_relinquish {
                                        log::trace!("lookback_1_continual original RevealRedraw: {:?}", action_info);
                                        log::trace!("lookback_1_continual considering: player: {} {:?}", self.history[i].player(), self.history[i].action_info());
                                        if let ActionInfo::RevealRedraw { reveal, relinquish, .. } = self.history[i].action_info_mut() {
                                            // CASE Player RevealRedraw revealed the card of interest and did not redraw it back
                                            // - pile got the redraw_card here => early exit
                                            if *reveal == redraw_card {
                                                // CASE player put the considered card into the pile
                                                // All card locations for redraw_card are known just after previous reveal
                                                // If current player has RevealRedrawn has redrawn redraw_card
                                                // Then last RevealRedraw must have left it inside
                                                // I need to know that player has the third card just after reveal, and before redraw
                                                if let Some(relinquish_card) = relinquish {
                                                    log::trace!("lookback1_continual set relinquish_card to: {:?}", *reveal);
                                                    *relinquish_card = *reveal;
                                                    self.regenerate_path();
                                                    log::trace!("End Regenerate Path lookback_continual");
                                                    self.printlog();
                                                    return true
                                                }
                                            }
                                            return false
                                        }
                                    }
                                },
                                ActionInfoName::ExchangeDrawChoice => {
                                    if let ActionInfo::ExchangeDrawChoice { draw, relinquish } = self.history[i].action_info() {
                                        if !draw.is_empty() || !relinquish.is_empty() {
                                            // TODO: [CASE] Technically there could be cases to handle where cards are known
                                            // For now we stop at any Ambassador
                                            return false
                                        }
                                    }
                                },
                                ActionInfoName::Start
                                |ActionInfoName::StartInferred => {
                                    debug_assert!(false, "Should not have Start when index is not 0");
                                },
                                ActionInfoName::ExchangeDraw => todo!(),
                                ActionInfoName::ExchangeChoice => todo!(),
                                ActionInfoName::Discard => {},
                            }
                        }
                        // Add to start
                        log::trace!("lookback1_continual adding to Start player_id: 6, card: {:?}", redraw_card);
                        debug_assert!(self.history[0].name() == ActionInfoName::Start, "wrong Significant Action at index 0!");
                        // TODO: HMMMMMMMM, i feel like there may be cases where it could have 2?
                        if !self.history[0].inferred_constraints()[6].contains(&redraw_card) {
                            self.history[0].add_inferred_constraints(6, redraw_card);
                        }
                        return true;
                    },
                    None => {
                        debug_assert!(false, "You should not be calling this on a RevealRedraw without a redraw card known");
                    }
                }
            },
            ActionInfo::Discard{discard: discard_considered} => {
                debug_assert!(false, "You should not be calling this on a discard card");
            },
            ActionInfo::Start
            | ActionInfo::StartInferred => {
                panic!("You should not be calling this on Start");
            },
            ActionInfo::ExchangeDraw { .. }
            | ActionInfo::ExchangeChoice { .. }
            | ActionInfo::ExchangeDrawChoice { .. } => {
                // unimplemented!();
            },
        }
        false
    }
    pub fn need_redraw_update_2(&self, index: usize, card_of_interest: Card, illegal_players: &[bool; 6]) -> bool {
        log::trace!("need_redraw_update_2 considered: {index}, card_of_interest: {:?}, illegal_players: {:?}", card_of_interest, illegal_players);
        log::trace!("need_redraw_update_2 considered: {index}, player: {:?} move {:?}", self.history[index].player(), self.history[index].action_info());
        // TODO: OPTIMIZE
        let mut players_had_revealed_or_discarded = [false; 6];
        // Prob can optimize to use u8s
        for i in (index+1..self.history.len()).rev() {
            let player_i = self.history[i].player() as usize;
            log::trace!("backward_pass checking i: {i} player_i: {player_i}, move: {:?}", self.history[i].action_info());
            match self.history[i].action_info() {
                ActionInfo::Discard { discard: discard_i } => {
                    if *discard_i == card_of_interest {
                        players_had_revealed_or_discarded[player_i] = true;
                    }
                },
                ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, relinquish } => {
                    // Maybe it exceeds maximum because I need to consider the pile for revealredraw redraw is known
                    if *reveal_i == card_of_interest {
                        // Reflecting that the player had this card before move i
                        players_had_revealed_or_discarded[player_i] = true;
                    }
                },
                ActionInfo::ExchangeDrawChoice { .. } => {
                    players_had_revealed_or_discarded[player_i] = false;
                },
                ActionInfo::ExchangeDraw { .. } => todo!(),
                ActionInfo::ExchangeChoice { .. } => todo!(),
                ActionInfo::Start => {},
                ActionInfo::StartInferred => {},
            }
        }
        let action_player = self.history[index].player() as usize;
        log::trace!("need_redraw_update for player {:?}, move: {:?}", self.history[self.history.len() - 1].player(), self.history[self.history.len() - 1].action_info());
        log::trace!("!illegal_players[action_player as usize]: {:?}", !illegal_players[action_player as usize]);
        log::trace!("players_had_revealed_or_discarded: {:?}", players_had_revealed_or_discarded);
        log::trace!("redraw_i.is_none(): {:?}", self.history[index].action_info());
        // TODO: OPTIMIZE obviously don't put this here
        let bool_2_cards_outside_player = if let ActionInfo::RevealRedraw { reveal, redraw , .. }= self.history[index].action_info() {
            if *reveal == card_of_interest && redraw.is_none() {
                let mut cards: [u8; 5] = [0; 5];
                cards[card_of_interest as usize] = 2;
                self.impossible_to_have_cards(index, action_player, &cards)
            } else {
                false
            }
        } else {
            false
        };
        log::trace!("bool_2_cards_outside_player returned: {}", bool_2_cards_outside_player);
        log::trace!("lookback_check_3_fwd_bwd_pass: {}", false);
        if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = self.history[index].action_info() {

            log::trace!("need_redraw_update: redraw_i.is_none() {}", redraw_i.is_none());
            
        }
        log::trace!("need_redraw_update: illegal_players {:?}", illegal_players);
        log::trace!("need_redraw_update: !illegal_players[action_player] {}", !illegal_players[action_player]);
        log::trace!("need_redraw_update: players_had_revealed_or_discarded {:?}", players_had_revealed_or_discarded);
        log::trace!("need_redraw_update: players_had_revealed_or_discarded[action_player] {}", players_had_revealed_or_discarded[action_player]);
        log::trace!("need_redraw_update: self.history[index - 1].public_constraints()[action_player] {:?}", self.history[index - 1].public_constraints()[action_player]);
        log::trace!("need_redraw_update: self.history[index - 1].public_constraints()[action_player].len() == 1 {}", self.history[index - 1].public_constraints()[action_player].len() == 1);
        
        if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = self.history[index].action_info() {
            redraw_i.is_none() 
            // Testing the removal of
            && !illegal_players[action_player] // skip if player RR after too (dk which revealredraw the player redrew)
            && players_had_revealed_or_discarded[action_player] // checks if player did reveal/redraw that card before
            && (
                // if you had 1 live, you must have withdrawn the card you Reveal/Discard later on
                self.history[index - 1].impossible_constraints()[action_player][card_of_interest as usize]
                || self.history[index - 1].public_constraints()[action_player].len() == 1
                || self.history[self.history.len() - 1].public_constraints().iter().map(|v| v.iter().filter(|c| **c == card_of_interest).count() as u8).sum::<u8>() == 3
                || (
                    // Testing player revealed last card and and same as dead card
                    // maybe action_player needs to be latest player
                    {
                        log::trace!("self.history[self.history.len() - 1].public_constraints():{:?}", self.history[self.history.len() - 2].public_constraints());
                        log::trace!("self.history[self.history.len() - 1].public_constraints()[action_player].len() == 1 = :{}", self.history[self.history.len() - 1].public_constraints()[action_player].len() == 1);
                        log::trace!("&& self.history[self.history.len() - 1].public_constraints()[action_player][0] == card_of_interest = :{}", self.history[self.history.len() - 1].public_constraints()[action_player].len() == 1 && self.history[self.history.len() - 1].public_constraints()[action_player][0] == card_of_interest);
                        action_player == self.history[self.history.len() - 1].player() as usize
                        && self.history[self.history.len() - 2].public_constraints()[action_player].len() == 1
                        && self.history[self.history.len() - 2].public_constraints()[action_player][0] == card_of_interest
                    }
                )
                // I like this
                || (
                    *reveal_i == card_of_interest &&
                    // had to have withdrawn if other 2 cards are outside player
                    self.history[index - 1].public_constraints().iter().enumerate().filter(|(p, _)| *p != action_player).map(|(_, v)| v.iter().filter(|c| **c == card_of_interest).count()).sum::<usize>()
                    + self.history[index - 1].inferred_constraints().iter().enumerate().filter(|(p, _)| *p != action_player).map(|(_, v)| v.iter().filter(|c| **c == card_of_interest).count()).sum::<usize>()
                    == 2
                )
                || (
                    // Player cannot have card
                    *reveal_i != card_of_interest 
                    && redraw_i.is_none()
                    && {
                        let mut cards: [u8; 5] = [0; 5];
                        cards[card_of_interest as usize] = 1;
                        let output = self.impossible_to_have_cards(index, action_player, &cards);
                        log::trace!("need_redraw_update_2 considered: {index}, card_of_interest: {:?}, illegal_players: {:?}", card_of_interest, illegal_players);
                        log::trace!("need_redraw_update_2 considered: {index}, player: {:?} move {:?}", self.history[index].player(), self.history[index].action_info());
                        log::trace!("need_redraw_update_2 player cannot have card evaluated to : {}", output);
                        output
                    }
                )
                || bool_2_cards_outside_player
                || false // *reveal != card_of_interest && 3 cards outside player
            )
        } else {
            false
        }
    }
    /// Returns true if relinquish needs to be updated
    /// Assumes index is not last index
    pub fn need_relinquish_update(&self, index: usize, card_of_interest: Card) -> bool {
        let action_player = self.history[index].player() as usize;
        let latest_player = self.history[self.history.len() - 1].player() as usize;
        if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = self.history[index].action_info() {
            if *reveal_i != card_of_interest 
            && redraw_i.is_none() 
            && action_player == latest_player
            {
                log::trace!("need_relinquish_update *reveal_i != card_of_interest: {}", *reveal_i != card_of_interest);
                log::trace!("need_relinquish_update redraw_i.is_none(): {}", redraw_i.is_none());
                log::trace!("need_relinquish_update action_player == latest_player: {}", action_player == latest_player);
                let mut cards: [u8; 5] = [0; 5];
                cards[*reveal_i as usize] = 1;
                // Cannot have card after
                let output = self.impossible_to_have_cards(index + 1, action_player, &cards);
                log::trace!("need_relinquish_update self.impossible_to_have_cards: {}", output);
                return output
            }
        }
        false
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
        !self.possible_to_have_cards_recurse(index_lookback, 999999999999, player_of_interest, &mut public_constraints, &mut inferred_constraints, &[0; 5])
        // !self.possible_to_have_cards_recurse(index_lookback - 1, index, player_of_interest, &mut public_constraints, &mut inferred_constraints, cards)
    }
    /// Does Backtracking to determine if at a particular point that particular player could not have had some set of cards at start of turn
    /// Assuming we won't be using this for ambassador?
    pub fn impossible_to_have_cards(&self, index: usize, player_of_interest: usize, cards: &[u8; 5]) -> bool {
        log::trace!("impossible_to_have_cards index: {}, player_of_interest: {}, cards: {:?}", index, player_of_interest, cards);
        debug_assert!(player_of_interest != 6 && cards.iter().sum::<u8>() <= 2 || player_of_interest == 6 && cards.iter().sum::<u8>() <= 3, "cards too long!");
        let mut public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(4); 7];
        let mut inferred_constraints: Vec<Vec<Card>> = public_constraints.clone();
        let latest_move = self.history.last().unwrap(); 
        match latest_move.action_info() {
            ActionInfo::Discard { discard } => {
                inferred_constraints[latest_move.player() as usize].push(*discard);
            },
            ActionInfo::RevealRedraw { reveal, redraw, .. } => {
                inferred_constraints[latest_move.player() as usize].push(*reveal);
                redraw.map(|pile_original_card| inferred_constraints[6].push(pile_original_card));
            },
            ActionInfo::ExchangeDrawChoice { draw, relinquish } => {
                inferred_constraints[latest_move.player() as usize].extend(relinquish.iter().copied());
                inferred_constraints[6].extend(draw.iter().copied());
            },
            ActionInfo::ExchangeDraw { .. } => todo!(),
            ActionInfo::ExchangeChoice { .. } => todo!(),
            ActionInfo::Start
            | ActionInfo::StartInferred => {},
        }

        // Do an unwrap_or false
        !self.possible_to_have_cards_recurse(self.history.len() - 2, index, player_of_interest, &mut public_constraints, &mut inferred_constraints, cards)
    }
    /// returns false if possible
    /// TODO: Consider passing in array to count inferred_so_far
    /// Traces the game tree in reverse (from latest move to earliest move) by backtracking
    /// Tracks possible paths known cards could have come from in the past
    /// If a state is found to satisfy cards at the index_of_interest return Some(true)
    /// If no state is every found return Some(false) or None
    /// Assume cards should be sorted before use
    pub fn possible_to_have_cards_recurse(&self, index_loop: usize, index_of_interest: usize, player_of_interest: usize, public_constraints: &mut Vec<Vec<Card>>, inferred_constraints: &mut Vec<Vec<Card>>, cards: &[u8; 5]) -> bool {
        // Will temporarily not use memo and generate all group_constraints from start
        // Needed for checks
        log::trace!("After");
        log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
        log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.history[index_loop].public_constraints(), inferred_constraints);
        if !self.is_valid_combination(index_loop, index_of_interest, inferred_constraints) {
            // early exit before terminal node
            log::trace!("is_valid_combination evaluated to false");
            return false
        }
        log::trace!("is_valid_combination evaluated to true");
        let mut current_card_counts: [u8; 5] = [0; 5];
        if index_loop == index_of_interest - 1 {
            // TODO: Terminal node
            // Check if possible to have cards
            for card in inferred_constraints[player_of_interest].iter() {
                current_card_counts[*card as usize] += 1;
            }
            for (card_num, (query_card_count, current_card_count)) in cards.iter().zip(current_card_counts.iter()).enumerate() {
                if *query_card_count > *current_card_count {
                    for _ in 0..(*query_card_count - *current_card_count) {
                        inferred_constraints[player_of_interest].push(Card::try_from(card_num as u8).unwrap());
                    }
                }
            }
            log::trace!("possible_to_have_cards_recurse at index_of_interest - 1, player_of_interest: {player_of_interest} added :{:?} to inferred_constraints to become: {:?}", cards, inferred_constraints);
            // Checking if valid as its possible to have inferred_constraint burst its limits only to get cards removed in code later, when handling the move
            let fulfilled: bool = self.is_valid_combination(index_loop, index_of_interest, inferred_constraints);
            // This is wrong, cause I need to run through the rest first before removing this
            if !fulfilled {
                for (card_num, (query_card_count, current_card_count)) in cards.iter().zip(current_card_counts.iter()).enumerate() {
                    if *query_card_count > *current_card_count {
                        for _ in 0..(*query_card_count - *current_card_count) {
                            if let Some(pos) = inferred_constraints[player_of_interest].iter().rposition(|c| *c as usize == card_num) {
                                inferred_constraints[player_of_interest].swap_remove(pos);
                            }
                        }
                    }
                }
                return false;
            }
        }
        let player_loop = self.history[index_loop].player() as usize;
        let mut response = false;
        match self.history[index_loop].action_info() {
            ActionInfo::Discard { discard } => {
                log::trace!("Before Discard");
                log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.history[index_loop].public_constraints(), inferred_constraints);
                let mut removed_discard = false;
                if let Some(pos) = public_constraints[player_loop].iter().rposition(|c| *c == *discard) {
                    public_constraints.swap_remove(pos);
                    removed_discard = true;
                }
                inferred_constraints[player_loop].push(*discard);
                // recurse
                response = self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards);
                if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *discard) {
                    inferred_constraints[player_loop].swap_remove(pos);
                }
                if removed_discard {
                    public_constraints[player_loop].push(*discard);
                }
                if index_loop == index_of_interest - 1 {
                    for (card_num, (query_card_count, current_card_count)) in cards.iter().zip(current_card_counts.iter()).enumerate() {
                        if *query_card_count > *current_card_count {
                            for _ in 0..(*query_card_count - *current_card_count) {
                                if let Some(pos) = inferred_constraints[player_of_interest].iter().rposition(|c| *c as usize == card_num) {
                                    inferred_constraints[player_of_interest].swap_remove(pos);
                                }
                            }
                        }
                    }
                }
                return response;
            },
            ActionInfo::RevealRedraw { reveal, redraw, relinquish } => {
                // Check if will burst before pushing
                match redraw {
                    Some(redraw_i) => {
                        log::trace!("Before Reveal Redraw");
                        log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
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
                        response = self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards);
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
                                    if index_loop == index_of_interest - 1 {
                                        for (card_num, (query_card_count, current_card_count)) in cards.iter().zip(current_card_counts.iter()).enumerate() {
                                            if *query_card_count > *current_card_count {
                                                for _ in 0..(*query_card_count - *current_card_count) {
                                                    if let Some(pos) = inferred_constraints[player_of_interest].iter().rposition(|c| *c as usize == card_num) {
                                                        inferred_constraints[player_of_interest].swap_remove(pos);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    return false;
                                }
                                log::trace!("Before Reveal Relinquish");
                                log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
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
                                        response = self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards);
                                    }
                                    if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *reveal) {
                                        inferred_constraints[player_loop].swap_remove(pos);
                                    }
                                    if removed_reveal {
                                        inferred_constraints[6].push(*reveal);
                                    }
                                    if index_loop == index_of_interest - 1 {
                                        for (card_num, (query_card_count, current_card_count)) in cards.iter().zip(current_card_counts.iter()).enumerate() {
                                            if *query_card_count > *current_card_count {
                                                for _ in 0..(*query_card_count - *current_card_count) {
                                                    if let Some(pos) = inferred_constraints[player_of_interest].iter().rposition(|c| *c as usize == card_num) {
                                                        inferred_constraints[player_of_interest].swap_remove(pos);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    return response;
                                }
                                let mut iter_cards = inferred_constraints[player_loop].clone();
                                iter_cards.sort_unstable();
                                iter_cards.dedup();
                                for (i, card_player) in iter_cards.iter().enumerate() {
                                    // Card Source was not from Pile
                                    log::trace!("Before Reveal Relinquish B");
                                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                    log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.history[index_loop].public_constraints(), inferred_constraints);
                
                                    if inferred_constraints[player_loop].len() < 2 {
                                        let mut bool_move_from_pile_to_player = false;
                                        if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *reveal) {
                                            inferred_constraints[6].swap_remove(pos);
                                            bool_move_from_pile_to_player = true;
                                        }
                                        inferred_constraints[player_loop].push(*reveal);
                                        
                                        if inferred_constraints.iter().map(|v| v.iter().filter(|c| **c == *reveal).count() as u8).sum::<u8>() < 4{
                                            response = self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards);
                                        }
                                        
                                        if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *reveal) {
                                            inferred_constraints[player_loop].swap_remove(pos);
                                        }
                                        if bool_move_from_pile_to_player {
                                            inferred_constraints[6].push(*reveal);
                                        }
                                        if response {
                                            if index_loop == index_of_interest - 1 {
                                                for (card_num, (query_card_count, current_card_count)) in cards.iter().zip(current_card_counts.iter()).enumerate() {
                                                    if *query_card_count > *current_card_count {
                                                        for _ in 0..(*query_card_count - *current_card_count) {
                                                            if let Some(pos) = inferred_constraints[player_of_interest].iter().rposition(|c| *c as usize == card_num) {
                                                                inferred_constraints[player_of_interest].swap_remove(pos);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            return true;
                                        }
                                    }
                                    // Card Source was from Pile
                                    if *card_player != *reveal {
                                        log::trace!("Before Reveal Relinquish C");
                                        log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
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
                                        response = self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards);

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
                                            if index_loop == index_of_interest - 1 {
                                                for (card_num, (query_card_count, current_card_count)) in cards.iter().zip(current_card_counts.iter()).enumerate() {
                                                    if *query_card_count > *current_card_count {
                                                        for _ in 0..(*query_card_count - *current_card_count) {
                                                            if let Some(pos) = inferred_constraints[player_of_interest].iter().rposition(|c| *c as usize == card_num) {
                                                                inferred_constraints[player_of_interest].swap_remove(pos);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
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
                                    if index_loop == index_of_interest - 1 {
                                        for (card_num, (query_card_count, current_card_count)) in cards.iter().zip(current_card_counts.iter()).enumerate() {
                                            if *query_card_count > *current_card_count {
                                                for _ in 0..(*query_card_count - *current_card_count) {
                                                    if let Some(pos) = inferred_constraints[player_of_interest].iter().rposition(|c| *c as usize == card_num) {
                                                        inferred_constraints[player_of_interest].swap_remove(pos);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    return false;
                                }
 
                                if inferred_constraints[player_loop].is_empty() {
                                    log::trace!("Before Reveal Redraw None");
                                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                    log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.history[index_loop].public_constraints(), inferred_constraints);
                
                                    let mut bool_move_from_pile_to_player = false;
                                    if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *reveal) {
                                        inferred_constraints[6].swap_remove(pos);
                                        bool_move_from_pile_to_player = true;
                                    }
                                    inferred_constraints[player_loop].push(*reveal);

                                    if inferred_constraints[player_loop].len() < 3
                                    && inferred_constraints.iter().map(|v| v.iter().filter(|c| **c == *reveal).count() as u8).sum::<u8>() < 4{
                                        // TODO: Recurse here in other version
                                        response = self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards);
                                    }
                                    // return variants;
                                    // TODO: remove if recursing
                                    if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *reveal) {
                                        inferred_constraints[player_loop].swap_remove(pos);
                                    }
                                    if bool_move_from_pile_to_player {
                                        inferred_constraints[6].push(*reveal);
                                    }
                                    // Add query card thing
                                    // if response {
                                        if index_loop == index_of_interest - 1 {
                                            for (card_num, (query_card_count, current_card_count)) in cards.iter().zip(current_card_counts.iter()).enumerate() {
                                                if *query_card_count > *current_card_count {
                                                    for _ in 0..(*query_card_count - *current_card_count) {
                                                        if let Some(pos) = inferred_constraints[player_of_interest].iter().rposition(|c| *c as usize == card_num) {
                                                            inferred_constraints[player_of_interest].swap_remove(pos);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    // }
                                    return response;
                                }
                                let mut iter_cards = inferred_constraints[player_loop].clone();
                                iter_cards.sort_unstable();
                                iter_cards.dedup();
                                // Doesnt handle empty case
                                for (_, card_player) in iter_cards.iter().enumerate() {
                                    log::trace!("Before Reveal Redraw None B");
                                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
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
                                            response = self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards);
                                        }

                                        if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *reveal) {
                                            inferred_constraints[player_loop].swap_remove(pos);
                                        }
                                        if bool_move_from_pile_to_player {
                                            inferred_constraints[6].push(*reveal);
                                        }
                                        if response {
                                            if index_loop == index_of_interest - 1 {
                                                for (card_num, (query_card_count, current_card_count)) in cards.iter().zip(current_card_counts.iter()).enumerate() {
                                                    if *query_card_count > *current_card_count {
                                                        for _ in 0..(*query_card_count - *current_card_count) {
                                                            if let Some(pos) = inferred_constraints[player_of_interest].iter().rposition(|c| *c as usize == card_num) {
                                                                inferred_constraints[player_of_interest].swap_remove(pos);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            return true;
                                        }
                                    }
                                    log::trace!("Before Reveal Redraw None C");
                                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
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
                                        response = self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards);
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
                                        if index_loop == index_of_interest - 1 {
                                            for (card_num, (query_card_count, current_card_count)) in cards.iter().zip(current_card_counts.iter()).enumerate() {
                                                if *query_card_count > *current_card_count {
                                                    for _ in 0..(*query_card_count - *current_card_count) {
                                                        if let Some(pos) = inferred_constraints[player_of_interest].iter().rposition(|c| *c as usize == card_num) {
                                                            inferred_constraints[player_of_interest].swap_remove(pos);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        return true
                                    }
                                }
                                // NOTE: This below actually doenst work lmao
                                // let mut variants: Vec<Vec<Vec<Card>>> = Self::return_variants_reveal_redraw_none(*reveal, player_loop, &inferred_constraints);
                                // for inferred_i in variants.iter_mut() {
                                //     log::trace!("Before Reveal Redraw None");
                                //     log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                //     log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
                
                                //     let response = self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_i, cards);
                                //     if response {
                                //         if index_loop == index_of_interest - 1 {
                                //             for (card_num, (query_card_count, current_card_count)) in cards.iter().zip(current_card_counts.iter()).enumerate() {
                                //                 if *query_card_count > *current_card_count {
                                //                     for _ in 0..(*query_card_count - *current_card_count) {
                                //                         if let Some(pos) = inferred_constraints[player_of_interest].iter().rposition(|c| *c as usize == card_num) {
                                //                             inferred_constraints[player_of_interest].swap_remove(pos);
                                //                         }
                                //                     }
                                //                 }
                                //             }
                                //         }
                                //         return response;
                                //     }
                                // }
                            },
                        }
                    },
                }
            },
            ActionInfo::ExchangeDrawChoice { .. } => {
                response = self.recurse_variants_exchange(index_loop, index_of_interest, player_of_interest, public_constraints, inferred_constraints, player_loop, cards);
            },
            ActionInfo::ExchangeDraw { .. } => todo!(),
            ActionInfo::ExchangeChoice { .. } => todo!(),
            ActionInfo::Start
            | ActionInfo::StartInferred => {
                // Managed to reach base
                log::trace!("possible_to_have_cards_recurse found true at index: {}", index_loop);
                response = true;
            },
        }
        // Did not find a valid combination
        if index_loop == index_of_interest - 1 {
            for (card_num, (query_card_count, current_card_count)) in cards.iter().zip(current_card_counts.iter()).enumerate() {
                if *query_card_count > *current_card_count {
                    for _ in 0..(*query_card_count - *current_card_count) {
                        if let Some(pos) = inferred_constraints[player_of_interest].iter().rposition(|c| *c as usize == card_num) {
                            inferred_constraints[player_of_interest].swap_remove(pos);
                        }
                    }
                }
            }
        }
        response
    }
    /// Return true if hypothesised card permutations cannot be shown to be impossible
    pub fn is_valid_combination(&self, index_loop: usize ,index_of_interest: usize, inferred_constraints: &Vec<Vec<Card>>) -> bool {
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
        // TODO: OPTIMIZE Do we really need this?
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
        return true;
        // TEST END
        let action_info = &self.history[index_loop];
        let action_public_constraint = action_info.public_constraints();
        let action_inferred_constraint = action_info.inferred_constraints();
        for player in 0..6 as usize {
            match public_constraints[player].len() + inferred_constraints[player].len() {
                2 => {
                    for card in action_inferred_constraint[player].iter() {
                        if !inferred_constraints[player].contains(card) {
                            log::trace!("is_valid_combination inferred_constraint for player: {player}, does not contain: {:?}", card);
                            log::trace!("is_valid_combination original inferred_constraint for player: {player}, : {:?}", action_inferred_constraint[player]);
                            return false
                        }
                    }
                },
                1 => {
                    if action_public_constraint[player].len() + action_inferred_constraint[player].len() == 2 {
                        for card in inferred_constraints[player].iter() {
                            if !action_inferred_constraint[player].contains(card) {
                                log::trace!("is_valid_combination original inferred_constraint for player: {player}, does not contain: {:?}", card);
                                log::trace!("is_valid_combination original inferred_constraint for player: {player}, : {:?}", action_inferred_constraint[player]);
                                return false
                            }
                        }
                    } 
                },
                0 => {},
                _ => {
                    log::trace!("is_valid_combination player {} has too many cards", player);
                    return false
                },
            }
        }
        if inferred_constraints[6].len() + action_inferred_constraint[6].len() > 3 {
            let mut count_total: u8 = 0;
            let mut action_inferred_counts: [u8; 5] = [0; 5];
            let mut inferred_counts: [u8; 5] = [0; 5];
            action_inferred_constraint[6].iter().for_each(|c| action_inferred_counts[*c as usize] += 1);
            inferred_constraints[6].iter().for_each(|c| inferred_counts[*c as usize] += 1);
            action_inferred_counts.iter().zip(inferred_counts.iter()).for_each(|(a, i)| if *i > * a {count_total += *i.max(a);} );
            return count_total <= 3
        }
        true
    }
    pub fn recurse_variants_exchange(&self, index_loop: usize, index_of_interest: usize, player_of_interest: usize, public_constraints: &mut Vec<Vec<Card>>, inferred_constraints: &mut Vec<Vec<Card>>, player_loop: usize, cards: &[u8; 5]) -> bool {
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
        log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
        log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
                
        if self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards) {
            return true;
        }
        // variants.push(inferred_constraints.clone());
        // 1 player_to_pile move, 0 pile_to_player move
        if inferred_constraints[6].len() < 3 && inferred_constraints[player_loop].len() > 0{
            for card_player in iter_cards_player.iter() {
            // move to pile
                // let mut player_hand = inferred_constraints[player_loop].clone();
                // let mut pile_hand = inferred_constraints[6].clone();
                if let Some(pos) = inferred_constraints[player_loop].iter().rposition(|c| *c == *card_player) {
                    log::trace!("Before Exchange 1 player_to_pile 0 pile_to_player");
                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                    log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                    inferred_constraints[player_loop].swap_remove(pos);
                    inferred_constraints[6].push(*card_player);
                    // let mut temp = inferred_constraints.clone();
                    // temp[player_loop] = inferred_constraints[player_loop].clone();
                    // temp[6] = inferred_constraints[6].clone();
                    // variants.push(temp);
                    if self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards) {
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
                // let mut player_hand = inferred_constraints[player_loop].clone();
                // let mut pile_hand = inferred_constraints[6].clone();
                if let Some(pos) = inferred_constraints[6].iter().rposition(|c| *c == *card_pile) {
                    log::trace!("Before Exchange 0 player_to_pile 1 pile_to_player");
                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                    log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                    inferred_constraints[6].swap_remove(pos);
                    inferred_constraints[player_loop].push(*card_pile);
                    // let mut temp = inferred_constraints.clone();
                    // temp[player_loop] = player_hand.clone();
                    // temp[6] = pile_hand.clone();
                    // variants.push(temp);
                    if self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards) {
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
                    // let mut player_hand = inferred_constraints[player_loop].clone();
                    // let mut pile_hand = inferred_constraints[6].clone();
                    log::trace!("Before Exchange 1 player_to_pile 1 pile_to_player");
                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
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
                    // let mut temp = inferred_constraints.clone();
                    // temp[player_loop] = player_hand.clone();
                    // temp[6] = pile_hand.clone();
                    // variants.push(temp);
                    if self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards) {
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
                log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                let card_0 = inferred_constraints[player_loop][0];
                let card_1 = inferred_constraints[player_loop][1];
                // let mut player_hand = inferred_constraints[player_loop].clone();
                // let mut pile_hand = inferred_constraints[6].clone();
                inferred_constraints[player_loop].clear();
                inferred_constraints[6].push(card_0);
                inferred_constraints[6].push(card_1);
                // let mut temp = inferred_constraints.clone();
                // temp[player_loop] = inferred_constraints[player_loop].clone();
                // temp[6] = pile_hand.clone();
                // variants.push(temp);
                if self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards) {
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
                        log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                        log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                        // let mut player_hand = inferred_constraints[player_loop].clone();
                        // let mut pile_hand = inferred_constraints[6].clone();
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
                        // let mut temp = inferred_constraints.clone();
                        // temp[player_loop] = player_hand.clone();
                        // temp[6] = pile_hand.clone();
                        // variants.push(temp);
                        if self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards) {
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
                            log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                            log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                            // let mut player_hand = inferred_constraints[player_loop].clone();
                            // let mut pile_hand = inferred_constraints[6].clone();
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
                            // let mut temp = inferred_constraints.clone();
                            // temp[player_loop] = player_hand.clone();
                            // temp[6] = pile_hand.clone();
                            // variants.push(temp);
                            if self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards) {
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
                            log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                            log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                            // let mut player_hand = inferred_constraints[player_loop].clone();
                            // let mut pile_hand = inferred_constraints[6].clone();
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
                            // let mut temp = inferred_constraints.clone();
                            // temp[player_loop] = player_hand.clone();
                            // temp[6] = pile_hand.clone();
                            // variants.push(temp);
                            if self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards) {
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
                                log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, index_of_interest: {index_of_interest}, player_of_interest: {player_of_interest} move: player: {} {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
        
                                // let mut player_hand = inferred_constraints[player_loop].clone();
                                // let mut pile_hand = inferred_constraints[6].clone();
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
                                // let mut temp = inferred_constraints.clone();
                                // temp[player_loop] = player_hand.clone();
                                // temp[6] = pile_hand.clone();
                                // variants.push(temp);
                                if self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, player_of_interest, public_constraints, inferred_constraints, cards) {
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

impl CoupConstraint for BackTrackCollectiveConstraint {
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
        history.push(start);
        // StartInferred takes the inferred information from start, and runs add_inferred_information
        // This seperation prevents handling cases where you add discovered information that is already inside due to add_inferred_information
        let mut start_inferred = SignificantAction::start_inferred(); 
        start_inferred.add_inferred_constraints(player, cards[0]);
        start_inferred.add_inferred_constraints(player, cards[1]);
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

    fn add_move(&mut self, player_id: u8, action: ActionInfo) {
        match action {
            ActionInfo::Discard { .. } => {
                let significant_action = SignificantAction::initial(self.move_no, player_id, action);
                // Pushing before due to recursion mechanic
                self.history.push(significant_action);
                self.calculate_stored_move_initial();
                // Handle inference
            },
            ActionInfo::RevealRedraw { .. } => {
                let significant_action = SignificantAction::initial(self.move_no, player_id, action);
                self.history.push(significant_action);
                self.calculate_stored_move_initial();
                // Handle inference
            },
            ActionInfo::ExchangeDrawChoice { .. } => {
                let significant_action = SignificantAction::initial(self.move_no, player_id, action);
                // Handle inference
                // TODO: This is temporary, unsure how might split between public and private
                // It is possible that we can just use private, since public is just private with empty vec?
                self.history.push(significant_action);
                self.calculate_stored_move_initial();
            },
            ActionInfo::ExchangeDraw { .. } => todo!(),
            ActionInfo::ExchangeChoice { .. } => todo!(),
            ActionInfo::Start 
            | ActionInfo::StartInferred => {
                // TODO: Consider removing Start, so we can eliminate this branch entirely
                debug_assert!(false, "should not be pushing this!");
            },
        }
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
impl CoupConstraintAnalysis for BackTrackCollectiveConstraint {
    fn public_constraints(&mut self) -> &Vec<Vec<Card>> {
        &self.public_constraints
    }

    fn sorted_public_constraints(&mut self) -> &Vec<Vec<Card>> {
        self.public_constraints.iter_mut().for_each(|v| v.sort_unstable());
        &self.public_constraints
    }
    
    fn inferred_constraints(&mut self) -> &Vec<Vec<Card>> {
        &self.inferred_constraints
    }
    
    fn sorted_inferred_constraints(&mut self) -> &Vec<Vec<Card>> {
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
    
    fn player_can_have_card_alive(&mut self, player: usize, card: Card) -> bool{
        !self.impossible_constraints[player][card as usize]
    }
    fn player_can_have_card_alive_lazy(&mut self, player: usize, card: Card) -> bool{
        self.player_can_have_card_alive(player, card)
    }
    
    fn player_can_have_cards_alive(&mut self, player: usize, cards: &[Card]) -> bool{
        if player < 6 {
            if cards.len() == 2 {
                return !self.impossible_constraints_2[player][cards[0] as usize][cards[1] as usize]
            } else if cards.len() == 1 {
                return self.player_can_have_card_alive(player, cards[0])
            }
        } else if player == 6 {
            if cards.len() == 1 {
                return self.player_can_have_card_alive(player, cards[0])
            } else if cards.len() == 2 {
                return !self.impossible_constraints_2[player][cards[0] as usize][cards[1] as usize]
            } else if cards.len() == 3 {
                return !self.impossible_constraints_3[cards[0] as usize][cards[1] as usize][cards[2] as usize]
            }
        }
        false
    }
    fn player_can_have_cards_alive_lazy(&mut self, player: usize, cards: &[Card]) -> bool{
        self.player_can_have_cards_alive(player, cards)
    }
    fn is_legal_move_public(&mut self, action_observation: &ActionObservation) -> bool {
        match action_observation {
            ActionObservation::Discard { player_id, card, no_cards } => {
                if *no_cards == 1 {
                    self.player_can_have_card_alive_lazy(*player_id, card[0])
                } else {
                    self.player_can_have_cards_alive_lazy(*player_id, card)
                }
            },
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
                self.player_can_have_card_alive_lazy(*player_id, *reveal)
            },
            _ => true,
        }
    }
    fn is_legal_move_private(&mut self, action_observation: &ActionObservation) -> bool {
        unimplemented!()
    }
}