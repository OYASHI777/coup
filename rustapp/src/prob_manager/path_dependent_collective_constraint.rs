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
    meta_data: PathDependentMetaData,
}

impl SignificantAction {
    pub fn initial(move_no: usize, player: u8, action_info: ActionInfo) -> Self {
        let pd_metadata = PathDependentMetaData::initial();
        Self {
            move_no,
            player,
            action_info,
            meta_data: pd_metadata,
        }
    }
    pub fn start() -> Self {
        let meta_data: PathDependentMetaData = PathDependentMetaData::start();
        Self {
            move_no: 0,
            player: 77,
            action_info: ActionInfo::Start,
            meta_data,
        }
    }
    pub fn start_inferred() -> Self {
        let meta_data: PathDependentMetaData = PathDependentMetaData::start();
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
    pub fn meta_data(&self) -> &PathDependentMetaData {
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
// TODO: implement Into<PathDependentMetaData> for PathDependentCollectiveConstraint
// TODO: Store SignificantAction
// TODO: Fix up add_inferred_cards API to not take vec_changes
// TOD: Fix up reveal_redraw API to make inline with add_inferred_card, even though it also adds a group
// TODO: [OPTIMIZE] impossible_constraints can be stored as a u64 / 7 u8s (56 bits)
#[derive(Clone, Debug)]
pub struct PathDependentMetaData {
    public_constraints: Vec<Vec<Card>>,
    inferred_constraints: Vec<Vec<Card>>,
    impossible_constraints: [[bool; 5]; 7], // For each player store an array of bool where each index is a Card, this represents whether a player cannot have a card true => cannot
}

impl PathDependentMetaData {
    pub fn initial() -> Self {
        Self::start()
    }
    pub fn start() -> Self {
        let public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::new()]; 
        let inferred_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(3)]; 
        let impossible_constraints: [[bool; 5]; 7] = [[false; 5]; 7];
        Self {
            public_constraints,
            inferred_constraints,
            impossible_constraints,
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

// impl Into<PathDependentMetaData> for PathDependentCollectiveConstraint {
//     fn into(self) -> PathDependentMetaData {
//         PathDependentMetaData {
//             public_constraints: self.public_constraints,
//             inferred_constraints: self.inferred_constraints,
//             impossible_constraints: self.impossible_constraints,
//         }
//     }
// }


// 1: Add recursion on finding inferred constraint
//      - Can possibly store a boolean that determines if any empty redraw is before a number, so no need to lookback for that
// 2: Optimize to consider where we might not need to recurse (non recursive method can get 1/250 games wrong)
//      - Consider memoizing by storing a card, that a revealredraw might need to redraw
//          - during forward pass, if its impossible for player to have that card, then redraw it
#[derive(Clone, Debug)]
/// A struct that helps in card counting. Stores all information known about cards by a particular player.
pub struct PathDependentCollectiveConstraint {
    // public_constraints:[[Option<Card>; 2]; 6], // Stores all the dead cards of dead players, None are all behind
    public_constraints: Vec<Vec<Card>>, // Stores all the dead cards of dead players, None are all behind
    // inferred_constraints:[[Option<Card>; 2]; 6], // Stores all the dead cards of dead players 
    inferred_constraints: Vec<Vec<Card>>, // Stores all the inferred cards of alive players 
    // Struct of arrays makes it more efficient to process rather than an array of structs
    group_constraints_amb: Vec<CompressedGroupConstraint>,
    group_constraints_ass: Vec<CompressedGroupConstraint>,
    group_constraints_cap: Vec<CompressedGroupConstraint>,
    group_constraints_duk: Vec<CompressedGroupConstraint>,
    group_constraints_con: Vec<CompressedGroupConstraint>,
    impossible_constraints: [[bool; 5]; 7], // For each player store an array of bool where each index is a Card, this represents whether a player cannot have a card true => cannot
    // TODO: It seems like this is only useful for determining if single_card_flag should be set to false
    //      => which is inferring that a player redrew that card.
    //      => can try without revealed_status
    //      => only concern is a possible case wehre single_card_flag should be false because it was redrawn, but we not sure where it is redrawn
    // revealed_status: Vec<Vec<(Option<Card>, usize)>>, 
    move_no: usize, // turn number (0 being the start) (post increment this so assign then increment)
    // TODO: Consider having indices for each player to make traversal over players faster
    history: Vec<SignificantAction>, // Stores 
    // The shared LRU cache is maintained here and passed to each constraint.
    // cache: Rc<RefCell<LruCache<ConstraintKey, SignificantAction>>>,
    // Revealed_status stores the cards and the players that have reveal_redrawn, and have yet to use ambassador (mix)
    // When reveal_redraw is done, the card is added for the corresponding player
    // If player has mixed, it gets emptied.
    // a card can be removed from revealed_status on Discard too, though it may not update all groups
}

impl PathDependentCollectiveConstraint {
    pub fn game_start() -> Self {
        let public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::new()]; 
        let inferred_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(3)]; 
        let mut group_constraints_amb: Vec<CompressedGroupConstraint> = Vec::with_capacity(5);
        let mut group_constraints_ass: Vec<CompressedGroupConstraint> = Vec::with_capacity(5);
        let mut group_constraints_cap: Vec<CompressedGroupConstraint> = Vec::with_capacity(5);
        let mut group_constraints_duk: Vec<CompressedGroupConstraint> = Vec::with_capacity(5);
        let mut group_constraints_con: Vec<CompressedGroupConstraint> = Vec::with_capacity(5);
        let mut card_num_constraint: CompressedGroupConstraint = CompressedGroupConstraint::zero();
        for i in 0..7 {
            card_num_constraint.set_player_flag(i, true);
        }
        card_num_constraint.set_alive_count(3);
        card_num_constraint.set_total_count(3);
        card_num_constraint.set_card(Card::Ambassador);
        group_constraints_amb.push(card_num_constraint);
        card_num_constraint.set_card(Card::Assassin);
        group_constraints_ass.push(card_num_constraint);
        card_num_constraint.set_card(Card::Captain);
        group_constraints_cap.push(card_num_constraint);
        card_num_constraint.set_card(Card::Duke);
        group_constraints_duk.push(card_num_constraint);
        card_num_constraint.set_card(Card::Contessa);
        group_constraints_con.push(card_num_constraint);
        let impossible_constraints: [[bool; 5]; 7] = [[false; 5]; 7];
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
            group_constraints_amb,
            group_constraints_ass,
            group_constraints_cap,
            group_constraints_duk,
            group_constraints_con,
            impossible_constraints,
            move_no: 1,
            history,
        }
    }
    /// TODO: change gamestart for different inferred starting hands
    /// Recreates start state based on Start ActionInfo which may include inferred cards
    fn regenerate_game_start(&mut self) {
        self.public_constraints.iter_mut().for_each(|v| v.clear());
        self.inferred_constraints = self.history.first().unwrap().inferred_constraints().clone();
        self.group_constraints_amb.clear();
        self.group_constraints_ass.clear();
        self.group_constraints_cap.clear();
        self.group_constraints_duk.clear();
        self.group_constraints_con.clear();
        // TODO: Make this nicer
        let mut card_num_constraint: CompressedGroupConstraint = CompressedGroupConstraint::zero();
        for i in 0..7 {
            card_num_constraint.set_player_flag(i, true);
        }
        card_num_constraint.set_alive_count(3);
        card_num_constraint.set_total_count(3);

        card_num_constraint.set_card(Card::Ambassador);
        self.group_constraints_amb.push(card_num_constraint);

        card_num_constraint.set_card(Card::Assassin);
        self.group_constraints_ass.push(card_num_constraint);

        card_num_constraint.set_card(Card::Captain);
        self.group_constraints_cap.push(card_num_constraint);

        card_num_constraint.set_card(Card::Duke);
        self.group_constraints_duk.push(card_num_constraint);

        card_num_constraint.set_card(Card::Contessa);
        self.group_constraints_con.push(card_num_constraint);

        self.impossible_constraints = [[false; 5]; 7];
        // !! Not gonna reset move_no
        // Not adding inferred information as sometimes a discard could try to insert
        // info that has been inferred from add_inferred_information
        // TODO: TEST or I could just not save the meta_data but let it infer anyways
        // self.add_inferred_information();
        log::trace!("regenerate_game_start");
        self.printlog();
    }
    pub fn to_meta_data(&mut self) -> PathDependentMetaData {
        PathDependentMetaData { 
            public_constraints: self.public_constraints.clone(), 
            inferred_constraints: self.inferred_constraints.clone(), 
            impossible_constraints: self.impossible_constraints.clone()
        }
    }
    /// Create a method to understand for the latest discard/inferred card, whether some previous move's 
    /// hidden information is known
    pub fn lookback_0(&self) {

    }
    /// Create a method to understand for the some discard/inferred card, whether any previous move's 
    /// hidden information is known
    /// or actually maybe don't need this, but you can repeat on the latest card over and over?
    /// might need to look forward too?
    /// TODO: Add for first Discard / RevealRedraw to update start state + its impossible states
    /// TODO: Make impossible constraints something u always update man
    // pub fn lookback_1(&mut self, index: usize) -> bool {
    //     // index is the index for history``
    //     let considered_move: &SignificantAction = &self.history[index];
    //     match *considered_move.action_info() {
    //         ActionInfo::RevealRedraw{reveal: reveal_considered, redraw: redraw_considered, ..} => {
    //             match redraw_considered {
    //                 Some(redraw_card) => {
    //                     // unimplemented!()
    //                 },
    //                 None => {
    //                     // Group A abstract to lookback_2 for inferred addition too
    //                 }
    //             }
    //         },
    //         ActionInfo::Discard{discard: discard_considered} => {
    //             // Group A abstract to lookback_2 for inferred addition too
    //             let player_index: u8 = considered_move.player();
    //             for i in (1..index).rev() {
    //                 let action_data = &mut self.history[i];
    //                 let action_player = action_data.player();
    //                 if action_player == player_index {
    //                     // Case 0
    //                     // RR or AMB with 1 life
    //                     let action_name = action_data.name();
    //                     match action_name { // This is just a get around of the partial borrowing rules...
    //                         ActionInfoName::RevealRedraw => {
    //                             let need_redraw_update = if let ActionInfo::RevealRedraw { redraw: redraw_i, .. } = action_data.action_info() {
    //                                 redraw_i.is_none()
    //                                     && action_data.player_cards_known(action_player) == 2 // This is after the reveal
    //                                     && !action_data.player_has_inferred_constraint(action_player, discard_considered) 
    //                                     // TODO: Consider maybe case where the player has 2 of discard_considered?
    //                             } else {
    //                                 false
    //                             };
    //                             if need_redraw_update {
    //                                 if let ActionInfo::RevealRedraw { redraw, .. } = action_data.action_info_mut() {
    //                                     *redraw = Some(discard_considered);
    //                                     self.regenerate_path();
    //                                     log::trace!("End Regenerate Path lookback_initial");
    //                                     self.printlog();
    //                                     // panic!();
    //                                     return true;
    //                                 }
    //                             } else {
    //                                 return false;
    //                             }
    //                         },
    //                         ActionInfoName::ExchangeDrawChoice => {
    //                             return false;
    //                         }
    //                         _ => {},
    //                     }
    //                 }
    //             }
    //         },
    //         _ => {
    //             // unimplemented!();
    //         }
    //     }
    //     false
    // }
    /// Used on the initial addition of move to history, not recalculation
    /// This is ran after reveal and after discard
    /// NOTE: A pretty good lookback would also be to check if player impossible to have some card at start of game
    /// 
    /// 3 ways to recursively go about this
    /// 1)  Call lookback once, if updated regenerate everything
    ///     - every step of regeneration calls a lookback too
    ///     - O(n) + O(n^2)
    ///     - But this may run the calculation on each node multiple times (expensive)
    /// 2)  Call lookback once
    ///     - As you traverse back in moves, if required call lookback on another move found along the way
    ///     - When reaching the end, regenerate everything
    ///     - O(n) + O(n)
    ///     - regeneration done once (if feasible)
    ///     - less complex for branching
    ///     - less cache efficient than (3)
    /// 3)  Call lookback once
    ///     - maintain a store of things to lookback on (Discard P1, RR P2)
    ///     - for each move check in the store if anything to update
    ///     - when reaching the end, regenerate everything
    ///     - O(n) + O(n)
    ///     - regeneration done once (if feasible)
    ///     - but more complex due to branches and search over store
    ///     - more cache efficient than (2)
    pub fn lookback_initial(&mut self) -> bool {
        // index is the index for history
        log::trace!("In lookback_initial");
        let index = self.history.len() - 1;
        let action_info = self.history[index].action_info().clone();
        let player_index = self.history[index].player();
        let mut bool_changes = false;
        let mut bool_skip_start_update = false;
        match action_info {
            ActionInfo::RevealRedraw{reveal: reveal_considered, redraw: redraw_considered, ..} => {
                log::trace!("lookback_initial considering: RevealRedraw: player {player_index}, reveal: {:?}, redraw: {:?}", reveal_considered, redraw_considered);
                let bool_all_other_cards_dead = self.public_constraints.iter().map(|v| v.iter().filter(|c| **c == reveal_considered).count()).sum::<usize>() == 2;
                match redraw_considered {
                    Some(redraw_card) => {
                        // TODO: [CASE] kinda same as discard
                        unimplemented!("for public info games, moves added will only be without redraw known");
                    },
                    None => {
                        let mut card_assured_players: Vec<u8> = Vec::with_capacity(3);
                        // TEST UNSURE how this interacts with AMB
                        // let mut reveal_players: Vec<u8> = Vec::with_capacity(3);
                        let mut reveal_players = [false; 6];
                        // let mut illegal_to_change: Vec<u8> = Vec::with_capacity(12);
                        let mut illegal_to_change = [false; 6];
                        // Only really cos at this point of inference we only regard the reveal of RevealRedraw as basically an inferred_constraint
                        card_assured_players.push(self.history[index].player());
                        // Logic same as discard below
                        // 2 because first 2 moves are Start and StartInferred
                        'iter_loop: for i in (2..index).rev() {
                            log::trace!("RR iter saw: {:?}", action_info);
                            let action_data = &self.history[i];
                            let action_player = self.history[i].player();
                            let action_name = action_data.name();
                            // TODO: [OPTIMIZE] it might be better to have this check inside the match instead
                            if action_player == player_index {
                                // Case 0
                                // RR or AMB with 1 life
                                match action_name { // This is just a get around of the partial borrowing rules...
                                    ActionInfoName::RevealRedraw => {
                                        // Testing add after
                                        // if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                        //     if *reveal_i == reveal_considered && *redraw_i != Some(*reveal_i) {
                                        //         reveal_players.push(action_player);
                                        //     }
                                        // }
                                        log::trace!("lookback_initial RevealRedraw checking past RevealRedraw");
                                        log::trace!("lookback_initial original player: {} RevealRedraw: {:?}", player_index, action_info);
                                        log::trace!("lookback_initial checked player: {} RevealRedraw: {:?}", action_data.player(), action_data.action_info());
                                        // let need_redraw_update = if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                        //     redraw_i.is_none()
                                        //     // This is after the discard
                                        //     // This is just before the RevealRedraw
                                        //     // CASE when all cards are known and not reveal_considered
                                        //     && (
                                        //         (
                                        //             self.history[i - 1].meta_data().public_constraints()[action_player as usize].len() + self.history[i - 1].meta_data().inferred_constraints()[action_player as usize].len() == 2
                                        //             && !self.history[i - 1].player_has_inferred_constraint(action_player, reveal_considered) // REQUIRED FOR MARKER A
                                        //         )
                                        //         || (
                                        //             // [CASE] All player cards are the same card
                                        //             // Then the person previously definitely redrew that
                                        //             // This should be after card revealed
                                        //             self.inferred_constraints[action_player as usize].len() + self.public_constraints[action_player as usize].len() == 2
                                        //             && self.inferred_constraints[action_player as usize].iter().all(|&c| c == reveal_considered) 
                                        //             && self.public_constraints[action_player as usize].iter().all(|&c| c == reveal_considered)
                                        //         )
                                        //         || (
                                        //             // MARKER A
                                        //             // before the RevealRedraw
                                        //             // All discard_considered known and outside of player
                                        //             // Either 2 cards known and last in pile (therefore they could even redraw)
                                        //             // Or 3 cards known and 1 is in pile (therefore they can redraw)
                                        //             //  - This ASSUMES that the latest_move played is legal (which it should be)
                                        //             //
                                        //             (
                                        //                 // TODO：[OPTIMIZE]
                                        //                 // CASE 2 cards outside of player, player has 2 lives, and player reveals reveal_considered
                                        //                 //  - 2 players != action_player have card and != pile
                                        //                 //      - if player reveal == reveal_considered => must have withdrawn
                                        //                 self.history[i - 1].known_card_count(reveal_considered) == 2 
                                        //                 && !self.history[i - 1].inferred_constraints()[action_player as usize].contains(&reveal_considered)
                                        //                 && *reveal_i == reveal_considered
                                        //             ) 
                                        //             || (
                                        //                 // Case where player cannot possibly have the card if not for the redraw
                                        //                 //  => all 3 cards are outside of the player
                                        //                 // TODO: [OPTIMIZE] you probably can use impossible constraints to figure this out
                                        //                 // CASE where 2 are known and we know theres one group with 1 alive 
                                        //                 // That excludes the current player
                                        //                 // maybe more specifically some are known dead, some are known alive
                                        //                 // and there is one group with 1 alive which includes pile and
                                        //                 // excludes players whom have known alive
                                        //                 // TODO: Document lookback_check
                                        //                 // *reveal_i != reveal_considered
                                        //                 // && self.history[i - 1].known_card_count(reveal_considered) == 2 
                                        //                 // && !self.history[i - 1].inferred_constraints()[6].contains(&reveal_considered) 
                                        //                 // && self.lookback_check(i - 1, action_player, reveal_considered).is_some()
                                        //                 self.lookback_check_2(i, action_player as usize, reveal_considered)
                                        //             )
                                        //             // || (
                                        //             //     bool_all_other_cards_dead
                                        //             //     && *reveal_i == reveal_considered
                                        //             // )
                                        //             || (
                                        //                 // We actually only need to check *reveal == reveal_considered && last 2 cards outside of player at that time
                                        //                 // => 2 not player_index revealed for first time before and player has not revealed redraw none before this
                                        //                 // => 2 outside player_index discarded after
                                        //                 // // maybe I can measure maximum/minimum possible outside player?
                                        //                 *reveal_i == reveal_considered
                                        //                 && {
                                        //                     // Getting total unique first time reveal_redraw before player, excluding player
                                        //                     let mut historical_first_time_reveal_count = 0;
                                        //                     let mut redraw_stack_pop_count = 0;
                                        //                     // Size expected to be small
                                        //                     let mut visited_players = [false; 6];
                                        //                     let mut visited_players_opp = [false; 6];
                                        //                     // visited_players[player_index as usize] = true;
                                        //                     for sig_act in self.history[2..i].iter() {
                                        //                         if !visited_players[sig_act.player() as usize] {
                                        //                             if let ActionInfo::RevealRedraw { reveal, redraw, .. } = sig_act.action_info() {
                                        //                                 if *reveal == reveal_considered {
                                        //                                     visited_players[sig_act.player() as usize] = true;
                                        //                                     historical_first_time_reveal_count += 1;
                                        //                                 } else if redraw.is_none() {
                                        //                                     // visited_players_opp[sig_act.player() as usize] = true;
                                        //                                     // erasure of progress because of AMB or redraw
                                        //                                     visited_players = [false; 6];
                                        //                                 } else if *redraw == Some(reveal_considered) {
                                        //                                     redraw_stack_pop_count += 1;
                                        //                                 }
                                        //                             }
                                        //                         }
                                        //                     }


                                        //                     // log::trace!("reveal_players {:?}, discard_players: {:?}", temp, card_assured_players);
                                        //                     // log::trace!("reveal_players.iter().filter(|p| **p != player_index).count() >= 3 - discard_players.len() = {}", reveal_players.iter().filter(|p| **p != player_index).count() >= 3 - card_assured_players.len());
                                        //                     // temp.iter().filter(|p| **p != player_index).count() >= 3 - card_assured_players.len()
                                        //                     // 1 is added as first reveal is an assured card
                                        //                     // let total_assured_cards = self.public_constraints.iter().map(|v| v.iter().filter(|c| **c == reveal_considered).count()).sum::<usize>() + 1;
                                        //                     // log::trace!("temp.iter().filter(|p| **p != player_index).count() >= 3 - total_assured_cards = :{}", temp.iter().filter(|p| **p != player_index).count() >= 3 - total_assured_cards);
                                        //                     // TODO: Choose one of the below! ALso do we include historical_first_time_reveal_count?
                                        //                     // reveal_players.iter().enumerate().filter(|(i, v)| *i != player_index as usize && **v).count()
                                        //                     // reveal_players.iter().zip(visited_players.iter()).enumerate().filter(|(i, (v0, v1))| *i != player_index as usize && (**v0 || **v1)).count()
                                        //                     // reveal_players.iter().zip(visited_players.iter()).enumerate().filter(|(i, (v0, v1))| **v0 || **v1).count()
                                        //                     reveal_players.iter().zip(visited_players.iter()).zip(visited_players_opp.iter()).enumerate().filter(|(i, ((v0, v1), v2))| (**v0 || **v1) && !**v2).count()
                                        //                     + card_assured_players.len()  // valid dead from latest move to current iter + dead before
                                        //                     + self.history[i - 1].public_constraints().iter().map(|v| v.iter().filter(|c| **c == reveal_considered).count()).sum::<usize>()
                                        //                     // + historical_first_time_reveal_count
                                        //                     >= 3 + redraw_stack_pop_count
                                        //                     // let total_assured_cards = card_assured_players.len();
                                        //                     // log::trace!("total_assured_cards: {}", total_assured_cards);
                                        //                     // log::trace!("temp: {:?}", temp);
                                        //                     // temp.iter().filter(|p| **p != player_index).count() >= 3 - total_assured_cards
                                        //                     // temp.len() >= 3 - self.public_constraints.iter().map(|v| v.iter().filter(|c| **c == reveal_considered).count()).sum::<usize>()
                                        //                 }
                                        //                 || {
                                        //                     *reveal_i == reveal_considered &&
                                        //                     // had to have withdrawn if other 2 cards are outside player
                                        //                     self.history[i - 1].public_constraints().iter().enumerate().filter(|(p, _)| *p != action_player as usize).map(|(_, v)| v.iter().filter(|c| **c == reveal_considered).count()).sum::<usize>()
                                        //                     + self.history[i - 1].inferred_constraints().iter().enumerate().filter(|(p, _)| *p != action_player as usize).map(|(_, v)| v.iter().filter(|c| **c == reveal_considered).count()).sum::<usize>()
                                        //                     == 2
                                        //                 }
                                        //             )
                                        //             // && (
                                        //             //     || self.lookback_check(i - 1, action_player, reveal_considered).is_some()
                                        //             //     // Temp test change location
                                        //             // )
                                        //             // === OLD ===
                                        //             // self.history[i - 1].known_card_count(reveal_considered) == 2 
                                        //             // && !self.history[i - 1].inferred_constraints()[6].contains(&reveal_considered)
                                        //             // // This doesnt actually make sense as *reveal_i == reveal_considered won't ever be true
                                        //             // // && !self.history[i - 1].inferred_constraints()[action_player as usize].contains(&reveal_considered)
                                        //             // && (
                                        //             //     // Consider seperating this and the above case
                                        //             //     *reveal_i == reveal_considered
                                        //             //     || self.lookback_check(i - 1, action_player, reveal_considered).is_some()
                                        //             //     // Temp test change location
                                        //             // )
                                        //             // === OLD ===
                                        //             || self.history[i - 1].known_card_count(reveal_considered) == 3
                                        //             && !self.history[i - 1].player_has_inferred_constraint(action_player, reveal_considered)
                                        //             // CASE 3 cards outside of player and (implied that player obviously won't reveal reveal_considered)
                                        //             // && !self.history[i - 1].player_has_inferred_constraint(action_player, *reveal_i)
                                        //         )
                                        //         // I think this below can be checked much earlier
                                        //         // This shit breaks for some reason
                                        //         || self.history[i - 1].public_constraints()[action_player as usize].len() == 1
                                        //     )
                                        // } else {
                                        //     false
                                        // };
                                        let need_redraw_update = self.need_redraw_update(i, reveal_considered, &illegal_to_change);
                                        if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                            if *reveal_i == reveal_considered && 
                                            *redraw_i != Some(*reveal_i) 
                                            && !card_assured_players.contains(&action_player) {
                                                // reveal_players.push(action_player);
                                                reveal_players[action_player as usize] = true;
                                            }
                                            if redraw_i.is_none() && i != index {
                                                // Exclude the revealredraw that was just played as its effectively considered a 
                                                // inferred_constraint
                                                illegal_to_change[action_player as usize] = true;
                                                card_assured_players.retain(|p| *p != action_player);
                                                log::trace!("card_assured_players removed: retained not player: {action_player}");
                                                log::trace!("card_assured_players: {:?}",card_assured_players);
                                            }
                                        }
                                        // if let ActionInfo::RevealRedraw { redraw: redraw_i, .. } = action_data.action_info() {
                                        //     log::trace!("redraw_i.is_none() = : {}", redraw_i.is_none());
                                        //     log::trace!("&&");
                                        //     log::trace!("self.history[i - 1].meta_data().public_constraints()[action_player as usize].len() + self.history[i - 1].meta_data().inferred_constraints()[action_player as usize].len() == 2 = {}", self.history[i - 1].meta_data().public_constraints()[action_player as usize].len() + self.history[i - 1].meta_data().inferred_constraints()[action_player as usize].len() == 2);
                                        //     log::trace!("&&");
                                        //     log::trace!("!action_data.player_has_inferred_constraint(action_player, reveal_considered) = {}", !self.inferred_constraints[action_player as usize].contains(&reveal_considered));
                                        //     log::trace!("||");
                                        //     log::trace!("(");
                                        //     log::trace!("self.public_constraints[action_player as usize].len() + self.inferred_constraints[action_player as usize].len() == 2 = {}", self.public_constraints[action_player as usize].len() + self.inferred_constraints[action_player as usize].len() == 2);
                                        //     log::trace!("&&");
                                        //     log::trace!("self.player_constraints_all_full(action_player, reveal_considered)) = {}", self.inferred_constraints[action_player as usize].iter().all(|&c| c == reveal_considered) &&
                                        //     self.public_constraints[action_player as usize].iter().all(|&c| c == reveal_considered));
                                        //     log::trace!(")");
                                        //     log::trace!("||");
                                        //     log::trace!("(");
                                        //     log::trace!("self.history[i - 1].known_card_count(reveal_considered) == 2 = {}", self.history[i - 1].known_card_count(reveal_considered) == 2);
                                        //     log::trace!("&&");
                                        //     log::trace!("   (");
                                        //     log::trace!("*reveal_i == reveal_considered = ?? reveal_i: {:?}, reveal_considered: {:?}", action_data.action_info(), reveal_considered);
                                        //     log::trace!("||");
                                        //     log::trace!("   (");
                                        //     log::trace!("self.lookback_check_2({i}, {action_player}, reveal_considered) = {}", self.lookback_check_2(i, action_player as usize, reveal_considered));
                                        //     log::trace!("   )");
                                        //     log::trace!("||");
                                        //     log::trace!("   (");
                                        //     log::trace!("bool_all_other_cards_dead: {}", bool_all_other_cards_dead);
                                        //     log::trace!("   )");
                                        //     log::trace!("||");
                                        //     log::trace!("self.history[i - 1].known_card_count(reveal_considered) == 3 = {}", self.history[i - 1].known_card_count(reveal_considered) == 3);
                                        //     log::trace!(")");
                                        // }
                                        log::trace!("need_redraw_update evaluated to {need_redraw_update}");
                                        if need_redraw_update {
                                            log::trace!("lookback_initial original RevealRedraw: {:?}", action_info);
                                            log::trace!("lookback_initial considering: player: {} {:?}", action_data.player(), action_data.action_info());
                                            if let ActionInfo::RevealRedraw { reveal, redraw, .. } = self.history[i].action_info_mut() {
                                                log::trace!("lookback_initial setting redraw to: {:?}", reveal_considered);
                                                *redraw = Some(reveal_considered);
                                                // Continuing only if the card had to have come from the pile
                                                if Some(*reveal) != *redraw {
                                                    self.lookback_1_continual(i);
                                                }
                                                bool_changes = true;
                                                bool_skip_start_update = true;
                                                break 'iter_loop;
                                                //  ==== OLD ===
                                                // self.regenerate_path();
                                                // log::trace!("End Regenerate Path lookback_initial A");
                                                // self.printlog();
                                                // panic!();
                                                // TODO: [CASE] Knowing the redraw means we need to update the pile earlier!
                                                // return true;
                                            }
                                        } else {
                                            // OLD
                                            // return false;
                                            if let ActionInfo::RevealRedraw { redraw, .. } = self.history[i].action_info() {
                                                if redraw.is_none() {
                                                    // Avoid updating start when loop ends
                                                    bool_skip_start_update = true;
                                                    break 'iter_loop;
                                                }
                                            }
                                        }
                                    },
                                    ActionInfoName::ExchangeDrawChoice => {
                                        // OLD
                                        // return false;
                                        bool_skip_start_update = true;
                                        break 'iter_loop;
                                    }
                                    ActionInfoName::Discard => {
                                        if let ActionInfo::Discard { discard } = action_data.action_info() {
                                            log::trace!("saw action_player: {action_player} discard: {:?} in iter", discard);
                                            if *discard == reveal_considered {
                                                log::trace!("Pushing action_player: {action_player} into discard_considered");
                                                card_assured_players.push(action_player);
                                            }
                                        }
                                    }
                                    _ => {},
                                }
                            } else {
                                match action_name {
                                    ActionInfoName::Discard => {
                                        if let ActionInfo::Discard { discard } = action_data.action_info() {
                                            log::trace!("saw action_player: {action_player} discard: {:?} in iter", discard);
                                            if *discard == reveal_considered {
                                                log::trace!("Pushing action_player: {action_player} into discard_considered");
                                                card_assured_players.push(action_player);
                                            }
                                        }
                                    },
                                    ActionInfoName::RevealRedraw => {
                                        // Hi sir its cos u dont update here lol
                                        log::trace!("saw action_player: {action_player} revealredraw: {:?} in iter", action_data);

                                        // let need_redraw_update = if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                        //     // bool_all_other_cards_dead
                                        //     // && reveal_players.contains(&action_player) // Do I need this?
                                        //     // || discard_players.contains(&action_player)
                                        //     // && redraw_i.is_none()
                                        //     // In this case the original reveal_considered is added to discard
                                        //     // TODO: Consider naming it inferred card?
                                        //     // reveal_players.contains(&action_player)
                                        //     reveal_players[action_player as usize]
                                        //     && redraw_i.is_none()
                                        //     && (
                                        //         (
                                        //             // self.history[i - 1].public_constraints()[action_player as usize].len() + self.history[i - 1].meta_data().inferred_constraints()[action_player as usize].len() > 1
                                        //             // && !self.history[i - 1].player_has_inferred_constraint(action_player, discard_considered)
                                        //             false
                                        //         ) // required for MARKER A)
                                        //         // || (
                                        //         //     self.history[i - 1].public_constraints()[action_player as usize].len() == 1
                                        //         // )
                                        //         || (
                                        //             // I like this
                                        //             self.history[i - 1].impossible_constraints()[action_player as usize][reveal_considered as usize]
                                        //         )
                                        //         || (
                                        //             // CASE all cards are dead and known based on latest move
                                        //             // This player had discarded the card
                                        //             // So if they had revealed it in the past, they had to have redrawn it
                                        //             // else they could not have discarded it
                                        //             // Here we assume bool_all_cards_dead == true
                                        //             *reveal_i == reveal_considered
                                        //             && {
                                        //                 let mut historical_first_time_reveal_count = 0;
                                        //                 let mut redraw_stack_pop_count = 0; 
                                        //                     // Size expected to be small
                                        //                 let mut visited_players = [false; 6];
                                        //                 let mut visited_players_opp = [false; 6];
                                        //                 // visited_players[player_index as usize] = true;
                                        //                 for sig_act in self.history[2..i].iter() {
                                        //                     if !visited_players[sig_act.player() as usize] {
                                        //                         if let ActionInfo::RevealRedraw { reveal, redraw, .. } = sig_act.action_info() {
                                        //                             if *reveal == reveal_considered {
                                        //                                 visited_players[sig_act.player() as usize] = true;
                                        //                                 historical_first_time_reveal_count += 1;
                                        //                             } else if redraw.is_none() {
                                        //                                 // visited_players_opp[sig_act.player() as usize] = true;
                                        //                                 // erasure of progress because of AMB or redraw
                                        //                                 visited_players = [false; 6];
                                        //                             } else if *redraw == Some(reveal_considered) {
                                        //                                 redraw_stack_pop_count += 1;
                                        //                             }
                                        //                         }
                                        //                     }
                                        //                 }
                                        //                 log::trace!("*reveal_i: {:?} == reveal_considered: {:?} = : {}", *reveal_i, reveal_considered, *reveal_i == reveal_considered);
                                        //                 log::trace!("player_index {:?}", player_index);
                                        //                 log::trace!("reveal_players {:?}, discard_players: {:?}", reveal_players, card_assured_players);
                                        //                 // log::trace!("reveal_players.iter().filter(|p| **p != player_index).count() >= 3 - discard_players.len() = {}", temp.iter().filter(|p| **p != player_index).count() >= 3 - card_assured_players.len());
                                        //                 // log::trace!("reveal_players.iter().filter(|p| **p != player_index).count() >= 3 - discard_players.len() = {}", temp.len() >= 3 - card_assured_players.len());
                                        //                 // Not excluding the RR here as the initial one was a RR
                                        //                 // NOTE: discard_pl
                                        //                 // Adding 1 as the reveal_considered is counted toward total_assured_cards
                                        //                 // let total_assured_cards = self.public_constraints.iter().map(|v| v.iter().filter(|c| **c == reveal_considered).count()).sum::<usize>() + 1;
                                        //                 // reveal_players.iter().enumerate().filter(|(i, v)| *i != player_index as usize && **v).count()
                                        //                 // reveal_players.iter().zip(visited_players.iter()).enumerate().filter(|(i, (v0, v1))| *i != player_index as usize && (**v0 || **v1)).count()
                                        //                 // reveal_players.iter().zip(visited_players.iter()).enumerate().filter(|(i, (v0, v1))| **v0 || **v1).count()
                                        //                 reveal_players.iter().zip(visited_players.iter()).zip(visited_players_opp.iter()).enumerate().filter(|(i, ((v0, v1), v2))| (**v0 || **v1) && !**v2).count()
                                        //                 + card_assured_players.len()  // valid dead from latest move to current iter + dead before
                                        //                 + self.history[i - 1].public_constraints().iter().map(|v| v.iter().filter(|c| **c == reveal_considered).count()).sum::<usize>()
                                        //                 // + historical_first_time_reveal_count
                                        //                 >= 3 + redraw_stack_pop_count
                                        //                 // let total_assured_cards = card_assured_players.len();
                                        //                 // temp.iter().filter(|p| **p != player_index).count() >= 3 - total_assured_cards
                                        //                 // temp.len() >= 3 - card_assured_players.len()
                                        //                 // temp.len() >= 3 - self.public_constraints.iter().map(|v| v.iter().filter(|c| **c == reveal_considered).count()).sum::<usize>()
                                        //             } 
                                        //             || {
                                        //                 *reveal_i == reveal_considered &&
                                        //                 // had to have withdrawn if other 2 cards are outside player
                                        //                 self.history[i - 1].public_constraints().iter().enumerate().filter(|(p, _)| *p != action_player as usize).map(|(_, v)| v.iter().filter(|c| **c == reveal_considered).count()).sum::<usize>()
                                        //                 + self.history[i - 1].inferred_constraints().iter().enumerate().filter(|(p, _)| *p != action_player as usize).map(|(_, v)| v.iter().filter(|c| **c == reveal_considered).count()).sum::<usize>()
                                        //                 == 2
                                        //             }
                                        //             // false
                                        //         )
                                        //     )
                                        // } else {
                                        //     false
                                        // };
                                        // This needs to be added after the check, if not the reveal will count to its own redraw count
                                        let need_redraw_update = self.need_redraw_update(i, reveal_considered, &illegal_to_change);
                                        if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                            if *reveal_i == reveal_considered && *redraw_i != Some(*reveal_i) 
                                            && !card_assured_players.contains(&action_player) {
                                                // reveal_players.push(action_player);
                                                reveal_players[action_player as usize] = true;
                                            }
                                            if redraw_i.is_none() && i != index {
                                                // Exclude the revealredraw that was just played as its effectively considered a 
                                                // inferred_constraint
                                                // Do not update any other of the same player after the closest RR has been considered
                                                illegal_to_change[action_player as usize] = true;
                                                card_assured_players.retain(|p| *p != action_player);
                                                log::trace!("card_assured_players removed: retained not player: {action_player}");
                                                log::trace!("card_assured_players: {:?}",card_assured_players);
                                            }
                                        }
                                        log::trace!("before need_redraw_update: {need_redraw_update}");
                                        if need_redraw_update {
                                            log::trace!("lookback_initial original RevealRedraw: {:?}", action_info);
                                            log::trace!("lookback_initial considering: player: {} {:?}", action_data.player(), action_data.action_info());
                                            if let ActionInfo::RevealRedraw { reveal, redraw, .. } = self.history[i].action_info_mut() {
                                                log::trace!("RR != lookback_initial setting redraw to: {:?}", reveal_considered);
                                                *redraw = Some(reveal_considered);
                                                // discard_players.retain(|p| *p != action_player);
                                                // TODO: In this case, we may not want to regenerate immediately
                                                // OPTIONS:
                                                //      - Regenerate now
                                                //      - Regenerate and allow another lookback during regeneration
                                                //      - Handle all lookbacks now, then regenerate
                                                // CASE:
                                                //              P1 Discard Duke
                                                //              P2 RR Reveal Duke
                                                //              P3 RR Reveal Duke
                                                //              P2 Discard Duke
                                                //              P3 Discard Duke
                                                //      When P3 Discard Duke, we know that both P2 and P3 redrew Dukes, but never before that
                                                //      So we would need to update both, and early exit may not be ideal
                                                // if Some(*reveal) != *redraw {
                                                //     self.lookback_1_continual(i);
                                                // }
                                                // self.regenerate_path();
                                                // log::trace!("End Regenerate Path lookback_initial C");
                                                // self.printlog();
                                                // panic!();
                                                // TODO: [CASE] Knowing the redraw means we need to update the pile earlier!
                                                // return true;
                                                bool_changes = true;
                                                // break 'iter_loop;
                                                // This is copied from discard
                                                // TODO: I have a feeling I need an exclude group for where there are multiple RevealRedraws
                                                // TODO: I have a feeling I only need to kick them out if reveal_redraw is None?
                                                // Like if we pass a reveal == redraw must we kick them?
                                                // What does this do
                                                // if redraw.is_none() {
                                                card_assured_players.retain(|p| *p != action_player);
                                                // }
                                            }
                                        }

                                    },
                                    _ => {}
                                }
                            }
                        }
                        // Else if its start (which means we did not hit a revealredraw or ambassador)
                        // TODO: Update start here (inferred)
                        // TODO: Somehow, maybe by regenerating, or not update impossible constraints too
                        // TODO: Change, this should only be done for the first time its added for discard
                        debug_assert!(self.history[0].name() == ActionInfoName::Start, "wrong Significant Action at index 0!");
                        // TODO: Fix bad_push TEMP
                        // Add 2 same cards case
                        // if !self.history[0].inferred_constraints()[player_index as usize].contains(&reveal_considered) {
                        if !bool_skip_start_update {
                            log::trace!("lookback_initial index: {:?}", index);
                            log::trace!("lookback_initial processed item: {:?}", self.history.last().unwrap().action_info());
                            log::trace!("lookback_initial adding inferred_constraint to start: (player: {}, card: {:?})", player_index, reveal_considered);
                            log::trace!("lookback_initial start before: {:?}", self.history.first().unwrap().meta_data());
                            self.history[0].add_inferred_constraints(player_index as usize, reveal_considered);
                            log::trace!("lookback_initial start after: {:?}", self.history.first().unwrap().meta_data());
                            bool_changes = true;
                        }
                            // TODO: Have to change calculation for game start for this to work
                            // TODO: Include impossible at game start too!
                            // self.regenerate_path();
                            // log::trace!("End Regenerate Path lookback_initial B");
                            // self.printlog();
                            // return true;
                        // }
                    }
                }
            },
            ActionInfo::Discard{discard: discard_considered} => {
                // Logic same as RevealRedraw above
                // TODO: [REFACTOR]
                // Group A abstract to lookback_2 for inferred addition too
                // TODO: THINK Why not just collect from self.public_constraints???
                // Try this out
                let mut discard_players: Vec<u8> = Vec::with_capacity(3);
                let mut backpass_discard_players = [false; 6];
                backpass_discard_players[self.history[index].player() as usize] = true;
                // TEST UNSURE how this interacts with AMB
                // let mut reveal_players: Vec<u8> = Vec::with_capacity(3);
                let mut reveal_players = [false; 6];
                // Add here as we don't loop over current index
                // May expand to all cards known later
                // let mut illegal_players: Vec<u8> = Vec::with_capacity(12);
                let mut illegal_players = [false; 6];
                let bool_all_cards_dead = self.public_constraints.iter().map(|v| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>() == 3;

                // Handled earlier
                log::trace!("Pushing index_player: {} into discard_considered", self.history[index].player());
                discard_players.push(self.history[index].player());
                'iter_loop: for i in (2..index).rev() {
                    let action_data = &self.history[i];
                    log::trace!("iter saw action_data: {:?}", action_data);
                    let action_player = action_data.player();
                    if action_player == player_index { // TODO: [OPTIMIZE] Im thinking maybe you only need to do this check in the match statement
                        // Case 0
                        // RR or AMB with 1 life
                        let action_name = action_data.name();
                        match action_name { // This is just a get around of the partial borrowing rules...
                            ActionInfoName::RevealRedraw => {
                                // Temp for testing [REFACTOR]
                                // if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                //     if *reveal_i == discard_considered 
                                //     && *redraw_i != Some(*reveal_i) 
                                //     && !discard_players.contains(&action_player){
                                //         reveal_players.push(action_player);
                                //     }
                                // }
                                // log::trace!("lookback_initial Discard checking past RevealRedraw");
                                // log::trace!("lookback_initial original player: {}, Discard: {:?}", player_index, action_info);
                                // log::trace!("lookback_initial checked player: {}, RevealRedraw: {:?}", action_data.player(), action_data.action_info());
                                // let need_redraw_update = if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                //     !illegal_players.contains(&action_player)
                                //     && redraw_i.is_none()
                                //     // This is after the discard
                                //     // This is just before the RevealRedraw
                                //     // CASE In previous RevealRedraw, 
                                //     // Player hand fully known
                                //     // Player did not have discard_considered
                                //     // Then the person previously definitely redrew that
                                //     && (
                                //         (
                                //             self.history[i - 1].public_constraints()[action_player as usize].len() + self.history[i - 1].meta_data().inferred_constraints()[action_player as usize].len() == 2
                                //             && !self.history[i - 1].player_has_inferred_constraint(action_player, discard_considered) // required for MARKER A
                                //         )
                                //         || (
                                //             // [CASE] All player cards are the same card
                                //             // Then the person previously definitely redrew that
                                //             self.inferred_constraints[action_player as usize].len() + self.public_constraints[action_player as usize].len() == 2
                                //             && self.inferred_constraints[action_player as usize].iter().all(|&c| c == discard_considered) 
                                //             && self.public_constraints[action_player as usize].iter().all(|&c| c == discard_considered)
                                //         )
                                //         || (
                                //             // MARKER A
                                //             // before the RevealRedraw
                                //             // All discard_considered known and outside of player
                                //             // Either 2 cards known and last in pile (therefore they could even redraw)
                                //             // Or 3 cards known and 1 is in pile (therefore they can redraw)
                                //             //  - This ASSUMES that the latest_move played is legal (which it should be)
                                //             // self.history[i - 1].known_card_count(discard_considered) >= 2
                                //             // && !self.history[i - 1].player_has_inferred_constraint(action_player, *reveal_i)
                                //             // TODO: [OPTIMIZE] don't call known_card_count twice, use a match statement
                                //             // !self.history[i - 1].player_has_inferred_constraint(action_player, discard_considered) // [OPTIMIZE] Repeated above
                                //             // ^ Imported from above (full_test_replay_11 fails with this)
                                //             (
                                //                 // TODO：[OPTIMIZE]
                                //                 self.history[i - 1].known_card_count(discard_considered) == 2 
                                //                 && !self.history[i - 1].inferred_constraints()[action_player as usize].contains(&discard_considered)
                                //                 && *reveal_i == discard_considered
                                //             ) 
                                //             || (
                                //                 self.lookback_check_2(i, action_player as usize, discard_considered)
                                //             )
                                //             || (
                                //                 // TEST
                                //                 // CASE
                                //                 // All cards dead and player had to have redrawn the same card revealed
                                //                 // As pile does not have the card to discard
                                //                 // Comparable case for RevealRedraw might be if 2 cards are dead + 1 revealed?
                                //                 // Unsure if all known is usable?
                                //                 (
                                //                     bool_all_cards_dead
                                //                     && *reveal_i == discard_considered
                                //                 )
                                //                 // Won't always have this as this requires a forward pass of recalculations to know this
                                //                 // After a new card has been added
                                //                 // && self.history[i-1].impossible_constraints()[6][*reveal_i as usize]
                                //                 // && self.history[i-1].inferred_constraints()[action_player as usize].iter().filter(|c| **c == discard_considered).count() < 2
                                //                 || (
                                //                     // TEMP TEST HACKY UNSURE how this interacts with Ambassador
                                //                     // I wonder if there is a case where P1 RR 4 times in  a row
                                //                     // and somehow this updates wrongly
                                //                     // NOTE: This somehow fixes alot of shit omg
                                //                     // NOTE: If overinference happens it is recommended to check if it does so with this commented out
                                //                     *reveal_i == discard_considered
                                //                     // && reveal_players.len() >= 3 - discard_players.len()
                                //                     // && reveal_players.iter().chain(discard_players.iter()).collect::<AHashSet<_>>().len() >= 3
                                //                     // && reveal_players.iter().collect::<AHashSet<_>>().len() >= 3 - discard_players.len()
                                //                     // Try 3:
                                //                     // After the player reveals
                                //                     // 2 other players discard / reveal (without AMB)
                                //                     // I guess can .retain() if player AMB
                                //                     // Then player discards again / reveals
                                //                     // && reveal_players.iter().filter(|p| **p != player_index).collect::<AHashSet<_>>().len() >= 3 - discard_players.len()
                                //                     && {
                                //                         // TODO: Change this is a really dumb implementation
                                //                         let mut historical_first_time_reveal_count = 0;
                                //                         let mut redraw_stack_pop_count = 0;
                                //                         // Size expected to be small
                                //                         let mut visited_players = [false; 6];
                                //                         let mut visited_players_opp = [false; 6];
                                //                         // visited_players[player_index as usize] = true;
                                //                         for sig_act in self.history[2..i].iter() {
                                //                             if !visited_players[sig_act.player() as usize] {
                                //                                 if let ActionInfo::RevealRedraw { reveal, redraw, .. } = sig_act.action_info() {
                                //                                     if *reveal == discard_considered {
                                //                                         visited_players[sig_act.player() as usize] = true;
                                //                                         historical_first_time_reveal_count += 1;
                                //                                     } else if redraw.is_none() {
                                //                                         // visited_players_opp[sig_act.player() as usize] = true;
                                //                                         // erasure of progress because of AMB or redraw
                                //                                         visited_players = [false; 6];
                                //                                     } else if *redraw == Some(discard_considered) {
                                //                                         redraw_stack_pop_count += 1;
                                //                                     }
                                //                                 }
                                //                             }
                                //                         }
                                //                         // TODO: [OPTIMIZE] I do belief u can use latest public_constraints
                                //                         log::trace!("reveal_players {:?}, discard_players: {:?}", reveal_players, discard_players);
                                //                         log::trace!("visited_players: {:?}", visited_players);
                                //                         log::trace!("self.history[i - 1].public_constraints(): {:?}", self.history[i - 1].public_constraints());
                                //                         log::trace!("reveal + visited_players: {}", reveal_players.iter().zip(visited_players.iter()).enumerate().filter(|(i, (v0, v1))| **v0 || **v1).count());
                                //                         log::trace!("discard_players.len(): {}", discard_players.len());
                                //                         log::trace!("discard_players.len(): {}", discard_players.len());
                                //                         log::trace!("dead before: {}", self.history[i - 1].public_constraints().iter().map(|v| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>());
                                //                         // log::trace!("reveal_players.iter().filter(|p| **p != player_index).count() >= 3 - discard_players.len() = {}", reveal_players.iter().filter(|p| **p != player_index).count() >= 3 - discard_players.len());
                                //                         // reveal_players.iter().enumerate().filter(|(i, v)| *i != player_index as usize && **v).count()
                                //                         // reveal_players.iter().zip(visited_players.iter()).enumerate().filter(|(i, (v0, v1))| *i != player_index as usize && (**v0 || **v1)).count()
                                //                         // reveal_players.iter().zip(visited_players.iter()).enumerate().filter(|(i, (v0, v1))| **v0 || **v1).count()
                                //                         reveal_players.iter().zip(visited_players.iter()).zip(visited_players_opp.iter()).enumerate().filter(|(i, ((v0, v1), v2))| (**v0 || **v1) && !**v2).count()
                                //                         + discard_players.len()  // valid dead from latest move to current iter + dead before
                                //                         + self.history[i - 1].public_constraints().iter().map(|v| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>()
                                //                         // + historical_first_time_reveal_count
                                //                         >= 3 + redraw_stack_pop_count
                                //                         // temp.iter().filter(|p| **p != player_index).count() >= 3 - discard_players.len()
                                //                         // temp.iter().filter(|p| **p != player_index).count() >= 3 - self.public_constraints.iter().map(|v| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>()
                                //                         // Checking for first time reveal_redraws too (+ extra inferred)!
                                //                         // Problem with this is that it can also include the current reveal_redraw in iter_loop => add 1 to deal with that
                                //                         // Its also not so simple
                                //                         // self.history[1].inferred_constraints().iter().map(|v| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>()
                                //                         // + self.public_constraints.iter().map(|v| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>() >= 3
                                //                     }
                                //                     || {
                                //                         *reveal_i == discard_considered && 
                                //                         self.history[i - 1].public_constraints().iter().enumerate().filter(|(p, _)| *p != action_player as usize).map(|(_, v)| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>()
                                //                         + self.history[i - 1].inferred_constraints().iter().enumerate().filter(|(p, _)| *p != action_player as usize).map(|(_, v)| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>()
                                //                         == 2
                                //                     }
                                //                     // false
                                //                 )
                                //             )
                                //             // === OLD ===
                                //             // self.history[i - 1].known_card_count(discard_considered) == 2 
                                //             // // Might be hacky, Idea Is that there are 2 non-pile players we know that have the card
                                //             // // And because of a previous RevealRedraw the last card is between pile and another player
                                //             // && !self.history[i - 1].inferred_constraints()[6].contains(&discard_considered) 
                                //             //     // This below breaks shit
                                //             // // && !self.history[i - 1].inferred_constraints()[action_player as usize].contains(&discard_considered)
                                                
                                //             // && (
                                //             //     // 2 cards outside of player, and player reveals discard_considered
                                //             //     *reveal_i == discard_considered
                                //             //     // 2 cards outside of player, and player redraws discard_considered
                                //             //     // i.e someone revealredraw or ambassador an item into it
                                //             //     // I'm wondering if the lookback_check revealredraw player should also be excluded
                                //             //     // Just like pile above
                                //             //     || self.lookback_check(i - 1, action_player, discard_considered).is_some()
                                //             //     // Temp test change location
                                //             //     // || !self.history[i - 1].inferred_constraints()[action_player as usize].contains(&discard_considered)
                                //             // )
                                //             // === OLD ===
                                //             || self.history[i - 1].known_card_count(discard_considered) == 3
                                //             && !self.history[i - 1].player_has_inferred_constraint(action_player, discard_considered)
                                //             // 3 cards outside of player and (implied that player obviously won't reveal discard_considered)
                                //             // && !self.history[i - 1].player_has_inferred_constraint(action_player, *reveal_i)
                                //         )
                                //         // I guess player could have 1 life and so would have to redraw that card
                                //         // TODO: [OPTIMIZE] the arrangement of these
                                //         || self.history[i - 1].public_constraints()[action_player as usize].len() == 1
                                //     )
                                // } else {
                                //     false
                                // };
                                let need_redraw_update = self.need_redraw_update(i, discard_considered, &illegal_players);
                                // TESTING matching above not letting reveal count to its own redraw
                                if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                    if *reveal_i == discard_considered 
                                    && *redraw_i != Some(*reveal_i) 
                                    && !discard_players.contains(&action_player){
                                        // reveal_players.push(action_player);
                                        reveal_players[action_player as usize] = true;
                                    }
                                    if redraw_i.is_none() {
                                        illegal_players[action_player as usize] = true;
                                        discard_players.retain(|p| *p != action_player);
                                    }
                                }
                                // if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                //     log::trace!("reveal_i: {:?}, *reveal_i: {:?}, redraw_i: {:?}", reveal_i, *reveal_i, redraw_i);
                                //     log::trace!("action_data.action_info(), {:?}", action_data.action_info());
                                //     log::trace!("redraw_i.is_none() = : {}", redraw_i.is_none());
                                //     log::trace!("&&");
                                //     log::trace!("self.history[i - 1].meta_data().public_constraints()[action_player as usize].len() + self.history[i - 1].meta_data().inferred_constraints()[action_player as usize].len() == 2 = {}", self.history[i - 1].meta_data().public_constraints()[action_player as usize].len() + self.history[i - 1].meta_data().inferred_constraints()[action_player as usize].len() == 2);
                                //     log::trace!("!self.history[i - 1].player_has_inferred_constraint(action_player, discard_considered): {:?}", !self.history[i - 1].player_has_inferred_constraint(action_player, discard_considered));
                                //     log::trace!("i: {} self.history: {:?}",i , self.history);
                                //     log::trace!("&&");
                                //     log::trace!("!action_data.player_has_inferred_constraint(action_player, discard_considered) = {}", !self.inferred_constraints[action_player as usize].contains(&discard_considered));
                                //     log::trace!("||");
                                //     log::trace!("(");
                                //     log::trace!("self.public_constraints[action_player as usize].len() + self.inferred_constraints[action_player as usize].len() == 2 = {}", self.public_constraints[action_player as usize].len() + self.inferred_constraints[action_player as usize].len() == 2);
                                //     log::trace!("&&");
                                //     log::trace!("self.player_constraints_all_full(action_player, discard_considered)) = {}", self.inferred_constraints[action_player as usize].iter().all(|&c| c == discard_considered) &&
                                //     self.public_constraints[action_player as usize].iter().all(|&c| c == discard_considered));
                                //     log::trace!(")");
                                //     log::trace!("||");
                                //     log::trace!("(");
                                //     log::trace!("self.history[i - 1].known_card_count(discard_considered) == 2 = {}", self.history[i - 1].known_card_count(discard_considered) == 2);
                                //     log::trace!("self.history[i - 1] = {:?}", self.history[i - 1]);
                                //     log::trace!("&&");
                                //     log::trace!("   (");
                                //     log::trace!("*reveal_i == discard_considered = ?? reveal_i: {:?}, discard_considered: {:?}", *reveal_i, discard_considered);
                                //     log::trace!("||");
                                //     log::trace!("self.lookback_check_2({i}, {action_player}, discard_considered) = {}", self.lookback_check_2(i, action_player as usize, discard_considered));
                                //     log::trace!("   )");
                                //     log::trace!("||");
                                //     log::trace!("   (");
                                //     log::trace!("bool_all_cards_dead = {}", bool_all_cards_dead);
                                //     log::trace!("*reveal_i == discard_considered = {}", *reveal_i == discard_considered);
                                //     log::trace!("reveal_players.len() >= 3 - discard_players.len() = {}", reveal_players.len() >= 3 - discard_players.len());
                                //     log::trace!("reveal_players {:?} discard_players {:?}", reveal_players, discard_players);
                                //     log::trace!("   )");
                                //     log::trace!("||");
                                //     log::trace!("self.history[i - 1].known_card_count(discard_considered) == 3 = {}", self.history[i - 1].known_card_count(discard_considered) == 3);
                                //     log::trace!(")");
                                // }
                                log::trace!("lookback_initial original Discard: {:?}", action_info);
                                log::trace!("lookback_initial considering: player: {} {:?}", action_data.player(), action_data.action_info());
                                
                                log::trace!("need_redraw_update evaluated to {need_redraw_update}");
                                if need_redraw_update {
                                    if let ActionInfo::RevealRedraw { reveal, redraw, .. } = self.history[i].action_info_mut() {
                                        log::trace!("lookback_initial setting redraw to: {:?}", discard_considered);
                                        *redraw = Some(discard_considered);
                                        if Some(*reveal) != *redraw {
                                            self.lookback_1_continual(i);
                                        }
                                        // self.regenerate_path();
                                        // log::trace!("End Regenerate Path lookback_initial C");
                                        // self.printlog();
                                        // // panic!();
                                        // // TODO: [CASE] Knowing the redraw means we need to update the pile earlier!
                                        // return true;
                                        bool_changes = true;
                                        bool_skip_start_update = true;
                                        break 'iter_loop;
                                    }
                                } else {
                                    // return false;
                                    // Case Same player but redraw is None
                                    // TODO: [REFACTOR] This shit is becoming pretty mangled
                                    if let ActionInfo::RevealRedraw { redraw, .. } = self.history[i].action_info() {
                                        if redraw.is_none() {
                                            // Avoid updating start when loop ends
                                            bool_skip_start_update = true;
                                            break 'iter_loop;
                                        }
                                    }
                                }
                            },
                            ActionInfoName::ExchangeDrawChoice => {
                                // return false;
                                bool_skip_start_update = true;
                                break 'iter_loop;
                            }
                            ActionInfoName::Discard => {
                                // Collecting same card discards found along the way
                                if let ActionInfo::Discard { discard } = action_data.action_info() {
                                    log::trace!("saw action_player: {action_player} discard: {:?} in iter", discard);
                                    if *discard == discard_considered {
                                        log::trace!("Pushing action_player: {action_player} into discard_considered");
                                        discard_players.push(action_player);
                                    }
                                }
                            },
                            _ => {},
                        }
                    } else {
                        // TEMP TEST [REFACTOR]
                        // if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                        //     if *reveal_i == discard_considered && *redraw_i != Some(*reveal_i) 
                        //     && !discard_players.contains(&action_player){
                        //         reveal_players.push(action_player);
                        //     }
                        //     if redraw_i.is_none() {
                        //         illegal_players.push(action_player);
                        //     }
                        // }
                        // Collecting same card discards found along the way
                        if let ActionInfo::Discard { discard } = action_data.action_info() {
                            if *discard == discard_considered {
                                log::trace!("Pushing action_player: {action_player} into discard_considered");
                                discard_players.push(action_player);
                            }
                        }
                        // TODO: THEORY CHECK [OPTIMIZE] Do we really need this bool_all_cards_dead? can mirror above?
                        // if bool_all_cards_dead {
                            let action_name = action_data.name();
                            match action_name {
                                ActionInfoName::Discard => {
                                },
                                ActionInfoName::RevealRedraw => {
                                    // TODO: Consider that we could pass a RR, the see a discard, thus adding back into discard_players
                                    // This leads us to look at multiple RevealRedraws after the first one
                                    log::trace!("lookback_initial Discard checking past RevealRedraw");
                                    log::trace!("lookback_initial original player: {}, Discard: {:?}", player_index, action_info);
                                    log::trace!("lookback_initial checked player: {}, RevealRedraw: {:?}", action_player, action_data.action_info());
                                    // TODO: Ok we need to expand this
                                    // Player may have had 2 lives at that point in time, so we don't really know
                                    // let need_redraw_update = if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                    //     !illegal_players.contains(&action_player) // skip if player RR after too (dk which revealredraw the player redrew)
                                    //     && discard_players.contains(&action_player)
                                    //     && redraw_i.is_none()
                                    //     && (
                                    //         (
                                    //             // self.history[i - 1].public_constraints()[action_player as usize].len() + self.history[i - 1].meta_data().inferred_constraints()[action_player as usize].len() > 1
                                    //             // && !self.history[i - 1].player_has_inferred_constraint(action_player, discard_considered)
                                    //             false
                                    //         ) // required for MARKER A)
                                    //         // || (
                                    //         //     self.history[i - 1].public_constraints()[action_player as usize].len() == 1
                                    //         // )
                                    //         || (
                                    //             // I like this
                                    //             self.history[i - 1].impossible_constraints()[action_player as usize][discard_considered as usize]
                                    //         )
                                    //         || (
                                    //             // CASE all cards are dead and known based on latest move
                                    //             // This player had discarded the card
                                    //             // So if they had revealed it in the past, they had to have redrawn it
                                    //             // else they could not have discarded it
                                    //             // because it was impossible for them to have that card the turn before
                                    //             // Here we assume bool_all_cards_dead == true
                                    //             *reveal_i == discard_considered
                                    //             // && bool_all_cards_dead
                                    //             && {
                                    //                 // TODO: REFACTOR
                                    //                 // But also consider may not needing to run this every iteration
                                    //                 let mut historical_first_time_reveal_count = 0;
                                    //                 let mut redraw_stack_pop_count = 0;
                                    //                 // Size expected to be small
                                    //                 let mut visited_players = [false; 6];
                                    //                 let mut visited_players_opp = [false; 6];
                                    //                 // visited_players[player_index as usize] = true;
                                    //                 for sig_act in self.history[2..i].iter() {
                                    //                     if !visited_players[sig_act.player() as usize] {
                                    //                         if let ActionInfo::RevealRedraw { reveal, redraw, .. } = sig_act.action_info() {
                                    //                             if *reveal == discard_considered {
                                    //                                 visited_players[sig_act.player() as usize] = true;
                                    //                                 historical_first_time_reveal_count += 1;
                                    //                             } else if redraw.is_none() {
                                    //                                 // visited_players_opp[sig_act.player() as usize] = true;
                                    //                                 // erasure of progress because of AMB or redraw
                                    //                                 // hmm could there be some stack action here?
                                    //                                 // maybe it kinda nullifies itself? oh tthats just visited_players lol
                                    //                                 visited_players = [false; 6];
                                    //                             } else if *redraw == Some(discard_considered) {
                                    //                                 redraw_stack_pop_count += 1;
                                    //                             }
                                    //                         }
                                    //                     }
                                    //                 }
                                    //                 // let mut temp = reveal_players.clone();
                                    //                 // temp.sort_unstable();
                                    //                 // temp.dedup();
                                    //                 // log::trace!("reveal_players {:?}, discard_players: {:?}", temp, discard_players);
                                    //                 // TODO: REFACTOR
                                    //                 log::trace!("reveal_players {:?}, discard_players: {:?}", reveal_players, discard_players);
                                    //                 log::trace!("visited_players {:?}", visited_players);
                                    //                 log::trace!("reveal + visited_players: {}", reveal_players.iter().zip(visited_players.iter()).enumerate().filter(|(i, (v0, v1))| **v0 || **v1).count());
                                    //                 log::trace!("discard_players.len(): {}", discard_players.len());
                                    //                 log::trace!("dead before: {}", self.history[i - 1].public_constraints().iter().map(|v| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>());
                                    //                 // reveal_players.iter().enumerate().filter(|(i, v)| *i != player_index as usize && **v).count()
                                    //                 // reveal_players.iter().zip(visited_players.iter()).enumerate().filter(|(i, (v0, v1))| *i != player_index as usize && (**v0 || **v1)).count()
                                    //                 // reveal_players.iter().zip(visited_players.iter()).enumerate().filter(|(i, (v0, v1))| **v0 || **v1).count()
                                    //                 reveal_players.iter().zip(visited_players.iter()).zip(visited_players_opp.iter()).enumerate().filter(|(i, ((v0, v1), v2))| (**v0 || **v1) && !**v2).count()
                                    //                 + discard_players.len()  // valid dead from latest move to current iter + dead before
                                    //                 + self.history[i - 1].public_constraints().iter().map(|v| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>()
                                    //                 // + historical_first_time_reveal_count
                                    //                 >= 3 + redraw_stack_pop_count
                                    //                 // temp.iter().filter(|p| **p != player_index).count() >= 3 - self.public_constraints.iter().map(|v| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>()
                                    //                 // Checking for first time reveal_redraws too (+ extra inferred)!
                                    //                 // Problem with this is that it can also include the current reveal_redraw in iter_loop => add 1 to deal with that
                                    //                 // Not so simple
                                    //                 // self.history[1].inferred_constraints().iter().map(|v| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>()
                                    //                 // + self.public_constraints.iter().map(|v| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>() >= 3
                                                    
                                    //             }
                                    //             || {
                                    //                 *reveal_i == discard_considered &&
                                    //                 // had to have withdrawn if other 2 cards are outside player
                                    //                 self.history[i - 1].public_constraints().iter().enumerate().filter(|(p, _)| *p != action_player as usize).map(|(_, v)| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>()
                                    //                 + self.history[i - 1].inferred_constraints().iter().enumerate().filter(|(p, _)| *p != action_player as usize).map(|(_, v)| v.iter().filter(|c| **c == discard_considered).count()).sum::<usize>()
                                    //                 == 2
                                    //             }
                                    //         )
                                    //     )
                                    // } else {
                                    //     false
                                    // };
                                    // // Adding after to prevent reveal counting to its own redraw
                                    // // TEMP TEST [REFACTOR]
                                    // // if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                    // //     if *reveal_i == discard_considered && *redraw_i != Some(*reveal_i) {
                                    // //         reveal_players.push(action_player);
                                    // //     }
                                    // // }
                                    let need_redraw_update = self.need_redraw_update(i, discard_considered, &illegal_players);
                                    if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = action_data.action_info() {
                                        if *reveal_i == discard_considered && *redraw_i != Some(*reveal_i) 
                                        && !discard_players.contains(&action_player){
                                            // reveal_players.push(action_player);
                                            reveal_players[action_player as usize] = true;
                                        }
                                        if redraw_i.is_none() {
                                            illegal_players[action_player as usize] = true;
                                            discard_players.retain(|p| *p != action_player);
                                        }
                                    }
                                    if need_redraw_update {
                                        log::trace!("lookback_initial original Discard: {:?}", action_info);
                                        log::trace!("lookback_initial considering: player: {} {:?}", action_data.player(), action_data.action_info());
                                        if let ActionInfo::RevealRedraw { reveal, redraw, .. } = self.history[i].action_info_mut() {
                                            log::trace!("lookback_initial setting redraw to: {:?}", discard_considered);
                                            *redraw = Some(discard_considered);
                                            // discard_players.retain(|p| *p != action_player);
                                            // TODO: In this case, we may not want to regenerate immediately
                                            // OPTIONS:
                                            //      - Regenerate now
                                            //      - Regenerate and allow another lookback during regeneration
                                            //      - Handle all lookbacks now, then regenerate
                                            // CASE:
                                            //              P1 Discard Duke
                                            //              P2 RR Reveal Duke
                                            //              P3 RR Reveal Duke
                                            //              P2 Discard Duke
                                            //              P3 Discard Duke
                                            //      When P3 Discard Duke, we know that both P2 and P3 redrew Dukes, but never before that
                                            //      So we would need to update both, and early exit may not be ideal
                                            // if Some(*reveal) != *redraw {
                                            //     self.lookback_1_continual(i);
                                            // }
                                            // self.regenerate_path();
                                            // log::trace!("End Regenerate Path lookback_initial C");
                                            // self.printlog();
                                            // panic!();
                                            // TODO: [CASE] Knowing the redraw means we need to update the pile earlier!
                                            // return true;
                                            bool_changes = true;
                                            // In case where its a different player RevealRedraw, don't need to break yet
                                            // iter_loop, break only when its same player
                                            // break 'iter_loop;
                                            // TODO: I have a feeling I need an exclude group for where there are multiple RevealRedraws
                                            // TODO: I have a feeling I only need to kick them out if reveal_redraw is None?
                                            // Like if we pass a reveal == redraw must we kick them?
                                            // What does this do
                                            discard_players.retain(|p| *p != action_player);
                                        }
                                    } else {
                                        // return false;
                                    }
                                },
                                ActionInfoName::ExchangeDrawChoice => {
                                    // Do nothing as its a differ
                                },
                                ActionInfoName::Start
                                |ActionInfoName::StartInferred => {
                                    debug_assert!(false, "You should not be here!");
                                },
                            }
                        // }
                    }
                }
                // Else if its start (which means we did not hit a revealredraw or ambassador)
                // TODO: Update start here (inferred)
                // TODO: Somehow, maybe by regenerating, or not update impossible constraints too
                // TODO: Change, this should only be done for the first time its added for discard
                debug_assert!(self.history[0].name() == ActionInfoName::Start, "wrong Significant Action at index 0!");
                // TODO: Fix bad_push TEMP
                // add 2 same cards case
                // Trying without conditional as Discard means player surely had the card?
                // But i think we need to handle the case where there are 2 dead cards and u need to add another in
                // if !self.history[0].inferred_constraints()[player_index as usize].contains(&discard_considered) {
                if !bool_skip_start_update {
                    log::trace!("lookback_initial processed item: {:?}", self.history.last().unwrap().action_info());
                    log::trace!("lookback_initial adding inferred_constraint to start: (player: {}, card: {:?})", player_index, discard_considered);
                    log::trace!("lookback_initial start before: {:?}", self.history.first().unwrap().meta_data());
                    self.history[0].add_inferred_constraints(player_index as usize, discard_considered);
                    log::trace!("lookback_initial start after: {:?}", self.history.first().unwrap().meta_data());
                    bool_changes = true;
                }
                    // TODO: Have to change calculation for game start for this to work
                    // TODO: Include impossible at game start too!
                    // self.regenerate_path();
                    // log::trace!("End Regenerate Path lookback_initial D");
                    // self.printlog();
                    // return true;
                // }
            },
            _ => {
                // unimplemented!();
            }
        }
        if bool_changes {
            self.regenerate_path();
            log::trace!("End Regenerate Path lookback_initial D");
            self.printlog();
        }
        bool_changes
        // false
    }
    /// Checks if anybody has revealredrawn a card into pile before
    /// return Some(index, player_id) if exists
    /// None
    pub fn lookback_check(&self, history_index: usize, player_id: u8, card: Card) -> Option<(usize, u8)> {
        // TODO: See full_test_repay_1.log
        // If player 4 RR Duke instead of P2
        // might have to update that too a re
        // maybe needs to be not the original player?
        log::trace!("In lookback_check");
        log::trace!("lookback_check history_index: {}, player_id: {}, card: {:?}", history_index, player_id, card);
        for i in (2..=history_index).rev() {
            let player_i = self.history[i].player() as usize;
            if player_i != player_id as usize {
                if let ActionInfo::RevealRedraw { reveal: temp_reveal, redraw: temp_redraw, .. } = self.history[i].action_info() {
                    // if self.history[i - 1].meta_data().public_constraints()[player_i].len() + self.history[i - 1].meta_data().inferred_constraints()[player_i].len() == 2
                    // && !self.history[i - 1].player_has_inferred_constraint(player_i, card) 
                    log::trace!("*reveal == card reveal: {:?} card: {:?}", *temp_reveal, card);
                    log::trace!("Some(*reveal) != *redraw *reveal: {:?}, *redraw: {:?}", *temp_reveal, *temp_redraw);
                    if *temp_reveal == card && Some(*temp_reveal) != *temp_redraw {
                        return Some((i, self.history[i].player()));
                    }
                }
            } else {
                // TODO: THEORY CHECK
                // Technically arent there cases where this person could have redrawn that card into the pile
                // Which is how the player later on could reveal it
                // As per the case in lookback_initial
                // if self.history[i].name() == ActionInfoName::RevealRedraw {
                //     return None
                // }
                match self.history[i].action_info() {
                    ActionInfo::RevealRedraw { reveal, redraw, relinquish } => {
                        // TODO: THEORY Handle relinquish
                        if redraw.is_none() {
                            return None;
                        }
                    },
                    ActionInfo::ExchangeDrawChoice { draw, relinquish } => {
                        // TODO: THEORY Handle relinquish cases
                        return None;
                    },
                    ActionInfo::Start
                    | ActionInfo::StartInferred => {debug_assert!(false, "Start should only be in index = 0")},
                    ActionInfo::Discard { .. } => {},
                }
            }
        }
        None
    }
    
    /// Checks for sase where player cannot possibly have the card if not for the redraw
    ///  => all 3 cards are outside of the player at history_index/revealredraw_index
    ///  => impossible for player to have that card alive
    pub fn lookback_check_2(&self, history_index: usize, player_id: usize, card: Card) -> bool {
        // I think for basically the same reasons as I had to rerepresent the code
        // I would need to rerun the calculations all the way, to figure out if its impossible
        // The best way would be to rely on impossible_constraints
        self.history[history_index - 1].impossible_constraints()[player_id][card as usize]
        // Getting first position where some player revealredraw the card to add it into pile network
        // if let Some(pos) = self.history
        // .iter()
        // .position(
        //     |a| 
        //     match a.action_info() {
        //         ActionInfo::RevealRedraw{reveal, redraw, relinquish} => {
        //             a.player() != player_id 
        //             && *reveal == card 
        //             && redraw.is_none()
        //             // If redraw is_some() it will be known in inferred cards
        //         },
        //         _ => {
        //             // There is probably an ambassador private information case to handle too
        //             false
        //         }
        //     }
        // ) {
        //     let mut group_constraint = CompressedGroupConstraint::zero();
        //     for i in pos..history_index {
        //         let action_player = self.history[i].player() as usize;
        //         match self.history[i].action_info() {
        //             ActionInfo::RevealRedraw { reveal, redraw, relinquish } => {
        //                 if redraw.is_some() {
        //                     if !group_constraint.get_player_flag(action_player) {
        //                         group_constraint.
        //                     }
        //                     group_constraint.set_player_flag(action_player, true);
        //                     group_constraint.set_player_flag(6, true);
        //                 }
        //             },
        //             ActionInfo::ExchangeDrawChoice { draw, relinquish } => {
        //                 // Handle private information in future
        //                 // unimplemented!()
        //                 group_constraint.set_player_flag(action_player, true);
        //                 group_constraint.set_player_flag(6, true);
        //             },
        //             ActionInfo::Start => {
        //                 debug_assert!("Start should only be in index 0");
        //             },
        //             ActionInfo::Discard { discard } => {
        //                 if group_constraint.get_player_flag(action_player) {

        //                 }
        //             },
        //         }
        //     }
        // }
        // false
    }
    // pub fn lookback_check_3_fwd_bwd_pass(&self, index: usize, card_of_interest: Card) -> bool {
        
    //     /// Actually combine all into here after that!
    //     /// Looksback at a particular index that is a RevealRedraw(reveal=Card, redraw=None)
    //     /// If reveal == Card of interest (latest reveal_considered of discard_considered)
    //     ///     Determines if just before the revealredraw, if the player had to have redrawn the same card
    //     ///     Because the player cannot have 2 of that card (as the rest are outside the player)
    //     ///     And the player must therefore redraw it, to be able to revealdiscard later on
    //     /// Assumes latest move ran is Discard or RevealRedraw
    //     /// TODO: [OPTIMIZE] can collect bwd pass states during outer loop
    //     let player_of_interest = self.history[index].player() as usize;
    //     log::trace!("index: {index} player_of_interest: {player_of_interest}, card_of_interest: {:?}", card_of_interest);
    //     log::trace!("last move in history: player: {:?}, {:?}", self.history[self.history.len() - 1], self.history[self.history.len() - 1].action_info());
    //     let bool_2_cards_outside_player = if let ActionInfo::RevealRedraw { reveal, redraw , .. }= self.history[index].action_info() {
    //         if *reveal == card_of_interest && redraw.is_none() {
    //             // TODO: Add logic here
    //             // Do Backpass
    //             // Test backpass check
    //             // Do forward pass
    //             // Test forward pass check
    //             // NOTE: forward_pass_group and backward_pass_group are used differently!
    //             //      they only use GroupConstraint as a convenient datastructure
                
    //             // Add initial to backward_pass_group 
    //             // backward_pass_group only counts revealed/discarded cards in count_alive()
    //             let mut backward_pass_group = CompressedGroupConstraint::zero();
    //             backward_pass_group.set_player_flag(self.history[self.history.len() - 1].player() as usize, true);
    //             backward_pass_group.add_alive_count(1);
    //             let mut forward_pass_group = CompressedGroupConstraint::zero();
    //             // backward pass
    //             log::trace!("backwardpass start: {:?}", backward_pass_group);
    //             log::trace!("fwd_bwd_pass last move: {:?}", self.history[self.history.len() - 1]);
    //             for i in (index+1..self.history.len()-1).rev() {
    //                 let player_i = self.history[i].player() as usize;
    //                 log::trace!("backward_pass checking i: {i} player_i: {player_i}, move: {:?}", self.history[i].action_info());
    //                 match self.history[i].action_info() {
    //                     ActionInfo::Discard { discard: discard_i } => {
    //                         if *discard_i == card_of_interest {
    //                             backward_pass_group.set_player_flag(player_i, true);
    //                             backward_pass_group.add_alive_count(1);
    //                         }
    //                     },
    //                     ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, relinquish } => {
    //                         if redraw_i.is_none() && self.history[i].player() == player_of_interest as u8 {
    //                             // illegal
    //                             return false
    //                         }
    //                         if *reveal_i == card_of_interest {
    //                             // Reflecting that the player had this card before move i
    //                             if *relinquish == Some(card_of_interest) {
    //                                 // No check as it went from player to pile
    //                                 // Not setting pile as we don't actually need to use it
    //                                 // backward_pass_group.set_player_flag(6, true);
    //                                 backward_pass_group.add_alive_count(1);
    //                             } else if relinquish.is_none() {
    //                                 if !backward_pass_group.get_player_flag(player_i) {
    //                                     backward_pass_group.set_player_flag(player_i, true);
    //                                     backward_pass_group.add_alive_count(1);
    //                                 }
    //                             }
    //                         } else if *redraw_i == Some(card_of_interest) {
    //                             // Reflecting that the pile had this card before move i
    //                             if !backward_pass_group.get_player_flag(player_i) { // This check is rightly player_i
    //                                 // Not setting pile as we don't actually need to use it
    //                                 // backward_pass_group.set_player_flag(6, true);
    //                                 backward_pass_group.add_alive_count(1);
    //                             }
    //                         } 
    //                     },
    //                     ActionInfo::ExchangeDrawChoice { draw, relinquish } => {
    //                         if self.history[i].player() == player_of_interest as u8 {
    //                             // illegal
    //                             return false
    //                         }
    //                     },
    //                     ActionInfo::Start => {},
    //                     ActionInfo::StartInferred => {},
    //                 }
    //                 log::trace!("backwardpass i: {:?}", backward_pass_group);
    //             }
    //             log::trace!("backward_pass_group: {:?}", backward_pass_group);
    //             if !backward_pass_group.get_player_flag(player_of_interest) {
    //                 log::trace!("lookback_check_3_fwd_bwd_pass early exit: {}", false);
    //                 return false;
    //             }
    //             // forward pass
    //             for i in 2..index {
    //                 let player_i = self.history[i].player() as usize;
    //                 match self.history[i].action_info() {
    //                     ActionInfo::Start => {},
    //                     ActionInfo::StartInferred => {},
    //                     ActionInfo::Discard { discard } => {
    //                         if *discard == card_of_interest {
    //                             forward_pass_group.add_dead_count(1);
    //                         }
    //                     },
    //                     ActionInfo::RevealRedraw { reveal, redraw, relinquish } => {
    //                         if *reveal == card_of_interest {
    //                             // relinquish first
    //                             if relinquish.is_none() {
    //                                 if redraw.is_none() {
    //                                     if !forward_pass_group.get_player_flag(player_i) {
    //                                         forward_pass_group.set_player_flag(player_i, true);
    //                                         forward_pass_group.set_player_flag(6, true);
    //                                         forward_pass_group.add_alive_count(1);
    //                                     }
    //                                 } else if *redraw == Some(card_of_interest) {
    //                                     if !forward_pass_group.get_player_flag(player_i) {
    //                                         forward_pass_group.set_player_flag(player_i, true);
    //                                         forward_pass_group.add_alive_count(1);
    //                                     }
    //                                 } else {
    //                                     if !forward_pass_group.get_player_flag(player_i) { // This rightly checks player_i
    //                                         forward_pass_group.set_player_flag(6, true);
    //                                         forward_pass_group.add_alive_count(1);
    //                                     }
    //                                 }
    //                             }
    //                         } else if redraw.is_none() || redraw.unwrap() == card_of_interest {
    //                             // Case when different reveal, and can redraw any kind of card
    //                             // Case when redraw the card_of_interest
    //                             // Case when different reveal and relinquish == reveal (it always is)
    //                             if forward_pass_group.get_player_flag(6) {
    //                                 forward_pass_group.set_player_flag(player_i, true);
    //                             }
    //                         }
    //                     },
    //                     ActionInfo::ExchangeDrawChoice { draw, relinquish } => {
    //                         // TODO: Technically, have to consider what draw and relinquish could be
    //                         if forward_pass_group.get_player_flag(6) {
    //                             forward_pass_group.set_player_flag(player_i, true);
    //                         }
    //                     },
    //                 }
    //             }
    //             log::trace!("forward_pass_group: {:?}", forward_pass_group);
    //             if !forward_pass_group.get_player_flag(player_of_interest) {
    //                 if forward_pass_group.count_alive() + forward_pass_group.count_dead() == 2 {
    //                     log::trace!("lookback_check_3_fwd_bwd_pass A : {}", true);
    //                     return true;
    //                 }
    //                 if forward_pass_group.count_dead() + backward_pass_group.count_alive() == 3 {
    //                     log::trace!("lookback_check_3_fwd_bwd_pass B : {}", true);
    //                     return true;
    //                 }
    //             }
    //         }
    //     };
    //     log::trace!("lookback_check_3_fwd_bwd_pass: {}", false);
    //     // false
    // }
    /// Returns the a vector of CompressedGroupConstraints, each of which represents the location of a unique card
    /// The combination returned is any combination with the minimum total unique cards possible
    /// Assumes the latest move is RevealRedraw reveal, redraw = None, or Discard
    pub fn backward_pass_minimum_cards_shown(&self, index: usize, card_of_interest: Card) -> Vec<CompressedGroupConstraint> {
        let mut backward_pass_group = CompressedGroupConstraint::zero();
        backward_pass_group.set_player_flag(self.history[self.history.len() - 1].player() as usize, true);
        // backward_pass_group.add_alive_count(1);
        backward_pass_group.add_total_count(1);
        // Prob can optimize to use u8s
        let mut backward_pass_groups: Vec<CompressedGroupConstraint> = Vec::with_capacity(3);
        backward_pass_groups.push(backward_pass_group);
        self.backward_pass_minimum_cards_shown_recurse(self.history.len() - 2, index, card_of_interest, backward_pass_groups.clone())
    }
    pub fn backward_pass_minimum_cards_shown_recurse(&self, index_loop: usize, index_of_interest: usize, card_of_interest: Card, backward_pass_groups: Vec<CompressedGroupConstraint>) -> Vec<CompressedGroupConstraint> {
        if index_loop == index_of_interest {
            return backward_pass_groups
        }
        let player_i = self.history[index_loop].player() as usize;
         

        match self.history[index_loop].action_info() {
            ActionInfo::Discard { discard } => {
                if *discard == card_of_interest {
                    let mut recurse_groups = backward_pass_groups.clone();
                    let mut new_groups = CompressedGroupConstraint::zero();
                    new_groups.set_player_flag(player_i, true);
                    new_groups.set_total_count(1);
                    recurse_groups.push(new_groups);
                    return self.backward_pass_minimum_cards_shown_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups)
                }
            },
            ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, relinquish } => {
                let mut min_backward_pass_groups: Option<Vec<CompressedGroupConstraint>> = None;
                let bool_some_group_pile_flag_is_true: bool = backward_pass_groups.iter().any(|g| g.get_player_flag(6));
                'outer: for (index_group, backward_pass_group) in backward_pass_groups.iter().enumerate() {
                    if *reveal_i == card_of_interest {
                        // Reflecting that the player had this card before move i
                        if *relinquish == Some(card_of_interest) {
                            // No check as it went from player to pile
                            // Not setting pile as we don't actually need to use it
                            // backward_pass_group.set_player_flag(6, true);
                            // backward_pass_group.add_alive_count(1);
                            // backward_pass_group.add_total_count(1);
                            // TESTING, was thinking i might need to consider how the pile moves too?
                            // Also consider needing to subtract?
                            if !bool_some_group_pile_flag_is_true {
                                let mut recurse_groups = backward_pass_groups.clone();
                                let mut new_groups = CompressedGroupConstraint::zero();
                                new_groups.set_player_flag(player_i, true);
                                new_groups.set_total_count(1);
                                recurse_groups.push(new_groups);
                                let found_group = self.backward_pass_minimum_cards_shown_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups);
                                min_backward_pass_groups = Some(
                                    match min_backward_pass_groups.take() {
                                        Some(curr) if found_group.len() < curr.len() => found_group,
                                        Some(curr) => curr,
                                        None => found_group,
                                    }
                                );
                            } else {
                                if backward_pass_group.get_player_flag(6) {
                                    // This is greedy, may consider backtracking for min
                                    let mut recurse_groups = backward_pass_groups.clone();
                                    recurse_groups[index_group].set_player_flag(6, false);
                                    recurse_groups[index_group].set_player_flag(player_i, true);
                                    let found_group = self.backward_pass_minimum_cards_shown_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups);
                                    min_backward_pass_groups = Some(
                                        match min_backward_pass_groups.take() {
                                            Some(curr) if found_group.len() < curr.len() => found_group,
                                            Some(curr) => curr,
                                            None => found_group,
                                        }
                                    );
                                }
                                continue 'outer;
                            }
                        } else if relinquish.is_none() {
                            if redraw_i.is_none() {
                                if !bool_some_group_pile_flag_is_true {
                                    let mut recurse_groups = backward_pass_groups.clone();
                                    let mut new_groups = CompressedGroupConstraint::zero();
                                    new_groups.set_player_flag(player_i, true);
                                    new_groups.set_total_count(1);
                                    recurse_groups.push(new_groups);
                                    let found_group = self.backward_pass_minimum_cards_shown_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups);
                                    min_backward_pass_groups = Some(
                                        match min_backward_pass_groups.take() {
                                            Some(curr) if found_group.len() < curr.len() => found_group,
                                            Some(curr) => curr,
                                            None => found_group,
                                        }
                                    );
                                } else {
                                    if backward_pass_group.get_player_flag(6) {
                                        let mut recurse_groups = backward_pass_groups.clone();
                                        recurse_groups[index_group].set_player_flag(6, false);
                                        recurse_groups[index_group].set_player_flag(player_i, true);
                                        let found_group = self.backward_pass_minimum_cards_shown_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups);
                                        min_backward_pass_groups = Some(
                                            match min_backward_pass_groups.take() {
                                                Some(curr) if found_group.len() < curr.len() => found_group,
                                                Some(curr) => curr,
                                                None => found_group,
                                            }
                                        );
                                    }
                                    continue 'outer;
                                }
                            } else if *redraw_i == Some(card_of_interest) {
                                // TODO: Actually order of discard then revealredraw matters!
                                // So i guess I should track in single_card_flags too huh
                                // P2 Discard DUK
                                // P2 RR DUK
                                // => 2 DUKS
                                // P2 RR DUK
                                // P2 Discard DUK
                                // => 1 DUK
                                let mut recurse_groups = backward_pass_groups.clone();
                                let mut new_groups = CompressedGroupConstraint::zero();
                                new_groups.set_player_flag(player_i, true);
                                new_groups.set_total_count(1);
                                recurse_groups.push(new_groups);
                                return self.backward_pass_minimum_cards_shown_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups)
                            } else {
                                if !bool_some_group_pile_flag_is_true {
                                    let mut recurse_groups = backward_pass_groups.clone();
                                    let mut new_groups = CompressedGroupConstraint::zero();
                                    new_groups.set_player_flag(player_i, true);
                                    new_groups.set_total_count(1);
                                    recurse_groups.push(new_groups);
                                    let found_group = self.backward_pass_minimum_cards_shown_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups);
                                    min_backward_pass_groups = Some(
                                        match min_backward_pass_groups.take() {
                                            Some(curr) if found_group.len() < curr.len() => found_group,
                                            Some(curr) => curr,
                                            None => found_group,
                                        }
                                    );
                                } else {
                                    if backward_pass_group.get_player_flag(6) {
                                        let mut recurse_groups = backward_pass_groups.clone();
                                        recurse_groups[index_group].set_player_flag(6, false);
                                        recurse_groups[index_group].set_player_flag(player_i, true);
                                        let found_group = self.backward_pass_minimum_cards_shown_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups);
                                        min_backward_pass_groups = Some(
                                            match min_backward_pass_groups.take() {
                                                Some(curr) if found_group.len() < curr.len() => found_group,
                                                Some(curr) => curr,
                                                None => found_group,
                                            }
                                        );
                                    }
                                    continue 'outer;
                                }
                            }
                        }
                    } else if *redraw_i == Some(card_of_interest) {
                        // Reflecting that the pile had this card before move i
                        let bool_some_group_player_i_flag_is_true = backward_pass_groups.iter().any(|g| g.get_player_flag(player_i));
                        if !bool_some_group_player_i_flag_is_true {
                            let mut recurse_groups = backward_pass_groups.clone();
                            let mut new_groups = CompressedGroupConstraint::zero();
                            new_groups.set_player_flag(6, true);
                            new_groups.set_total_count(1);
                            recurse_groups.push(new_groups);
                            let found_group = self.backward_pass_minimum_cards_shown_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups);
                            min_backward_pass_groups = Some(
                                match min_backward_pass_groups.take() {
                                    Some(curr) if found_group.len() < curr.len() => found_group,
                                    Some(curr) => curr,
                                    None => found_group,
                                }
                            );
                        } else {
                            if backward_pass_group.get_player_flag(player_i) {
                                let mut recurse_groups = backward_pass_groups.clone();
                                recurse_groups[index_group].set_player_flag(6, true);
                                recurse_groups[index_group].set_player_flag(player_i, false);
                                let found_group = self.backward_pass_minimum_cards_shown_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups);
                                min_backward_pass_groups = Some(
                                    match min_backward_pass_groups.take() {
                                        Some(curr) if found_group.len() < curr.len() => found_group,
                                        Some(curr) => curr,
                                        None => found_group,
                                    }
                                );
                            }
                            continue 'outer;
                        }
                    } else {
                        // redraw.is_none() && *reveal_i != card_of_interest
                        // if relinquish.is_none() {
                        //     if backward_pass_group.get_player_flag(player_i) {
                        //         backward_pass_group.set_player_flag(player_i, false);
                        //         // backward_pass_group.sub_alive_count(1);
                        //         backward_pass_group.sub_total_count(1);
                        //     }
                        // }
                        // handleWhat if relinquish is known?

                        // redraw is_some() -> no need for change
                        // redraw is_none9) -> handled in illegal_players
                        min_backward_pass_groups = Some(self.backward_pass_minimum_cards_shown_recurse(index_loop - 1, index_of_interest, card_of_interest, backward_pass_groups.clone()));
                    }
                }
                // TODO: THINK about unwrap
                return min_backward_pass_groups.unwrap()
            },
            ActionInfo::ExchangeDrawChoice { draw, relinquish } => {
                // TODO:
            },
            ActionInfo::Start
            | ActionInfo::StartInferred => {},
        }
        // some kind of min function
        self.backward_pass_minimum_cards_shown_recurse(index_loop - 1, index_of_interest, card_of_interest, backward_pass_groups)
    }
    /// Returns the a vector of CompressedGroupConstraints, each of which represents the location of a unique card
    /// The combination returned is any combination with the minimum total unique cards possible
    /// Assumes the latest move is RevealRedraw reveal = =card_of_interest, redraw = None, or Discard
    pub fn backward_max_player_cards(&self, index: usize, card_of_interest: Card) -> u8 {
        log::trace!("backward_max_player_cards handled: player: {}, action: {:?}", self.history[index].player(), self.history[index].action_info());
        let mut backward_pass_group = CompressedGroupConstraint::zero();
        backward_pass_group.set_player_flag(self.history[self.history.len() - 1].player() as usize, true);
        // backward_pass_group.add_alive_count(1);
        backward_pass_group.add_total_count(1);
        // Prob can optimize to use u8s
        let mut backward_pass_groups: Vec<CompressedGroupConstraint> = Vec::with_capacity(3);
        backward_pass_groups.push(backward_pass_group);
        log::trace!("backward_pass_groups after adding latest move: {:?}", backward_pass_groups);
        let max_player_cards_combination = self.backward_max_player_cards_recurse(self.history.len() - 2, index, card_of_interest, backward_pass_groups.clone()).unwrap_or(Vec::with_capacity(0));
        log::trace!("max_player_cards_combination returned: {:?}", max_player_cards_combination);
        log::trace!("backward_max_player_cards player: {:?}, action: {:?}, max_count: {}", self.history[index].player(), self.history[index].action_info(), max_player_cards_combination.iter().filter(|group| group.get_player_flag(self.history[index].player() as usize)).count() as u8);
        // 1 is added because we do not check the index_of_interest!
        // and it adds another of that card for player of interest
        max_player_cards_combination.iter().filter(|group| group.get_player_flag(self.history[index].player() as usize)).count() as u8
    }
    /// NOTE: It is quite likely that in future
    /// Will need to use the actual proper forward pass
    /// And a backward pass including all possible cards TT
    /// TODO: Change the redraw logic to try both cases
    /// TODO: Add ambassador logic
    /// TODO: Check all logic
    pub fn backward_max_player_cards_recurse(&self, index_loop: usize, index_of_interest: usize, card_of_interest: Card, backward_pass_groups: Vec<CompressedGroupConstraint>) -> Option<Vec<CompressedGroupConstraint>> {

        let player_of_interest = self.history[index_of_interest].player() as usize;
        // Can't have more than 3 cards in the game!
        if backward_pass_groups.len() > 3 {
            log::trace!("backward_pass_groups len too long!: {:?}", backward_pass_groups);
            return None
        }
        // Goal here is to check if the reached state is a legitimate state to be checked
        if index_loop == index_of_interest - 1{
            // Do check here
            // but i think will need to do more than just the card_of_interest next
            // The revealed card_of_interest won't be in the index_loop constraint
            // Think will need to do a legit forward pass at this rate
            let mut after_reveal_constraint = self.history[index_loop].clone();
            if !after_reveal_constraint.inferred_constraints()[player_of_interest].contains(&card_of_interest) {
                after_reveal_constraint.add_inferred_constraints(player_of_interest, card_of_interest);
            }

            let total_known_cards = after_reveal_constraint.public_constraints().iter().map(|v| v.iter().filter(|c| **c == card_of_interest).count() as u8).sum::<u8>()
            + after_reveal_constraint.inferred_constraints().iter().map(|v| v.iter().filter(|c| **c == card_of_interest).count() as u8).sum::<u8>();
            // TODO: [OPTIMIZE] THIS can just use the thing above
            let mut total_required_cards: [u8; 7] = [0; 7];
            after_reveal_constraint.public_constraints()
            .iter()
            .enumerate()
            .for_each(|(p, v)| total_required_cards[p] += v.iter().filter(|c| **c == card_of_interest).count() as u8);
            after_reveal_constraint.inferred_constraints()
            .iter()
            .enumerate()
            .for_each(|(p, v)| total_required_cards[p] += v.iter().filter(|c| **c == card_of_interest).count() as u8);

            let mut degrees_of_freedom: u8 = 0;
            let mut backward_pass_card_counts: [u8; 7] = [0; 7];
            backward_pass_groups.iter()
            .for_each(
                |group| {
                    if let Some(player) = group.iter_true_player_flags().next() {
                        backward_pass_card_counts[player] += 1;
                    }
                }
            );
            backward_pass_card_counts.iter().zip(total_required_cards.iter())
            .for_each(|(b, r)| {
                degrees_of_freedom += if b > r {
                    b - r
                } else {
                    0
                }
            });
            // impossibe state
            if degrees_of_freedom + total_known_cards> 3 {
                log::trace!("after_reveal_constraint : {:?}", after_reveal_constraint);
                log::trace!("total_required_cards : {:?}", total_required_cards);
                log::trace!("backward_pass_groups to {:?}", backward_pass_groups);
                log::trace!("backward_pass_card_counts : {:?}", backward_pass_card_counts);
                log::trace!("backward_pass_groups evaluated: {:?}", false);
                log::trace!("total_known_cards : {:?}", total_known_cards);
                log::trace!("degrees_of_freedom : {:?}", degrees_of_freedom);
                return None
            } 
            log::trace!("after_reveal_constraint : {:?}", after_reveal_constraint);
            log::trace!("total_required_cards : {:?}", total_required_cards);
            log::trace!("backward_pass_groups to {:?}", backward_pass_groups);
            log::trace!("backward_pass_card_counts : {:?}", backward_pass_card_counts);
            log::trace!("backward_pass_groups evaluated: {:?}", false);
            log::trace!("total_known_cards : {:?}", total_known_cards);
            log::trace!("degrees_of_freedom : {:?}", degrees_of_freedom);
            return Some(backward_pass_groups)
        } 
         
        let player_i = self.history[index_loop].player() as usize;
        match self.history[index_loop].action_info() {
            ActionInfo::Discard { discard } => {
                if *discard == card_of_interest {
                    let mut recurse_groups = backward_pass_groups.clone();
                    let mut new_groups = CompressedGroupConstraint::zero();
                    new_groups.set_player_flag(player_i, true);
                    new_groups.set_total_count(1);
                    recurse_groups.push(new_groups);
                    log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                                    log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                log::trace!("backward_pass_groups is : {:?}", &recurse_groups);
                    return self.backward_max_player_cards_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups)
                }
            },
            ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, relinquish } => {
                let mut max_backward_pass_groups: Option<Vec<CompressedGroupConstraint>> = None;
                let bool_some_group_pile_flag_is_true: bool = backward_pass_groups.iter().any(|g| g.get_player_flag(6));
                // TODO: OPTIMIZE Its actually better to bring this loop inside, as the conditionals are invariant...
                'outer: for (index_group, backward_pass_group) in backward_pass_groups.iter().enumerate() {
                    if *reveal_i == card_of_interest {
                        // Reflecting that the player had this card before move i
                        if *relinquish == Some(card_of_interest) {
                            // No check as it went from player to pile
                            // Not setting pile as we don't actually need to use it
                            // backward_pass_group.set_player_flag(6, true);
                            // backward_pass_group.add_alive_count(1);
                            // backward_pass_group.add_total_count(1);
                            // TESTING, was thinking i might need to consider how the pile moves too?
                            // Also consider needing to subtract?
                            let mut recurse_groups = backward_pass_groups.clone();
                            // TODO: THINK It could be either a new card went from pile to player or player redrew same card
                            // CASES player redraws same card
                            // CASES player redraws card from pile so there are 2 cards?
                            // TODO: Actually maybe should move loop inside, then I can do each case considering if there are some combinations in the deck
                            if backward_pass_group.get_player_flag(6) {
                                // CASE card went from pile to player
                                recurse_groups[index_group].set_player_flag(player_i, true);
                                recurse_groups[index_group].set_player_flag(6, false);
                                log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                log::trace!("max_backward_pass_groups considering case where pile card moves to player");
                                log::trace!("backward_pass_groups is : {:?}", &recurse_groups);
                            } else if backward_pass_group.get_player_flag(player_i) {
                                // CASES player redraws same card 
                                let mut new_groups = CompressedGroupConstraint::zero();
                                new_groups.set_player_flag(player_i, true);
                                new_groups.set_total_count(1);
                                recurse_groups.push(new_groups);
                                log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                log::trace!("backward_pass_groups is : {:?}", &recurse_groups);
                            }
                            if let Some(found_group) = self.backward_max_player_cards_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups) {
                                max_backward_pass_groups = Some(
                                    match max_backward_pass_groups.take() {
                                        Some(curr) => {
                                            if curr.iter().filter(|group| group.get_player_flag(player_i)).count()
                                            < found_group.iter().filter(|group| group.get_player_flag(player_i)).count() {
                                                found_group
                                            } else {
                                                curr
                                            }
                                        },
                                        None => found_group,
                                    }
                                );
                            }
                        } else if relinquish.is_none() {
                            if redraw_i.is_none() {
                                // CASE when the player did not relinquish the card
                                let mut recurse_groups = backward_pass_groups.clone();
                                // Check as there may be a group where player_i has the card already (don't double add)
                                if backward_pass_group.get_player_flag(player_i) {
                                    let mut new_groups = CompressedGroupConstraint::zero();
                                    new_groups.set_player_flag(player_i, true);
                                    new_groups.set_total_count(1);
                                    recurse_groups.push(new_groups);
                                }
                                log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                log::trace!("max_backward_pass_groups considering case where player redrew card");
                                log::trace!("backward_pass_groups is : {:?}", &recurse_groups);
                                if let Some(found_group) = self.backward_max_player_cards_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups) {
                                    max_backward_pass_groups = Some(
                                        match max_backward_pass_groups.take() {
                                            Some(curr) => {
                                                if curr.iter().filter(|group| group.get_player_flag(player_i)).count()
                                                < found_group.iter().filter(|group| group.get_player_flag(player_i)).count() {
                                                    found_group
                                                } else {
                                                    curr
                                                }
                                            },
                                            None => found_group,
                                        }
                                    );
                                }

                                if bool_some_group_pile_flag_is_true {
                                    // CASE when the player did relinquish the card
                                    if backward_pass_group.get_player_flag(6) {
                                        let mut recurse_groups = backward_pass_groups.clone();
                                        recurse_groups[index_group].set_player_flag(6, false);
                                        recurse_groups[index_group].set_player_flag(player_i, true);
                                        log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                        log::trace!("max_backward_pass_groups considering case where player relinquish card");
                                        log::trace!("backward_pass_groups is : {:?}", &recurse_groups);
                                        if let Some(found_group) = self.backward_max_player_cards_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups) {
                                            max_backward_pass_groups = Some(
                                                match max_backward_pass_groups.take() {
                                                    Some(curr) => {
                                                        if curr.iter().filter(|group| group.get_player_flag(player_i)).count()
                                                        < found_group.iter().filter(|group| group.get_player_flag(player_i)).count() {
                                                            found_group
                                                        } else {
                                                            curr
                                                        }
                                                    },
                                                    None => found_group,
                                                }
                                            );
                                        }
  
                                    }
                                    continue 'outer;
                                }
                            } else if *redraw_i == Some(card_of_interest) {
                                // TODO: Actually order of discard then revealredraw matters!
                                // So i guess I should track in single_card_flags too huh
                                // P2 Discard DUK
                                // P2 RR DUK
                                // => 2 DUKS
                                // P2 RR DUK
                                // P2 Discard DUK
                                // => 1 DUK
                                // TODO: THEORY FIX what if they do this twice?
                                // TODO: Add a check for if player already has the card
                                // TODO: go 2 different paths for if its the current card or if its another card
                                let mut recurse_groups = backward_pass_groups.clone();
                                if backward_pass_group.get_player_flag(6) {
                                    recurse_groups[index_group].set_player_flag(player_i, true);
                                    recurse_groups[index_group].set_player_flag(6, false);
                                } else {
                                    let mut new_groups = CompressedGroupConstraint::zero();
                                    new_groups.set_player_flag(player_i, true);
                                    new_groups.set_total_count(1);
                                    recurse_groups.push(new_groups);
                                }
                                log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                log::trace!("backward_pass_groups is : {:?}", &recurse_groups);
                                return self.backward_max_player_cards_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups)
                            } else {
                                if !bool_some_group_pile_flag_is_true {
                                    let mut recurse_groups = backward_pass_groups.clone();
                                    let mut new_groups = CompressedGroupConstraint::zero();
                                    new_groups.set_player_flag(player_i, true);
                                    new_groups.set_total_count(1);
                                    recurse_groups.push(new_groups);
                                    log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                    log::trace!("backward_pass_groups is : {:?}", &recurse_groups);
                                    if let Some(found_group) = self.backward_max_player_cards_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups) {
                                        max_backward_pass_groups = Some(
                                            match max_backward_pass_groups.take() {
                                                Some(curr) => {
                                                    if curr.iter().filter(|group| group.get_player_flag(player_i)).count()
                                                    < found_group.iter().filter(|group| group.get_player_flag(player_i)).count() {
                                                        found_group
                                                    } else {
                                                        curr
                                                    }
                                                },
                                                None => found_group,
                                            }
                                        );
                                    }

                                } else {
                                    if backward_pass_group.get_player_flag(6) {
                                        let mut recurse_groups = backward_pass_groups.clone();
                                        recurse_groups[index_group].set_player_flag(6, false);
                                        recurse_groups[index_group].set_player_flag(player_i, true);
                                        log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                        log::trace!("backward_pass_groups is : {:?}", &recurse_groups);
                                        if let Some(found_group) = self.backward_max_player_cards_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups) {
                                            max_backward_pass_groups = Some(
                                                match max_backward_pass_groups.take() {
                                                    Some(curr) => {
                                                        if curr.iter().filter(|group| group.get_player_flag(player_i)).count()
                                                        < found_group.iter().filter(|group| group.get_player_flag(player_i)).count() {
                                                            found_group
                                                        } else {
                                                            curr
                                                        }
                                                    },
                                                    None => found_group,
                                                }
                                            );
                                        }
                                        
                                    }
                                    continue 'outer;
                                }
                            }
                        }
                    } else if *redraw_i == Some(card_of_interest) {
                        // Reflecting that the pile had this card before move i
                        let bool_some_group_player_i_flag_is_true = backward_pass_groups.iter().any(|g| g.get_player_flag(player_i));
                        if !bool_some_group_player_i_flag_is_true {
                            let mut recurse_groups = backward_pass_groups.clone();
                            let mut new_groups = CompressedGroupConstraint::zero();
                            new_groups.set_player_flag(6, true);
                            new_groups.set_total_count(1);
                            recurse_groups.push(new_groups);
                            log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                            log::trace!("backward_pass_groups is : {:?}", &recurse_groups);
                            if let Some(found_group) = self.backward_max_player_cards_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups) {
                                max_backward_pass_groups = Some(
                                    match max_backward_pass_groups.take() {
                                        Some(curr) => {
                                            if curr.iter().filter(|group| group.get_player_flag(player_i)).count()
                                            < found_group.iter().filter(|group| group.get_player_flag(player_i)).count() {
                                                found_group
                                            } else {
                                                curr
                                            }
                                        },
                                        None => found_group,
                                    }
                                );
                            }
                        } else {
                            if backward_pass_group.get_player_flag(player_i) {
                                let mut recurse_groups = backward_pass_groups.clone();
                                recurse_groups[index_group].set_player_flag(6, true);
                                recurse_groups[index_group].set_player_flag(player_i, false);
                                log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                                log::trace!("backward_pass_groups is : {:?}", &recurse_groups);
                                if let Some(found_group) = self.backward_max_player_cards_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups) {
                                    max_backward_pass_groups = Some(
                                        match max_backward_pass_groups.take() {
                                            Some(curr) => {
                                                if curr.iter().filter(|group| group.get_player_flag(player_i)).count()
                                                < found_group.iter().filter(|group| group.get_player_flag(player_i)).count() {
                                                    found_group
                                                } else {
                                                    curr
                                                }
                                            },
                                            None => found_group,
                                        }
                                    );
                                }

                            }
                            continue 'outer;
                        }
                    } else if redraw_i.is_none() {
                        // Case when player redrew same card back
                        max_backward_pass_groups = self.backward_max_player_cards_recurse(index_loop - 1, index_of_interest, card_of_interest, backward_pass_groups.clone());
                        log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                        log::trace!("max_backward_pass_groups considering case where player redrew the same card back");
                        log::trace!("backward_pass_groups is : {:?}", max_backward_pass_groups);
                        // Case where player relinquished and redrew card of interest
                        // So a card may have gone from pile to player, and we should reverse that 
                        if backward_pass_groups[index_group].get_player_flag(player_i) {
                            // Lmao u really only need to check the current group omg
                            let mut recurse_groups = backward_pass_groups.clone();
                            recurse_groups[index_group].set_player_flag(6, true);
                            recurse_groups[index_group].set_player_flag(player_i, false);
                            log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                            log::trace!("max_backward_pass_groups considering case where player redrew the card of interest");
                            log::trace!("backward_pass_groups is : {:?}", &recurse_groups);
                            if let Some(found_group) = self.backward_max_player_cards_recurse(index_loop - 1, index_of_interest, card_of_interest, recurse_groups) {
                                max_backward_pass_groups = Some(
                                    match max_backward_pass_groups.take() {
                                        Some(curr) => {
                                            if curr.iter().filter(|group| group.get_player_flag(player_i)).count()
                                            < found_group.iter().filter(|group| group.get_player_flag(player_i)).count() {
                                                found_group
                                            } else {
                                                curr
                                            }
                                        },
                                        None => found_group,
                                    }
                                );
                            }
                        }
                    } else {
                        
                        // redraw.is_none() && *reveal_i != card_of_interest
                        // if relinquish.is_none() {
                        //     if backward_pass_group.get_player_flag(player_i) {
                        //         backward_pass_group.set_player_flag(player_i, false);
                        //         // backward_pass_group.sub_alive_count(1);
                        //         backward_pass_group.sub_total_count(1);
                        //     }
                        // }
                        // handleWhat if relinquish is known?

                        // redraw is_some() -> no need for change
                        // redraw is_none9) -> handled in illegal_players
                        max_backward_pass_groups = self.backward_max_player_cards_recurse(index_loop - 1, index_of_interest, card_of_interest, backward_pass_groups.clone());
                        log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                        log::trace!("backward_pass_groups is : {:?}", max_backward_pass_groups);
                    }
                }
                log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                log::trace!("max_backward_pass_groups considering player {}, action {:?}", self.history[index_loop].player(), self.history[index_loop].action_info());
                log::trace!("max_backward_pass_groups returned node : {:?}", max_backward_pass_groups);
                return max_backward_pass_groups
            },
            ActionInfo::ExchangeDrawChoice { draw, relinquish } => {
                // TODO:
            },
            ActionInfo::Start
            | ActionInfo::StartInferred => {},
        }
        // some kind of min function
        self.backward_max_player_cards_recurse(index_loop - 1, index_of_interest, card_of_interest, backward_pass_groups)
    }
    /// Does Backtracking to determine if at a particular point that particular player could not have had some set of cards
    /// Assuming we won't be using this for ambassador?
    pub fn impossible_to_have_cards(&self, index: usize, cards: &Vec<Card>) -> bool {
        debug_assert!(self.history[index].player != 6 && cards.len() == 2, "cards too long! should have size of at most 2");
        debug_assert!(self.history[index].player == 6 && cards.len() == 3, "cards too long! should have size of at most 3 for pile");
        let mut public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(3), Vec::with_capacity(3), Vec::with_capacity(3), Vec::with_capacity(3), Vec::with_capacity(3), Vec::with_capacity(3), Vec::with_capacity(4), ];
        let mut inferred_constraints: Vec<Vec<Card>> = public_constraints.clone();
        let mut inferred_counts: [u8; 5] = [0; 5];
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
            ActionInfo::Start
            | ActionInfo::StartInferred => {},
        }

        // Do an unwrap_or false
        !self.possible_to_have_cards_recurse(self.history.len() - 2, index, &mut public_constraints, &mut inferred_constraints, cards).unwrap_or(false)
    }
    /// returns false if possible
    /// TODO: Consider passing in array to count inferred_so_far
    /// Traces the game tree in reverse (from latest move to earliest move) by backtracking
    /// Tracks possible paths known cards could have come from in the past
    /// If a state is found to satisfy cards at the index_of_interest return Some(true)
    /// If no state is every found return Some(false) or None
    pub fn possible_to_have_cards_recurse(&self, index_loop: usize, index_of_interest: usize, public_constraints: &mut Vec<Vec<Card>>, inferred_constraints: &mut Vec<Vec<Card>>, cards: &Vec<Card>) -> Option<bool> {
        // Will temporarily not use memo and generate all group_constraints from start
        // TODO: OPTIMIZE store the group_constraints in history
        //      - or store all impossible_combinations in history to make checking invalid states faster and less memory usage too
        return None; // TODO: REMOVE
        // if self.is_valid_combination(index_loop, index_of_interest, inferred_counts, inferred_constraints, cards).is_none() {
        //     // early exit before terminal node
        //     return None
        // }
        let player_loop = self.history[index_loop].player() as usize;
        match self.history[index_loop].action_info() {
            ActionInfo::Discard { discard } => {
                if let Some(pos) = inferred_constraints[player_loop].iter().position(|c| *c == *discard) {
                    inferred_constraints.swap_remove(pos);
                }
                public_constraints[player_loop].push(*discard);
                // recurse
                return self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, public_constraints, inferred_constraints, cards);
            },
            ActionInfo::RevealRedraw { reveal, redraw, relinquish } => {
                // Check if will burst before pushing
                match redraw {
                    Some(redraw_i) => {
                        if let Some(pos) = inferred_constraints[player_loop].iter().position(|c| *c == *redraw_i) {
                            inferred_constraints[player_loop].swap_remove(pos);
                        } 
                        inferred_constraints[6].push(*redraw_i);
                        if let Some(pos) = inferred_constraints[6].iter().position(|c| *c == *reveal) {
                            inferred_constraints[6].swap_remove(pos);
                        } 
                        inferred_constraints[player_loop].push(*reveal);
                    },
                    None => {
                        match relinquish {
                            Some(relinquish_i) => {
                                // swap cards around sir
                                // relinquish_i == *reveal always
                                // Case 0: player redrew card != reveal
                                // Case 1: player redrew card == reveal (reveal from pile)
                                
                            },
                            None => {
                                // Case 0: player redrew card != reveal
                                // Case 1: player redrew card == reveal (same as original)
                                // Case 2: player redrew card == reveal (reveal from pile)
                                // Gets a Vec of index, Card for both player and pile
                                let mut source_cards: Vec<(usize, Card)> = inferred_constraints[player_loop]
                                    .iter()
                                    .copied()
                                    .map(|c| (player_loop, c))
                                    .chain(
                                        inferred_constraints[6]
                                            .iter()
                                            .copied()
                                            .map(|c| (6, c)),
                                    )
                                    .collect();
                                let mut variants: Vec<Vec<Vec<Card>>> = Vec::with_capacity(12);
                                Self::build_variants_reveal_redraw_none(*reveal, &source_cards, 0, player_loop, 6, 0, 0, &inferred_constraints, &mut variants);
                                for inferred_i in variants.iter() {
                                    let response = self.possible_to_have_cards_recurse(index_loop - 1, index_of_interest, public_constraints, &mut inferred_i, cards);
                                    if let Some(inner) = response {
                                        if inner {
                                            return Some(true);
                                        }
                                    }
                                }
                            },
                        }
                    },
                }
            },
            ActionInfo::ExchangeDrawChoice { draw, relinquish } => {},
            ActionInfo::Start
            | ActionInfo::StartInferred => {},
        }
        // 
        // Did not find a valid combination
        Some(false)
    }
    pub fn return_variants_reveal_redraw_none(reveal: Card, player_loop: usize, inferred_constraints: &Vec<Vec<Card>>) -> Vec<Vec<Vec<Card>>> {
        let mut variants: Vec<Vec<Vec<Card>>> = Vec::with_capacity(12);
        if inferred_constraints[player_loop].len() + inferred_constraints[6].len() == 5 
        && !inferred_constraints[player_loop].contains(&reveal)
        && !inferred_constraints[6].contains(&reveal) {
            // This state cannot be arrive after the reveal_redraw
            return Vec::with_capacity(0);
        }
        // TODO: OPTIMIZE Probably don't need this source thing
        // This needs to have player indexes before pile indexes
        // we check what player redrew first, which moves item from player to pile
        // then we check what could have moved from pile to player
        let source_cards: Vec<(usize, Card)> = inferred_constraints[player_loop]
            .iter()
            .copied()
            .map(|c| (player_loop, c))
            .chain(
                inferred_constraints[6]
                    .iter()
                    .copied()
                    .map(|c| (6, c)),
            )
            .collect();
        // TODO: Consider moving this out of this function
        // Self::build_variants_reveal_redraw_none(reveal, &source_cards, 0, player_loop, 6, 0, 0, &inferred_constraints, &mut variants);
        Self::build_variants_reveal_redraw_none_opt(reveal, &source_cards, 0, player_loop, 6, 0, 0, &inferred_constraints, &mut variants);
        return variants
    }
    // TODO: modify inferred_constraints and recurse when no longer testing this
    pub fn return_variants_reveal_redraw_none_opt(reveal: Card, player_loop: usize, inferred_constraints: &Vec<Vec<Card>>) -> Vec<Vec<Vec<Card>>> {
        let mut variants: Vec<Vec<Vec<Card>>> = Vec::with_capacity(12);
        if inferred_constraints[player_loop].len() + inferred_constraints[6].len() == 5 
        && !inferred_constraints[player_loop].contains(&reveal)
        && !inferred_constraints[6].contains(&reveal) {
            // This state cannot be arrive after the reveal_redraw
            return Vec::with_capacity(0);
        }
        let mut player_hand = inferred_constraints[player_loop].clone();
        let mut pile_hand = inferred_constraints[6].clone();
        if player_hand.is_empty() {
            // let mut bool_move_from_pile_to_player = false;
            if let Some(pos) = pile_hand.iter().rposition(|c| *c == reveal) {
                pile_hand.swap_remove(pos);
            }
            player_hand.push(reveal);
            let mut temp = inferred_constraints.clone();
            temp[player_loop] = player_hand.clone();
            temp[6] = pile_hand.clone();
            if temp[player_loop].len() < 3
            && temp.iter().map(|v| v.iter().filter(|c| **c == reveal).count() as u8).sum::<u8>() < 4{
                // TODO: Recurse here in other version
                variants.push(temp);
            }
            return variants;
            // TODO: remove if recursing
        }
        // Doesnt handle empty case
        for (_, card_player) in inferred_constraints[player_loop].iter().enumerate() {
            // Card Source was not from Pile
            let mut bool_move_from_pile_to_player = false;
            if let Some(pos) = pile_hand.iter().rposition(|c| *c == reveal) {
                pile_hand.swap_remove(pos);
                bool_move_from_pile_to_player = true;
            }
            player_hand.push(reveal);
            let mut temp = inferred_constraints.clone();
            temp[player_loop] = player_hand.clone();
            temp[6] = pile_hand.clone();

            if let Some(pos) = player_hand.iter().rposition(|c| *c == reveal) {
                player_hand.swap_remove(pos);
            }
            if bool_move_from_pile_to_player {
                pile_hand.push(reveal);
            }
            // Probably need push only if certain conditions met
            if temp[player_loop].len() < 3  
            && temp[6].len() < 4 
            && temp.iter().map(|v| v.iter().filter(|c| **c == reveal).count() as u8).sum::<u8>() < 4{
                // TODO: Recurse here in other version
                variants.push(temp);
            }
            // TEMP
            // player_hand.sort();
            // pile_hand.sort();
            // let mut checker = inferred_constraints.clone();
            // checker[player_loop].sort();
            // checker[6].sort();
            // if player_hand != checker[player_loop] {
            //     log::warn!("failed 1 to pop player hand properly");
            //     log::warn!("player_hand now: {:?}", player_hand);
            // }
            // if pile_hand != checker[6] {
            //     log::warn!("failed 1 to pop pile hand properly");
            //     log::warn!("pile_hand now: {:?}", pile_hand);
            // }
            // Card Source was from Pile
            player_hand = inferred_constraints[player_loop].clone();
            pile_hand = inferred_constraints[6].clone();
            bool_move_from_pile_to_player = false;
            if let Some(pos) = player_hand.iter().position(|c| *c == *card_player) {
                player_hand.swap_remove(pos);
            }
            log::trace!("player_hand after removal of card_player");
            pile_hand.push(*card_player);
            if let Some(pos) = pile_hand.iter().rposition(|c| *c == reveal) {
                pile_hand.swap_remove(pos);
                bool_move_from_pile_to_player = true;
            }
            player_hand.push(reveal);
            let mut temp = inferred_constraints.clone();
            temp[player_loop] = player_hand.clone();
            temp[6] = pile_hand.clone();

            // Probably need push only if certain conditions met
            if temp[player_loop].len() < 3  
            && temp[6].len() < 4 
            && temp.iter().map(|v| v.iter().filter(|c| **c == reveal).count() as u8).sum::<u8>() < 4{
                // TODO: Recurse here in other version
                variants.push(temp);
            }

            if let Some(pos) = player_hand.iter().rposition(|c| *c == reveal) {
                player_hand.swap_remove(pos);
            }
            if bool_move_from_pile_to_player {
                pile_hand.push(reveal);
            }
            if let Some(pos) = pile_hand.iter().rposition(|c| c == card_player) {
                pile_hand.swap_remove(pos);
            }
            player_hand.push(*card_player);
            // TEMP
            // player_hand.sort();
            // pile_hand.sort();
            // let mut checker = inferred_constraints.clone();
            // checker[player_loop].sort();
            // checker[6].sort();
            // if player_hand != checker[player_loop] {
            //     log::warn!("failed 2 to pop player hand properly");
            // }
            // if pile_hand != checker[6] {
            //     log::warn!("failed 2 to pop pile hand properly");
            // }
        }
        return variants
    }
    /// Builds possible previous inferred_constraint states
    /// All cards have a source
    /// card == reveal in player hand can come from
    /// - player without moving
    /// - player after revealing then redrawing the same
    /// - pile after revealing then redrawing different
    /// - player after revealing then redrawing different
    /// 
    /// Pretty overkill way to solve this, but I think will be useful reference for AMB
    fn build_variants_reveal_redraw_none(
        reveal: Card,
        cards: &[(usize, Card)],
        idx: usize,
        player_loop: usize,
        pile_index: usize,
        player_to_pile_count: u8, // dst -> src
        pile_to_player_count: u8, // dst -> src 
        current: &Vec<Vec<Card>>,
        variants: &mut Vec<Vec<Vec<Card>>>,
    ) {
        // build an intermediate state
        // src: [[Ambassador], [], [], [], [], [], [Ambassador, Assassin, Ambassador]], 
        // dest: [[Ambassador], [], [], [], [], [], [Ambassador, Assassin]]
        // This is ok, but Im thinking u technically don't need this cos it will be searched by
        // TODO: OPTIMIZE and remove subsets
        // [[Ambassador], [], [], [], [], [], [Ambassador, Assassin]]
        // src: []
        // dest: [[Ambassador, Ambassador], [], [], [], [], [], [Assassin, Captain, Captain]]
        // P0 RR AMB None
        // src: [[Ambassador], [], [], [], [], [], [Assassin, Assassin, Assassin]]
        // dest: [[Ambassador, Assassin], [], [], [], [], [], [Assassin, Assassin]]
        // src should have 2 Ambassador as 1 stayed and one was revealed
        // if you redrew a card, and the pile is 3 cards after that means the other card had to have been relinquished...
        // P0 RR AMB None
        // Would be impossible here as there is no room for AMB in player or pile in dest,
        // dest: [[Captain, Duke], [], [], [], [], [], [Captain, Captain, Contessa]]
        // src: [[[Ambassador, Ambassador], [], [], [], [], [], [Captain, Captain, Ambassador]], [[Ambassador, Ambassador], [], [], [], [], [], [Captain, Captain, Ambassador]]]
        // dest: [[Ambassador, Ambassador], [], [], [], [], [], [Captain, Captain]]
        if idx == cards.len() {
            // TODO: OPTIMIZE the push
            let mut source_constraints = current.clone();
            // Player redrew same card
            // Not exactly right as player_to_pile could be a non ambassador
            // Add if player_to_pile is 0
            if pile_to_player_count != 1 || !source_constraints[player_loop].contains(&reveal) {
                source_constraints[player_loop].push(reveal);
            }
            if source_constraints[player_loop].len() < 3  
            && source_constraints[pile_index].len() < 4 
            && source_constraints.iter().map(|v| v.iter().filter(|c| **c == reveal).count() as u8).sum::<u8>() < 4{
                variants.push(source_constraints);
            }
            return;
        }

        // Destructure this card's source and value
        let (dst, card) = cards[idx];
        // OPTIMIZE: probably could just run this as the ONLY case for reveal redraw as the other case will be a more specific case of this
        // CASE: Card originally left player hand and returned to player and is the same unique card 
        // T                :  [C0]    []
        // T after reveal   :  []    [C0]
        // T + 1            :  [C0]    []
        if dst == player_loop && card == reveal {
            // No need to add reveal anymore
            Self::build_variants_reveal_redraw_none(reveal, cards, cards.len(), player_loop, pile_index, 1, 1, current, variants);
        } 
        // src same as dest and RR card is diff

        // CASE: player Card did not come from pile after reveal
        // CASE: pile Card did not come from player after reveal
        // ADDITIONALLY: if player reveal and redrew the same card, it is considered seperate cards
        Self::build_variants_reveal_redraw_none(reveal, cards, idx + 1, player_loop, pile_index, player_to_pile_count, pile_to_player_count, current, variants);
        let is_player_card = dst == player_loop;
        let could_have_swapped = if is_player_card {
            player_to_pile_count < 1 
        } else {
            pile_to_player_count < 1 && card == reveal
        };
        // CASE: player Card came from pile after reveal
        // CASE: pile Card came from player after reveal
        // ADDITIONALLY: if player reveal and redrew the same card, it is considered seperate cards
        if could_have_swapped {
            let (src, new_player_to_pile_count, new_pile_to_player_count) = if is_player_card {
                (pile_index, player_to_pile_count + 1, pile_to_player_count)
            } else {
                (player_loop, player_to_pile_count, pile_to_player_count + 1)
            };
            
            let mut new_constraints = current.clone();
            if let Some(pos) = new_constraints[dst].iter().position(|&c| c == card) {
                new_constraints[dst].swap_remove(pos);
                new_constraints[src].push(card);
                Self::build_variants_reveal_redraw_none(reveal, cards, idx + 1, player_loop, pile_index, new_player_to_pile_count, new_pile_to_player_count, &new_constraints, variants);
            }
        }
    }
    /// Builds possible previous inferred_constraint states
    /// All cards have a source
    /// card == reveal in player hand can come from
    /// - player without moving
    /// - player after revealing then redrawing the same
    /// - pile after revealing then redrawing different
    /// - player after revealing then redrawing different
    /// 
    /// Not trying that many cases here as some of them just lead to more specific sets that will already be covered
    /// e.g. 
    /// src: [[Ambassador], [], [], [], [], [], [Assassin, Ambassador]], [Ambassador, Ambassador], [], [], [], [], [], [Ambassador, Assassin]]
    /// dest: [[Ambassador], [], [], [], [], [], [Ambassador, Assassin]]
    /// the second is a more specific case of the first item in src
    fn build_variants_reveal_redraw_none_opt(
        reveal: Card,
        cards: &[(usize, Card)],
        idx: usize,
        player_loop: usize,
        pile_index: usize,
        player_to_pile_count: u8, // dst -> src
        pile_to_player_count: u8, // dst -> src 
        current: &Vec<Vec<Card>>,
        variants: &mut Vec<Vec<Vec<Card>>>,
    ) {
        // build an intermediate state
        // src: [[Ambassador], [], [], [], [], [], [Ambassador, Assassin, Ambassador]], 
        // dest: [[Ambassador], [], [], [], [], [], [Ambassador, Assassin]]
        // This is ok, but Im thinking u technically don't need this cos it will be searched by
        // TODO: OPTIMIZE and remove subsets
        // [[Ambassador], [], [], [], [], [], [Ambassador, Assassin]]
        // src: []
        // dest: [[Ambassador, Ambassador], [], [], [], [], [], [Assassin, Captain, Captain]]
        // P0 RR AMB None
        // src: [[Ambassador], [], [], [], [], [], [Assassin, Assassin, Assassin]]
        // dest: [[Ambassador, Assassin], [], [], [], [], [], [Assassin, Assassin]]
        // src should have 2 Ambassador as 1 stayed and one was revealed
        // if you redrew a card, and the pile is 3 cards after that means the other card had to have been relinquished...
        // P0 RR AMB None
        // Would be impossible here as there is no room for AMB in player or pile in dest,
        // dest: [[Captain, Duke], [], [], [], [], [], [Captain, Captain, Contessa]]
        // src: [[[Ambassador, Ambassador], [], [], [], [], [], [Captain, Captain, Ambassador]], [[Ambassador, Ambassador], [], [], [], [], [], [Captain, Captain, Ambassador]]]
        // dest: [[Ambassador, Ambassador], [], [], [], [], [], [Captain, Captain]]
        if idx == cards.len() {
            // TODO: OPTIMIZE the push
            let mut source_constraints = current.clone();
            // Player redrew same card
            // Not exactly right as player_to_pile could be a non ambassador
            // Add if player_to_pile is 0
            if !source_constraints[player_loop].contains(&reveal)
            { // TESTING OPTIMIZE
                source_constraints[player_loop].push(reveal);
            }
            if source_constraints[player_loop].len() < 3  
            && source_constraints[pile_index].len() < 4 
            && source_constraints.iter().map(|v| v.iter().filter(|c| **c == reveal).count() as u8).sum::<u8>() < 4{
                variants.push(source_constraints);
            }
            return;
        }

        // CASE: player Card did not come from pile after reveal
        // CASE: pile Card did not come from player after reveal
        // ADDITIONALLY: if player reveal and redrew the same card, it is considered seperate cards
        Self::build_variants_reveal_redraw_none_opt(reveal, cards, idx + 1, player_loop, pile_index, player_to_pile_count, pile_to_player_count, current, variants);
        // Destructure this card's source and value
        let (dst, card) = cards[idx];
        // OPTIMIZE: probably could just run this as the ONLY case for reveal redraw as the other case will be a more specific case of this
        // CASE: Card originally left player hand and returned to player and is the same unique card 
        // T                :  [C0]    []
        // T after reveal   :  []    [C0]
        // T + 1            :  [C0]    []
        if dst == player_loop && card == reveal {
            // No need to add reveal anymore
            Self::build_variants_reveal_redraw_none_opt(reveal, cards, cards.len(), player_loop, pile_index, 1, 1, current, variants);
            return
        } 

        let is_player_card = dst == player_loop;
        let could_have_swapped = if is_player_card {
            player_to_pile_count < 1 
            // Testing OPTIMIZE: exclude all other reveal redraw same cards
            // && card != reveal
        } else {
            pile_to_player_count < 1 && card == reveal
        };
        // CASE: player Card came from pile after reveal
        // CASE: pile Card came from player after reveal
        // ADDITIONALLY: if player reveal and redrew the same card, it is considered seperate cards
        if could_have_swapped {
            let (src, new_player_to_pile_count, new_pile_to_player_count) = if is_player_card {
                (pile_index, player_to_pile_count + 1, pile_to_player_count)
            } else {
                (player_loop, player_to_pile_count, pile_to_player_count + 1)
            };
            
            let mut new_constraints = current.clone();
            if let Some(pos) = new_constraints[dst].iter().position(|&c| c == card) {
                new_constraints[dst].swap_remove(pos);
                new_constraints[src].push(card);
                Self::build_variants_reveal_redraw_none_opt(reveal, cards, idx + 1, player_loop, pile_index, new_player_to_pile_count, new_pile_to_player_count, &new_constraints, variants);
            }
        }
    }
    // TODO: not use this function
    // literally its fully known what other combination could there be bro
    pub fn modify_variants_reveal_redraw(reveal: Card, redraw: Card, player_loop: usize, inferred_constraints: &mut Vec<Vec<Card>>) {
        if let Some(pos) = inferred_constraints[player_loop].iter().position(|c| *c == redraw) {
            inferred_constraints[player_loop].swap_remove(pos);
        } 
        inferred_constraints[6].push(redraw);
        if let Some(pos) = inferred_constraints[6].iter().position(|c| *c == reveal) {
            inferred_constraints[6].swap_remove(pos);
        } 
        inferred_constraints[player_loop].push(reveal);
        // ish not exact...
    }
    pub fn return_variants_reveal_redraw(reveal: Card, redraw: Card, player_loop: usize, inferred_constraints: &Vec<Vec<Card>>) -> Vec<Vec<Vec<Card>>> {
        let mut variants: Vec<Vec<Vec<Card>>> = Vec::new();
        let mut temp = inferred_constraints.clone();
        if let Some(pos) = temp[player_loop].iter().position(|c| *c == redraw) {
            temp[player_loop].swap_remove(pos);
        } 
        temp[6].push(redraw);
        if let Some(pos) = temp[6].iter().position(|c| *c == reveal) {
            temp[6].swap_remove(pos);
        } 
        temp[player_loop].push(reveal);
        if temp[6].len() < 4 {
            variants.push(temp);
        }
        return variants
    }
    pub fn return_variants_reveal_relinquish(reveal: Card, redraw: Card, player_loop: usize, inferred_constraints: &Vec<Vec<Card>>) -> Vec<Vec<Vec<Card>>> {
        if inferred_constraints[6].len() == 3
        && !inferred_constraints[6].contains(&reveal) {
            // This state cannot be arrive after the reveal_redraw
            return Vec::with_capacity(0);
        }
        let source_cards: Vec<(usize, Card)> = inferred_constraints[player_loop]
            .iter()
            .copied()
            .map(|c| (player_loop, c))
            .chain(
                inferred_constraints[6]
                    .iter()
                    .copied()
                    .map(|c| (6, c)),
            )
            .collect();
        let mut variants: Vec<Vec<Vec<Card>>> = Vec::with_capacity(12);
        Self::build_variants_reveal_relinquish(reveal, redraw, &source_cards, 0, player_loop, 6, 0, 0, &inferred_constraints, &mut variants);
        return variants
    }
    pub fn return_variants_reveal_relinquish_opt(reveal: Card, player_loop: usize, inferred_constraints: &Vec<Vec<Card>>) -> Vec<Vec<Vec<Card>>> {
        let mut variants: Vec<Vec<Vec<Card>>> = Vec::with_capacity(12);
        if inferred_constraints[6].len() == 3
        && !inferred_constraints[6].contains(&reveal) {
            // This state cannot be arrive after the reveal_redraw
            return Vec::with_capacity(0);
        }
        let mut player_hand = inferred_constraints[player_loop].clone();
        let mut pile_hand = inferred_constraints[6].clone();
        if player_hand.is_empty() {
            // let mut bool_move_from_pile_to_player = false;
            if let Some(pos) = pile_hand.iter().rposition(|c| *c == reveal) {
                pile_hand.swap_remove(pos);
            }
            player_hand.push(reveal);
            let mut temp = inferred_constraints.clone();
            temp[player_loop] = player_hand.clone();
            temp[6] = pile_hand.clone();
            if temp[player_loop].len() < 3
            && temp.iter().map(|v| v.iter().filter(|c| **c == reveal).count() as u8).sum::<u8>() < 4{
                // TODO: Recurse here in other version
                variants.push(temp);
            }
            return variants;
            // TODO: remove if recursing
        }
        for (i, card_player) in inferred_constraints[player_loop].iter().enumerate() {
            // Card Source was from Pile
            let mut bool_move_from_pile_to_player = false;
            player_hand.swap_remove(i);
            pile_hand.push(*card_player);
            if let Some(pos) = pile_hand.iter().rposition(|c| *c == reveal) {
                pile_hand.swap_remove(pos);
                bool_move_from_pile_to_player = true;
            }
            player_hand.push(reveal);
            let mut temp = inferred_constraints.clone();
            temp[player_loop] = player_hand.clone();
            temp[6] = pile_hand.clone();

            // TODO: Recurse here in other version
            variants.push(temp);

            if let Some(pos) = player_hand.iter().rposition(|c| *c == reveal) {
                player_hand.swap_remove(pos);
            }
            if bool_move_from_pile_to_player {
                pile_hand.push(reveal);
            }
            if let Some(pos) = pile_hand.iter().rposition(|c| c == card_player) {
                pile_hand.swap_remove(pos);
            }
            player_hand.push(*card_player);
        }
        variants
    }
    /// Builds possible previous inferred_constraint states
    fn build_variants_reveal_relinquish(
        reveal: Card,
        redraw: Card,
        cards: &[(usize, Card)],
        idx: usize,
        player_loop: usize,
        pile_index: usize,
        player_to_pile_count: u8, // dst -> src
        pile_to_player_count: u8, // dst -> src 
        current: &Vec<Vec<Card>>,
        variants: &mut Vec<Vec<Vec<Card>>>,
    ) {
        // Ok im thinking we gotta do the relinquish part for sure
        // Then for cards in player hand, just see if source is hand or pile
        // there simple

        // P0 RR AMB CAP
        // Would be impossible here as there is no room for CAP in player in dest,
        // src: [[[Ambassador, Ambassador], [], [], [], [], [], [Captain, Captain]]]
        // dest: [[Ambassador, Ambassador], [], [], [], [], [], [Captain, Captain]]
        if idx == cards.len() {
            // TODO: OPTIMIZE the push
            let mut source_constraints = current.clone();
            if !source_constraints[player_loop].contains(&reveal) {
                source_constraints[player_loop].push(reveal);
            }
            if source_constraints[player_loop].len() < 3 
            && source_constraints[pile_index].len() < 4 
            && source_constraints.iter().map(|v| v.iter().filter(|c| **c == reveal).count() as u8).sum::<u8>() < 4
            {
                variants.push(source_constraints);
            }
            return;
        }
    
        // Destructure this card's source and value
        let (dst, card) = cards[idx];
    
        // OPTION A: leave it in the same container
        Self::build_variants_reveal_relinquish(reveal, redraw, cards, idx + 1, player_loop, pile_index, player_to_pile_count, pile_to_player_count, current, variants);
        let is_player_card = dst == player_loop;
        let could_have_swapped = if is_player_card {
            player_to_pile_count < 1 
        } else {
            pile_to_player_count < 1 
            && card == reveal 
        };
        // OPTION B: move it to the other container
        if could_have_swapped {
            let (src, new_player_to_pile_count, new_pile_to_player_count) = if is_player_card {
                (pile_index, player_to_pile_count + 1, pile_to_player_count)
            } else {
                (player_loop, player_to_pile_count, pile_to_player_count + 1)
            };
            
            let mut new_constraints = current.clone();
            if let Some(pos) = new_constraints[dst].iter().position(|&c| c == card) {
                new_constraints[dst].swap_remove(pos);
                new_constraints[src].push(card);
                Self::build_variants_reveal_relinquish(reveal, redraw, cards, idx + 1, player_loop, pile_index, new_player_to_pile_count, new_pile_to_player_count, &new_constraints, variants);
            }
        }
    }
    /// Return true if hypothesised card permutations cannot be shown to be impossible
    pub fn is_valid_combination(&self, index_loop: usize ,index_of_interest: usize, public_constraints: &Vec<Vec<Card>>, inferred_constraints: &Vec<Vec<Card>>, cards: &Vec<Card>) -> Option<bool> {
        let invalid_state: bool = false;
        if index_loop == index_of_interest - 1 {
            // Check actual constraints at leaf node
            // All public_constraints inside actual
            // All 
        } else {
            if invalid_state {
                return None;
            }
        }
        Some(true)
    }
    /// determines if a previous player 
    pub fn need_redraw_update(&self, index: usize, card_of_interest: Card, illegal_players: &[bool; 6]) -> bool {
        let action_player = self.history[index].player() as usize;
        log::trace!("need_redraw_update for player {:?}, move: {:?}", self.history[self.history.len() - 1].player(), self.history[self.history.len() - 1].action_info());
        log::trace!("!illegal_players[action_player as usize]: {:?}", !illegal_players[action_player as usize]);
        log::trace!("redraw_i.is_none(): {:?}", self.history[index].action_info());
        
        // Actually combine all into here after that!
        // Looksback at a particular index that is a RevealRedraw(reveal=Card, redraw=None)
        // If reveal == Card of interest (latest reveal_considered of discard_considered)
        //     Determines if just before the revealredraw, if the player had to have redrawn the same card
        //     Because the player cannot have 2 of that card (as the rest are outside the player)
        //     And the player must therefore redraw it, to be able to revealdiscard later on
        // Assumes latest move ran is Discard or RevealRedraw
        // TODO: [OPTIMIZE] can collect bwd pass states during outer loop
        log::trace!("index: {index} action_player: {action_player}, card_of_interest: {:?}", card_of_interest);
        log::trace!("last move in history: player: {:?}, {:?}", self.history[self.history.len() - 1], self.history[self.history.len() - 1].action_info());
        // Add initial to backward_pass_group 
        // backward_pass_group only counts revealed/discarded cards in count_alive()
        // NOTE: forward_pass_group and backward_pass_group are used differently!
        //      they only use GroupConstraint as a convenient datastructure
        // Count of backward_pass_group is meant to count unique cards
        // flags are roughly meant to track where the cards can be
        // backward_pass_group to count unique cards
        let mut backward_pass_group = CompressedGroupConstraint::zero();
        backward_pass_group.set_player_flag(self.history[self.history.len() - 1].player() as usize, true);
        // backward_pass_group.add_alive_count(1);
        backward_pass_group.add_total_count(1);
        let mut forward_pass_group = CompressedGroupConstraint::zero();
        let mut players_had_revealed_or_discarded = [false; 6];
        // Prob can optimize to use u8s
        let mut backward_pass_groups: Vec<CompressedGroupConstraint> = Vec::with_capacity(3);
        backward_pass_groups.push(backward_pass_group);
        players_had_revealed_or_discarded[self.history[self.history.len() - 1].player() as usize] = true;
        // backward pass
        // TODO: fix total count exceeding error
        log::trace!("backwardpass start: {:?}", backward_pass_group);
        log::trace!("fwd_bwd_pass last move: {:?}", self.history[self.history.len() - 1]);
        'outer: for i in (index+1..self.history.len()-1).rev() {
            let player_i = self.history[i].player() as usize;
            log::trace!("backward_pass checking i: {i} player_i: {player_i}, move: {:?}", self.history[i].action_info());
            match self.history[i].action_info() {
                ActionInfo::Discard { discard: discard_i } => {
                    if *discard_i == card_of_interest {
                        backward_pass_group.set_player_flag(player_i, true);
                        // backward_pass_group.add_alive_count(1);
                        backward_pass_group.add_total_count(1);
                    }
                    if *discard_i == card_of_interest {
                        players_had_revealed_or_discarded[player_i] = true;
                        let mut new_groups = CompressedGroupConstraint::zero();
                        new_groups.set_player_flag(player_i, true);
                        new_groups.set_total_count(1);
                        backward_pass_groups.push(new_groups);
                    }
                },
                ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, relinquish } => {
                    // Maybe it exceeds maximum because I need to consider the pile for revealredraw redraw is known
                    if *reveal_i == card_of_interest {
                        // Reflecting that the player had this card before move i
                        if *relinquish == Some(card_of_interest) {
                            players_had_revealed_or_discarded[player_i] = true;
                            // No check as it went from player to pile
                            // Not setting pile as we don't actually need to use it
                            // backward_pass_group.set_player_flag(6, true);
                            // backward_pass_group.add_alive_count(1);
                            // backward_pass_group.add_total_count(1);
                            // TESTING, was thinking i might need to consider how the pile moves too?
                            // Also consider needing to subtract?
                            if !backward_pass_group.get_player_flag(6) {
                                backward_pass_group.set_player_flag(6, true);
                                // backward_pass_group.add_alive_count(1);
                                backward_pass_group.add_total_count(1);
                            }
                            for group in backward_pass_groups.iter_mut() {
                                if group.get_player_flag(6) {
                                    // This is greedy, may consider backtracking for min
                                    group.set_player_flag(6, false);
                                    group.set_player_flag(player_i, true);
                                }
                                continue 'outer;
                            }
                            let mut new_groups = CompressedGroupConstraint::zero();
                            new_groups.set_player_flag(player_i, true);
                            new_groups.set_total_count(1);
                            backward_pass_groups.push(new_groups);
                        } else if relinquish.is_none() {
                            players_had_revealed_or_discarded[player_i] = true;
                            if redraw_i.is_none() {
                                if !backward_pass_group.get_player_flag(player_i) {
                                    backward_pass_group.set_player_flag(player_i, true);
                                    // backward_pass_group.add_alive_count(1);
                                    backward_pass_group.add_total_count(1);
                                } 
                                for group in backward_pass_groups.iter_mut() {
                                    if group.get_player_flag(6) {
                                        // This is greedy, may consider backtracking for min
                                        group.set_player_flag(6, false);
                                        group.set_player_flag(player_i, true);
                                    }
                                    continue 'outer;
                                }
                                let mut new_groups = CompressedGroupConstraint::zero();
                                new_groups.set_player_flag(player_i, true);
                                new_groups.set_total_count(1);
                                backward_pass_groups.push(new_groups);
                            } else if *redraw_i == Some(card_of_interest) {
                                // TODO: Actually order of discard then revealredraw matters!
                                // So i guess I should track in single_card_flags too huh
                                // P2 Discard DUK
                                // P2 RR DUK
                                // => 2 DUKS
                                // P2 RR DUK
                                // P2 Discard DUK
                                // => 1 DUK
                                if !backward_pass_group.get_player_flag(player_i) {
                                    backward_pass_group.set_player_flag(player_i, true);
                                    // backward_pass_group.add_alive_count(1);
                                    backward_pass_group.add_total_count(1);
                                }
                                let mut new_groups = CompressedGroupConstraint::zero();
                                new_groups.set_player_flag(player_i, true);
                                new_groups.set_total_count(1);
                                backward_pass_groups.push(new_groups);
                            } else {
                                // Not setting pile as we don't actually need to use it
                                // backward_pass_group.set_player_flag(6, true);
                                // backward_pass_group.add_alive_count(1);
                                // TESTING
                                // If u reveal the card that means pile had the card before that turn
                                // How does it interact with a RR None group?
                                // if !backward_pass_group.get_player_flag(6) {
                                //     backward_pass_group.set_player_flag(6, true);
                                //     // backward_pass_group.add_alive_count(1);
                                //     backward_pass_group.add_total_count(1);
                                // }
                                // backward_pass_group.add_total_count(1);
                                for group in backward_pass_groups.iter_mut() {
                                    if group.get_player_flag(6) {
                                        // This is greedy, may consider backtracking for min
                                        group.set_player_flag(6, false);
                                        group.set_player_flag(player_i, true);
                                    }
                                    continue 'outer;
                                }
                                let mut new_groups = CompressedGroupConstraint::zero();
                                new_groups.set_player_flag(player_i, true);
                                new_groups.set_total_count(1);
                                backward_pass_groups.push(new_groups);
                            }
                        }
                    } else if *redraw_i == Some(card_of_interest) {
                        // Reflecting that the pile had this card before move i
                        if !backward_pass_group.get_player_flag(player_i) { // This check is rightly player_i
                            // Not setting pile as we don't actually need to use it
                            // backward_pass_group.set_player_flag(6, true);
                            // backward_pass_group.add_alive_count(1);
                            backward_pass_group.add_total_count(1);
                        } else {
                            // backward_pass_group.set_player_flag(6, true);
                            backward_pass_group.set_player_flag(player_i, false);
                        }
                        for group in backward_pass_groups.iter_mut() {
                            if group.get_player_flag(player_i) {
                                // This is greedy, may consider backtracking for min
                                group.set_player_flag(6, true);
                                group.set_player_flag(player_i, false);
                            }
                            continue 'outer;
                        }
                        let mut new_groups = CompressedGroupConstraint::zero();
                        new_groups.set_player_flag(6, true);
                        new_groups.set_total_count(1);
                        backward_pass_groups.push(new_groups);
                    } else {
                        // redraw.is_none() && *reveal_i != card_of_interest
                        // if relinquish.is_none() {
                        //     if backward_pass_group.get_player_flag(player_i) {
                        //         backward_pass_group.set_player_flag(player_i, false);
                        //         // backward_pass_group.sub_alive_count(1);
                        //         backward_pass_group.sub_total_count(1);
                        //     }
                        // }
                        // handleWhat if relinquish is known?

                        // redraw is_some() -> no need for change
                        // redraw is_none9) -> handled in illegal_players
                    }
                    // if *reveal_i == card_of_interest {
                    //     // Reflecting that the player had this card before move i
                    //     if *relinquish == Some(card_of_interest) {
                    //         // No check as it went from player to pile
                    //         // Not setting pile as we don't actually need to use it
                    //         // backward_pass_group.set_player_flag(6, true);
                    //         // backward_pass_group.add_alive_count(1);
                    //         // backward_pass_group.add_total_count(1);
                    //         // TESTING, was thinking i might need to consider how the pile moves too?
                    //         // Also consider needing to subtract?
                    //         backward_pass_group.set_player_flag(player_i, true);
                    //         if !backward_pass_group.get_player_flag(6) {
                    //             backward_pass_group.set_player_flag(6, true);
                    //             // backward_pass_group.add_alive_count(1);
                    //             backward_pass_group.add_total_count(1);
                    //         }
                    //     } else if relinquish.is_none() {
                    //         if redraw_i.is_none() {
                    //             if !backward_pass_group.get_player_flag(player_i) {
                    //                 backward_pass_group.set_player_flag(player_i, true);
                    //                 // backward_pass_group.add_alive_count(1);
                    //                 backward_pass_group.add_total_count(1);
                    //             } 
                    //         } else if *redraw_i == Some(card_of_interest) {
                    //             // TODO: Actually order of discard then revealredraw matters!
                    //             // So i guess I should track in single_card_flags too huh
                    //             // P2 Discard DUK
                    //             // P2 RR DUK
                    //             // => 2 DUKS
                    //             // P2 RR DUK
                    //             // P2 Discard DUK
                    //             // => 1 DUK
                    //             if !backward_pass_group.get_player_flag(player_i) {
                    //                 backward_pass_group.set_player_flag(player_i, true);
                    //                 // backward_pass_group.add_alive_count(1);
                    //                 backward_pass_group.add_total_count(1);
                    //             }
                    //         } else {
                    //             // Not setting pile as we don't actually need to use it
                    //             // backward_pass_group.set_player_flag(6, true);
                    //             // backward_pass_group.add_alive_count(1);
                    //             // TESTING
                    //             // If u reveal the card that means pile had the card before that turn
                    //             // How does it interact with a RR None group?
                    //             // if !backward_pass_group.get_player_flag(6) {
                    //             //     backward_pass_group.set_player_flag(6, true);
                    //             //     // backward_pass_group.add_alive_count(1);
                    //             //     backward_pass_group.add_total_count(1);
                    //             // }
                    //             // backward_pass_group.add_total_count(1);
                    //         }
                    //     }
                    // } else if *redraw_i == Some(card_of_interest) {
                    //     // Reflecting that the pile had this card before move i
                    //     if !backward_pass_group.get_player_flag(player_i) { // This check is rightly player_i
                    //         // Not setting pile as we don't actually need to use it
                    //         // backward_pass_group.set_player_flag(6, true);
                    //         // backward_pass_group.add_alive_count(1);
                    //         backward_pass_group.add_total_count(1);
                    //     } else {
                    //         // backward_pass_group.set_player_flag(6, true);
                    //         backward_pass_group.set_player_flag(player_i, false);
                    //     }
                    // } else {
                    //     // redraw.is_none() && *reveal_i != card_of_interest
                    //     // if relinquish.is_none() {
                    //     //     if backward_pass_group.get_player_flag(player_i) {
                    //     //         backward_pass_group.set_player_flag(player_i, false);
                    //     //         // backward_pass_group.sub_alive_count(1);
                    //     //         backward_pass_group.sub_total_count(1);
                    //     //     }
                    //     // }
                    //     // handleWhat if relinquish is known?
                    // }
                    // OLD
                    // if redraw_i.is_none() && self.history[i].player() == action_player as u8 {
                    // if redraw_i.is_none() {
                    //     // illegal
                    //     // return false
                    //     if backward_pass_group.get_player_flag(player_i) {
                    //         backward_pass_group.set_player_flag(player_i, false);
                    //         backward_pass_group.sub_total_count(1);
                    //     }
                    // }
                    // if *reveal_i == card_of_interest {
                    //     // Reflecting that the player had this card before move i
                    //     if *relinquish == Some(card_of_interest) {
                    //         // No check as it went from player to pile
                    //         // Not setting pile as we don't actually need to use it
                    //         // backward_pass_group.set_player_flag(6, true);
                    //         backward_pass_group.add_total_count(1);
                    //     } else if relinquish.is_none() {
                    //         if !backward_pass_group.get_player_flag(player_i) {
                    //             backward_pass_group.set_player_flag(player_i, true);
                    //             backward_pass_group.add_total_count(1);
                    //         }
                    //     }
                    // } else if *redraw_i == Some(card_of_interest) {
                    //     // Reflecting that the pile had this card before move i
                    //     if !backward_pass_group.get_player_flag(player_i) { // This check is rightly player_i
                    //         // Not setting pile as we don't actually need to use it
                    //         // backward_pass_group.set_player_flag(6, true);
                    //         backward_pass_group.add_total_count(1);
                    //     }
                    // } 
                },
                ActionInfo::ExchangeDrawChoice { draw, relinquish } => {
                    if backward_pass_group.get_player_flag(player_i) {
                        backward_pass_group.set_player_flag(player_i, false);
                        // backward_pass_group.sub_alive_count(1);
                        backward_pass_group.sub_total_count(1);
                    }
                    // TODO: Handle this with groups
                    // if self.history[i].player() == action_player as u8 {
                    //     // illegal
                    //     return false
                    // }
                },
                ActionInfo::Start => {},
                ActionInfo::StartInferred => {},
            }
            // log::trace!("backwardpass i: {:?}", backward_pass_group);
            log::trace!("backwardpassgroups i: {:?}", backward_pass_groups);
        }
        // hijacking to test backtracking
        backward_pass_groups = self.backward_pass_minimum_cards_shown(index, card_of_interest);
        let max_action_player_cards_of_interest = self.backward_max_player_cards(index, card_of_interest);
        log::trace!("max_action_player_cards_of_interest: {:?}", max_action_player_cards_of_interest);
        log::trace!("backward_pass_groups: {:?}", backward_pass_groups);
        // forward pass
        for i in 2..index {
            let player_i = self.history[i].player() as usize;
            match self.history[i].action_info() {
                ActionInfo::Start => {},
                ActionInfo::StartInferred => {},
                ActionInfo::Discard { discard } => {
                    if *discard == card_of_interest {
                        forward_pass_group.add_dead_count(1);
                        if forward_pass_group.get_player_flag(player_i) {
                            forward_pass_group.sub_alive_count(1);
                        }
                    }
                },
                ActionInfo::RevealRedraw { reveal, redraw, relinquish } => {
                    if *reveal == card_of_interest {
                        // relinquish first
                        if relinquish.is_none() {
                            if redraw.is_none() {
                                if !forward_pass_group.get_player_flag(player_i) {
                                    forward_pass_group.set_player_flag(player_i, true);
                                    forward_pass_group.set_player_flag(6, true);
                                    forward_pass_group.add_alive_count(1);
                                }
                            } else if *redraw == Some(card_of_interest) {
                                if !forward_pass_group.get_player_flag(player_i) {
                                    forward_pass_group.set_player_flag(player_i, true);
                                    forward_pass_group.add_alive_count(1);
                                }
                            } else {
                                if !forward_pass_group.get_player_flag(player_i) { // This rightly checks player_i
                                    forward_pass_group.set_player_flag(6, true);
                                    forward_pass_group.add_alive_count(1);
                                }
                            }
                        }
                    } else if redraw.is_none() || redraw.unwrap() == card_of_interest {
                        // Case when different reveal, and can redraw any kind of card
                        // Case when redraw the card_of_interest
                        // Case when different reveal and relinquish == reveal (it always is)
                        if forward_pass_group.get_player_flag(6) {
                            forward_pass_group.set_player_flag(player_i, true);
                        }
                    }
                },
                ActionInfo::ExchangeDrawChoice { draw, relinquish } => {
                    // TODO: Technically, have to consider what draw and relinquish could be
                    if forward_pass_group.get_player_flag(6) {
                        forward_pass_group.set_player_flag(player_i, true);
                    }
                },
            }
        }
        let mut combined = forward_pass_group;
        let mut total_count = combined.count_alive() + combined.count_dead();
        // again this is greedy and you could probably find the biggest group or something
        for group in backward_pass_groups.iter() {
            if combined.part_list_is_mut_excl(*group) {
                total_count += 1;
                log::trace!("Compared combined: {:?} with group_i: {:?} and increased total_count to: {}", combined, group, total_count);
                for i in 0..7 {
                    // TODO: OPTIMIZE
                    // Just make a bitwise OR
                    if !combined.get_player_flag(i) {
                        combined.set_player_flag(i, group.get_player_flag(i));
                    }
                }
            }
        }
        log::trace!("forward_pass_group before check: {:?}", forward_pass_group);
        let bool_2_cards_outside_player = if let ActionInfo::RevealRedraw { reveal, redraw , .. }= self.history[index].action_info() {
            if *reveal == card_of_interest && redraw.is_none() {
                // This is just !illegal_players
                // if !backward_pass_group.get_player_flag(action_player) {
                //     log::trace!("lookback_check_3_fwd_bwd_pass early exit: {}", false);
                //     return false
                // }
                
                log::trace!("forward_pass_group: {:?}", forward_pass_group);
                if !forward_pass_group.get_player_flag(action_player) {
                    if forward_pass_group.count_alive() + forward_pass_group.count_dead() == 2 {
                        log::trace!("lookback_check_3_fwd_bwd_pass A : {}", true);
                        return true
                    }
                    // How is it that order matters for the above and this check?
                    // it cant both be true tho?
                    if max_action_player_cards_of_interest > 1 {
                        // player could have had 2 alive cards
                        log::trace!("lookback_check_3_fwd_bwd_pass B : {}, total_count: {}", false, total_count);
                        return false
                    }
                    // Maybe do the ME union as per before?
                    // if forward_pass_group.count_dead() + backward_pass_group.get_total_count() == 3 {
                    //     log::trace!("lookback_check_3_fwd_bwd_pass B : {}", true);
                    //     return true
                    // }
                    log::trace!("lookback_check_3_fwd_bwd_pass total_count: {}", total_count);
                    // I think might need to do another DP for maximum cards for player
                    if total_count == 3 {
                        log::trace!("lookback_check_3_fwd_bwd_pass B : {}, total_count: {}", true, total_count);
                        return true
                    }
                }
            }
            false
        } else {
            false
        };
        log::trace!("lookback_check_3_fwd_bwd_pass: {}", false);
        if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = self.history[index].action_info() {

            log::trace!("need_redraw_update: redraw_i.is_none() {}", redraw_i.is_none());
            log::trace!("need_redraw_update: illegal_players {:?}", illegal_players);
            log::trace!("need_redraw_update: !illegal_players[action_player] {}", !illegal_players[action_player]);
            log::trace!("need_redraw_update: players_had_revealed_or_discarded {:?}", players_had_revealed_or_discarded);
            log::trace!("need_redraw_update: players_had_revealed_or_discarded[action_player] {}", players_had_revealed_or_discarded[action_player]);
            log::trace!("need_redraw_update: self.history[index - 1].public_constraints()[action_player] {:?}", self.history[index - 1].public_constraints()[action_player]);
            log::trace!("need_redraw_update: self.history[index - 1].public_constraints()[action_player].len() == 1 {}", self.history[index - 1].public_constraints()[action_player].len() == 1);

        }
        
        if let ActionInfo::RevealRedraw { reveal: reveal_i, redraw: redraw_i, .. } = self.history[index].action_info() {
            redraw_i.is_none() 
            && !illegal_players[action_player] // skip if player RR after too (dk which revealredraw the player redrew)
            && players_had_revealed_or_discarded[action_player] // checks if player did reveal/redraw that card before
            && (
                // if you had 1 live, you must have withdrawn the card you Reveal/Discard later on
                self.history[index - 1].impossible_constraints()[action_player][card_of_interest as usize]
                || self.history[index - 1].public_constraints()[action_player].len() == 1
                || self.history[self.history.len() - 1].public_constraints().iter().map(|v| v.iter().filter(|c| **c == card_of_interest).count() as u8).sum::<u8>() == 3
                // I like this
                || (
                    *reveal_i == card_of_interest &&
                    // had to have withdrawn if other 2 cards are outside player
                    self.history[index - 1].public_constraints().iter().enumerate().filter(|(p, _)| *p != action_player).map(|(_, v)| v.iter().filter(|c| **c == card_of_interest).count()).sum::<usize>()
                    + self.history[index - 1].inferred_constraints().iter().enumerate().filter(|(p, _)| *p != action_player).map(|(_, v)| v.iter().filter(|c| **c == card_of_interest).count()).sum::<usize>()
                    == 2
                )
                || bool_2_cards_outside_player
                || false // *reveal != card_of_interest && 3 cards outside player
            )
        } else {
            false
        }
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
                                ActionInfoName::Discard => {},
                            }
                            // match &mut self.history[i].action_info {
                            //     ActionInfo::RevealRedraw { *reveal, redraw, relinquish } => {
                            //         // CASE player put the considered card into the pile
                            //         // If all cards for redraw are known
                            //         // If current player has RevealRedrawn before they must have withdrawn the Discarded card
                            //         // 
                            //         // CASE Player RevealRedraw same card
                            //         // - did not change the pile state => no early exit
                            //         if Some(*reveal) == *redraw {
                            //             continue;
                            //         }
                            //         // CASE Player RevealRedraw revealed the card of interest and did not redraw it back
                            //         // - pile got the redraw_card here => early exit
                            //         if reveal == redraw_card {
                            //             // CASE player put the considered card into the pile
                            //             // All card locations for redraw_card are known just after previous reveal
                            //             // If current player has RevealRedrawn has redrawn redraw_card
                            //             // Then last RevealRedraw must have left it inside
                            //             // I need to know that player has the third card just after reveal, and before redraw
                            //             if self.history[i - 1].known_card_count(redraw_card) == 2 
                            //             && !self.history[i - 1].inferred_constraints()[self.history[i].player() as usize].contains(&redraw_card) {
                            //                 if let Some(relinquish_card) = relinquish {
                            //                     *relinquish_card = *reveal;
                            //                     return true
                            //                 }
                            //             }
                            //             return false
                            //         }
                            //         //
                            //     },
                            //     ActionInfo::ExchangeDrawChoice { draw, relinquish } => {
                            //         if !draw.is_empty() || !relinquish.is_empty() {
                            //             // TODO: [CASE] Technically there could be cases to handle where cards are known
                            //             // For now we stop at any Ambassador
                            //             return false
                            //         }
                            //     },
                            //     ActionInfo::Start => {
                            //         debug_assert!(false, "Should not have Start when index is not 0");
                            //     },
                            //     ActionInfo::Discard { .. } => {},
                            // }
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
            }
            ActionInfo::ExchangeDrawChoice { draw, relinquish } => {
                // unimplemented!();
            }
        }
        false
    }
    /// Your generic lookback for an inferred constraint
    pub fn lookback_2(&self, index: usize) {

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
            ActionInfo::Start => {
                // TODO: Change
                // STATUS: Currently testing with not storing inferred items, but only impossible for bad_push
                // self.regenerate_game_start();
                // // We try saving only the state before additional inference
                // // The issue with this is that we won't get to read impossible?
                // self.history[history_index].meta_data = self.to_meta_data();
                // self.add_inferred_information();
                // // But we save impossible constraints anyway so future states can read it
                // // TODO: THEORY CHECK
                // self.generate_impossible_constraints();
                // // Setting impossible constraints for meta_data
                // self.history[history_index].set_impossible_constraints(&self.impossible_constraints);
                // log::trace!("calculate_stored_move generate_impossible_constraints history_index: {history_index}");
                // log::info!("recalculated_stored_move end: player: {player_id} {} {:?}", history_index, self.history[history_index].action_info());
                // self.printlog();
                return
            },
            ActionInfo::StartInferred => {
                // TODO: Change
                // STATUS: Currently testing with not storing inferred items, but only impossible for bad_push
                self.regenerate_game_start();
                // We try saving only the state before additional inference
                // The issue with this is that we won't get to read impossible?
                self.add_inferred_information();
                // But we save impossible constraints anyway so future states can read it
                // TODO: THEORY CHECK
                // self.generate_impossible_constraints();
                // Setting impossible constraints for meta_data
                // self.history[history_index].set_impossible_constraints(&self.impossible_constraints);
                // log::trace!("calculate_stored_move generate_impossible_constraints history_index: {history_index}");
                // log::info!("recalculated_stored_move end: player: {player_id} {} {:?}", history_index, self.history[history_index].action_info());
                // self.printlog();
                // return
            },
            ActionInfo::Discard{ discard} => {
                self.death(history_index, player_id as usize, discard);
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
                    None => {
                        // TODO: [OPTIMIZE] If lookback_1 checks based on reveal
                        //  Then whenever u regenerate, you don't really have to lookback as there is nothing new to check
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

impl PathDependentCollectiveConstraint {
    fn group_constraints(&self) -> [&Vec<CompressedGroupConstraint>;5] {
        [&self.group_constraints_amb, 
        &self.group_constraints_ass, 
        &self.group_constraints_cap, 
        &self.group_constraints_duk, 
        &self.group_constraints_con]
    }
    fn group_constraints_mut(&mut self) -> [&mut Vec<CompressedGroupConstraint>;5] {
        [&mut self.group_constraints_amb, 
        &mut self.group_constraints_ass, 
        &mut self.group_constraints_cap, 
        &mut self.group_constraints_duk, 
        &mut self.group_constraints_con]
    }
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
    /// Returns true if there are no constraints
    pub fn is_empty(&self) -> bool {
        // [COMBINE SJ]
        self.public_constraints.iter().all(|v| v.is_empty()) && 
        self.inferred_constraints.iter().all(|v| v.is_empty()) && 
        self.group_constraints_amb.is_empty() &&
        self.group_constraints_ass.is_empty() &&
        self.group_constraints_cap.is_empty() &&
        self.group_constraints_duk.is_empty() &&
        self.group_constraints_con.is_empty()
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
    /// Returns true if redundant on basis of inferred and public info
    /// 
    /// NOTE:
    /// EXAMPLE:
    /// - part list includes every alive players
    pub fn is_known_information(&self, group: &CompressedGroupConstraint) -> bool {
        let participation_list: [bool; 7] = group.get_list();
        for player_id in 0..6 as usize {
            if !participation_list[player_id] && self.player_is_alive(player_id) {
                // if not in group and player is alive
                return false
            }
        }
        // Returns true if all players outside group are dead
        // i.e. all players alive are inside the group 
        // Group must include pile or it is false
        participation_list[6]
    }
    /// Logs the state
    pub fn printlog(&self) {
        log::info!("{}", format!("Public Constraints: {:?}", self.public_constraints));
        log::info!("{}", format!("Inferred Constraints: {:?}", self.inferred_constraints));
        log::info!("{}", format!("Group Constraints:"));
        log::info!("{}", format!("\t AMB: {:?}", self.group_constraints_amb));
        log::info!("{}", format!("\t ASS: {:?}", self.group_constraints_ass));
        log::info!("{}", format!("\t CAP: {:?}", self.group_constraints_cap));
        log::info!("{}", format!("\t DUK: {:?}", self.group_constraints_duk));
        log::info!("{}", format!("\t CON: {:?}", self.group_constraints_con));
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
    /// Adds to tracked public constraints
    /// NOTE:
    /// - Only adds a dead player
    /// - Does not assume anything
    /// - Does not consider the informational state of collective constraint
    pub fn add_dead_player_constraint(&mut self, player_id : usize, card: Card) {
        debug_assert!(self.dead_card_count()[card as usize] < 3, "Too many cards in dead_card_count for card: {:?}, found: {}", card, self.dead_card_count()[card as usize]);
        debug_assert!(self.player_is_alive(player_id), "Cannot add more dead cards to player that is already dead!, Current player public_constraint len: {}", self.public_constraints[player_id].len());
        log::trace!("In add_dead_player_constraint");
        // PD Obsolete
        // self.dead_card_count[card as usize] += 1;
        self.public_constraints[player_id].push(card);
        if let Some(pos) = self.inferred_constraints[player_id].iter().position(|&c| c == card) {
            self.inferred_constraints[player_id].swap_remove(pos);
        }
        if self.clear_group_constraints(card) {
            return
        }
        let mut i: usize = 0;
        let group_constraints = &mut self.group_constraints_mut()[card as usize];
        while i < group_constraints.len() {
            if group_constraints[i].get_player_flag(player_id) {
                if group_constraints[i].count_alive() > 1 {
                    group_constraints[i].add_dead_count(1);
                    group_constraints[i].sub_alive_count(1);
                } else {
                    group_constraints.swap_remove(i);
                    continue;
                }
            }
            i += 1;
        }
    }
    /// Removes all group_constraints that have particular card and replace with another group with total_count == 3 if all cards are known
    /// in either inferred_constraints or public_constraints
    /// We need the 3 group to stay, for impossible cases
    /// Does not change the internal redundant state
    ///     - If groups are not internally redundant => returns them as not internally redundant
    ///     - If groups are internally redundant => probably returns them as internally redundant
    pub fn clear_group_constraints(&mut self, card: Card) -> bool {
        // TODO: Don't Clear() all, you still want to keep the group that has all 3, for impossible testing
        // TODO: Maybe clear if not 3 idk... or add in just 1 group
        // TODO: Clear() and add 1 group if all dead, else i guess just leave them?
        log::trace!("In clear_group_constraints");
        let total_dead_known = self.public_constraints.iter().map(|v| v.iter().filter(|c| **c == card).count() as u8).sum::<u8>();
        let total_alive_known = self.inferred_constraints.iter().map(|v| v.iter().filter(|c| **c == card).count() as u8).sum::<u8>();
        if total_dead_known + total_alive_known == 3 {
            // Clears group_constraints if all cards are known and in inferred / dead, and leaves a 3 group for impossible_cards to check
            self.group_constraints_mut()[card as usize].clear();
            let mut group = CompressedGroupConstraint::zero();
            group.set_card(card);
            for player in 0..7 {
                if self.public_constraints[player].contains(&card) {
                    group.set_player_flag(player, true);
                    continue;
                }
                if self.inferred_constraints[player].contains(&card) {
                    group.set_player_flag(player, true);
                }
            }
            group.set_dead_count(total_dead_known);
            group.set_alive_count(total_alive_known);
            group.set_total_count(3);
            self.group_constraints_mut()[card as usize].push(group);
            log::info!("End of clear_group_constraints");
            self.printlog();
            return true
        }
        false
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
    /// Calculates all the known cards that are within player and pile
    /// - Assumption here is that there are no solo group constraints that represent 1 player only!
    pub fn total_known_alive_with_player_and_pile(&mut self, player_id: usize) -> [u8; 5] {
        let mut output: [u8; 5] = [0; 5];
        // [COMBINE SJ]
        for card in self.inferred_constraints[player_id].iter() {
            output[*card as usize] += 1;
        }
        for (card_num, group_constraints) in [&self.group_constraints_amb, &self.group_constraints_ass, &self.group_constraints_cap, &self.group_constraints_duk, &self.group_constraints_con].iter().enumerate() {
            for group in group_constraints.iter() {
                if group.part_list_is_subset_of_player_and_pile(player_id) {
                    // Technically this should only be a group of pile and player if it works properly
                    debug_assert!(group.get_player_flag(player_id) && group.get_player_flag(6), "Either Player or Pile are false, assumption failed!");
                    output[card_num] = output[card_num].max(group.count_alive());
                }
            }
        }
        output
    }
    // TODO: [THEORY REVIEW] Read through and theory check
    // TODO: Investigate Initial group prune relevance here
    // TODO: Investigate if how inferred groups can be produced here 
    // TODO: CHECK how it updates inferred group => If last group, all inferred should be removed. If first card dead, remove one from inferred.
    /// Adds a public constraint, and prunes group constraints that are redundant
    /// NOTE:
    /// - Assumes no group is redundant before adding
    /// - Assumes no dead player info is in groups before adding
    /// - Leaves no group redundant after adding
    /// - Leaves no dead player info in groups
    /// CASES:
    /// We update group_constraints where player_id flag is true only
    /// Player dies and reveal card A
    /// group constraints with group.card() == A
    ///      - Reveal A => (alive, alive) [A, !A] and a group with [1 0 0 0 0 0 1] A 2 => flag = false and count_alive - 1, inferred A - 1
    ///      - Reveal A => (alive, alive) [A, A] and a group with [1 0 0 0 0 0 1] A 2 => flag = false and count_alive - 2, inferred A - 1
    ///      - Reveal A => (alive, alive) [A, A] and a group with [1 0 0 0 0 0 1] A 3 => flag = false and count_alive - 2, inferred A - 1
    ///      - Reveal A => (alive, alive) [A, X] and a group with [1 0 0 0 0 0 1] A 3 => count_alive - 1, count_dead + 1, inferred A - 1
    ///      - Reveal A => (alive, alive) [A, X] and a group with [1 0 0 0 0 0 1] A 2 => count_alive - 1, count_dead + 1, inferred A + 1
    ///      - Reveal A => (alive, alive) [A, X] and a group with [1 0 0 0 0 0 1] A 1 => remove
    ///      - Reveal A => (dead, alive) [!A, A] and a group with [1 0 0 0 0 0 1] A 2 => flag = false and count_alive - 1
    ///      - Reveal A => (dead, alive) [!A, A] and a group with [1 0 0 0 0 0 1] A 1 => remove
    ///      - Reveal A => (dead, alive) [A, A] and a group with [1 0 0 0 0 0 1] A 2 => flag = false and count_alive - 1, count_dead - 1
    ///      - Reveal A => (dead, alive) [A, A] and a group with [1 0 0 0 0 0 1] A 1 => remove
    /// CONCLUSION 1: If all of the player's cards are known, since revealed card is dead, we can change flag to false and subtract the number of alive cards and dead cards
    /// CONCLUSION 2: Remove one of dead card from inferred group if it is inside
    /// CONCLUSION 2: If all flags 0 => remove | If alive_count less than equals to 0 => remove
    /// group constraints with group.card() != A | (d0a3 -> dead 0 alive 3)
    ///      - Reveal A => (alive, now dead) [C, A] and a group with [1 0 0 0 0 0 1] C d0a3 => flag = false, count_alive - 1
    ///      - Reveal A => (alive, now dead) [C, A] and a group with [1 0 0 0 0 0 1] C d0a2 => flag = false, count_alive - 1
    ///      - Reveal A => (alive, now dead) [C, A] and a group with [1 0 0 0 0 0 1] C d0a1 => remove
    ///      - Reveal A => (alive, now dead) [C, A] and a group with [1 0 0 0 0 0 1] C d1a2 => flag = false, count_alive - 1
    ///      - Reveal A => (alive, now dead) [C, A] and a group with [1 0 0 0 0 0 1] C d2a1 => remove
    ///      - Reveal A => (now dead, alive) [A, X] and a group with [1 0 0 0 0 0 1] C d?a? => unchanged => prune cases that add info to inferred group are handled later
    ///      - Reveal A => (dead, now dead) [C, A] and a group with [1 0 0 0 0 0 1] C d1a2 => flag = false, count_dead - 1
    ///      - Reveal A => (dead, now dead) [C, A] and a group with [1 0 0 0 0 0 1] C d1a1 => flag = false, count_dead - 1
    /// CONCLUSION 1: If all of the player's cards are known, since revealed card is alive, we can change flag to false and subtract the number of alive cards and dead cards
    /// CONCLUSION 2: If all flags 0 => remove | If alive_count less than equals to 0 => remove
    /// CONCLUSION 3: If not all of a player's cards are known, leave then unchanged
    /// CONCLUSION 4: Groups where player_id is 0 are unaffected. They will be handled later in generate inferred constraints, and any further redundant pruning.
    /// FINAL CONCLUSION: Its the same treatment regardless of what card the group is, because we really are comparing a player's hand to the groups.
    ///                   The algo is less about a reveal and more about how to update groups after new information is added from the reveal.
    /// TODO: Exact same as reveal except you add to public constraint
    pub fn death(&mut self, history_index: usize, player_id: usize, card: Card) {
        log::trace!("In death");
        // [THINK] does death_group adjustment need to occur before a wipe and add 3-group
        // [OLD]
        // self.add_dead_player_constraint(player_id, card);
        // [NEW]
        // Check before recursing
        // if self.lookback_1(history_index) {
        //     // If true, it will have rerun the entire history including the current move
        //     return
        // }
        self.add_dead_card(player_id, card);
        // TODO: ADD COMPLEMENT PRUNE is probably useful here since its not done in group_redundant_prune()
        // TODO: [THOT] Group constraints being a subset of info in inferred constraints mean it can be pruned too
        //      - like if inferred info reflects the same thing as group constraint
        // QUESTION: How does inferred constraints help in determining if group is redundant? Should this be pruned above?
        // TODO: Needs to do dead player prune
        log::trace!("After add_dead_player_constraint");
        // TODO: [OPTIMIZE] Add subset groups like in RevealRedraw, so don't need to run add_inferred_groups
        self.printlog();
        self.group_redundant_prune();
        self.add_inferred_information();
        // Change revealed_status at the end after all groups using it have been updated
        // This is handled in add_dead_card
        // self.revealed_status[player_id].clear();
    }
    /// Used for when move is added to history and not for recalculation
    pub fn death_initial(&mut self, player_id: usize, card: Card) {
        log::trace!("In death_initial");
        // [THINK] does death_group adjustment need to occur before a wipe and add 3-group
        // [OLD]
        // self.add_dead_player_constraint(player_id, card);
        // [NEW]

        self.add_dead_card_initial(player_id, card);
        // TODO: ADD COMPLEMENT PRUNE is probably useful here since its not done in group_redundant_prune()
        // TODO: [THOT] Group constraints being a subset of info in inferred constraints mean it can be pruned too
        //      - like if inferred info reflects the same thing as group constraint
        // QUESTION: How does inferred constraints help in determining if group is redundant? Should this be pruned above?
        // TODO: Needs to do dead player prune
        log::trace!("After add_dead_player_constraint");
        // TODO: [OPTIMIZE] Add subset groups like in RevealRedraw, so don't need to run add_inferred_groups
        self.printlog();
        self.group_redundant_prune();
        self.add_inferred_information();
        // Change revealed_status at the end after all groups using it have been updated
        // This is handled in add_dead_card
        // self.revealed_status[player_id].clear();
    }
    // TODO: Temp remove revealed_status part
    /// Adds dead card to public constraint.
    /// Adjusts groups affected by the discarded card.
    /// If discarded card is part of the revealredraw network (revealed_status)
    /// Checks if it affects groups with single_card == 1
    /// Generates appropriate subset groups
    /// Player 1 Reveals Captain
    /// [0 1 0 0 0 0 1] [0 1 0 0 0 0 0] 1 Alive Captain
    /// Player 5 Reveals Duke
    /// [0 0 0 0 0 1 1] [0 0 0 0 0 1 0] 1 Alive Duke
    /// This should modify
    /// [0 1 0 0 0 0 1] [0 1 0 0 0 0 0] 1 Alive Captain
    /// => [0 1 0 0 0 1 1] [0 1 0 0 0 1 0] 1 Alive Captain
    /// Player 5 Discards Captain
    /// Since Player 5 has a Captain group with Single Flag 1, 
    /// If the complementary group [1 0 1 1 1 0 0] has all the remaining Captains,
    /// => The Discarded Captain must come from the Single Card in the group
    /// So we can adjust all groups with Player 5 Single Flag 1, including other cards
    pub fn add_dead_card(&mut self, player_id: usize, card: Card) {
        // TODO: [COMBINE] Combine with add_dead_player_constraint
        // TODO: [OPTIMIZE] Actually don't need to search all the groups if you know who has revealed before
        // Revealed_status stores a state of who has revealed, and which cards have been revealed
        // P1 => [0 1 0 0 0 0]
        // P5 => [0 1 0 0 1 0]
        // P1 mix => [0 0 0 0 1 0]
        // OK maybe dont need to store which cards have been revealed?
        // If player has revealed this card before
        log::info!("In add_dead_card: {}, card: {:?}", player_id, card);
        // Adding dead card
        self.public_constraints[player_id].push(card);
        
        // If the dead card was already inferred, remove it
        if let Some(pos) = self.inferred_constraints[player_id].iter().position(|&c| c == card) {
            self.inferred_constraints[player_id].swap_remove(pos);
        }
        self.dead_card_prune(player_id, card);
    }
    pub fn add_dead_card_initial(&mut self, player_id: usize, card: Card) {
        // TODO: [COMBINE] Combine with add_dead_player_constraint
        // TODO: [OPTIMIZE] Actually don't need to search all the groups if you know who has revealed before
        // Revealed_status stores a state of who has revealed, and which cards have been revealed
        // P1 => [0 1 0 0 0 0]
        // P5 => [0 1 0 0 1 0]
        // P1 mix => [0 0 0 0 1 0]
        // OK maybe dont need to store which cards have been revealed?
        // If player has revealed this card before
        log::info!("In add_dead_card: {}, card: {:?}", player_id, card);
        // Adding dead card
        self.public_constraints[player_id].push(card);
        
        // If the dead card was already inferred, remove it
        if let Some(pos) = self.inferred_constraints[player_id].iter().position(|&c| c == card) {
            self.inferred_constraints[player_id].swap_remove(pos);
        }
        // Check before recursing
        if self.lookback_initial() {
            // If true, it will have rerun the entire history including the current move
            return
        }
        self.dead_card_prune(player_id, card);
    }
    pub fn dead_card_prune(&mut self, player_id: usize, card: Card) {
        // Clear all if 
        // TODO: [OPTIMIZE] use to prevent unneeded evaluation
        // [QN] rethink this clear, does it affect the operation below?
        //      - I think not as if you know all the cards for a card, you cant infer more from it...
        //      - But you should think about the single flags maybe?
        let bool_all_cards_dead_or_known: bool = self.clear_group_constraints(card);
        log::info!("After clear_group_constraints");

        // === REMOVED REVEALED_STATUS LOGIC ===

        // Remove flags for that player
        // TODO: But i think we do need the prune part lol (setting discarded single flag to false)
        let mut new_inferred: Vec<CompressedGroupConstraint> = Vec::with_capacity(5);
        let mut group_constraints = [&mut self.group_constraints_amb, 
        &mut self.group_constraints_ass, 
        &mut self.group_constraints_cap, 
        &mut self.group_constraints_duk, 
        &mut self.group_constraints_con];
        
        let card_num_range = (0..5).filter(|x| *x != card as usize);
        let mut dead_card_counts: [u8; 5] = [0; 5];
        for card in self.public_constraints[player_id].iter() {
            dead_card_counts[*card as usize] += 1;
        }
        let mut inferred_card_counts: [u8; 5] = [0; 5];
        for card in self.inferred_constraints[player_id].iter() {
            inferred_card_counts[*card as usize] += 1;
        }
        // Handle groups where cards are not same as discarded card
        // Get total cards known
        // Check all cards known about player
        // Case A: Totally dead => update group
        // Case B: 1 dead another card known => update group
        // Case C: 1 dead another card unknown => update group
        
        if self.public_constraints[player_id].len() == 2 {
            // Case A: Totally dead => update group
            // Handle groups where cards are same as discarded card
            if !bool_all_cards_dead_or_known && dead_card_counts[card as usize] != 3 { // else handled in clear_group_constraints
                let groups = &mut group_constraints[card as usize];
                let mut i: usize = 0;
                while i < groups.len() {
                    let group = &mut groups[i];
                    if group.get_player_flag(player_id) {
                        group.set_player_flag(player_id, false);
                        if group.count_alive() > 1 {
                            group.sub_alive_count(1);
                            // -1 because we have not added the current dead_card in this turn
                            group.sub_dead_count(dead_card_counts[card as usize] - 1);
                            // 1 + dcd - 1 = dcd
                            group.sub_total_count(dead_card_counts[card as usize]);
    
                            if group.is_single_player_part_list() {
                                new_inferred.push(*group);
                                groups.swap_remove(i);
                                log::trace!("Group removed!");
                                continue;
                            }
                        } else {
                            groups.swap_remove(i);
                            continue;
                        }
                    }
                    i += 1;
                }
            }
            // Case A: Totally dead => update group
            // Handle groups where cards are not same as discarded card
            for card_num in card_num_range {
                if dead_card_counts[card_num] != 3{
                    let groups = &mut group_constraints[card_num];
                    let mut i: usize = 0;
                    while i < groups.len() {
                        let group = &mut groups[i];
                        if group.get_player_flag(player_id) {
                            group.set_player_flag(player_id, false);
                            if inferred_card_counts[card_num] < group.count_alive() {
                                group.sub_alive_count(inferred_card_counts[card_num]);
                                group.sub_dead_count(dead_card_counts[card_num]);
                                group.sub_total_count(inferred_card_counts[card_num] + dead_card_counts[card_num]);
                                if group.is_single_player_part_list() {
                                    new_inferred.push(*group);
                                    groups.swap_remove(i);
                                    log::trace!("Group removed!");
                                    continue;
                                }
                            } else {
                                groups.swap_remove(i);
                                continue;
                            }
                        }
                        i += 1;
                    }
                }
            }
        } else {
            // self.public_constraints[player_id].len() == 1
            // Case B: 1 dead another card known => update group
            // Case C: 1 dead another card unknown => update group
            if self.inferred_constraints[player_id].len() == 1 {
                // Case B: 1 dead another card known => update group
                // Handle groups where cards are same as discarded card
                if !bool_all_cards_dead_or_known && dead_card_counts[card as usize] != 3 { // else handled in clear_group_constraints
                    let groups = &mut group_constraints[card as usize];
                    let mut i: usize = 0;
                    while i < groups.len() {
                        let group = &mut groups[i];
                        if group.get_player_flag(player_id) {
                            group.set_player_flag(player_id, false);
                            if group.count_alive() > 1 + inferred_card_counts[card as usize] {
                                // -1 for dead card, - for other alive_cards
                                group.sub_alive_count(1 + inferred_card_counts[card as usize]);
                                group.sub_total_count(1 + inferred_card_counts[card as usize]);
                                if group.is_single_player_part_list() {
                                    // TODO: [OPTIMIZE] maybe don't push into group if it might be DUP or its already in inferred_constraints?
                                    new_inferred.push(*group);
                                    groups.swap_remove(i);
                                    log::trace!("Group removed!");
                                    continue;
                                }
                            } else {
                                groups.swap_remove(i);
                                continue;
                            }
                        }
                        i += 1;
                    }
                }
                // Case B: 1 dead another card known => update group
                // Handle groups where cards are not same as discarded card
                for card_num in card_num_range {
                    if dead_card_counts[card_num] != 3 {
                        let groups = &mut group_constraints[card_num];
                        let mut i: usize = 0;
                        while i < groups.len() {
                            let group = &mut groups[i];
                            if group.get_player_flag(player_id) {
                                group.set_player_flag(player_id, false);
                                if group.count_alive() > inferred_card_counts[card_num] {
                                    group.sub_alive_count(inferred_card_counts[card_num]);
                                    group.sub_total_count(inferred_card_counts[card_num]);
                                    if group.is_single_player_part_list() {
                                        new_inferred.push(*group);
                                        groups.swap_remove(i);
                                        log::trace!("Group removed!");
                                        continue;
                                    }
                                } else {
                                    groups.swap_remove(i);
                                    continue;
                                }
                            }
                            i += 1;
                        }
                    }
                }
            } else {
                // Case C: 1 dead another card unknown => update group
                // Handle groups where cards are same as discarded card
                if !bool_all_cards_dead_or_known && dead_card_counts[card as usize] != 3{ // else handled in clear_group_constraints
                    let groups = &mut group_constraints[card as usize];
                    let mut i: usize = 0;
                    while i < groups.len() {
                        let group = &mut groups[i];
                        if group.get_player_flag(player_id) {
                            // Set false cos u have 1 life after anyways, so nothing really affected for path dependent approach
                            // Case where the single card is the card being discard, will be handled in lookback
                            group.set_single_card_flag(player_id, false);
                            if group.count_alive() > 1{
                                // -1 for dead card
                                group.add_dead_count(1);
                                group.sub_alive_count(1);
                            } else {
                                groups.swap_remove(i);
                                continue;
                            }
                        }
                        i += 1;
                    }
                }
            }
        }
        // Deal with new_inferred cards
        log::info!("Before add_inferred_card_bulk");
        self.printlog();
        self.add_inferred_card_bulk(new_inferred);
    }
    /// Bulk add_inferred_card
    pub fn add_inferred_card_bulk(&mut self, mut single_flag_batch: Vec<CompressedGroupConstraint>) -> bool{
        // TODO: Inferred card needs to add other things inside other than just the group adjustment
        // [THINK] I think we need to keep our single person inferred group if the single card flag is 1 to allow us to differentiate in add_dead_card...
        // TODO: See reveal, and death()
        log::info!("In add_inferred_card_bulk");
        let mut card_changes: Vec<Vec<usize>> = vec![Vec::with_capacity(2); 5]; // Store (player_id, bool_counts => false: 1, true: 2)
        // Add new_inferred_constraints
        // TODO: [FIX] Adding new_inferred constraints requires reconsidering the entire group_constraints list not just the new_groups
        let mut bool_changes = false;
        // TODO: CHECK, Im not sure why old implementation does not remove dups
        // TODO: [OPTIMIZE] prety sure you can insert if not already inside
        single_flag_batch.sort_unstable_by_key(|c| c.num());
        single_flag_batch.dedup();
        while let Some(single_flag_group) = single_flag_batch.pop() {
            // Something is wrong if this panics, all groups should have a single flag, not no flags
            let player_id = single_flag_group.get_set_players().iter().position(|b| *b).unwrap();
            let card: Card = single_flag_group.card();
            log::trace!("add_inferred_card_bulk: adding group: {}", single_flag_group);
            bool_changes = self.add_inferred_card(player_id, card, single_flag_group.count_alive(), &mut card_changes).0 || bool_changes;
            // batch prune
            // This is formatted like this because of some &mutable and immutable borrow issue with the compiler..
            let self_groups = [&mut self.group_constraints_amb, 
                                                                        &mut self.group_constraints_ass, 
                                                                        &mut self.group_constraints_cap, 
                                                                        &mut self.group_constraints_duk, 
                                                                        &mut self.group_constraints_con];
            // Ensures no inferred constraint makes a group redundant
            // [OPTIMIZE] is this even required? I guess its for removing groups affected by the inferred additions
            for (card_num, changes) in card_changes.iter().enumerate() {
                let mut i: usize = 0;
                'group_removal: while i < self_groups[card_num].len() {
                    for &player_id in changes {
                        // If player flag is true and number of cards player now has
                        if self_groups[card_num][i].get_player_flag(player_id) {
                            let count_alive = self.inferred_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                            if count_alive >= self_groups[card_num][i].count_alive() {
                                log::trace!("add_inferred_card_bulk removing group: {}", self_groups[card_num][i]);
                                self_groups[card_num].swap_remove(i);
                                continue 'group_removal;
                            }
                        }
                    }
                    i += 1;
                }
            }
        }
        bool_changes
    }
    /// To facilitate recursive addition of inferred_card
    /// Should store newly discovered inferred groups and return that
    /// Returns
    /// [0] => bool of whether changes were made
    /// [1] => More discovered inferred constraints expressed as a single flag CompressedGroupConstraint
    /// [2] => changes to be used for batch prune | Store? (player_id, bool_counts => false: 1, true: 2)
    /// TODO [OPTIMIZE] where even are we using the second return?
    /// TODO: Create a standalone without changes vec
    pub fn add_inferred_card(&mut self, player_id: usize, card: Card, alive_count: u8, card_changes: &mut Vec<Vec<usize>>) -> (bool, Vec<CompressedGroupConstraint>) {
        // TODO: [IMPLEMENT] Need to add the case for single_card_flag on card != card_num ZZ2A as per add_dead_cards
        log::info!("In add_inferred_card");
        let mut bool_changes = false;
        // [OPTIMIZE] See bulk, maybe dont even need card_changes
        // [OPTIMIZE] cant u just add the difference here instead of a branch
        // nothing special just standard changes
        // [OPTIMIZE][IMPT] check for if inferred_constraints contains card before pushing it in! Its unneeded allocations
        match alive_count {
            1 => {
                // One card known
                if !self.inferred_constraints[player_id].contains(&card) {
                    log::trace!("");
                    log::trace!("=== add_subset_groups Inferred 1 === ");
                    log::trace!("add_sub_groups adding player considered: {}", player_id);
                    log::trace!("add_sub_groups Before self.public_constraints: {:?}", self.public_constraints);
                    log::trace!("add_sub_groups Before self.inferred_constraints: {:?}", self.inferred_constraints);
                    self.inferred_constraints[player_id].push(card);
                    log::trace!("self.inferred_constraints: {:?}", self.inferred_constraints);
                    debug_assert!(self.inferred_constraints[player_id].len() < 4, "F1");
                    card_changes[card as usize].push(player_id);
                    log::trace!("add_sub_groups After self.public_constraints: {:?}", self.public_constraints);
                    log::trace!("add_sub_groups After self.inferred_constraints: {:?}", self.inferred_constraints);
                    bool_changes = true;
                    // TODO: Needs to prune the groups too...
                    // TODO: Adjust counts properly too... or remove inferred_counts
                }
            },
            2 => {
                if player_id == 6 {
                    let current_count = self.inferred_constraints[player_id].iter().filter(|c| **c == card).count() as u8; 
                    if alive_count > current_count {
                        let no_to_push = alive_count - current_count;
                        log::trace!("Add_inferred_card adding {no_to_push} of card: {:?}", card);
                        for _ in 0..no_to_push {
                            self.inferred_constraints[player_id].push(card);
                            log::trace!("self.inferred_constraints: {:?}", self.inferred_constraints);
                            debug_assert!(self.inferred_constraints[player_id].len() < 4, "F2");
                            bool_changes = true;
                        }
                    }
                } else {
                    // Both cards known
                    if self.inferred_constraints[player_id] != vec![card; 2] {
                        log::trace!("");
                        log::trace!("=== add_subset_groups Inferred 2 === ");
                        log::trace!("add_sub_groups adding player considered: {}", player_id);
                        log::trace!("add_sub_groups Before self.public_constraints: {:?}", self.public_constraints);
                        log::trace!("add_sub_groups Before self.inferred_constraints: {:?}", self.inferred_constraints);
                        self.inferred_constraints[player_id].clear();
                        self.inferred_constraints[player_id].push(card);
                        self.inferred_constraints[player_id].push(card);
                        log::trace!("self.inferred_constraints: {:?}", self.inferred_constraints);
                        debug_assert!(self.inferred_constraints[player_id].len() < 3, "F3");
                        card_changes[card as usize].push(player_id);
                        log::trace!("add_sub_groups After self.public_constraints: {:?}", self.public_constraints);
                        log::trace!("add_sub_groups After self.inferred_constraints: {:?}", self.inferred_constraints);
                        // TODO: Needs to prune the groups too... some batch function that prunes groups based on changes?
                        bool_changes = true;
                    }
                }
                // TODO: Adjust counts properly too... or remove inferred_counts
            },
            3 => {
                if player_id == 6 {
                    let current_count = self.inferred_constraints[player_id].iter().filter(|c| **c == card).count() as u8;
                    if alive_count > current_count {
                        let no_to_push = alive_count - current_count;
                        for _ in 0..no_to_push {
                            self.inferred_constraints[player_id].push(card);
                            log::trace!("self.inferred_constraints: {:?}", self.inferred_constraints);
                            debug_assert!(self.inferred_constraints[player_id].len() < 4, "F4");
                            bool_changes = true;
                        }
                    }
                } else {
                    log::trace!("alive_count: {}", alive_count);
                    debug_assert!(false, "You really should not be here... there should only be alive_count of 2 or 1 for a single player!");
                }
                // TODO: Adjust counts properly too... or remove inferred_counts
            },
            _ => {
                log::trace!("alive_count: {}", alive_count);
                debug_assert!(false, "You really should not be here... there should only be alive_count of 2 or 1 for a single player!");
            }
        }
        let bool_all_cards_dead_or_known: bool = self.clear_group_constraints(card);
        // === REMOVED REVEALED_STATUS LOGIC ===
        // TODO: But i think we maybe need prune part? lol (leave single flag alone)
        // For now im not gonna prune because I need supergroups anyways so.
        let mut new_inferred = Vec::with_capacity(5); // temp dummy

        // Case A: 1 inferred other card dead => update group
        // Case B: all cards inferred or known => update group
        // Case C: 1 inferred no other card known => update group
        let mut group_constraints = [&mut self.group_constraints_amb, 
        &mut self.group_constraints_ass, 
        &mut self.group_constraints_cap, 
        &mut self.group_constraints_duk, 
        &mut self.group_constraints_con];
        if bool_changes {
            // TODO: [FIX/OPTIMIZE] change card_num_range to exclude those with full inferred + dead card counts known
            //      - Not having this leaves empty group_constraints, which may or may not be a good thing
            let card_num_range = (0..5).filter(|x| *x != card as usize);
            let mut dead_card_counts: [u8; 5] = [0; 5];
            for card in self.public_constraints[player_id].iter() {
                dead_card_counts[*card as usize] += 1;
            }
            let mut inferred_card_counts: [u8; 5] = [0; 5];
            for card in self.inferred_constraints[player_id].iter() {
                inferred_card_counts[*card as usize] += 1;
            }
            let dead_card_count = self.public_constraints[player_id].len();
            let inferred_card_count = self.inferred_constraints[player_id].len();
            if player_id != 6 {

                if dead_card_count + inferred_card_count == 2 {
                    // Case A: 1 inferred other card dead => update group
                    // Case B: all cards inferred or known => update group
                    // Handle groups where cards are same as inferred card
                    if !bool_all_cards_dead_or_known {
                        let groups = &mut group_constraints[card as usize];
                        let mut i: usize = 0;
                        while i < groups.len() {
                            let group = &mut groups[i];
                            if group.get_player_flag(player_id) {
                                group.set_player_flag(player_id, false);
                                if group.count_alive() > inferred_card_counts[card as usize] {
                                    group.sub_alive_count(inferred_card_counts[card as usize]);
                                    group.sub_dead_count(dead_card_counts[card as usize]);
                                    group.sub_total_count(inferred_card_counts[card as usize] + dead_card_counts[card as usize]);
                                    if group.is_single_player_part_list() {
                                        new_inferred.push(*group);
                                        groups.swap_remove(i);
                                        log::trace!("Group removed!");
                                        continue;
                                    }
                                } else {
                                    groups.swap_remove(i);
                                    continue;
                                }
                            }
                            i += 1;
                        }
                    }
                    // Seems the same...
                    // Case A: 1 inferred other card dead => update group
                    // Case B: all cards inferred or known => update group
                    // Handle groups where cards are not same as inferred card
                    for card_num in card_num_range {
                        let groups = &mut group_constraints[card_num];
                        let mut i: usize = 0;
                        while i < groups.len() {
                            let group = &mut groups[i];
                            if group.get_player_flag(player_id) {
                                group.set_player_flag(player_id, false);
                                if group.count_alive() > inferred_card_counts[card_num as usize] {
                                    group.sub_alive_count(inferred_card_counts[card_num as usize]);
                                    group.sub_dead_count(dead_card_counts[card_num]);
                                    group.sub_total_count(inferred_card_counts[card_num as usize] + dead_card_counts[card_num]);
                                    if group.is_single_player_part_list() {
                                        new_inferred.push(*group);
                                        groups.swap_remove(i);
                                        log::trace!("Group removed!");
                                        continue;
                                    }
                                } else {
                                    groups.swap_remove(i);
                                    continue;
                                }
                            }
                            i += 1;
                        }
                    }
                } else {
                    if inferred_card_count != 1 {
                        println!("{:?}", self.public_constraints);
                        println!("{:?}", self.inferred_constraints);
                        panic!();
                    }
                    debug_assert!(inferred_card_count == 1, "In the case where you have added a card, and dead_card + inferred_cards != 2, it should only be when inferred_cards == 1");
                    debug_assert!(self.inferred_constraints[player_id].first() == Some(&card), "Since bool_changes == true, you must have added Card == card in");
                    // Case C: 1 inferred no other card known => update group
                    // It should also only
                    if !bool_all_cards_dead_or_known {
                        let groups = &mut group_constraints[card as usize];
                        let mut i: usize = 0;
                        while i < groups.len() {
                            let group = &mut groups[i];
                            if group.get_player_flag(player_id)  && group.count_alive() == 1{
                                groups.swap_remove(i);
                                continue;
                            }
                            i += 1;
                        }
                    }
                }    
            } else {
                if inferred_card_count == 3 {
                    // Case B: all cards inferred or known => update group
                    // Handle groups where cards are same as inferred card
                    if !bool_all_cards_dead_or_known {
                        let groups = &mut group_constraints[card as usize];
                        let mut i: usize = 0;
                        while i < groups.len() {
                            let group = &mut groups[i];
                            if group.get_player_flag(player_id) {
                                group.set_player_flag(player_id, false);
                                if group.count_alive() > inferred_card_counts[card as usize] {
                                    group.sub_alive_count(inferred_card_counts[card as usize]);
                                    group.sub_total_count(inferred_card_counts[card as usize]);
                                    if group.is_single_player_part_list() {
                                        new_inferred.push(*group);
                                        groups.swap_remove(i);
                                        log::trace!("Group removed!");
                                        continue;
                                    }
                                } else {
                                    groups.swap_remove(i);
                                    continue;
                                }
                            }
                            i += 1;
                        }
                    }
                    // Case B: all cards inferred or known => update group
                    // Handle groups where cards are not same as inferred card
                    for card_num in card_num_range {
                        let groups = &mut group_constraints[card_num];
                        let mut i: usize = 0;
                        while i < groups.len() {
                            let group = &mut groups[i];
                            if group.get_player_flag(player_id) {
                                group.set_player_flag(player_id, false);
                                if group.count_alive() > inferred_card_counts[card_num as usize] {
                                    group.sub_alive_count(inferred_card_counts[card_num as usize]);
                                    group.sub_total_count(inferred_card_counts[card_num as usize]);
                                    if group.is_single_player_part_list() {
                                        new_inferred.push(*group);
                                        groups.swap_remove(i);
                                        log::trace!("Group removed!");
                                        continue;
                                    }
                                } else {
                                    groups.swap_remove(i);
                                    continue;
                                }
                            }
                            i += 1;
                        }
                    }
                } else {
                    // Case D more than 0 inferred, but not full
                    if !bool_all_cards_dead_or_known {
                        let groups = &mut group_constraints[card as usize];
                        let mut i: usize = 0;
                        while i < groups.len() {
                            let group = &mut groups[i];
                            if group.get_player_flag(player_id) && group.count_alive() <= inferred_card_counts[card as usize] {
                                groups.swap_remove(i);
                                continue;
                            }
                            i += 1;
                        }
                    }
                    for card_num in card_num_range {
                        let groups = &mut group_constraints[card_num];
                        let mut i: usize = 0;
                        while i < groups.len() {
                            let group = &mut groups[i];
                            if group.get_player_flag(player_id) && group.count_alive() <= inferred_card_counts[card_num as usize] {
                                groups.swap_remove(i);
                                continue;
                            }
                            i += 1;
                        }
                    }
                }
            }
        }
        // TODO: REMOVE commented above when this is stable
        // let new_inferred = if bool_changes {
        //     self.inferred_card_prune(player_id, card)
        // } else {
        //     Vec::with_capacity(0)
        // };
        // TODO: Inferred card needs to prune too!
        (bool_changes, new_inferred)
    }
    pub fn add_inferred_card_unchecked(&mut self, player_id: usize, card: Card, alive_count: u8, card_changes: &mut Vec<Vec<usize>>) -> (bool, Vec<CompressedGroupConstraint>) {
        // TODO: [IMPLEMENT] Need to add the case for single_card_flag on card != card_num ZZ2A as per add_dead_cards
        log::info!("In add_inferred_card");
        let mut bool_changes = false;
        // [OPTIMIZE] See bulk, maybe dont even need card_changes
        // [OPTIMIZE] cant u just add the difference here instead of a branch
        // nothing special just standard changes
        // [OPTIMIZE][IMPT] check for if inferred_constraints contains card before pushing it in! Its unneeded allocations
        match alive_count {
            1 => {
                // One card known
                log::trace!("");
                log::trace!("=== add_subset_groups Inferred 1 === ");
                log::trace!("add_sub_groups adding player considered: {}", player_id);
                log::trace!("add_sub_groups Before self.public_constraints: {:?}", self.public_constraints);
                log::trace!("add_sub_groups Before self.inferred_constraints: {:?}", self.inferred_constraints);
                self.inferred_constraints[player_id].push(card);
                log::trace!("self.inferred_constraints: {:?}", self.inferred_constraints);
                debug_assert!(self.inferred_constraints[player_id].len() < 4, "F1");
                card_changes[card as usize].push(player_id);
                log::trace!("add_sub_groups After self.public_constraints: {:?}", self.public_constraints);
                log::trace!("add_sub_groups After self.inferred_constraints: {:?}", self.inferred_constraints);
                bool_changes = true;
                // TODO: Needs to prune the groups too...
                // TODO: Adjust counts properly too... or remove inferred_counts
            },
            2 => {
                unimplemented!();
            },
            3 => {
                unimplemented!()
            },
            _ => {
                log::trace!("alive_count: {}", alive_count);
                debug_assert!(false, "You really should not be here... there should only be alive_count of 2 or 1 for a single player!");
            }
        }
        let new_inferred = if bool_changes {
            self.inferred_card_prune(player_id, card)
        } else {
            Vec::with_capacity(0)
        };
        // TODO: Inferred card needs to prune too!
        (bool_changes, new_inferred)
    }
    fn inferred_card_prune(&mut self, player_id: usize, card: Card) -> Vec<CompressedGroupConstraint>{
        let bool_all_cards_dead_or_known: bool = self.clear_group_constraints(card);
        // === REMOVED REVEALED_STATUS LOGIC ===
        // TODO: But i think we maybe need prune part? lol (leave single flag alone)
        // For now im not gonna prune because I need supergroups anyways so.
        let mut new_inferred = Vec::with_capacity(5); // temp dummy

        // Case A: 1 inferred other card dead => update group
        // Case B: all cards inferred or known => update group
        // Case C: 1 inferred no other card known => update group
        let mut group_constraints = [&mut self.group_constraints_amb, 
        &mut self.group_constraints_ass, 
        &mut self.group_constraints_cap, 
        &mut self.group_constraints_duk, 
        &mut self.group_constraints_con];
        // TODO: [FIX/OPTIMIZE] change card_num_range to exclude those with full inferred + dead card counts known
        //      - Not having this leaves empty group_constraints, which may or may not be a good thing
        let card_num_range = (0..5).filter(|x| *x != card as usize);
        let mut dead_card_counts: [u8; 5] = [0; 5];
        for card in self.public_constraints[player_id].iter() {
            dead_card_counts[*card as usize] += 1;
        }
        let mut inferred_card_counts: [u8; 5] = [0; 5];
        for card in self.inferred_constraints[player_id].iter() {
            inferred_card_counts[*card as usize] += 1;
        }
        let dead_card_count = self.public_constraints[player_id].len();
        let inferred_card_count = self.inferred_constraints[player_id].len();
        if player_id != 6 {

            if dead_card_count + inferred_card_count == 2 {
                // Case A: 1 inferred other card dead => update group
                // Case B: all cards inferred or known => update group
                // Handle groups where cards are same as inferred card
                if !bool_all_cards_dead_or_known {
                    let groups = &mut group_constraints[card as usize];
                    let mut i: usize = 0;
                    while i < groups.len() {
                        let group = &mut groups[i];
                        if group.get_player_flag(player_id) {
                            group.set_player_flag(player_id, false);
                            if group.count_alive() > inferred_card_counts[card as usize] {
                                group.sub_alive_count(inferred_card_counts[card as usize]);
                                group.sub_dead_count(dead_card_counts[card as usize]);
                                group.sub_total_count(inferred_card_counts[card as usize] + dead_card_counts[card as usize]);
                                if group.is_single_player_part_list() {
                                    new_inferred.push(*group);
                                    groups.swap_remove(i);
                                    log::trace!("Group removed!");
                                    continue;
                                }
                            } else {
                                groups.swap_remove(i);
                                continue;
                            }
                        }
                        i += 1;
                    }
                }
                // Seems the same...
                // Case A: 1 inferred other card dead => update group
                // Case B: all cards inferred or known => update group
                // Handle groups where cards are not same as inferred card
                for card_num in card_num_range {
                    let groups = &mut group_constraints[card_num];
                    let mut i: usize = 0;
                    while i < groups.len() {
                        let group = &mut groups[i];
                        if group.get_player_flag(player_id) {
                            group.set_player_flag(player_id, false);
                            if group.count_alive() > inferred_card_counts[card_num as usize] {
                                group.sub_alive_count(inferred_card_counts[card_num as usize]);
                                group.sub_dead_count(dead_card_counts[card_num]);
                                group.sub_total_count(inferred_card_counts[card_num as usize] + dead_card_counts[card_num]);
                                if group.is_single_player_part_list() {
                                    new_inferred.push(*group);
                                    groups.swap_remove(i);
                                    log::trace!("Group removed!");
                                    continue;
                                }
                            } else {
                                groups.swap_remove(i);
                                continue;
                            }
                        }
                        i += 1;
                    }
                }
            } else {
                if inferred_card_count != 1 {
                    println!("{:?}", self.public_constraints);
                    println!("{:?}", self.inferred_constraints);
                    panic!();
                }
                debug_assert!(inferred_card_count == 1, "In the case where you have added a card, and dead_card + inferred_cards != 2, it should only be when inferred_cards == 1");
                debug_assert!(self.inferred_constraints[player_id].first() == Some(&card), "Since bool_changes == true, you must have added Card == card in");
                // Case C: 1 inferred no other card known => update group
                // It should also only
                if !bool_all_cards_dead_or_known {
                    let groups = &mut group_constraints[card as usize];
                    let mut i: usize = 0;
                    while i < groups.len() {
                        let group = &mut groups[i];
                        if group.get_player_flag(player_id)  && group.count_alive() == 1{
                            groups.swap_remove(i);
                            continue;
                        }
                        i += 1;
                    }
                }
            }    
        } else {
            if inferred_card_count == 3 {
                // Case B: all cards inferred or known => update group
                // Handle groups where cards are same as inferred card
                if !bool_all_cards_dead_or_known {
                    let groups = &mut group_constraints[card as usize];
                    let mut i: usize = 0;
                    while i < groups.len() {
                        let group = &mut groups[i];
                        if group.get_player_flag(player_id) {
                            group.set_player_flag(player_id, false);
                            if group.count_alive() > inferred_card_counts[card as usize] {
                                group.sub_alive_count(inferred_card_counts[card as usize]);
                                group.sub_total_count(inferred_card_counts[card as usize]);
                                if group.is_single_player_part_list() {
                                    new_inferred.push(*group);
                                    groups.swap_remove(i);
                                    log::trace!("Group removed!");
                                    continue;
                                }
                            } else {
                                groups.swap_remove(i);
                                continue;
                            }
                        }
                        i += 1;
                    }
                }
                // Case B: all cards inferred or known => update group
                // Handle groups where cards are not same as inferred card
                for card_num in card_num_range {
                    let groups = &mut group_constraints[card_num];
                    let mut i: usize = 0;
                    while i < groups.len() {
                        let group = &mut groups[i];
                        if group.get_player_flag(player_id) {
                            group.set_player_flag(player_id, false);
                            if group.count_alive() > inferred_card_counts[card_num as usize] {
                                group.sub_alive_count(inferred_card_counts[card_num as usize]);
                                group.sub_total_count(inferred_card_counts[card_num as usize]);
                                if group.is_single_player_part_list() {
                                    new_inferred.push(*group);
                                    groups.swap_remove(i);
                                    log::trace!("Group removed!");
                                    continue;
                                }
                            } else {
                                groups.swap_remove(i);
                                continue;
                            }
                        }
                        i += 1;
                    }
                }
            } else {
                // Case D more than 0 inferred, but not full
                if !bool_all_cards_dead_or_known {
                    let groups = &mut group_constraints[card as usize];
                    let mut i: usize = 0;
                    while i < groups.len() {
                        let group = &mut groups[i];
                        if group.get_player_flag(player_id) && group.count_alive() <= inferred_card_counts[card as usize] {
                            groups.swap_remove(i);
                            continue;
                        }
                        i += 1;
                    }
                }
                for card_num in card_num_range {
                    let groups = &mut group_constraints[card_num];
                    let mut i: usize = 0;
                    while i < groups.len() {
                        let group = &mut groups[i];
                        if group.get_player_flag(player_id) && group.count_alive() <= inferred_card_counts[card_num as usize] {
                            groups.swap_remove(i);
                            continue;
                        }
                        i += 1;
                    }
                }
            }
        }

        // TODO: Inferred card needs to prune too!
        new_inferred
    }
    // TODO: [THEORY CHECK]
    // - !!! If already inside, should not add. because the player could just be reveal info we already know
    // [THOT]: Information only gets added if the "wave function collapses"-ish, when any particular set of players' cards are fully determined
    // [THOT]: To define carefully what "fully determined" means
    // [THOT]: Like "fuzzy info" that is probabilistic only "set-resolves" if for some set, all the items are known
    // [THOT]: We are interested in the set resolving of a set of just 1 player
    // QUESTION: how does this resolving idea pair with possible card combinations a player can have?
    // [THOT]: Every group constraint defines and in group and an out group, from which we can piece together sets of information by taking the union over the whole group
    /// Does the Reveal part of RevealRedraw
    /// - Only prunes those that can be immediately found to be redundant, without comparing to other groups
    /// - Assumes player_id is alive and thus joint_constraint is empty, public_constraint may or may not be empty
    /// - Assumes no group is redundant before adding
    /// - Assumes no dead player info is in groups before adding
    /// - Leaves no group redundant after adding
    /// - Does modify a group based on inferred constraints
    /// - Leaves no dead player info in groups
    /// - New information discovery must update inferred constraints as well as all affected group_constraints!
    ///     - This new information is reflected in changed in inferred constraint, and PRUNES, not addition to groups (as new information is not a set)
    ///     - Calls a function that mines the information to generate all other inferred information, which may add groups
    /// - Does not reflect information change from swapping of cards in Ambassador and RevealRedraw
    /// 1) ADDS inferred info
    /// 2) Prunes group based on inferred info
    /// CASES:
    /// We update group_constraints where player_id flag is true only
    ///     - if its false, nothing really is touched there until mixing
    /// group constraints with group.card() == A
    ///      - Reveal A => (alive, alive) [A, B] and a group with [1 0 0 0 0 0 1] A 2 => flag = false and count_alive - 1
    ///      - Reveal A => (alive, alive) [A, A] and a group with [1 0 0 0 0 0 1] A 2 => remove
    ///      - Reveal A => (alive, alive) [A, A] and a group with [1 0 0 0 0 0 1] A 3 => flag = false and count_alive - 2
    ///      - Reveal A => (alive, alive) [A, X] and a group with [1 0 0 0 0 0 1] A 3 => no change leave untouched
    ///      - Reveal A => (alive, alive) [A, X] and a group with [1 0 0 0 0 0 1] A 2 => no change leave untouched
    ///      - Reveal A => (alive, alive) [A, X] and a group with [1 0 0 0 0 0 1] A 1 => remove
    ///      - Reveal A => (alive, alive) [A, !A] and a group with [1 0 0 0 0 0 1] A 2 => flag = false and count_alive - 1
    ///      - Reveal A => (dead, alive) [!A, A] and a group with [1 0 0 0 0 0 1] A 2 => flag = false and count_alive - 1
    ///      - Reveal A => (dead, alive) [!A, A] and a group with [1 0 0 0 0 0 1] A 1 => remove
    ///      - Reveal A => (dead, alive) [A, A] and a group with [1 0 0 0 0 0 1] A 2 => flag = false and count_alive - 1
    ///      - Reveal A => (dead, alive) [A, A] and a group with [1 0 0 0 0 0 1] A 1 => remove
    /// CONCLUSION 1: If all of the player's cards are known, since revealed card is alive, we can change flag to false and subtract the number of alive cards and dead cards
    /// CONCLUSION 2: If all flags 0 => remove | If alive_count less than equals to 0 => remove
    /// CONCLUSION 3: If not all of a player's cards are known, leave then unchanged
    /// group constraints with group.card() != A | (d0a3 -> dead 0 alive 3)
    ///      - Reveal A => (alive, alive) [C, A] and a group with [1 0 0 0 0 0 1] C d0a3 => flag = false, count_alive - 1
    ///      - Reveal A => (alive, alive) [C, A] and a group with [1 0 0 0 0 0 1] C d0a2 => flag = false, count_alive - 1
    ///      - Reveal A => (alive, alive) [C, A] and a group with [1 0 0 0 0 0 1] C d0a1 => flag = false, count_alive - 1
    ///      - Reveal A => (alive, alive) [C, A] and a group with [1 0 0 0 0 0 1] C d1a2 => flag = false, count_alive - 1
    ///      - Reveal A => (alive, alive) [C, A] and a group with [1 0 0 0 0 0 1] C d2a1 => remove
    ///      - Reveal A => (alive, alive) [A, X] and a group with [1 0 0 0 0 0 1] C d?a? => unchanged => prune cases that add info to inferred group are handled later
    ///      - Reveal A => (dead, alive) [C, A] and a group with [1 0 0 0 0 0 1] C d1a2 => flag = false, count_dead - 1
    ///      - Reveal A => (dead, alive) [C, A] and a group with [1 0 0 0 0 0 1] C d1a1 => flag = false, count_dead - 1
    /// CONCLUSION 1: If all of the player's cards are known, since revealed card is alive, we can change flag to false and subtract the number of alive cards and dead cards
    /// CONCLUSION 2: If all flags 0 => remove | If alive_count less than equals to 0 => remove
    /// CONCLUSION 3: If not all of a player's cards are known, leave then unchanged
    /// CONCLUSION 4: Groups where player_id is 0 are unaffected. They will be handled later in generate inferred constraints, and any further redundant pruning.
    /// FINAL CONCLUSION: Its the same treatment regardless of what card the group is, because we really are comparing a player's hand to the groups.
    ///                   The algo is less about a reveal and more about how to update groups after new information is added from the reveal.
    pub fn reveal(&mut self, history_index: usize, player_id: usize, card: Card) {
        // PD Obsolete
        // self.increment_move_count();
        log::trace!("In reveal");
        // if self.lookback_1(history_index) {
        //     // If true, it will have rerun the entire history including the current move
        //     return
        // }
        // TODO: Combine adjustment and addition to constraint to allow clear() like in death()
        if !self.inferred_player_constraint_contains(player_id, card) {
            // Adds information to inferred constraint if it isn't already there
            self.add_inferred_player_constraint(player_id, card);
        }
        // Commented out as it removes some group required by mut excl addition
        log::info!("After adding reveal inferred card");
        self.printlog();
        // But won't this be done in subset groups?
        // TODO: Test without reveal_group_adjustment
        self.reveal_group_adjustment(player_id, card);
        // TODO: [TEST] Can this add_inferred_groups go above?
        self.add_inferred_information();
        log::info!("After adding add_inferred_information");
        self.printlog();
        self.clear_group_constraints(card);
        // [THOT] It feels like over here when you reveal something, you lead to information discovery! 
        // [THOT] So one might be able to learn information about the hands of other players?
    }
    pub fn reveal_initial(&mut self, player_id: usize, card: Card) {
        // PD Obsolete
        // self.increment_move_count();
        log::trace!("In reveal");
        // TODO: Combine adjustment and addition to constraint to allow clear() like in death()
        if !self.inferred_player_constraint_contains(player_id, card) {
            // Adds information to inferred constraint if it isn't already there
            self.add_inferred_player_constraint(player_id, card);
        }
        if self.lookback_initial() {
            // If true, it will have rerun the entire history including the current move
            return
        }
        // Commented out as it removes some group required by mut excl addition
        log::info!("After adding reveal inferred card");
        self.printlog();
        // But won't this be done in subset groups?
        // TODO: Test without reveal_group_adjustment
        self.reveal_group_adjustment(player_id, card);
        // TODO: [TEST] Can this add_inferred_groups go above?
        self.add_inferred_information();
        self.clear_group_constraints(card);
        // [THOT] It feels like over here when you reveal something, you lead to information discovery! 
        // [THOT] So one might be able to learn information about the hands of other players?
    }
    // TODO: Review the purpose of this... should I match the amb case?
    /// Updates groups affected by revealing of information in reveal, removing groups and adding subset groups
    ///     - [A] [1 0 0 0 1 0 0] 1 Captain, player 0 reveals a Captain, this group is now redundant
    ///         - Remove group
    ///     - [B] [1 0 0 0 1 0 0] 1 Captain, player 0 reveals a Duke, if player_card_known == 1
    ///         - Nothing changes
    ///     - [C] [1 0 0 0 1 0 0] 2 Captain, player 0 reveals a Captain, player 0 inferred: [Duke, Captain]
    ///         - Remove player flag as player cant have Captain
    ///         - Dont remove group me thinks, so u dont recreate it in ME Union
    ///         - Effectively create a subset group
    ///     - [C] [1 0 0 0 1 0 0] 2 Captain, player 0 reveals a Captain, player 0 inferred: [Captain], player 0 single_flag: 1 => keep at 1
    ///         - Do nothing
    ///     - [C] [1 0 0 0 1 0 0] 1 Captain, player 0 reveals a Captain, player 0 inferred: [Captain], player 0 single_flag: 1 => keep at 1
    ///         - Remove group
    ///     - [D] [1 0 0 0 1 0 0] 2 Captain, player 0 reveals a Duke, player 0 inferred: [Duke, Captain]
    ///         - Remove player flag as player cant have Captain
    ///         - Dont remove group me thinks, so u dont recreate it in ME Union
    ///         - Effectively create a subset group
    ///     - [C] [1 0 0 0 1 0 0] 2 Captain, player 0 reveals a Duke, player 0 inferred: [Duke, Contessa]
    ///         - Remove player flag as player cant have Captain
    ///     - [D] [1 0 0 0 1 0 0] 2 Duke, player 0 reveals a Duke, player 0 inferred: [Duke, Contessa]
    ///         a) Add another group with 1 Duke and remove player 0, or
    ///         b) Remove player 0 and subtract alive and dead duke
    ///     - [E] If only 1 of a player's card is known, we do not adjust group as we cannot minimie it
    /// SPECIFICS:
    /// - See documentation in reveal
    /// NOTE:
    /// - Assumes there may be groups that became redundant after information is revealed
    /// - Only modifies and removes groups that are affected or have become redundant after reveal
    /// - May leave groups that are redundant when compared with other groups
    /// 
    /// Assumes:
    /// - In the duplicate redundancy case, this does not change the state of internal redundancy
    ///     - If groups are not internally redundant, it leaves group not internally redundant
    ///     - If groups are internally redundant, it leaves group internally redundant
    pub fn reveal_group_adjustment(&mut self, player_id: usize, card: Card) {
        log::trace!("In reveal_group_adjustment");
        let player_alive_card_count: [u8; 5] = self.player_inferred_card_counts(player_id);
        let player_dead_card_count: [u8; 5] = self.player_dead_card_counts(player_id);
        log::trace!("player_alive_card_count: {:?}", player_alive_card_count);
        log::trace!("player_dead_card_count: {:?}", player_dead_card_count);
        let player_cards_known = self.player_cards_known(player_id);
        // Won't this remove many groups that will need to be readded by mutually exclusive additions...
        // Removes groups that are now redundant
        log::trace!("Player_cards_known: {}", player_cards_known);
        if player_cards_known != 2 { 
            for (card_num, group_constraints) in [&mut self.group_constraints_amb, &mut self.group_constraints_ass, &mut self.group_constraints_cap, &mut self.group_constraints_duk, &mut self.group_constraints_con].iter_mut().enumerate() {
                let mut i: usize = 0;
                while i < group_constraints.len() {
                    let group: &mut CompressedGroupConstraint = &mut group_constraints[i];
                    // Update only groups affected by the revealed information => i.e. those with player_id flag as true
                    if group.get_player_flag(player_id) && group.count_alive() > 0 {
                        // NOTE: We only have 3 dead 0 alive groups to facilitate impossible cards and no other 0 alive groups
                        // player 1 pile 0
                        // NOTE: This does not prunes [3 dead 0 alive] because a player cant reveal and alive card if all 3 cards are dead
                        if group.count_alive() <= player_alive_card_count[card_num] {
                            // [PLAYER ONLY PRUNE] knowing the player had the card now makes this group obsolete | all possible alive cards in the group are with the player
                            // No need to modify this as the information from the player's pile swap gets added at the end
                            log::trace!("=== Reveal Group Adjustment remove redundant group === ");
                            log::trace!("Removing Group: {}", group);
                            log::trace!("Reason: group.count_alive() <= player_alive_card_count[card]: {}", player_alive_card_count[card_num]);
                            group_constraints.swap_remove(i);
                            continue;
                        } 
                        // TODO: [OPTIMIZE] Consider adding subset & inferred groups here to not run it outside, if all subsets added, only need to ME Union
                    }
                    i += 1;
                }
            }
        } else {
            for (card_num, group_constraints) in [&mut self.group_constraints_amb, &mut self.group_constraints_ass, &mut self.group_constraints_cap, &mut self.group_constraints_duk, &mut self.group_constraints_con].iter_mut().enumerate() {
                let mut groups_to_add: Vec<CompressedGroupConstraint> = Vec::with_capacity(5);
                let mut i: usize = 0;
                while i < group_constraints.len() {
                    let group: &mut CompressedGroupConstraint = &mut group_constraints[i];
                    // Update only groups affected by the revealed information => i.e. those with player_id flag as true
                    if group.get_player_flag(player_id) && group.count_alive() > 0 {
                        // NOTE: We only have 3 dead 0 alive groups to facilitate impossible cards and no other 0 alive groups
                        // player 1 pile 0
                        // NOTE: This does not prunes [3 dead 0 alive] because a player cant reveal and alive card if all 3 cards are dead
                        if group.count_alive() <= player_alive_card_count[card_num] {
                            // [PLAYER ONLY PRUNE] knowing the player had the card now makes this group obsolete | all possible alive cards in the group are with the player
                            // No need to modify this as the information from the player's pile swap gets added at the end
                            log::trace!("=== Reveal Group Adjustment remove redundant group === ");
                            log::trace!("Removing Group: {}", group);
                            log::trace!("Reason: group.count_alive() <= player_alive_card_count[card]: {}", player_alive_card_count[card_num]);
                            group_constraints.swap_remove(i);
                            continue;
                        } 
                        // This really is just creating a subset group
                        // if we know both of a player's cards including the revealed card (player has at least 1 alive cos reveal)
                        // Adjust the group to remove the player's flag
                        log::trace!("=== Reveal Group Adjustment player_cards_known == 2 === ");
                        log::trace!("Original Group: {}", group);
                        log::trace!("Player {player_id} public_constraints: {:?}, inferred_constraints: {:?}", self.public_constraints[player_id], self.inferred_constraints[player_id]);
                        let mut readd_group = group.clone();
                        readd_group.set_player_flag(player_id, false);
                        // Indicate that only 1 of the players' card was revealed, and used in the redraw
                        readd_group.set_single_card_flag(player_id, false);
                        if readd_group.none_in() {
                                log::trace!("removing empty group: {}", group);
                                group_constraints.swap_remove(i);
                            continue;
                        }
                        readd_group.count_alive_subtract(player_alive_card_count[card_num]);
                        readd_group.count_dead_subtract(player_dead_card_count[card_num]);
                        log::trace!("Group adjusted for re-adding to: {}", readd_group);
                        // repushing 
                        //      - only works for duplicate redundancy, not subset redundancy 
                        //      - as order is not preserved in subset redundancy as it can remove groups currently in group_constraints
                        if readd_group.get_total_count() != 0 {
                            groups_to_add.push(readd_group);
                        }
                    }
                    i += 1;
                }
                // Add to groups
                for group in groups_to_add {
                    log::trace!("Trying to Add group: {}", group);
                    Self::non_redundant_push(group_constraints, group);
                }
            }
        }
        log::info!("=== After Reveal Group Adjustment ===");
        self.printlog();
    }
    // TODO: [THEORY CHECK]
    // TODO: [TEST] 
    // TODO: [TODO] separating the dilution of information by creating 2 functions, both call the same mixing, but call different dilution steps
    // e.g. reveal_redraw => that called redraw() then dilutes information
    // TODO: [MOVE DOCUMENTATION] to docstring 
    // TODO: Rename ot mix
    /// Mixes 1 card
    /// Consists of 2 steps:
    /// - Updating current groups with information inferred from the mixing player and the pile
    /// - Dissipating information from the inferred constraints, and adding new groups to track how the cards have spread
    /// - Removing redundant groups
    /// 
    /// Param details:
    /// - Reveal_card == Some(card) for RevealRedraw 
    /// - Reveal_card == None for Ambassador 
    /// Assumptions:
    /// 
    /// - Assumes all possibly inferred information is fully reflected and stored in the inferred constraint
    /// - Mixing adds the inferred information into groups that are affected by the mix
    /// - Mixing adds new groups to show how cards only the pile has or only the player has are now possibly with either of them (player union pile)
    /// - [Handled elsewhere] Information is "diluted" or "dissipated", since there is a reduction in absolutely known information about a particular player's state
    /// - [Handled elsewhere] This should ideally still reflect all possibly inferred information!
    pub fn mix(&mut self, player_id: usize) {
        // Now I could selectively check if there are changes, and choose to redundant prune only sometimes
        // But odds are, there is some set where player flag is 0 so we will need to do it anyways
        debug_assert!(self.player_is_alive(player_id), "Dead player cant do things man");
        debug_assert!(player_id != 6, "Player_id here cannot be pile!");
        // [MIXING] groups so a union between player and pile is formed
        // IDEA: So i guess updating would be taking missing information known from pile or player and adding it in? pile info wont be loss, player info wont be loss, pile & player info will be added in later 
        
        // [MIXING] Here we add information to current groups that gain it from the mix e.g. groups where player is 0 and pile is 1 or vice versa
        log::trace!("In mix");
        for (card_num, group_constraints) in [&mut self.group_constraints_amb, &mut self.group_constraints_ass, &mut self.group_constraints_cap, &mut self.group_constraints_duk, &mut self.group_constraints_con].iter_mut().enumerate() {
            let player_inferred_count = self.inferred_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
            let player_dead_count = self.public_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
            let pile_inferred_count = self.inferred_constraints[6].iter().filter(|c| **c as usize == card_num).count() as u8;
            let mut i: usize = 0;
            while i < group_constraints.len() {
                let group: &mut CompressedGroupConstraint = &mut group_constraints[i];
                // consider 2 dimensions, player_flag and pile_flag 0 1, 1 0, 1 1? no 0 0
                if !group.get_player_flag(player_id) {
                    if group.get_player_flag(6) {
                        // Here player is 0 and pile is 1
                        // We add player information that it is originally missing
                        group.set_player_flag(player_id, true);
                        // Since both parts now participated in the mix, we can erase the 1 card participation status for player
                        group.set_single_card_flag(player_id, false);
                        group.add_dead_count(player_dead_count);
                        // TODO: put debug_assert somewhere sensible
                        // debug_assert!(single_count + joint_count + self.dead_card_count()[group_card as usize] < 3, "???");
                        // FIX: min 1 because only 1 card is exchanged
                        group.count_alive_add(player_inferred_count);
                    }
                } else {
                    if !group.get_player_flag(6) {
                        // Here player is 1 and pile is 0
                        // We add pile information that it is originally missing
                        group.set_player_flag(6, true);
                        group.set_single_card_flag(player_id, false);
                        // group.count_alive_add(self.inferred_pile_constraints[card_num]); 
                        group.count_alive_add(pile_inferred_count); 
                    } else {
                        group.set_single_card_flag(player_id, false);
                        // Here player is 1 and pile is 1, we do a simple check
                        // If somehow you have learnt of more inferred information, than prune the group
                        // FIX: we do nth here, handle elsewhere
                        // if player_alive_card_count[card_num] > group.count_alive() {
                        //     group_constraints.swap_remove(i);
                        //     continue;
                        // }
                        // TODO [OPTIMIZE REMOVAL]: Case [1 0 0 0 0 0 1] 2 Captain, player 1 has 1 dead duke
                        // inferred_constraint for pile => has 1 Captain no need to remove this captain
                    }
                }
                i += 1;
            }
        }
        // OR operation on the false booleans of impossible constraints => AND operation on true
        // When player and pile mix if the other party's can have impossible card, it becomes possible for the referenced party
        // for (a, b) i
        for i in 0..5 {
            self.impossible_constraints[player_id][i] &= self.impossible_constraints[6][i];
            self.impossible_constraints[6][i] = self.impossible_constraints[player_id][i];
        }
    }
    // TODO: Check if same issue is present as with ambassador, see removal iteration implementation in ambassador
    /// Removes current inferred constraints and adds them to group based on mix
    /// RevealRedraw dilution of inferred information
    /// Adjust inferred constraints
    /// NOTE:
    /// - Information is "diluted" or "dissipated", since there is a reduction in absolutely known information about a particular player's state
    /// - This should ideally still reflect all possibly inferred information!
    /// - May leave redundant information in the collective constraint
    /// 
    /// Assumes:
    /// - Groups not internally redundant
    /// Inferred knowledge cases [REVEALREDRAW] [ALL CARDS WITH PILE + PLAYER]
    /// These arent just cases for how to represent the new group constraint
    /// These also represent what we can infer, if its pile >= 1 we have a 1 inferred card of the pile
    /// representing the inferred knowledge ()
    /// player (dead, alive) = (A, A) Pile (A, X, X)  => Reveal A=> Pile has >= 1 A
    /// player (dead, alive) = (A, X) Pile (A, X, X)  => Reveal A=> Pile has >= 0 A (No inferred info for pile)
    /// player (dead, alive) = (A, X) Pile (X, X, X)  => Reveal A=> Pile has >= 0 A (No inferred info for pile)
    /// player (dead, alive) = (!A, X) Pile (A, A, A) => Reveal A => Pile has >= 2 A
    /// player (dead, alive) = (!A, X) Pile (A, A, X) => Reveal A => Pile has >= 1 A
    /// player (dead, alive) = (!A, X) Pile (A, X, X) => Reveal A => Pile has >= 0 A (No inferred info for pile)
    /// player (dead, alive) = (!A, A) Pile (A, A, X) => Reveal A => Pile has >= 2 A
    /// player (dead, alive) = (!A, A) Pile (A, X, X) => Reveal A => Pile has >= 1 A
    /// player (dead, alive) = (!A, A) Pile (X, X, X) => Reveal A => Pile has >= 0 A (No inferred info for pile)
    /// player (alive, alive) = (A, A) Pile (A, X, X) => Reveal A => Pile has >= 1 A, player inferred A -1
    /// player (alive, alive) = (A, A) Pile (X, X, X) => Reveal A => Pile has >= 0 A (No inferred info for pile)
    /// player (alive, alive) = (X, A) Pile (A, A, X) => Reveal A => Pile has >= 2 A
    /// player (alive, alive) = (X, A) Pile (A, X, X) => Reveal A => Pile has >= 1 A
    /// player (alive, alive) = (X, A) Pile (X, X, X) => Reveal A => Pile has >= 0 A (No inferred info for pile)
    /// player (alive, alive) = (X, A) Pile (A, A, X) => Reveal !A => Pile has >= 1 A
    /// player (alive, alive) = (X, A) Pile (A, X, X) => Reveal !A => Pile has >= 0 A (No inferred info for pile)
    /// player (alive, alive) = (X, A) Pile (X, X, X) => Reveal !A => Pile has >= 0 X (No inferred info for pile)
    /// player (alive, alive) = (X, X) Pile (A, A, A) => Reveal !A => Pile has >= 2 A 
    /// player (alive, alive) = (X, X) Pile (A, A, X) => Reveal !A => Pile has >= 1 A 
    /// player (alive, alive) = (X, X) Pile (A, X, X) => Reveal !A => Pile has >= 0 X (No inferred info for pile)
    /// player (alive, alive) = (A, !A) Pile (A, A, X) => Pile has >= 1 A
    /// player (alive, alive) = (!A, !A) Pile (A, A, A) => Pile has >= 2 A
    /// player (alive, alive) = (B, A) Pile (A, A, X) => Pile has >= 2 A
    /// DEFINITION 0: When we say pile/player inferred - 1, we mean that the number of inferred card X decreases by 1
    /// CONCLUSION 0: In all cases, if player reveal A, player inferred A - 1, pile inferred A remains constant
    /// CONCLUSION 1: In all cases, if player reveal !A, player inferred !A - 1, pile inferred A - 1,
    /// COLLORARY 1: In all cases if player reveals some card A, player inferred A - 1,for all cards !A pile number of inferred !A - 1
    /// COLLORARY 1b: If player reveals some card A, player inferred A - 1, pile inferred A remains constant,for all cards !A pile number of inferred !A - 1
    /// CONCLUSION 2: (dead, alive) reveal A inferred from pile remains same for A,... other cards?
    /// CONCLUSION 3: (dead, alive) reveal !A inferred from pile remove one A,... other cards?
    /// CONCLUSION 4: There is kind of a symmetry here, Reveal !A basically tells us what to do with other cards in group that arent revealed
    /// I think COLLORARY 1b forms the entire rule set.
    /// QUESTION: Following COLLORARY 1b, how should dropped inferences be converted to group constraints?
    /// I think it should be the original inferred for both, but the group of both would contain all the inferred counts from both players for a particular card
    /// TODO: [THINK]
    /// QUESTION: How about if we know both of them have a some number of As, but not specifically who?
    /// I guess for reveal_redraw, this should be handled in reveal, for (dead, alive) the union will collapse to be only ambassador, or clearly with player
    /// For (alive, alive)?
    /// Adding all new group to dissipate known information about player_id and pile
    pub fn redraw(&mut self, player_id: usize, card: Card) {
        // [DILUTING INFERRED INFORMATION] Mixing causes the card the player revealed to be dissipated and shared between player and hte pile
        // Here we Manage the dissipation of inferred information by:
        // - Properly subtracting the appropriate amount from inferred pile constraint
        // - Adding the information into the group constraints => on how the known cards have "spread" from player or pile or BOTH (player union pile) 
        log::trace!("In redraw");
        let mut card_counts: [u8; 5] = self.get_inferred_card_counts(6);
        card_counts[card as usize] += 1;
        let dead_counts: [u8; 5] = self.get_public_card_counts(player_id);
        // only subtract 1 card here as only 1 is revealed and moved out of player's hand 
        // TODO: [CHANGE] COLLORARY 1b, Adding of group constraints should be for all inferred cards in the player union pile
        // COLLORARY 1b: If player reveals some card A, player inferred A - 1, pile inferred A remains constant,for all cards !A pile number of inferred !A - 1
        // TODO: [PROBLEM / THINK] SHLDNT This section be below the group adjustments... else it will override
        //                      - How does this interact with player_dead_card_count below?
        let mut group_constraints = [&mut self.group_constraints_amb, 
                                                                            &mut self.group_constraints_ass, 
                                                                            &mut self.group_constraints_cap, 
                                                                            &mut self.group_constraints_duk, 
                                                                            &mut self.group_constraints_con];

        log::trace!("=== Start redraw inferred adjustment ===");
        log::info!("{}", format!("Public Constraints: {:?}", self.public_constraints));
        log::info!("{}", format!("Inferred Constraints: {:?}", self.inferred_constraints));
        log::info!("{}", format!("Group Constraints:"));
        log::info!("{}", format!("\t AMB: {:?}", group_constraints[0]));
        log::info!("{}", format!("\t ASS: {:?}", group_constraints[1]));
        log::info!("{}", format!("\t CAP: {:?}", group_constraints[2]));
        log::info!("{}", format!("\t DUK: {:?}", group_constraints[3]));
        log::info!("{}", format!("\t CON: {:?}", group_constraints[4]));
        // Adjust groups
        // CASE revealed card
        // player true pile false => set pile to true 
        // player false pile true => set player to true, single_flags true , add dead_cards too
        // player false pile false => no change 
        // player true pile true => if 1 alive, set single_flags true, if 2 alive u can't simply set single_flags to true
        // CASE other card
        // player true pile false => no change as it was not the card that move with the pile 
        // player false pile true => set player to true , add dead_cards
        // player false pile false => no change 
        // player true pile true => set single_flags false? 
        // TODO: Maybe we consider how this interacts with reveal adjustment
        for (card_num, card_group_constraints) in group_constraints.iter_mut().enumerate() {
            let player_dead_card_count = self.public_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
            // TODO: [PROBLEM] Determine if alive_counts here should use the pre or post mix adjusted inferred_card_counts
            // Consider using the alive_count before subtraction, and consider using player + pile alive count, and use max of that or current group alive count
            // Alive here applies in the player false pile true case, so should it include the revealed card?
            //          Should it apply the counts before subtraction? I think yes?
            // Like if pile + player had more alive, then should use that amount!
            let player_alive_card_count = self.inferred_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
            // let mut add_groups: Vec<CompressedGroupConstraint> = Vec::with_capacity(card_group_constraints.len());
            if card_num == card as usize {
                let mut i: usize = 0;
                while i < card_group_constraints.len() {
                    log::trace!("Redraw Group Adjustment considering group (A): {}", card_group_constraints[i]);
                    if card_group_constraints[i].get_player_flag(player_id) {
                        if !card_group_constraints[i].get_player_flag(6) {
                            // let mut readd_group = card_group_constraints[i].clone();
                            card_group_constraints[i].set_player_flag(6, true);
                            // card_group_constraints.swap_remove(i);
                            // let (_, modified) = Self::non_redundant_push_tracked(card_group_constraints, readd_group);
                            // if modified {
                            //     continue;
                            // }
                            log::trace!("Redraw Group Adjustment group changed (a) to: {}", card_group_constraints[i]);
                        } else {
                            if card_group_constraints[i].count_alive() == 1 {
                                // let mut readd_group = card_group_constraints[i].clone();
                                // Lets try this
                                card_group_constraints[i].set_single_card_flag(player_id, true);
                                // card_group_constraints.swap_remove(i);
                                // let (_, modified) = Self::non_redundant_push_tracked(card_group_constraints, readd_group);
                                // if modified {
                                //     continue;
                                // }
                                log::trace!("Redraw Group Adjustment group changed (b) to: {}", card_group_constraints[i]);
                            }
                        }
                    } else {
                        if card_group_constraints[i].get_player_flag(6) {
                            // let mut readd_group = card_group_constraints[i].clone();
                            card_group_constraints[i].set_player_flag(player_id, true);
                            // Indicate that only 1 of the players' card was revealed, and used in the redraw
                            card_group_constraints[i].set_single_card_flag(player_id, true);
                            card_group_constraints[i].add_dead_count(player_dead_card_count);
                            card_group_constraints[i].add_alive_count(player_alive_card_count);
                            card_group_constraints[i].add_total_count(player_dead_card_count + player_alive_card_count);
                            // card_group_constraints.swap_remove(i);
                            // let (_, modified) = Self::non_redundant_push_tracked(card_group_constraints, readd_group);
                            // if modified {
                            //     continue;
                            // }
                            log::trace!("Redraw Group Adjustment group changed (c) to: {}", card_group_constraints[i]);
                        }
                    }
                    i += 1;
                }
            } else {
                let mut i: usize = 0;
                while i < card_group_constraints.len() {
                    log::trace!("Redraw Group Adjustment considering group (B): {}", card_group_constraints[i]);
                    if !card_group_constraints[i].get_player_flag(player_id) {
                        if card_group_constraints[i].get_player_flag(6) {
                            // let mut readd_group = card_group_constraints[i].clone();
                            card_group_constraints[i].set_player_flag(player_id, true);
                            // Indicate that only 1 of the players' card was revealed, and used in the redraw
                            card_group_constraints[i].add_dead_count(player_dead_card_count);
                            card_group_constraints[i].set_single_card_flag(player_id, true);
                            // NOTE: We do not add_alive count if we set single_card_flag == true 
                            //       as the inferred alive count is not part of single flag
                            card_group_constraints[i].add_total_count(player_dead_card_count);
                            // card_group_constraints.swap_remove(i);
                            // let (_, modified) = Self::non_redundant_push_tracked(card_group_constraints, readd_group);
                            // if modified {
                            //     continue;
                            // }
                            log::trace!("Redraw Group Adjustment group (B) changed (a) to: {}", card_group_constraints[i]);
                        }
                    } else {
                        if card_group_constraints[i].get_player_flag(6) {
                            // let mut readd_group = card_group_constraints[i].clone();
                            // // Let's try this
                            card_group_constraints[i].set_single_card_flag(player_id, false);
                            // card_group_constraints.swap_remove(i);
                            // Self::non_redundant_push(card_group_constraints, readd_group);
                            // continue;
                            log::trace!("Redraw Group Adjustment group (B) changed (b) to: {}", card_group_constraints[i]);
                        }
                    }
                    i += 1;
                }
            }
        }
        log::trace!("=== After redraw group adjustment ===");
        log::info!("{}", format!("Public Constraints: {:?}", self.public_constraints));
        log::info!("{}", format!("Inferred Constraints: {:?}", self.inferred_constraints));
        log::info!("{}", format!("Group Constraints:"));
        log::info!("{}", format!("\t AMB: {:?}", group_constraints[0]));
        log::info!("{}", format!("\t ASS: {:?}", group_constraints[1]));
        log::info!("{}", format!("\t CAP: {:?}", group_constraints[2]));
        log::info!("{}", format!("\t DUK: {:?}", group_constraints[3]));
        log::info!("{}", format!("\t CON: {:?}", group_constraints[4]));
        // TODO: See if I need to add the player + pile 1 group? Is it in the whole flow?
        // player inferred A - 1
        if let Some(pos) = self.inferred_constraints[player_id].iter().position(|c| *c == card) {
            self.inferred_constraints[player_id].swap_remove(pos);
            // PD Obsolete
            // self.inferred_card_count[card as usize] -= 1;
        }
        for inferred_card in [Card::Ambassador, Card::Assassin, Card::Captain, Card::Duke, Card::Contessa] {
            // Removing 1 of each non-inferred cards from pile inferred_constraints
            if inferred_card != card {
                // Dissipating Information from pile
                // pile number inferred - 1
                // self.subtract_inferred_pile_constraint(card);
                if let Some(pos) = self.inferred_constraints[6].iter().position(|&c| c == inferred_card) {
                    self.inferred_constraints[6].swap_remove(pos);
                }
            }
            if card_counts[inferred_card as usize] > 0 {
                // Adding Dissipated information to groups appropriately
                // TODO: Add method to add groups only if it is not already inside
                let group = CompressedGroupConstraint::new_with_pile(player_id, inferred_card, dead_counts[inferred_card as usize], card_counts[inferred_card as usize]);
                log::trace!("");
                log::trace!("=== dilution_reveal dissipated information");
                log::trace!("added group: {}", group);
                // self.add_group_constraint(group);
                Self::non_redundant_push(group_constraints[inferred_card as usize], group);
            }
        }
        log::trace!("=== End Redraw Inferred Subtractions ===");
        self.printlog();
        // let player_count_dead = self.public_constraints[player_id].iter().filter(|c| **c == card).count() as u8;
        // let player_pile_reveal_card_group = CompressedGroupConstraint::new_with_pile(player_id, card, player_count_dead, 1);
        // Self::non_redundant_push(group_constraints[card as usize], player_pile_reveal_card_group);
        self.group_redundant_prune();
        log::trace!("=== End Redraw ===");
        self.printlog();
    }
    // TODO: Review group_constraint addition method
    /// Ambassador Dilution of inferred knowledge
    /// Adjust inferred knowledge and avoids having to run subset groups again
    /// NOTE:
    /// - Information is "diluted" or "dissipated", since there is a reduction in absolutely known information about a particular player's state
    /// - This should ideally still reflect all possibly inferred information!
    /// - May leave redundant information in the collective constraint
    /// No group to add
    /// Subtraction needs to be done cautiously, might be the same as mix in all its edge cases
    /// CASE: (dead, alive) (A, A) Pile: (A, X, X) => pile will have >= 1 A
    /// CASE: (dead, alive) (A, X) Pile: (A, A, X) => pile will have >= 1 A
    /// CASE: (dead, alive) (A, X) Pile: (A, X, X) => pile will have >= 0 A
    /// CASE: (dead, alive) (A, X) Pile: (X, X, X) => pile will have >= 0 A
    /// CASE: (dead, alive) (X, X) Pile: (A, A, A) => pile will have >= 2 A
    /// CASE: (dead, alive) (X, X) Pile: (A, A, X) => pile will have >= 1 A
    /// CASE: (dead, alive) (X, X) Pile: (A, X, X) => pile will have >= 0 A
    /// CASE: (alive, alive) (A, A) Pile: (A, X, X) => pile will have >= 1 A
    /// CASE: (alive, alive) (A, X) Pile: (A, A, X) => pile will have >= 1 A
    /// CASE: (alive, alive) (A, X) Pile: (A, X, X) => pile will have >= 0 A
    /// CASE: (alive, alive) (A, X) Pile: (X, X, X) => pile will have >= 0 A
    /// CASE: (alive, alive) (X, X) Pile: (A, A, A) => pile will have >= 1 A
    /// CASE: (alive, alive) (X, X) Pile: (A, A, X) => pile will have >= 0 A
    /// CASE: (alive, alive) (X, X) Pile: (A, X, X) => pile will have >= 0 A
    /// ADDITION: Same applies but if player || pile has that amount of alive cards, so total is not just found from inferred, but also from group_constraints
    /// CONCLUSION: Inferred pile constraint for some card A will be total circulating A - no_alive cards?
    /// TODO: might need to consider the group constraints? if they add to the total circulating?
    /// TODO: [THEORY CHECK]
    /// CASE: (alive, alive) (X, X) Pile: (A, X, X) but we know the union of both have 3 As so total circulating has to include this!
    pub fn ambassador_inferred_adjustment(&mut self, player_id: usize) {
        // [DILUTING INFERRED INFORMATION] Mixing causes the inferred constraints to be dissipated from knowing a particular player has a card
        //                                  to knowing some groups of players have a card
        // Here we Manage the dissipation of inferred information by:
        // - Properly subtracting the appropriate amount from inferred pile constraint
        // - Adding the information into the group constraints => on how the known cards have "spread" from player or pile or BOTH (player union pile) 
        // [COMBINE SJ]
        let player_lives: u8 = self.player_lives(player_id);
        let total_circulating_card_counts: [u8; 5] = self.total_known_alive_with_player_and_pile(player_id);
        for inferred_card in [Card::Ambassador, Card::Assassin, Card::Captain, Card::Duke, Card::Contessa] {
            // TODO: [CHANGE] Adding of group constraints should be for all inferred cards in the player union pile + dead cards
            // Reducing by player_lives, as that is the max one can take from the pile
            // let total_remaining = total_circulating_card_counts[inferred_card as usize] - player_lives;
            // total_removal should be total_current_inferred - total_remaining
            // Adding then subtracting to prevent overflows
            log::trace!("=== Ambassador_inferred_adjustment ===");
            log::trace!("Considering player: {}", player_id);
            log::trace!("counts: {} + player_lives: {} - circulating_counts: {}", self.inferred_constraints[player_id].iter().filter(|c| **c == inferred_card).count(), player_lives, total_circulating_card_counts[inferred_card as usize]);
            // TODO: [REMOVE] This is here to replicate a Integer subtraction with overflow error.
            assert!(self.inferred_constraints[6].iter().filter(|c| **c == inferred_card).count() as u8 + player_lives >= total_circulating_card_counts[inferred_card as usize], "Found your error");
            // Removing from pile
            let total_removal = self.inferred_constraints[6].iter().filter(|c| **c == inferred_card).count() as u8 + player_lives - total_circulating_card_counts[inferred_card as usize];
            for _ in 0..total_removal {
                if let Some(pos) = self.inferred_constraints[6].iter().position(|c| *c == inferred_card) {
                    self.inferred_constraints[6].swap_remove(pos);
                }
            }
            // Add group constraints
            let dead_cards_count = self.player_dead_card_count(player_id, inferred_card);
            // TODO: Add method to add groups only if it is not already inside
            let group = CompressedGroupConstraint::new_with_pile(player_id, inferred_card, dead_cards_count, total_circulating_card_counts[inferred_card as usize]);
            self.add_group_constraint(group);
        }
    }
    /// Function to call for move RevealRedraw
    pub fn reveal_redraw(&mut self, history_index: usize, player_id: usize, card: Card) {
        // Abit dumb to seperate it like this, but if not it gets abit messy and I have more branchs :/
        
        self.reveal(history_index, player_id, card);
        // Actually shouldnt this only move the player's card in
        // mix here is not the same as ambassador, as inferred should not be touched. And since we know the revealed card
        // To rigorously show how to mix if group is not the same card, and 1 player 0 pile
        log::trace!("=== After Reveal Intermediate State ===");
        self.printlog();
        self.redraw(player_id, card);
        // PD Replace
        // self.revealed_status[player_id].push((Some(card), self.reveal_redraw_move_counter));


        // if !self.revealed_status[player_id].contains(&card) {
        //     self.revealed_status[player_id].push(card);
        // }
        // TODO: add_inferred_groups() here and test if it adds anything by panicking
        // self.add_inferred_groups();
        // self.group_redundant_prune();
        // Add the stuff here
    }
        /// Used for when move is added to history and not for recalculation
    pub fn reveal_redraw_initial(&mut self, player_id: usize, card: Card) {
        // Abit dumb to seperate it like this, but if not it gets abit messy and I have more branchs :/
        
        
        
        // self.reveal_initial(player_id, card);
        // PD Obsolete
        // self.increment_move_count();
        log::trace!("In reveal_redraw_initial");
        // TODO: Combine adjustment and addition to constraint to allow clear() like in death()
        if !self.inferred_player_constraint_contains(player_id, card) {
            // Adds information to inferred constraint if it isn't already there
            self.add_inferred_player_constraint(player_id, card);
        }
        if self.lookback_initial() {
            // If true, it will have rerun the entire history including the current move
            return
        }
        // Commented out as it removes some group required by mut excl addition
        log::info!("After adding reveal inferred card");
        self.printlog();
        // But won't this be done in subset groups?
        // TODO: Test without reveal_group_adjustment
        self.reveal_group_adjustment(player_id, card);
        // TODO: [TEST] Can this add_inferred_groups go above?
        self.add_inferred_information();
        self.clear_group_constraints(card);
        // [THOT] It feels like over here when you reveal something, you lead to information discovery! 
        // [THOT] So one might be able to learn information about the hands of other players?
        
        // Actually shouldnt this only move the player's card in
        // mix here is not the same as ambassador, as inferred should not be touched. And since we know the revealed card
        // To rigorously show how to mix if group is not the same card, and 1 player 0 pile
        log::trace!("=== After Reveal Intermediate State ===");
        self.printlog();
        self.redraw(player_id, card);
    }
    /// Function to call when the card revealed and the redrawn card is the same card
    pub fn reveal_redraw_same(&mut self, player_id: usize, card: Card) {
        // No need to lookback_1 here as it would already have been done in the normal case
        if !self.inferred_constraints[player_id].contains(&card) {
            // TODO: refactor this
            log::trace!("reveal_redraw_same player_id: {}, card: {:?}", player_id, card);
            self.add_inferred_card(player_id, card, 1, &mut vec![Vec::with_capacity(0); 6]);
        }
    }
    /// Function to call when both the card revealed and redrawn are known and not the same
    pub fn reveal_redraw_diff(&mut self, player_id: usize, reveal: Card, redraw: Card) {
        // Double inferred 
        log::trace!("reveal_redraw_diff player_id: {}, reveal: {:?}, redraw: {:?}", player_id, reveal, redraw);
        self.printlog();
        if !self.inferred_constraints[player_id].contains(&reveal) {
            self.add_inferred_card(player_id, reveal, 1, &mut vec![Vec::with_capacity(0); 6]);
        }
        if !self.inferred_constraints[6].contains(&redraw) {
            self.add_inferred_card(6, redraw, 1, &mut vec![Vec::with_capacity(0); 6]);
        }
        log::trace!("reveal_redraw_diff after adding inferred cards");
        self.printlog();
        // Swapping Cards - removing from inventories
        if let Some(pos) = self.inferred_constraints[player_id].iter().position(|c| *c == reveal) {
            self.inferred_constraints[player_id].remove(pos);
        }
        if let Some(pos) = self.inferred_constraints[6].iter().position(|c| *c == redraw) {
            self.inferred_constraints[6].remove(pos);
        }
        log::trace!("reveal_redraw_diff after removing inferred cards");
        self.printlog();
        // Swapping Cards - adjusting group_constraints
        self.swap_mix(player_id, reveal, 6, redraw);
        log::trace!("reveal_redraw_diff after swap_mix");
        self.printlog();
        // Double inferred - do not check for containment as this comes from pile not own hand
        // self.add_inferred_card_unchecked(player_id, redraw, 1, &mut vec![Vec::with_capacity(0); 6]);
        // self.add_inferred_card_unchecked(6, reveal, 1, &mut vec![Vec::with_capacity(0); 6]);
        self.swap_add_inferred_cards_unchecked(player_id, redraw, 6, reveal);
        // TODO: Determine if we actually need this
        //  How does this interact with add_inferred and swap_mix?
        log::trace!("reveal_redraw_diff after swap add_inferred_cards");
        self.printlog();
        self.group_redundant_prune();
    }
    /// Function to call when both the card revealed is left in the pile
    /// Assumes redraw is None
    /// Assumes what is revealed is relinquish and passed to the pile
    pub fn reveal_redraw_relinquish(&mut self, player_id: usize, relinquish: Card) {
        // Double inferred 
        log::trace!("reveal_redraw_relinquish player_id: {}, relinquish: {:?}", player_id, relinquish);
        self.printlog();
        if !self.inferred_constraints[player_id].contains(&relinquish) {
            self.add_inferred_card(player_id, relinquish, 1, &mut vec![Vec::with_capacity(0); 6]);
        }
        log::trace!("reveal_redraw_relinquish after adding inferred cards");
        self.printlog();
        // Relinquishing Card - removing from player inventory
        if let Some(pos) = self.inferred_constraints[player_id].iter().position(|c| *c == relinquish) {
            self.inferred_constraints[player_id].remove(pos);
        }
        log::trace!("reveal_redraw_relinquish after removing inferred cards");
        self.printlog();
        // Swapping Cards - adjusting group_constraints
        // How to mix?
        self.relinquish_mix(player_id, 6,relinquish);
        log::trace!("reveal_redraw_diff after swap_mix");
        self.printlog();
        // Double inferred - do not check for containment as this comes from pile not own hand
        self.add_inferred_card_unchecked(6, relinquish, 1, &mut vec![Vec::with_capacity(0); 6]);
        // TODO: Determine if we actually need this
        //  How does this interact with add_inferred and swap_mix?
        log::trace!("reveal_redraw_diff after swap add_inferred_cards");
        self.printlog();
        self.group_redundant_prune();
    }
    /// Mixes groups when some known cards of a player is swapped with some known cards of another player
    /// For now its a unique card per player
    /// TODO: Extend for Ambassador
    /// Assumes groups where alive_counts == 0 does not exist
    /// card_a [1 0 0 0 0 0 0] => card_a [1 0 0 0 0 0 1] (card_a moves from a to b)
    /// card_a [0 0 0 0 0 0 1] => card_a [0 0 0 0 0 0 1] (card_a does not move from b to a)
    /// card_b [1 0 0 0 0 0 0] => card_b [1 0 0 0 0 0 0] (card_b does not move from b to a)
    /// card_b [0 0 0 0 0 0 1] => card_b [1 0 0 0 0 0 0] (card_b moves from b to a)
    pub fn swap_mix(&mut self, player_a: usize, card_a: Card, player_b: usize, card_b: Card) {
        // TODO: THEORY CHECK and DOCUMENT
        // For now im thinking for player 0 and player 6
        let player_b_dead_card_a: u8 = self.public_constraints[player_b].iter().filter(|c| **c == card_a).count() as u8;
        let player_b_inferred_card_a: u8 = self.inferred_constraints[player_b].iter().filter(|c| **c == card_a).count() as u8;
        let player_a_dead_card_b: u8 = self.public_constraints[player_a].iter().filter(|c| **c == card_b).count() as u8;
        let player_a_inferred_card_b: u8 = self.inferred_constraints[player_a].iter().filter(|c| **c == card_b).count() as u8;
        log::trace!("swap_mix player_a: {player_a}, card_a: {:?}, player_b: {player_b}, card_b: {:?}", card_a, card_b);
        log::trace!("swap_mix player_b_dead_card_a: {:?}", player_b_dead_card_a);
        log::trace!("swap_mix player_b_inferred_card_a: {:?}", player_b_inferred_card_a);
        log::trace!("swap_mix player_a_dead_card_b: {:?}", player_a_dead_card_b);
        log::trace!("swap_mix player_a_inferred_card_b: {:?}", player_a_inferred_card_b);
        let group_constraints = self.group_constraints_mut();
        // remove groups that are known when cards were revealed
        if card_a == card_b {
            group_constraints[card_a as usize].retain(
                |group| {
                    // Gonna let compiler optimize this one for better documentation
                    !(
                        // Removing groups where both flags true
                        // And count <= 2
                        group.get_player_flag(player_a)
                        && group.get_player_flag(player_b)
                        && group.count_alive() < 3
                    )
                    ||
                    !(
                        // If any alive and count == 1
                        group.count_alive() < 2
                        && (
                            group.get_player_flag(player_a)
                            || group.get_player_flag(player_b)
                        )
                    )
                }
            )
        } else {
            group_constraints[card_a as usize].retain(
                |group| {
                    // Removing groups is player_a flag == true
                    // and the count_alive is 1
                    !group.get_player_flag(player_a)
                    || group.count_alive() > 1
                }
            );
            group_constraints[card_b as usize].retain(
                |group| {
                    // Removing groups is player_b flag == true
                    // and the count_alive is 1
                    !group.get_player_flag(player_b)
                    || group.count_alive() > 1
                }
            );
            // Groups certainly lost a card as the card left their group
            // and a differnt card came in 
            group_constraints[card_a as usize]
            .iter_mut()
            .filter(|group| group.get_player_flag(player_a) && !group.get_player_flag(player_b))
            .for_each(|group| {
                group.sub_alive_count(1);
                group.sub_total_count(1);
            });
            group_constraints[card_b as usize]
            .iter_mut()
            .filter(|group| group.get_player_flag(player_b) && !group.get_player_flag(player_a))
            .for_each(|group| {
                group.sub_alive_count(1);
                group.sub_total_count(1);
            });
        }
        // TODO: OPTIMIZE, actually don't need both for if player_id is 6
        // CASE:
        //  both have flags but the player has single_flag
        //  since we don't know for certain if that card that was previously given from the player is the same that returned
        //  we have to tag it as false
        group_constraints[card_b as usize]
        .iter_mut()
        .filter(|group| group.get_player_flag(player_a) && group.get_player_flag(player_b))
        .for_each(|group| {
            group.set_single_card_flag(player_a, false);
        });
        group_constraints[card_a as usize]
        .iter_mut()
        .filter(|group| group.get_player_flag(player_a) && group.get_player_flag(player_b))
        .for_each(|group| {
            group.set_single_card_flag(player_b, false);
        });
        if card_a != card_b {
            group_constraints[card_a as usize]
            .iter_mut()
            .filter(|group| !group.get_player_flag(player_a) && group.get_player_flag(player_b))
            .for_each(|group| {
                group.add_alive_count(1);
                group.add_total_count(1);
            });
            group_constraints[card_b as usize]
            .iter_mut()
            .filter(|group| !group.get_player_flag(player_b) && group.get_player_flag(player_a))
            .for_each(|group| {
                group.add_alive_count(1);
                group.add_total_count(1);
            });
            // CASE:
            //  both have flags but the player has single_flag
            //  since we don't know for certain if that card that was previously given from the player is the same that returned
            //  we have to tag it as false
            // ANOTHER CASE:
            // maybe since P3 has 1 life when they RR
            // the card moving means P3 can't have it!
            // swap_mix
        }
    }
    // pub fn swap_mix(&mut self, player_a: usize, card_a: Card, player_b: usize, card_b: Card) {
    //     // TODO: THEORY CHECK and DOCUMENT
    //     // For now im thinking for player 0 and player 6
    //     let player_b_dead_card_a: u8 = self.public_constraints[player_b].iter().filter(|c| **c == card_a).count() as u8;
    //     let player_b_inferred_card_a: u8 = self.inferred_constraints[player_b].iter().filter(|c| **c == card_a).count() as u8;
    //     let player_a_dead_card_b: u8 = self.public_constraints[player_a].iter().filter(|c| **c == card_b).count() as u8;
    //     let player_a_inferred_card_b: u8 = self.inferred_constraints[player_a].iter().filter(|c| **c == card_b).count() as u8;
    //     log::trace!("swap_mix player_a: {player_a}, card_a: {:?}, player_b: {player_b}, card_b: {:?}", card_a, card_b);
    //     log::trace!("swap_mix player_b_dead_card_a: {:?}", player_b_dead_card_a);
    //     log::trace!("swap_mix player_b_inferred_card_a: {:?}", player_b_inferred_card_a);
    //     log::trace!("swap_mix player_a_dead_card_b: {:?}", player_a_dead_card_b);
    //     log::trace!("swap_mix player_a_inferred_card_b: {:?}", player_a_inferred_card_b);
    //     let group_constraints = self.group_constraints_mut();
    //     // remove groups that are known when cards were revealed
    //     if card_a == card_b {
    //         group_constraints[card_a as usize].retain(
    //             |group| {
    //                 // Gonna let compiler optimize this one for better documentation
    //                 !(
    //                     // Removing groups where both flags true
    //                     // And count <= 2
    //                     group.get_player_flag(player_a)
    //                     && group.get_player_flag(player_b)
    //                     && group.count_alive() < 3
    //                 )
    //                 ||
    //                 !(
    //                     // If any alive and count == 1
    //                     group.count_alive() < 2
    //                     && (
    //                         group.get_player_flag(player_a)
    //                         || group.get_player_flag(player_b)
    //                     )
    //                 )
    //             }
    //         )
    //     } else {
    //         group_constraints[card_a as usize].retain(
    //             |group| {
    //                 // Removing groups is player_a flag == true
    //                 // and the count_alive is 1
    //                 !group.get_player_flag(player_a)
    //                 || group.count_alive() > 1
    //             }
    //         );
    //         group_constraints[card_b as usize].retain(
    //             |group| {
    //                 // Removing groups is player_b flag == true
    //                 // and the count_alive is 1
    //                 !group.get_player_flag(player_b)
    //                 || group.count_alive() > 1
    //             }
    //         );
    //         // Groups certainly lost a card as the card left their group
    //         // and a differnt card came in 
    //         group_constraints[card_a as usize]
    //         .iter_mut()
    //         .filter(|group| group.get_player_flag(player_a) && !group.get_player_flag(player_b))
    //         .for_each(|group| {
    //             group.sub_alive_count(1);
    //             group.sub_total_count(1);
    //         });
    //         group_constraints[card_b as usize]
    //         .iter_mut()
    //         .filter(|group| group.get_player_flag(player_b) && !group.get_player_flag(player_a))
    //         .for_each(|group| {
    //             group.sub_alive_count(1);
    //             group.sub_total_count(1);
    //         });
    //     }
    //     // group_constraints[card_a as usize]
    //     // .iter_mut()
    //     // .filter(|group| group.get_player_flag(player_a) && !group.get_player_flag(player_b))
    //     // .for_each(|group| {
    //     //     group.set_player_flag(player_b, true);
    //     //     group.add_dead_count(player_b_dead_card_a);
    //     //     group.add_alive_count(player_b_inferred_card_a);
    //     //     group.add_total_count(player_b_dead_card_a + player_b_inferred_card_a);
    //     // });
    //     // group_constraints[card_b as usize]
    //     // .iter_mut()
    //     // .filter(|group| group.get_player_flag(player_b) && !group.get_player_flag(player_a))
    //     // .for_each(|group| {
    //     //     group.set_player_flag(player_a, true);
    //     //     group.add_dead_count(player_a_dead_card_b);
    //     //     group.add_alive_count(player_a_inferred_card_b);
    //     //     group.add_total_count(player_a_dead_card_b + player_a_inferred_card_b);
    //     // });
    //     // TODO: OPTIMIZE, actually don't need both for if player_id is 6
    //     // CASE:
    //     //  both have flags but the player has single_flag
    //     //  since we don't know for certain if that card that was previously given from the player is the same that returned
    //     //  we have to tag it as false
    //     group_constraints[card_b as usize]
    //     .iter_mut()
    //     .filter(|group| group.get_player_flag(player_a) && group.get_player_flag(player_b))
    //     .for_each(|group| {
    //         group.set_single_card_flag(player_a, false);
    //     });
    //     group_constraints[card_a as usize]
    //     .iter_mut()
    //     .filter(|group| group.get_player_flag(player_a) && group.get_player_flag(player_b))
    //     .for_each(|group| {
    //         group.set_single_card_flag(player_b, false);
    //     });
    //     if card_a != card_b {
    //         group_constraints[card_a as usize]
    //         .iter_mut()
    //         .filter(|group| !group.get_player_flag(player_a) && group.get_player_flag(player_b))
    //         .for_each(|group| {
    //             group.add_alive_count(1);
    //             group.add_total_count(1);
    //         });
    //         group_constraints[card_b as usize]
    //         .iter_mut()
    //         .filter(|group| !group.get_player_flag(player_b) && group.get_player_flag(player_a))
    //         .for_each(|group| {
    //             group.add_alive_count(1);
    //             group.add_total_count(1);
    //         });
    //         // CASE:
    //         //  both have flags but the player has single_flag
    //         //  since we don't know for certain if that card that was previously given from the player is the same that returned
    //         //  we have to tag it as false
    //         // ANOTHER CASE:
    //         // maybe since P3 has 1 life when they RR
    //         // the card moving means P3 can't have it!
    //         // swap_mix
    //     }
    // }
    /// Adds inferred cards at the end of a reveal_redraw_diff
    pub fn swap_add_inferred_cards_unchecked(&mut self, player_a: usize, card_a: Card, player_b: usize, card_b: Card) {
        // TODO: [IMPLEMENT] Need to add the case for single_card_flag on card != card_num ZZ2A as per add_dead_cards
        log::info!("In add_inferred_card");
        // Custom inferred_prune
        // self.inferred_card_prune(player_a, card_a);
        // self.inferred_card_prune(player_b, card_b);
        self.inferred_constraints[player_b].push(card_b);
        self.inferred_constraints[player_a].push(card_a);
        // TODO: Inferred card needs to prune too!
    }
    /// Used in revealredraw_relinquish
    /// Player a and Player b swap a card 
    ///     - the card player_a gave player_b is known
    ///     - the card player_b gave player_a is unknown
    /// The convention here is that player_a relinquishes card to player b
    /// Assumptions follow that of revealredraw_relinquish
    pub fn relinquish_mix(&mut self, player_a: usize, player_b: usize, relinquish_card: Card) {
        // TODO: THEORY CHECK and DOCUMENT
        // For now im thinking for player 0 and player 6
        // get all card counts
        let relinquish_card_num = relinquish_card as usize;
        let player_a_dead_card_count = self.player_dead_card_count(player_a, relinquish_card);
        let player_a_inferred_card_count = self.player_inferred_card_count(player_a, relinquish_card);
        let player_a_dead_card_counts = self.player_dead_card_counts(player_a);
        let player_b_dead_card_counts = self.player_dead_card_counts(player_b);
        let player_a_inferred_card_counts = self.player_inferred_card_counts(player_a);
        let player_b_inferred_card_counts = self.player_inferred_card_counts(player_b);
        // log::trace!("swap_mix player_a: {player_a}, card_a: {:?}, player_b: {player_b}, card_b: {:?}", card_a, card_b);
        // log::trace!("swap_mix player_b_dead_card_a: {:?}", player_b_dead_card_a);
        // log::trace!("swap_mix player_b_inferred_card_a: {:?}", player_b_inferred_card_a);
        // log::trace!("swap_mix player_a_dead_card_b: {:?}", player_a_dead_card_b);
        // log::trace!("swap_mix player_a_inferred_card_b: {:?}", player_a_inferred_card_b);
        let group_constraints = self.group_constraints_mut();
        group_constraints[relinquish_card_num]
        .iter_mut()
        .filter(
            |group| group.get_player_flag(player_a) && !group.get_player_flag(player_b)
        )
        .for_each(
            |group| {
                let player_b_dead_card_count = player_b_dead_card_counts[relinquish_card_num];
                let player_b_inferred_card_count = player_b_inferred_card_counts[relinquish_card_num];
                group.set_player_flag(player_b, true);
                group.add_dead_count(player_b_dead_card_count);
                group.add_alive_count(player_b_inferred_card_count);
                group.add_total_count(player_b_dead_card_count + player_b_inferred_card_count);
        });
        for card_num in 0..5 {
            group_constraints[card_num]
            .iter_mut()
            .filter(
                |group| group.get_player_flag(player_b) && !group.get_player_flag(player_a)
            )
            .for_each(
                |group| {
                    let player_a_dead_card_count = player_a_dead_card_counts[card_num];
                    let player_a_inferred_card_count = player_a_inferred_card_counts[card_num];
                    group.set_player_flag(player_a, true);
                    group.add_dead_count(player_a_dead_card_count);
                    group.add_alive_count(player_a_inferred_card_count);
                    group.add_total_count(player_a_dead_card_count + player_a_inferred_card_count);
                }
            )
        }
    }
    /// Function to call for move Ambassador, without considering private information seen by the player who used Ambassador
    pub fn ambassador_public(&mut self, player_id: usize) {
        log::trace!("=== Before Mix ===");
        // PD Obsolete
        // self.increment_move_count();
        self.printlog();
        self.mix(player_id);
        log::trace!("=== After Mix ===");
        self.printlog();
        self.ambassador_inferred_adjustment(player_id);
        // Some groups can be inferred even after adjustment. E.g. before mix [1 0 0 0 0 0 1] 3 Duke => inferred_pile 1 Duke
        log::trace!("=== After ambassador_inferred_adjustment ===");
        self.printlog();
        self.group_redundant_prune();
        log::trace!("=== After ambassador group_redundant_prune ===");
        self.printlog();
        // You might think that add_inferred_groups is not required for ambassador()
        // There is a case, where
        // [0 1 0 0 0 1 1] has 3 Captains, player 1 and 5 each have 1 life => inferred pile constraints has [Captain]
        // we obviously have derived
        // [0 0 0 0 0 1 1] has 2 Captains
        // [0 1 0 0 0 0 1] has 2 Captains
        // When Player 5 Exchanges, they change to the following
        // [0 0 0 0 0 1 1] has 2 Captains
        // [0 1 0 0 0 1 1] has 2 Captains
        // If we do not run add_inferred_groups things seem fine, we will keep the inferred pile constraints [Captain]
        // But if Player 1 Exchanges, since we no long have [0 1 0 0 0 0 1] has 2 Captains in group constraints
        // inferred pile constraints loses the [Captain] and becomes empty [] even though it can still be derived from the groups
        //      - This is because it is not picked up in self.total_known_alive_with_player_and_pile(player_id) in self.ambassador_inferred_adjustment()
        self.add_inferred_information(); // TODO: [OPTIMIZE] Maybe might only require add_subset groups?
        // [FIX]
        // self.revealed_status[player_id].clear();

        // PD Replace
        // self.revealed_status[player_id].push((None, self.reveal_redraw_move_counter));
    }
    /// Function to call for move Ambassador, when considering private information seen by the player who used Ambassador
    pub fn ambassador_private(&mut self, player_id: usize) {
        // represent ambassador inferred cards
        // represent player inferred cards
        // "reveal" of sorts
        // dilution
        // swap?
        // PD Replace
        // self.increment_move_count();
        todo!()
    }
    // TODO: [TEST]
    // TODO: This here gives reason to seperate group constraints by cards
    /// Adds a group => consisting of player and pile, as well as the card
    /// - checks if anything in the group_constraints makes it redundant
    /// - checks if it makes anything in group_constraints redundant
    /// NOTE:
    /// - Assumes the group constraints and internally consistent, and no particular group makes another group redundant
    /// - If group constraints are internally consistent, this leaves it internally consistent
    /// - If group constraints may be internally inconsistent, this may leave it internally inconsistent
    /// - Assumes redundancy is transitive, 
    ///     which is important as it allows us to remove a stored group j knowing that if the added group is found to be redundant later, 
    ///     both groups would still be redundant
    /// - group passed in should include the relevant dead_counts too 
    /// - Leaves self.group_constraint internally consistent
    pub fn add_group_constraint(&mut self, group: CompressedGroupConstraint) {
        if self.is_known_information(&group) {
            return
        }
        let group_constraints = match group.card() {
            Card::Ambassador => &mut self.group_constraints_amb,
            Card::Assassin => &mut self.group_constraints_ass,
            Card::Captain => &mut self.group_constraints_cap,
            Card::Duke => &mut self.group_constraints_duk,
            Card::Contessa => &mut self.group_constraints_con,
        };
        let mut j: usize = 0;
        while j < group_constraints.len() {
            if group_constraints[j].part_list_is_subset_of(&group) &&
            group.count_alive() <= group_constraints[j].count_alive() {
                return
            }
            if group.part_list_is_subset_of(&group_constraints[j]) &&
            group_constraints[j].count_alive() <= group.count_alive() {
                group_constraints.swap_remove(j);
                continue;
            }
            j += 1;
        }
        group_constraints.push(group);
    }
    // TODO: [TEST]
    /// Loops through group_constraints, and removes redundant constraints
    /// Compares them internally
    /// NOTE:
    /// - Assumes groups where all of a particular card is dead will not exist before this as they are implicitly pruned in
    ///   reveal_group_adjustment 
    /// [B] is not informational subset of [A]
    /// [A] Card: Captain, Flags: [0 1 1 0 1 0 1], Single Card Flags: [0 0 0 0 0 0 0], 1 dead 2 alive 3 total
    /// [B] Card: Captain, Flags: [1 1 1 1 1 1 1], Single Card Flags: [0 0 1 0 0 0 0], 1 dead 1 alive 2 total
    /// TODO: [REDUNDANCY OPTIMIZE] Consider that in this case [B] should prob be info subset of A since the single flag B has is not even in A part list
    /// [A] Card: Captain, Flags: [0 1 0 0 1 0 1], Single Card Flags: [0 0 0 0 0 0 0], 1 dead 2 alive 3 total
    /// [B] Card: Captain, Flags: [1 1 1 1 1 1 1], Single Card Flags: [0 0 1 0 0 0 0], 1 dead 1 alive 2 total
    pub fn group_redundant_prune(&mut self) {
        let mut i: usize = 0;
        let mut j: usize = 0;
        for group_constraints in [&mut self.group_constraints_amb, &mut self.group_constraints_ass, &mut self.group_constraints_cap, &mut self.group_constraints_duk, &mut self.group_constraints_con].iter_mut() {
            'outer:  while i < group_constraints.len() {
                j = i + 1;
                // Holding out on this cause of not being able to borrow self as immutable
                // if self.is_known_information(&group_constraints[i]) {
                //     group_constraints.swap_remove(i);
                //     continue 'outer;
                // }
                'inner: while j < group_constraints.len() {
                    // If group i is == group j
                    if group_constraints[i] == group_constraints[j] {
                        group_constraints.swap_remove(i);
                        continue 'outer;
                    }

                    // Subset redundance
                    // Dead has to be the same if not we remove some dead groups that we would actually need in the group
                    // For impossibility to be determined
                    if group_constraints[i].count_dead() == group_constraints[j].count_dead() {
                        // If group i is made redundant by group j
                        if group_constraints[j].part_list_is_subset_of(&group_constraints[i]) &&
                        group_constraints[i].count_alive() < group_constraints[j].count_alive() 
                        // && group_constraints[i].single_card_flags_is_subset_of(group_constraints[j]) 
                        {
                            // NOTE: DO NOT SET THIS TO <= EQUALITY BREAKS INFERRED GROUPS IDK WHY
                            // I set to <= tee hee
                            group_constraints.swap_remove(i);
                            continue 'outer;
                        }
                        // If group j is made redundant by group i
                        if group_constraints[i].part_list_is_subset_of(&group_constraints[j]) &&
                        group_constraints[j].count_alive() < group_constraints[i].count_alive() 
                        // && group_constraints[j].single_card_flags_is_subset_of(group_constraints[i])
                        {
                            // NOTE: DO NOT SET THIS TO <= EQUALITY BREAKS INFERRED GROUPS IDK WHY
                            // I set to <= tee hee
                            group_constraints.swap_remove(j);
                            continue 'inner;
                        }
                    }
                    j += 1;
                }
                i += 1;
            }
        }
    }
    /// This is a temporary function that removes unneeded groups with total_count == 3
    /// This should be modified in future such that we simply do not add such groups in
    /// This reduces errors for some reason
    pub fn temp_remove_redundant_three_groups(&mut self) {
        // TODO: [OPTIMIZE] Should I just exclude this?
        for group in self.group_constraints_mut().iter_mut() {
            let mut i: usize = 0;
            'outer: while i < group.len() {
                let mut j: usize = i + 1;
                if group[i].get_total_count() == 3 {
                    'inner: while j < group.len() {
                        if group[j].get_total_count() == 3 {
                            // [BUG] single_card_flags subset is weird?
                            if group[j].part_list_is_subset_of(&group[i]) && group[i].single_card_flags_is_subset_of(group[j]){
                                group.swap_remove(i);
                                continue 'outer;
                            }
                            if group[i].part_list_is_subset_of(&group[j]) && group[j].single_card_flags_is_subset_of(group[i]){
                                group.swap_remove(j);
                                continue 'inner;
                            }
                        }
                        j += 1;
                    }
                }
                i += 1;
            }
        }
    }
    // TODO: [ALT] Make alternate version of this that adds with 2n checks for when you use it with a particular group added in mind.
    // TODO: Theory Check & Document
    // Or just use reveal LMAO, bacause thats what reveal does?
    /// this function adds all the groups that can be inferred
    /// 
    /// Assumes self.group_constraints is not internally redundant
    /// Leaves self.group_constraint not internally redundant
    pub fn add_inferred_information(&mut self) {
        // DATA STRUCTURE: STORING IMPOSSIBLE ALIVE STATES
        // CASE 0: Player cannot have 4 cards
        // CASE 0b: Player cannot have 3 types of cards and there is only 1 of each remaining card left and player has 2 lives
        // CASE 1: All cards are dead => No one else can have that card alive
        // CASE 2: All cards are dead or known in inferred constraints => No one else can have that card alive
        // CASE 3: All cards are dead or known in some set of players => No one else outside that set can have that card alive
        // CASE 4: All cards are known in some set of players => No player in that set can have a alive card that is outside of the alive cards in that set
        // CASE 5: Some cards are known for 2 players => Let s_min be the amount of lives for the player with the least lives.
        //         If the set contains n_i alive cards for the ith card. The other player must have at least (n_i - s_min) of the ith card.
        //       e.g. [1 0 0 0 0 0 1] has 3 alive Dukes. Player 0 has a dead captain. Therefore, pile must have at least 3 - 1 = 2 Dukes.
        //          This is represented as pile having 2 Dukes in inferred constraints, and [1 0 0 0 0 0 1] 3 Alive Dukes. [0 0 0 0 0 0 1] 2 Alive Dukes is redundant.
        // Case 5 cont: Some cards are known for n players. => Let total lives for any player in the set, except player j be s_-j. For each card i in the group. 
        //              Each player j, must have at least (alive_count_i - s_-j)^+ cards 
        //       e.g. 2 players are known to have 3 Dukes and 1 Captain all alive. Each have at least 1 
        // Case 5 QN: Need to further consider that we inferred the cards for some players
        //       e.g. [1 0 0 0 0 0 1] has 3 alive Dukes. 
        //              - Player 0 has an inferred Duke. Therefore, pile must have at least 3 - 1 - 1 = 1 Dukes.
        //              - Player 0 has an inferred Captain. Therefore, pile must have at least 3 - 0 - 1 = 2 Dukes.
        //              - or 3 - 2 lives + 1 Number of Dukes
        //          3 - inferred alive of player 0 - alive unknown card space
        // Case 5 cont: Some cards are known for n players. => 
        //              Let total alive and unknown card space for any player in the set, except player j be s_-j = total alive cards - known alive cards 
        //              Let inferred alive card count for any player in the set, except player j be inf_-j. For each card i in the group. 
        //              Each player j, must have at least (alive_count_i - s_-j - inf_-j)^+ cards 
        //       e.g. 2 players are known to have 3 Dukes and 1 Captain all alive. Each have at least 1 
        //       NOTE: This can be done recursively based on the 1 player removal case, wont ever have to check the subtract 3 case...
        // Does this continue for also further sub groups, for all combinations of 1 that are subset of [1 0 0 0 0 0 1]? => dynamic programming
        // [1 1 0 0 0 1 0] => splits into [0 1 0 0 0 1 0], [1 0 0 0 0 1 0], [1 1 0 0 0 0 0] which we can add to a queue and further process?
        // == ALGO DRAFT ==
        // Start with a queue of part list, fill it with the super union one
        // Get info for the first part list in the queue
        // Get info for the 1 player removal case, for each one, add the removal part list in, if necessary add new information in
        //  - Add info that is not redundant
        //  - Let it remove info that is redundant
        //  - Inferred cards basically only added in the 2 player part list removal case
        // NOTE: In a sense after processing [0 1 1 1 1 1 1], [1 0 1 1 1 1 1] might add [0 0 1 1 1 1 1] info that may be relevant to the first part list, which may warrant recursion
        //      - But the point of this is not to create the perfect function, but to have impossible cases and assured cases accurately reflected
        //      - Also you realise that each inferred group only results in needing to check 1 other broader group, in this case [0 1 1 1 1 1 1] so we could just add that in again?
        // IMPOSSIBLE CASES => Group outside of player has all the cards, so if any group has all the cards, all players outside will be updated to not having it
        // Repeat
        // Store visited sets in a vec
        // Maybe to avoid running this function, having new groups added, then running again, I can get the superset of part lists, then dynamically work downwards from there?
        // CASE 6: Many cards are known and the remaining players form a group because of the constraints
        // Handle Case 5
        // Get group counts for current part list
        // Creating and adding new inferred groups
        // Runs both
        self.temp_remove_redundant_three_groups();
        log::trace!("In add_inferred_groups");
        let mut bool_continue = false;
        // let add_subset =self.add_subset_groups();
        // let mut_excl_changes = self.add_mutually_exclusive_unions();
        // let mut inf_exc_pl = self.add_inferred_except_player();
        // bool_continue = add_subset || mut_excl_changes || inf_exc_pl;
        // // Then runs one if the other is still true
        // // idea here is that subset groups adds all the smaller groups, mut excl adds the larger groups
        // //      - we get smaller groups from larger groups, so if no new larger groups added, no new smaller groups can be inferred
        // //      - we get larger groups from smaller groups, so if no new smaller groups added, no new larger groups can be inferred
        // //      - Technically the functions also add inferred groups, so if no inferred groups added, no new information added, no need to run either  
        
        // while inf_exc_pl {
        //     // Runs a
        //     println!("Running again");
        //     let add_subset = self.add_subset_groups();
        //     log::info!("add_subset_groups added groups: {}", add_subset);
        //     if add_subset {
        //         let mut_excl_changes = self.add_mutually_exclusive_unions();
        //         log::info!("add_mutually_exclusive_unions added groups: {}", mut_excl_changes);
        //         if mut_excl_changes {
        //             inf_exc_pl = self.add_inferred_except_player();
        //             log::info!("add_inferred_except_player added groups: {}", inf_exc_pl);
        //         } else {
        //             inf_exc_pl = false;
        //         }
        //     } else {
        //         inf_exc_pl = false;
        //     }
        // }
        // === unopt ===
        // TODO: [OPTIMIZE] Probably can do like mut excl recurse for the inferred groups?
        log::info!("Before add_subset_groups");
        self.printlog();
        self.add_subset_groups_unopt();
        log::info!("After add_subset_groups");
        self.printlog();
        let mut_excl_changes = self.add_mutually_exclusive_unions();
        log::info!("After add_mutually_exclusive_unions");
        self.printlog();
        let inf_com_pl = self.add_inferred_complement_of_player();
        self.printlog();
        let inf_exc_pl = self.add_inferred_except_player();
        log::info!("After add_inferred_except_player");
        self.printlog();
        let inf_rem_neg = self.add_inferred_remaining_negation();
        // let inf_rem_neg = false;
        log::info!("After add_inferred_remaining_negation");
        self.printlog();
        bool_continue = mut_excl_changes || inf_com_pl || inf_exc_pl || inf_rem_neg;
        // bool_continue = mut_excl_changes || inf_exc_pl;
        while bool_continue {
            self.add_subset_groups_unopt();
            log::info!("After add_subset_groups");
            self.printlog();
            let mut_excl_changes = self.add_mutually_exclusive_unions();
            log::info!("After add_mutually_exclusive_unions");
            self.printlog();
            // This kinda needs to maximal informative set to work. Perhaps to run both above to completion first?
            let inf_com_pl = self.add_inferred_complement_of_player();
            self.printlog();
            let inf_exc_pl = self.add_inferred_except_player();
            log::info!("After add_inferred_except_player");
            self.printlog();
            let inf_rem_neg = self.add_inferred_remaining_negation();
            log::info!("After add_inferred_remaining_negation");
            self.printlog();
            bool_continue = mut_excl_changes || inf_com_pl || inf_exc_pl || inf_rem_neg;
            // bool_continue = mut_excl_changes || inf_exc_pl;
        }
        // === adjusted to fix need for add_inferred_except_player to have maximal informative set else it adds wrongly
        // log::info!("Before add_subset_groups");
        // self.printlog();
        // let subset_changes = self.add_subset_groups_unopt();
        // log::info!("After add_subset_groups");
        // self.printlog();
        // let mut_excl_changes = self.add_mutually_exclusive_unions();
        // log::info!("After add_mutually_exclusive_unions");
        // self.printlog();
        // bool_continue = mut_excl_changes || subset_changes;
        // while bool_continue {
        //     bool_continue = self.add_subset_groups_unopt();
        //     log::info!("After add_subset_groups");
        //     self.printlog();
        //     bool_continue = self.add_mutually_exclusive_unions() || bool_continue;
        //     log::info!("After add_mutually_exclusive_unions");
        //     self.printlog();
        // }
        // // This kinda needs to maximal informative set to work. Perhaps to run both above to completion first?
        // let inf_exc_pl = self.add_inferred_except_player();
        // log::info!("After add_inferred_except_player");
        // self.printlog();
        // if inf_exc_pl {
        //     self.add_inferred_groups();
        // }
    }
    /// Assumes groups in vec all have same card as group input
    /// Assumes vec is not internally redundant
    /// adds group into vec, if it is not redundant.
    /// Maintains internal non-redundancy of vec
    fn non_redundant_push(vec: &mut Vec<CompressedGroupConstraint>, group: CompressedGroupConstraint) {
        let mut i: usize = 0;
        while i < vec.len() {
            // Testing duplicate redundance
            if group == vec[i] {
                return
            }
            // TODO: Think abit more about what makes single_card_flags redundant
            // Subset redundance
            if vec[i].single_card_flags_equal(group) && vec[i].count_dead() == group.count_dead() {
                if vec[i].part_list_is_subset_of(&group) && 
                group.count_alive() < vec[i].count_alive() 
                // && group.single_card_flags_is_subset_of(vec[i])
                {
                    // NOTE: DO NOT SET THIS TO <= EQUALITY BREAKS INFERRED GROUPS IDK WHY
                    // group is redundant
                    return
                }
                if group.part_list_is_subset_of(&vec[i]) &&
                vec[i].count_alive() < group.count_alive() 
                // && vec[i].single_card_flags_is_subset_of(group)
                {
                    // NOTE: DO NOT SET THIS TO <= EQUALITY BREAKS INFERRED GROUPS IDK WHY
                    vec.swap_remove(i);
                    continue;
                }
            }
            i += 1;
        }
        vec.push(group);
    }
    /// Assumes groups in vec all have same card as group input
    /// Assumes vec is not internally redundant
    /// Used for a group of single flags, but represented with a single vec without regard for card_num
    fn non_duplicate_push(vec: &mut Vec<CompressedGroupConstraint>, group: CompressedGroupConstraint) {
        let mut i: usize = 0;
        while i < vec.len() {
            // Testing duplicate redundance
            if group == vec[i] {
                return
            }
            i += 1;
        }
        vec.push(group);
    }
    /// Assumes groups in vec all have same card as group input
    /// Assumes vec is not internally redundant
    /// adds group into vec, if it is not redundant.
    /// Maintains internal non-redundancy of vec
    /// Returns true if anything was added were made
    /// This is more relaxed in that it won't consider redundant if [1 1 0 0 0 0 0] 2 vs [1 1 1 0 0 0 0] 2, as we need this ??
    /// can be more relaxed with >, 
    /// >= is stricter and makes more redundant
    /// returns (bool, bool)
    ///     - [0] is true if group is added
    ///     - [1] is true if vec is modified by swap_remove
    fn non_redundant_push_tracked(vec: &mut Vec<CompressedGroupConstraint>, group: CompressedGroupConstraint) -> (bool, bool) {
        let mut i: usize = 0;
        let mut bool_vec_modified_removed: bool = false;
        while i < vec.len() {
            // Testing duplicate redundance
            if group == vec[i] {
                return (false, false)
            }
            // Subset redundance
            if vec[i].single_card_flags_equal(group) && vec[i].count_dead() == group.count_dead() {
                if vec[i].part_list_is_subset_of(&group) && 
                group.count_alive() < vec[i].count_alive() 
                // && group.single_card_flags_is_subset_of(vec[i])
                {
                    // NOTE: DO NOT SET THIS TO <= EQUALITY BREAKS INFERRED GROUPS IDK WHY
                    // group is redundant
                    log::trace!("non_redundant_push_tracked did not add group: {}", group);
                    log::trace!("non_redundant_push_tracked in vec: {:?}", vec);
                    return (false, bool_vec_modified_removed)
                }
                if group.part_list_is_subset_of(&vec[i]) &&
                vec[i].count_alive() < group.count_alive() 
                // && vec[i].single_card_flags_is_subset_of(group)
                {
                    // NOTE: DO NOT SET THIS TO <= EQUALITY BREAKS INFERRED GROUPS IDK WHY
                    vec.swap_remove(i);
                    bool_vec_modified_removed = true;
                    continue;
                }
            }
            i += 1;
        }
        log::trace!("non_redundant_push_tracked added group: {}", group);
        log::trace!("non_redundant_push_tracked in vec: {:?}", vec);
        vec.push(group);
        (true, bool_vec_modified_removed)
    }
    /// Adds subset groups to self.group_constraints
    /// 
    /// Assumptions:
    /// Assumes self.group_constraints is not internally redundant
    /// 
    /// Returns:
    /// - bool => Represents whether any additional information was added to self.group_constraints
    /// 
    /// Flow:
    /// === This Function ===
    /// - Loops through self.group_constraints and finds sub groups and adds them to new_groups
    /// - Adds inferred constraints in new_groups to self.inferred_constraints
    /// - Removes redundant groups in self.group_constraint as a result of new inferred_constraints
    /// - returns true if literally anything changes
    /// [OPTIMIZE] I think only need to repeat if inferred group is added?
    pub fn add_subset_groups_unopt(&mut self) -> bool {
        // There technically is alot of repeated code, but i want to be able to pass ownership through the recursed input instead of just a &mut to reduce memory usage
        // In addition the first step should not add groups, so to not make use of branches, its seperated as such
        // Recursion stops of no more groups to add
        let mut bool_changes = false;
        if self.group_constraints().iter().all(|v| v.is_empty()) {
            return false
        }
        log::trace!("In add_subset_groups");
        // TODO: You can optimiz
        let mut new_groups: Vec<Vec<CompressedGroupConstraint>> = vec![Vec::with_capacity(3); 5];
        let mut new_inferred_constraints: Vec<CompressedGroupConstraint> = Vec::with_capacity(3);
        // Inferring from self???
        // TODO: Change to reference
        for (card_num, group_constraint) in self.group_constraints().iter().enumerate() {
            for group in group_constraint.iter() {
                // Get inferred groups, add then to new_groups
                let part_list: [bool; 7] = group.get_set_players();
                let flags_count = part_list.iter().filter(|b| **b).count() as u8;
                // log::trace!("add_subset_groups flags_count: {}", flags_count);
                if flags_count > 1 {
                    // Creation of new groups (may have only 1 player_flag)
                    // group => [1 1 0 1 0 0 1], add [0 1 0 1 0 0 1], [1 0 0 1 0 0 1], [1 1 0 0 0 0 1], []
                    for (player, player_flag) in part_list.iter().enumerate() {
                        if *player_flag {
                            // Lets say we know this
                            // Player 0 has a dead captain
                            // Group: Card: Captain, Flags: [0 0 0 0 1 0 1], 0 dead 1 alive 1 total
                            // If Player 5 RevealRedraw Captain
                            // Group: Card: Captain, Flags: [0 0 0 0 1 1 1], 0 dead 2 alive 2 total
                            // However, only 1 of player 5's cards actually gets mixed into the pile
                            // When Player 4 discards contessa and is completely dead this reduces to
                            // Group: Card: Captain, Flags: [0 0 0 0 0 1 1], 0 dead 2 alive 2 total
                            // But since only 1 of player 5's card is actually part of this mix
                            // We can conclude that the pile must have 2-1 = 1 Captain!
                            // Because Player 5 can have at most 1 Captain! not 2!
                            // 2 lives, 1 known other card =>
                            // 2 lives, single_card_flag => 1
                            // 1 life, single_card_flag => 1
                            // 1 life, 1 known other card => 0
                            let player_lives: u8 = if player != 6 {
                                2 - (self.public_constraints[player].len() as u8).max(group.get_single_card_flag(player) as u8)
                            } else {
                                3 // Not 0
                            };
                            let player_inferred_diff_cards: u8 = self.inferred_constraints[player].iter().filter(|c| **c as usize != card_num).count() as u8;
                            log::trace!("add_subset_groups_unopt group: {:?}", group);
                            log::trace!("player: {player}, public_constraints: {:?}", self.public_constraints);
                            log::trace!("player: {player}, inferred_constraints: {:?}", self.inferred_constraints);
                            log::trace!("player_lives: {player_lives}, player_inferred_diff_cards: {player_inferred_diff_cards}");
                            let max_holdable_spaces: u8 = if player_lives == 1 && player_inferred_diff_cards == 1 {
                                1
                            } else if self.inferred_constraints[player].len() == 2 {
                                player_lives - player_inferred_diff_cards
                                // 0 // Handles case where we know both of a players' cards
                            } else {
                                // TESTING subtract_overflow_1
                                // Actually this can overflow if we know both of a players' cards
                                player_lives - player_inferred_diff_cards
                            };
                            // if group.count_alive() + player_inferred_diff_cards > player_lives {
                            if group.count_alive() > max_holdable_spaces {
                                // Cards contained is n - player_lives + player_inferred_cards for new group
                                log::trace!("");
                                log::trace!("=== add_subset_groups Reference === ");
                                log::trace!("add_subset_groups player considered: {:?}", player);
                                log::trace!("add_subset_groups parent group: {}", group);
                                log::trace!("add_subset_groups public_constraints: {:?}", self.public_constraints);
                                log::trace!("player: {}, player_lives: {}, player_inferred_diff_cards: {}, group.count_alive(): {}", player, player_lives, player_inferred_diff_cards, group.count_alive());
                                log::trace!("add_subset_groups inferred_constraints: {:?}", self.inferred_constraints);
                                let mut new_group: CompressedGroupConstraint = *group;
                                new_group.set_player_flag(player, false);
                                new_group.set_single_card_flag(player, false);
                                let dead_card_count = self.public_constraints[player].iter().filter(|c| **c as usize == card_num).count() as u8;
                                new_group.sub_dead_count(dead_card_count);
                                // new_group.set_alive_count(group.count_alive() + player_inferred_diff_cards - player_lives );
                                new_group.set_alive_count(group.count_alive() - max_holdable_spaces);
                                new_group.set_total_count(new_group.count_alive() + new_group.count_dead());
                                // Required to meet assumptions of recursive function
                                if flags_count > 2 {
                                    log::trace!("add_subset_groups found group for new_groups: {}", new_group);
                                    PathDependentCollectiveConstraint::non_redundant_push(&mut new_groups[card_num], new_group);
                                } else {
                                    // only 1 flag after removal, and so should be added to inferred constraints later
                                    log::trace!("add_subset_groups found single group for new_inferred_constraints: {}", new_group);
                                    // Non duplicate used as new_inferred_constraints can have different card types
                                    PathDependentCollectiveConstraint::non_duplicate_push(&mut new_inferred_constraints, new_group);
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut card_changes: Vec<Vec<usize>> = vec![Vec::with_capacity(2); 5]; // Store (player_id, bool_counts => false: 1, true: 2)
        // Add new_inferred_constraints
        // TODO: [FIX] Adding new_inferred constraints requires reconsidering the entire group_constraints list not just the new_groups
        while let Some(single_flag_group) = new_inferred_constraints.pop() {
            log::trace!("add_subset considering to add single_flag_group: {}", single_flag_group);
            let alive_count = single_flag_group.count_alive();
            // Something is wrong if this panics, all groups should have a single flag, not no flags
            let player_id = single_flag_group.get_set_players().iter().position(|b| *b).unwrap();
            let card: Card = single_flag_group.card();
            match alive_count {
                1 => {
                    // One card known
                    if !self.inferred_constraints[player_id].contains(&card) {
                        log::trace!("");
                        log::trace!("=== add_subset_groups Inferred 1 === ");
                        log::trace!("add_sub_groups adding player considered: {}", player_id);
                        log::trace!("add_sub_groups adding 1 card single_flag_group: {}", single_flag_group);
                        log::trace!("add_sub_groups Before self.public_constraints: {:?}", self.public_constraints);
                        log::trace!("add_sub_groups Before self.inferred_constraints: {:?}", self.inferred_constraints);
                        self.inferred_constraints[player_id].push(card);
                        debug_assert!(self.inferred_constraints[player_id].len() < 4, "UO1");
                        card_changes[card as usize].push(player_id);
                        log::trace!("add_sub_groups After self.public_constraints: {:?}", self.public_constraints);
                        log::trace!("add_sub_groups After self.inferred_constraints: {:?}", self.inferred_constraints);
                        bool_changes = true;
                        // TODO: Needs to prune the groups too...
                        // TODO: Adjust counts properly too... or remove inferred_counts
                    }
                },
                2 => {
                    if player_id == 6 {
                        let current_count = self.inferred_constraints[player_id].iter().filter(|c| **c == card).count() as u8; 
                        if alive_count > current_count {
                            let no_to_push = alive_count - current_count;
                            for _ in 0..no_to_push {
                                self.inferred_constraints[player_id].push(card);
                                debug_assert!(self.inferred_constraints[player_id].len() < 4, "UO2");
                                bool_changes = true;
                            }
                        }
                    } else {
                        // Both cards known
                        if self.inferred_constraints[player_id] != vec![card; 2] {
                            log::trace!("");
                            log::trace!("=== add_subset_groups Inferred 2 === ");
                            log::trace!("add_sub_groups adding player considered: {}", player_id);
                            log::trace!("add_sub_groups adding 2 cards single_flag_group: {}", single_flag_group);
                            log::trace!("add_sub_groups Before self.public_constraints: {:?}", self.public_constraints);
                            log::trace!("add_sub_groups Before self.inferred_constraints: {:?}", self.inferred_constraints);
                            self.inferred_constraints[player_id].clear();
                            self.inferred_constraints[player_id].push(card);
                            self.inferred_constraints[player_id].push(card);
                            debug_assert!(self.inferred_constraints[player_id].len() < 3, "UO3");
                            card_changes[card as usize].push(player_id);
                            log::trace!("add_sub_groups After self.public_constraints: {:?}", self.public_constraints);
                            log::trace!("add_sub_groups After self.inferred_constraints: {:?}", self.inferred_constraints);
                            // TODO: Needs to prune the groups too... some batch function that prunes groups based on changes?
                            bool_changes = true;
                        }
                    }
                    // TODO: Adjust counts properly too... or remove inferred_counts
                },
                3 => {
                    if player_id == 6 {
                        let current_count = self.inferred_constraints[player_id].iter().filter(|c| **c == card).count() as u8;
                        if alive_count > current_count {
                            let no_to_push = alive_count - current_count;
                            for _ in 0..no_to_push {
                                self.inferred_constraints[player_id].push(card);
                                debug_assert!(self.inferred_constraints[player_id].len() < 4, "UO4");
                                bool_changes = true;
                            }
                        }
                    } else {
                        log::trace!("group: {}", single_flag_group);
                        log::trace!("alive_count: {}", alive_count);
                        debug_assert!(false, "You really should not be here... there should only be alive_count of 2 or 1 for a single player!");
                    }
                    // TODO: Adjust counts properly too... or remove inferred_counts
                },
                _ => {
                    log::trace!("group: {}", single_flag_group);
                    log::trace!("alive_count: {}", alive_count);
                    debug_assert!(false, "You really should not be here... there should only be alive_count of 2 or 1 for a single player!");
                }
            }
        }

        // batch prune
        // This is formatted like this because of some &mutable and immutable borrow issue with the compiler..
        let self_groups = [&mut self.group_constraints_amb, 
                                                                    &mut self.group_constraints_ass, 
                                                                    &mut self.group_constraints_cap, 
                                                                    &mut self.group_constraints_duk, 
                                                                    &mut self.group_constraints_con];
        // Ensures no inferred constraint makes a group redundant
        for (card_num, changes) in card_changes.iter().enumerate() {
            let mut i: usize = 0;
            'group_removal: while i < self_groups[card_num].len() {
                for &player_id in changes {
                    // If player flag is true and number of cards player now has
                    if self_groups[card_num][i].get_player_flag(player_id) {
                        let count_alive = self.inferred_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                        if count_alive >= self_groups[card_num][i].count_alive() {
                            self_groups[card_num].swap_remove(i);
                            continue 'group_removal;
                        }
                    }
                }
                i += 1;
            }
        }
        let mut self_groups = self.group_constraints_mut();
        for (card_num, group_constraints) in new_groups.iter_mut().enumerate() {
            while let Some(group) = group_constraints.pop() {
                bool_changes = Self::non_redundant_push_tracked(&mut self_groups[card_num], group).0 || bool_changes;
            }
        }
        if bool_changes  {
            return self.add_subset_groups_unopt();
        } 
        false
    }
    /// Adds subset groups to self.group_constraints
    /// 
    /// Assumptions:
    /// Assumes self.group_constraints is not internally redundant
    /// 
    /// Returns:
    /// - bool => Represents whether any additional information was added to self.group_constraints
    /// 
    /// Flow:
    /// === This Function ===
    /// - Loops through self.group_constraints and finds sub groups and adds them to new_groups
    /// - Adds inferred constraints in new_groups to self.inferred_constraints
    /// - Removes redundant groups in self.group_constraint as a result of new inferred_constraints
    /// - Starts recursion with new_groups as reference_group_constraint
    /// === Recursive Function ===
    /// - Recurses with new_groups being the reference group
    ///     - Finds sub groups from new_groups and adds them to new_new_groups
    ///     - Adds new_groups to self
    ///     - Adds inferred constraints in new_groups to self.inferred_constraints
    ///     - Removes redundant groups in self.group_constraint as a result of new inferred_constraints
    ///     - Recurses with new_new_groups being the new reference group
    pub fn add_subset_groups(&mut self) -> bool {
        // There technically is alot of repeated code, but i want to be able to pass ownership through the recursed input instead of just a &mut to reduce memory usage
        // In addition the first step should not add groups, so to not make use of branches, its seperated as such
        // Recursion stops of no more groups to add
        if self.group_constraints().iter().all(|v| v.is_empty()) {
            return false
        }
        log::trace!("In add_subset_groups");
        // TODO: You can optimiz
        let mut new_groups: Vec<Vec<CompressedGroupConstraint>> = vec![Vec::with_capacity(3); 5];
        let mut new_inferred_constraints: Vec<CompressedGroupConstraint> = Vec::with_capacity(3);
        // Inferring from self???
        // TODO: Change to reference
        for (card_num, group_constraint) in self.group_constraints().iter().enumerate() {
            for group in group_constraint.iter() {
                // Get inferred groups, add then to new_groups
                let part_list: [bool; 7] = group.get_set_players();
                let flags_count = part_list.iter().filter(|b| **b).count() as u8;
                // log::trace!("add_subset_groups flags_count: {}", flags_count);
                if flags_count > 1 {
                    // Creation of new groups (may have only 1 player_flag)
                    // group => [1 1 0 1 0 0 1], add [0 1 0 1 0 0 1], [1 0 0 1 0 0 1], [1 1 0 0 0 0 1], []
                    for (player, player_flag) in part_list.iter().enumerate() {
                        if *player_flag {
                            // Lets say we know this
                            // Player 0 has a dead captain
                            // Group: Card: Captain, Flags: [0 0 0 0 1 0 1], 0 dead 1 alive 1 total
                            // If Player 5 RevealRedraw Captain
                            // Group: Card: Captain, Flags: [0 0 0 0 1 1 1], 0 dead 2 alive 2 total
                            // However, only 1 of player 5's cards actually gets mixed into the pile
                            // When Player 4 discards contessa and is completely dead this reduces to
                            // Group: Card: Captain, Flags: [0 0 0 0 0 1 1], 0 dead 2 alive 2 total
                            // But since only 1 of player 5's card is actually part of this mix
                            // We can conclude that the pile must have 2-1 = 1 Captain!
                            // Because Player 5 can have at most 1 Captain! not 2!
                            let player_lives: u8 = if player != 6 {
                                2 - (self.public_constraints[player].len() as u8).max(group.get_single_card_flag(player) as u8)
                            } else {
                                3 // Not 0
                            };
                            let player_inferred_diff_cards: u8 = self.inferred_constraints[player].iter().filter(|c| **c as usize != card_num).count() as u8;
                            // log::trace!("player: {}, player_lives: {}, player_inferred_diff_cards: {}, group.count_alive(): {}", player, player_lives, player_inferred_diff_cards, group.count_alive());
                            if group.count_alive() + player_inferred_diff_cards > player_lives {
                                // Cards contained is n - player_lives + player_inferred_cards for new group
                                log::trace!("");
                                log::trace!("=== add_subset_groups Reference === ");
                                log::trace!("add_subset_groups player considered: {:?}", player);
                                log::trace!("add_subset_groups parent group: {}", group);
                                log::trace!("add_subset_groups public_constraints: {:?}", self.public_constraints);
                                log::trace!("add_subset_groups inferred_constraints: {:?}", self.inferred_constraints);
                                let mut new_group: CompressedGroupConstraint = *group;
                                new_group.set_player_flag(player, false);
                                new_group.set_single_card_flag(player, false);
                                let dead_card_count = self.public_constraints[player].iter().filter(|c| **c as usize == card_num).count() as u8;
                                new_group.sub_dead_count(dead_card_count);
                                new_group.set_alive_count(group.count_alive() + player_inferred_diff_cards - player_lives );
                                new_group.set_total_count(new_group.count_alive() + new_group.count_dead());
                                // Required to meet assumptions of recursive function
                                if flags_count > 2 {
                                    log::trace!("add_subset_groups found group for new_groups: {}", new_group);
                                    PathDependentCollectiveConstraint::non_redundant_push(&mut new_groups[card_num], new_group);
                                } else {
                                    // only 1 flag after removal, and so should be added to inferred constraints later
                                    log::trace!("add_subset_groups found group for new_inferred_constraints: {}", new_group);
                                    // Non duplicate used as new_inferred_constraints can have different card types
                                    PathDependentCollectiveConstraint::non_duplicate_push(&mut new_inferred_constraints, new_group);
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut card_changes: Vec<Vec<usize>> = vec![Vec::with_capacity(2); 5]; // Store (player_id, bool_counts => false: 1, true: 2)
        // Add new_inferred_constraints
        // TODO: [FIX] Adding new_inferred constraints requires reconsidering the entire group_constraints list not just the new_groups
        while let Some(single_flag_group) = new_inferred_constraints.pop() {
            let alive_count = single_flag_group.count_alive();
            // Something is wrong if this panics, all groups should have a single flag, not no flags
            let player_id = single_flag_group.get_set_players().iter().position(|b| *b).unwrap();
            let card: Card = single_flag_group.card();
            match alive_count {
                1 => {
                    // One card known
                    if !self.inferred_constraints[player_id].contains(&card) {
                        log::trace!("");
                        log::trace!("=== add_subset_groups Inferred 1 === ");
                        log::trace!("add_sub_groups adding player considered: {}", player_id);
                        log::trace!("add_sub_groups adding 1 card single_flag_group: {}", single_flag_group);
                        log::trace!("add_sub_groups Before self.public_constraints: {:?}", self.public_constraints);
                        log::trace!("add_sub_groups Before self.inferred_constraints: {:?}", self.inferred_constraints);
                        self.inferred_constraints[player_id].push(card);
                        debug_assert!(self.inferred_constraints[player_id].len() < 3, "SS5");
                        card_changes[card as usize].push(player_id);
                        log::trace!("add_sub_groups After self.public_constraints: {:?}", self.public_constraints);
                        log::trace!("add_sub_groups After self.inferred_constraints: {:?}", self.inferred_constraints);
                        // TODO: Needs to prune the groups too...
                        // TODO: Adjust counts properly too... or remove inferred_counts
                    }
                },
                2 => {
                    if player_id == 6 {
                        let current_count = self.inferred_constraints[player_id].iter().filter(|c| **c == card).count() as u8;
                        if alive_count > current_count {
                            let no_to_push = 2 - current_count;
                            for _ in 0..no_to_push {
                                self.inferred_constraints[player_id].push(card);
                                debug_assert!(self.inferred_constraints[player_id].len() < 4, "SS6");
                            }
                        }
                    } else {
                        // Both cards known
                        if self.inferred_constraints[player_id] != vec![card; 2] {
                            log::trace!("");
                            log::trace!("=== add_subset_groups Inferred 2 === ");
                            log::trace!("add_sub_groups adding player considered: {}", player_id);
                            log::trace!("add_sub_groups adding 2 cards single_flag_group: {}", single_flag_group);
                            log::trace!("add_sub_groups Before self.public_constraints: {:?}", self.public_constraints);
                            log::trace!("add_sub_groups Before self.inferred_constraints: {:?}", self.inferred_constraints);
                            self.inferred_constraints[player_id].clear();
                            self.inferred_constraints[player_id].push(card);
                            self.inferred_constraints[player_id].push(card);
                            debug_assert!(self.inferred_constraints[player_id].len() < 3, "SS7");
                            card_changes[card as usize].push(player_id);
                            log::trace!("add_sub_groups After self.public_constraints: {:?}", self.public_constraints);
                            log::trace!("add_sub_groups After self.inferred_constraints: {:?}", self.inferred_constraints);
                            // TODO: Needs to prune the groups too... some batch function that prunes groups based on changes?
                        }
                    }
                    // TODO: Adjust counts properly too... or remove inferred_counts
                },
                3 => {
                    if player_id == 6 {
                        let current_count = self.inferred_constraints[player_id].iter().filter(|c| **c == card).count() as u8;
                        if alive_count > current_count {
                            let no_to_push = alive_count - current_count;
                            for _ in 0..no_to_push {
                                self.inferred_constraints[player_id].push(card);
                                debug_assert!(self.inferred_constraints[player_id].len() < 4, "SS8");
                            }
                        }
                    } else {
                        log::trace!("group: {}", single_flag_group);
                        log::trace!("alive_count: {}", alive_count);
                        debug_assert!(false, "You really should not be here... there should only be alive_count of 2 or 1 for a single player!");
                    }
                    // TODO: Adjust counts properly too... or remove inferred_counts
                },
                _ => {
                    log::trace!("group: {}", single_flag_group);
                    log::trace!("alive_count: {}", alive_count);
                    debug_assert!(false, "You really should not be here... there should only be alive_count of 2 or 1 for a single player!");
                }
            }
        }

        // batch prune
        // This is formatted like this because of some &mutable and immutable borrow issue with the compiler..
        let self_groups = [&mut self.group_constraints_amb, 
                                                                    &mut self.group_constraints_ass, 
                                                                    &mut self.group_constraints_cap, 
                                                                    &mut self.group_constraints_duk, 
                                                                    &mut self.group_constraints_con];
        // Ensures no inferred constraint makes a group redundant
        for (card_num, changes) in card_changes.iter().enumerate() {
            let mut i: usize = 0;
            'group_removal: while i < self_groups[card_num].len() {
                for &player_id in changes {
                    // If player flag is true and number of cards player now has
                    if self_groups[card_num][i].get_player_flag(player_id) {
                        let count_alive = self.inferred_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                        if count_alive >= self_groups[card_num][i].count_alive() {
                            self_groups[card_num].swap_remove(i);
                            continue 'group_removal;
                        }
                    }
                }
                i += 1;
            }
        }
        // recurse, new_groups should not be internally redundant, self.group_constraints should not be internally redundant
        if new_groups.iter().any(|v| !v.is_empty()) || card_changes.iter().any(|v| !v.is_empty())  {
            return self.add_subset_groups_recurse(new_groups);
        } 
        false
    }
    /// Recursively Adds groups from the inferred subset of another gorup
    /// - e.g. [1 1 0 0 0 0 0] 2 Duke and player 0 has 1 life, player 2 has 2 lives. We know player 2 has at least 1 Duke. As player 1 can have at most 1 Duke.
    /// - e.g. [1 0 0 0 0 0 1] has 3 alive Dukes. 
    ///     - Player 0 has an inferred Duke. Therefore, pile must have at least 3 - 1 - 1 = 1 Dukes.
    ///     - or 3 - 2 lives + 0 Number of non-Dukes
    ///     - Player 0 has an inferred Captain. Therefore, pile must have at least 3 - 0 - 1 = 2 Dukes.
    ///     - or 3 - 2 lives + 1 Number of non-Dukes
    /// - e.g. [1 1 0 0 0 0 1] 3 Duke
    /// - e.g. [1 1 0 0 0 0 0] has 3 alive Dukes. 
    ///     - Player 6 has an inferred Duke. Therefore, pile must have at least 3 - 3 = 0 Dukes.
    ///     - or 3 - 3 lives + 0 Number of non-Dukes
    ///     - Player 6 has an inferred Captain. Therefore, pile must have at least 3 - 3 + 1 = 1 Dukes.
    ///     - or 3 - 3 lives + 1 Number of non-Dukes
    /// Helps infer all possible subgroups iteratively
    /// - By generating subgroups, adding the new subgroups in, and repeating the process on the new subgroups we eventually infer all the possible subgroups
    /// 
    /// Returns:
    /// - bool => Represents whether any additional information was added to self.group_constraints
    /// 
    /// Assumptions:
    /// Assumes reference_group_constraints is not internally redundant
    /// Assumes self.group_constraints is not internally redundant
    /// Assumes self has been compared with self already
    /// 
    /// Flow:
    /// - Compares self with reference_group_constraints and adds ME unions to new_groups
    /// - Compares reference_group_constraints with reference_group_constraints adds ME unions to new_groups
    ///     - Adds reference_group_constraints to self
    ///     - Recurses with new_groups being the new reference_group_constraints
    fn add_subset_groups_recurse(&mut self, mut reference_group_constraints: Vec<Vec<CompressedGroupConstraint>>) -> bool {
        log::trace!("In add_subset_groups_recurse");
        // TODO: You can optimiz
        let mut new_groups: Vec<Vec<CompressedGroupConstraint>> = vec![Vec::with_capacity(3); 5];
        let mut new_inferred_constraints: Vec<CompressedGroupConstraint> = Vec::with_capacity(3);
        // Inferring from self???
        // TODO: Change to reference
        for (card_num, group_constraint) in reference_group_constraints.iter().enumerate() {
            for group in group_constraint.iter() {
                // Get inferred groups, add then to new_groups
                let part_list: [bool; 7] = group.get_set_players();
                let flags_count = part_list.iter().filter(|b| **b).count() as u8;
                if flags_count > 1 {
                    // Creation of new groups (may have only 1 player_flag)
                    // group => [1 1 0 1 0 0 1], add [0 1 0 1 0 0 1], [1 0 0 1 0 0 1], [1 1 0 0 0 0 1], []
                    for (player, player_flag) in part_list.iter().enumerate() {
                        if *player_flag {
                            let player_lives: u8 = if player != 6 {
                                // Adjusted for single card flag
                                // TODO: Document this
                                2 - (self.public_constraints[player].len() as u8).max(group.get_single_card_flag(player) as u8)
                            } else {
                                3 // Not 0
                            };
                            let player_inferred_diff_cards: u8 = self.inferred_constraints[player].iter().filter(|c| **c as usize != card_num).count() as u8;
                            if group.count_alive() + player_inferred_diff_cards > player_lives {
                                // Cards contained is n - player_lives + player_inferred_cards for new group
                                log::trace!("");
                                log::trace!("=== add_subset_groups_recurse Reference === ");
                                log::trace!("add_subset_groups_recurse adding player considered: {}", player);
                                log::trace!("add_subset_groups_recurse parent group: {}", group);
                                log::trace!("add_subset_groups_recurse public_constraints: {:?}", self.public_constraints);
                                log::trace!("add_subset_groups_recurse inferred_constraints: {:?}", self.inferred_constraints);
                                let mut new_group: CompressedGroupConstraint = *group;
                                new_group.set_player_flag(player, false);
                                new_group.set_single_card_flag(player, false);
                                let dead_card_count = self.public_constraints[player].iter().filter(|c| **c as usize == card_num).count() as u8;
                                new_group.sub_dead_count(dead_card_count);
                                new_group.set_alive_count(group.count_alive() + player_inferred_diff_cards - player_lives);
                                new_group.set_total_count(new_group.count_alive() + new_group.count_dead());
                                // Required to meet assumptions of recursive function
                                if flags_count > 2 {
                                    log::trace!("add_subset_groups added to new_groups: {}", new_group);
                                    PathDependentCollectiveConstraint::non_redundant_push(&mut new_groups[card_num], new_group);
                                } else {
                                    // only 1 flag after removal, and so should be added to inferred constraints later
                                    log::trace!("add_subset_groups added to new_inferred_constraints: {}", new_group);
                                    // Non duplicate used as new_inferred_constraints can have different card types
                                    PathDependentCollectiveConstraint::non_duplicate_push(&mut new_inferred_constraints, new_group);
                                }
                            }
                        }
                    }
                }
            }
        }
        // Add reference group to self if not redundant
        // Ensures self.group_constraints is not internally redundant
        let mut self_groups = self.group_constraints_mut();
        let mut bool_changes = false;
        for (card_num, group_constraints) in reference_group_constraints.iter_mut().enumerate() {
            while let Some(group) = group_constraints.pop() {
                bool_changes = Self::non_redundant_push_tracked(&mut self_groups[card_num], group).0 || bool_changes;
            }
        }
        let mut card_changes: Vec<Vec<usize>> = vec![Vec::with_capacity(2); 5]; // Store (player_id, bool_counts => false: 1, true: 2)
        // Add new_inferred_constraints
        // FIX: Can be 3 in the case of pile
        while let Some(single_true_group) = new_inferred_constraints.pop() {
            let alive_count = single_true_group.count_alive();
            // Something is wrong if this panics, all groups should have a single flag, not no flags
            let player_id = single_true_group.get_set_players().iter().position(|b| *b).unwrap();
            let card = single_true_group.card();
            match alive_count {
                1 => {
                    // One card known
                    if !self.inferred_constraints[player_id].contains(&card) {
                        log::trace!("");
                        log::trace!("=== add_subset_groups_recurse Inferred 1 === ");
                        log::trace!("add_sub_groups_recurse adding player considered: {}", player_id);
                        log::trace!("add_sub_groups_recurse adding 1 card single_flag_group: {}", single_true_group);
                        log::trace!("add_sub_groups_recurse Before self.public_constraints: {:?}", self.public_constraints);
                        log::trace!("add_sub_groups_recurse Before self.inferred_constraints: {:?}", self.inferred_constraints);
                        self.inferred_constraints[player_id].push(card);
                        debug_assert!(self.inferred_constraints[player_id].len() < 4, "S1");
                        card_changes[card as usize].push(player_id);
                        log::trace!("add_sub_groups_recurse After self.public_constraints: {:?}", self.public_constraints);
                        log::trace!("add_sub_groups_recurse After self.inferred_constraints: {:?}", self.inferred_constraints);
                        // TODO: Needs to prune the groups too...
                        // TODO: Adjust counts properly too... or remove inferred_counts
                    }
                },
                2 => {
                    if player_id == 6 {
                        let current_count = self.inferred_constraints[player_id].iter().filter(|c| **c == card).count() as u8;
                        // TODO: [OPTIMIZE] use saturating_sub here!
                        if alive_count > current_count {
                            let no_to_push = alive_count - current_count;
                            for _ in 0..no_to_push {
                                self.inferred_constraints[player_id].push(card);
                                debug_assert!(self.inferred_constraints[player_id].len() < 4, "S2");
                            }
                        }
                    } else {
                        // Both cards known
                        if self.inferred_constraints[player_id] != vec![card; 2] {
                            log::trace!("");
                            log::trace!("=== add_subset_groups_recurse Inferred 2 === ");
                            log::trace!("add_sub_groups_recurse adding player considered: {}", player_id);
                            log::trace!("add_sub_groups_recurse adding 2 cards single_flag_group: {}", single_true_group);
                            log::trace!("add_sub_groups_recurse Before self.public_constraints: {:?}", self.public_constraints);
                            log::trace!("add_sub_groups_recurse Before self.inferred_constraints: {:?}", self.inferred_constraints);
                            self.inferred_constraints[player_id].clear();
                            self.inferred_constraints[player_id].push(card);
                            self.inferred_constraints[player_id].push(card);
                            debug_assert!(self.inferred_constraints[player_id].len() < 3, "S3");
                            card_changes[card as usize].push(player_id);
                            log::trace!("add_sub_groups_recurse After self.public_constraints: {:?}", self.public_constraints);
                            log::trace!("add_sub_groups_recurse After self.inferred_constraints: {:?}", self.inferred_constraints);
                            // TODO: Needs to prune the groups too... some batch function that prunes groups based on changes?
                        }
                    }
                    // TODO: Adjust counts properly too... or remove inferred_counts
                },
                3 => {
                    if player_id == 6 {
                        let current_count = self.inferred_constraints[player_id].iter().filter(|c| **c == card).count() as u8;
                        if alive_count > current_count {
                            let no_to_push = alive_count - current_count;
                            for _ in 0..no_to_push {
                                self.inferred_constraints[player_id].push(card);
                                debug_assert!(self.inferred_constraints[player_id].len() < 3, "S4");
                            }
                        }
                    } else {
                        log::trace!("group: {}", single_true_group);
                        log::trace!("alive_count: {}", alive_count);
                        debug_assert!(false, "You really should not be here... there should only be alive_count of 2 or 1 for a single player!");
                    }
                    // TODO: Adjust counts properly too... or remove inferred_counts
                },
                _ => {
                    log::trace!("group: {}", single_true_group);
                    log::trace!("alive_count: {}", alive_count);
                    debug_assert!(false, "You really should not be here... there should only be alive_count of 2 or 1 for a single player!");
                }
            }
        }

        // batch prune
        // This is formatted like this because of some &mutable and immutable borrow issue with the compiler..
        let self_groups = [&mut self.group_constraints_amb, 
                                                                    &mut self.group_constraints_ass, 
                                                                    &mut self.group_constraints_cap, 
                                                                    &mut self.group_constraints_duk, 
                                                                    &mut self.group_constraints_con];
        // Ensures no inferred constraint makes a group redundant
        for (card_num, changes) in card_changes.iter().enumerate() {
            let mut i: usize = 0;
            'group_removal: while i < self_groups[card_num].len() {
                for &player_id in changes {
                    // If player flag is true and number of cards player now has
                    if self_groups[card_num][i].get_player_flag(player_id) {
                        let count_alive = self.inferred_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                        if count_alive >= self_groups[card_num][i].count_alive() {
                            self_groups[card_num].swap_remove(i);
                            continue 'group_removal;
                        }
                    }
                }
                i += 1;
            }
        }
        // recurse, new_groups should not be internally redundant, self.group_constraints should not be internally redundant
        if new_groups.iter().any(|v| !v.is_empty()) || 
        card_changes.iter().any(|v| !v.is_empty()) {
            return self.add_subset_groups_recurse(new_groups) || true;
        } else {
            // Returns true if any additions to self.group_constraints were made
            return bool_changes
        }
    }
    /// Recursively Adds groups from the union of Mutually Exclusive Groups
    /// - e.g. [1 1 0 0 0 0 0] 1 Duke and [0 0 1 1 0 0 0] 1 Duke => [1 1 1 1 0 0 0] 2 Duke
    /// 
    /// Helps the build the maximally informative unions
    /// - By combining 2 ME groups, adding the new groups in, and combining new groups with existing ones we eventually build up the set that properly combines all info within it
    /// - e.g. [1 1 0 0 0 0 0] 1 Duke, [0 0 1 1 0 0 0] 1 Duke, [0 1 1 0 0 0 0] 1 Duke, a simple union would miss out the inferred [1 1 1 1 0 0 0] 2 Duke 
    /// 
    /// Returns:
    /// - bool => Represents whether any additional information was added to self.group_constraints
    /// 
    /// Assumptions:
    /// Assumes reference_group_constraints is not internally redundant
    /// Assumes self.group_constraints is not internally redundant
    /// Assumes self has been compared with self already
    /// 
    /// Flow:
    /// - Compares self with reference_group_constraints and adds ME unions to new_groups
    /// - Compares reference_group_constraints with reference_group_constraints adds ME unions to new_groups
    ///     - Adds reference_group_constraints to self
    ///     - Recurses with new_groups being the new reference_group_constraints
    pub fn add_mutually_exclusive_unions_recurse(&mut self, mut reference_group_constraints: Vec<Vec<CompressedGroupConstraint>>) -> bool {
        log::trace!("In add_mutually_exclusive_unions_recurse");
        if reference_group_constraints.iter().all(|v| v.is_empty()) {
            log::trace!("exit add_mutually_exclusive_unions_recurse");
            return false
        }
        let mut new_group_constraints: Vec<Vec<CompressedGroupConstraint>> = vec![Vec::with_capacity(3); 5];
        // Find new groups and add to the new_group_constraints Vec
        for (card_num, group_constraints) in reference_group_constraints.iter().enumerate() {
            for reference_group_i in group_constraints.iter() {
                // Compare reference group with self.group_constraints
                for self_group in self.group_constraints_mut()[card_num].iter() {
                    if self_group.part_list_is_mut_excl(*reference_group_i) {
                        // Add in
                        log::trace!("");
                        log::trace!("=== add_mutually_exclusive_unions_recurse ref vs self");
                        log::trace!("add_mutually_exclusive_unions_recurse: group_i: {}", reference_group_i);
                        log::trace!("add_mutually_exclusive_unions_recurse: self_group: {}", self_group);
                        let new_group: CompressedGroupConstraint = CompressedGroupConstraint::mutually_exclusive_union(*reference_group_i, *self_group);
                        log::trace!("add_mutually_exclusive_unions_recurse: group_i + self_group = new_group: {}", new_group);
                        // TODO: Change new_group to a union
                        Self::non_redundant_push(&mut new_group_constraints[card_num], new_group);
                    }
                }
                // Compare reference group with reference group
                for reference_group_j in reference_group_constraints[card_num].iter() {
                    if reference_group_i.part_list_is_mut_excl(*reference_group_j) {
                        // Bitwise Union is a fast way to get their
                        log::trace!("");
                        log::trace!("=== add_mutually_exclusive_unions_recurse ref vs ref");
                        log::trace!("add_mutually_exclusive_unions_recurse: group_i: {}", reference_group_i);
                        log::trace!("add_mutually_exclusive_unions_recurse: group_j: {}", reference_group_j);
                        let new_group: CompressedGroupConstraint = CompressedGroupConstraint::mutually_exclusive_union(*reference_group_i, *reference_group_j);
                        log::trace!("add_mutually_exclusive_unions_recurse: group_i + group_j = new_group: {}", new_group);
                        Self::non_redundant_push(&mut new_group_constraints[card_num], new_group);
                    }
                }
                // Compare reference group with inferred constraints and public constraints
                for (player_id, &player_flag) in reference_group_i.get_set_players().iter().enumerate() {
                    if !player_flag {
                        let same_alive_card_count = self.inferred_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                        let same_dead_card_count = self.public_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                        if same_alive_card_count + same_dead_card_count > 0 {
                            log::trace!("");
                            log::trace!("=== add_mutually_exclusive_unions_recurse ref vs inferred & public");
                            log::trace!("add_mutually_exclusive_unions_recurse public_constraints: {:?}", self.public_constraints);
                            log::trace!("add_mutually_exclusive_unions_recurse inferred_constraints: {:?}", self.inferred_constraints);
                            log::trace!("add_mutually_exclusive_unions_recurse reference_group_i: {}, player_id: {}, player_flag: {}", reference_group_i, player_id, player_flag);
                            log::trace!("add_mutually_exclusive_unions_recurse same_alive_card_count: {}, same_dead_card_count: {}", same_alive_card_count, same_dead_card_count);
                            let mut new_group: CompressedGroupConstraint = *reference_group_i;
                            new_group.set_player_flag(player_id, true);
                            new_group.add_alive_count(same_alive_card_count);
                            new_group.add_dead_count(same_dead_card_count);
                            new_group.set_total_count(new_group.count_alive() + new_group.count_dead());
                            log::trace!("add_mutually_exclusive_unions_recurse new_group: {}", new_group);
                            Self::non_redundant_push(&mut new_group_constraints[card_num], new_group);
                        }
                    }
                }
            }
        }
        // Add reference group to self if not redundant
        let mut self_groups = self.group_constraints_mut();
        let mut bool_changes = false;
        for (card_num, group_constraints) in reference_group_constraints.iter_mut().enumerate() {
            while let Some(group) = group_constraints.pop() {
                bool_changes = Self::non_redundant_push_tracked(&mut self_groups[card_num], group).0 || bool_changes;
            }
        }
        // Self group is not internally redundant as all groups added were through non_redundant_push
        // New_group_constraints is not internally redundant, as it has been added through non_redundant_push
        // This satisfies the assumptions for recursion
        // Recurse
        return self.add_mutually_exclusive_unions_recurse(new_group_constraints) || bool_changes;
    }
    /// Adds mutually exclusive unions to self.group_constraints
    /// - mutually exclusive unions are unions of 2 mutually exclusive group_constraints
    ///     - May combine group_constraints with inferred_constraints that are mutually exclusive
    ///     - Combines only 2 groups, but iteratively does so until no new group needs to be added.
    ///       In doing so, will add all larger groups too, that could be done by combining multiple ME groups
    /// Returns:
    /// - bool => Represents whether any additional information was added to self.group_constraints
    /// 
    /// Assumptions:
    /// Assumes self.group_constraints is not internally redundant
    /// 
    /// Flow:
    /// === This Function ===
    /// - Compares self.group_constraints with self.group_constraints and adds mutually exclusive unions to new_groups
    /// - Starts recursion with new_groups as reference_group_constraint
    /// === Recursive Function ===
    /// - Recurses with new_groups being the reference group
    ///     - Compares self with new_groups and adds ME unions to new_new_groups
    ///     - Compares new_groups with new_groups adds ME unions to new_new_groups
    ///     - Adds new_groups to self
    ///     - Recurses with new_new_groups being the new reference group
    pub fn add_mutually_exclusive_unions(&mut self) -> bool {
        log::trace!("In add_mutually_exclusive_unions");
        if self.group_constraints().iter().all(|v| v.is_empty()) {
            log::trace!("exit add_mutually_exclusive_unions");
            return false
        }
        let mut new_group_constraints: Vec<Vec<CompressedGroupConstraint>> = vec![Vec::with_capacity(3); 5];
        // Find new groups and add to the new_group_constraints Vec
        let reference_group_constraints = self.group_constraints();
        for (card_num, group_constraints) in reference_group_constraints.iter().enumerate() {
            for reference_group_i in group_constraints.iter() {
                // Compare reference group (self) with reference group (self)
                for reference_group_j in reference_group_constraints[card_num].iter() {
                    if reference_group_i.part_list_is_mut_excl(*reference_group_j) {
                        // Bitwise Union is a fast way to get their
                        log::trace!("");
                        log::trace!("=== add_mutually_exclusive_unions ref vs ref");
                        log::trace!("add_mutually_exclusive_unions: group_i: {}, group_j: {}", reference_group_i, reference_group_j);
                        let new_group: CompressedGroupConstraint = CompressedGroupConstraint::mutually_exclusive_union(*reference_group_i, *reference_group_j);
                        log::trace!("add_mutually_exclusive_unions: group_i + group_j = new_group: {}", new_group);
                        Self::non_redundant_push(&mut new_group_constraints[card_num], new_group);
                    }
                }
                // Compare reference group with inferred constraints and public constraints
                for (player_id, &player_flag) in reference_group_i.get_set_players().iter().enumerate() {
                    if !player_flag {
                        let same_alive_card_count = self.inferred_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                        let same_dead_card_count: u8 = self.public_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                        if same_alive_card_count + same_dead_card_count > 0 {
                            let mut new_group: CompressedGroupConstraint = *reference_group_i;
                            // log::trace!("");
                            // log::trace!("=== add_mutually_exclusive_unions ref vs inferred & public");
                            // log::trace!("add_mutually_exclusive_unions public_constraints: {:?}", self.public_constraints);
                            // log::trace!("add_mutually_exclusive_unions inferred_constraints: {:?}", self.inferred_constraints);
                            // log::trace!("same_alive_card_count: {}", same_alive_card_count);
                            // log::trace!("same_dead_card_count: {}", same_dead_card_count);
                            // log::trace!("add_mutually_exclusive_unions reference_group_i: {}, player_id: {}, player_flag: {}", reference_group_i, player_id, player_flag);
                            new_group.set_player_flag(player_id, true);
                            new_group.add_alive_count(same_alive_card_count);
                            new_group.add_dead_count(same_dead_card_count);
                            new_group.set_total_count(new_group.count_alive() + new_group.count_dead());
                            log::trace!("Adding group: {}, same_alive_card_count: {}, same_dead_card_count: {}", new_group, same_alive_card_count, same_dead_card_count);
                            Self::non_redundant_push(&mut new_group_constraints[card_num], new_group);
                        }
                    }
                }
            }
        }
        // Self group is not internally redundant as all groups added were through non_redundant_push
        // New_group_constraints is not internally redundant, as it has been added through non_redundant_push
        // This satisfies the assumptions for recursion
        // Recurse
        return self.add_mutually_exclusive_unions_recurse(new_group_constraints);
    }
    /// Adds inferred constraints based on algo that checks cards in group "except" player
    /// Assumes:
    /// - The maximally informative group can be found through just finding the largest counts in the group excluding the player
    ///     - This should be met if add_subset and add_mutually_exclusive are done
    /// e.g.
    /// [0 1 0 0 0 1 1] 
    /// Alive: 3 Amb, 2 Cap, 1 Cont
    /// Dead: 1 Ass
    /// Player 5 cannot have Captain from a group that has both Captains outside 5 [0 1 0 0 0 0 1] 2 Captain
    /// If pile has 1 Captain,
    /// AMB = 3 - max holdable outside player 5
    /// max holdable = (2 + 3) - (1 Dead Ass) - (2 Captain)
    ///              = 2
    /// Player 5 therefore has 3 - 2 = 1 AMB
    /// or 3 + (1 Dead Ass) + 2 Captain from sub group - 5 spaces outside player
    /// We can just loop through every group after we have done subset group + mut excl union since we keep basically all the groups
    /// So i guess we could also infer at the end, if group count == 3?
    /// for some player i
    ///     find max holdable alive count and death outside that player
    ///     if condition met:
    ///         add to inferred group (if not already there)
    /// if something changed, recurse entire inferred
    ///     - Since adding inferred did not update all the groups
    /// Can we get more info from impossible groups
    /// Consider maximal subset redundance in future?
    ///     - subset but also single_flags considered
    ///     - both death and alive must be lower than or equal to it
    pub fn add_inferred_except_player(&mut self) -> bool {
        // TODO: [OPTIMIZE / THINK] Consider there is just a generalised subset prune, and you can just put this in subset prune!
        // TODO: [OPTIMIZE / THINK] I wonder if there would be groups implied by the case where its outside a group rather than a player...
        // TODO: [OPTIMIZE / THINK] IntSet to skip some looked over items i guess? or just not have redundant shit bro
        log::trace!("In add_inferred_except_player");
        let mut players: Vec<usize> = Vec::with_capacity(7);
        let mut bool_change = false;
        for i in 0..7 {
            // Only consider for players that can be inferred about
            if self.public_constraints[i].len() + self.inferred_constraints[i].len() < 2 {
                players.push(i);
            }
        }
        for (card_num, groups) in [&self.group_constraints_amb, 
            &self.group_constraints_ass, 
            &self.group_constraints_cap, 
            &self.group_constraints_duk, 
            &self.group_constraints_con]
            .iter().enumerate() {
            for group in groups.iter() {
                // If count == 1, no one else other than player inside
                // If count == 2, then it would have been inferrable with subset as there is only 1 group outside
                let group_alive_count = group.count_alive();
                if group.part_list_count() > 2 && group_alive_count > 1{
                    // for player 
                    // TODO: [REFACTOR]
                    let mut player_index: usize = 0;
                    while player_index < players.len() {
                        let player = players[player_index];
                        if group.get_player_flag(player) {
                            let mut part_list_excl_player: CompressedGroupConstraint = group.get_blank_part_list();
                            part_list_excl_player.set_player_flag(player, false);
                            // Gets maximal holdable number of cards in the group outside of the player

                            let mut maximal_alive_card_counts: [u8; 5] = [0; 5];
                            //  ==== Handles case where no groups in group_constraints
                            // TODO: [OPTIMIZE] Initialise an all counts_alive [[u8; 5]; 7] array outside of loop
                            //      Then gather number here
                            for temp_id in part_list_excl_player.iter_true_player_flags() {
                                for temp_card in self.inferred_constraints[temp_id].iter() {
                                    if *temp_card as usize != card_num {
                                        maximal_alive_card_counts[*temp_card as usize] += 1;
                                    }
                                }
                            }
                            //  ==== End Handles case where no groups in group_constraints
                            // TODO: [OPTIMIZE] with a filter on card_num_inner so the loop does not have the conditional
                            for (card_num_inner, complement_groups) in self.group_constraints().iter().enumerate() {
                                if card_num_inner != card_num {
                                    for complement_group in complement_groups.iter() {
                                        // See Assumptions! This requires add_subsets and add_mut excl unions to be ran for this to work
                                        if complement_group.part_list_is_subset_of(&part_list_excl_player) {
                                            log::trace!("complement group: {} is subset of group: {}", complement_group, part_list_excl_player);
                                            maximal_alive_card_counts[card_num_inner] = std::cmp::max(maximal_alive_card_counts[card_num_inner], complement_group.count_alive());
                                            log::trace!("maximal_alive_card_counts is now: {:?}", maximal_alive_card_counts);
                                        }
                                    }
                                }
                            }
                            // All dead other than player
                            log::trace!("player: {player} self.public_constraints before maximal_holdable_dead: {:?}", self.public_constraints);
                            let mut complement_maximal_holdable_dead: u8 = 0;
                            // TODO: [OPTIMIZE] Change to this
                            // let mut complement_maximal_holdable_dead: u8 = (0..7 as usize)
                            //     .filter(|i| group.get_player_flag(i) && i != player)
                            //     .map(|i| self.public_constraints[i].len() as u8)
                            //     .sum();
                            for i in 0..7 as usize {
                                if group.get_player_flag(i) && i != player {
                                    complement_maximal_holdable_dead += self.public_constraints[i].len() as u8;
                                }
                            }
                            // We have got the
                            let complement_maximal_holdable_alive = maximal_alive_card_counts.iter().sum::<u8>();
                            // All spaces other than player
                            let complement_maximal_holdable_spaces = part_list_excl_player.max_spaces();
                            // This should not overflow as spaces should always be > both
                            let max_free_spaces = complement_maximal_holdable_spaces - complement_maximal_holdable_dead - complement_maximal_holdable_alive; 
                            // log::trace!("=== add_inferred_except_player discovery ===");
                            // log::trace!("Parent Group: {}", group);
                            // log::trace!("part list excl player: {}, count: {}", part_list_excl_player, part_list_excl_player.part_list_count());
                            // log::trace!("Current Player: {}", player);
                            // log::trace!("Max_free_spaces: {} = complement_maximal_holdable_spaces: {} - complement_maximal_holdable_dead: {} - complement_maximal_holdable_alive: {}", max_free_spaces, complement_maximal_holdable_spaces, complement_maximal_holdable_dead, complement_maximal_holdable_alive);
                            // log::trace!("Complement_max_alive array: {:?}", maximal_alive_card_counts);
                            // log::trace!("Complement_max_dead: {}", complement_maximal_holdable_dead);
                            // log::trace!("Complement_max_alive: {}", complement_maximal_holdable_alive);
                            // log::trace!("group_alive_count: {} >? max_free_spaces{}", group_alive_count, max_free_spaces);
                            if group_alive_count > max_free_spaces {
                                let inferred_counts = group_alive_count - max_free_spaces;
                                let known_counts= self.inferred_constraints[player].iter().filter(|c| **c as usize == card_num).count() as u8;
                                log::trace!("inferred_counts: {} = group_alive_count: {} - max_free_spaces: {}", inferred_counts, group_alive_count, max_free_spaces);
                                if inferred_counts > known_counts {
                                    log::trace!("add_inferred_except_player DISCOVERED player: {player} has card: {:?}", Card::try_from(card_num as u8).unwrap());
                                    for _ in 0..(inferred_counts - known_counts) {
                                        self.inferred_constraints[player].push(Card::try_from(card_num as u8).unwrap());
                                        bool_change = true;
                                    }
                                    if player < 6 && self.public_constraints[player].len() + self.inferred_constraints[player].len() == 2 {
                                        // Removing player from search list if all is known
                                        // Alternatively u could just recurse here, because u need to recurse for the new inferred constraint anyways
                                        // [OPTIMIZE]
                                        if let Some(pos) = players.iter().position(|p| *p == player) {
                                            players.swap_remove(pos);
                                        }
                                    } else if player == 6 && self.inferred_constraints[player].len() == 3 {
                                        if let Some(pos) = players.iter().position(|p| *p == 6) {
                                            players.swap_remove(pos);
                                        }
                                    }
                                }
                            } 
                        }
                        player_index += 1;
                    }
                }
            }
        }
        bool_change
    }
    /// Adds inferred constraints based on algo that checks cards in group "except" player
    /// 
    /// Assumes maximally informative group is within the group_constraints
    /// Does not modify group_constraints
    /// Only adds inferred_group_constraints
    /// 
    /// Only checks for each player and is not dependent on groups in the group_constraints
    pub fn add_inferred_complement_of_player(&mut self) -> bool {
        // TODO: [OPTIMIZE / THINK] Consider there is just a generalised subset prune, and you can just put this in subset prune!
        // TODO: [OPTIMIZE / THINK] I wonder if there would be groups implied by the case where its outside a group rather than a player...
        // TODO: [OPTIMIZE / THINK] IntSet to skip some looked over items i guess? or just not have redundant shit bro
        log::trace!("In add_inferred_complement_player");
        let mut players: Vec<usize> = Vec::with_capacity(7);
        let mut bool_change = false;
        for i in 0..7 {
            // Only consider for players that can be inferred about
            if self.public_constraints[i].len() + self.inferred_constraints[i].len() < 2 {
                players.push(i);
            }
        }
        let mut remaining_alive_counts = self.dead_card_count();
        log::trace!("dead_card_count: {:?}", remaining_alive_counts);
        remaining_alive_counts.iter_mut().for_each(|x| *x = 3 - *x);
        // Creating alive_counts for each player
        let mut player_remaining_alive_counts: Vec<[u8; 5]> = vec![remaining_alive_counts.clone(); players.len()];
        let mut player_alive_counts_to_subtract: Vec<[u8; 5]> = vec![[0; 5]; players.len()]; 
        
        // Adding inferred_alive_cards that belong to other players
        for i in 0..7 as usize {
            for (index, player) in players.iter().enumerate() {
                if i != *player {
                    for card in self.inferred_constraints[i].iter() {
                        player_alive_counts_to_subtract[index][*card as usize] += 1;
                    }
                }
            }
        }
        log::trace!("Before Looping through group_constraints");
        log::trace!("public_constraints: {:?}", self.public_constraints);
        log::trace!("inferred_constraints: {:?}", self.inferred_constraints);
        log::trace!("remaining_alive_counts: {:?}", remaining_alive_counts);
        log::trace!("players: {:?}", players);
        log::trace!("player_alive_counts_to_subtract: {:?}", player_alive_counts_to_subtract);

        // Loop through every group, and add amount to subtract based on those alive
        for (card_num, groups) in [&self.group_constraints_amb, 
            &self.group_constraints_ass, 
            &self.group_constraints_cap, 
            &self.group_constraints_duk, 
            &self.group_constraints_con]
            .iter().enumerate() {
                // Early exit
                if remaining_alive_counts[card_num] == 0 {
                    continue;
                }
                for group in groups.iter() {
                    for (index, player) in players.iter().enumerate() {
                        if !group.get_player_flag(*player) {
                            player_alive_counts_to_subtract[index][group.card_num()] = player_alive_counts_to_subtract[index][group.card_num()].max(group.count_alive());
                        }
                    }
                }
            }
        // Subtract all alive from the remaining for each player
        for (remaining, subtract) in player_remaining_alive_counts
        .iter_mut()
        .zip(player_alive_counts_to_subtract.iter())
            {
                // TODO: WARN: ERROR: There is an attemt to subtract with overflow here
                for (r, s) in remaining.iter_mut().zip(subtract.iter()) {
                    *r = *r - *s;
                }
            }
        log::trace!("Before Looping through group_constraints");
        log::trace!("public_constraints: {:?}", self.public_constraints);
        log::trace!("inferred_constraints: {:?}", self.inferred_constraints);
        log::trace!("remaining_alive_counts: {:?}", remaining_alive_counts);
        log::trace!("players: {:?}", players);
        log::trace!("player_alive_counts_to_subtract: {:?}", player_alive_counts_to_subtract);
    
        
        // Check each and add inferred groups as necessary
        for (index, player) in players.iter().enumerate() {
            if 2 - self.public_constraints[*player].len() as u8 >= player_remaining_alive_counts[index].iter().sum::<u8>() {
                // If player lives >= remaining counts
                for (card_num, count) in player_remaining_alive_counts[index].iter().enumerate() {
                    match *count {
                        2 => {
                            //TODO: add debug_assert
                            if self.inferred_constraints[*player].contains(&Card::try_from(card_num as u8).unwrap()) {
                                if self.inferred_constraints[*player].len() == 1 {
                                    log::trace!("A: Adding card_num: {card_num} to player: {player}");
                                    self.inferred_constraints[*player].push(Card::try_from(card_num as u8).unwrap());
                                    bool_change = true;
                                }
                            } else {
                                debug_assert!(self.inferred_constraints[*player].len() == 0, "Trying to add 2 items will cause player to have > 3 cards");
                                log::trace!("B: Adding 2x card_num: {card_num} to player: {player}");
                                self.inferred_constraints[*player].push(Card::try_from(card_num as u8).unwrap());
                                self.inferred_constraints[*player].push(Card::try_from(card_num as u8).unwrap());
                                bool_change = true;
                            }
                        }
                        1 => {
                            if !self.inferred_constraints[*player].contains(&Card::try_from(card_num as u8).unwrap()) {
                                log::trace!("C: Adding x card_num: {card_num} to player: {player}");
                                self.inferred_constraints[*player].push(Card::try_from(card_num as u8).unwrap());
                                bool_change = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        log::trace!("END inferred_constraints: {:?}", self.inferred_constraints);
        bool_change
    }
    /// Adds inferred cards, based on knowing all remaining cards - a sub_group within
    /// We know all the possible cards that must be in the set of players whom at least 1 card are unknown
    /// e.g.
    ///     Public Constraints: [[Ambassador, Duke], [Assassin, Duke], [Ambassador, Assassin], [Captain, Captain], [], [Assassin, Duke], []]
    ///     Inferred Constraints: [[], [], [], [], [], [], [Contessa]]
    ///     We know players [0 0 0 0 1 0 1] must have 1 Ambassador 1 Captain 3 Contessa
    ///     
    ///     If we know also the following:
    ///         { Card: Contessa, Flags: [0 0 0 0 1 0 1], Single Card Flags: [0 0 0 0 1 0 0], 0 dead 2 alive 2 total}
    ///         { Card: Captain, Flags: [0 0 0 0 1 0 1], Single Card Flags: [0 0 0 0 1 0 0], 0 dead 1 alive 1 total}
    ///         { Card: Ambassador, Flags: [0 0 0 0 1 0 1], Single Card Flags: [0 0 0 0 1 0 0], 0 dead 1 alive 1 total}
    ///     
    ///     We can conclude that Player 4 has 1 Contessa, as it is their last card
    /// Currently only implemented for groups with 1 single_card_flag
    /// !!! Also 2 single_flag_groups of different cards, might imply be that both cards in a players' hand is part of the single_flag_group, so there may be more counted than expected
    /// !!! Perhaps this is only usable if a player with single_flag has RR only once since their last amb? else we need to expand the single_flag_group
    ///     Not quite, if the player has only 1 life then its obvious that for that it should be included
    /// TODO: [OPTIMIZE] I think it even does this for ridiculous outer groups like the group of everything in it
    /// TODO: [OPTIMIZE] Consider not having this run until later in the game, don't really need it if not much has been revealed yet
    pub fn add_inferred_remaining_negation(&mut self) -> bool {
        // Get the remaining card group + card counts
        let mut full_group_flags = CompressedGroupConstraint::zero();
        full_group_flags.set_player_flag(0, (self.public_constraints[0].len() + self.inferred_constraints[0].len()) != 2);
        full_group_flags.set_player_flag(1, (self.public_constraints[1].len() + self.inferred_constraints[1].len()) != 2);
        full_group_flags.set_player_flag(2, (self.public_constraints[2].len() + self.inferred_constraints[2].len()) != 2);
        full_group_flags.set_player_flag(3, (self.public_constraints[3].len() + self.inferred_constraints[3].len()) != 2);
        full_group_flags.set_player_flag(4, (self.public_constraints[4].len() + self.inferred_constraints[4].len()) != 2);
        full_group_flags.set_player_flag(5, (self.public_constraints[5].len() + self.inferred_constraints[5].len()) != 2);
        full_group_flags.set_player_flag(6, (self.public_constraints[6].len() + self.inferred_constraints[6].len()) != 3);
        
        let mut full_group_total_card_freq: [u8; 5] = [3; 5]; // All Alive counts in full_group_flags only
        let mut full_group_known_card_freq: [u8; 5] = [0; 5]; // All known Alive counts only
        let mut player_unknown_alive_count: [u8; 7] = [2, 2, 2, 2, 2, 2, 3]; // All unknown Alive counts only
        let mut player_lives: [u8; 7] = [2, 2, 2, 2, 2, 2, 3];
        for i in 0..6 as usize {
            for card in self.public_constraints[i].iter() {
                full_group_total_card_freq[*card as usize] -= 1;
                player_unknown_alive_count[i] -= 1;
                player_lives[i] -= 1;
            }
        }
        for i in 0..7 as usize {
            for card in self.inferred_constraints[i].iter() {
                if !full_group_flags.get_player_flag(i) {
                    full_group_total_card_freq[*card as usize] -= 1;
                } else {
                    full_group_known_card_freq[*card as usize] += 1;
                }
                player_unknown_alive_count[i] -= 1;
            }
        }
        // Store in HashSet
        // TODO: [OPTIMIZE] Change maybe to IntMap
        let capacity = (self.group_constraints_amb.len() + self.group_constraints_ass.len() + self.group_constraints_cap.len() + self.group_constraints_duk.len() + self.group_constraints_con.len() + 1) / 2;
        let mut groups_set: AHashSet<CompressedGroupConstraint> = AHashSet::with_capacity(capacity);
        // Have to compare every group with every other group 
        for (card_num_outer, card_groups_outer) in [&self.group_constraints_amb, &self.group_constraints_ass, &self.group_constraints_cap, &self.group_constraints_duk, &self.group_constraints_con].iter().enumerate() {
            if full_group_total_card_freq[card_num_outer] - full_group_known_card_freq[card_num_outer] == 0 {
                // Skip if u cannot possible get a negation for this as all the cards are already dead
                // Skip also if all the alive cards that could be known are already known, 
                // these groups will be included in the inner_groups for coherence
                log::trace!("add_inferred_remaining_negation full_group_flags: {:?}", full_group_flags);
                log::trace!("add_inferred_remaining_negation public constraint: {:?}", self.public_constraints);
                log::trace!("add_inferred_remaining_negation inferred constraint: {:?}", self.inferred_constraints);
                log::trace!("add_inferred_remaining_negation full_group_total_card_freq: {:?}", full_group_total_card_freq);
                log::trace!("add_inferred_remaining_negation full_group_known_card_freq: {:?}", full_group_known_card_freq);
                log::trace!("add_inferred_remaining_negation player_unknown_alive_count: {:?}", player_unknown_alive_count);
                log::trace!("add_inferred_remaining_negation continue full_group_total_card_freq[card_num_outer]: {:?}, full_group_known_card_freq[card_num_outer]: {:?}", full_group_total_card_freq[card_num_outer], full_group_known_card_freq[card_num_outer]);
                continue;
            }
            'outer: for group_outer in card_groups_outer.iter() {
                let group_key: CompressedGroupConstraint = group_outer.get_blank_part_list_and_single_card_flags();
                let mut full_group_minus_group_key_card_freq: [u8; 5] = [0; 5]; // Alive count
                // Can we skip entire card groups based on whats in full group?
                if group_outer.part_list_is_subset_of(&full_group_flags) && 
                // group_outer.has_single_card_flag_for_any_players_with_zero_counts(&player_unknown_alive_count) && // Check if single_card_flag is 1 for any of the players with unknown_alive_count > 0
                !groups_set.contains(&group_key) && // The above checks are alot faster than indexing a hashmap
                {
                    // Check if there is actually any space difference between full group and group outer
                    let mut bool_check = false;
                    for i in 0..7 as usize {
                        if full_group_flags.get_player_flag(i) {
                            if group_outer.get_player_flag(i) {
                                if group_outer.get_single_card_flag(i) && player_unknown_alive_count[i] == 2 {
                                    // player_unknown_alive_count[i] == 2 or player_lives[i] == 2
                                    bool_check = true;
                                    break
                                }
                            } else {
                                if player_unknown_alive_count[i] > 0 {
                                    bool_check = true;
                                    break
                                }
                            }

                        } 
                    }
                    bool_check
                }
                {
                    log::trace!("add_inferred_remaining_negation full_group_flags: {:?}", full_group_flags);
                    log::trace!("add_inferred_remaining_negation public constraint: {:?}", self.public_constraints);
                    log::trace!("add_inferred_remaining_negation inferred constraint: {:?}", self.inferred_constraints);
                    log::trace!("add_inferred_remaining_negation full_group_total_card_freq: {:?}", full_group_total_card_freq);
                    log::trace!("add_inferred_remaining_negation full_group_known_card_freq: {:?}", full_group_known_card_freq);
                    log::trace!("add_inferred_remaining_negation player_unknown_alive_count: {:?}", player_unknown_alive_count);
                    log::trace!("add_inferred_remaining_negation checking part_list_is_subset group_outer {:?}, full_group_flags: {:?}", group_outer, full_group_flags);
                    // Check if difference in available spaces is <= 3
                    //  full [1 1 1 0 0 0 0] current [1 1 0 0 0 0 0] => difference is number of unknown alive for that player
                    //  full [1 1 1 0 0 0 0] current [1 1 1 0 0 0 0] with single_flags [0 0 1 0 0 0 0] 
                    //      Player has 2 Lives 0 alive cards known => difference is 2 - 1 = 1 
                    //      Player has 2 Lives 1 alive cards known card => max difference is 2 - 1 = 1
                    //      Player has 2 Lives 2 alive cards known => no difference
                    //      Player has 1 Lives 0 alive cards known => difference = 2 - 1 = 1
                    //      Player has 1 Lives 1 alive cards known => difference = 2 - 2 = 0
                    //      These are all just unknown_alive cards
                    // Think this part should be min_diff
                    // let mut maximum_difference: u8 = 0;
                    // for player in 0..7 as usize {
                    //     if full_group_flags.get_player_flag(player)  {
                    //         maximum_difference += player_unknown_alive_count[player];
                    //     } 
                    //     // here if full_group flag is 0, group_key flag is 0, see conditionals above
                    //     if maximum_difference > 3 {
                    //         // log::trace!("add_inferred_remaining_negation max diff: {} > 3", maximum_difference);
                    //         // log::trace!("add_inferred_remaining_negation full_group_flags: {:?}", full_group_flags);
                    //         // log::trace!("add_inferred_remaining_negation player_unknown_alive_count: {:?}", player_unknown_alive_count);
                    //         continue 'outer;
                    //     }
                    // }
                    for (card_num_inner, card_groups_inner) in [&self.group_constraints_amb, &self.group_constraints_ass, &self.group_constraints_cap, &self.group_constraints_duk, &self.group_constraints_con].iter().enumerate() {
                        for group_inner in card_groups_inner.iter() {
                            if group_inner.part_list_is_subset_of(&group_outer) 
                            && group_outer.single_card_flags_is_subset_of(*group_inner) 
                            {
                                log::trace!("add_inferred_remaining_negation found outer_group: {}", group_outer);
                                log::trace!("add_inferred_remaining_negation found inner_group: {}", group_inner);
                                log::trace!("group_card_freq changed from: {:?}", full_group_minus_group_key_card_freq);
                                full_group_minus_group_key_card_freq[card_num_inner] = full_group_minus_group_key_card_freq[card_num_inner].max(group_inner.count_alive());
                                log::trace!("group_card_freq changed to: {:?}", full_group_minus_group_key_card_freq);
                            }
                            log::trace!("add_inferred_remaining_negation ignored outer_group: {}", group_outer);
                            log::trace!("add_inferred_remaining_negation ignored inner_group: {}", group_inner);
                        }
                    }
                    let mut group_key_max_alive_spaces: u8 = group_key.get_slots();
                    for player in group_key.iter_true_player_flags() {
                        for card in self.inferred_constraints[player].iter() {
                            full_group_minus_group_key_card_freq[*card as usize] = full_group_minus_group_key_card_freq[*card as usize].max(self.inferred_constraints[player].iter().filter(|c| **c == *card).count() as u8);
                        }
                        if group_key.get_single_card_flag(player) {
                            if self.public_constraints[player].len() == 2 {
                                group_key_max_alive_spaces -= 1;
                            }
                        } else {
                            group_key_max_alive_spaces -= self.public_constraints[player].len() as u8;
                        }
                    }
                    groups_set.insert(group_key);
                    // Might need to loop over inferred_constraints too, but im pretty sure add_mut_excl already includes those naturally
                    let group_key_card_freq = full_group_minus_group_key_card_freq.clone(); // Those inside group_key
                    // Modify group_card_freq to be cards in the all alive counts in full group -
                    for (found_alive_counts, all_alive_counts) in full_group_minus_group_key_card_freq.iter_mut().zip(full_group_total_card_freq.iter()) {
                        // found_alive_counts is assumed to include known alive counts (inferred_constraints) already
                        //  as inferred_constraints is assumed in be inside group_constraints due to add_mut_excl_groups
                        *found_alive_counts = *all_alive_counts - *found_alive_counts;
                    }
                    let group_key_alive_found: u8 = group_key_card_freq.iter().sum::<u8>();
                    log::trace!("add_inferred_remaining_negation Considering adding info");
                    log::trace!("add_inferred_remaining_negation public constraint: {:?}", self.public_constraints);
                    log::trace!("add_inferred_remaining_negation inferred constraint: {:?}", self.inferred_constraints);
                    log::trace!("add_inferred_remaining_negation full_group_total_card_freq: {:?}", full_group_total_card_freq);
                    log::trace!("add_inferred_remaining_negation full_group_known_card_freq: {:?}", full_group_known_card_freq);
                    log::trace!("add_inferred_remaining_negation player_unknown_alive_count: {:?}", player_unknown_alive_count);
                    log::trace!("add_inferred_remaining_negation player_lives: {:?}", player_lives);
                    log::trace!("add_inferred_remaining_negation group_key: {}", group_key);
                    log::trace!("add_inferred_remaining_negation full_group_minus_group_key_card_freq: {:?}", full_group_minus_group_key_card_freq);
                    log::trace!("add_inferred_remaining_negation group_key_alive_found: {:?}", group_key_alive_found);
                    log::trace!("add_inferred_remaining_negation group_key_card_freq: {:?}", group_key_card_freq);
                    // Counts of total number of cards inferred in the negation set
                    let negation_inferred_counts = full_group_minus_group_key_card_freq.iter().sum::<u8>();
                    log::trace!("Negation counts: {}", negation_inferred_counts);
                    // NOTE WARN TODO: [OPTIMIZE] This debug_assert is here and commented out as we still reach here with high negation_inferred_counts which is a 
                    //                              source of inefficiency
                    // debug_assert!(negation_inferred_counts < 4, "Seems like the max_difference continue 'outer; is not working as intended");
                    
                    // Compare group_keys with full_groups to see how many negation spaces there are
                    // This is the maximum number of spaces we can fill up from the negation
                    // e.g. full group [1 1 0 1 1 0 0] with lives [2 1 2 1 2 1 NA]
                    //      group_key  [1 1 0 1 1 0 0] with scf   [1 0 0 1 0 0 0]
                    //      negation spaces becomes               [1 0 0 0 0 0 0]
                    let mut max_negation_spaces: u8 = 0;
                    for player in group_key.iter_true_player_flags_and_single_card_flags() {
                        max_negation_spaces += player_lives[player] - 1;
                    }

                    // Find the inferred amount from the negation
                    match negation_inferred_counts {
                        1 => {
                            if max_negation_spaces == negation_inferred_counts {
                                for player in full_group_flags.iter_true_player_flags() {
                                    // if player_unknown_alive_count[player] == 1 {
                                        if !group_key.get_player_flag(player) {
                                            if let Some(card_num) = full_group_minus_group_key_card_freq.iter().position(|c| *c == negation_inferred_counts) {
                                                log::trace!("add_inferred_remaining_negation trying to add A1 card_num: {} for player: {}", card_num, player);
                                                let card_found= Card::try_from(card_num as u8).unwrap();
                                                if !self.inferred_constraints[player].contains(&card_found) {
                                                    // This has been observed once before
                                                    println!("A1");
                                                    self.inferred_constraints[player].push(card_found);
                                                    return true;
                                                }
                                            } 
                                        } else if group_key.get_single_card_flag(player) && group_key.get_player_flag(player) && player_lives[player] == 2 {
                                            if let Some(card_num) = full_group_minus_group_key_card_freq.iter().position(|c| *c == negation_inferred_counts) {
                                                log::trace!("add_inferred_remaining_negation trying to add A2 card_num: {} for player: {}", card_num, player);
                                                let card_found= Card::try_from(card_num as u8).unwrap();
                                                if !self.inferred_constraints[player].contains(&card_found) {
                                                    // Observed to have been used
                                                    self.inferred_constraints[player].push(card_found);
                                                    return true;
                                                }
                                            }
                                        }
                                    // }
                                }
                            } else {
                                // for player in full_group_flags.iter_true_player_flags() {
                                //     if player_unknown_alive_count[player] == 1 {
                                //         if !group_key.get_player_flag(player) {
                                //             if let Some(card_num) = group_card_freq.iter().position(|c| *c == negation_inferred_counts) {
                                //                 log::trace!("add_inferred_remaining_negation trying to add A3 card_num: {} for player: {}", card_num, player);
                                //                 let card_found= Card::try_from(card_num as u8).unwrap();
                                //                 self.inferred_constraints[player].push(card_found);
                                //                 return true;
                                //             } 
                                //         } 
                                //     }
                                // }
                            }
                            // The negation might be for a player not of single_card_flag tho
                            // Feels like this fails to account for the cases where [1 1 1 0 0 0 0] [1 1 0 0 0 0 0] where P0 has 1 live and P2 has 2 lives
                            // if group_key.single_card_flag_counts() == 1 {
                            //     for player in group_key.iter_true_player_flags_and_single_card_flags() {
                            //         // Expand for like more general difference
                            //         if let Some(card_num) = group_card_freq.iter().position(|c| *c == negation_inferred_counts) {
                            //             // [OPTIMIZE] u really just don;t need card_changes for this
                            //             // Not sure if need to update groups after this inferred_constraint is added...
                            //             log::trace!("add_inferred_remaining_negation trying to add A card_num: {} for player: {}", card_num, player);
                            //             let card_found= Card::try_from(card_num as u8).unwrap();
                            //             if !self.inferred_constraints[player].contains(&card_found) {
                            //                 self.inferred_constraints[player].push(card_found);
                            //                 return true;
                            //             }
                            //         }
                            //     }
                            // } else {
                            //     // Groups that have full_group flag 1, group_key flag 0 and no single_card_flags
                            //     // Yet to find case where this fixes smth
                            //     for player in 0..7 as usize {
                            //         if full_group_flags.get_player_flag(player) && !group_key.get_player_flag(player) {
                            //             if let Some(card_num) = group_card_freq.iter().position(|c| *c == negation_inferred_counts) {
                            //                 let card_found= Card::try_from(card_num as u8).unwrap();
                            //                 if !self.inferred_constraints[player].contains(&card_found) {
                            //                     log::trace!("add_inferred_remaining_negation trying to add B card_num: {} for player: {}", card_num, player);
                            //                     self.inferred_constraints[player].push(card_found);
                            //                     return true;
                            //                 }
                            //             }
                            //         } 
                            //     }
                            // }
                        },
                        2 => {
                            // If its just 1 player we infer 2 cards for
                            //      => give to that player
                            // If its split between 2 players
                            //      if boths cards are the same give to both players
                            match group_key.single_card_flag_counts() {
                                // 0 => {
                                //     // Groups that have full_group flag 1, group_key flag 0 and no single_card_flags
                                //     // Yet to observe case where this fixes smth, but we know if fixes stuff cos it makes less errors in general
                                //     // TODO: [TEST] document a case or so by commenting out
                                //     for player in full_group_flags.iter_true_player_flags() {
                                //         let mut bool_changed = false;
                                //         if !group_key.get_player_flag(player) && player_lives[player] == 2{
                                //             if player_lives[player] == 2 {
                                //                 for card_num in 0..5 as usize {
                                //                     match group_card_freq[card_num] {
                                //                         1 => {
                                //                             let card_found= Card::try_from(card_num as u8).unwrap();
                                //                             if !self.inferred_constraints[player].contains(&card_found) {
                                //                                 log::trace!("add_inferred_remaining_negation trying to add C1 card_num: {} for player: {}", card_num, player);
                                //                                 self.inferred_constraints[player].push(card_found);
                                //                                 bool_changed = true;
                                //                             }
                                //                         },
                                //                         2 => {
                                //                             let card_found= Card::try_from(card_num as u8).unwrap();
                                //                             let current_count: usize = self.inferred_constraints[player].iter().filter(|c| **c == card_found).count();
                                //                             match current_count {
                                //                                 0 => {
                                //                                     log::trace!("add_inferred_remaining_negation trying to add C2 x2 card_num: {} for player: {}", card_num, player);
                                //                                     self.inferred_constraints[player].push(card_found);
                                //                                     self.inferred_constraints[player].push(card_found);
                                //                                     bool_changed = true;
                                //                                 },
                                //                                 1 => {
                                //                                     log::trace!("add_inferred_remaining_negation trying to add C3 card_num: {} for player: {}", card_num, player);
                                //                                     self.inferred_constraints[player].push(card_found);
                                //                                     bool_changed = true;
                                //                                 },
                                //                                 _ => {}
                                //                             }
                                //                         }
                                //                         _ => {}
                                //                     } 
                                //                 }
                                //                 if bool_changed {
                                //                     return true;
                                //                 }
                                //             } else if player_lives[player] == 1 {
                                //                 for card_num in 0..5 {
                                //                     match group_card_freq[card_num] {
                                //                         2 => {
                                //                             // [OPTIMIZE] Don't think this contains check is actually needed as it should be unknown by construction, thus not inside
                                //                             let card_found= Card::try_from(card_num as u8).unwrap();
                                //                             if !self.inferred_constraints[player].contains(&card_found) {
                                //                                 log::trace!("add_inferred_remaining_negation trying to add C4 card_num: {} for player: {}", card_num, player);
                                //                                 self.inferred_constraints[player].push(card_found);
                                //                                 bool_changed = true;
                                //                             }
                                //                         },
                                //                         _ => {
                                //                             // unable to tell which card belongs to which player
                                //                             // Since the cards are both unknown alive cards
                                //                         },
                                //                     }
                                //                 }

                                //             }
                                //             if bool_changed {
                                //                 return true;
                                //             }
                                //         } 
                                //     }
                                // },
                                // 2 => {
                                //     // Could have earlier case present itself here, cos the single flag could be for somone
                                //         // => who has 1 live, so all the cards are in player where full_group_flag == 1. group_key_flag == 0
                                //     // single player + pile
                                //     // single player + n players
                                //     // single player + n players + pile
                                //     let mut bool_changed = false;
                                //     for player in full_group_flags.iter_true_player_flags() {
                                //         if !group_key.get_player_flag(player) {
                                //             match player_unknown_alive_count[player] {
                                //                 2 => {
                                //                     // all cards are for this particular player
                                //                     // Receivable spots is player_unknown_alive amount
                                //                     // which could be 2
                                //                     // TODO: [OPTIMIZE] do we need so many if contains check if we know player_unknown_alive_count anyways
                                //                     for card_num in 0..5 {
                                //                         match group_card_freq[card_num] {
                                //                             1 => {
                                //                                 let card_found= Card::try_from(card_num as u8).unwrap();
                                //                                 log::trace!("add_inferred_remaining_negation trying to add D1 card_num: {} for player: {}", card_num, player);
                                //                                 self.inferred_constraints[player].push(card_found);
                                //                                 bool_changed = true;
                                //                             },
                                //                             2 => {
                                //                                 let card_found= Card::try_from(card_num as u8).unwrap();
                                //                                 log::trace!("add_inferred_remaining_negation trying to add D2 x2 card_num: {} for player: {}", card_num, player);
                                //                                 self.inferred_constraints[player].push(card_found);
                                //                                 self.inferred_constraints[player].push(card_found);
                                //                                 return true;
                                //                             },
                                //                             _ => {},
                                //                         }
                                //                     }
                                //                     if bool_changed {
                                //                         return true;
                                //                     }
                                //                 },
                                //                 1 => {
                                //                     for card_num in 0..5 {
                                //                         match group_card_freq[card_num] {
                                //                             2 => {
                                //                                 let card_found= Card::try_from(card_num as u8).unwrap();
                                //                                 // unknown_alive_count is 1 so theres 1 space
                                //                                 log::trace!("add_inferred_remaining_negation trying to add D3 card_num: {} for player: {}", card_num, player);
                                //                                 self.inferred_constraints[player].push(card_found);
                                //                                 bool_changed = true;
                                //                             },
                                //                             _ => {},
                                //                         }
                                //                     }
                                //                 },
                                //                 _ => {},
                                //             }
                                //         } else if player_lives[player] == 2 && group_key.get_single_card_flag(player) {
                                //             match player_unknown_alive_count[player] {
                                //                 0 => {},
                                //                 _ => {
                                //                     for card_num in 0..5 {
                                //                         match group_card_freq[card_num] {
                                //                             2 => {
                                //                                 let card_found= Card::try_from(card_num as u8).unwrap();
                                //                                 log::trace!("add_inferred_remaining_negation trying to add D4 card_num: {} for player: {}", card_num, player);
                                //                                 self.inferred_constraints[player].push(card_found);
                                //                                 bool_changed = true; // cos negation_inferred_counts is only 2
                                //                             },
                                //                             _ => {},
                                //                         }
                                //                     }
                                //                 },
                                //             }
                                //         }
                                //     }
                                //     // Exit after player as there can be more than 1 player receiving a card
                                //     if bool_changed {
                                //         return true;
                                //     }
                                // },
                                _ => {
                                    // Could have earlier case present itself here, cos the single flag could be for somone
                                        // => who has 1 live, so all the cards are in player where full_group_flag == 1. group_key_flag == 0
                                    // single player + pile
                                    // single player + n players
                                    // single player + n players + pile
                                    // TODO: [REFACTOR] This might be the generalised version?... might handle cases where single_card_flag_counts == 2 does not
                                    let mut bool_changed = false;
                                    // if max_negation_spaces >= negation_inferred_counts {
                                    //     for player in full_group_flags.iter_true_player_flags() {
                                    //         if !group_key.get_player_flag(player) {
                                    //             match player_unknown_alive_count[player] {
                                    //                 2 => {
                                    //                     // all cards are for this particular player
                                    //                     // Receivable spots is player_unknown_alive amount
                                    //                     // which could be 2
                                    //                     // TODO: [OPTIMIZE] do we need so many if contains check if we know player_unknown_alive_count anyways
                                    //                     for card_num in 0..5 {
                                    //                         match group_card_freq[card_num] {
                                    //                             1 => {
                                    //                                 let card_found= Card::try_from(card_num as u8).unwrap();
                                    //                                 log::trace!("add_inferred_remaining_negation trying to add E1 card_num: {} for player: {}", card_num, player);
                                    //                                 println!("E1");
                                    //                                 self.inferred_constraints[player].push(card_found);
                                    //                                 bool_changed = true;
                                    //                             },
                                    //                             2 => {
                                    //                                 let card_found= Card::try_from(card_num as u8).unwrap();
                                    //                                 log::trace!("add_inferred_remaining_negation trying to add E2 x2 card_num: {} for player: {}", card_num, player);
                                    //                                 println!("E2");
                                    //                                 self.inferred_constraints[player].push(card_found);
                                    //                                 self.inferred_constraints[player].push(card_found);
                                    //                                 bool_changed = true;
                                    //                             },
                                    //                             _ => {},
                                    //                         }
                                    //                     }
                                    //                     if bool_changed {
                                    //                         return true;
                                    //                     }
                                    //                 },
                                    //                 1 => {
                                    //                     for card_num in 0..5 {
                                    //                         match group_card_freq[card_num] {
                                    //                             2 => {
                                    //                                 let card_found= Card::try_from(card_num as u8).unwrap();
                                    //                                 // unknown_alive_count is 1 so theres 1 space
                                    //                                 log::trace!("add_inferred_remaining_negation trying to add E3 card_num: {} for player: {}", card_num, player);
                                    //                                 println!("E3");
                                    //                                 self.inferred_constraints[player].push(card_found);
                                    //                                 bool_changed = true;
                                    //                             },
                                    //                             _ => {},
                                    //                         }
                                    //                     }
                                    //                 },
                                    //                 _ => {},
                                    //             }
                                    //         } else if player_lives[player] == 2 && group_key.get_single_card_flag(player) {
                                    //             match player_unknown_alive_count[player] {
                                    //                 0 => {},
                                    //                 _ => {
                                    //                     for card_num in 0..5 {
                                    //                         match group_card_freq[card_num] {
                                    //                             2 => {
                                    //                                 let card_found= Card::try_from(card_num as u8).unwrap();
                                    //                                 log::trace!("add_inferred_remaining_negation trying to add E4 card_num: {} for player: {}", card_num, player);
                                    //                                 println!("E4");
                                    //                                 self.inferred_constraints[player].push(card_found);
                                    //                                 bool_changed = true; // cos negation_inferred_counts is only 2
                                    //                             },
                                    //                             _ => {},
                                    //                         }
                                    //                     }
                                    //                 },
                                    //             }
                                    //         }
                                    //     }
                                    //     if bool_changed {
                                    //         return true;
                                    //     }
                                    // } else {
                                        // size of alive in group_keys - known alive in group_keys (number of slots remaining in group_keys)
                                        // negation_inferred cards (cards outside group_keys but in full_group)
                                        // max_negation_spaces (total number of people with single_flag_groups and external slots)
                                        log::trace!("add_inferred_remaining_negation 2 case empty slots?");
                                        log::trace!("add_inferred_remaining_negation 2 full_group: {}", full_group_flags);
                                        log::trace!("add_inferred_remaining_negation 2 group_key: {}", group_key);
                                        log::trace!("add_inferred_remaining_negation 2 group_key_max_alive_spaces: {}, group_key_alive_found: {}", group_key_max_alive_spaces, group_key_alive_found);
                                        if group_key_max_alive_spaces > group_key_alive_found {
                                            log::trace!("add_inferred_remaining_negation 2 case empty slots");
                                            let open_slots_in_group_keys = group_key_max_alive_spaces - group_key_alive_found;
                                            match open_slots_in_group_keys {
                                                0 => {},
                                                1 => {
                                                    // handle 3 card and 2 card to fill case
                                                    log::trace!("add_inferred_remaining_negation open_slots = 1 case group_card_freq: {:?}", full_group_minus_group_key_card_freq);
                                                    for card_num in 0..5 {
                                                        match full_group_minus_group_key_card_freq[card_num] {
                                                            3 => {
                                                                // TODO: Swap this check to outside
                                                                if max_negation_spaces == 1 {
                                                                    let mut only_open_single_flag_slot: Option<usize> = None;
                                                                    let mut counter_open_single_flag_slot: u8 = 0;
                                                                    for player in full_group_flags.iter_true_player_flags() {
                                                                        if !group_key.get_player_flag(player) {
                                                                            if self.inferred_constraints[player].len() < 2 {
                                                                                log::trace!("add_inferred_remaining_negation 2 A added card_num: {}", card_num);
                                                                                self.inferred_constraints[player].push(Card::try_from(card_num as u8).unwrap());
                                                                                return true;
                                                                            }
                                                                        } else {
                                                                            if group_key.get_single_card_flag(player) && player_lives[player] == 2 && self.inferred_constraints[player].len() == 0 {
                                                                                if only_open_single_flag_slot.is_none() {
                                                                                    only_open_single_flag_slot = Some(player);
                                                                                    counter_open_single_flag_slot += 1
                                                                                } else {
                                                                                    counter_open_single_flag_slot += 1
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                    // If no cases found, check single card cases
                                                                    if counter_open_single_flag_slot == 1 {
                                                                        if let Some(player) = only_open_single_flag_slot {
                                                                            log::trace!("add_inferred_remaining_negation 2 B added card_num: {}", card_num);
                                                                            self.inferred_constraints[player].push(Card::try_from(card_num as u8).unwrap());
                                                                            return true;
                                                                        }
                                                                    }
                                                                } else if max_negation_spaces == 2 {
                                                                    // TESTING
                                                                    // NEVER Encountered so far...
                                                                    // Will handle other cases first
                                                                    for player in full_group_flags.iter_true_player_flags() {
                                                                        // Think need to consider all the open spaces first
                                                                        // This may not be the best
                                                                        if !group_key.get_player_flag(player)  {
                                                                            if player_lives[player] == 2 {
                                                                                if self.inferred_constraints[player].len() == 0 {
                                                                                    log::trace!("add_inferred_remaining_negation 2 C added card_num: {}", card_num);
                                                                                    println!("Added 2CA");
                                                                                    self.inferred_constraints[player].push(Card::try_from(card_num as u8).unwrap());
                                                                                    self.inferred_constraints[player].push(Card::try_from(card_num as u8).unwrap());
                                                                                    return true;
                                                                                } else if self.inferred_constraints[player].len() == 1 {
                                                                                    log::trace!("add_inferred_remaining_negation 2 C added card_num: {}", card_num);
                                                                                    println!("Added 2CB");
                                                                                    self.inferred_constraints[player].push(Card::try_from(card_num as u8).unwrap());
                                                                                    return true;
                                                                                }
                                                                            } else {
                                                                                if self.inferred_constraints[player].len() < 2 {
                                                                                    log::trace!("add_inferred_remaining_negation 2 C added card_num: {}", card_num);
                                                                                    println!("Added 2CC");
                                                                                    self.inferred_constraints[player].push(Card::try_from(card_num as u8).unwrap());
                                                                                    return true;
                                                                                }
                                                                            }
                                                                        } 
                                                                    }
                                                                    // Handle single card flag case?
                                                                }
                                                            },
                                                            2 => {
                                                                // TODO: Swap this check to outside
                                                                if max_negation_spaces == 1 {
                                                                    let mut only_open_single_flag_slot: Option<usize> = None;
                                                                    let mut counter_open_single_flag_slot: u8 = 0;
                                                                    for player in full_group_flags.iter_true_player_flags() {
                                                                        if !group_key.get_player_flag(player) {
                                                                            if self.inferred_constraints[player].len() < 2 {
                                                                                log::trace!("add_inferred_remaining_negation 2 A added card_num: {}", card_num);
                                                                                self.inferred_constraints[player].push(Card::try_from(card_num as u8).unwrap());
                                                                                return true;
                                                                            }
                                                                        } else {
                                                                            if group_key.get_single_card_flag(player) && player_lives[player] == 2 && self.inferred_constraints[player].len() == 0 {
                                                                                if only_open_single_flag_slot.is_none() {
                                                                                    only_open_single_flag_slot = Some(player);
                                                                                    counter_open_single_flag_slot += 1
                                                                                } else {
                                                                                    counter_open_single_flag_slot += 1
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                    // If no cases found, check single card cases
                                                                    if counter_open_single_flag_slot == 1 {
                                                                        if let Some(player) = only_open_single_flag_slot {
                                                                            log::trace!("add_inferred_remaining_negation 2 B added card_num: {}", card_num);
                                                                            self.inferred_constraints[player].push(Card::try_from(card_num as u8).unwrap());
                                                                            return true;
                                                                        }
                                                                    }
                                                                }
                                                            },
                                                            _ => {},
                                                        }
                                                    }
                                                },
                                                2 => {
                                                    // handle 3 card to fill case
                                                },
                                                _ => {}
                                            }
                                        }

                                        // for player in full_group_flags.iter_true_player_flags() {
                                        //     if !group_key.get_player_flag(player) {
                                        //         match player_unknown_alive_count[player] {
                                        //             2 => {
                                        //                 // all cards are for this particular player
                                        //                 // Receivable spots is player_unknown_alive amount
                                        //                 // which could be 2
                                        //                 // TODO: [OPTIMIZE] do we need so many if contains check if we know player_unknown_alive_count anyways
                                        //                 for card_num in 0..5 {
                                        //                     match group_card_freq[card_num] {
                                        //                         1 => {
                                        //                             let card_found= Card::try_from(card_num as u8).unwrap();
                                        //                             log::trace!("add_inferred_remaining_negation trying to add E5 card_num: {} for player: {}", card_num, player);
                                        //                             self.inferred_constraints[player].push(card_found);
                                        //                             bool_changed = true;
                                        //                         },
                                        //                         2 => {
                                        //                             let card_found= Card::try_from(card_num as u8).unwrap();
                                        //                             log::trace!("add_inferred_remaining_negation trying to add E6 x2 card_num: {} for player: {}", card_num, player);
                                        //                             self.inferred_constraints[player].push(card_found);
                                        //                             self.inferred_constraints[player].push(card_found);
                                        //                             return true;
                                        //                         },
                                        //                         _ => {},
                                        //                     }
                                        //                 }
                                        //                 if bool_changed {
                                        //                     return true;
                                        //                 }
                                        //             },
                                        //             1 => {
                                        //                 for card_num in 0..5 {
                                        //                     match group_card_freq[card_num] {
                                        //                         2 => {
                                        //                             let card_found= Card::try_from(card_num as u8).unwrap();
                                        //                             // unknown_alive_count is 1 so theres 1 space
                                        //                             log::trace!("add_inferred_remaining_negation trying to add E7 card_num: {} for player: {}", card_num, player);
                                        //                             self.inferred_constraints[player].push(card_found);
                                        //                             bool_changed = true;
                                        //                         },
                                        //                         _ => {},
                                        //                     }
                                        //                 }
                                        //             },
                                        //             _ => {},
                                        //         }
                                        //     }
                                        // }
                                        // // Exit after player as there can be more than 1 player receiving a card
                                        // if bool_changed {
                                        //     return true;
                                        // }
                                    // }
                                },
                            }
                        },
                        3 => {
                            // If its just 1 player (pile) we infer 3 cards for
                            //      => give to the pile
                            // If shared between 2 players
                            //      If its split 2 cards, 1 card
                            //          => for the player that has 2 slots to receive
                            //             give the card that has count of 2 
                            //      If split 3 same cards,
                            //          => give to all players
                            // If shared between 3 players
                            //      If split 3 same cards,
                            //          => give to all players
                        },
                        _ => {
                            // debug_assert!(false, "you should not be here");
                            // unsafe {
                            //     unreachable_unchecked();
                            // }
                        },
                    }
                    
                }
            }
        }

        // Checking for possible cards to infer
        false
    }
    /// Assumes a maximally informative group is present
    fn generate_impossible_constraints(&mut self) {
        // TODO: [OPTIMIZE] this dead_cards and card_player_counts can be combined
        let mut dead_cards: [u8; 5] = [0; 5];
        let mut inferred_cards: [u8; 5] = [0; 5];
        self.public_constraints
            .iter()
            .flatten()
            .for_each(|&c| dead_cards[c as usize] += 1);
        self.inferred_constraints
            .iter()
            .flatten()
            .for_each(|&c| inferred_cards[c as usize] += 1);
        let mut dead_card_player_counts: [[u8; 7]; 5] = [[0; 7]; 5];
        let mut inferred_card_player_counts: [[u8; 7]; 5] = [[0; 7]; 5];
        self.public_constraints
        .iter()
        .enumerate()
        .for_each(|(player_id, v)| {
            for card in v.iter() {
                dead_card_player_counts[*card as usize][player_id] += 1;
            }
        });
        self.inferred_constraints
        .iter()
        .enumerate()
        .for_each(|(player_id, v)| {
            for card in v.iter() {
                inferred_card_player_counts[*card as usize][player_id] += 1;
            }
        });
        // TODO: OPTIMIZE? You probably could update each move
        self.impossible_constraints = [[false; 5]; 7];
        log::trace!("generate_impossible_constraints public_constraint: {:?}", self.public_constraints);
        log::trace!("generate_impossible_constraints inferred_constraint: {:?}", self.inferred_constraints);
        log::trace!("generate_impossible_constraints dead_cards: {:?}", dead_cards);
        // TODO: [OPTIMIZE], because there are too many groups, maybe only check for player-ids that are not dead players (or are eligible)
        for (card_num, group_constraints) in  [&self.group_constraints_amb, 
            &self.group_constraints_ass, 
            &self.group_constraints_cap, 
            &self.group_constraints_duk, 
            &self.group_constraints_con]
            .iter().enumerate() {
            for group in group_constraints.iter() {
                if group.count() == 3 { 
                    for player_id in group.iter_false_player_flags() {
                        // log::trace!("self.impossible_constraints group: {:?}", group);
                        // log::trace!("self.impossible_constraints B setting: {:?}, card: {:?}", player_id, card_num);
                        self.impossible_constraints[player_id][card_num] = true;
                    }
                } else if group.count_alive() + dead_cards[card_num] == 3 {
                    for player_id in group.iter_false_player_flags() {
                        // log::trace!("self.impossible_constraints group: {:?}", group);
                        // log::trace!("self.impossible_constraints C setting: {:?}, card: {:?}", player_id, card_num);
                        self.impossible_constraints[player_id][card_num] = true;
                    }
                } else {
                    // Since I don't always add all the groups
                    let mut count_outside_group: u8 = 0;
                    let mut player_flags: Vec<usize> = group.iter_false_player_flags().collect();
                    group.iter_false_player_flags().for_each(|player| {
                        count_outside_group += dead_card_player_counts[card_num][player];
                        count_outside_group += inferred_card_player_counts[card_num][player];
                        if inferred_card_player_counts[card_num][player] > 0 {
                            if let Some(pos) = player_flags.iter().position(|p| *p == player) {
                                player_flags.remove(pos);
                            }
                        }
                    });
                    if count_outside_group + group.count() == 3 {
                        for player_id in player_flags {
                            if !self.inferred_constraints[player_id].contains(&Card::try_from(card_num as u8).unwrap()) {
                                self.impossible_constraints[player_id][card_num] = true;
                            }
                        }
                    }
                }
            }
        }
        log::trace!("generate_impossible_constraints after group pass: {:?}", self.impossible_constraints);
        for card_num in 0..5 as usize{
            if dead_cards[card_num] == 3 {
                self.impossible_constraints[0][card_num] = true;
                self.impossible_constraints[1][card_num] = true;
                self.impossible_constraints[2][card_num] = true;
                self.impossible_constraints[3][card_num] = true;
                self.impossible_constraints[4][card_num] = true;
                self.impossible_constraints[5][card_num] = true;
                self.impossible_constraints[6][card_num] = true;
            }
        }
        log::trace!("generate_impossible_constraints after dead_card pass: {:?}", self.impossible_constraints);
        // TODO: [OPTIMIZE] not sure which order of loops is better
        for player_id in 0..7 {
            for card_num in 0..5 as usize {
                if dead_cards[card_num] + inferred_cards[card_num] == 3 {
                    if !self.inferred_constraints[player_id].contains(&Card::try_from(card_num as u8).unwrap()) {
                        self.impossible_constraints[player_id][card_num] = true;
                    }
                }
            }
        }
        log::trace!("generate_impossible_constraints after inferred_card pass: {:?}", self.impossible_constraints);
        for player_id in 0..6 {
            // Setting impossible constraints when player's entire hand is known
            if self.public_constraints[player_id].len() + self.inferred_constraints[player_id].len() == 2 {
                self.impossible_constraints[player_id] = [true; 5];
                for card in self.inferred_constraints[player_id].iter() {
                    self.impossible_constraints[player_id][*card as usize] = false;
                }
            }
        }
        // Setting impossible constraints when pile's entire hand is known
        if self.inferred_constraints[6].len() == 3 {
            self.impossible_constraints[6] = [true; 5];
            for card in self.inferred_constraints[6].iter() {
                self.impossible_constraints[6][*card as usize] = false;
            }
        }
        log::trace!("generate_impossible_constraints after full_hand pass: {:?}", self.impossible_constraints);
    }
    /// Temp name, checks if should include the incumbent group
    /// Full Group: The General large group for which we know all the cards of
    ///     => Should be a blank part list group
    /// Main Group: Current group we are trying to get information about
    ///     => Should be a blank part list group
    /// Incumbent Group: Group we are considering for inclusion in Main Group
    ///     => Is not a blank part list group
    /// 
    /// Returns true if
    ///     a) main group is subset of full_group, and
    ///     b) 
    // pub fn negation_check(full_group_flags: CompressedCollectiveConstraint, main_group: CompressedGroupConstraint, incumbent_group: CompressedCollectiveConstraint, player_unknown_alive_count: &[u8; 7]) -> bool {
    //     main_group.part_list_is_subset_of(&full_group_flags) && 
    //     // main_group.has_single_card_flag_for_any_players_with_zero_counts(player_unknown_alive_count) && // Check if single_card_flag is 1 for any of the players with unknown_alive_count > 0
    //     // If main_group has single_card_flag == 1, incumbent must have it as 1 too
    //     main_group.single_card_flags_is_subset_of(incumbent_group)
    // }
    /// Returns an array indexed by [player][card] that indicates if a player can have a particular card
    /// true => impossible
    /// false => possible
    pub fn generate_one_card_impossibilities_player_card_indexing(&self) -> [[bool; 5]; 7] {
        return self.impossible_constraints.clone();
        let mut impossible_cards: [[bool; 5]; 7] = [[false; 5]; 7];
        // This first part is here until the grouping part auto includes the inferred groups... probably in mutual exclusive groups
        // TODO: Remove this for loop
        let mut dead_cards: [u8; 5] = [0; 5];
        for i in 0..6 {
            for c in self.public_constraints[i].iter() {
                dead_cards[*c as usize] += 1;
            }
        }
        for player_id in 0..6 as usize{
            if self.public_constraints[player_id].len() + self.inferred_constraints[player_id].len() == 2 {
                impossible_cards[player_id] = [true; 5];
                for card in self.inferred_constraints[player_id].iter() {
                    impossible_cards[player_id][*card as usize] = false;
                }
            }
        }
        if self.inferred_constraints[6].len() == 3 {
            impossible_cards[6] = [true; 5];
            for card in self.inferred_constraints[6].iter() {
                impossible_cards[6][*card as usize] = false;
            }
        }
        // TODO: [OPTIMIZE], because there are too many groups, maybe only check for player-ids that are not dead players (or are eligible)
        'outer: for (card_num, group_constraints) in self.group_constraints().iter().enumerate() {
            for group in group_constraints.iter() {
                if group.count_dead() == 3 {
                    for player_id in 0..7 as usize {
                        log::trace!("impossible_cards group: {:?}", group);
                        log::trace!("impossible_cards A setting: {:?}, card: {:?}", player_id, card_num);
                        impossible_cards[player_id][card_num] = true;
                    }
                    continue 'outer;
                } else if group.count() == 3{
                    for player_id in 0..7 as usize {
                        if !group.get_player_flag(player_id) {
                            log::trace!("impossible_cards group: {:?}", group);
                            log::trace!("impossible_cards B setting: {:?}, card: {:?}", player_id, card_num);
                            impossible_cards[player_id][card_num] = true;
                        }
                    }
                } else if group.count_alive() == (3 - dead_cards[card_num]) {
                    for player_id in 0..7 {
                        if !group.get_player_flag(player_id) {
                            log::trace!("impossible_cards group: {:?}", group);
                            log::trace!("impossible_cards C setting: {:?}, card: {:?}", player_id, card_num);
                            impossible_cards[player_id][card_num] = true;
                        }
                    }
                }
            }
        }
        log::trace!("impossible_cards before: {:?}", &impossible_cards);
        let mut group_sets: AHashSet<CompressedGroupConstraint> = AHashSet::with_capacity(
            self.group_constraints_amb.len() + 
            self.group_constraints_ass.len() + 
            self.group_constraints_cap.len() + 
            self.group_constraints_duk.len() + 
            self.group_constraints_con.len()
        );
        for groups_i in [&self.group_constraints_amb, &self.group_constraints_ass, &self.group_constraints_cap, &self.group_constraints_duk, &self.group_constraints_con].iter() {
            for group_i in groups_i.iter() {
                let group_key = group_i.get_blank_part_list();
                if group_sets.contains(&group_key) {
                    continue;
                }
                let mut card_freq_dead: [u8; 5] = [0; 5];
                for i in 0..6 {
                    if group_key.get_player_flag(i) {
                        for c in self.public_constraints[i].iter() {
                            card_freq_dead[*c as usize] += 1;
                        }
                    }
                }
                let mut card_freq_alive: [u8; 5] = [0; 5];
                for (card_num_j, groups_j) in [&self.group_constraints_amb, &self.group_constraints_ass, &self.group_constraints_cap, &self.group_constraints_duk, &self.group_constraints_con].iter().enumerate() {
                    for group_j in groups_j.iter() {
                        if group_j.part_list_is_subset_of(&group_key) {
                            log::trace!("group_j: {} is subset of group_key: {}", group_j, group_key);
                            card_freq_alive[card_num_j] = card_freq_alive[card_num_j].max(group_j.count_alive());
                        }
                    }
                }
                let mut total_slots: u8 = 0;
                for i in 0..6 {
                    if group_key.get_player_flag(i) {
                        total_slots += 2;
                    }
                }
                if group_key.get_player_flag(6) {
                    total_slots += 3;
                }
                log::trace!("impossible_cards total_slots: {}", total_slots);
                if total_slots == (card_freq_alive.iter().sum::<u8>() + card_freq_dead.iter().sum::<u8>()) {
                    log::trace!("impossible_cards group_key: {}", group_key);
                    log::trace!("impossible_cards card_freq_alive: {:?}", card_freq_alive);
                    log::trace!("impossible_cards card_freq_dead: {:?}", card_freq_dead);
                    let zero_indices: Vec<usize> = card_freq_alive.iter()
                    .enumerate()
                    .filter(|&(_, c)| *c == 0)
                    .map(&|(i, _)| i)
                    .collect();
                    for player in 0..6 {
                        if group_key.get_player_flag(player) {
                            for card in zero_indices.iter() {
                                log::trace!("impossible_cards setting: {:?}, card: {:?}", player, card);
                                impossible_cards[player][*card] = true;
                            }
                        }
                    }
                    if group_key.get_player_flag(6) {
                        for card in zero_indices.iter() {
                            if card_freq_dead[*card] == 0 {
                                impossible_cards[6][*card] = true;
                            }
                        }
                    }
                }
                group_sets.insert(group_key);
            }
        }
        impossible_cards
    }
    /// This is currently broken
    pub fn generate_one_card_impossibilities_card_player_indexing(&mut self) -> [[bool; 7]; 5] {
        unimplemented!("Yet to integrate with self.impossible_constraints");
        let mut impossible_cards: [[bool; 7]; 5] = [[false; 7]; 5];
        for player_id in 0..7 as usize{
            if self.public_constraints[player_id].len() + self.inferred_constraints[player_id].len() == 2 {
                for card_num in 0..5 as usize{
                    impossible_cards[card_num][player_id] = true;
                }
                for card in self.public_constraints[player_id].iter() {
                    impossible_cards[*card as usize][player_id] = false;
                }
                for card in self.inferred_constraints[player_id].iter() {
                    impossible_cards[*card as usize][player_id] = false;
                }
            }
        }
        for (card_num, group_constraints) in self.group_constraints().iter().enumerate() {
            for group in group_constraints.iter() {
                if group.count() == 3 {
                    for player_id in 0..7 as usize {
                        if !group.get_player_flag(player_id) {
                            impossible_cards[card_num][player_id] = true;
                        }
                    }
                }
            }
        }
        todo!("FIX");
        impossible_cards
    }
    pub fn debug_panicker(&self) {

        for player in 0..6 {
            if self.public_constraints[player].len() + self.inferred_constraints[player].len() > 2 {
                self.printlog();
                debug_assert!(false, "invalid state reached!");
            }
        }
        if self.public_constraints[6].len() != 0 {
            self.printlog();
            debug_assert!(false, "invalid state reached!");
        }
        if self.inferred_constraints[6].len() > 3 {
            self.printlog();
            debug_assert!(false, "invalid state reached!");
        }
        let mut card_counts: [u8; 5] = [0; 5];
        for player in 0..7 {
            for card in self.public_constraints[player].iter() {
                card_counts[*card as usize] += 1;
            }
            for card in self.inferred_constraints[player].iter() {
                card_counts[*card as usize] += 1;
            }
        }
        if card_counts.iter().any(|amt| *amt > 3) {
            log::trace!("Too many cards! card_counts: {:?}", card_counts);
            self.printlog();
            debug_assert!(false, "invalid state reached!");
            
        }
    }
    pub fn check_three(&self) {
        for (card_num, groups) in self.group_constraints().iter().enumerate() {
            let mut temp_bool = true;
            for item in groups.iter() {
                if item.count() == 3 {
                    temp_bool = false;
                }
            }
            if temp_bool {
                panic!("card_num: {} lost its threes", card_num);
            }
        }
    } 
}