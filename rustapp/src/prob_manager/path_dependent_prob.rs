// Journey here
// Tried to iteratively find naive probability by filtering
// Concurrent and normal iteration times are around 0.1 s calculation of belief is around 0.1 seconds
// This is too long
// Tried instead to save into hashmap and store in bson

use crate::history_public::{Card, AOName, ActionObservation};
use super::path_dependent_collective_constraint::{ActionInfo, PathDependentCollectiveConstraint};

pub struct PathDependentCardCountManager {
    // a vec of constraints to push and pop
    // dead cards to push or pop
    // Will not locally store game history, jsut the constraint history
    constraint_history: Vec<PathDependentCollectiveConstraint>, // I think None is stored if there are no changes
    constraint_history_move_no: Vec<usize>, // TODO: determine if more optimal to put in constraint_history
    move_no: usize,
    // The shared LRU cache is maintained here and passed to each constraint.
    // cache: Rc<RefCell<LruCache<ConstraintKey, ActionMetaData>>>,
}
impl PathDependentCardCountManager {
    /// Constructor
    pub fn new() -> Self {
        PathDependentCardCountManager{
            constraint_history: Vec::with_capacity(120),
            constraint_history_move_no: Vec::with_capacity(120),
            move_no: 1, // First move will be move 1, post-increment this (saving 0 for initial game state)
        }
    }
    /// Returns everything to original state
    pub fn reset(&mut self) {
        self.constraint_history.clear();
        self.constraint_history.push(PathDependentCollectiveConstraint::game_start());
        self.move_no = 1;
    }
    /// Logs the constraint's log
    pub fn printlog(&self) {
        log::trace!("{}", format!("Constraint History Len{}", self.constraint_history.len()));
        if let Some(constraint) = self.constraint_history.last() {
            constraint.printlog();
        } else {
            log::trace!("Failed to print log, empty constraint history");;
        }
    }
    /// Gets the Latest Constraint
    pub fn latest_constraint(&self) -> &PathDependentCollectiveConstraint {
        // Should never pop() to 0
        self.constraint_history.last().unwrap()
    }
    /// Entrypoint for any action done, updates history accordingly
    /// Assumes knowledge of public information but not private information
    pub fn push_ao_public(&mut self, ao: &ActionObservation){

        // Handle different move types
        let ao_name = ao.name();
        if ao_name == AOName::Discard {
            match ao.no_cards() {
                1 => {
                    let temp_card = ao.cards().first().unwrap();
                    // unwrap is fine as the vec should never be size 0
                    let mut last_constraint = self.constraint_history.last().unwrap().clone();
                    let action_info = ActionInfo::Discard { discard: *temp_card };
                    last_constraint.add_move(ao.player_id() as u8, action_info);
                    // last_constraint.sort_unstable();
                    self.constraint_history.push(last_constraint);
                    self.constraint_history_move_no.push(self.move_no);
                    
                },
                2 => {
                    let temp_cards = ao.cards();
                    let mut last_constraint = self.constraint_history.last().unwrap().clone();
                    let action_info = ActionInfo::Discard { discard: temp_cards[0] };
                    last_constraint.add_move(ao.player_id() as u8, action_info);
                    let action_info = ActionInfo::Discard { discard: temp_cards[1] };
                    last_constraint.add_move(ao.player_id() as u8, action_info);
                    // last_constraint.sort_unstable();
                    self.constraint_history.push(last_constraint);
                    self.constraint_history_move_no.push(self.move_no);
                },
                _ => {
                    debug_assert!(false,"Unexpected no_cards case");
                }
            }
        } else if ao_name == AOName::RevealRedraw{
            let mut last_constraint = self.constraint_history.last().unwrap().clone();
            let action_info = ActionInfo::RevealRedraw { reveal: ao.card(), redraw: None };
            last_constraint.add_move(ao.player_id() as u8, action_info);
            // last_constraint.sort_unstable();
            self.constraint_history.push(last_constraint);
            self.constraint_history_move_no.push(self.move_no);
        } else if ao_name == AOName::ExchangeChoice {
            let mut last_constraint = self.constraint_history.last().unwrap().clone();
            let action_info = ActionInfo::ExchangeDrawChoice { draw: Vec::with_capacity(2), relinquish: Vec::with_capacity(2) };
            last_constraint.add_move(ao.player_id() as u8, action_info);
            self.constraint_history.push(last_constraint);
            self.constraint_history_move_no.push(self.move_no);
        }
        // shove move_no into CollectiveConstraint
        // post_increment: move_no is now the number of the next move
        self.move_no += 1;
    }
    /// Entrypoint for any action done, updates history accordingly
    /// Assumes knowledge of both public and private information
    pub fn push_ao_private(&mut self, ao: &ActionObservation){

        // Handle different move types
        unimplemented!();
        // shove move_no into CollectiveConstraint
        // post_increment: move_no is now the number of the next move
        self.move_no += 1;
    }
    /// pop latest move
    pub fn pop(&mut self) {
        if self.move_no > 1 {
            self.move_no -= 1;
            if self.constraint_history_move_no.last() == Some(&self.move_no) {
                self.constraint_history_move_no.pop();
                self.constraint_history.pop();
            }
        }
    }
    // ==== DELETE ALL BELOW BEFORE STABLE ===
    pub fn debug_panicker(&self) {
        self.latest_constraint().debug_panicker();
    }
    pub fn check_three(&self) {
        self.latest_constraint().check_three();
    }
}
