use crate::history_public::Card;
use super::compressed_group_constraint::CompressedGroupConstraint;

pub enum ActionInfo {
    Discard {discard: Card}, // Player | Card
    RevealRedraw {reveal: Card, redraw: Option<Card>}, // Player | Reveal Card | Redraw Option<Card>
    ExchangeDrawChoice {draw: Vec<Card>, relinquish: Vec<Card>}, // Player | Draw Vec<Card> | Return Vec<Card>
}

impl ActionInfo {
    /// All private information is known
    pub fn fully_known(&self) -> bool {
        match self {
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
}
pub struct SignificantAction {
    move_no: usize,
    player: u8,
    action_info: ActionInfo,
    meta_data: PathDependentMetaData,
}

impl SignificantAction {
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
}

// TODO: implement Into<PathDependentMetaData> for PathDependentCollectiveConstraint
pub struct PathDependentMetaData {
    public_constraints: Vec<Vec<Card>>,
    inferred_constraints: Vec<Vec<Card>>,
    impossible_constraints: [[bool; 5]; 7], // For each player store an array of bool where each index is a Card, this represents whether a player cannot have a card true => cannot
}

impl PathDependentMetaData {
    pub fn public_constraints(&self) -> &Vec<Vec<Card>> {
        &self.public_constraints
    }
    pub fn inferred_constraints(&self) -> &Vec<Vec<Card>> {
        &self.inferred_constraints
    }   
    pub fn impossible_constraints(&self) -> &[[bool; 5]; 7] {
        &self.impossible_constraints
    }   
}   

#[derive(Clone)]
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
    move_no: usize, // turn number
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
        Self {
            public_constraints,
            inferred_constraints,
            group_constraints_amb,
            group_constraints_ass,
            group_constraints_cap,
            group_constraints_duk,
            group_constraints_con,
            impossible_constraints,
            move_no: 0,
            history,
        }
    }
    fn regenerate_game_start(&mut self) {
        self.public_constraints = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::new()]; 
        self.inferred_constraints = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(3)]; 
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
        self.group_constraints_amb = group_constraints_amb;
        self.group_constraints_ass = group_constraints_ass;
        self.group_constraints_cap = group_constraints_cap;
        self.group_constraints_duk = group_constraints_duk;
        self.group_constraints_con = group_constraints_con;
        self.impossible_constraints = [[false; 5]; 7];
        // Not gonna reset move_no i guess
    }
    /// Create a method to understand for the latest discard/inferred card, whether some previous move's 
    /// hidden information is known
    pub fn lookback_0(&self) {

    }
    /// Create a method to understand for the some discard/inferred card, whether any previous move's 
    /// hidden information is known
    /// or actually maybe don't need this, but you can repeat on the latest card over and over?
    /// might need to look forward too?
    pub fn lookback_1(&mut self, index: usize) -> bool {
        // index is the index for history
        let considered_move: &SignificantAction = &self.history[index];
        match considered_move.action_info() {
            ActionInfo::RevealRedraw(reveal_considered, redraw_considered) => {
                match redraw_considered {
                    Some(redraw) => {

                    },
                    None => {
                        
                    }
                }
            },
            ActionInfo::Discard(discard_considered) => {
                let player_index: u8 = considered_move.player();
                for i in (0..index).rev() {
                    let action_data = &self.history[i];
                    let action_player = action_data.player();
                    if action_player == player_index {
                        // Case 0
                        // RR or AMB with 1 life
                        match action_data.action_info_mut() {
                            ActionInfo::RevealRedraw(reveal_current, redraw_current) => {
                                // 1 card unknown
                                if redraw_current.is_none() 
                                && action_data.public_constraints()[action_player as usize].len() + action_data.inferred_constraints()[action_player as usize].len() == 1 
                                && !action_data.public_constraints()[action_player as usize].contains(&discard_considered)
                                && !action_data.inferred_constraints()[action_player as usize].contains(&discard_considered)
                                {
                                    *redraw_current = Some(discard_considered);
                                    // TODO: recurse
                                    return true;
                                }
                            },
                            ActionInfo::ExchangeDrawChoice(draw, relinquish) => {
        
                            },
                            ActionInfo::Discard(discard) => {}
                        }
                    }
                    unimplemented!();
                }
            },
            _ => {
                unimplemented!();
            }
        }
    }
    /// Your generic lookback for an inferred constraint
    pub fn lookback_2(&self, index: usize) {

    }
    /// Recalculate current Constraint from scratch using history
    /// Can recursively call itself
    fn regenerate_path(&mut self) {
        self.regenerate_game_start();
        let mut skip_starting_empty_ambassador: bool = true;
        for action in self.history.iter() {
            // run the update for that action
            // if action is an starting empty ambassador, continue
            // Should just run 2 loops so you skip the branch really
            unimplemented!();
        }
    }
    /// handles pushing of LATEST moves only
    pub fn push_ao(&mut self) {

    }
    // Add other normal methods for inference
}