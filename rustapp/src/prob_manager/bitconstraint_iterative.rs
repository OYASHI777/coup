use rayon::join;

use crate::history_public::Card;
use std::collections::VecDeque;
use super::constraint::GroupConstraint;

// TODO: public constraint as a u32 3 bits per card x 6 players? Or probably better to just have a Vec that reserves space and reduces need for conversion
// TODO: joint constraint as u64 3bits per card x 6 players?
// TODO: but this is helpful for storeing the game state and sending it to a neural network...

/// From right to left, where 0 represents the last item
/// Bits 0..=6 represent boolean flags indicating which players are involved
/// Bits 7..=9 represent the Card number from 0..=4
/// Bits 10..=11 represent the dead count 0..=2
/// Bits 12..=13 represent the alive count 1..=3
/// Bits 14..=15 represent total count = dead + alive 1..=3
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct CompressedGroupConstraint(u16);

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

// [FIRST GLANCE PRIORITY] Consider making a private constraint, to contain players' private information, to generate the public, and each players' understanding all at once
// [FIRST GLANCE PRIORITY] Add inferred impossible cards for each player? Then just check inferred joint else all but impossible cards to generate?
// [FIRST GLANCE PRIORITY] Consider processing all new items to add with redundant checks in bulk
// [FIRST GLANCE PRIORITY] Check redundant addition process. New item can make more than 1 item redundant. e.g. [1 0 0 0 0 0 0] 2 cards makes both [1 0 1 0 0 0 0] 1 cards and [1 1 0 0 0 0 0] 1 cards redundant
impl CompressedGroupConstraint {
    const START_FLAGS: [u16; 6] = [0b0000_0000_0100_0001, 0b0000_0000_0100_0010, 0b0000_0000_0100_0100, 0b0000_0000_0100_1000, 0b0000_0000_0101_0000, 0b0000_0000_0110_0000,];
    const PLAYER_BITS: u16 = 0b0000_0000_0111_1111; // Bits 0-6
    const CARD_SHIFT: u8 = 7;
    const CARD_MASK: u16 = 0b0000_0011_1000_0000; // Bits 7-9
    const DEAD_COUNT_SHIFT: u8 = 10;
    const DEAD_COUNT_MASK: u16 = 0b0000_1100_0000_0000; // Bits 10-11
    const ALIVE_COUNT_SHIFT: u8 = 12;
    const ALIVE_COUNT_MASK: u16 = 0b0011_0000_0000_0000; // Bits 12-13
    const TOTAL_COUNT_SHIFT: u8 = 14;
    const TOTAL_COUNT_MASK: u16 = 0b1100_0000_0000_0000; // Bits 14-15
    const MAX_PLAYERS: u8 = 7;
    pub fn new(player_id: usize, card: Card, count_dead: u8, count_alive: u8) -> Self {
        debug_assert!(player_id < 6);
        let mut output = CompressedGroupConstraint(Self::START_FLAGS[player_id]);
        output.set_card(card);
        output.set_alive_count(count_alive);
        output.set_dead_count(count_dead);
        output.set_total_count((count_dead + count_alive));
        output
    }

    pub fn set_player_flag(&mut self, player_id: usize, value: bool) {
        debug_assert!(player_id < 7);
        if value {
            self.0 |= 1 << player_id;
        } else {
            self.0 &= !(1 << player_id);
        }
    }
    pub fn get_player_flag(&self, player_id: usize) -> bool {
        debug_assert!(player_id < 7);
        (self.0 & (1 << player_id)) != 0
    }
    pub fn get_player_flags(&self) -> u16 {
        self.0 & Self::PLAYER_BITS
    }
    pub fn set_card(&mut self, card: Card) {
        debug_assert!(u8::from(card) < 8, "Card value out of bounds");
        // Clear existing card bits
        self.0 &= !(0b111 << Self::CARD_SHIFT);
        // Set new card bits
        self.0 |= (card as u16 & 0b111) << Self::CARD_SHIFT;
    }
    pub fn get_card(&self) -> Card {
        let num = ((self.0 & (0b111 << Self::CARD_SHIFT)) >> Self::CARD_SHIFT) as u8;
        Card::try_from(num).expect("Invalid value found")
    }
    /// Sets the dead count (0-3).
    pub fn set_dead_count(&mut self, count: u8) {
        debug_assert!(count < 4, "Dead count out of bounds");
        // Clear existing dead count bits
        self.0 &= !(0b11 << Self::DEAD_COUNT_SHIFT);
        // Set new dead count bits
        self.0 |= (count as u16 & 0b11) << Self::DEAD_COUNT_SHIFT;
    }

    /// Retrieves the current dead count.
    fn get_dead_count(&self) -> u8 {
        ((self.0 & (0b11 << Self::DEAD_COUNT_SHIFT)) >> Self::DEAD_COUNT_SHIFT) as u8
    }

    /// Sets the alive count (0-3).
    pub fn set_alive_count(&mut self, count: u8) {
        debug_assert!(count < 4, "Alive count out of bounds");
        // Clear existing alive count bits
        self.0 &= !(0b11 << Self::ALIVE_COUNT_SHIFT);
        // Set new alive count bits
        self.0 |= (count as u16 & 0b11) << Self::ALIVE_COUNT_SHIFT;
    }
    
    /// Retrieves the current alive count.
    fn get_alive_count(&self) -> u8 {
        ((self.0 & (0b11 << Self::ALIVE_COUNT_SHIFT)) >> Self::ALIVE_COUNT_SHIFT) as u8
    }
    /// Sets the total count (0-3).
    pub fn set_total_count(&mut self, count: u8) {
        debug_assert!(count < 4, "Total count out of bounds");
        // Clear existing total count bits
        self.0 &= !(0b11 << Self::TOTAL_COUNT_SHIFT);
        // Set new total count bits
        self.0 |= (count as u16 & 0b11) << Self::TOTAL_COUNT_SHIFT;
    }
    fn add_dead_count(&mut self, amount: u8) {
        let dead_bits = (self.0 & Self::DEAD_COUNT_MASK) >> Self::DEAD_COUNT_SHIFT;
        let new_count = dead_bits + (amount as u16);
        // TODO: Change this, this is meant to count total dead cards, up to 15, not just those of the card!
        debug_assert!(new_count < 4, "Dead count would exceed maximum");
        self.0 = (self.0 & !Self::DEAD_COUNT_MASK) | (new_count << Self::DEAD_COUNT_SHIFT);
    }

    fn sub_dead_count(&mut self, amount: u8) {
        let dead_bits = (self.0 & Self::DEAD_COUNT_MASK) >> Self::DEAD_COUNT_SHIFT;
        debug_assert!(dead_bits >= amount as u16, "Dead count would go below zero");
        self.0 = (self.0 & !Self::DEAD_COUNT_MASK) | ((dead_bits - amount as u16) << Self::DEAD_COUNT_SHIFT);
    }

    fn add_alive_count(&mut self, amount: u8) {
        let alive_bits = (self.0 & Self::ALIVE_COUNT_MASK) >> Self::ALIVE_COUNT_SHIFT;
        let new_count = alive_bits + (amount as u16);
        debug_assert!(new_count < 4, "Alive count would exceed maximum");
        self.0 = (self.0 & !Self::ALIVE_COUNT_MASK) | (new_count << Self::ALIVE_COUNT_SHIFT);
    }

    fn sub_alive_count(&mut self, amount: u8) {
        let alive_bits = (self.0 & Self::ALIVE_COUNT_MASK) >> Self::ALIVE_COUNT_SHIFT;
        debug_assert!(alive_bits >= amount as u16, "Alive count would go below zero");
        self.0 = (self.0 & !Self::ALIVE_COUNT_MASK) | ((alive_bits - amount as u16) << Self::ALIVE_COUNT_SHIFT);
    }

    fn add_total_count(&mut self, amount: u8) {
        let total_bits = (self.0 & Self::TOTAL_COUNT_MASK) >> Self::TOTAL_COUNT_SHIFT;
        let new_count = total_bits + (amount as u16);
        debug_assert!(new_count < 4, "Total count would exceed maximum");
        self.0 = (self.0 & !Self::TOTAL_COUNT_MASK) | (new_count << Self::TOTAL_COUNT_SHIFT);
    }

    fn sub_total_count(&mut self, amount: u8) {
        let total_bits = (self.0 & Self::TOTAL_COUNT_MASK) >> Self::TOTAL_COUNT_SHIFT;
        debug_assert!(total_bits >= amount as u16, "Total count would go below zero");
        self.0 = (self.0 & !Self::TOTAL_COUNT_MASK) | ((total_bits - amount as u16) << Self::TOTAL_COUNT_SHIFT);
    }
    /// Updates the total_count based on dead_count and alive_count.
    ///
    /// The total_count is set to the sum of dead_count and alive_count, clamped to a maximum of 3.
    pub fn update_total_count(&mut self) {
        let dead = self.get_dead_count();
        let alive = self.get_alive_count();
        let total = dead + alive;
        debug_assert!(total<4);
        // Clear existing total_count bits
        self.0 &= !Self::TOTAL_COUNT_MASK;

        // Set new total_count bits
        self.0 |= (total as u16 & 0b11) << Self::TOTAL_COUNT_SHIFT;
    }
    /// Retrieves the total_count (1-3).
    ///
    /// # Example
    ///
    /// ```
    /// let total = constraint.get_total_count();
    /// ```
    pub fn get_total_count(&self) -> u8 {
        ((self.0 & Self::TOTAL_COUNT_MASK) >> Self::TOTAL_COUNT_SHIFT) as u8
    }
    /// Clears all player flags.
    pub fn clear_players(&mut self) {
        self.0 &= !Self::PLAYER_BITS;
    }
    /// Retrieves all set player flags as a fixed-size array of boolean values.
    ///
    /// Each element in the returned array corresponds to a player.
    /// The array will have exactly 7 elements, where each index (0-6) represents a player.
    ///
    /// # Example
    ///
    /// ```
    /// let player_flags = constraint.get_set_players();
    /// ```
    pub fn get_set_players(&self) -> [bool; 7] {
        [
            self.get_player_flag(0),
            self.get_player_flag(1),
            self.get_player_flag(2),
            self.get_player_flag(3),
            self.get_player_flag(4),
            self.get_player_flag(5),
            self.get_player_flag(6),
        ]
    }
}
impl CompressedGroupConstraint {
    /// Constructor method that initialised based on a list of flags with each index representing player id
    pub fn new_list(participation_list: [bool; 7], card: Card, count_dead: usize, count_alive: usize) -> Self {
        debug_assert!(count_dead < 4, "Dead count must be less than 4");
        debug_assert!(count_alive < 4, "Alive count must be less than 4");
        debug_assert!(count_dead + count_alive < 4, "Total count must be less than 4");
        let mut value: u16 = 0;
        for (index, flag) in participation_list.iter().enumerate() {
            if *flag {
                value |= 1 << index;
            }
        }
        value |= (card as u16 & 0b111) << Self::CARD_SHIFT;
        value |= (count_dead as u16 & 0b11) << Self::DEAD_COUNT_SHIFT;
        value |= (count_alive as u16 & 0b11) << Self::ALIVE_COUNT_SHIFT;
        value |= ((count_dead + count_alive) as u16 & 0b11) << Self::TOTAL_COUNT_SHIFT;
        CompressedGroupConstraint(value)
    }
    /// Constructor method that initialised based on a u16 0b00000000 for e.g. with each bit STARTING FROM THE RIGHT representing a player id flag
    /// From right to left, where 0 represents the last item
    /// Bits 0..=6 represent boolean flags indicating which players are involved
    /// Bits 7..=9 represent the Card number from 0..=4
    /// Bits 10..=11 represent the dead count 0..=2
    /// Bits 12..=13 represent the alive count 1..=3
    /// Bits 14..=15 represent total count = dead + alive 1..=3
    pub fn new_bit(participation_flags: u16, card: Card, count_dead: usize, count_alive: usize) -> Self {
        debug_assert!(participation_flags < 0b10000000, "Participation flag should be < 0b10000000 as there are only 7 players");
        debug_assert!(count_dead < 4, "Dead count must be less than 4");
        debug_assert!(count_alive < 4, "Alive count must be less than 4");
        debug_assert!(count_dead + count_alive < 4, "Total count must be less than 4");
        let mut value: u16 = participation_flags;
        value |= (card as u16 & 0b111) << Self::CARD_SHIFT;
        value |= (count_dead as u16 & 0b11) << Self::DEAD_COUNT_SHIFT;
        value |= (count_alive as u16 & 0b11) << Self::ALIVE_COUNT_SHIFT;
        value |= ((count_dead + count_alive) as u16 & 0b11) << Self::TOTAL_COUNT_SHIFT;
        CompressedGroupConstraint(value)
    }
    /// Adds a player to the group represented by the group constraint
    /// Example: If the participation list originally included Players {1, 2, 5}, then adding 6 would make it {1, 2, 5, 6}.
    pub fn group_add(&mut self, player_id: usize){
        debug_assert!(player_id < 7, "Invalid Player Id");
        self.set_player_flag(player_id, true);
    }
    /// Replaces current participation list with a new one
    pub fn group_add_list(&mut self, list: [bool; 7]){
        self.0 &= !Self::PLAYER_BITS;
        for (index, flag) in list.iter().enumerate() {
            if *flag {
                self.0 |= 1 << index;
            }
        }
    }
    /// Removes a player from the group represented by the group constraint
    /// Example: If the participation list originally included Players {1, 2, 5}, then subtracting 5 would make it {1, 2}.
    pub fn group_subtract(&mut self, player_id: usize){
        debug_assert!(player_id < 7, "Invalid Player Id");
        let mask = !(0b1 << player_id);
        self.0 &= mask;
    }
    /// Adds number to dead count and increases total count too
    pub fn count_dead_add(&mut self, num: u8){
        self.add_dead_count(num);
        self.add_total_count(num);
    }
    /// Adds number to alive count and increases total count too
    pub fn count_alive_add(&mut self, num: u8){
        self.add_alive_count(num);
        self.add_total_count(num);
    }
    /// Subtracts number to dead count and decreases total count too
    pub fn count_dead_subtract(&mut self, num: u8){
        self.sub_dead_count(num);
        self.sub_total_count(num);
    }
    /// Subtracts number to alive count and decreases total count too
    pub fn count_alive_subtract(&mut self, num: u8){
        self.sub_alive_count(num);
        self.sub_total_count(num);
    }
    // TODO: Merge and refactor the wrappers
    // TODO: Review if its better to return a usize
    /// Returns count of alive + dead cards present in the group
    pub fn count(&self) -> u8 {
        self.get_total_count()
    }
    /// Returns count of dead cards present in the group
    pub fn count_dead(&self) -> u8 {
        self.get_dead_count()
    }
    /// Returns count of alive cards present in the group
    pub fn count_alive(&self) -> u8 {
        self.get_alive_count()
    }
    /// Returns the flag of whether a player is in the group being represented by the GroupConstraint
    pub fn indicator(&self, player_id : usize) -> bool {
        self.get_player_flag(player_id)
    }
    // TODO: Determine if this is the fastest way
    /// Returns true if participation list includes all players
    pub fn all_in(&self) -> bool {
        (self.0 & Self::PLAYER_BITS) == 0b0000_0000_0111_1111
    }
    // TODO: Determine if this is the fastest way
    /// Returns true if participation list includes all players
    pub fn none_in(&self) -> bool {
        (self.0 & Self::PLAYER_BITS) == 0b0000_0000_0000_0000
    }
    /// Returns a list of flags (the participation list) that indicates the set of players being specified to have a certain count of alive cards and dead cards
    pub fn get_list(&self) -> [bool; 7]{
        self.get_set_players()
    }
    pub fn get_flags(&self) -> u16 {
        self.0 & Self::PLAYER_BITS
    }
    /// Gets the Card thats stored
    pub fn card(&self) -> Card {
        self.get_card()
    } 
    /// Returns true if input group makes self redundant group constraint to store
    /// - The participation lists have to be equal
    /// - alive and dead counts of self have to be less than the input group
    pub fn is_subset_of(&self, group: &Self) -> bool {
        // Returns true if group makes self redundant
        // Its redundant if they have the same participation list and their counts are equal
        // Its also redundant if they have the same participation list and group has a higher count
        //      If a group has at least 2 Dukes, it also fulfils having at least 1 Duke.
        //      Therefore having at least 1 Duke is redundant

        if self.get_card() == group.get_card() &&
        // If participation lists are the same
        (self.get_flags() == group.get_flags()) &&
        // self.count() <= group.count() && 
        self.count_dead() <= group.count_dead() &&
        self.count_alive() <= group.count_alive() {
            return true
        }
        return false
    }
    /// Returns true if self's partipation list is subset of the input group's participation list
    /// Returns true if both participation lists are equal
    pub fn part_list_is_subset_of(&self, group: &Self) -> bool {
        // Checks if self participation list is a subset of group's participation list
        (group.0 & Self::PLAYER_BITS) == (self.0 & Self::PLAYER_BITS) | (group.0 & Self::PLAYER_BITS)
    }
    /// Returns true if self's partipation list is subset of the input group's participation list
    /// Returns true if both participation lists are equal
    pub fn part_list_is_subset_of_arr(&self, arr: &[bool; 7]) -> bool {
        // Checks if self participation list is a subset of group's participation list
        !((self.get_player_flag(0) & !arr[0]) |
        (self.get_player_flag(1) & !arr[1]) |
        (self.get_player_flag(2) & !arr[2]) |
        (self.get_player_flag(3) & !arr[3]) |
        (self.get_player_flag(4) & !arr[4]) |
        (self.get_player_flag(5) & !arr[5]) |
        (self.get_player_flag(6) & !arr[6]))
    }
    /// Returns true if the participation lists of self and group are mutually exclusive
    pub fn part_list_is_mut_excl(&self, group: Self) -> bool {
        // Checks if the groups are mutually exclusive
        ((self.0 & Self::PLAYER_BITS) & (group.0 & Self::PLAYER_BITS)) == 0
    }
    /// Returns true if participation list is subset of a group of only player and pile as true
    pub fn part_list_is_subset_of_player_and_pile(&self, player_id: usize) -> bool {
        Self::START_FLAGS[player_id] == (self.0 & Self::PLAYER_BITS) | Self::START_FLAGS[player_id]
    }
    // TODO: [TEST]
    // TODO: Refactor and make this redundant
    /// TODO: Consider refactoring this to use the bit representation
    /// TODO: Also considering why make a function this way... like u pass a mut ref and return it what even...
    pub fn list_union<'a>(list1: &'a mut [bool; 7], list2: &[bool; 7]) -> &'a [bool; 7] {
        list1.iter_mut().zip(list2).for_each(|(a, b)| *a |= b);
        list1
    }
        /// Test to determine if input group_in_question is considered redundant because of the information provided in group_in_reference
    /// Note: this is a test that decides the redundancy of group_in_question and not group_to_reference
    /// YOU WILL NOT NECESSARILY GET THE SAME OUTPUT IF YOU SWAP THEM
    /// This is different from the original
    pub fn is_redundant(&self, group_to_reference: &CompressedGroupConstraint) -> bool {
        if self == group_to_reference {
            return true
        }
        if self.get_card() == group_to_reference.get_card() {
            if self.get_flags() == group_to_reference.get_flags() && 
            // Same Participation list / player flags
            self.count_alive() < group_to_reference.count_alive() {
                // dead_count is skipped because self cannot possibly have more
                // Can use < here instead of <= as == is covered earlier
                // [SUBSET] group in question is redundant as its informational content is fully described by group_to_reference
                return true
            }
            // Don't recall why old version checks remaining counts
            if group_to_reference.part_list_is_subset_of(self) &&
            self.count_alive() == group_to_reference.count_alive() {
                // [SUBSET] group in questions is redundant as its informational content is fully described by group_to_reference
                return true
            }
        }
        false
    }
    /// Get union of 2 mutually exclusive CompressedGroupConstraint
    pub fn mutually_exclusive_union(group_i: CompressedGroupConstraint, group_j: CompressedGroupConstraint) -> CompressedGroupConstraint {
        debug_assert!(group_i.part_list_is_mut_excl(group_j), "Part List of groups must be mutually exclusive! Current groups are i:{:016b}, j: {:016b}", group_i.0, group_j.0);
        debug_assert!(group_i.card() == group_j.card(), "Cannot make a union of groups with different cards! i: {:?}, j: {:?}", group_i.card(), group_j.card());
        let mut new_group: CompressedGroupConstraint = CompressedGroupConstraint(group_i.0 | group_j.0);
        // TODO: Implement a Union
        new_group.set_card(group_i.card());
        let total_dead = group_i.count_dead() + group_j.count_dead();
        let total_alive = group_i.count_alive() + group_j.count_alive();
        new_group.set_dead_count(total_dead);
        new_group.set_alive_count(total_alive);
        new_group.set_total_count(total_alive + total_dead);
        new_group
    }
}

/// A struct that helps in card counting. Stores all information known about cards by a particular player.
pub struct CompressedCollectiveConstraint {
    // TODO: [OPTIMIZE] Consider if can just combine public and joint constraints
    // public_constraints:[[Option<Card>; 2]; 6], // Stores all the dead cards of dead players, None are all behind
    public_constraints: Vec<Vec<Card>>, // Stores all the dead cards of dead players, None are all behind
    // inferred_constraints:[[Option<Card>; 2]; 6], // Stores all the dead cards of dead players 
    inferred_constraints: Vec<Vec<Card>>, // Stores all the dead cards of dead players 
    inferred_pile_constraints: [u8; 5], // Stores number of each card where card as usize is the index
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
        // [COMBINE SJ]
        let public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2); 6];
        // [COMBINE SJ]
        let inferred_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2); 6];
        let inferred_pile_constraints: [u8; 5] = [0; 5];
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
            inferred_pile_constraints,
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
        self.inferred_pile_constraints == [0; 5] &&
        self.group_constraints_amb.is_empty() &&
        self.group_constraints_ass.is_empty() &&
        self.group_constraints_cap.is_empty() &&
        self.group_constraints_duk.is_empty() &&
        self.group_constraints_con.is_empty()
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
    // TODO: [REFACTOR] Consider not exposing inner item
    // pub fn get_jc_hm(&self) -> &HashMap<usize, Vec<Card>> {
    // pub fn joint_constraints(&self) -> &[[Option<Card>; 2]; 6] {
    //     // [COMBINE SJ] DELETE
    //     &self.public_joint_constraints
    // }
    // TODO: [REFACTOR] Consider not exposing inner item
    // pub fn get_pc_hm(&self) -> &HashMap<usize, Card> {
    // pub fn public_constraints(&self) -> &[Option<Card>; 6] {
    //     // [COMBINE SJ] DELETE
    //     &self.public_single_constraints
    // }
    // TODO: [REFACTOR] Consider not exposing inner item
    // pub fn get_gc_vec(&self) -> &Vec<GroupConstraint>{
    // pub fn group_constraints(&self) -> &Vec<CompressedGroupConstraint>{
    //     &self.group_constraints
    // }
    // TODO: [REFACTOR] Consider not exposing inner item
    // pub fn jc_hm(&mut self) -> &mut HashMap<usize, Vec<Card>> {
    // pub fn joint_constraints_mut(&mut self) -> &mut [[Option<Card>; 2]; 6] {
    //     // [COMBINE SJ] DELETE
    //     &mut self.public_joint_constraints
    // }
    // TODO: [REFACTOR] Consider not exposing inner item
    // pub fn pc_hm(&mut self) -> &mut HashMap<usize, Card> {
    // pub fn public_constraints_mut(&mut self) -> &mut [Option<Card>; 6] {
    //     // [COMBINE SJ] DELETE
    //     &mut self.public_single_constraints
    // }
    // TODO: [REFACTOR] Consider not exposing inner item
    // pub fn gc_vec(&mut self) -> &mut Vec<GroupConstraint>{
    // pub fn group_constraints_mut(&mut self) -> &mut Vec<CompressedGroupConstraint>{
    //     &mut self.group_constraints
    // }
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
        let mut output: u8 = 0;
        // [COMBINE SJ] just do a for loop
        // TODO: Just count the len()
        output += self.public_constraints[player_id].len() as u8;
        output += self.inferred_constraints[player_id].len() as u8;
        output
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
        self.dead_card_count[card as usize] += 1;
        // [COMBINE SJ]
        // if self.public_constraints[player_id][0].is_none() {
        //     self.public_constraints[player_id][0] = Some(card)
        // } else {
        //     self.public_constraints[player_id][1] = Some(card)
        // }
        self.public_constraints[player_id].push(card);
    }
    /// Adds to tracked inferred constraints
    pub fn add_inferred_player_constraint(&mut self, player_id: usize, card: Card) {
        debug_assert!(player_id < 6, "Use proper player_id thats not pile");
        debug_assert!(self.inferred_constraints[player_id].len() < 2, "Adding inferred knowledge to fully known player!");
        self.inferred_card_count[card as usize] += 1;
        // [COMBINE SJ]
        // if self.inferred_constraints[player_id][0].is_none() { // If player has only 1 inferred card, it would be at index 0
        //     self.inferred_constraints[player_id][0] = Some(card);
        // } else {
        //     self.inferred_constraints[player_id][1] = Some(card);
        // }
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
    #[inline]
    /// Increments card counter for inferred pile constraints
    pub fn add_inferred_pile_constraint(&mut self, card: Card) {
        self.inferred_pile_constraints[card as usize] += 1;
        self.inferred_card_count[card as usize] += 1;
    }
    /// Removes a specific card from the inferred pile constraints if it exists
    pub fn subtract_inferred_pile_constraint(&mut self, card: Card) {
        if self.inferred_pile_constraints[card as usize] > 0 {
           self.inferred_pile_constraints[card as usize] -= 1; 
           self.inferred_card_count[card as usize] -= 1;
        }
    }
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
    /// Empties inferred_pile_constraints
    /// NOTE:
    ///     - Debugging only
    pub fn empty_inferred_pile_constraint(&mut self) {
        // Try not to use this as it doesnt adjust the inferred card count
        self.inferred_pile_constraints = [0; 5];
    }
    #[inline]
    /// Return true if inferred player constraint contains a particular card 
    pub fn inferred_player_constraint_contains(&self, player_id: usize, card: Card) -> bool {
        // [COMBINE SJ] rewrite
        self.inferred_constraints[player_id].contains(&card)
    }
    /// Return true if inferred pile constraint contains a particular card 
    pub fn inferred_pile_constraint_contains(&self, card: Card) -> bool {
        self.inferred_pile_constraints[card as usize] != 0
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
                    if output[card_num] < group.count_alive() {
                        output[card_num] = group.count_alive();
                    }
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
        self.add_dead_player_constraint(player_id, card);
        self.reveal_group_adjustment(player_id);
        // TODO: ADD COMPLEMENT PRUNE is probably useful here since its not done in group_redundant_prune()
        // TODO: [THOT] Group constraints being a subset of info in inferred constraints mean it can be pruned too
        //      - like if inferred info reflects the same thing as group constraint
        // QUESTION: How does inferred constraints help in determining if group is redundant? Should this be pruned above?
        // TODO: Needs to do dead player prune
        self.group_redundant_prune();
        self.generate_inferred_constraints();
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
        if !self.inferred_player_constraint_contains(player_id, card) {
            // Adds information to inferred constraint if it isn't already there
            self.add_inferred_player_constraint(player_id, card);
        }
        self.reveal_group_adjustment(player_id);

        // TODO: ADD COMPLEMENT PRUNE is probably useful here since its not done in group_redundant_prune()
        // TODO: [THOT] Group constraints being a subset of info in inferred constraints mean it can be pruned too
        //      - like if inferred info reflects the same thing as group constraint
        // QUESTION: How does inferred constraints help in determining if group is redundant? Should this be pruned above?
        self.group_redundant_prune();
        self.generate_inferred_constraints();
        // [THOT] It feels like over here when you reveal something, you lead to information discovery! 
        // [THOT] So one might be able to learn information about the hands of other players?
    }
    /// Updates groups affected by revealing of information in death/reveal
    /// SPECIFICS:
    /// - See documentation in reveal and death
    /// NOTE:
    /// - Assumes there may be groups that became redundant after information is revealed
    /// - Only modifies and removes groups that are affected or have become redundant after death/reveal
    /// - May leave groups that are redundant when compared with other groups
    pub fn reveal_group_adjustment(&mut self, player_id: usize) {
        let player_alive_card_count: [u8; 5] = self.player_alive_card_counts(player_id);
        let player_dead_card_count: [u8; 5] = self.player_dead_card_counts(player_id);
        let player_cards_known = self.player_cards_known(player_id);
        let mut i: usize = 0;
        for (card_num, group_constraints) in [&mut self.group_constraints_amb, &mut self.group_constraints_ass, &mut self.group_constraints_cap, &mut self.group_constraints_duk, &mut self.group_constraints_con].iter_mut().enumerate() {
            while i < group_constraints.len() {
                let group: &mut CompressedGroupConstraint = &mut group_constraints[i];
                // Update only groups affected by the revealed information => i.e. those with player_id flag as true
                if group.get_player_flag(player_id)  {
                    if group.count_alive() <= player_alive_card_count[card_num] {
                        // [PLAYER ONLY PRUNE] knowing the player had the card now makes this group obsolete | all possible alive cards in the group are with the player
                        // No need to modify this as the information from the player's pile swap gets added at the end
                        group_constraints.swap_remove(i);
                        continue;
                    } else if player_cards_known == 2 {
                        // if we know both of a player's cards including the revealed card (player has at least 1 alive cos reveal)
                        group.set_player_flag(player_id, false);
                        if group.none_in() {
                            group_constraints.swap_remove(i);
                            continue;
                        }
                        group.count_alive_subtract(player_alive_card_count[card_num]);
                        group.count_dead_subtract(player_dead_card_count[card_num]);
                    }
                }
                i += 1;
            }
        }
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
    // e.g. reveal_redraw => that called mix() then dilutes information
    // e.g. ambassador => that calls mix() then dilutes information
    // TODO: [MOVE DOCUMENTATION] to docstring 
    /// Mixes cards.
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
        let player_alive_card_count: [u8; 5] = self.player_alive_card_counts(player_id);
        let mut i: usize = 0;
        for (card_num, group_constraints) in [&mut self.group_constraints_amb, &mut self.group_constraints_ass, &mut self.group_constraints_cap, &mut self.group_constraints_duk, &mut self.group_constraints_con].iter_mut().enumerate() {
            while i < group_constraints.len() {
                let group = &mut group_constraints[i];
                // consider 2 dimensions, player_flag and pile_flag 0 1, 1 0, 1 1? no 0 0
                if !group.get_player_flag(player_id) {
                    if group.get_player_flag(6) {
                        // Here player is 0 and pile is 1
                        // We add player information that it is originally missing
                        group.set_player_flag(player_id, true);
                        // [COMBINE SJ]
                        if let Some(dead_card) = self.public_constraints[player_id].get(0) {
                            if card_num == *dead_card as usize {
                                group.add_dead_count(1);
                            }
                        }
                        // TODO: put debug_assert somewhere sensible
                        // debug_assert!(single_count + joint_count + self.dead_card_count()[group_card as usize] < 3, "???");
                        group.count_alive_add(player_alive_card_count[card_num]);
                    }
                } else {
                    if !group.get_player_flag(6) {
                        // Here player is 1 and pile is 0
                        // We add pile information that it is originally missing
                        group.set_player_flag(6, true);
                        group.count_alive_add(self.inferred_pile_constraints[card_num]); 
                    } else {
                        // Here player is 1 and pile is 1, we do a simple check
                        // If somehow you have learnt of more inferred information, than prune the group
                        if player_alive_card_count[card_num] > group.count_alive() {
                            group_constraints.swap_remove(i);
                            continue;
                        }
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
    /// RevealRedraw dilution of inferred information
    /// Adjust inferred constraints
    /// NOTE:
    /// - Information is "diluted" or "dissipated", since there is a reduction in absolutely known information about a particular player's state
    /// - This should ideally still reflect all possibly inferred information!
    /// - May leave redundant information in the collective constraint
    /// Inferred knowledge cases [REVEALREDRAW] [ALL CARDS WITH PILE + PLAYER]
    /// These arent just cases for how to represent the new group constraint
    /// These also represent what we can infer, if its pile >= 1 we have a 1 inferred card of the pile
    /// player (dead, alive) = (A, A) Pile (A, X, X) => Pile has >= 1 A
    /// player (dead, alive) = (A, X) Pile (A, A, X) => Pile has >= 1 A
    /// player (dead, alive) = (A, X) Pile (A, X, X) => Pile has >= 0 A (No inferred info for pile)
    /// player (dead, alive) = (A, X) Pile (X, X, X) => Pile has >= 0 A (No inferred info for pile)
    /// player (dead, alive) = (!A, X) Pile (A, A, A) => Pile has >= 2 A
    /// player (dead, alive) = (!A, X) Pile (A, A, X) => Pile has >= 1 A
    /// player (dead, alive) = (!A, X) Pile (A, X, X) => Pile has >= 0 A (No inferred info for pile)
    /// player (dead, alive) = (!A, A) Pile (A, A, X) => Pile has >= 2 A
    /// player (dead, alive) = (!A, A) Pile (A, X, X) => Pile has >= 1 A
    /// player (dead, alive) = (!A, A) Pile (X, X, X) => Pile has >= 0 A (No inferred info for pile)
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
    pub fn dilution_reveal(&mut self, player_id: usize, card: Card) {
        // [DILUTING INFERRED INFORMATION] Mixing causes the inferred constraints to be dissipated from knowing a particular player has a card
        //                                  to knowing some groups of players have a card
        // Here we Manage the dissipation of inferred information by:
        // - Properly subtracting the appropriate amount from inferred pile constraint
        // - Adding the information into the group constraints => on how the known cards have "spread" from player or pile or BOTH (player union pile) 
        let mut card_counts: [u8; 5] = self.inferred_pile_constraints.clone();
        card_counts[card as usize] += 1;
        // only subtract 1 card here as only 1 is revealed and moved out of player's hand 
        // TODO: [CHANGE] COLLORARY 1b, Adding of group constraints should be for all inferred cards in the player union pile
        // COLLORARY 1b: If player reveals some card A, player inferred A - 1, pile inferred A remains constant,for all cards !A pile number of inferred !A - 1
        for inferred_card in [Card::Ambassador, Card::Assassin, Card::Captain, Card::Duke, Card::Contessa] {
            if inferred_card != card {
                // Dissipating Information from pile
                // pile number inferred - 1
                self.subtract_inferred_pile_constraint(card);
            }
            if card_counts[inferred_card as usize] > 0 {
                // Adding Dissipated information to groups appropriately
                // [COMBINE SJ]
                let dead_count = self.player_dead_card_count(player_id, inferred_card);
                // TODO: Add method to add groups only if it is not already inside
                let group = CompressedGroupConstraint::new(player_id, inferred_card, dead_count, card_counts[inferred_card as usize]);
                self.add_group_constraint(group);
            }
        }
        // Get pile counts
        // Get reveal card and add it to count
        // Add groups for all those counts with dead_card if reqruied
        // TODO: [LIKE BELOW] Groups need to be added for all information in pile
        // Dissipating information from player
        // player inferred A - 1
        self.subtract_inferred_player_constraints(player_id, card);
    }
    /// Ambassador Dilution of inferred knowledge
    /// Adjust inferred knowledge
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
    /// CONCLUSION: Inferred pile constraint for some card A will be total circulating A - no_alive cards?
    /// TODO: might need to consider the group constraints? if they add to the total circulating?
    /// TODO: [THEORY CHECK]
    /// CASE: (alive, alive) (X, X) Pile: (A, X, X) but we know the union of both have 3 As so total circulating has to include this!
    pub fn dilution_ambassador(&mut self, player_id: usize) {
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
            self.inferred_pile_constraints[inferred_card as usize] = total_circulating_card_counts[inferred_card as usize] - player_lives;
            // Add group constraints
            let dead_cards_count = self.player_dead_card_count(player_id, inferred_card);
            // TODO: Add method to add groups only if it is not already inside
            let group = CompressedGroupConstraint::new(player_id, inferred_card, dead_cards_count, total_circulating_card_counts[inferred_card as usize]);
            self.add_group_constraint(group);
        }
    }
    /// Function to call for move RevealRedraw
    pub fn reveal_redraw(&mut self, player_id: usize, card: Card) {
        // Abit dumb to seperate it like this, but if not it gets abit messy and I have more branchs :/
        self.reveal(player_id, card);
        self.mix(player_id);
        self.dilution_reveal(player_id, card);
        self.group_redundant_prune();
        // Add the stuff here
    }
    /// Function to call for move Ambassador, without considering private information seen by the player who used Ambassador
    pub fn ambassador_public(&mut self, player_id: usize) {
        self.mix(player_id);
        self.dilution_ambassador(player_id);
        self.group_redundant_prune();
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
    /// NOTE:
    /// - Assumes groups where all of a particular card is dead will not exist before this as they are implicitly pruned in
    ///   reveal_group_adjustment 
    pub fn group_redundant_prune(&mut self) {
        let mut i: usize = 0;
        let mut j: usize = 0;
        for group_constraints in [&mut self.group_constraints_amb, &mut self.group_constraints_ass, &mut self.group_constraints_cap, &mut self.group_constraints_duk, &mut self.group_constraints_con].iter_mut() {
            'outer:  while i < group_constraints.len() - 1 {
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
                    // If group i is made redundant by group j
                    if group_constraints[j].part_list_is_subset_of(&group_constraints[i]) &&
                    group_constraints[i].count_alive() <= group_constraints[j].count_alive() {
                        group_constraints.swap_remove(i);
                        continue 'outer;
                    }
                    // If group j is made redundant by group i
                    if group_constraints[i].part_list_is_subset_of(&group_constraints[j]) &&
                    group_constraints[j].count_alive() <= group_constraints[i].count_alive() {
                        group_constraints.swap_remove(j);
                        continue 'inner;
                    }
                    j += 1;
                }
                i += 1;
            }
        }
    }
    // TODO: [ALT] Make alternate version of this that adds with 2n checks for when you use it with a particular group added in mind.
    // TODO: Theory Check & Document
    // Or just use reveal LMAO, bacause thats what reveal does?
    /// Im assuming self.group_constraints is not internally redundant
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
        let mut bool_continue = self.add_subset_groups() || self.add_mutually_exclusive_unions();
        // Then runs one if the other is still true
        // idea here is that subset groups adds all the smaller groups, mut excl adds the larger groups
        //      - we get smaller groups from larger groups, so if no new larger groups added, no new smaller groups can be inferred
        //      - we get larger groups from smaller groups, so if no new smaller groups added, no new larger groups can be inferred
        //      - Technically the functions also add inferred groups, so if no inferred groups added, no new information added, no need to run either  
        while bool_continue {
            // Runs a
            bool_continue = self.add_subset_groups();
            if bool_continue {
                bool_continue = self.add_mutually_exclusive_unions();
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
            if group.part_list_is_subset_of(&vec[i]) && 
            group.count_alive() <= vec[i].count_alive() {
                // group is redundant
                return
            }
            if vec[i].part_list_is_subset_of(&group) &&
            vec[i].count_alive() <= group.count_alive() {
                vec.swap_remove(i);
                continue;
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
    fn non_redundant_push_tracked(vec: &mut Vec<CompressedGroupConstraint>, group: CompressedGroupConstraint) -> bool {
        let mut i: usize = 0;
        while i < vec.len() {
            if group.part_list_is_subset_of(&vec[i]) && 
            group.count_alive() <= vec[i].count_alive() {
                // group is redundant
                return false
            }
            if vec[i].part_list_is_subset_of(&group) &&
            vec[i].count_alive() <= group.count_alive() {
                vec.swap_remove(i);
                continue;
            }
            i += 1;
        }
        vec.push(group);
        true
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
                if flags_count > 1 {
                    // Creation of new groups (may have only 1 player_flag)
                    // group => [1 1 0 1 0 0 1], add [0 1 0 1 0 0 1], [1 0 0 1 0 0 1], [1 1 0 0 0 0 1], []
                    for (player, player_flag) in part_list.iter().enumerate() {
                        if *player_flag {
                            let player_lives: u8 = 2 - self.public_constraints[player].len() as u8;
                            let player_inferred_cards: u8 = self.inferred_constraints[player].iter().map(|c| (*c as usize == card_num) as u8).sum();
                            if group.count_alive() > player_lives - player_inferred_cards {
                                // Cards contained is n - player_lives + player_inferred_cards for new group
                                let mut new_group: CompressedGroupConstraint = *group;
                                new_group.set_player_flag(player, false);
                                for card in self.public_constraints[player].iter() {
                                    if *card as usize == card_num {
                                        new_group.sub_dead_count(1);
                                    }
                                }
                                for card in self.inferred_constraints[player].iter() {
                                    if *card as usize == card_num {
                                        new_group.set_alive_count(group.count_alive() - player_lives + player_inferred_cards);
                                    }
                                }
                                new_group.set_total_count(new_group.count_alive() + new_group.count_dead());
                                // Required to meet assumptions of recursive function
                                if flags_count > 2 {
                                    CompressedCollectiveConstraint::non_redundant_push(&mut new_groups[card_num], new_group);
                                } else {
                                    // only 1 flag after removal, and so should be added to inferred constraints later
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
            let card = single_flag_group.card();
            match alive_count {
                2 => {
                    // Both cards known
                    if self.inferred_constraints[player_id] != vec![card; 2] {
                        self.inferred_constraints[player_id].clear();
                        self.inferred_constraints[player_id].push(card);
                        self.inferred_constraints[player_id].push(card);
                        card_changes[card as usize].push(player_id);
                        // TODO: Needs to prune the groups too... some batch function that prunes groups based on changes?
                    }
                    // TODO: Adjust counts properly too... or remove inferred_counts
                },
                1 => {
                    // One card known
                    if !self.inferred_constraints[player_id].contains(&card) {
                        self.inferred_constraints[player_id].push(card);
                        card_changes[card as usize].push(player_id);
                        // TODO: Needs to prune the groups too...
                        // TODO: Adjust counts properly too... or remove inferred_counts
                    }
                },
                _ => {
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
            while i < self_groups[card_num].len() {
                for &player_id in changes {
                    // If player flag is true and number of cards player now has
                    if self_groups[card_num][i].get_player_flag(player_id) {
                        let count_alive = self.inferred_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                        if count_alive >= self_groups[card_num][i].count_alive() {
                            self_groups[card_num].swap_remove(i);
                            continue;
                        }
                    }
                }
                i += 1;
            }
        }
        // recurse, new_groups should not be internally redundant, self.group_constraints should not be internally redundant
        if card_changes.iter().any(|v| !v.is_empty()) {
            return self.add_subset_groups_recurse(new_groups);
        } 
        false
    }
    /// Recursively Adds groups from the inferred subset of another gorup
    /// - e.g. [1 1 0 0 0 0 0] 2 Duke and player 0 has 1 life, player 2 has 2 lives. We know player 2 has at least 1 Duke. As player 1 can have at most 1 Duke.
    /// - e.g. [1 0 0 0 0 0 1] has 3 alive Dukes. 
    ///     - Player 0 has an inferred Duke. Therefore, pile must have at least 3 - 1 - 1 = 1 Dukes.
    ///     - Player 0 has an inferred Captain. Therefore, pile must have at least 3 - 0 - 1 = 2 Dukes.
    ///     - or 3 - 2 lives + 1 Number of Dukes
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
                            let player_lives: u8 = 2 - self.public_constraints[player].len() as u8;
                            let player_inferred_cards: u8 = self.inferred_constraints[player].iter().map(|c| (*c as usize == card_num) as u8).sum();
                            if group.count_alive() > player_lives - player_inferred_cards {
                                // Cards contained is n - player_lives + player_inferred_cards for new group
                                let mut new_group: CompressedGroupConstraint = *group;
                                new_group.set_player_flag(player, false);
                                for card in self.public_constraints[player].iter() {
                                    if *card as usize == card_num {
                                        new_group.sub_dead_count(1);
                                    }
                                }
                                for card in self.inferred_constraints[player].iter() {
                                    if *card as usize == card_num {
                                        new_group.set_alive_count(group.count_alive() - player_lives + player_inferred_cards);
                                    }
                                }
                                new_group.set_total_count(new_group.count_alive() + new_group.count_dead());
                                // Required to meet assumptions of recursive function
                                if flags_count > 2 {
                                    CompressedCollectiveConstraint::non_redundant_push(&mut new_groups[card_num], new_group);
                                } else {
                                    // only 1 flag after removal, and so should be added to inferred constraints later
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
                bool_changes = Self::non_redundant_push_tracked(&mut self_groups[card_num], group) || bool_changes;
            }
        }
        let mut card_changes: Vec<Vec<usize>> = vec![Vec::with_capacity(2); 5]; // Store (player_id, bool_counts => false: 1, true: 2)
        // Add new_inferred_constraints
        while let Some(single_flag_group) = new_inferred_constraints.pop() {
            let alive_count = single_flag_group.count_alive();
            // Something is wrong if this panics, all groups should have a single flag, not no flags
            let player_id = single_flag_group.get_set_players().iter().position(|b| *b).unwrap();
            let card = single_flag_group.card();
            match alive_count {
                2 => {
                    // Both cards known
                    if self.inferred_constraints[player_id] != vec![card; 2] {
                        self.inferred_constraints[player_id].clear();
                        self.inferred_constraints[player_id].push(card);
                        self.inferred_constraints[player_id].push(card);
                        card_changes[card as usize].push(player_id);
                        // TODO: Needs to prune the groups too... some batch function that prunes groups based on changes?
                    }
                    // TODO: Adjust counts properly too... or remove inferred_counts
                },
                1 => {
                    // One card known
                    if !self.inferred_constraints[player_id].contains(&card) {
                        self.inferred_constraints[player_id].push(card);
                        card_changes[card as usize].push(player_id);
                        // TODO: Needs to prune the groups too...
                        // TODO: Adjust counts properly too... or remove inferred_counts
                    }
                },
                _ => {
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
            while i < self_groups[card_num].len() {
                for &player_id in changes {
                    // If player flag is true and number of cards player now has
                    if self_groups[card_num][i].get_player_flag(player_id) {
                        let count_alive = self.inferred_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                        if count_alive >= self_groups[card_num][i].count_alive() {
                            self_groups[card_num].swap_remove(i);
                            continue;
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
        if reference_group_constraints.iter().all(|v| v.is_empty()) {
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
                        let new_group: CompressedGroupConstraint = CompressedGroupConstraint::mutually_exclusive_union(*reference_group_i, *self_group);
                        // TODO: Change new_group to a union
                        Self::non_redundant_push(&mut new_group_constraints[card_num], new_group);
                    }
                }
                // Compare reference group with reference group
                for reference_group_j in reference_group_constraints[card_num].iter() {
                    if reference_group_i.part_list_is_mut_excl(*reference_group_j) {
                        // Bitwise Union is a fast way to get their
                        let new_group: CompressedGroupConstraint = CompressedGroupConstraint::mutually_exclusive_union(*reference_group_i, *reference_group_j);
                        Self::non_redundant_push(&mut new_group_constraints[card_num], new_group);
                    }
                }
                // Compare reference group with inferred groups
                for (player_id, &player_flag) in reference_group_i.get_set_players().iter().enumerate() {
                    if !player_flag {
                        let same_alive_card_count = self.inferred_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                        if same_alive_card_count > 0 {
                            let same_dead_card_count: u8 = self.public_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                            let mut new_group: CompressedGroupConstraint = *reference_group_i;
                            new_group.set_player_flag(player_id, true);
                            new_group.add_alive_count(same_alive_card_count);
                            new_group.add_dead_count(same_dead_card_count);
                            new_group.set_total_count(new_group.count_alive() + new_group.count_dead());
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
                bool_changes = Self::non_redundant_push_tracked(&mut self_groups[card_num], group) || bool_changes;
            }
        }
        // Self group is not internally redundant as all groups added were through non_redundant_push
        // New_group_constraints is not internally redundant, as it has been added through non_redundant_push
        // This satisfies the assumptions for recursion
        // Recurse
        return self.add_mutually_exclusive_unions_recurse(new_group_constraints) || bool_changes;
    }
    // TODO: Fill function and assumptions
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
        if self.group_constraints().iter().all(|v| v.is_empty()) {
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
                        let new_group: CompressedGroupConstraint = CompressedGroupConstraint::mutually_exclusive_union(*reference_group_i, *reference_group_j);
                        Self::non_redundant_push(&mut new_group_constraints[card_num], new_group);
                    }
                }
                // Compare reference group with inferred groups
                for (player_id, &player_flag) in reference_group_i.get_set_players().iter().enumerate() {
                    if !player_flag {
                        let same_alive_card_count = self.inferred_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                        if same_alive_card_count > 0 {
                            let same_dead_card_count: u8 = self.public_constraints[player_id].iter().filter(|c| **c as usize == card_num).count() as u8;
                            let mut new_group: CompressedGroupConstraint = *reference_group_i;
                            new_group.set_player_flag(player_id, true);
                            new_group.add_alive_count(same_alive_card_count);
                            new_group.add_dead_count(same_dead_card_count);
                            new_group.set_total_count(new_group.count_alive() + new_group.count_dead());
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
    pub fn add_subset_and_mutually_exclusive_unions(&mut self) {
        todo!()
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
}