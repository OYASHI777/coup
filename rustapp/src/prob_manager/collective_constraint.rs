use crate::history_public::Card;
use super::{compressed_group_constraint::CompressedGroupConstraint, constraint::GroupConstraint};

// [FIRST GLANCE PRIORITY] Let death, revealredraw, ambassador mechanisms handle redundancies. Let seperate method do inference.
// [FIRST GLANCE PRIORITY]      - 3S generate_inferred_constraints => create this, probably will need to remove redundant groups when inferred constraints are added, use reveal?
// [FIRST GLANCE PRIORITY]      - 4A peek_pile and or swap => to think about how to account for private ambassador info. Add all into inferred, prune then swap based on private info? (private info mix) 
// [FIRST GLANCE PRIORITY]      - 5S All card combos: POSTULATE: clearly if A is possible, B is possible unless player can have only either A or B but not both
// [FIRST GLANCE PRIORITY]      - 0B Combine ME Union and sub union additions into single function
// [FIRST GLANCE PRIORITY]      - ?  Consider overall design flow from reveal to end, to ensure it makes sense for a sufficient unopt 1st version
// [FIRST GLANCE PRIORITY]      - 6S If it works consider storing impossibilities externally / not storing at all, seperate group_constraints with 3 to a different Vec?
// [FIRST GLANCE PRIORITY]      - 6S Dead count can be stored externally to this and generated when required
// [FIRST GLANCE PRIORITY]      - 6S inferred counts can generated when required instead of always being allocated
// [FIRST GLANCE PRIORITY] Consider if counts should be stored at all
// TODO: [CHECK] Data structure, should I still store counts?
// TODO: [OPTIMIZE] If you can maintain non internal redundancy throughout, no need self.redundant_prune()
// TODO: [OPTIMIZE] Why do you have full groups with less than 3 total => redundant
// TODO: [OPTIMIZE] Why do you have groups with same info just with 1 for single flags => redundant
// TODO: [OPTIMIZE / THINK] In revealredraw we dont remove groups with 3 because they are the parent group that needs to be kept for inference, but this leads to too many parent groups and slows down subset union
//      - Do we even need this feature? Its there for now cos i thot its needed to infer other groups
//      - Actually prevents some errors
//      - If we do need a total 3 group, we only need the one with the smallest part list
// TODO: [OPTIMIZE / THINK] In add_subset_groups from [1 1 1 1 1 1] I add all -1 subsets to a newgroup, should I check if they are redundant before adding in?
//      - Im thinking this might be more efficient than blindly adding all in, then generating more from that, you can prune the branch early on
// TODO: [IMPLEMENT / THINK] I think in add_subset_groups u can remove a person if that person has impossible alive constraint
// TODO: [OPTIMIZE / THINK] Must you do subset_groups removing 1 person at a time? Cos there are people that u could just immediately remove, like fully dead peopl?
// TODO: [OPTIMIZE / THINK] Add add_subset_groups to reveal and death, so no need to run inferred?
//      - Figure out if everything will still be represented
//      - Maybe ME in redraw? hmm
// TODO: mutually exclusive group additions should consider unions between individual players too? else sometimes we miss out on the 3 of a kind, when inferred are added
//      - wait should this already consider unions with individual players? in ME Union
// TODO: Consider that RevealRedraw multiple times provides hidden info that might be missed.
// TODO: [HANDLE IMPOSSIBLE CASE] Consider case when group is fully filled
// TODO: [CHECK] redraw_inferred_adjustment => determine if relevant/efficient, can compare against amb which works fine
// TODO: IDEA maybe reveal_group_adjustment is not needed as it does what add_inferred_groups would do
// TODO: IDEA what is a ME union, are there groups we can avoid adding, or make redundant to save space?
//      - Like perhaps you don't really need a union with an individual that doenst have the same card? idk
// HMM: I realise if a group has all but 1 slot known, [1 1 0 0 0 0 0] 1 Duke 2 Contessa, All parties cant have both Cap and Ass
// HMM: If a player has 1 known card, it obviously precludes many combos
// TODO: Add the complement group to all that is known => might need to add the FullGroup => also to add to Compressed (i guess complements with <=3 players / total unknown slots <?)
//      - Or maybe complements of fully known players?
//      - Dont implement this, complements should be fully implied in add_subsets
//      - Complements are useful for impossible cases more so than inferred constraints, in that regard, only groups containing a not fully known player is useful
//      - If you do implement it, groups can be built up from a ME way...
// [IMPLEMENTATION NOTE]: Redundancy is currently not subset based but equality based, this allows all info to be used.
// [FIRST GLANCE PRIORITY] Consider making a private constraint, to contain players' private information, to generate the public, and each players' understanding all at once
// [FIRST GLANCE PRIORITY] Add inferred impossible cards for each player? Then just check inferred joint else all but impossible cards to generate?
// [FIRST GLANCE PRIORITY] Consider processing all new items to add with redundant checks in bulk
// [FIRST GLANCE PRIORITY] Check redundant addition process. New item can make more than 1 item redundant. e.g. [1 0 0 0 0 0 0] 2 cards makes both [1 0 1 0 0 0 0] 1 cards and [1 1 0 0 0 0 0] 1 cards redundant

#[derive(Clone)]
/// A struct that helps in card counting. Stores all information known about cards by a particular player.
pub struct CompressedCollectiveConstraint {
    // TODO: [OPTIMIZE] Consider if can just combine public and joint constraints
    // public_constraints:[[Option<Card>; 2]; 6], // Stores all the dead cards of dead players, None are all behind
    public_constraints: Vec<Vec<Card>>, // Stores all the dead cards of dead players, None are all behind
    // inferred_constraints:[[Option<Card>; 2]; 6], // Stores all the dead cards of dead players 
    inferred_constraints: Vec<Vec<Card>>, // Stores all the inferred cards of alive players 
    // [ALT] TODO: Change group_constraints to by card so Vec<Vec<CompressedGroupConstraint>> ... maybe remove Card from it? or make this an object?
    group_constraints_amb: Vec<CompressedGroupConstraint>,
    group_constraints_ass: Vec<CompressedGroupConstraint>,
    group_constraints_cap: Vec<CompressedGroupConstraint>,
    group_constraints_duk: Vec<CompressedGroupConstraint>,
    group_constraints_con: Vec<CompressedGroupConstraint>,
    impossible_constraints: [[bool; 5]; 7], // For each player store an array of bool where each index is a Card, this represents whether a player cannot have a card true => cannot
    dead_card_count: [u8; 5], // each index represents the number of dead cards for the Card enum corresponding to that index
    inferred_card_count: [u8; 5], // each index represents the number of inferred cards for the Card enum
}
/// Constructors, Gettors, Simple Checks
impl CompressedCollectiveConstraint {
    /// Constructor that returns an empty CompressedCollectiveConstraint
    pub fn new() -> Self {
        let public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::new()]; 
        let inferred_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(2),Vec::with_capacity(3)]; 
        let group_constraints_amb: Vec<CompressedGroupConstraint> = Vec::with_capacity(5);
        let group_constraints_ass: Vec<CompressedGroupConstraint> = Vec::with_capacity(5);
        let group_constraints_cap: Vec<CompressedGroupConstraint> = Vec::with_capacity(5);
        let group_constraints_duk: Vec<CompressedGroupConstraint> = Vec::with_capacity(5);
        let group_constraints_con: Vec<CompressedGroupConstraint> = Vec::with_capacity(5);
        let impossible_constraints: [[bool; 5]; 7] = [[false; 5]; 7];
        let dead_card_count: [u8; 5] = [0; 5];
        let inferred_card_count: [u8; 5] = [0; 5];
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
            dead_card_count,
            inferred_card_count,
        }
    }
    /// Constructor that returns an CompressedCollectiveConstraint at start of game
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
        let dead_card_count: [u8; 5] = [0; 5];
        let inferred_card_count: [u8; 5] = [0; 5];
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
            dead_card_count,
            inferred_card_count,
        }
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
    #[inline]
    pub fn player_is_alive(&self, player_id: usize) -> bool {
        self.public_constraints[player_id].len() < 2
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
    // TODO: [TEST] this with random game generator
    // TODO: [OPTIMIZE] Perhaps can use bits to do this faster?
    /// "Is Group Complement of self's public constraints and joint constraints?"
    /// [DEPRECATING]if player 0, hasa dead card, knowing every other player has ata least 2 of that card, means player  0 cant have that card, and its not redundant
    /// Tells us if information in a group is exactly mutually exclusive from information in public_constraint and joint_constraint
    /// Group is mutually exclusive if all public_constraints and joint_constraints that have the same Card as the GroupConstraint's Card
    /// apply to players not represented in the participation list (false)
    /// By "exactly" mutual exclusive, we mean that all players must be represented in the public_constraint, joint_constraint and GroupConstraint
    /// This is why its called "Complement" because we want to know if a particular group is the set complement of the information known in the 
    /// public and joint constraints.
    /// We compare only participation list and Card
    pub fn is_complement_of_pcjc(&self, group: &CompressedGroupConstraint) -> bool {
        // TODO: Consider creating a version that includes inferred knowledge
        let participation_list: [bool; 7] = group.get_list();
        // TODO: [OPTIMIZE] Just loop unroll this
        for player in 0..6 as usize {
            // [COMBINE SJ]
            if participation_list[player] || 
            self.public_constraints[player].contains(&group.card()){
                continue;
            }
            return false
        }
        // If you reach here, its basically true just dependent on the center pile (player 6)
        panic!("Deprecated");
        participation_list[6]
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
    pub fn sort_unstable(&mut self) {
        for vec in self.public_constraints.iter_mut() {
            vec.sort_unstable();
        }
        for vec in self.inferred_constraints.iter_mut() {
            vec.sort_unstable();
        }
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
    pub fn player_alive_card_count(&self, player_id: usize, card: Card) -> u8 {
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
    pub fn player_alive_card_counts(&self, player_id: usize) -> [u8; 5] {
        let mut output: [u8; 5] = [0; 5];
        // [COMBINE SJ]
        for card in self.inferred_constraints[player_id].iter() {
            output[*card as usize] += 1;
        }
        output
    }
    pub fn dead_card_count(&self) -> &[u8; 5] {
        &self.dead_card_count
    }
    /// Returns number of a player's cards are known, i.e. Dead or inferred
    pub fn player_cards_known(&self, player_id: usize) -> u8 {
        (self.public_constraints[player_id].len() + self.inferred_constraints[player_id].len()) as u8
    }
}
/// Adds a public constraint without pruning group constraints that are redundant
impl CompressedCollectiveConstraint {
    /// Adds to tracked public constraints
    /// NOTE:
    /// - Only adds a dead player
    /// - Does not assume anything
    /// - Does not consider the informational state of collective constraint
    pub fn add_dead_player_constraint(&mut self, player_id : usize, card: Card) {
        debug_assert!(self.dead_card_count[card as usize] < 3, "Too many cards in dead_card_count for card: {:?}, found: {}", card, self.dead_card_count[card as usize]);
        debug_assert!(self.player_is_alive(player_id), "Cannot add more dead cards to player that is already dead!, Current player public_constraint len: {}", self.public_constraints[player_id].len());
        log::trace!("In add_dead_player_constraint");
        self.dead_card_count[card as usize] += 1;
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
            return true
        }
        false
    }
    /// Adds to tracked inferred constraints
    pub fn add_inferred_player_constraint(&mut self, player_id: usize, card: Card) {
        debug_assert!(player_id < 6, "Use proper player_id thats not pile");
        debug_assert!(self.inferred_constraints[player_id].len() < 2, "Adding inferred knowledge to fully known player!");
        self.inferred_card_count[card as usize] += 1;
        self.inferred_constraints[player_id].push(card);
    }
    /// Removes a specific card from the inferred player constraints if it exists
    /// NOTE:
    ///     - Only subtracts from inferred_card_count if card actually exists
    pub fn subtract_inferred_player_constraints(&mut self, player_id: usize, card: Card) {
        // [COMBINE SJ]
        // if self.inferred_constraints[player_id][1] == Some(card){
        //     self.inferred_constraints[player_id][1] = None;
        //     self.inferred_card_count[card as usize] -= 1;
        // } else if self.inferred_constraints[player_id][0] == Some(card) {
        //     self.inferred_constraints[player_id][0] = self.inferred_constraints[player_id][1];
        //     self.inferred_constraints[player_id][1] = None;
        //     self.inferred_card_count[card as usize] -= 1;
        // }
        if let Some(pos) = self.inferred_constraints[player_id].iter().position(|c| *c == card) {
            self.inferred_constraints[player_id].swap_remove(pos);
            self.inferred_card_count[card as usize] -= 1;
        }
    }
    /// Removes a specific card from the inferred pile constraints if it exists
    // pub fn subtract_inferred_pile_constraint(&mut self, card: Card) {
    //     if self.inferred_pile_constraints[card as usize] > 0 {
    //        self.inferred_pile_constraints[card as usize] -= 1; 
    //        self.inferred_card_count[card as usize] -= 1;
    //     }
    // }
    #[inline]
    /// Empties stores inferred player constraints
    /// NOTE:
    ///     - Debugging only
    pub fn empty_inferred_player_constraints(&mut self, player_id: usize) {
        debug_assert!(player_id < 6, "Use proper player_id thats not pile");
        // [COMBINE SJ]
        // for item in self.inferred_constraints[player_id] {
        //     if let Some(card) = item {
        //         self.inferred_card_count[card as usize] -= 1;
        //     }
        // }
        // self.inferred_constraints[player_id] = [None; 2];
        for card in self.inferred_constraints[player_id].iter() {
            self.inferred_card_count[*card as usize] -= 1;
        }
        self.inferred_constraints[player_id].clear();
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

    /// Removes a public constraint, and adjust the public_constraints and joint_constraints appropriately
    /// NOTE:
    /// - This does not modify the group_constraints that have dead_counts
    /// - This is only intended to be used for simple debugging
    /// - This should handle the group_constraints if it is intended to be used algorithmically 
    pub fn remove_public_constraint(&mut self, player_id: usize, card: Card) {
        debug_assert!(self.public_constraints[player_id].contains(&card), "Removing card constraint that does not exist in public_constraints");
        // [COMBINE SJ] DELETE
        // if self.public_constraints[player_id][1] == Some(card) {
        //     self.public_constraints[player_id][1] = None;
        // } else if self.public_constraints[player_id][0] == Some(card) {
        //     self.public_constraints[player_id][0] = self.public_constraints[player_id][0];
        //     self.public_constraints[player_id][1] = None;
        // }
        if let Some(pos) = self.public_constraints[player_id].iter().position(|c| *c == card) {
            self.public_constraints.swap_remove(pos);
            self.dead_card_count[card as usize] -= 1; 
        }
    }
    /// Removes all the constraints for a particular player and updates dead_card_count
    /// NOTE:
    /// - This does not modify the group_constraints that have dead_counts
    /// - This is only intended to be used for simple debugging
    /// - This should handle the group_constraints if it is intended to be used algorithmically 
    pub fn remove_constraints(&mut self, player_id: usize) {
        // [COMBINE SJ]
        // if let Some(card) = self.public_constraints[player_id][0] {
        //     self.dead_card_count[card as usize] -= 1;
        // }
        // if let Some(card) = self.public_constraints[player_id][1] {
        //     self.dead_card_count[card as usize] -= 1;
        // }
        for card in self.public_constraints[player_id].iter() {
            self.dead_card_count[*card as usize] -= 1;
        }
        // if let Some(card) = self.inferred_constraints[player_id][0] {
        //     self.inferred_card_count[card as usize] -= 1;
        // }
        // if let Some(card) = self.inferred_constraints[player_id][1] {
        //     self.inferred_card_count[card as usize] -= 1;
        // }
        for card in self.inferred_constraints[player_id].iter() {
            self.inferred_card_count[*card as usize] -= 1;
        }
        self.public_constraints[player_id].clear();
        self.inferred_constraints[player_id].clear();
    }
    // TODO: [ALT] Make this a check whenever inserting a group or modifying a group to avoid this disgusting mess 
    // It may also be that a HashSet is too slow as its not cache efficient
    /// Checks all group_constraints and removes the duplicates
    /// NOTE:
    /// - This is used for debugging
    /// - Mostly unnecessary to optimize or to make new traits because of
    pub fn remove_duplicate_groups(&mut self) {
        // TODO: Make CompressedGroupConstraint hashable...
        // let mut seen: HashSet<CompressedGroupConstraint> = HashSet::new();
        // let mut i: usize = 0;
        // while i < self.group_constraints.len() {
        //     if seen.insert(self.group_constraints[i]) {
        //         i += 1;
        //     } else {
        //         self.group_constraints.swap_remove(i);
        //     }
        // }
    }
    // TODO: [TEST]
    /// Updates knowledge when RevealRedraw is done or Discard is done
    /// bool_card_dead:
    /// - RevealRedraw/Exchange: bool_card_dead = false as card that is revealed is not dead, but reshuffled
    /// - Discard: bool_card_dead = true as card that is revealed is dead
    /// count:
    /// - Refers to the number of cards revealed
    /// Pruning:
    /// - [INITIAL PRUNE] if player 0 revealredraws a Duke, we prune the groups that had player 0 duke and oldcount <= newcount
    // pub fn group_initial_prune(&mut self, player_id: usize, card: Card, count: usize, bool_card_dead: bool) {
    //     debug_assert!(player_id <= 6, "Player ID Wrong");
    //     // The assumption here is that this will only be called by an alive player.
    //     // [COMBINE SJ]
    //     let player_dead_count: usize = count + (self.public_single_constraints[player_id] == Some(card)) as usize; 
    //     debug_assert!(self.public_joint_constraints[player_id] == [None, None], "Impossible Case Reached! The assumption here is that this will only be called by an alive player.");
    //     let mut i: usize = 0;
    //     while i < self.group_constraints.len() {
    //         let group = &mut self.group_constraints[i];
    //         if group.card() == card && group.get_player_flag(player_id) {
    //             if group.count() <= player_dead_count as u8 {
    //                 self.group_constraints.swap_remove(i);
    //                 continue;
    //             } else if bool_card_dead {
    //                 group.count_dead_add(count as u8);
    //                 group.count_alive_subtract(count as u8);
    //                 i += 1;
    //                 continue;
    //             }
    //         }
    //         if self.is_complement_of_pcjc(&self.group_constraints[i]) {
    //             self.group_constraints.swap_remove(i);
    //             continue;
    //         }
    //         i += 1;
    //     }
    // }
    // TODO: [ALT] to see if swap_remove checks can be in the if group.indicator(player_id) under a different paradigm
    // TODO: [CHECK THEORY]
    /// Assumes group_initial_prune was used before this
    /// Prunes a group constraints based on a dead player's cards (I think? TODO: Validate)
    // pub fn group_dead_player_prune(&mut self, player_id: usize, card_vec: [Card; 2]) {
    //     // ??? Referenced implementation Assumes group_initial_prune was used before this
    //     // [NEW]
    //     let mut i: usize = 0;
    //     let mut bool_subtract: bool = false;
    //     while i < self.group_constraints.len() {
    //         let group: &mut CompressedGroupConstraint = &mut self.group_constraints[i];
    //         let card: Card = group.card();
    //         if group.indicator(player_id) {
    //             group.group_subtract(player_id);
    //             bool_subtract = true;
    //             if card_vec.contains(&card) {
    //                 let subtract_count: u8 = match (card_vec[0] == card_vec[1]) {
    //                     true => 2,
    //                     false => 1,
    //                 };
    //                 group.count_dead_subtract(subtract_count);
    //                 debug_assert!(group.count() == 0, "Unexpected 0 found!");
    //             }
    //         }
    //         if group.count_alive() == 0 || //
    //         self.dead_card_count[card as usize] == 3 || // [DEAD PRUNE] Prune group if all cards have been shown dead for some card. There are only 3 of each card
    //         self.is_complement_of_pcjc(&self.group_constraints[i]) // [COMPLEMENT PRUNE] if group union all public union joint constraint is a full set it just means the card could be anywhere
    //         {
    //             self.group_constraints.swap_remove(i);
    //             continue;
    //         }
    //         i += 1;
    //     }
    //     self.group_redundant_prune();
    // }
    /// This function is used when a player looks at a pile and sees the cards
    pub fn peek_pile(&mut self, cards: [Card; 2]) {
        todo!()
    }
        // TODO: [TEST]
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
    pub fn death(&mut self, player_id: usize, card: Card) {
        log::trace!("In death");
        self.add_dead_player_constraint(player_id, card);
        // TODO: ADD COMPLEMENT PRUNE is probably useful here since its not done in group_redundant_prune()
        // TODO: [THOT] Group constraints being a subset of info in inferred constraints mean it can be pruned too
        //      - like if inferred info reflects the same thing as group constraint
        // QUESTION: How does inferred constraints help in determining if group is redundant? Should this be pruned above?
        // TODO: Needs to do dead player prune
        log::trace!("After add_dead_player_constraint");
        // TODO: [OPTIMIZE] Add subset groups like in RevealRedraw, so don't need to run add_inferred_groups
        self.printlog();
        self.group_redundant_prune();
        self.add_inferred_groups();
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
    pub fn reveal(&mut self, player_id: usize, card: Card) {
        log::trace!("In reveal");
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
        self.add_inferred_groups();
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
        let player_alive_card_count: [u8; 5] = self.player_alive_card_counts(player_id);
        let player_dead_card_count: [u8; 5] = self.player_dead_card_counts(player_id);
        log::trace!("player_alive_card_count: {:?}", player_alive_card_count);
        log::trace!("player_dead_card_count: {:?}", player_dead_card_count);
        let player_cards_known = self.player_cards_known(player_id);
        // Won't this remove many groups that will need to be readded by mutually exclusive additions...
        // Removes groups that are now redundant
        log::trace!("Player_cards_known: {}", player_cards_known);
        if player_cards_known != 2 { // This is an invariant
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
    /// When called looks at all the public, inferred, and group constraints to determine new inferred constraints
    /// Updates all items that need to be tracked
    /// Will likely call reveal and recurse, or use the same mechanism, to add multiple inferred before recursing
    pub fn generate_inferred_constraints(&mut self) {
        todo!()
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
        log::trace!("In redraw");
        let mut i: usize = 0;
        for (card_num, group_constraints) in [&mut self.group_constraints_amb, &mut self.group_constraints_ass, &mut self.group_constraints_cap, &mut self.group_constraints_duk, &mut self.group_constraints_con].iter_mut().enumerate() {
            let player_inferred_count = self.inferred_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
            let player_dead_count = self.public_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
            let pile_inferred_count = self.inferred_constraints[6].iter().filter(|c| **c as usize == card_num).count() as u8;
            while i < group_constraints.len() {
                let group = &mut group_constraints[i];
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
                        // group.count_alive_add(self.inferred_pile_constraints[card_num]); 
                        group.count_alive_add(pile_inferred_count); 
                    } else {
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

        // log::trace!("=== After redraw_inferred_adjustment inferred adjustment ===");
        // log::info!("{}", format!("Public Constraints: {:?}", self.public_constraints));
        // log::info!("{}", format!("Inferred Constraints: {:?}", self.inferred_constraints));
        // log::info!("{}", format!("Group Constraints:"));
        // log::info!("{}", format!("\t AMB: {:?}", group_constraints[0]));
        // log::info!("{}", format!("\t ASS: {:?}", group_constraints[1]));
        // log::info!("{}", format!("\t CAP: {:?}", group_constraints[2]));
        // log::info!("{}", format!("\t DUK: {:?}", group_constraints[3]));
        // log::info!("{}", format!("\t CON: {:?}", group_constraints[4]));
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
                    if card_group_constraints[i].get_player_flag(player_id) {
                        if !card_group_constraints[i].get_player_flag(6) {
                            // let mut readd_group = card_group_constraints[i].clone();
                            card_group_constraints[i].set_player_flag(6, true);
                            // card_group_constraints.swap_remove(i);
                            // let (_, modified) = Self::non_redundant_push_tracked(card_group_constraints, readd_group);
                            // if modified {
                            //     continue;
                            // }
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
                        }
                    }
                    i += 1;
                }
            } else {
                let mut i: usize = 0;
                while i < card_group_constraints.len() {
                    if !card_group_constraints[i].get_player_flag(player_id) {
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
                        }
                    } else {
                        if card_group_constraints[i].get_player_flag(6) {
                            // let mut readd_group = card_group_constraints[i].clone();
                            // // Let's try this
                            card_group_constraints[i].set_single_card_flag(player_id, false);
                            // card_group_constraints.swap_remove(i);
                            // Self::non_redundant_push(card_group_constraints, readd_group);
                            // continue;
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
            self.inferred_card_count[card as usize] -= 1;
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
        // let player_count_dead = self.public_constraints[player_id].iter().filter(|c| **c == card).count() as u8;
        // let player_pile_reveal_card_group = CompressedGroupConstraint::new_with_pile(player_id, card, player_count_dead, 1);
        // Self::non_redundant_push(group_constraints[card as usize], player_pile_reveal_card_group);
        self.group_redundant_prune();
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
    pub fn reveal_redraw(&mut self, player_id: usize, card: Card) {
        // Abit dumb to seperate it like this, but if not it gets abit messy and I have more branchs :/
        self.reveal(player_id, card);
        // Actually shouldnt this only move the player's card in
        // mix here is not the same as ambassador, as inferred should not be touched. And since we know the revealed card
        // To rigorously show how to mix if group is not the same card, and 1 player 0 pile
        log::trace!("=== After Reveal Intermediate State ===");
        self.printlog();
        self.redraw(player_id, card);
        // TODO: add_inferred_groups() here and test if it adds anything by panicking
        // self.add_inferred_groups();
        // self.group_redundant_prune();
        // Add the stuff here
    }
    /// Function to call for move Ambassador, without considering private information seen by the player who used Ambassador
    pub fn ambassador_public(&mut self, player_id: usize) {
        self.mix(player_id);
        self.ambassador_inferred_adjustment(player_id);
        // Some groups can be inferred even after adjustment. E.g. before mix [1 0 0 0 0 0 1] 3 Duke => inferred_pile 1 Duke
        self.group_redundant_prune();
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
        self.add_inferred_groups(); // TODO: [OPTIMIZE] Maybe might only require add_subset groups?
    }
    /// Function to call for move Ambassador, when considering private information seen by the player who used Ambassador
    pub fn ambassador_private(&mut self, player_id: usize) {
        // represent ambassador inferred cards
        // represent player inferred cards
        // "reveal" of sorts
        // dilution
        // swap?
        todo!()
    }
    // TODO: [ALT] Try to see if you can do a 2n checks instead of n^2, by just checking if the added item makes anything redundant or if it is redundant so you shift 
    // TODO: [CHECK THEORY]
    // TODO: figure out why count is used? isnt revealredraw always one?
    // work to the add instead of just a generic check on all group constraints
    /// Used for RevealRedraw
    /// - Only prunes those that can be immediately found to be redundant, without comparing to other groups
    /// - Assumes player_id is alive and thus joint_constraint is empty, public_constraint may or may not be empty
    /// - Assumes no group is redundant before adding
    /// - Assumes no dead player info is in groups before adding
    /// - Does not leaves no group redundant after adding
    /// - Does not remove a group based on inferred constraints
    /// - Leaves no dead player info in groups
    // pub fn add_group_constraint(&mut self, player_id: usize, card: Card, count: u8) {
    //     // TODO: Rename to RevealRedraw
    //     // DO we need to initial prune here? like those players that
    //     // TODO: [THOT] what if player reveals card and its the last card of its kind?
    //     // TODO: [THOT] Can we just prune based on representing new information and doing a redundant check?
    //     // TODO: [THOT] Or maybe make a method that compares 2 groups, then make a modification to the first based off the second?
    //     // TODO: [THOT] For list method, consider split revealing card, and shuffling. Revealing card changes LEGAL CARD LIST in a similar way...
    //     //          So perhaps you can do reveal, then prune all, then shuffle all?
    //     // TODO: [Implement] Have to label those with pile false as true too!
    //     // TODO: [Implement] Refactor to 1 check player flag, 2 check pile flag
    //     // TODO: [FINAL] Check if change_flag can be move out or not
    //     let mut change_flag: bool = false;
    //     let player_card_count: u8 = count + (self.public_single_constraints[player_id] == Some(card)) as u8;
    //     let mut i: usize = 0;
    //     while i < self.group_constraints.len() {
    //         let group = &mut self.group_constraints[i];
    //         if !group.get_player_flag(player_id) && group.get_player_flag(6) {
    //             change_flag = true;
    //             group.group_add(player_id);
    //             // Old Group: Duke [0 1 0 0 0 0 1] Count 1
    //             // Move RevealRedraw a Duke is revealed and shuffled into pile
    //             // Add Group: Duke [0 0 1 0 0 0 1] Count 1
    //             // Intuitively there will be 1 Duke at first with player 2 now shuffled among himself and the pile
    //             // And 1 Duke that was originally with Player 1 & pile
    //             // After RevealRedraw there are in total 2 Dukes among player 1,2 and Pile
    //             // So we must increment the old counter by new_count
    //             if group.card() == card {
    //                 group.count_alive_add(count);
    //                 if Some(card) == self.public_single_constraints[player_id] {
    //                     group.count_dead_add(1);
    //                 }
    //                 debug_assert!(group.count() <= 3, "Impossible case reached!");
    //                 debug_assert!(group.count_alive() <= 3, "Impossible case reached!");
    //                 debug_assert!(group.count_dead() <= 3, "Impossible case reached!");
    //             } else {
    //                 if self.public_single_constraints[player_id] == Some(group.card()) {
    //                     // Adding if player has dead_card thats equal to the group card
    //                     group.count_dead_add(1);
    //                 }
    //             }
    //             if group.all_in() {
    //                 // [FULL PRUNE] because group constraint just means there could be a Duke anywhere (anyone or the pile might have it)
    //                 self.group_constraints.swap_remove(i);
    //                 continue;
    //             } else if self.is_complement_of_pcjc(&self.group_constraints[i]) {
    //                 // [COMPLEMENT PRUNE] if group union all public union joint constraint is a full set it just means the card could be anywhere
    //                 self.group_constraints.swap_remove(i);
    //                 continue;
    //             }
    //         } else if group.get_player_flag(player_id)  {
    //             // TODO: Check if need initial prune
    //             // TODO: Maybe rearrange group.player flag and pile flag
    //             // META-CASE 1
    //             // In these examples, player_id == 2, player_flag == true, pile_flag in {true, false}, player alive card always >= 1, group.card() == card == Duke
    //             // CASE 1: player has 2 Duke (dead, alive) = (1, 1)
    //             // [0, 0, 1, 0, 0, 1, 0] Duke where (dead, alive) = (n, 0) => GROUP INCLUDES PLAYER WHO HAS ALIVE=0, (0, n) => GROUP INCLUDES PLAYER WHO HAS DEAD CARD, (1, 1) => PRUNE, (1, 2) => Handle here, (2, 1) => PRUNE
    //             // PRUNED because what we add at the end is at least better information
    //             // CASE 2: player has 1 Duke (dead, alive) = (0, 1), 1 alive other card, 
    //             // [0, 0, 1, 0, 0, 1, 0] Duke where (dead, alive) = (n, 0) => GROUP INCLUDES PLAYER WHO HAS ALIVE=0, (0, 1) => PRUNE, (0, 2) => HANDLE, (0, 3) => HANDLE, (1, 1) => PRUNE, (1, 2) => HANDLE, (2, 1) => PRUNE
    //             // CASE 3: player has 1 Duke (dead, alive) = (0, 1), 1 dead other card
    //             // [0, 0, 1, 0, 0, 1, 0] Duke where (dead, alive) = (n, 0) => GROUP INCLUDES PLAYER WHO HAS ALIVE=0, (0, 1) => PRUNE, (0, 2) => HANDLE, (0, 3) => HANDLE, (1, 1) => PRUNE, (1, 2) => HANDLE, (2, 1) => PRUNE
    //             // CASE IGNORE: 2 Dead Duke, 2 Alive Duke, Not a possible to reach here!
    //             // CONCLUSION 1: Seems like we PRUNE when group.alive_count() == 1 (Like in add_public_constraint)

    //             // META-CASE 2
    //             // In these examples, player_id == 2, player_flag == true, pile_flag in {true, false}, player alive card always >= 1, group.card() != card, card == Duke
    //             // In some cases, this revelation might tell us, certain players DONT have card, and so allow us to update the LEGAL CARD LIST
    //             // If pile_flag == false make it true, if pile_flag == true, leave it
    //             // player_flag = false, if both player cards are known! KNOWN => player has dead card, and current card, KNOWN => we know player current card and unrevealed card
    //             // TODO: [IMPLEMENT] save private inferred info => single and joint, can store there if inferred for quicker access
    //             // CASE 1: player has 2 Duke (dead, alive) = (1, 1)
    //             // CASE 2: player has 1 Duke (dead, alive) = (0, 1), 1 alive other card, 
    //             // If pile_flag == false make it true, if pile_flag == true, leave it
    //             // group.card() != Duke where (dead, alive) = (n, 0) => GROUP ALIVE_COUNT > 0, (0, n) => MIX
    //             // CASE 3: player has 1 Duke (dead, alive) = (0, 1), 1 dead other card
    //             // If pile_flag == false make it true, if pile_flag == true, leave it
    //             // TODO: [THEORY CHECK] all cases, merge with below
    //             // TODO: THEN split by whether pile flag true/false
    //             if group.card() == card {
    //                 if group.count_alive() == 1 {
    //                     // [SUBSET PRUNE] knowing the player had the card now makes this group obsolete
    //                     // No need to modify this as the information from the player's pile swap gets added at the end
    //                     self.group_constraints.swap_remove(i);
    //                     continue;
    //                 }
    //             }
    //             if !group.get_player_flag(6) {
    //                 // TODO: Consider when group needs to be modified if current player flag is 1 and this reveals all their cards
    //                 group.set_player_flag(6, true);
    //                 self.empty_inferred_pile_constraint();
    //                 // TODO: you don't empty all the players' cards, because only 1 card is revealed!
    //                 self.subtract_inferred_player_constraints(player_id, card);
    //                 change_flag = true;
    //             }
    //         }
    //         i += 1;
    //     }
    //     let addition = if Some(card) == self.public_single_constraints[player_id] {
    //         CompressedGroupConstraint::new(player_id, card, 1, count)
    //     } else {
    //         CompressedGroupConstraint::new(player_id, card, 0, count)
    //     };
    //     if change_flag {
    //         if !self.is_complement_of_pcjc(&addition) {
    //             self.group_constraints.push(addition);
    //         }
    //         self.group_redundant_prune();
    //     } else {
    //         if !self.is_complement_of_pcjc(&addition) {
    //             // Probably abstract this into a function
    //             // No need to check redundancy among all, just what is being added
    //             let mut i: usize = 0;
    //             while i < self.group_constraints.len() {
    //                 if addition.is_redundant(&self.group_constraints[i]) {
    //                     break;
    //                 }
    //                 if self.group_constraints[i].is_redundant(&addition) {
    //                     self.group_constraints.swap_remove(i);
    //                     continue;
    //                 }
    //                 i += 1;
    //             }
    //         }
    //     }
    // }
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
        for group in self.group_constraints_mut().iter_mut() {
            let mut i: usize = 0;
            'outer: while i < group.len() {
                let mut j: usize = i + 1;
                if group[i].get_total_count() == 3 {
                    'inner: while j < group.len() {
                        if group[j].get_total_count() == 3 {
                            if group[j].part_list_is_subset_of(&group[i]) {
                                group.swap_remove(i);
                                continue 'outer;
                            }
                            if group[i].part_list_is_subset_of(&group[j]) {
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
    pub fn add_inferred_groups(&mut self) {
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
        let add_subset =self.add_subset_groups();
        let mut_excl_changes = self.add_mutually_exclusive_unions();
        let mut inf_exc_pl = self.add_inferred_except_player();
        bool_continue = add_subset || mut_excl_changes || inf_exc_pl;
        // Then runs one if the other is still true
        // idea here is that subset groups adds all the smaller groups, mut excl adds the larger groups
        //      - we get smaller groups from larger groups, so if no new larger groups added, no new smaller groups can be inferred
        //      - we get larger groups from smaller groups, so if no new smaller groups added, no new larger groups can be inferred
        //      - Technically the functions also add inferred groups, so if no inferred groups added, no new information added, no need to run either  
        
        while inf_exc_pl {
            // Runs a
            println!("Running again");
            let add_subset = self.add_subset_groups();
            log::info!("add_subset_groups added groups: {}", add_subset);
            if add_subset {
                let mut_excl_changes = self.add_mutually_exclusive_unions();
                log::info!("add_mutually_exclusive_unions added groups: {}", mut_excl_changes);
                if mut_excl_changes {
                    inf_exc_pl = self.add_inferred_except_player();
                    log::info!("add_inferred_except_player added groups: {}", inf_exc_pl);
                } else {
                    inf_exc_pl = false;
                }
            } else {
                inf_exc_pl = false;
            }
        }
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
                                    CompressedCollectiveConstraint::non_redundant_push(&mut new_groups[card_num], new_group);
                                } else {
                                    // only 1 flag after removal, and so should be added to inferred constraints later
                                    log::trace!("add_subset_groups found group for new_inferred_constraints: {}", new_group);
                                    CompressedCollectiveConstraint::non_redundant_push(&mut new_inferred_constraints, new_group);
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut card_changes: Vec<Vec<usize>> = vec![Vec::with_capacity(2); 5]; // Store (player_id, bool_counts => false: 1, true: 2)
        // Add new_inferred_constraints
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
                        card_changes[card as usize].push(player_id);
                        log::trace!("add_sub_groups After self.public_constraints: {:?}", self.public_constraints);
                        log::trace!("add_sub_groups After self.inferred_constraints: {:?}", self.inferred_constraints);
                        // TODO: Needs to prune the groups too...
                        // TODO: Adjust counts properly too... or remove inferred_counts
                    }
                },
                2 => {
                    if player_id == 6 {
                        let no_to_push = 2 - self.inferred_constraints[player_id].iter().filter(|c| **c == card).count();
                        for _ in 0..no_to_push {
                            self.inferred_constraints[player_id].push(card);
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
                        let no_to_push = alive_count - self.inferred_constraints[player_id].iter().filter(|c| **c == card).count() as u8;
                        for _ in 0..no_to_push {
                            self.inferred_constraints[player_id].push(card);
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
                                    CompressedCollectiveConstraint::non_redundant_push(&mut new_groups[card_num], new_group);
                                } else {
                                    // only 1 flag after removal, and so should be added to inferred constraints later
                                    log::trace!("add_subset_groups added to new_inferred_constraints: {}", new_group);
                                    CompressedCollectiveConstraint::non_redundant_push(&mut new_inferred_constraints, new_group);
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
                        card_changes[card as usize].push(player_id);
                        log::trace!("add_sub_groups_recurse After self.public_constraints: {:?}", self.public_constraints);
                        log::trace!("add_sub_groups_recurse After self.inferred_constraints: {:?}", self.inferred_constraints);
                        // TODO: Needs to prune the groups too...
                        // TODO: Adjust counts properly too... or remove inferred_counts
                    }
                },
                2 => {
                    if player_id == 6 {
                        let no_to_push = alive_count - self.inferred_constraints[player_id].iter().filter(|c| **c == card).count() as u8;
                        for _ in 0..no_to_push {
                            self.inferred_constraints[player_id].push(card);
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
                        let no_to_push = alive_count - self.inferred_constraints[player_id].iter().filter(|c| **c == card).count() as u8;
                        for _ in 0..no_to_push {
                            self.inferred_constraints[player_id].push(card);
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
                            log::trace!("");
                            log::trace!("=== add_mutually_exclusive_unions ref vs inferred & public");
                            log::trace!("add_mutually_exclusive_unions public_constraints: {:?}", self.public_constraints);
                            log::trace!("add_mutually_exclusive_unions inferred_constraints: {:?}", self.inferred_constraints);
                            log::trace!("add_mutually_exclusive_unions reference_group_i: {}, player_id: {}, player_flag: {}", reference_group_i, player_id, player_flag);
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
                    let mut player_index: usize = 0;
                    while player_index < players.len() {
                        let player = players[player_index];
                        if group.get_player_flag(player) {
                            let mut complement_part_list: CompressedGroupConstraint = group.get_blank_part_list();
                            complement_part_list.set_player_flag(player, false);
                            // Gets maximal holdable number of cards in the group outside of the player
                            let mut maximal_alive_card_counts: [u8; 5] = [0; 5];
                            for (card_num_inner, complement_groups) in self.group_constraints().iter().enumerate() {
                                if card_num_inner != card_num {
                                    for complement_group in complement_groups.iter() {
                                        // See Assumptions! This requires add_subsets and add_mut excl unions to be ran for this to work
                                        if complement_group.part_list_is_subset_of(&complement_part_list) {
                                            maximal_alive_card_counts[card_num_inner] = std::cmp::max(maximal_alive_card_counts[card_num_inner], complement_group.count_alive());
                                        }
                                    }
                                }
                            }
                            // All dead other than player
                            let mut complement_maximal_holdable_dead: u8 = 0;
                            for i in 0..7 as usize {
                                if group.get_player_flag(i) && i != player {
                                    complement_maximal_holdable_dead += self.public_constraints[i].len() as u8;
                                }
                            }
                            // We have got the
                            let complement_maximal_holdable_alive = maximal_alive_card_counts.iter().sum::<u8>();
                            // All spaces other than player
                            let complement_maximal_holdable_spaces = complement_part_list.max_spaces();
                            // This should not overflow as spaces should always be > both
                            let max_free_spaces = complement_maximal_holdable_spaces - complement_maximal_holdable_dead - complement_maximal_holdable_alive; 
                            log::info!("=== add_inferred_except_player discovery ===");
                            log::info!("Parent Group: {}", group);
                            log::info!("Complement part list: {}, count: {}", complement_part_list, complement_part_list.part_list_count());
                            log::info!("Current Player: {}", player);
                            log::info!("Max_free_spaces: {} = complement_maximal_holdable_spaces: {} - complement_maximal_holdable_dead: {} - complement_maximal_holdable_alive: {}", max_free_spaces, complement_maximal_holdable_spaces, complement_maximal_holdable_dead, complement_maximal_holdable_alive);
                            log::info!("Complement_max_alive array: {:?}", maximal_alive_card_counts);
                            log::info!("Complement_max_dead: {}", complement_maximal_holdable_dead);
                            log::info!("Complement_max_alive: {}", complement_maximal_holdable_alive);
                            if group_alive_count > max_free_spaces {
                                let inferred_counts = group_alive_count - max_free_spaces;
                                let known_counts= self.inferred_constraints[player].iter().filter(|c| **c as usize == card_num).count() as u8;
                                log::info!("inferred_counts: {} = group_alive_count: {} - max_free_spaces: {}", inferred_counts, group_alive_count, max_free_spaces);
                                if inferred_counts > known_counts {
                                    log::info!("add_inferred_except_player discovered player: {player} has card: {:?}", Card::try_from(card_num as u8).unwrap());
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
    /// Returns an array indexed by [player][card] that indicates if a player can have a particular card
    /// true => impossible
    /// false => possible
    pub fn generate_one_card_impossibilities_player_card_indexing(&mut self) -> [[bool; 5]; 7] {
        let mut impossible_cards: [[bool; 5]; 7] = [[false; 5]; 7];
        // This first part is here until the grouping part auto includes the inferred groups... probably in mutual exclusive groups
        // TODO: Remove this for loop
        for player_id in 0..7 as usize{
            if self.public_constraints[player_id].len() + self.inferred_constraints[player_id].len() == 2 {
                impossible_cards[player_id] = [true; 5];
                for card in self.inferred_constraints[player_id].iter() {
                    impossible_cards[player_id][*card as usize] = false;
                }
            }
        }
        // TODO: [OPTIMIZE], because there are too many groups, maybe only check for player-ids that are not dead players (or are eligible)
        'outer: for (card_num, group_constraints) in self.group_constraints().iter().enumerate() {
            for group in group_constraints.iter() {
                if group.count_dead() == 3 {
                    for player_id in 0..7 as usize {
                        impossible_cards[player_id][card_num] = true;
                    }
                    continue 'outer;
                } else if group.count() == 3{
                    for player_id in 0..7 as usize {
                        if !group.get_player_flag(player_id) {
                            impossible_cards[player_id][card_num] = true;
                        }
                    }
                }
            }
        }
        impossible_cards
    }
    /// This is currently broken
    pub fn generate_one_card_impossibilities_card_player_indexing(&mut self) -> [[bool; 7]; 5] {
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
    /// General method that considers the entire Collective Constraint and generates/updates the card state
    pub fn update_legal_cards_state(&mut self) {
        // [THOT] Changes only on dead card, reveal redraw, ambassador
        //        - Dead Card: Makes a player's card known, may reveal info that can be inferred from group
        //        - RevealRedraw: Same as Dead Card but not permanent, maybe could infer as per dead card, but do an OR like ambassador after?
        //        - Ambassador: Just an OR between player's list and the Ambassador, then redundant prune and complement prune
        //        - Private Info Amb: Just [0 0 0 0 0 0 1] for cards left inside private info about players own cards
        //        - Private Info layer perhaps can just be the insertion of [0 0 0 0 0 0 1] to the public info layer before calculating
        //        This approach would have simplicity in that AMB is easy, RevealRedraw and Deadcard but update the same way, then RevealRedraw calls AMB Mix
        // [THOT] We can have all info about player cards - what they cant have as the list?
        // [THOT] Optimistic - we can assume all cards and substract what they can't have
        // [THOT] If joint constraint, we know we don't have to generate list for that player
        // [THOT] If group constraint only has 1 player_flag, we know a player has that particular card
        // [THOT] If we know all a group's cards, we know what cards the players in the cant have, and what cards the players outside the group cant have 
        // [THOT] 2 options:
        //          1. updating while traversing group in dead_card or reveal_redraw => perhaps optimal?
        //          2. updating after => more general (more favourable)
        //              Job of add constraint and public constraint is soley removal of redundancies, and proper updating of groups
        //        Regardless, AMB is trivially easy to update
        // TODO: IMPLEMENT private single and joint constraint, which would be helpful if somehow we discover players can have a particular card!
        todo!()
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