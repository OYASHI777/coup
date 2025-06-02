// Journey here
// Tried to iteratively find naive probability by filtering
// Concurrent and normal iteration times are around 0.1 s calculation of belief is around 0.1 seconds
// This is too long
// Tried instead to save into hashmap and store in bson

// TODO: REFACTOR ActionInfo and ActionInfoName to BacktrackManager or its own file
use crate::history_public::{Card, AOName, ActionObservation};
use super::backtracking_collective_constraints::{ActionInfo, ActionInfoName, BacktrackMetaData};
use super::coup_const::MAX_GAME_LENGTH;
// TODO: Store also a version of constraint_history but split by players
// TODO: Improve analysis interface when using the manager... using last_constraint then the analysis is very clunky
// So it is easier to know the first time a player does something
// May be useful later
pub struct BackTrackCardCountManager 
{
    private_player: Option<usize>,
    constraint_history: Vec<BacktrackMetaData>, 
    move_no_history: Vec<usize>, // TODO: determine if more optimal to put in constraint_history
    player_history: Vec<u8>,
    action_history: Vec<ActionInfo>,
    move_no: usize,
}
impl BackTrackCardCountManager {
    /// Constructor
    pub fn new() -> Self {
        let mut constraint_history = Vec::with_capacity(MAX_GAME_LENGTH);
        // constraint_history.push(C::game_start());
        let mut move_no_history = Vec::with_capacity(MAX_GAME_LENGTH);
        // move_no_history.push(0);
        let player_history = Vec::with_capacity(MAX_GAME_LENGTH);
        let action_history = Vec::with_capacity(MAX_GAME_LENGTH);
        Self {
            private_player: None,
            constraint_history,
            move_no_history,
            player_history,
            action_history,
            move_no: 1, // First move will be move 1, post-increment this (saving 0 for initial game state)
        }
    }
    /// Adding private player starting hand
    pub fn start_public(&mut self) {
        let start_public = BacktrackMetaData::start_public();
        self.constraint_history.push(start_public);
        self.player_history.push(7); // basically None
        self.action_history.push(ActionInfo::StartInferred);
        self.move_no_history.push(0);
        self.move_no = 1;
    }
    /// Adding private player starting hand
    pub fn start_private(&mut self, player: usize, cards: &[Card; 2]) {
        self.private_player = Some(player);
        let start_private = BacktrackMetaData::start_private(player, cards);
        self.constraint_history.push(start_private);
        self.player_history.push(7); // basically None
        self.action_history.push(ActionInfo::StartInferred);
        self.move_no_history.push(0);
        self.move_no = 1;
    }
    /// Returns everything to original state
    pub fn reset(&mut self) {
        self.private_player = None;
        self.constraint_history.clear();
        self.move_no_history.clear();
        self.player_history.clear();
        self.action_history.clear();
        self.move_no = 1;
    }
    /// Logs the constraint's log
    pub fn printlog(&self) {
        log::trace!("{}", format!("Constraint History Len{}", self.constraint_history.len()));
        log::trace!("PathDependentCardCountManager history_move_no: {:?}", self.move_no_history);
        log::trace!("PathDependentCardCountManager move_no: {:?}", self.move_no);
        if let Some(constraint) = self.constraint_history.last() {
            constraint.printlog();
        } else {
            log::trace!("Failed to print log, empty constraint history");;
        }
    }
    /// Gets the Latest Constraint
    pub fn latest_constraint(&self) -> &C {
        // Should never pop() to 0
        self.constraint_history.last().unwrap()
    }
    pub fn latest_constraint_mut(&mut self) -> &mut C {
        // Should never pop() to 0
        self.constraint_history.last_mut().unwrap()
    }
    /// Entrypoint for any action done, updates history accordingly
    /// Assumes knowledge of public information but not private information
    pub fn push_ao_public(&mut self, ao: &ActionObservation){
        // Handle different move types
        todo!("inline last_constraint.add_move");
        match ao {
            ActionObservation::Discard { player_id, card, no_cards } => {
                // Assumes no_cards is either 1 or 2 only
                let mut last_constraint = self.constraint_history.last().unwrap().clone();
                let action_info = ActionInfo::Discard { discard: card[0] };
                last_constraint.add_move(*player_id as u8, action_info);
                if *no_cards == 2 {
                    let action_info = ActionInfo::Discard { discard: card[1] };
                    last_constraint.add_move(*player_id as u8, action_info);
                }
                self.constraint_history.push(last_constraint);
                self.move_no_history.push(self.move_no);
            },
            ActionObservation::RevealRedraw { player_id, reveal, .. } => {
                let mut last_constraint = self.constraint_history.last().unwrap().clone();
                let action_info = ActionInfo::RevealRedraw { reveal: *reveal, redraw: None, relinquish: None };
                log::trace!("Adding move RevealRedraw");
                last_constraint.add_move(*player_id as u8, action_info);
                self.constraint_history.push(last_constraint);
                self.move_no_history.push(self.move_no);
            },
            ActionObservation::ExchangeDraw { player_id, .. } => {
                let mut last_constraint = self.constraint_history.last().unwrap().clone();
                let action_info = ActionInfo::ExchangeDraw { draw: Vec::with_capacity(2) };
                log::trace!("Adding move ExchangeChoice");
                last_constraint.add_move(*player_id as u8, action_info);
                self.constraint_history.push(last_constraint);
                self.move_no_history.push(self.move_no);
            },
            ActionObservation::ExchangeChoice { player_id, .. } => {
                let mut last_constraint = self.constraint_history.last().unwrap().clone();
                let action_info = ActionInfo::ExchangeChoice { relinquish: Vec::with_capacity(2) };
                log::trace!("Adding move ExchangeChoice");
                last_constraint.add_move(*player_id as u8, action_info);
                self.constraint_history.push(last_constraint);
                self.move_no_history.push(self.move_no);
            },
            _ => {},
        }
        // shove move_no into CollectiveConstraint
        // post_increment: move_no is now the number of the next move
        self.move_no += 1;
    }
    /// Entrypoint for any action done, updates history accordingly
    /// Assumes knowledge of private information
    pub fn push_ao_private(&mut self, ao: &ActionObservation){
        todo!("inline last_constraint.add_move");
        // Handle different move types
        match ao {
            ActionObservation::Discard { player_id, card, no_cards } => {
                // Assumes no_cards is either 1 or 2 only
                let mut last_constraint = self.constraint_history.last().unwrap().clone();
                let action_info = ActionInfo::Discard { discard: card[0] };
                last_constraint.add_move(*player_id as u8, action_info);
                if *no_cards == 2 {
                    let action_info = ActionInfo::Discard { discard: card[1] };
                    last_constraint.add_move(*player_id as u8, action_info);
                }
                self.constraint_history.push(last_constraint);
                self.move_no_history.push(self.move_no);
            },
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
                let mut last_constraint = self.constraint_history.last().unwrap().clone();
                let action_info = ActionInfo::RevealRedraw { reveal: *reveal, redraw: Some(*redraw), relinquish: None };
                last_constraint.add_move(*player_id as u8, action_info);
                // last_constraint.sort_unstable();
                self.constraint_history.push(last_constraint);
                self.move_no_history.push(self.move_no);
            },
            ActionObservation::ExchangeDraw { player_id, card } => {
                let mut last_constraint = self.constraint_history.last().unwrap().clone();
                let action_info = ActionInfo::ExchangeDraw { draw: card.to_vec() };
                last_constraint.add_move(*player_id as u8, action_info);
                self.constraint_history.push(last_constraint);
                self.move_no_history.push(self.move_no);
            },
            ActionObservation::ExchangeChoice { player_id, relinquish } => {
                let mut last_constraint = self.constraint_history.last().unwrap().clone();
                let action_info = ActionInfo::ExchangeChoice { relinquish: relinquish.to_vec() };
                last_constraint.add_move(*player_id as u8, action_info);
                self.constraint_history.push(last_constraint);
                self.move_no_history.push(self.move_no);
            },
            _ => {},
        }
        // shove move_no into CollectiveConstraint
        // post_increment: move_no is now the number of the next move
        self.move_no += 1;
    }
    /// pop latest move
    pub fn pop(&mut self) {
        if self.move_no > 1 {
            self.move_no -= 1;
            if self.move_no_history.last() == Some(&self.move_no) {
                self.constraint_history.pop();
                self.move_no_history.pop();
                self.player_history.pop();
                self.action_history.pop();
            }
        }
    }
}


impl CoupConstraintAnalysis for BackTrackCardCountManager
{
    fn public_constraints(&self) -> &Vec<Vec<Card>> {
        self.latest_constraint().public_constraints()
    }

    fn sorted_public_constraints(&mut self) -> &Vec<Vec<Card>> {
        self.latest_constraint_mut().sorted_public_constraints()
    }

    fn inferred_constraints(&mut self) -> &Vec<Vec<Card>> {
        self.latest_constraint_mut().inferred_constraints()
    }

    fn sorted_inferred_constraints(&mut self) -> &Vec<Vec<Card>> {
        self.latest_constraint_mut().sorted_inferred_constraints()
    }

    fn player_impossible_constraints(&mut self) -> &[[bool; 5]; 7] {
        self.latest_constraint_mut().player_impossible_constraints()
    }

    fn player_impossible_constraints_paired(&mut self) -> &[[[bool; 5]; 5]; 7] {
        self.latest_constraint_mut().player_impossible_constraints_paired()
    }

    fn player_impossible_constraints_triple(&mut self) -> &[[[bool; 5]; 5]; 5] {
        self.latest_constraint_mut().player_impossible_constraints_triple()
    }

    fn player_can_have_card_alive(&self, player: u8, card: Card) -> bool {
        self.latest_constraint().player_can_have_card_alive(player, card)
    }

    fn player_can_have_cards_alive(&self, player: u8, cards: &Vec<Card>) -> bool {
        self.latest_constraint().player_can_have_cards_alive(player, cards)
    }
}

impl CoupConstraint for BackTrackCardCountManager {
    fn game_start_public() -> Self {
        todo!("Get from collective constraint");
    }
    fn game_start_private(player: usize, cards: &[Card; 2]) -> Self{
        todo!("Get from collective constraint");
    }

    fn add_move(&mut self, player_id: u8, action: ActionInfo) {
        todo!("Get from collective constraint");
    }

    fn printlog(&self){
        todo!("Get from collective constraint");
    };
}

/// A trait providing the interface for a constraint
pub trait CoupConstraint: Clone {
    /// Initializes the state at beginning of the game
    fn game_start_public() -> Self;
    /// Initializes the state at beginning of the game
    fn game_start_private(player: usize, cards: &[Card; 2]) -> Self;

    /// Records a public move into the constraint.
    fn add_move(&mut self, player_id: u8, action: ActionInfo);

    /// Emit debug info about the constraint.
    fn printlog(&self);
}

pub trait CoupConstraintAnalysis {
    /// Returns reference to latest Public Constraints
    fn public_constraints(&self) -> &Vec<Vec<Card>>;
    /// Returns reference to latest sorted Public Constraints
    fn sorted_public_constraints(&mut self) -> &Vec<Vec<Card>>;
    /// Returns reference to latest Inferred Constraints
    fn inferred_constraints(&mut self) -> &Vec<Vec<Card>>;
    /// Returns reference to latest sorted Inferred Constraints
    fn sorted_inferred_constraints(&mut self) -> &Vec<Vec<Card>>;
    /// Returns reference to array[player][card] storing whether a player can have a card alive
    fn player_impossible_constraints(&mut self) -> &[[bool; 5]; 7];
    /// Returns reference to array[player][card_i][card_j] storing whether a player can have a card_i and card_j alive
    fn player_impossible_constraints_paired(&mut self) -> &[[[bool; 5]; 5]; 7];
    /// Returns reference to array[card_i][card_j][card_k] storing whether pile can have card_i, card_j, and card_k
    fn player_impossible_constraints_triple(&mut self) -> &[[[bool; 5]; 5]; 5];
    /// Returns true if player can have a particular card alive
    fn player_can_have_card_alive(&self, player: u8, card: Card) -> bool;
    /// Returns true if player can have a collection of cards alive
    fn player_can_have_cards_alive(&self, player: u8, cards: &Vec<Card>) -> bool;
} 