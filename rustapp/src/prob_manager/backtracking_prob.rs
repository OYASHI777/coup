// Journey here
// Tried to iteratively find naive probability by filtering
// Concurrent and normal iteration times are around 0.1 s calculation of belief is around 0.1 seconds
// This is too long
// Tried instead to save into hashmap and store in bson

// TODO: REFACTOR ActionInfo and ActionInfoName to BacktrackManager or its own file
use crate::history_public::{Card, AOName, ActionObservation};
use super::backtracking_collective_constraints::{ActionInfo, ActionInfoName};
// TODO: Store also a version of constraint_history but split by players
// TODO: Improve analysis interface when using the manager... using last_constraint then the analysis is very clunky
// So it is easier to know the first time a player does something
// May be useful later
pub struct BackTrackCardCountManager<C> 
    where
        C: CoupConstraint,
{
    // a vec of constraints to push and pop
    // dead cards to push or pop
    // Will not locally store game history, jsut the constraint history
    private_player: Option<usize>,
    constraint_history: Vec<C>, // I think None is stored if there are no changes
    constraint_history_move_no: Vec<usize>, // TODO: determine if more optimal to put in constraint_history
    move_no: usize,
    // The shared LRU cache is maintained here and passed to each constraint.
    // cache: Rc<RefCell<LruCache<ConstraintKey, ActionMetaData>>>,
}
impl<C: CoupConstraint> BackTrackCardCountManager<C> {
    /// Constructor
    pub fn new() -> Self {
        let mut constraint_history = Vec::with_capacity(120);
        // constraint_history.push(C::game_start());
        let mut constraint_history_move_no = Vec::with_capacity(120);
        // constraint_history_move_no.push(0);
        Self {
            private_player: None,
            constraint_history,
            constraint_history_move_no,
            move_no: 1, // First move will be move 1, post-increment this (saving 0 for initial game state)
        }
    }
    // /// Constructor
    // pub fn new() -> Self {
    //     let mut constraint_history = Vec::with_capacity(120);
    //     constraint_history.push(C::game_start());
    //     let mut constraint_history_move_no = Vec::with_capacity(120);
    //     constraint_history_move_no.push(0);
    //     Self {
    //         private_player: None,
    //         constraint_history,
    //         constraint_history_move_no,
    //         move_no: 1, // First move will be move 1, post-increment this (saving 0 for initial game state)
    //     }
    // }
    /// Adding private player starting hand
    pub fn start_public(&mut self) {
        self.constraint_history.push(C::game_start_public());
        self.constraint_history_move_no.push(0);
        self.move_no = 1;
    }
    /// Adding private player starting hand
    pub fn start_private(&mut self, player: usize, cards: &[Card; 2]) {
        self.private_player = Some(player);
        // TODO: Add cards
        self.constraint_history.push(C::game_start_private(player, cards));
        self.constraint_history_move_no.push(0);
        self.move_no = 1;
    }
    /// Returns everything to original state
    pub fn reset(&mut self) {
        self.private_player = None;
        self.constraint_history.clear();
        // self.constraint_history.push(C::game_start());
        self.constraint_history_move_no.clear();
        // self.constraint_history_move_no.push(0);
        self.move_no = 1;
    }
    /// Logs the constraint's log
    pub fn printlog(&self) {
        log::trace!("{}", format!("Constraint History Len{}", self.constraint_history.len()));
        log::trace!("PathDependentCardCountManager history_move_no: {:?}", self.constraint_history_move_no);
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
        // TODO: might need to seperate exchangedraw and exchangechoice for private!
        // Handle different move types
        let ao_name = ao.name();
        if ao_name == AOName::Discard {
            match ao.no_cards() {
                1 => {
                    let temp_card = ao.cards().first().unwrap();
                    // unwrap is fine as the vec should never be size 0
                    let mut last_constraint = self.constraint_history.last().unwrap().clone();
                    let action_info = ActionInfo::Discard { discard: *temp_card };
                    log::trace!("Adding move discard 1");
                    last_constraint.add_move(ao.player_id() as u8, action_info);
                    // last_constraint.sort_unstable();
                    self.constraint_history.push(last_constraint);
                    self.constraint_history_move_no.push(self.move_no);
                    
                },
                2 => {
                    let temp_cards = ao.cards();
                    let mut last_constraint = self.constraint_history.last().unwrap().clone();
                    let action_info = ActionInfo::Discard { discard: temp_cards[0] };
                    log::trace!("Adding move discard 2");
                    last_constraint.add_move(ao.player_id() as u8, action_info);
                    let action_info = ActionInfo::Discard { discard: temp_cards[1] };
                    log::trace!("Adding move discard 2");
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
            let action_info = ActionInfo::RevealRedraw { reveal: ao.card(), redraw: None, relinquish: None };
            log::trace!("Adding move RevealRedraw");
            last_constraint.add_move(ao.player_id() as u8, action_info);
            // last_constraint.sort_unstable();
            self.constraint_history.push(last_constraint);
            self.constraint_history_move_no.push(self.move_no);
        } else if ao_name == AOName::ExchangeChoice {
            let mut last_constraint = self.constraint_history.last().unwrap().clone();
            let action_info = ActionInfo::ExchangeDrawChoice { draw: Vec::with_capacity(2), relinquish: Vec::with_capacity(2) };
            log::trace!("Adding move ExchangeChoice");
            last_constraint.add_move(ao.player_id() as u8, action_info);
            self.constraint_history.push(last_constraint);
            self.constraint_history_move_no.push(self.move_no);
        }
        // shove move_no into CollectiveConstraint
        // post_increment: move_no is now the number of the next move
        self.move_no += 1;
    }
    /// Entrypoint for any action done, updates history accordingly
    /// Assumes knowledge of private information
    pub fn push_ao_private(&mut self, ao: &ActionObservation){
        // TODO: might need to seperate exchangedraw and exchangechoice for private!
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
                self.constraint_history_move_no.push(self.move_no);
            },
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
                let mut last_constraint = self.constraint_history.last().unwrap().clone();
                let action_info = ActionInfo::RevealRedraw { reveal: *reveal, redraw: Some(*redraw), relinquish: None };
                last_constraint.add_move(*player_id as u8, action_info);
                // last_constraint.sort_unstable();
                self.constraint_history.push(last_constraint);
                self.constraint_history_move_no.push(self.move_no);
            },
            ActionObservation::ExchangeDraw { player_id, card } => {
                todo!("Change the ActionInfo to split Draw and Choice")
            },
            ActionObservation::ExchangeChoice { player_id, no_cards, hand, relinquish } => {
                todo!("Change the ActionInfo to split Draw and Choice")
            },
            _ => {},
        }
        // shove move_no into CollectiveConstraint
        // post_increment: move_no is now the number of the next move
        self.move_no += 1;
    }
    /// Entrypoint for any action done, updates history accordingly
    /// Assumes knowledge of both public and private information
    pub fn push_ao(&mut self, player: usize, action_info: &ActionInfo){
        let mut last_constraint = self.constraint_history.last().unwrap().clone();
        last_constraint.add_move(player as u8, action_info.clone());
        // shove move_no into CollectiveConstraint
        // post_increment: move_no is now the number of the next move
        self.constraint_history.push(last_constraint);
        self.constraint_history_move_no.push(self.move_no);
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
}


impl<C> CoupConstraintAnalysis for BackTrackCardCountManager<C>
where
    C: CoupConstraint + CoupConstraintAnalysis,
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