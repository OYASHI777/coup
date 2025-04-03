// Journey here
// Tried to iteratively find naive probability by filtering
// Concurrent and normal iteration times are around 0.1 s calculation of belief is around 0.1 seconds
// This is too long
// Tried instead to save into hashmap and store in bson

use crate::history_public::{Card, AOName, ActionObservation};
use super::path_dependent_collective_constraint::PathDependentCollectiveConstraint;

pub struct PathDependentCardCountManager {
    // a vec of constraints to push and pop
    // dead cards to push or pop
    // Will not locally store game history, jsut the constraint history
    game_start: PathDependentCollectiveConstraint, // To use a different PathDependentPathDependentCollectiveConstraint
    constraint_history: Vec<PathDependentCollectiveConstraint>, // I think None is stored if there are no changes
    constraint_history_move_no: Vec<usize>, // TODO: determine if more optimal to put in constraint_history
    move_number: usize,
    // The shared LRU cache is maintained here and passed to each constraint.
    // cache: Rc<RefCell<LruCache<ConstraintKey, ActionMetaData>>>,
}
impl PathDependentCardCountManager {
    /// Constructor
    pub fn new() -> Self {
        PathDependentCardCountManager{
            game_start: PathDependentCollectiveConstraint::game_start(),
            constraint_history: Vec::with_capacity(120),
            constraint_history_move_no: Vec::with_capacity(120),
            move_number: 1, // First move will be move 1, post-increment this (saving 0 for initial game state)
        }
    }
    /// Returns everything to original state
    pub fn reset(&mut self) {
        self.constraint_history = Vec::with_capacity(120);
        self.move_no = 1;
    }
    /// Logs the constraint's log
    pub fn printlog(&self) {
        log::trace!("{}", format!("Constraint History Len{}", self.constraint_history.len()));
        if let Some(constraint) = self.constraint_history.last() {
            constraint.printlog();
        } else {
            self.game_start.printlog();
        }
    }
    /// Gets the Latest Constraint
    pub fn latest_constraint(&self) -> &PathDependentCollectiveConstraint {
        self.constraint_history.last().unwrap_or(&self.game_start)
    }
    /// Entrypoint for any action done, updates history accordingly
    /// Assumes knowledge of public information but not private information
    pub fn push_ao_public(&mut self, ao: &ActionObservation){

        // Handle different move types
        unimplemented!();
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
        self.move_no -= 1;
        if self.constraint_history_move_no.last() == Some(&self.move_no) {
            self.constraint_history_move_no.pop();
            self.constraint_history.pop();
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
